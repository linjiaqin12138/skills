# 内核 ELF 段布局与 RELRO 坑

> 交互版（推荐）：[figures/elf-relro-got/index.html](figures/elf-relro-got/index.html)

裸机内核是固定地址 EXEC。Limine 按 PT_LOAD 权限映射页；同页既要 R 又要 RW 会 PANIC。

本项目修法：`.got` 收进 `.data`（`impl/kernel/linker.ld`）；`-znorelro`（`impl/.cargo/config.toml`）。

细节、示意图与自测题见 HTML 页。
