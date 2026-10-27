# PIC、LAPIC 与 IOAPIC

## 一句话

8259 PIC 负责把传统外部 IRQ 汇集给 CPU；LAPIC 是 CPU 本地的中断控制器并自带定时器，因此本项目的 100 Hz tick 可以直接由 LAPIC 产生向量 32，无需经过 PIC 或 IOAPIC；IOAPIC 留给下一步，把键盘这类外部设备的 IRQ 路由到 LAPIC，而 IDT 始终负责按向量分派处理入口。

## 去学

1. [从零开始自制操作系统 EP8-1 外中断与APIC (P1)](https://www.bilibili.com/video/BV1mq4y1i7b4/)
   - **看哪一段：** 全片中从 8259 PIC 过渡到 APIC 的概念讲解；具体寄存器和实现细节可先略过。
   - **看完能回答：** 为什么这个操作系统教程用 APIC 接管外中断，而不是继续只用 8259 PIC？

2. [OSDev Notes：APIC](https://github.com/dreamportdev/Osdev-Notes/blob/master/02_Architecture/07_APIC.md)
   - **看哪一段：** “What is APIC”“Types of APIC”和“Handling Interrupts”。
   - **看完能回答：** LAPIC timer 为什么不需要经过 IOAPIC，而键盘这类外部设备为什么需要？

3. [OSDev Notes：Timer](https://github.com/dreamportdev/Osdev-Notes/blob/master/02_Architecture/08_Timers.md)
   - **看哪一段：** “Types and Characteristics”“Calibrating Timers”，以及“Local APIC Timer”的开头。
   - **看完能回答：** PIT 不再负责周期 tick 后，为什么仍能作为 LAPIC timer 的校准参考？

## 回到项目

当前 M3 Bite1 把旧的 PIC 外部中断投递路径换成 LAPIC timer 的本地路径：初始化屏蔽两片 legacy PIC，校准后把 LAPIC timer 配置为周期模式并打开 CPU 可屏蔽中断，得到每 10 ms 一次的 100 Hz IRQ32（`impl/kernel/src/lapic.rs:85-129`、`impl/kernel/src/lapic.rs:193-213`）。LAPIC 的启用与 spurious vector 设置见 `impl/kernel/src/lapic.rs:131-142`。

LAPIC 寄存器所在的那段物理地址不是写死的常数，它由 CPU 通过 MSR `IA32_APIC_BASE` 告诉内核。MSR 是什么、为什么只能用 `rdmsr` 读写，见 [x86 的 MSR：型号特定寄存器](x86-msr.html)。

每次定时器中断只把 `AtomicU64` 计数器加一，再向 LAPIC 写 EOI（End of Interrupt，中断处理完成），不在中断上下文打印或操作 console（`impl/kernel/src/lapic.rs:124-129`）。CPU 仍通过 IDT 找到处理入口：向量 32 有独立、可返回的入口，保存寄存器、调用 handler、恢复寄存器后用 `iretq` 返回（`impl/kernel/src/idt.rs:241-300`）。换的是中断来源与投递控制器，不是废弃 IDT。

**屏蔽 PIC 不等于停用 PIT，二者不是同一个设备。** PIC 是中断控制器；PIT 是可编程定时器。这里 PIT channel 2 只提供已知为 10 ms 的校准参照，用来测量 LAPIC 计数速度，不负责产生运行期的 100 Hz tick（`impl/kernel/src/lapic.rs:193-207`）。当前 Bite 不需要 IOAPIC；下一 Bite 的键盘中断来自外部设备，才由 IOAPIC 把键盘 IRQ 路由到 LAPIC，再交给既有 IDT 分派路径。

## 自测

既然 legacy PIC 已被屏蔽，为什么 `calibrate()` 仍然可以使用 PIT channel 2？
