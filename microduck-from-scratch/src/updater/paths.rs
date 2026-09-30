//! 目录布局（交接 M7 指定，原版 updater 的 LocalDir 形态对应物）：
//!
//! ```text
//! <root>/source/                 # release 源：<ver>.manifest.json + <ver>.tar
//! <root>/install/releases/<ver>/ # 已安装版本（解包产物，含 bin/miniduckd）
//! <root>/install/.staging-<ver>  # 解包暂存（装完 rename 进 releases/）
//! <root>/install/current -> releases/<ver>   # 消费方经此读
//! <root>/state/pending.json      # boot counter
//! <root>/state/update-log.jsonl  # 追加流水
//! <root>/state/update.lock       # 单飞锁
//! ```
//!
//! 默认根 /tmp/miniduck-update/，env `MINIDUCK_UPDATER_ROOT` 覆盖（验收隔离用）。

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UpdaterPaths {
    pub root: PathBuf,
}

impl UpdaterPaths {
    pub fn from_env() -> Self {
        let root = std::env::var("MINIDUCK_UPDATER_ROOT")
            .unwrap_or_else(|_| "/tmp/miniduck-update".into());
        Self { root: root.into() }
    }

    pub fn source(&self) -> PathBuf {
        self.root.join("source")
    }

    pub fn install(&self) -> PathBuf {
        self.root.join("install")
    }

    pub fn releases(&self) -> PathBuf {
        self.install().join("releases")
    }

    pub fn release(&self, version: &str) -> PathBuf {
        self.releases().join(version)
    }

    /// 暂存目录是 releases 的邻居而不是它的子目录：rename(2) 要求同一
    /// 文件系统，邻居关系保证了这一点（原版 store.rs 同理，tmp 与目标同父）。
    pub fn staging(&self, version: &str) -> PathBuf {
        self.install().join(format!(".staging-{version}"))
    }

    pub fn current_link(&self) -> PathBuf {
        self.install().join("current")
    }

    pub fn current_tmp_link(&self) -> PathBuf {
        self.install().join(".current.tmp")
    }

    pub fn state(&self) -> PathBuf {
        self.root.join("state")
    }

    pub fn pending(&self) -> PathBuf {
        self.state().join("pending.json")
    }

    pub fn log(&self) -> PathBuf {
        self.state().join("update-log.jsonl")
    }

    pub fn lock(&self) -> PathBuf {
        self.state().join("update.lock")
    }

    /// 启动时建齐目录骨架（幂等）。
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.source())?;
        std::fs::create_dir_all(self.releases())?;
        std::fs::create_dir_all(self.state())
    }
}
