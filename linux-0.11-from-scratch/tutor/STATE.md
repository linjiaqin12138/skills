# STATE — Linux 0.11 from scratch（Rust 重写）

## 当前位置
- Phase: 3（逐层手搓）进行中
- 里程碑: **M3 进行中**——Bite1（LAPIC 100Hz）、Bite2（IOAPIC 键盘 + IrqLock）与 **Bite3（ACPI MADT/ISO 硬件发现）** 均已验收通过（`accept/m03-b01.sh` / `m03-b02.sh` / `m03-b03.sh`）。M0/M1/M2 已通过
- 教程：M0～M2 既有；M3 最新 **Bite3** `book/m03-interrupts/b03-acpi-madt.html`（8 面板）。新概念 `acpi-madt`；`apic-vs-pic` / `ps2-keyboard` 同步更新
- 环境：同前——`~/.local/bin/make` / `qemu-system-x86_64`；工具链镜像 `linux011-rust-toolchain`；Limine v10.8.5-binary；hdc stub 1MB

## 下一步动作
**停下，等用户确认本 Bite（M3 Bite3）。** 确认后再定 M3 是否收尾或进入 M4。
- 当前验收：`bash accept/m03-b03.sh`
- D16 剩余（向量 34~255、异常 iret）→ M5/M9；D17 MMIO 窄映射 → M5；D3 行规程 → M13
- 知识依赖黑盒回访：写 `0x64` 发 PS/2 命令（需要时）
- **子代理模型**：不指定 model，跟主会话；勿默认绑 Claude（用户 2026-10-29）

**同目录多 Bite 图约定（2026-10-29）**：权威 `arch.svg` / `figures/arch-diff.svg` 跟最新 Bite；旧 Bite 切片另存 `arch-b01.svg` / `arch-diff-b01.svg`，旧课文改链。禁止对已演进代码的旧课整页 `am-render`（`src=` 会漂移）。

**教程工作流**：源稿 `.md` → `bin/am-render`（`--style strict`，`-o` 同名 html）。禁止 `am patch`、禁止为修链接而重渲会漂移的旧课（可外科改 HTML 链接）。

## 项目固定决策（v2，2026-10-03）
- **目标架构：x86_64**；Limine v10；Rust `x86_64-unknown-none`
- **用户态：自写 Rust mini-sh + coreutils-lite**；hdc 仍作 MINIX 硬验收
- **syscall ABI：syscall**；**上下文切换：软件**；**内存：每进程页表 + HHDM**
- **实现语言：Rust**；用户零基础就地教；代码尽量优雅易懂
- 终态：`spec/features.md`

## 关键路径
- upstream / book-cn / phase-1·2：同前
- **当前架构权威图：`book/arch/current.svg`（M3 Bite3）**；本课 `book/m03-interrupts/arch.svg`；diff `figures/arch-diff.svg`（Bite2→Bite3）；Bite1/Bite2 历史切片已分别保存；MMIO 子图 `figures/lapic-mmio.svg`
- **功能清单**：§1 M0 已交付；§6 tty 推进至键盘回显（行规程待 M13）；§2 调度仍仅有 100Hz tick 基础设施
- 偏好：`~/.clone-from-scratch-tutor/user-prefs.md` + `tutor/user-prefs.md`

## 偏差登记簿计数
- spec/deviations.md：**19 条**——待收敛 6（D2/D3/D6/D16/D17/D19）、永久豁免·架构 9（D1/D4/D7–D11/D13/D14）、待裁决 1（D5）、已收敛 4（D12/D15/D18 及 D12 历史）。D18 已于 M3 Bite3 收敛；D19 随 D3/M13

## 本机环境侦察（续）

| 依赖 | 状态 |
|---|---|
| GNU Make / qemu / Limine / fireworks | ✅ 同前 |
| docker 构建 `linux011-rust-build` | ✅ |

**结论：M3 Bite3 全链路已通（`make -C impl build && bash accept/m03-b03.sh`）；Bite2 回归通过。**

## Phase 0 侦察结论摘要
- 一句话本质：BIOS 加载引导 → i386 保护模式多进程 + MINIX + tty（我们走 x86_64+Limine 现代化路径）
- 主循环：原版 `init/main.c` fork sh；我们的路径按里程碑演进
- 技术栈对照与 hdc 下载策略：见既有 Phase 0 记录
