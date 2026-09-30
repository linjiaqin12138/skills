//! IPC 层：mini-updaterd 自己的 Unix socket，复用 lib.rs 的 NDJSON
//! JSON-RPC 帧。方法面：update.check / update.apply / update.rollback /
//! update.status / update.log（+hello）。mutating 两个走 SO_PEERCRED-lite
//! 门控 + 单飞锁。

use std::sync::{Arc, Mutex};

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed, LinesCodec};

use super::engine::{Engine, Phase};
use super::journal;
use super::lock::{self, AcquireError};
use super::paths::UpdaterPaths;
use super::peer;
use super::store;
use crate::{API_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, Request, ServerMessage};

const INVALID_PARAMS: i32 = -32602;

pub struct Server {
    pub paths: UpdaterPaths,
    pub engine: Arc<tokio::sync::Mutex<Engine>>,
    pub phase: Arc<Mutex<Phase>>,
    /// socket 文件的 owner uid（bind 后由 metadata 读出），peer 门控基准。
    pub sock_owner_uid: u32,
}

pub async fn serve(listener: UnixListener, server: Arc<Server>) -> std::io::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let server = server.clone();
        tokio::spawn(async move {
            serve_conn(stream, server).await;
        });
    }
}

async fn serve_conn(stream: UnixStream, server: Arc<Server>) {
    let mut framed = Framed::new(stream, LinesCodec::new());
    loop {
        let line = match framed.next().await {
            Some(Ok(line)) => line,
            _ => return,
        };
        let req: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let msg = ServerMessage::err(0, PARSE_ERROR, e.to_string());
                if framed.send(serde_json::to_string(&msg).unwrap()).await.is_err() {
                    return;
                }
                continue;
            }
        };
        let resp = dispatch(&server, &framed, &req).await;
        if framed.send(serde_json::to_string(&resp).unwrap()).await.is_err() {
            return;
        }
    }
}

async fn dispatch(server: &Arc<Server>, framed: &Framed<UnixStream, LinesCodec>, req: &Request) -> ServerMessage {
    match req.method.as_str() {
        "hello" => ServerMessage::ok(
            req.id,
            json!({ "service": "mini-updaterd", "api_version": API_VERSION }),
        ),
        "update.check" => {
            let mut available: Vec<String> = Vec::new();
            if let Ok(rd) = std::fs::read_dir(server.paths.source()) {
                for entry in rd.flatten() {
                    let name = entry.file_name();
                    let Some(name) = name.to_str() else { continue };
                    if let Some(ver) = name.strip_suffix(".manifest.json") {
                        // 只列 manifest 能解析的——坏 manifest 的 release
                        // apply 也会被拒，列出来是误导。
                        if super::manifest::Manifest::load(&entry.path()).is_ok() {
                            available.push(ver.to_owned());
                        }
                    }
                }
            }
            available.sort();
            ServerMessage::ok(
                req.id,
                json!({
                    "available": available,
                    "current": store::current_version(&server.paths),
                }),
            )
        }
        "update.status" => {
            let phase = *server.phase.lock().expect("phase mutex poisoned");
            let current = store::current_version(&server.paths);
            let previous = current
                .as_deref()
                .and_then(|c| journal::previous_for(&server.paths, c));
            // 只读路径不锁 engine：apply 全程持锁，status 必须仍能读。
            let pending = super::engine::load_pending(&server.paths);
            ServerMessage::ok(
                req.id,
                json!({
                    "phase": phase,
                    "current": current,
                    "previous": previous,
                    "pending": pending,
                }),
            )
        }
        "update.log" => {
            let n = req.params.get("n").and_then(Value::as_u64).unwrap_or(10) as usize;
            let all = journal::entries(&server.paths);
            let tail: Vec<_> = all.iter().skip(all.len().saturating_sub(n)).collect();
            ServerMessage::ok(req.id, json!({ "entries": tail }))
        }
        "update.apply" => {
            let Some(version) = req.params.get("version").and_then(Value::as_str) else {
                return ServerMessage::err(
                    req.id,
                    INVALID_PARAMS,
                    "update.apply wants {version: \"x.y.z\"}",
                );
            };
            match gate_peer(framed, server.sock_owner_uid) {
                Ok(uid) => match try_lock(&server.paths, req.id) {
                    Ok(_lock) => {
                        let mut engine = server.engine.lock().await;
                        pack(req.id, engine.apply(version, Some(uid)).await)
                    }
                    Err(resp) => resp,
                },
                Err(resp) => resp.with_id(req.id),
            }
        }
        "update.rollback" => match gate_peer(framed, server.sock_owner_uid) {
            Ok(uid) => match try_lock(&server.paths, req.id) {
                Ok(_lock) => {
                    let mut engine = server.engine.lock().await;
                    pack(req.id, engine.rollback(Some(uid)).await)
                }
                Err(resp) => resp,
            },
            Err(resp) => resp.with_id(req.id),
        },
        other => ServerMessage::err(req.id, METHOD_NOT_FOUND, format!("method not found: {other}")),
    }
}

fn pack(id: u64, result: super::engine::OpResult) -> ServerMessage {
    match result {
        Ok(result) => ServerMessage::ok(id, result),
        Err((code, message)) => ServerMessage::err(id, code, message),
    }
}

/// SO_PEERCRED-lite 门控（原版 ipc.rs:78-97/531-535 的语义，缩为
/// uid==0 || owner，D40）。peer_cred 失败同样拒绝——unproven 不是 allowed。
fn gate_peer(
    framed: &Framed<UnixStream, LinesCodec>,
    owner_uid: u32,
) -> Result<u32, DeniedResponse> {
    let cred = framed
        .get_ref()
        .peer_cred()
        .map_err(|e| DeniedResponse(format!("PermissionDenied: peer_cred failed: {e}")))?;
    if peer::peer_allowed(cred.uid(), owner_uid) {
        Ok(cred.uid())
    } else {
        Err(DeniedResponse(format!(
            "PermissionDenied: uid {} may not mutate (allowed: uid 0 or socket owner {owner_uid})",
            cred.uid()
        )))
    }
}

struct DeniedResponse(String);

impl DeniedResponse {
    fn with_id(self, id: u64) -> ServerMessage {
        ServerMessage::err(id, super::ERR_PERMISSION, self.0)
    }
}

/// 单飞锁：Busy / 锁 IO 错误直接变成 RPC 错误；拿到锁后由调用方持有
/// 到操作结束（updaterd 被杀时内核代放，见 lock.rs 的取舍注释）。
fn try_lock(paths: &UpdaterPaths, id: u64) -> Result<lock::UpdateLock, ServerMessage> {
    match lock::try_acquire(&paths.lock()) {
        Ok(l) => Ok(l),
        Err(AcquireError::Busy) => Err(ServerMessage::err(
            id,
            super::ERR_BUSY,
            "Busy: another update is in progress",
        )),
        Err(AcquireError::Io(e)) => Err(ServerMessage::err(
            id,
            super::ERR_FAILED,
            format!("acquire update.lock: {e}"),
        )),
    }
}
