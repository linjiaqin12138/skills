# M1 验收档案：内核 printk / framebuffer 控制台

> 里程碑目标（phase-2.md M1）：framebuffer 控制台（Limine 显存 + 8x16 字体 + 滚屏）+ `core::fmt`；验收 = 屏幕多行格式化文本 + 串口同内容。
> 状态：**M1 已通过**——Bite1（2026-10-12）+ Bite2（2026-10-15）。

## Bite2「printk! + 滚屏」

### 验收命令与实测输出（主 agent 复跑 2026-10-15）

```
$ make check-m1-b2
…（make build / ISO 略）…
>>> serial.log:
limine: Loading executable `boot():/kernel.elf`...
M0: kernel alive
M1: fb 800x600x32
M1: fb mask r8@16 g8@8 b8@0 model=1
M1: fmt hex=0x002a dec=42 str=ok
M1: scroll line 00
…（00–59 全量 60 行）…
M1: scroll line 59
>>> asserting serial markers…
M0: kernel alive
M1: fb 800x600x32
M1: fmt hex=0x002a dec=42 str=ok
M1: scroll line 00
M1: scroll line 59
screendump size=1440015
check-m1-b2 OK
```

目检：screendump（800x600，37 行可见）顶部为 `M1: scroll line 24`、底部 `line 59` 连续无断行——60 行压测触发滚屏，前面回归行按设计滚出屏幕（串口有全量）。

注：`check-m1-b1` target 已随本 Bite 退役（demo 输出被替换，旧断言不再成立；其回归断言由 check-m1-b2 继承）。Bite1 验收记录保留下方存档。

### 涉及代码文件（本 Bite 改动）

| 文件 | 改动 |
|---|---|
| `kernel/src/console.rs` | `impl core::fmt::Write for Console`（write_str 逐字节上屏+镜像串口）；`newline()` 触底调滚屏；全局 `static mut CONSOLE` + `init()`/`global()`（`addr_of_mut!`，无锁假设见 D15）；`printk!` 宏 |
| `kernel/src/fb.rs` | 新增 `scroll_up(fb, rows, r, g, b)`：`core::ptr::copy` 整屏上移 + 底行填背景色 |
| `kernel/src/main.rs` | demo 改走 printk!：fmt 演示行 + 60 行滚屏压测；删掉手写 `write_u64` |
| `Makefile` | 新增 `check-m1-b2`；退役 `check-m1-b1`；chown 修为 `--reference=/work`（rootless 映射坑，见下） |

### 本 Bite 踩坑（已修）

rootless docker 下 `chown -R $(id -u):$(id -g)` 是错的：宿主机 uid 1000 在容器内映射为 0，容器内 chown 1000 反而映射到宿主机 100999，导致宿主机对 build/ 彻底失写权限。修为 `chown -R --reference=/work /work/build`（取挂载点属主，两种 docker 模式都对）。

### 偏差变动

- D12 收敛：fmt::Write + printk! 双写 + 滚屏落地。
- 新登记 D15（待收敛→M3）：全局 console 无锁 `static mut`，引入中断后需换锁。

## Bite1「拿到显存并写出固定字符串」（存档）

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
