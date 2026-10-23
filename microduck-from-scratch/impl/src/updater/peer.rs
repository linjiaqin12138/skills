//! SO_PEERCRED-lite（交接 M7 第 10 条）：mutating 方法的单点门控。
//!
//! 语义对照原版 ipc.rs:78-97 / 531-535：uid==0 或 ==socket 文件 owner
//! 放行，其余拒绝；peer_cred 调用本身失败也拒绝（unproven 不是
//! allowed）。完整 allow_uids/allow_gids 配置不做（D40，容器单用户）。

/// 放行判定（纯函数，单测直接打）。
pub fn peer_allowed(uid: u32, owner_uid: u32) -> bool {
    uid == 0 || uid == owner_uid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_and_owner_pass_others_deny() {
        assert!(peer_allowed(0, 1000)); // root 永远放行
        assert!(peer_allowed(0, 0));
        assert!(peer_allowed(1000, 1000)); // socket owner
        assert!(!peer_allowed(1001, 1000)); // 旁观者拒绝
    }
}
