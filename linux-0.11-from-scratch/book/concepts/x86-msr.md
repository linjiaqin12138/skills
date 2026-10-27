---
template: doc
theme: paper
title: x86 的 MSR：型号特定寄存器
subtitle: 既不在内存也不在 I/O 端口空间，只能用 rdmsr / wrmsr 读写的一类控制寄存器
source: x86-64 概念深读
---

# x86 的 MSR：型号特定寄存器

> 概念深读（M3 Bite1 背景卡片承诺）。面向没有寄存器背景的读者。
> 读完你应能回答：MSR 是什么、为什么不能用普通指令访问；LAPIC 的 MMIO 基址为什么要先 `rdmsr` 读出来。

## 一句话

**MSR**（Model-Specific Register）是 x86 额外提供的一类控制寄存器。它们**既不在内存里，也不在 I/O 端口空间**，只能用 `rdmsr` / `wrmsr` 两条专用指令读写。编号和每一位的含义由 CPU 厂商定义，所以叫「型号特定」。

## 它住在哪：和别的寄存器对比

| 类别 | 例子 | 怎么访问 | 属于哪个地址空间 |
|---|---|---|---|
| 通用寄存器 | RAX、RBX、RSP | `mov rax, 1` | 指令内部，不占地址空间 |
| 段寄存器 | CS、DS、SS | `mov cs, ax` | 同上 |
| 控制寄存器 | CR0、CR3、CR4 | `mov cr3, rax` | 同上 |
| **MSR** | `IA32_APIC_BASE`、`IA32_TSC` | **`rdmsr` / `wrmsr`** | **不在任何地址空间里** |
| 内存 | 变量、数组、MMIO | `mov rax, [rip+x]` | 线性地址空间 |
| I/O 端口 | COM1、PIT、8259 | `in` / `out` | I/O 端口空间（64 KiB） |

关键差别在最后两行：MSR 没有地址。你没法写 `mov rax, [0xFEE00000]` 去读一个 MSR——那读到的是内存；也没法用 `in`/`out`——那是端口空间。想碰 MSR，只能用 CPU 专门为它留的两条指令。

## 怎么读写

```asm
rdmsr                    ; ECX = MSR 编号；读出 EDX:EAX
wrmsr                    ; ECX = MSR 编号；EDX:EAX = 要写的 64 位值
```

一个 MSR 是 64 位，但指令一次只搬 32 位，所以约定用两个通用寄存器拼：

- `EAX` = 低 32 位
- `EDX` = 高 32 位

本项目把它包成了两个 Rust 函数（`impl/kernel/src/lapic.rs:215-236`）：

```rust
fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack));
    }
    ((high as u64) << 32) | low as u64
}
```

`in("ecx")` 是输入操作数（MSR 编号），`out("eax")` / `out("edx")` 是输出操作数（读出来的两半）。写的时候反过来用 `in("eax")` / `in("edx")` 把高低 32 位送回去。

## 本项目用它干了什么：LAPIC 基址

LAPIC 的寄存器住在一段固定物理地址（通常 `0xFEE00000`）。但**这个地址不是写死的常数**，而是 CPU 用一个 MSR 告诉内核的：`IA32_APIC_BASE`，编号 `0x1B`。

这个 MSR 里装着：

| 位 | 含义 |
|---|---|
| 低 32 位 | LAPIC 寄存器的物理基址（`0xFEE00000`） |
| bit 11（EN） | x2APIC 模式开关：1 = 用 MSR 访问 LAPIC 寄存器，0 = 用 MMIO 访问 |
| bit 12（XE） | x2APIC 扩展格式开关 |

于是 LAPIC 初始化是「MSR 决定 MMIO 基址，MMIO 承载实际寄存器」的两层结构：

```text
read_msr(0x1B)        →  拿到 0xFEE00000
        ↓
map_mmio_page(...)    →  把这一页映射成可访问的虚拟地址
        ↓
lapic.write(EOI, 0)   →  往虚拟地址 + 0x0B0 写 0
```

对应代码在 `impl/kernel/src/lapic.rs:131-143`（`enable_lapic`）。

## 一个已知边界：x2APIC 模式

EN=1 时，CPU 进入 **x2APIC** 模式，LAPIC 寄存器改用一组 MSR 访问（`IA32_APIC_ICR`、`IA32_APIC_LVT_TIMER` 等），此时 `0xFEE00000` 那段 MMIO 不再有效。

当前代码读 MSR 拿到基址后，**始终用 MMIO 访问 LAPIC 寄存器**，没有按 EN 位分流。硬件总是从 xAPIC 起步、且引导器不会擅自切 x2APIC，所以现在能跑通；但这是内核后续必须处理的一处边界。

## 去学

1. [Wikipedia：Model-specific register](https://en.wikipedia.org/wiki/Model-specific_register)
   - **看哪一段：** "Usage"一节，知道 MSR 编号由厂商分配、Linux 用 `msr-index.h` 集中定义。
   - **看完能回答：** 为什么说 MSR 的编号是「型号特定」的，换一款 CPU 就得查另一张表？

2. [OSDev Wiki：Model Specific Register](https://wiki.osdev.org/Model_Specific_Register)
   - **看哪一段：** 开头对 MSR 的定义，以及 `IA32_APIC_BASE`、`IA32_MISC_ENABLE` 两个例子。
   - **看完能回答：** `IA32_APIC_BASE` 的 EN 位置 1 之后会发生什么？

3. Intel SDM Volume 2（指令集参考）`RDMSR` / `WRMSR` 条目，Volume 3（系统编程指南）APIC 章节
   - **看哪一段：** SDM Vol.3 的 "x2APIC" 与 "MSR-based APIC" 段落。
   - **看完能回答：** x2APIC 为什么能突破 xAPIC 只能寻址 255 个 LAPIC 的限制？

## 回到项目

M3 Bite1 的 LAPIC 初始化就是这条链路的第一个实例：`enable_lapic` 先 `rdmsr` 读 `IA32_APIC_BASE`（`impl/kernel/src/lapic.rs:131-143`），取出物理基址后再决定是否置位 EN 位；`read_msr` / `write_msr` 两个包装在 `impl/kernel/src/lapic.rs:215-236`。

更完整的上下文见 [M3 Bite1 · LAPIC 定时器：第一个自己到来的中断](../m03-interrupts/b01-lapic-timer.html)，以及 [PIC、LAPIC 与 IOAPIC](apic-vs-pic.md)。

## 自测

1. 为什么不能用 `mov rax, [0xFEE00000]` 读 `IA32_APIC_BASE`？那个地址上到底是什么？
2. `rdmsr` 返回的 64 位值为什么要拆成 EDX 和 EAX 两个寄存器？
3. 如果 EN 位是 1，当前 `lapic.rs` 的 MMIO 访问方式还能工作吗？为什么？
