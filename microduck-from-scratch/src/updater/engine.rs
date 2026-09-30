//! 更新引擎：apply / rollback / 崩溃恢复的编排，boot counter 裁决。
//!
//! 进程 supervisor 语义（D37）：没有 systemd，updaterd 直接 spawn/kill
//! miniduckd 子进程替代 `systemctl restart`。原版五裁决里的 Degraded
//! 砍掉（D41）——mini 的健康判定是布尔，healthy 与"其余一切"两类。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{gate, journal, manifest, store};
use super::journal::{LogEntry, Outcome};
use super::paths::UpdaterPaths;

/// MAX_BOOT_ATTEMPTS：原版 engine.rs:38。试用版启动第 2 次还过不了门，
/// 就承认它是砖，回滚。
pub const MAX_BOOT_ATTEMPTS: u32 = 2;
/// 默认健康门预算 30s（拍脑袋——代理假设：原版 health timeout 来自
/// 配置 HealthCheck::Socket{timeout}，无单一默认常量）。env
/// `MINIDUCK_UPDATER_GATE_MS` 覆盖，验收用 3~5s。
pub const DEFAULT_GATE: Duration = Duration::from_secs(30);
/// 杀旧 daemon 时 SIGTERM 后的宽限（拍脑袋——代理假设：FakeIo 进程
/// 退出是即时的，2s 足够慷慨）。
const SIGTERM_GRACE: Duration = Duration::from_secs(2);

/// 阶段枚举：原版 proto 命名子集（duck-ipc-proto/src/lib.rs `Phase`），
/// 砍掉 Preflight/Checking/Downloading/RunningPreHook/RunningPostHook
/// （那些对应我们没有的网络下载与 hooks，D41）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Verifying,
    Extracting,
    Swapping,
    Applying,
    HealthGate,
    Committing,
    RollingBack,
}

/// boot counter：swap 之前 arm（写盘），门过了才删（confirm）。
/// 每次 updaterd 启动发现它还在，就知道上次死在门没跑完的窗口里。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pending {
    pub version: String,
    pub previous: Option<String>,
    pub boots: u32,
}

/// 崩溃恢复裁决（纯函数，单测直接打）：
/// healthy 且还没把启动预算用完 → confirm；否则回滚。
pub fn recovery_verdict(healthy: bool, boots: u32) -> RecoveryVerdict {
    if healthy && boots < MAX_BOOT_ATTEMPTS {
        RecoveryVerdict::Confirm
    } else {
        RecoveryVerdict::Rollback
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryVerdict {
    Confirm,
    Rollback,
}

/// 一次 mutating 操作的结果：Ok(result JSON) 或 Err((rpc_code, message))。
pub type OpResult = Result<serde_json::Value, (i32, String)>;

/// pending.json 读取（自由函数：status 等只读路径不锁 engine 也能用）。
pub fn load_pending(paths: &UpdaterPaths) -> Option<Pending> {
    let text = std::fs::read_to_string(paths.pending()).ok()?;
    serde_json::from_str(&text).ok()
}

pub struct Engine {
    pub paths: UpdaterPaths,
    pub phase: Arc<Mutex<Phase>>,
    pub gate_budget: Duration,
    pub daemon_sock: String,
    pub daemon_cwd: PathBuf,
    daemon: Option<Child>,
}

impl Engine {
    pub fn new(
        paths: UpdaterPaths,
        phase: Arc<Mutex<Phase>>,
        gate_budget: Duration,
        daemon_sock: String,
        daemon_cwd: PathBuf,
    ) -> Self {
        Self { paths, phase, gate_budget, daemon_sock, daemon_cwd, daemon: None }
    }

    fn set_phase(&self, phase: Phase) {
        *self.phase.lock().expect("phase mutex poisoned") = phase;
    }

    // ---- 进程 supervisor（D37） ----

    /// 从 `install/current/bin/miniduckd` spawn daemon：继承 env
    /// （MINIDUCK_POLICY / ORT_DYLIB_PATH 都走这条路），setsid 脱离
    /// 会话——updaterd 退出（含 kill -9）不杀 daemon，场景 E 依赖这个。
    fn spawn_daemon(&mut self) -> Result<(), String> {
        let bin = self.paths.current_link().join("bin/miniduckd");
        if !bin.exists() {
            return Err(format!("no daemon binary at {}", bin.display()));
        }
        let mut cmd = Command::new(&bin);
        cmd.current_dir(&self.daemon_cwd);
        // SAFETY: pre_exec 在 fork 后 exec 前的子进程里跑；setsid 只做
        // 会话脱离，不碰内存。失败时子进程报错退出，spawn 返回 Err。
        unsafe {
            std::os::unix::process::CommandExt::pre_exec(&mut cmd, || {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = cmd.spawn().map_err(|e| format!("spawn {}: {e}", bin.display()))?;
        eprintln!("mini-updaterd: spawned miniduckd pid {} from {}", child.id(), bin.display());
        self.daemon = Some(child);
        Ok(())
    }

    /// 先 SIGTERM 短等再 SIGKILL（交接 M7 第 4 条）。
    fn kill_daemon(&mut self) {
        let Some(mut child) = self.daemon.take() else { return };
        let pid = child.id() as i32;
        // SAFETY: kill(2) 发信号，pid 来自我们自己的子进程。
        unsafe { libc::kill(pid, libc::SIGTERM) };
        let deadline = std::time::Instant::now() + SIGTERM_GRACE;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Ok(None) => {
                    unsafe { libc::kill(pid, libc::SIGKILL) };
                    let _ = child.wait();
                    return;
                }
                Err(_) => return,
            }
        }
    }

    /// 清上次 updaterd 死后留下的孤儿 daemon（场景 E1：旧 daemon 还活
    /// 着，不杀就会和新 spawn 的抢 /tmp/miniduckd.sock——虽然后 bind 者
    /// 抢得走文件名，但两个控制循环跑同一台机器人没有定义）。按 /proc
    /// 的 comm 匹配 `miniduckd*`（投毒版 wrapper exec 出的进程名是
    /// miniduckd-real）。
    fn kill_orphan_daemons() {
        let mut pids = Vec::new();
        if let Ok(rd) = std::fs::read_dir("/proc") {
            for entry in rd.flatten() {
                let name = entry.file_name();
                let Some(pid) = name.to_str().and_then(|s| s.parse::<i32>().ok()) else {
                    continue;
                };
                if pid == std::process::id() as i32 {
                    continue;
                }
                let Ok(comm) = std::fs::read_to_string(entry.path().join("comm")) else {
                    continue;
                };
                if !comm.trim().starts_with("miniduckd") {
                    continue;
                }
                // 跳过僵尸：容器 PID 1 不收割，kill -9 updaterd 留下的
                // daemon 死后变僵尸，/proc comm 还在，重复杀只会刷日志。
                if let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) {
                    if stat.rsplit(')').next().is_some_and(|s| s.trim().starts_with('Z')) {
                        continue;
                    }
                }
                pids.push(pid);
            }
        }
        for &pid in &pids {
            eprintln!("mini-updaterd: reaping orphan daemon pid {pid}");
            unsafe { libc::kill(pid, libc::SIGTERM) };
        }
        if !pids.is_empty() {
            std::thread::sleep(Duration::from_millis(300));
            for &pid in &pids {
                unsafe { libc::kill(pid, libc::SIGKILL) };
            }
        }
    }

    // ---- 崩溃恢复（每次启动先跑，先于 IPC bind） ----

    pub async fn recover(&mut self) {
        if let Err(e) = store::cleanup_staging(&self.paths) {
            eprintln!("mini-updaterd: cleanup staging: {e}");
        }

        // pending 存在 = 上次死在 swap 之后、confirm 之前。boots+=1 写回
        // （先记账再行动：再死一次计数也涨，MAX_BOOT_ATTEMPTS 才兜得住）。
        let pending = self.load_pending().map(|mut p| {
            p.boots += 1;
            if let Err(e) = self.save_pending(&p) {
                eprintln!("mini-updaterd: bump pending boots: {e}");
            }
            p
        });

        Self::kill_orphan_daemons();
        if self.paths.current_link().exists() {
            if let Err(e) = self.spawn_daemon() {
                eprintln!("mini-updaterd: spawn daemon at boot: {e}");
            }
        }

        let Some(pending) = pending else { return };
        eprintln!(
            "mini-updaterd: pending trial {} (boot {}) found, re-running health gate",
            pending.version, pending.boots
        );
        self.set_phase(Phase::HealthGate);
        let report = gate::poll(&self.daemon_sock, self.gate_budget).await;
        match recovery_verdict(report.healthy, pending.boots) {
            RecoveryVerdict::Confirm => {
                self.confirm(&pending, format!("recovered after crash: {}", report.detail), None);
            }
            RecoveryVerdict::Rollback => {
                let reason = if report.healthy {
                    format!("boot budget exhausted ({} >= {MAX_BOOT_ATTEMPTS})", pending.boots)
                } else {
                    format!("recovered trial still unhealthy: {}", report.detail)
                };
                self.rollback_to(pending.previous.as_deref(), &pending.version, &reason, None).await;
                self.clear_pending();
            }
        }
        self.set_phase(Phase::Idle);
    }

    // ---- apply / rollback（IPC 层已持单飞锁） ----

    pub async fn apply(&mut self, version: &str, peer_uid: Option<u32>) -> OpResult {
        let from = store::current_version(&self.paths);

        // Verifying：manifest 解析 + sha256。拒绝不留任何副作用。
        self.set_phase(Phase::Verifying);
        let manifest_path = self.paths.source().join(format!("{version}.manifest.json"));
        let manifest = match manifest::Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(e) => return Err(self.reject(from, version, e, peer_uid)),
        };
        if let Err(e) = manifest.verify_artifact(&self.paths.source()) {
            return Err(self.reject(from, version, e, peer_uid));
        }

        // Extracting：验过才解包，staging → rename。
        self.set_phase(Phase::Extracting);
        if let Err(e) = store::install_release(&self.paths, version) {
            return Err(self.reject(from, version, e, peer_uid));
        }

        // swap 之前 arm pending（交接 M7 第 6 条）：死在 swap 与 confirm
        // 之间时，重启靠它知道自己在试用哪个版本。
        let pending = Pending { version: version.into(), previous: from.clone(), boots: 0 };
        if let Err(e) = self.save_pending(&pending) {
            return Err(self.reject(from, version, format!("arm pending: {e}"), peer_uid));
        }

        self.set_phase(Phase::Applying);
        self.kill_daemon();
        self.set_phase(Phase::Swapping);
        if let Err(e) = store::set_current(&self.paths, version) {
            return Err(self.reject(from, version, format!("swap current: {e}"), peer_uid));
        }
        if let Err(e) = self.spawn_daemon() {
            return Err(self.reject(from, version, e, peer_uid));
        }

        // 故障注入（场景 E）：swap 落盘、pending 已 arm、新版 daemon 已在
        // 跑——kill -9 能落在的最坏窗口。立刻死，不写 log、不 confirm。
        if std::env::var("MINIDUCK_UPDATER_EXIT_AFTER_SWAP").as_deref() == Ok("1") {
            eprintln!("mini-updaterd: MINIDUCK_UPDATER_EXIT_AFTER_SWAP=1, exiting after swap");
            std::process::exit(1);
        }

        self.set_phase(Phase::HealthGate);
        let report = gate::poll(&self.daemon_sock, self.gate_budget).await;
        if report.healthy {
            self.confirm(&pending, "health gate passed".to_owned(), peer_uid);
            self.set_phase(Phase::Idle);
            Ok(serde_json::json!({
                "outcome": "committed", "from": from, "to": version,
            }))
        } else {
            let reason = format!("health gate rejected {version}: {}", report.detail);
            self.rollback_to(from.as_deref(), version, &reason, peer_uid).await;
            // 试用结束（不管是 confirm 还是 rollback），pending 都必须清，
            // 否则下次启动会把已回滚的 trial 再裁决一遍。
            self.clear_pending();
            self.set_phase(Phase::Idle);
            Ok(serde_json::json!({
                "outcome": "rolled_back", "from": from, "to": version, "reason": reason,
            }))
        }
    }

    /// 手动回滚：换回 previous（从 log 推导），同样过健康门。
    pub async fn rollback(&mut self, peer_uid: Option<u32>) -> OpResult {
        let Some(current) = store::current_version(&self.paths) else {
            return Err((super::ERR_FAILED, "no current release".into()));
        };
        let Some(previous) = journal::previous_for(&self.paths, &current) else {
            return Err((super::ERR_FAILED, format!("no previous release for {current}")));
        };
        self.set_phase(Phase::RollingBack);
        let reason = "manual rollback";
        self.rollback_to(Some(&previous), &current, reason, peer_uid).await;
        self.set_phase(Phase::Idle);
        Ok(serde_json::json!({
            "outcome": "rolled_back", "from": current, "to": previous, "reason": reason,
        }))
    }

    // ---- 内部 ----

    /// 拒绝：写 Rejected 行，返回 RPC 错误。调用方保证无副作用。
    fn reject(
        &self,
        from: Option<String>,
        version: &str,
        reason: String,
        peer_uid: Option<u32>,
    ) -> (i32, String) {
        let entry = LogEntry::new(from, version, Outcome::Rejected)
            .reason(reason.clone())
            .peer_uid(peer_uid);
        if let Err(e) = journal::append(&self.paths, &entry) {
            eprintln!("mini-updaterd: append log: {e}");
        }
        (super::ERR_REJECTED, format!("Rejected: {reason}"))
    }

    /// confirm：删 pending + 写 Committed 行（peer_uid 照记——放行也要
    /// 记账，崩溃恢复路径没有 peer 则为 None）。
    fn confirm(&self, pending: &Pending, reason: String, peer_uid: Option<u32>) {
        self.set_phase(Phase::Committing);
        self.clear_pending();
        let entry = LogEntry::new(pending.previous.clone(), &pending.version, Outcome::Committed)
            .reason(reason)
            .peer_uid(peer_uid);
        if let Err(e) = journal::append(&self.paths, &entry) {
            eprintln!("mini-updaterd: append log: {e}");
        }
        eprintln!("mini-updaterd: committed {}", pending.version);
    }

    /// 换回 previous 并 respawn，门再确认旧版 healthy 才算收拾完
    /// （交接 M7 第 5 条）。previous 不存在则保持现状报病。
    async fn rollback_to(
        &mut self,
        previous: Option<&str>,
        attempted: &str,
        reason: &str,
        peer_uid: Option<u32>,
    ) {
        self.set_phase(Phase::RollingBack);
        let prev = previous.map(str::to_owned);
        let detail = match &prev {
            Some(prev) if self.paths.release(prev).is_dir() => {
                self.kill_daemon();
                let swap = store::set_current(&self.paths, prev);
                let respawn = swap.map_err(|e| e.to_string()).and_then(|()| self.spawn_daemon());
                match respawn {
                    Ok(()) => {
                        // 门再确认旧版。旧版是已经 Commit 过的版本，正常
                        // 应该立刻 healthy；确认失败要喊出来——这时机器人
                        // 两头不靠，log 是唯一线索。
                        let report =
                            gate::poll(&self.daemon_sock, self.gate_budget).await;
                        if report.healthy {
                            format!("{reason}; rolled back to {prev}, re-gate healthy")
                        } else {
                            format!(
                                "{reason}; rolled back to {prev} but re-gate says: {}",
                                report.detail
                            )
                        }
                    }
                    Err(e) => format!("{reason}; rollback to {prev} failed: {e}"),
                }
            }
            _ => format!("{reason}; no previous release to roll back to, staying on {attempted}"),
        };
        eprintln!("mini-updaterd: {detail}");
        let entry = LogEntry::new(prev, attempted, Outcome::RolledBack)
            .reason(detail)
            .peer_uid(peer_uid);
        if let Err(e) = journal::append(&self.paths, &entry) {
            eprintln!("mini-updaterd: append log: {e}");
        }
    }

    pub fn load_pending(&self) -> Option<Pending> {
        load_pending(&self.paths)
    }

    fn clear_pending(&self) {
        if let Err(e) = std::fs::remove_file(self.paths.pending()) {
            // NotFound 不算错（比如 rollback_to 里已经清过）。
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("mini-updaterd: clear pending: {e}");
            }
        }
    }

    fn save_pending(&self, pending: &Pending) -> std::io::Result<()> {
        let f = std::fs::File::create(self.paths.pending())?;
        serde_json::to_writer(&f, pending).expect("Pending is serializable");
        f.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_verdict_truth_table() {
        // healthy 且 boots < MAX_BOOT_ATTEMPTS → confirm
        assert_eq!(recovery_verdict(true, 0), RecoveryVerdict::Confirm);
        assert_eq!(recovery_verdict(true, 1), RecoveryVerdict::Confirm);
        // boots 预算用完，即使 healthy 也回滚（原版：预算决定何时问，
        // 用完即放弃——engine.rs:1839 BootCounter::exhausted）
        assert_eq!(recovery_verdict(true, 2), RecoveryVerdict::Rollback);
        assert_eq!(recovery_verdict(true, 3), RecoveryVerdict::Rollback);
        // unhealthy 一律回滚
        assert_eq!(recovery_verdict(false, 0), RecoveryVerdict::Rollback);
        assert_eq!(recovery_verdict(false, 1), RecoveryVerdict::Rollback);
    }

    #[test]
    fn pending_roundtrip() {
        use crate::updater::store::tests::TempDir;
        let tmp = TempDir::new("pending");
        let paths = UpdaterPaths { root: tmp.path().to_path_buf() };
        paths.ensure_dirs().unwrap();
        let engine = Engine::new(
            paths,
            Arc::new(Mutex::new(Phase::Idle)),
            Duration::from_millis(1),
            "/nonexistent.sock".into(),
            tmp.path().to_path_buf(),
        );
        assert!(engine.load_pending().is_none());
        let p = Pending { version: "2.0.0".into(), previous: Some("1.0.0".into()), boots: 1 };
        engine.save_pending(&p).unwrap();
        let got = engine.load_pending().unwrap();
        assert_eq!(got.version, "2.0.0");
        assert_eq!(got.previous.as_deref(), Some("1.0.0"));
        assert_eq!(got.boots, 1);
    }
}
