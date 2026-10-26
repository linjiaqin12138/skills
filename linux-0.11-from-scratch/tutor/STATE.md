# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M3 进行中**——Bite1（LAPIC 定时器 IRQ32 @100Hz + jiffies + 可返回中断入口）2026-10-26 独立验收 `bash accept/m03-b01.sh` 通过，记录见 `tutor/runs/m03-b01.txt`；verifier 另用宿主时钟测得 99.49~99.77 Hz。M0/M1/M2 已通过
- 教程：**M0 Bite1 `book/m00-boot/b01-boot-skeleton.html` + Bite2 `book/m00-boot/b02-iso-boot.html`（2026-10-19 已迁移到草稿+am-render 工作流）**；M1 Bite1 `b01-framebuffer.html` + Bite2 `b02-printk-scroll.html`（book/m01-printk/，尚无草稿，HTML 为唯一产物）；**M2 Bite1 `book/m02-exceptions/b01-gdt-idt.html` + Bite2 `book/m02-exceptions/b02-panic-printing.html`（源稿为同名 `.draft.md`）**；**M3 Bite1 `book/m03-interrupts/b01-lapic-timer.html`（源稿同名 `.draft.md`，8 面板）**；概念 `linear-framebuffer.md` + `gdt-long-mode.html` + `idt-exception-frame.html` + `apic-vs-pic.md`
- 环境：GNU Make 用 `~/.local/bin/make`（apt 下载后用户级解包）；qemu 宿主机用 `~/.local/bin/qemu-system-x86_64`（从 deb 解压，非 apt 安装；缺 ROM 时验收加 `-net none`）；工具链镜像 `linux011-rust-toolchain`✅；`upstream/limine`（v10.8.5-binary）+ `upstream/limine-protocol`（limine.h/PROTOCOL）已恢复；`hdc-0.11.img` 现为 1MB stub（M1 不读盘）

## 下一步动作
用户确认后进入 **M3 Bite2「IOAPIC 键盘 + console 锁」**：键盘扫描码经 IOAPIC 路由到 LAPIC，IDT 装第二个可返回外部向量；此时 handler 需要在中断上下文打印，**必须收敛 D15**（`static mut` console 换锁/临界区）。D16 剩余向量（33~255 与异常 iret）留到 M3 收尾与 M5/M9；D17 的 MMIO 窄映射在 M5 收敛。
- 当前验收命令：`bash accept/m03-b01.sh`（M3 Bite1，内部执行 `make check-m3-b1`）。`check-m2-b2` 因正常启动路径已去掉故意 panic 而仅作历史目标保留，结果见 `book/m02-exceptions/acceptance.md`。
- 2026-10-26 记录：**K3.1 Pro 配额用尽后，课文与概念页改由主模型（MiniMax M3.1）撰写**，实现与独立验收仍由 K3.1 Pro 完成。

**2026-10-13**：Limine 协议已改为官方 `limine` crate 0.6.5（D14）。

**教程/概念页工作流变更（2026-10-19，取代下条旧教训）**：草稿是唯一编辑对象，落盘为 `<页面同名>.draft.md` 与 HTML 同目录入 git；修订一律改草稿 + `bin/am-render <草稿> -o <页面>.html` 整页重渲（封装固定 `--style strict`，渲染后把内嵌图片还原为相对路径链接，`data-am-src` 保留原路径）。禁止 `am patch`、禁止手工改 HTML；验收只浏览器目检，不把 HTML 读进上下文。`src=` 代码块会跟随当前代码重渲，正文 `文件:行号` 引用须同次同步（本次已把代码领读面板的 gdt.rs 引用从 20-47/62-96 修正为 22-49/64-98，漂移源是 2026-10-19 用户加的两行 `// Q:` 注释）。

**2026-10-19 提问补录**：用户问「CS 换掉后为什么能保证新 CS 下 retfq 后面就是 mov ds 那串指令」，解答已整理成条目写进 M2 Bite1「细节展开」（三个折叠面板：返回地址是链接期常量 / 长模式 CS 不改取指地址 / 换错 CS 会 #GP 而非跑错代码），草稿已改 + `am-render` 整页重渲，Bite1 状态不变（仍待用户确认）。

**2026-10-19 M0 教程迁移**：M0 两篇老格式教程（无 frontmatter、mermaid、STE strict 不过）改写到当前规范并整页重渲——现为 `book/m00-boot/b01-boot-skeleton.md` 与 `book/m00-boot/b02-iso-boot.md`（`git mv` 保留历史），各自产出同名 `.html`。迁移内容：补 frontmatter + Lead、mermaid 换 am 的 `flow`/`sequence` 组件、新增自测面板（4 题 / 5 题）与决策卡片、代码领读改 `src=` 空代码块并逐条核对当前行号（M1/M2 已演进处保留内联块并标注漂移）、STE 修到 0 warnings。顺带修 `bin/am-render` 的去内嵌 bug：它拿 `data-am-src` 只按进程 cwd 解析相对路径，导致草稿里写相对 `figures/*` 的图仍留在 HTML 里（1.8MB），现在改成 cwd 找不到就回退到 HTML 所在目录；M2 页重渲验证无回归。

**am patch 工具教训（2026-10-15，已被上面的草稿工作流取代）**：对旧 Bite 页面执行 `am patch` 会重读所有 `src=` 引用的**当前**代码文件——若代码已被后续 Bite 改过，旧页面的代码嵌入块和 arch.svg 会漂移成新内容，与旧正文矛盾。

## 项目固定决策（v2，2026-10-03）
- **目标架构：x86_64**（用户明确选择，抛弃 i386 历史包袱）；引导器 Limine v10（已克隆 upstream/limine）；Rust target x86_64-unknown-none（官方，已装）
- **用户态：自写 Rust mini-sh + coreutils-lite**（原版 a.out 二进制在 64 位内核上物理不可执行）；hdc-0.11.img 仍挂载读数据作 MINIX 兼容硬验收
- **syscall ABI：syscall 指令 x86_64 惯例**（rax=号，rdi/rsi/rdx 传参），调用号数值对照原版 74 项表
- **上下文切换：软件切换**（x86_64 无硬件任务切换；TSS 仅留 ring3 切栈用途）
- **内存隔离：每进程独立页表 + 更高半核**（段槽位随 ISA 退役，概念进文档）
- **实现语言：Rust**（用户明确要求）；**用户 Rust 零基础**：每段代码用到的 Rust 语法就地教
- **代码风格要求（用户 2026-10-03 补充）：尽可能优雅易懂**——coder/writer 交接必须携带此要求
- 终态功能点对齐：spec/features.md 逐项过（P0 硬门槛，P1/STUB/FORK Phase 4 核对）

## 关键路径
- reference（只读对照）: `linux-0.11-from-scratch/upstream/`（limine binary + limine-protocol；Linux-0.11 源码克隆可按需补回；`.gitignore` 忽略整个 upstream/）
- Rust 语法书（检索用，只读）: `upstream/book-cn/`（若缺失可再浅克隆 rust-lang-cn/book-cn）；在线版 https://rustwiki.org/zh-CN/book/
- 原项目构建: 待 Linux-0.11 reference 补回后 `cd upstream && make && make start`
- 最新架构图: `book/arch/upstream.svg`（原项目总图）；**本工程当前架构权威图：`book/arch/current.svg`（M3 Bite1 顶层运行时路径，源 `book/arch/src/current.json`）**；本课副本 `book/m03-interrupts/arch.svg`；arch-diff：`book/m03-interrupts/figures/arch-diff.svg`（M2→M3）；M2 分层子图 `book/m02-exceptions/figures/arch-idt.svg`；M3 MMIO 结构子图 `book/m03-interrupts/figures/lapic-mmio.svg`
- **画图工具链**：默认 fireworks（`skills/fireworks-tech-graph/`）。M3 Bite1 已从 JSON 产出顶层图、diff 与 MMIO 子图，并通过 validate / render / check 与目检；M2 顶层/IDT 子图同工具链；M1 历史 SVG 仍用 d2，Style 1 配色沿用
- Phase 1 拆解文档: `tutor/phases/phase-1.md`
- Phase 2 路线图: `tutor/phases/phase-2.md`
- **功能清单: `spec/features.md`**。计数：**42 项，已交付 1 / 未交付 41**（M0 交付 §1；M1 完结推进 §6 tty 的屏幕写显路径——framebuffer+字体+滚屏+printk! 已通并注记进度，整项（含转义序列/键盘/行规程）待 M3/M13；M3 Bite1 推进 §2 调度的 100Hz tick 基础设施，键盘仍待 M3 Bite2）
- 用户偏好: `~/.clone-from-scratch-tutor/user-prefs.md` + `tutor/user-prefs.md`

## 偏差登记簿计数
- spec/deviations.md：**17 条**——待收敛 6（D2/D3/D6/D15/D16/D17）、永久豁免·架构 9（D1/D4/D7/D8/D9/D10/D11/D13/D14）、待裁决 1（D5）、已收敛 1（D12，M1 Bite2）。D16 于 M3 Bite1 部分收敛（IRQ32 可返回），D17 为新增（M5 收敛）

## 本机环境侦察（续）

| 依赖 | 状态 | 解决方式 |
|---|---|---|
| GNU Make | ✅ `~/.local/bin/make` 4.4.1 | apt 下载 deb，用户级解包，无 sudo |
| Rust 工具链 | 容器镜像 `linux011-rust-toolchain`✅ | `make toolchain` |
| Limine | `upstream/limine` v10.8.5-binary✅ | tag 克隆 |
| limine.h / PROTOCOL | `upstream/limine-protocol/`✅ | 协议仓 / jsdelivr |
| qemu-system-x86_64 | ✅ `~/.local/bin` 包装（deb 解压）；系统 apt 未装（无 sudo） | PATH 含 `~/.local/bin`；BIOS 已补 SeaBIOS |
| hdc-0.11.img | 1MB stub（非原版镜像） | M10 前换回真镜像 |
| fireworks-tech-graph | ✅ `skills/fireworks-tech-graph/` | M2/M3 已用 JSON 产出并验收多张图 |

**结论：M3 Bite1 全链路已通（`make -C impl build && bash accept/m03-b01.sh`）；M2/M1 全链路历史通过。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载软盘/硬盘引导扇区 → 启动一个 i386 保护模式、支持多进程 + MINIX 文件系统 + tty 终端的单内核 OS
- 主循环位置：`init/main.c:106` main() —— 初始化内存/中断/设备/调度后 `move_to_user_mode()`，fork 出 init 进程跑 /bin/sh，task0 自己 `for(;;) pause()`
- 目录地图：boot（3 个汇编：bootsect/setup/head）、init/main.c、impl/kernel/（调度/信号/系统调用/异常 + blk_drv/chr_drv/math 三个驱动子目录）、mm/（内存管理 + 缺页）、fs/（17 个文件，MINIX 文件系统 + buffer cache）、lib/（内核用的 C 库子集）、include/（内核头文件）、tools/（build.sh 拼引导镜像）
- 技术栈：C（gcc 1.40 时代语法）+ AT&T 汇编；无外部服务/网络；数据输入只有 hdc-0.11.img 磁盘镜像
- 外部依赖：qemu（运行）、hdc-0.11.img（根文件系统，直接下载使用，不手搓）
