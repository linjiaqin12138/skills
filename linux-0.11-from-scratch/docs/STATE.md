# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M2 进行中**——Bite1（GDT 扁平接管 + IDT + CPU 异常接管）2026-10-18 复跑 `make check-m2-b1` 通过，待用户确认后进入 Bite2；M1 已通过
- 教程：M1 Bite1 `tutorial-bite1-framebuffer.html` + Bite2 `tutorial-bite2-printk-scroll.html`（m1-printk/）；**M2 Bite1 `m2-exceptions/tutorial-bite1-gdt-idt.html`**；概念 `linear-framebuffer.md` + `gdt-long-mode.html` + `idt-exception-frame.html`
- 环境：GNU Make 用 `~/.local/bin/make`（apt 下载后用户级解包）；qemu 宿主机用 `~/.local/bin/qemu-system-x86_64`（从 deb 解压，非 apt 安装；缺 ROM 时验收加 `-net none`）；工具链镜像 `linux011-rust-toolchain`✅；`reference/limine`（v10.8.5-binary）+ `reference/limine-protocol`（limine.h/PROTOCOL）已恢复；`hdc-0.11.img` 现为 1MB stub（M1 不读盘）

## 下一步动作
用户确认后进入 **M2 Bite2「panic 打印」**：panic_handler 接 printk!，panic!() 打印消息与位置而非静默 halt（main.rs:77 附近）。伏笔已埋：Rust `1/0` 走 panic 不走 #DE，Bite1 教程坑 2 已点到。
- M2 之后是 M3「中断：LAPIC 定时器 100Hz + IOAPIC 键盘」（phase-2.md），届时 D15（static mut console 换锁）与 D16 前半（IDT 向量 32~255 空洞）必须收敛。
- Makefile 验收命令：`check-m1-b2`（M1）、`check-m2-b1`（M2 Bite1）；`check-m1-b1` 已退役。

**2026-10-13**：Limine 协议已改为官方 `limine` crate 0.6.5（D14）。

**am patch 工具教训（2026-10-15）**：对旧 Bite 页面执行 `am patch` 会重读所有 `src=` 引用的**当前**代码文件——若代码已被后续 Bite 改过，旧页面的代码嵌入块和 arch.svg 会漂移成新内容，与旧正文矛盾。维护旧页面用手术式插入（从试渲染提取新块、git checkout 恢复、手工插入渲染区 + #am-source 草稿区），不直接 am patch 定稿。Bite1 页面已按此处理。

## 项目固定决策（v2，2026-10-03）
- **目标架构：x86_64**（用户明确选择，抛弃 i386 历史包袱）；引导器 Limine v10（已克隆 reference/limine）；Rust target x86_64-unknown-none（官方，已装）
- **用户态：自写 Rust mini-sh + coreutils-lite**（原版 a.out 二进制在 64 位内核上物理不可执行）；hdc-0.11.img 仍挂载读数据作 MINIX 兼容硬验收
- **syscall ABI：syscall 指令 x86_64 惯例**（rax=号，rdi/rsi/rdx 传参），调用号数值对照原版 74 项表
- **上下文切换：软件切换**（x86_64 无硬件任务切换；TSS 仅留 ring3 切栈用途）
- **内存隔离：每进程独立页表 + 更高半核**（段槽位随 ISA 退役，概念进文档）
- **实现语言：Rust**（用户明确要求）；**用户 Rust 零基础**：每段代码用到的 Rust 语法就地教
- **代码风格要求（用户 2026-10-03 补充）：尽可能优雅易懂**——coder/writer 交接必须携带此要求
- 终态功能点对齐：feature-inventory.md 逐项过（P0 硬门槛，P1/STUB/FORK Phase 4 核对）

## 关键路径
- reference（只读对照）: `linux-0.11-from-scratch/reference/`（limine binary + limine-protocol；Linux-0.11 源码克隆可按需补回；`.gitignore` 忽略整个 reference/）
- Rust 语法书（检索用，只读）: `reference/book-cn/`（若缺失可再浅克隆 rust-lang-cn/book-cn）；在线版 https://rustwiki.org/zh-CN/book/
- 原项目构建: 待 Linux-0.11 reference 补回后 `cd reference && make && make start`
- 最新架构图: `docs/arch/original.svg`（原项目总图）；**本工程当前架构权威图：`docs/milestones/m2-exceptions/assets/dst/arch.svg`（M2 Bite1 顶层图）**；分层子图：`…/m2-exceptions/assets/dst/arch-idt.svg`；arch-diff：`…/m2-exceptions/assets/dst/arch-diff.svg`（M1→M2）
- **画图工具链**：默认 fireworks（`skills/fireworks-tech-graph/`）。M2 Bite1 已从 JSON 产出顶层图、IDT 子图和 diff，并通过 validate / render / check 与 Chrome 目检；M1 历史 SVG 仍用 d2，Style 1 配色沿用
- Phase 1 拆解文档: `docs/phase-1.md`
- Phase 2 路线图: `docs/phase-2.md`
- **功能清单: `docs/feature-inventory.md`**。计数：**42 项，已交付 1 / 未交付 41**（M0 交付 §1；M1 完结推进 §6 tty 的屏幕写显路径——framebuffer+字体+滚屏+printk! 已通并注记进度，整项（含转义序列/键盘/行规程）待 M3/M13）
- 用户偏好: `~/.clone-from-scratch-tutor/user-prefs.md` + `docs/user-prefs.md`

## 偏差登记簿计数
- docs/deviations.md：**16 条**——待收敛 5（D2/D3/D6/D15/D16）、永久豁免·架构 9（D1/D4/D7/D8/D9/D10/D11/D13/D14）、待裁决 1（D5）、已收敛 1（D12，M1 Bite2）

## 本机环境侦察（续）

| 依赖 | 状态 | 解决方式 |
|---|---|---|
| GNU Make | ✅ `~/.local/bin/make` 4.4.1 | apt 下载 deb，用户级解包，无 sudo |
| Rust 工具链 | 容器镜像 `linux011-rust-toolchain`✅ | `make toolchain` |
| Limine | `reference/limine` v10.8.5-binary✅ | tag 克隆 |
| limine.h / PROTOCOL | `reference/limine-protocol/`✅ | 协议仓 / jsdelivr |
| qemu-system-x86_64 | ✅ `~/.local/bin` 包装（deb 解压）；系统 apt 未装（无 sudo） | PATH 含 `~/.local/bin`；BIOS 已补 SeaBIOS |
| hdc-0.11.img | 1MB stub（非原版镜像） | M10 前换回真镜像 |
| fireworks-tech-graph | ✅ `skills/fireworks-tech-graph/` | M2 已用 JSON 产出并验收三张图 |

**结论：M1 全链路已通（`make build && make check-m1-b2`）。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载软盘/硬盘引导扇区 → 启动一个 i386 保护模式、支持多进程 + MINIX 文件系统 + tty 终端的单内核 OS
- 主循环位置：`init/main.c:106` main() —— 初始化内存/中断/设备/调度后 `move_to_user_mode()`，fork 出 init 进程跑 /bin/sh，task0 自己 `for(;;) pause()`
- 目录地图：boot（3 个汇编：bootsect/setup/head）、init/main.c、kernel/（调度/信号/系统调用/异常 + blk_drv/chr_drv/math 三个驱动子目录）、mm/（内存管理 + 缺页）、fs/（17 个文件，MINIX 文件系统 + buffer cache）、lib/（内核用的 C 库子集）、include/（内核头文件）、tools/（build.sh 拼引导镜像）
- 技术栈：C（gcc 1.40 时代语法）+ AT&T 汇编；无外部服务/网络；数据输入只有 hdc-0.11.img 磁盘镜像
- 外部依赖：qemu（运行）、hdc-0.11.img（根文件系统，直接下载使用，不手搓）
