# crate 是什么

**crate 是 Rust 编译器一次编译的最小单位。** 一个 crate 有自己的根文件、自己的模块树，以及对外可见的接口。别的代码要使用它，只能通过 `use` 引进它公开的项。

语法层面的 `crate::`、`use`、模块见 [本仓库里的 Rust 语法](rust-syntax.md)。

这个仓库是一个 **package**（一份 `Cargo.toml`，名字叫 `miniduck`），里面有好几个 crate：

| crate | 根文件 | 谁能 `use` 它 |
|---|---|---|
| 库 `miniduck` | `src/lib.rs` | 同包里的二进制、示例，以及以后别的项目 |
| 二进制 `miniduckd` | `src/main.rs` | 不能被别人 `use`，只能运行 |
| 二进制 `mini-duckctl` | `src/bin/mini-duckctl.rs` | 同上 |
| 各个示例 | `examples/*.rs` | 同上，用 `cargo run --example ...` 跑 |

库内部写 `crate::io::RobotIo` 时，`crate` 就是当前这个库的根，也就是 `lib.rs`。所以 `policy.rs` 里的 `crate::obs::Observation` 指的是本库的 `obs` 模块。

二进制要用不了 `crate::` 去摸库，因为它们是另一个 crate。`main.rs` 写的是 `use miniduck::policy::Policy`：`miniduck` 是库 crate 的名字。

`Cargo.toml` 里的 `tokio`、`serde`、`ort` 也各是一个外部 crate。`use tokio::net::UnixListener` 就是在用别人编译好的 crate。
