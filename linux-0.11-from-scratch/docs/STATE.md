# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M0 已完成**（Bite1 能编译 + Bite2 能启动，验收档案 `docs/milestones/m0-boot/acceptance.md`，2026-10-09 主 agent 复跑通过）
- 教程：Bite1 `docs/milestones/m0-boot/tutorial.md`（图引用 Bite1 时点快照 arch-bite1.svg）；Bite2 `docs/milestones/m0-boot/tutorial-bite2-iso-boot.md`；概念文档 `docs/concepts/boot-from-power-on.md`（phase-2 卡片 A 承诺，已兑现）
- 环境：qemu 10.2.1✅；工具链镜像✅；`make build && make check` 一键开发循环已通（串口断言 `M0: kernel alive`）

## 下一步动作
用户确认 M0 后进入 **M1「内核 printk」**（phase-2.md M1 行）：Limine framebuffer + 自绘 8x16 字体渲染 + 滚屏控制台 + `core::fmt` 格式化打印；验收 = 屏幕多行格式化文本（截图目检）+ 串口同内容日志（grep 断言）。M0 遗留认知：Limine 的页表是过渡用品，内存管理里程碑（M4 起）自建页表接管；UEFI 引导路径已配置未实测【合理推断：宿主机未确认 OVMF】。
Bite2 期间的新经验：①`limine bios-install` 服务的是 isohybrid/硬盘引导路径，纯光盘 El Torito 实测不依赖（破坏实验证实，build.mk 注释已订正）；②qemu 同时挂硬盘+光盘时默认硬盘优先，`-boot d` 已固化进 Makefile。

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
- Rust 语法书（检索用，只读）: `reference/book-cn/`（浅克隆 rust-lang-cn/book-cn；语法卡片出处链接写在线版 https://rustwiki.org/zh-CN/book/，`src/*.md` 文件名与 URL 一一对应；书上没讲的链官方英文文档并标注）
- 原项目构建: `cd reference && make && make start`（需要 qemu + hdc-0.11.img，img 已在仓库根目录）
- 最新架构图: `docs/arch/original.svg`（原项目总图，d2 画的，保留作历史）；**本工程当前架构权威图：`docs/milestones/m0-boot/assets/dst/arch.svg`（M0 完结版）**；从 M0 起本工程架构图一律用 fireworks-tech-graph
- **画图工具链（2026-10-09 订正，后续画图 subagent 必须沿用）**：按 `skills/draw-diagram` 路由——时序/流程嵌 mermaid；架构图默认 fireworks-tech-graph，**脚本在 `vendor/fireworks-tech-graph/`（工作区 skills 仓库根下，2026-10-09 从上游 clone；原 `skills/fireworks-tech-graph` 路径已失效，`~/.claude/skills/` 下是工作流不同的 npm 旧版，勿用）**。工作流：`python3 vendor/fireworks-tech-graph/scripts/fireworks.py validate/render/check/export-html`；几何问题靠 `check` 文本报告修，**定稿时才渲染 PNG 目检一次**。模型目检：先 `sed` 把 SVG 的 font-family 改成 `'Noto Sans CJK SC', sans-serif`，再 `uv run --with cairosvg python3 -c "import cairosvg; cairosvg.svg2png(...)"`（cairosvg **并未** pip --user 装好、系统 python 无 pip，用 uv；firefox headless 勿用）。风格用 Style 1 或 4（浅色）；**Style 1 实际配色 control=紫/write=绿/data=橙**（与 references 文档描述不符，沿用此经验）；fireworks arrow color 覆盖不改 marker 色；`.section` CSS 类色不随容器 stroke，改容器标签色要改 CSS 规则（export-html 拒绝 text 上的内联 style）。arch-diff = 复制同一份 JSON 只改颜色，与全图天然像素对齐；删除模块没有虚线框字段，用灰填充+灰描边代替
- Phase 1 拆解文档: `docs/phase-1.md`（模块契约 + 5 张决策卡片 + 概念词汇表）
- Phase 2 路线图: `docs/phase-2.md`（16 里程碑 + 3 张项目级决策卡）
- **功能清单: `docs/feature-inventory.md`**（铁律 8 核对依据，P0/P1/STUB/FORK 四级，Phase 4 逐项过）。计数：**42 项，已交付 1 / 未交付 41**（M0 交付 §1「上电→长模式→main」）
- 用户偏好: `skills/clone-from-scrach-tutor/user-prefs.md`（TS 背景；不主动 TS 类比，先直讲、必要时生活类比；Rust 零基础要教语法；正文主干要短；重"为什么"与失效边界、破坏实验）

## 偏差登记簿计数
- docs/deviations.md：11 条——待收敛 3（D2/D3/D6）、永久豁免·架构 7（D1/D4/D7/D8/D9/D10/D11）、待裁决 1（D5 在 Phase 4）

## 本机环境侦察（2026-10-03 补，Phase 0 原本漏了这层）

**工具链架构（2026-10-03 用户定）：编译全部在 docker 容器（`linux011-rust-toolchain`，见 docker/Dockerfile），qemu 在宿主机跑**（显示/键盘不便容器化）。宿主机唯一要装的是 `sudo apt install qemu-system-x86`（xorriso/gcc/make 都在容器里了）。工作区根 Makefile 已备好：`make toolchain / kernel / build / run / debug`。**构建容器常驻复用（2026-10-07 改）**：`kernel`/`build` 依赖 `ctr` 目标自动完成"镜像不存在→build、容器 `linux011-rust-build` 不存在→create、没在跑→start"，之后一律 `docker exec` 进同一容器编译，不再每次新建。宿主机早前装的 rustup 保留给编辑器/rust-analyzer 用，不再是权威构建环境。**rootless docker 两个坑（2026-10-07 踩实）**：①`--network host` 不生效，容器不共享宿主机网络，代理穿透无从谈起——默认无代理；②`docker run` 不要加 `-u $(id -u)`，容器内非 root uid 经 subuid 映射后在宿主机不是本用户，写挂载卷 Permission denied——容器内 root 恰好映射回宿主机当前用户，产物归属正确。

| 依赖 | 状态 | 解决方式 |
|---|---|---|
| Rust 工具链 | **容器镜像 `linux011-rust-toolchain` 已构建并冒烟验证**（rustc 1.100.0-nightly / x86_64-unknown-none / limine 10.8.5 / xorriso 1.5.6 / make 4.4.1） | `make toolchain` 可重建 |
| Limine 引导器 | 已克隆 reference/limine（v10.8.5），容器内编译其部署工具 | 下载：`git clone --depth 1 -b v10.8.5-binary https://github.com/limine-bootloader/limine.git reference/limine`。**注意：官方已废弃 `binary` 分支，改用 `vX.Y.Z-binary` tag 发布**；Dockerfile 里 COPY + make |
| qemu-system-x86_64 | ✅ 已装（10.2.1），`make run`/`make check` 均实测通过 | — |
| xorriso / gcc / make | 容器内 | 无需宿主机安装 |
| 网络 | **默认无代理**（2026-10-07 起；Makefile/Dockerfile 均不带 proxy 配置）。docker 为 **rootless 模式**：`--network host` 不生效，容器不共享宿主机网络，容器内 `127.0.0.1` 指向容器自己 | **docker 守护进程拉取镜像可能受限**；基础镜像可用 daocloud 镜像站拉取后 retag：`docker pull docker.m.daocloud.io/rustlang/rust:nightly-slim && docker tag … rustlang/rust:nightly-slim` |
| 硬件资源 | 8 核 / 414G 可用 | 充足 |

**结论：环境零阻塞——编译在容器、qemu 在宿主机，`make build && make check` 全链路已通（2026-10-09）。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载软盘/硬盘引导扇区 → 启动一个 i386 保护模式、支持多进程 + MINIX 文件系统 + tty 终端的单内核 OS
- 主循环位置：`init/main.c:106` main() —— 初始化内存/中断/设备/调度后 `move_to_user_mode()`，fork 出 init 进程跑 /bin/sh，task0 自己 `for(;;) pause()`
- 目录地图：boot（3 个汇编：bootsect/setup/head）、init/main.c、kernel/（调度/信号/系统调用/异常 + blk_drv/chr_drv/math 三个驱动子目录）、mm/（内存管理 + 缺页）、fs/（17 个文件，MINIX 文件系统 + buffer cache）、lib/（内核用的 C 库子集）、include/（内核头文件）、tools/（build.sh 拼引导镜像）
- 技术栈：C（gcc 1.40 时代语法）+ AT&T 汇编；无外部服务/网络；数据输入只有 hdc-0.11.img 磁盘镜像
- 外部依赖：qemu（运行）、hdc-0.11.img（根文件系统，直接下载使用，不手搓）
