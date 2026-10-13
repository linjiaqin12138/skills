# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M1 进行中** — **Bite1 已通过**（2026-10-12，`make check-m1-b1`；验收档案 `docs/milestones/m1-printk/acceptance.md`）；Bite2 未开始
- 教程：M1 Bite1 `docs/milestones/m1-printk/tutorial-bite1-framebuffer.html`；概念 `docs/concepts/linear-framebuffer.md`；M0 仍见 `docs/milestones/m0-boot/`
- 环境：qemu 宿主机用 `~/.local/bin/qemu-system-x86_64`（从 deb 解压，非 apt 安装；缺 ROM 时验收加 `-net none`）；工具链镜像 `linux011-rust-toolchain`✅；`reference/limine`（v10.8.5-binary）+ `reference/limine-protocol`（limine.h/PROTOCOL）已恢复；`hdc-0.11.img` 现为 1MB stub（M1 不读盘）

## 下一步动作
用户确认后进入 **M1 Bite2「printk! + 滚屏」**：`core::fmt::Write` 接到 console、`printk!` 宏双写屏幕+串口、光标到底滚屏；验收 = 串口 grep 格式化多行 + screendump 目检。本 Bite 遗留：D12（无 fmt/滚屏）待 Bite2 收敛。

**2026-10-13**：Limine 协议已改为官方 `limine` crate 0.6.5（D14）；`make check-m1-b1` 复跑通过。

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
- 最新架构图: `docs/arch/original.svg`（原项目总图）；**本工程当前架构权威图：`docs/milestones/m1-printk/assets/dst/arch.svg`（M1 Bite1）**；arch-diff：`…/arch-diff.svg`
- **画图工具链**：默认 fireworks（`vendor/fireworks-tech-graph/`，2026-10-12 本机网络未能 clone，M1 Bite1 **权威 SVG 暂用 d2**；fireworks 兼容 JSON 已落在 `m1-printk/assets/src/arch*.json`，工具可用后可重渲）。工作流见既有 STATE 经验；Style 1 配色沿用
- Phase 1 拆解文档: `docs/phase-1.md`
- Phase 2 路线图: `docs/phase-2.md`
- **功能清单: `docs/feature-inventory.md`**。计数：**42 项，已交付 1 / 未交付 41**（M0 交付 §1；M1 Bite1 推进显示路径，尚未勾完 P0 tty 整项——待 M1 完结/后续 tty 里程碑）
- 用户偏好: `skills/clone-from-scrach-tutor/user-prefs.md` + `docs/user-prefs.md`

## 偏差登记簿计数
- docs/deviations.md：**14 条**——待收敛 4（D2/D3/D6/D12）、永久豁免·架构 9（D1/D4/D7/D8/D9/D10/D11/D13/D14）、待裁决 1（D5）

## 本机环境侦察（续）

| 依赖 | 状态 | 解决方式 |
|---|---|---|
| Rust 工具链 | 容器镜像 `linux011-rust-toolchain`✅ | `make toolchain` |
| Limine | `reference/limine` v10.8.5-binary✅ | tag 克隆 |
| limine.h / PROTOCOL | `reference/limine-protocol/`✅ | 协议仓 / jsdelivr |
| qemu-system-x86_64 | ✅ `~/.local/bin` 包装（deb 解压）；系统 apt 未装（无 sudo） | PATH 含 `~/.local/bin`；BIOS 已补 SeaBIOS |
| hdc-0.11.img | 1MB stub（非原版镜像） | M10 前换回真镜像 |
| fireworks-tech-graph | ❌ 2026-10-12 未能 clone | M1 暂 d2；JSON 已备 |

**结论：M1 Bite1 全链路已通（`make build && make check-m1-b1`）。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载软盘/硬盘引导扇区 → 启动一个 i386 保护模式、支持多进程 + MINIX 文件系统 + tty 终端的单内核 OS
- 主循环位置：`init/main.c:106` main() —— 初始化内存/中断/设备/调度后 `move_to_user_mode()`，fork 出 init 进程跑 /bin/sh，task0 自己 `for(;;) pause()`
- 目录地图：boot（3 个汇编：bootsect/setup/head）、init/main.c、kernel/（调度/信号/系统调用/异常 + blk_drv/chr_drv/math 三个驱动子目录）、mm/（内存管理 + 缺页）、fs/（17 个文件，MINIX 文件系统 + buffer cache）、lib/（内核用的 C 库子集）、include/（内核头文件）、tools/（build.sh 拼引导镜像）
- 技术栈：C（gcc 1.40 时代语法）+ AT&T 汇编；无外部服务/网络；数据输入只有 hdc-0.11.img 磁盘镜像
- 外部依赖：qemu（运行）、hdc-0.11.img（根文件系统，直接下载使用，不手搓）
