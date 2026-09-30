//! mini-updaterd 的核心库：M7 对原版 `reference/updater/` 的迷你复刻。
//!
//! 设计对照 `reference/docs/design/updater-design.md`，代码全部从零写。
//! 砍掉的机制（登记在 docs/deviations.md D37–D41）：systemd（改为直接
//! spawn/kill miniduckd 子进程）、minisign 验签（只 sha256）、GitHub/HF
//! 网络源（只 LocalDir）、hooks/orphan 检查/transcript/Degraded 裁决/
//! subscribe 推送/self-update、allow_uids/allow_gids（SO_PEERCRED 缩为
//! uid==0 || socket owner）。

pub mod engine;
pub mod gate;
pub mod ipc;
pub mod journal;
pub mod lock;
pub mod manifest;
pub mod paths;
pub mod peer;
pub mod store;

// 应用层错误码。JSON-RPC 把 -32000..=-32099 留给 server error，从中自取
// （拍脑袋——代理假设：原版 proto 用字符串枚举而不是数字码，我们这里沿用
// miniduckd 的数字码风格）。
pub const ERR_BUSY: i32 = -32001;
pub const ERR_PERMISSION: i32 = -32002;
pub const ERR_REJECTED: i32 = -32003;
pub const ERR_FAILED: i32 = -32004;
