//! updaterd 视角的 robotd 客户端：safeToRestart / modelApi。

use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::{JSONRPC, Request, SafeToRestartResult, ServerMessage};

pub const ROBOT_RPC_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeToRestart {
    Yes,
    No(String),
    Incompatible(String),
    Unreachable,
}

impl SafeToRestart {
    pub fn permits_restart(&self) -> bool {
        !matches!(self, SafeToRestart::No(_) | SafeToRestart::Incompatible(_))
    }
}

async fn ask(path: &str, method: &str, to: Duration) -> Option<Value> {
    let exchange = async {
        let stream = UnixStream::connect(path).await.ok()?;
        let (read_half, mut write_half) = stream.into_split();

        let req = Request {
            jsonrpc: JSONRPC.into(),
            id: Some(1),
            method: method.into(),
            params: Value::Null,
        };
        let mut line = serde_json::to_vec(&req).ok()?;
        line.push(b'\n');
        write_half.write_all(&line).await.ok()?;
        write_half.flush().await.ok()?;

        let mut reply = String::new();
        tokio::io::BufReader::new(read_half)
            .read_line(&mut reply)
            .await
            .ok()?;

        let msg: ServerMessage = serde_json::from_str(reply.trim()).ok()?;
        match msg {
            ServerMessage::Response { result: Some(result), .. } => Some(result),
            _ => None,
        }
    };

    match timeout(to, exchange).await {
        Ok(result) => result,
        Err(_) => None,
    }
}

pub async fn safe_to_restart(path: &str, to: Duration) -> SafeToRestart {
    let Some(result) = ask(path, "robot.safeToRestart", to).await else {
        return SafeToRestart::Unreachable;
    };
    match serde_json::from_value::<SafeToRestartResult>(result) {
        Ok(answer) if answer.safe => SafeToRestart::Yes,
        Ok(answer) => SafeToRestart::No(
            answer.reason.unwrap_or_else(|| "robot reports it is not safe to restart".into()),
        ),
        Err(e) => SafeToRestart::Incompatible(e.to_string()),
    }
}

pub async fn model_api(path: &str, to: Duration) -> Option<u32> {
    let result = ask(path, "robot.modelApi", to).await?;
    serde_json::from_value::<crate::ModelApiResult>(result)
        .ok()
        .map(|answer| answer.model_api)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::net::UnixListener;
    use tokio::task::JoinHandle;

    fn tmp_sock() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("miniduck-robot-test-{n}.sock"))
    }

    fn canned_server(result: Value) -> (std::path::PathBuf, JoinHandle<()>) {
        let path = tmp_sock();
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).expect("bind");
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (read_half, mut write_half) = stream.into_split();
            let mut req = String::new();
            tokio::io::BufReader::new(read_half)
                .read_line(&mut req)
                .await
                .expect("read request");
            let msg = ServerMessage::ok(Some(1), result);
            let mut line = serde_json::to_vec(&msg).expect("encode");
            line.push(b'\n');
            write_half.write_all(&line).await.expect("write");
            write_half.flush().await.expect("flush");
        });
        (path, handle)
    }

    #[tokio::test]
    async fn safe_true_means_yes() {
        let (path, server) = canned_server(serde_json::json!({ "safe": true }));
        let verdict = safe_to_restart(path.to_str().unwrap(), Duration::from_secs(1)).await;
        server.await.expect("server");
        assert_eq!(verdict, SafeToRestart::Yes);
    }

    #[tokio::test]
    async fn safe_false_with_reason_means_no() {
        let (path, server) =
            canned_server(serde_json::json!({ "safe": false, "reason": "the robot is walking" }));
        let verdict = safe_to_restart(path.to_str().unwrap(), Duration::from_secs(1)).await;
        server.await.expect("server");
        assert_eq!(verdict, SafeToRestart::No("the robot is walking".into()));
    }

    #[tokio::test]
    async fn missing_safe_field_is_incompatible() {
        let (path, server) = canned_server(serde_json::json!({ "reason": "the robot is walking" }));
        let verdict = safe_to_restart(path.to_str().unwrap(), Duration::from_secs(1)).await;
        server.await.expect("server");
        assert!(
            matches!(verdict, SafeToRestart::Incompatible(_)),
            "expected Incompatible, got {verdict:?}"
        );
        assert!(!verdict.permits_restart());
    }

    #[tokio::test]
    async fn unreachable_socket_is_unreachable() {
        let verdict = safe_to_restart("/tmp/miniduck-robot-test-does-not-exist.sock", Duration::from_millis(100)).await;
        assert_eq!(verdict, SafeToRestart::Unreachable);
        assert!(verdict.permits_restart());
    }

    #[tokio::test]
    async fn model_api_returns_value() {
        let (path, server) = canned_server(serde_json::json!({ "model_api": 2 }));
        let api = model_api(path.to_str().unwrap(), Duration::from_secs(1)).await;
        server.await.expect("server");
        assert_eq!(api, Some(2));
    }
}
