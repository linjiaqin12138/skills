# M3 验收档案（中断：LAPIC 定时器）

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

**架构决定**：`decisions/0001-interrupt-context-minimal-work.md`（中断上下文只做计数与 EOI）、`decisions/0002-lapic-mmio-mapping.md`（按需补一页 MMIO 映射），状态均为「起草」，待用户确认本 Bite 后转「已采纳」。

**架构图**：`book/arch/current.svg`（顶层运行时路径）、`book/m03-interrupts/arch.svg`（本课副本）、`book/m03-interrupts/figures/arch-diff.svg`（M2→M3 变动）、`book/m03-interrupts/figures/lapic-mmio.svg`（MMIO 映射结构单独成图）。图源为 `book/arch/src/current.json` 与 `book/m03-interrupts/figures/src/*.json`；fireworks validate / render / check 均通过。

**教程与概念页复验（2026-10-28 按现行 writer / concept-writer 重写）**：源稿 `b01-lapic-timer.md`（由 `.draft.md` 迁入）strict 渲染 0 警告，8 个面板（问题 / 背景 / 设计 / 架构决定 / 代码领读 / 验收含 raw qemu 命令 / 与原版差异与坑 / 自测 5 题）。概念页 `apic-vs-pic`、`x86-msr`、`hhdm`、`mmio-page-mapping` 均含「回到项目」用点对照表；「去学」仅作深入。
