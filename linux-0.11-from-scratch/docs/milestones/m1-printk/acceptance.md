# M1 验收档案：内核 printk / framebuffer 控制台

> 里程碑目标（phase-2.md M1）：framebuffer 控制台（Limine 显存 + 8x16 字体 + 滚屏）+ `core::fmt`；验收 = 屏幕多行格式化文本 + 串口同内容。
> 状态：**Bite1 已通过**（2026-10-12）；Bite2（printk!/fmt/滚屏）未开始。

## Bite1「拿到显存并写出固定字符串」

### 验收命令与实测输出（主 agent 复跑 2026-10-12）

```
$ make check-m1-b1
…（make build / ISO 略）…
>>> serial.log:
limine: Loading executable `boot():/kernel.elf`...
M1: fb 800x600x32
M1: fb mask r8@16 g8@8 b8@0 model=1
M1: hello framebuffer
M1: console line 2
M0: kernel alive
>>> asserting serial markers…
M0: kernel alive
M1: fb 800x600x32
M1: hello framebuffer
M1: console line 2
screendump size=1440015
check-m1-b1 OK
```

目检：`assets/dst/m1-bite1-screendump.png`（由 `build/m1-bite1.ppm` 转出）可见黑底两行：
`M1: hello framebuffer` / `M1: console line 2`。

### 涉及代码文件

| 文件 | 职责 |
|---|---|
| `kernel/src/limine_fb.rs` | 经 `limine` crate 声明请求 + `first_framebuffer()`（2026-10-13 起不再手写布局） |
| `kernel/Cargo.toml` | 依赖 `limine = "0.6.5"` |
| `kernel/src/fb.rs` | `put_pixel` / `clear`（按 mask 打包） |
| `kernel/src/font.rs` | 自制 8×16 ASCII 点阵 |
| `kernel/src/console.rs` | 光标 + `put_char` / `write_str`（无滚屏） |
| `kernel/src/main.rs` | 取 FB、画两行、串口同步 |
| `limine.conf` | `resolution: 800x600x32` |
| `Makefile` | `check-m1-b1` |
| `kernel/linker.ld` / `.cargo/config.toml` | `.got` 进 `.data`；`-znorelro`（Limine ELF 同页坑） |

### 教程

- Bite1：`tutorial-bite1-framebuffer.html`
- 概念：`docs/concepts/linear-framebuffer.md`
- 架构：`assets/dst/arch.svg` + `arch-diff.svg`（d2 渲染；fireworks JSON 在 `assets/src/` 备日后重渲）
