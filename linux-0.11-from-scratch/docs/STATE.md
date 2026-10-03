# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M0 进行中**（共 2 个 Bite）
- Bite: **M0-Bite1「能编译的内核」已交付（待用户确认）**；教程 `docs/milestones/m0-boot/tutorial.md`（骨架）+ arch.svg/arch-diff.svg 已落盘
- 环境：qemu 10.2.1✅；工具链镜像✅

## 下一步动作
用户确认 Bite1 后进 **M0-Bite2「能启动的内核」**：limine.conf + xorriso ISO（补 docker/build.mk）+ `make run` 串口断言 `M0: kernel alive`。Bite2 收尾时 writer 需：整合全章、补 acceptance.md、产出概念文档 docs/concepts/boot-from-power-on.md（phase-2 卡片 A 承诺）、顺手核对 limine base revision 3 的支持范围（教程坑 4 有【合理推断】标注）。
已知小瑕疵（不阻塞）：arch-diff.svg 里 KERNEL 容器标签仍是蓝色（其余全绿）；writer 发现 fireworks arrow color 覆盖不改 marker 色，Style 1 实际配色 control=紫/write=绿/data=橙（与 references 文档描述不符）——后续画图 subagent 沿用此经验。

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
- reference（只读对照）: `linux-0.11-from-scratch/reference/`（浅克隆 https://github.com/yuan-xy/Linux-0.11，~13.4k 行 C/汇编/头文件）
- 原项目构建: `cd reference && make && make start`（需要 qemu + hdc-0.11.img，img 已在仓库根目录）
- 最新架构图: `docs/arch/original.svg`（原项目总图，d2 画的，保留作历史）；**从 M0 起本工程架构图一律用 fireworks-tech-graph**
- **画图工具链（2026-10-03 定稿，后续画图 subagent 必须沿用）**：按 `skills/draw-diagram` 路由——时序/流程嵌 mermaid；架构图默认 fireworks-tech-graph（`skills/fireworks-tech-graph`，声明式 JSON 手动坐标）。工作流：`python3 skills/fireworks-tech-graph/scripts/fireworks.py validate/render/check/export-html`；几何问题靠 `check` 文本报告修，**定稿时才渲染 PNG 目检一次**。模型目检：先 `sed` 把 SVG 的 font-family 改成 `'Noto Sans CJK SC', sans-serif`，再 `python3 -c "import cairosvg; cairosvg.svg2png(...)"`（cairosvg 已 pip --user 装好；firefox headless 本机不稳定，勿用）。风格用 Style 1 或 4（浅色）。arch-diff = 复制同一份 JSON 只改颜色，与全图天然像素对齐；删除模块没有虚线框字段，用灰填充+灰描边代替
- Phase 1 拆解文档: `docs/phase-1.md`（模块契约 + 5 张决策卡片 + 概念词汇表）
- Phase 2 路线图: `docs/phase-2.md`（16 里程碑 + 3 张项目级决策卡）
- **功能清单: `docs/feature-inventory.md`**（铁律 8 核对依据，P0/P1/STUB/FORK 四级，Phase 4 逐项过）
- 用户偏好: `skills/clone-from-scrach-tutor/user-prefs.md`（TS 背景；不主动 TS 类比，先直讲、必要时生活类比；Rust 零基础要教语法；正文主干要短；重"为什么"与失效边界、破坏实验）

## 偏差登记簿计数
- docs/deviations.md：11 条——待收敛 3（D2/D3/D6）、永久豁免·架构 7（D1/D4/D7/D8/D9/D10/D11）、待裁决 1（D5 在 Phase 4）

## 本机环境侦察（2026-10-03 补，Phase 0 原本漏了这层）

**工具链架构（2026-10-03 用户定）：编译全部在 docker 容器（`linux011-rust-toolchain`，见 docker/Dockerfile），qemu 在宿主机跑**（显示/键盘不便容器化）。宿主机唯一要装的是 `sudo apt install qemu-system-x86`（xorriso/gcc/make 都在容器里了）。工作区根 Makefile 已备好：`make toolchain / build / run / debug`。宿主机早前装的 rustup 保留给编辑器/rust-analyzer 用，不再是权威构建环境。

| 依赖 | 状态 | 解决方式 |
|---|---|---|
| Rust 工具链 | **容器镜像 `linux011-rust-toolchain` 已构建并冒烟验证**（rustc 1.100.0-nightly / x86_64-unknown-none / limine 10.8.5 / xorriso 1.5.6 / make 4.4.1） | `make toolchain` 可重建（走代理 --network host） |
| Limine 引导器 | 已克隆 reference/limine（v10 binary），容器内编译其部署工具 | Dockerfile 里 COPY + make |
| qemu-system-x86_64 | **缺，需 sudo apt** | `sudo apt install qemu-system-x86`——用户亲手跑；兜底 bochs（慢） |
| xorriso / gcc / make | 容器内 | 无需宿主机安装 |
| 网络 | 代理 127.0.0.1:7890（docker build 用 --network host 穿透） | **docker 守护进程拉取走不了代理**（需 sudo 改 systemd，不做）；基础镜像用 daocloud 镜像站拉取后 retag：`docker pull docker.m.daocloud.io/rustlang/rust:nightly-slim && docker tag … rustlang/rust:nightly-slim` |
| 硬件资源 | 8 核 / 414G 可用 | 充足 |

**结论：唯一阻塞项是宿主机的 qemu；编译环境docker化后其余零安装。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载软盘/硬盘引导扇区 → 启动一个 i386 保护模式、支持多进程 + MINIX 文件系统 + tty 终端的单内核 OS
- 主循环位置：`init/main.c:106` main() —— 初始化内存/中断/设备/调度后 `move_to_user_mode()`，fork 出 init 进程跑 /bin/sh，task0 自己 `for(;;) pause()`
- 目录地图：boot（3 个汇编：bootsect/setup/head）、init/main.c、kernel/（调度/信号/系统调用/异常 + blk_drv/chr_drv/math 三个驱动子目录）、mm/（内存管理 + 缺页）、fs/（17 个文件，MINIX 文件系统 + buffer cache）、lib/（内核用的 C 库子集）、include/（内核头文件）、tools/（build.sh 拼引导镜像）
- 技术栈：C（gcc 1.40 时代语法）+ AT&T 汇编；无外部服务/网络；数据输入只有 hdc-0.11.img 磁盘镜像
- 外部依赖：qemu（运行）、hdc-0.11.img（根文件系统，直接下载使用，不手搓）
