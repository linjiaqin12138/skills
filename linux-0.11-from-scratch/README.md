# Linux 0.11 从零手搓

用 Rust 在 x86_64 上逐里程碑复刻 Linux 0.11 的进程、内存、文件系统与终端能力。

- 现在读到：[M2 Bite2 · panic!：让 Rust 错误可见](book/m02-exceptions/b02-panic-printing.html)
- 目录：[book/index.md](book/index.md)
- 实现在 `impl/`。当前验收：在仓库根执行 `bash accept/m02-b02.sh`
- 上游：`upstream/` 为只读参考；Limine 使用 v10.8.5-binary
