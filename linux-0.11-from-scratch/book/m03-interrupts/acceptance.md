# M3 验收档案（中断：LAPIC 定时器 + IOAPIC 键盘）

## Bite3：ACPI MADT / ISO 硬件发现（2026-10-30 通过）

**验收命令**：`bash accept/m03-b03.sh`（exit 0；内部 `make check-m3-b3`）；`bash accept/m03-b02.sh` 回归也通过。

**涉及代码文件**：`acpi.rs`（新建）；`keyboard.rs` / `ioapic.rs` / `mmio.rs` / `main.rs` / `Makefile`（改）。

**实测**：串口报告 `MADT IOAPIC base=0xfec00000 IRQ1 -> GSI 1 polarity=high trigger=edge`；该值来自 RSDP→XSDT→MADT，不是硬编码。连续 `hello`、jiffies 100/200/300/400、无 panic / 异常；screendump 1,440,015 字节。

**架构决定**：`decisions/0005` 已采纳，取代 0004。

**架构图**：Bite3 `arch.svg` 与 `figures/arch-diff.svg`；Bite2 历史切片 `arch-b02.svg` / `figures/arch-diff-b02.svg`。

**教程**：`b03-acpi-madt.html`；概念页新增 `acpi-madt`。

**偏差**：D18 已收敛；D17 静态页表池扩至 8，仍待 M5。

## Bite2：IOAPIC 键盘 + console 锁（2026-10-29 通过）

**验收命令**：`bash accept/m03-b02.sh`（exit 0；内部 `make check-m3-b2`）

**涉及代码文件**：`irq_lock.rs`、`mmio.rs`、`ioapic.rs`、`keyboard.rs`（新建）；`console.rs` / `idt.rs` / `lapic.rs` / `main.rs` / `Makefile`（改）

**实测**：独立 verifier 3/3 通过；串口含连续 `hello`；jiffies 样本 4 条/次；无 panic；screendump 1,440,015 字节。记录：[tutor/runs/m03-b02.txt](../../tutor/runs/m03-b02.txt)

**架构决定**：`decisions/0003`、`0004`（状态起草，待用户确认后转已采纳）

**架构图**：`book/arch/current.svg`（Bite2 顶层）、本课 `arch.svg`、`figures/arch-diff.svg`（Bite1→Bite2）。Bite1 历史切片另存 `arch-b01.svg` / `figures/arch-diff-b01.svg`。

**教程**：`b02-keyboard-console-lock.html`；概念页 `apic-vs-pic`（更新）、`irq-critical-section`、`ps2-keyboard`。

**偏差**：D15 已收敛；D16 部分再收敛；D17 备注更新；新增 D18/D19。

## Bite1：LAPIC 定时器 IRQ32 @100Hz（2026-10-26 通过）

**验收命令**：`bash accept/m03-b01.sh`（exit 0；内部执行 `make check-m3-b1`）

**涉及代码文件**：
- `impl/kernel/src/lapic.rs`（新建）：LAPIC 使能、PIT channel 2 校准、100Hz 周期定时器、legacy PIC 屏蔽、MMIO 窄映射、AtomicU64 jiffies
- `impl/kernel/src/idt.rs`（向量 32 独立可返回入口：保存/恢复 15 个通用寄存器后 `iretq`；M2 异常路径不变）
- `impl/kernel/src/main.rs`（正常启动路径去掉故意 panic；普通上下文按 100 tick 观察并打印 jiffies）
- `impl/Makefile`（`check-m3-b1`）

**实测结果**：M0/M1/M2 回归 marker 全部保留；`M0: kernel alive` 恰好一次；无 panic、无 EXCEPTION；串口与屏幕同时出现 `M3: LAPIC timer 100 Hz` 与 `M3: jiffies=100/200/300/400`；`build/m3-bite1.ppm` 为 1,440,015 字节的完整 800×600 帧。

**独立验收强度**：verifier 连跑三次验收脚本全部通过，另加两次宿主时钟对照测量——600 jiffies / 6.014 s = 99.77 Hz，600 jiffies / 6.031 s = 99.49 Hz，逐段间隔 0.991~1.015 s。脚本自身的样本数区间（6 秒窗口 3~8 个样本）只能把频率框在约 50~133 Hz，真正证明频率的是这两次宿主时钟测量。verifier 还解码了 PPM 字形，确认 jiffies 行确实画在 framebuffer 上。

**验收记录**：[tutor/runs/m03-b01.txt](../../tutor/runs/m03-b01.txt)

**覆盖缺口（记录在案，未修）**：脚本未直接断言频率、只检查截图大小未检查内容；MMIO 映射分支在日志中不可观测；寄存器保存/恢复只有间接证据；本次构建为增量编译。

**架构决定**：`decisions/0001`、`0002` 已在进入 Bite2 时改为「已采纳」。

**架构图（Bite1 历史）**：`arch-b01.svg`、`figures/arch-diff-b01.svg`（M2→Bite1）；MMIO 子图 `figures/lapic-mmio.svg`。权威 `current.svg` / `arch.svg` / `arch-diff.svg` 已演进为 Bite2。

**教程与概念页复验（2026-10-28 按现行 writer / concept-writer 重写）**：源稿 `b01-lapic-timer.md`（由 `.draft.md` 迁入）strict 渲染 0 警告，8 个面板（问题 / 背景 / 设计 / 架构决定 / 代码领读 / 验收含 raw qemu 命令 / 与原版差异与坑 / 自测 5 题）。概念页 `apic-vs-pic`、`x86-msr`、`hhdm`、`mmio-page-mapping` 均含「回到项目」用点对照表；「去学」仅作深入。
