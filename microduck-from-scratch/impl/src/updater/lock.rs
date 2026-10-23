//! 单飞锁：`state/update.lock` 上的 flock(LOCK_EX|LOCK_NB)。
//!
//! 取舍（交接 M7 第 8 条给了三个选项）：选 libc flock 而不是 O_EXCL 锁
//! 文件——场景 E 故意 kill -9 updaterd，O_EXCL 文件会残留，把重启后的
//! apply 全卡成 Busy；flock 随进程死亡由内核自动释放，天然免疫 stale
//! lock。fs2 是同一语义的封装，但 libc 已在依赖树里，不为此多一个依赖。

use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd as _;
use std::path::Path;

pub struct UpdateLock {
    // fd 持有即锁持有；Drop 关 fd 自动放锁。
    _file: File,
}

#[derive(Debug)]
pub enum AcquireError {
    Busy,
    Io(io::Error),
}

pub fn try_acquire(path: &Path) -> Result<UpdateLock, AcquireError> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(AcquireError::Io)?;
    // SAFETY: fd 来自刚打开的 File，有效；flock 不改变文件内容。
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        return Ok(UpdateLock { _file: file });
    }
    let err = io::Error::last_os_error();
    match err.raw_os_error() {
        Some(libc::EWOULDBLOCK) => Err(AcquireError::Busy),
        _ => Err(AcquireError::Io(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::store::tests::TempDir;

    #[test]
    fn second_holder_gets_busy() {
        let tmp = TempDir::new("lock");
        let path = tmp.path().join("update.lock");
        let first = try_acquire(&path).unwrap();
        assert!(matches!(try_acquire(&path), Err(AcquireError::Busy)));
        drop(first);
        // 放锁后可再拿（kill -9 时由内核代做这一步）。
        assert!(try_acquire(&path).is_ok());
    }
}
