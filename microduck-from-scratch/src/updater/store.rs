//! 安装存储：解包进 staging、rename 落位、current 原子换指向。
//!
//! 原子性语义对照原版 store.rs:165-203：`current` 换指向 = 先 symlink 到
//! `.current.tmp` 再 `rename(2)` 覆盖，随后 fsync 父目录——rename 在同一
//! 目录内是原子的，并发读者要么看到旧指向要么看到新指向，没有中间态。

use std::io;
use std::path::Path;
use std::process::Command;

use super::paths::UpdaterPaths;

/// 当前版本：以 readlink 为准（交接 M7：current 版本不另开状态文件）。
pub fn current_version(paths: &UpdaterPaths) -> Option<String> {
    let target = std::fs::read_link(paths.current_link()).ok()?;
    target.file_name().map(|n| n.to_string_lossy().into_owned())
}

/// 原子换指向：tmp symlink + rename 覆盖 + fsync 父目录。
pub fn set_current(paths: &UpdaterPaths, version: &str) -> io::Result<()> {
    let link = paths.current_link();
    let tmp = paths.current_tmp_link();
    // 上次 crash 残留的 tmp 链接不能挡住这次 swap（原版同：store.rs:181）。
    let _ = std::fs::remove_file(&tmp);
    // 相对目标（releases/<ver>）：root 整体搬走链接仍然有效。
    std::os::unix::fs::symlink(Path::new("releases").join(version), &tmp)?;
    std::fs::rename(&tmp, &link).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    fsync_dir(&paths.install())
}

/// 解包 artifact 到 `.staging-<ver>`，完成后 rename 进 `releases/<ver>`。
/// 任何一步失败都清掉 staging，不留半成品（sha256 校验在调用方先做）。
///
/// 原版 artifact 是 .tar.zst 用内嵌解码；我们调系统 `tar -xf` 解未压缩
/// .tar（D39，验收环境有 GNU tar，不值得为一个教学复刻拉 tar+zstd crate）。
pub fn install_release(paths: &UpdaterPaths, version: &str) -> Result<(), String> {
    let staging = paths.staging(version);
    let target = paths.release(version);
    if target.exists() {
        // 幂等：artifact 已过 sha256 校验（调用方先做），releases/ 里
        // 的目录视为同一内容的物化，重 apply 直接跳过解包。这让"重装
        // 当前版"（场景 D 用）和"回滚后再升回来"成为合法操作。
        return Ok(());
    }
    cleanup_path(&staging)?;
    std::fs::create_dir_all(&staging).map_err(|e| format!("mkdir staging: {e}"))?;

    let result = (|| -> Result<(), String> {
        let tar = paths.source().join(format!("{version}.tar"));
        let status = Command::new("tar")
            .arg("-xf")
            .arg(&tar)
            .arg("-C")
            .arg(&staging)
            .status()
            .map_err(|e| format!("spawn tar: {e}"))?;
        if !status.success() {
            return Err(format!("tar -xf {} exited {status}", tar.display()));
        }
        std::fs::rename(&staging, &target).map_err(|e| format!("rename into releases: {e}"))?;
        fsync_dir(&paths.releases()).map_err(|e| format!("fsync releases: {e}"))?;
        Ok(())
    })();

    if result.is_err() {
        let _ = cleanup_path(&staging);
    }
    result
}

/// 启动时清上次 crash 的残留：staging 目录与 tmp symlink（原版
/// engine.rs:1762 附近"delete staging leftovers"的对应物）。
pub fn cleanup_staging(paths: &UpdaterPaths) -> io::Result<()> {
    let _ = std::fs::remove_file(paths.current_tmp_link());
    for entry in std::fs::read_dir(paths.install())? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(".staging-") {
            cleanup_path(&entry.path()).map_err(io::Error::other)?;
        }
    }
    Ok(())
}

fn cleanup_path(path: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove {}: {e}", path.display())),
    }
}

pub fn fsync_dir(dir: &Path) -> io::Result<()> {
    std::fs::File::open(dir)?.sync_all()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 极简 tempdir：不引 tempfile crate，验收环境只有自己跑测试。
    pub struct TempDir(std::path::PathBuf);

    impl TempDir {
        pub fn new(tag: &str) -> Self {
            static N: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "m7-test-{tag}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }

        pub fn write(&self, name: &str, text: &str) -> std::path::PathBuf {
            let p = self.0.join(name);
            std::fs::write(&p, text).unwrap();
            p
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn paths_in(tmp: &TempDir) -> UpdaterPaths {
        let paths = UpdaterPaths { root: tmp.path().to_path_buf() };
        paths.ensure_dirs().unwrap();
        paths
    }

    #[test]
    fn atomic_swap_repoints_current() {
        let tmp = TempDir::new("swap");
        let paths = paths_in(&tmp);
        std::fs::create_dir_all(paths.release("1.0.0")).unwrap();
        std::fs::create_dir_all(paths.release("2.0.0")).unwrap();

        set_current(&paths, "1.0.0").unwrap();
        assert_eq!(current_version(&paths).as_deref(), Some("1.0.0"));

        set_current(&paths, "2.0.0").unwrap();
        assert_eq!(current_version(&paths).as_deref(), Some("2.0.0"));
        // rename 是覆盖语义：交换全程 current 都指向某个完整版本，
        // tmp 链接不留痕。
        assert!(!paths.current_tmp_link().exists());
    }

    #[test]
    fn leftover_tmp_link_does_not_block_swap() {
        let tmp = TempDir::new("swap-crash");
        let paths = paths_in(&tmp);
        std::fs::create_dir_all(paths.release("1.0.0")).unwrap();
        // 模拟上次 crash 在 rename 之前：tmp 是悬空 symlink。
        std::os::unix::fs::symlink("releases/never-existed", paths.current_tmp_link()).unwrap();
        set_current(&paths, "1.0.0").unwrap();
        assert_eq!(current_version(&paths).as_deref(), Some("1.0.0"));
    }

    #[test]
    fn cleanup_staging_removes_leftovers() {
        let tmp = TempDir::new("staging");
        let paths = paths_in(&tmp);
        std::fs::create_dir_all(paths.staging("9.9.9")).unwrap();
        std::os::unix::fs::symlink("releases/x", paths.current_tmp_link()).unwrap();
        cleanup_staging(&paths).unwrap();
        assert!(!paths.staging("9.9.9").exists());
        assert!(!paths.current_tmp_link().exists());
    }
}
