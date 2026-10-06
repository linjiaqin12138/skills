//! 健康门：轮询 miniduckd 的 robot.health 直到拿到裁决或预算耗尽。
//!
//! 对照原版 engine.rs:69 `HEALTH_POLL_INTERVAL = 500ms` 与
//! engine.rs:2276-2305 `robot_verdict`（"The gate polls, so it will see
//! the transition"；timeout 即失败，"unproven is not healthy"）。
//! 原版五裁决里的 Degraded 我们没有（D41，mini 健康判定是布尔）。

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use crate::{JSONRPC, Request, ServerMessage};

/// 原版 engine.rs:69。
pub const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// 单次连接尝试的硬超时（拍脑袋——代理假设：原版对 hung binary 有
/// SELF_TEST_TIMEOUT 之类的保护，这里一次 RPC 尝试给 1s 足够）。
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(1);

pub struct GateReport {
    pub healthy: bool,
    /// 判病原因 / 最后一次连接错误，写进 log 行。
    pub detail: String,
}

/// 在 budget 内每 500ms 问一次 robot.health；healthy 立即放行，
/// 预算耗尽 = 失败（回滚的依据）。
pub async fn poll(sock_path: &str, budget: Duration) -> GateReport {
    let deadline = tokio::time::Instant::now() + budget;
    loop {
        let detail = match tokio::time::timeout(ATTEMPT_TIMEOUT, ask_once(sock_path)).await {
            Ok(Ok(report)) if report.healthy => return report,
            Ok(Ok(report)) => report.detail,
            Ok(Err(e)) => e,
            Err(_) => format!("health RPC timed out after {}ms", ATTEMPT_TIMEOUT.as_millis()),
        };
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return GateReport { healthy: false, detail };
        }
        tokio::time::sleep(HEALTH_POLL_INTERVAL.min(deadline - now)).await;
    }
}

/// 问一次：hello + robot.health。hello 是协议规定的入场第一件事
/// （lib.rs 注释），健康门不特殊。
async fn ask_once(sock_path: &str) -> Result<GateReport, String> {
    let stream = UnixStream::connect(sock_path)
        .await
        .map_err(|e| format!("connect {sock_path}: {e}"))?;
    let mut framed = Framed::new(stream, LinesCodec::new());

    for (id, method) in [(1u64, "hello"), (2, "robot.health")] {
        let req = Request { jsonrpc: JSONRPC.into(), id: Some(id), method: method.into(), params: Value::Null };
        framed
            .send(serde_json::to_string(&req).expect("Request is serializable"))
            .await
            .map_err(|e| format!("send {method}: {e}"))?;
        let line = framed
            .next()
            .await
            .ok_or_else(|| format!("daemon closed during {method}"))?
            .map_err(|e| format!("read {method}: {e}"))?;
        if method == "robot.health" {
            let msg: ServerMessage =
                serde_json::from_str(&line).map_err(|e| format!("parse health reply: {e}"))?;
            return match msg {
                ServerMessage::Response { result: Some(result), .. } => {
                    let healthy = result
                        .get("healthy")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let detail = if healthy {
                        "healthy".to_owned()
                    } else {
                        format!(
                            "unhealthy: {}",
                            result.get("reason").cloned().unwrap_or(Value::Null)
                        )
                    };
                    Ok(GateReport { healthy, detail })
                }
                _ => Err(format!("unexpected health reply: {line}")),
            };
        }
    }
    unreachable!("robot.health arm always returns")
}
