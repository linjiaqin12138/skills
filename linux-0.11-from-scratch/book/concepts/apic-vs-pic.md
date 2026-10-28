---
template: doc
theme: paper
title: PIC、LAPIC 与 IOAPIC
subtitle: 谁产生中断、谁转发、谁进 IDT；本课只用 LAPIC 定时器
source: x86-64 概念深读
---

Lead: 8259 PIC 汇集传统外中断。LAPIC 是每颗 CPU 本地的中断控制器，自带定时器。IOAPIC 把外设 IRQ 路由到某颗 LAPIC。本课的 100 Hz tick 由 LAPIC timer 直接产生向量 32，不经过 PIC 或 IOAPIC。

> 概念深读（M3 Bite1 背景卡片承诺）。面向第一次见到这些硬件名词的读者。
> 读完你应能回答：PIC / LAPIC / IOAPIC / PIT 各管什么；为什么本课屏蔽 PIC 仍能用 PIT 校准；IRQ 与向量号怎么对齐。

## 一句话

四个名字经常混着说，其实管四件不同的事：

| 名字 | 全称 | 一句话职责 |
|---|---|---|
| PIC | 8259 Programmable Interrupt Controller | 把传统外设 IRQ 线汇集后交给 CPU |
| LAPIC | Local APIC | 每颗 CPU 本地的中断控制器，自带定时器 |
| IOAPIC | I/O APIC | 把外设 IRQ 路由到指定 LAPIC |
| PIT | Programmable Interval Timer | 可编程定时器；本课只当 10 ms 秒表 |

**IDT 始终负责按向量号找入口。** 换的是中断从哪来、谁投递给 CPU，不是废弃 IDT。

## 四者怎么串

```flow LR
[外设] -> [IOAPIC]: 外设 IRQ
[IOAPIC] -> [LAPIC]: 路由
[LAPIC timer] -> [LAPIC]: 本地定时
[LAPIC] -> [CPU]: 向量号
[CPU] -> [IDT]: 查入口
[PIT] --> [校准秒表]: 本课不投递 IRQ
[PIC] --> [已屏蔽]: 本课不用
```

本课实际走的是短路径：`LAPIC timer → LAPIC → CPU → IDT → 向量 32`。IOAPIC 留给下一 Bite 的键盘。PIC 在初始化时整片屏蔽。

## IRQ 与向量号

**IRQ 是外设侧的线号；向量号是 CPU 查 IDT 用的下标。** 两者不必相等。

| 区间 | 谁占用 | 本课怎么用 |
|---|---|---|
| 0–31 | CPU 异常（`#DE`、`#PF` 等） | 异常入口仍打印后停机 |
| 32–255 | 可给外设 / LAPIC 本地源 | 定时器选向量 32 |

**CPU 异常与 PIC 无关。** 执行除零时，CPU 自己产生 `#DE`，固定走 **IDT[0]**。这条路径不经过 PIC / LAPIC / IOAPIC。哪怕 PIC 也被编成「IRQ0 → 向量 0」，除零仍然是异常入口；撞车的坏处是：硬件 IRQ 与异常共用同一表项，分不清来源，栈帧约定还可能不一致。

**PIC 的默认基址是 BIOS 配的，不是 0。** PC 上主片 IRQ0–7 常见落在向量 `0x08`–`0x0F`，从片在 `0x70` 起。保护模式下这正好压在 CPU 保留的异常区（0–31）里，例如 IRQ0 与 `#DF`（向量 8）同号。所以仍用 PIC 的内核会做 **remap**：把主片基址改到 32、从片到 40，让外设 IRQ 避开 0–31。

**Limine 不做这次 remap。** 协议约定：交接时 legacy PIC 各线已全部屏蔽；IOAPIC 里 Fixed / Lowest Priority 项也会被屏蔽。本课同样整片屏蔽 PIC（`lapic.rs:210-213`），定时器用 LAPIC 的 LVT 直接写成向量 32（`TIMER_VECTOR = 32`，`lapic.rs:13`）。课文里的「IRQ32」指这条定时器进 IDT 的向量号，不是 PIC 线号。

下一 Bite 键盘从外设来，才需要 IOAPIC 把键盘 IRQ 映射成某个 ≥32 的向量，再交给同一条 IDT 分派路径。

## PIC 与 PIT 不是同一个设备

```callout warn 常见混淆
屏蔽 legacy PIC 不等于停用 PIT。PIC 是中断控制器；PIT 是可编程定时器。二者独立。
```

本课初始化先屏蔽两片 8259（`outb(0x21, 0xff)` / `outb(0xa1, 0xff)`，`impl/kernel/src/lapic.rs:210-213`）。这样旧的 PIC 投递路径安静下来，避免和 LAPIC 抢向量。

校准仍用 PIT channel 2：门控端口给出已知 10 ms 脉冲，量 LAPIC 在这段时间里数了多少（`impl/kernel/src/lapic.rs:193-208`）。PIT 在这里不产生运行期 tick，也不需要 PIC 转发。

## LAPIC 本课用到什么

LAPIC 对每个 CPU 一份。寄存器住在一段物理 MMIO（基址常为 `0xFEE00000`，以 MSR `IA32_APIC_BASE` 为准）。本课用到的 offset 与位域如下：

![本课用到的 LAPIC MMIO 寄存器位域：SVR、LVT Timer、Divide、Count、EOI](figures/apic-vs-pic/lapic-registers.svg)

本课用到的动作：

1. **全局使能**：必要时置 MSR bit 11（`APIC_ENABLE`）。
2. **软件使能**：写 Spurious Vector 寄存器（偏移 `0xF0`），置 bit 8（`LAPIC_SOFTWARE_ENABLE`），低 8 位选伪中断向量 `0xFF`。
3. **映射 MMIO**：HHDM 不保证覆盖设备页，按需补一页（见 [map_mmio_page](mmio-page-mapping.html)）。
4. **校准并开周期定时器**：divider=16，LVT 写周期模式与向量 32，最后 `sti`。
5. **每次 tick**：handler 只给 `jiffies` 加一，再写 EOI（偏移 `0xB0`）。

对应入口：`enable_lapic` 与 `init`（`impl/kernel/src/lapic.rs:85-143`）。MSR 细节见 [x86 的 MSR](x86-msr.html)。

```callout info 两道使能开关
MSR 的 APIC Global Enable 与 SVR 的 Software Enable 是两道开关。只开其一，LAPIC 仍可能不接中断。本课两处都写。
```

## 本课有 / 没有

| 能力 | 本课 | 说明 |
|---|---|---|
| LAPIC timer 100 Hz | ok | 直接产生向量 32 |
| 屏蔽 legacy PIC | ok | 避免旧路径干扰 |
| PIT 作校准秒表 | ok | 不负责运行期 tick |
| IOAPIC 路由外设 | no | 留给键盘 Bite |
| PIC 投递外中断 | no | 已整片屏蔽 |

## 回到项目

| 用点 | 代码 | 本页哪里讲 |
|---|---|---|
| 屏蔽 PIC、PIT 只校准 | `lapic.rs:94`、`:193-213`、`:210-213` | PIC≠PIT；本课有/没有 |
| 向量 32 / IRQ 含义 | `lapic.rs:13`、LVT 写入 | IRQ 与向量号 |
| MSR 全局使能 + SVR 软件使能 | `lapic.rs:131-142` | LAPIC 本课用到什么 |
| timer → IDT，不经 IOAPIC | `lapic.rs:85-117` | 四者怎么串 |

更完整的课文见 [M3 Bite1 · LAPIC 定时器](../m03-interrupts/b01-lapic-timer.html)。相关概念：[x86 的 MSR](x86-msr.html)、[HHDM](hhdm.html)、[map_mmio_page](mmio-page-mapping.html)。

## 去学

想继续深入时再看。删掉本节，上面的用点仍应能对上。

1. [从零开始自制操作系统 EP8-1 外中断与APIC (P1)](https://www.bilibili.com/video/BV1mq4y1i7b4/)
   - **看哪一段：** 从 8259 PIC 过渡到 APIC 的概念讲解；寄存器细节可先略过。
   - **看完能回答：** 为什么教程用 APIC 接管外中断，而不是继续只用 8259？

2. [OSDev Notes：APIC](https://github.com/dreamportdev/Osdev-Notes/blob/master/02_Architecture/07_APIC.md)
   - **看哪一段：** “What is APIC”“Types of APIC”和“Handling Interrupts”。
   - **看完能回答：** LAPIC timer 为什么不需要经过 IOAPIC，而键盘为什么需要？

3. [OSDev Notes：Timer](https://github.com/dreamportdev/Osdev-Notes/blob/master/02_Architecture/08_Timers.md)
   - **看哪一段：** “Types and Characteristics”“Calibrating Timers”，以及“Local APIC Timer”开头。
   - **看完能回答：** PIT 不再负责周期 tick 后，为什么仍能作为校准参考？

## 自测

1. 既然 legacy PIC 已被屏蔽，为什么 `calibrate()` 仍能用 PIT channel 2？
2. 本课的「IRQ32」是外设线号，还是 IDT 向量号？它是谁写进 LVT 的？
3. 下一 Bite 要接键盘时，缺的是 PIC、LAPIC，还是 IOAPIC？为什么？
