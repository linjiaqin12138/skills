//! 更新流水：`state/update-log.jsonl`，每次 attempt 一行 NDJSON，
//! append + sync_data。它同时是"上一个版本"的索引——previous 从最后一行
//! 匹配 current 的 Committed 推导，不另开状态文件（交接 M7 第 7 条）。

use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use super::paths::UpdaterPaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Committed,
    RolledBack,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unix 秒。不引 chrono：机器可读优先于人类可读，`date -d @…` 能看。
    pub ts: u64,
    /// 发起时的 current（Rejected 时 = 没动过的 current）。
    pub from: Option<String>,
    /// 目标版本。
    pub to: String,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// SO_PEERCRED-lite：发起 mutating 调用的 peer uid；崩溃恢复等
    /// 非 RPC 路径为 null（放行/拒绝都要记账，交接 M7 第 10 条）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_uid: Option<u32>,
}

impl LogEntry {
    pub fn new(from: Option<String>, to: &str, outcome: Outcome) -> Self {
        Self {
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            from,
            to: to.into(),
            outcome,
            reason: None,
            peer_uid: None,
        }
    }

    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn peer_uid(mut self, uid: Option<u32>) -> Self {
        self.peer_uid = uid;
        self
    }
}

/// 追加一行并 sync_data：崩溃恢复靠这行判断"上次进行到哪"，丢了它
/// boot counter 就失去了对照。
pub fn append(paths: &UpdaterPaths, entry: &LogEntry) -> std::io::Result<()> {
    let mut line = serde_json::to_string(entry).expect("LogEntry is serializable");
    line.push('\n');
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.log())?;
    f.write_all(line.as_bytes())?;
    f.sync_data()
}

pub fn entries(paths: &UpdaterPaths) -> Vec<LogEntry> {
    let Ok(text) = std::fs::read_to_string(paths.log()) else {
        return Vec::new();
    };
    // 坏行跳过而不是整个拒读：log 是审计线索，一行损坏不该没收全部历史。
    text.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// previous = 最后一行 `to == current` 的 Committed 的 `from`。
/// 出厂镜像（bootstrap 直接建 symlink，无 log）→ None。
pub fn previous_for(paths: &UpdaterPaths, current: &str) -> Option<String> {
    entries(paths)
        .iter()
        .rev()
        .find(|e| e.outcome == Outcome::Committed && e.to == current)
        .and_then(|e| e.from.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::store::tests::TempDir;

    #[test]
    fn previous_comes_from_last_matching_commit() {
        let tmp = TempDir::new("journal");
        let paths = UpdaterPaths { root: tmp.path().to_path_buf() };
        paths.ensure_dirs().unwrap();

        append(&paths, &LogEntry::new(Some("1.0.0".into()), "2.0.0", Outcome::Committed)).unwrap();
        append(&paths, &LogEntry::new(Some("2.0.0".into()), "3.0.0", Outcome::RolledBack)).unwrap();
        // current 回到 2.0.0：previous 仍是 1.0.0（RolledBack 不改写历史）。
        assert_eq!(previous_for(&paths, "2.0.0").as_deref(), Some("1.0.0"));
        // 从未 Commit 过的版本没有 previous。
        assert_eq!(previous_for(&paths, "9.9.9"), None);
    }
}
