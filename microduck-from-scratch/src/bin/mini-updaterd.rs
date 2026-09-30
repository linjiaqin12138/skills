//! mini-updaterd：M7 的迷你更新守护进程。
//!
//! 职责（对照 reference/docs/design/updater-design.md，偏差 D37–D41）：
//! 崩溃恢复（boot counter 重跑健康门）→ 进程 supervisor（spawn/kill
//! miniduckd 子进程，替代 systemd）→ 自己的 NDJSON socket 服务
//! update.* 方法面。updaterd 退出不杀 daemon（setsid 脱离会话）。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use miniduck::updater::engine::{DEFAULT_GATE, Engine, Phase};
use miniduck::updater::ipc;
use miniduck::updater::paths::UpdaterPaths;
use tokio::net::UnixListener;

const SOCK_PATH: &str = "/tmp/mini-updaterd.sock";
const DAEMON_SOCK: &str = "/tmp/miniduckd.sock";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = UpdaterPaths::from_env();
    paths.ensure_dirs()?;

    let gate_budget = std::env::var("MINIDUCK_UPDATER_GATE_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_GATE);
    let sock_path = std::env::var("MINIDUCK_UPDATER_SOCK").unwrap_or_else(|_| SOCK_PATH.into());
    let daemon_cwd = std::env::var("MINIDUCK_DAEMON_CWD")
        .map(Into::into)
        .unwrap_or_else(|_| std::env::current_dir().expect("cwd"));

    let phase = Arc::new(Mutex::new(Phase::Idle));
    let mut engine = Engine::new(
        paths.clone(),
        phase.clone(),
        gate_budget,
        DAEMON_SOCK.into(),
        daemon_cwd,
    );

    // 恢复先于服务：pending 在 = 上次死在最坏窗口，先把局面收拾到
    // confirm/rollback 之一，再开门接客。
    engine.recover().await;

    let _ = std::fs::remove_file(&sock_path);
    let listener = UnixListener::bind(&sock_path)?;
    // D8 同款：socket 文件 0660 是第一道门，SO_PEERCRED-lite 是第二道。
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&sock_path, std::fs::Permissions::from_mode(0o660))?;
    }
    let owner_uid = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(&sock_path)?.uid()
    };
    eprintln!(
        "mini-updaterd listening on {sock_path} (root {}, gate {}ms)",
        paths.root.display(),
        gate_budget.as_millis()
    );

    let server = Arc::new(ipc::Server {
        paths: paths.clone(),
        engine: Arc::new(tokio::sync::Mutex::new(engine)),
        phase,
        sock_owner_uid: owner_uid,
    });
    ipc::serve(listener, server).await?;
    Ok(())
}
