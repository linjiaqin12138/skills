---
template: doc
theme: paper
title: ACPI MADT：固件怎样描述中断硬件
subtitle: RSDP、RSDT/XSDT、MADT 变长条目与 ISO 路由
source: x86-64 概念深读
---

Lead: 内核不能猜 IOAPIC 在哪里。Limine 提供 RSDP，RSDP 指向根表，根表指向 MADT。MADT 的 IOAPIC 项回答“寄存器在哪”，ISO 项回答“ISA IRQ 接到哪个 GSI、怎样触发”。

> 概念深读（M3 Bite3）。面向第一次见到 ACPI 表解析的读者。

## 一条发现链

```flow LR
[Limine] -> [RSDP]: 物理地址
[RSDP] -> [RSDT / XSDT]: 32/64 位表地址
[RSDT / XSDT] -> [MADT]: signature = APIC
[MADT] -> [IOAPIC 项]: type 1
[MADT] -> [ISO 项]: type 2
```

RSDP 的 revision 决定走哪条根表路径：0 使用 RSDT，条目是 32 位物理地址；2 及以上使用 XSDT，条目是 64 位物理地址。MADT 在根表中的 signature 是 `APIC`，不是字符串 `MADT`。

## 地址不是直接可用的指针

RSDP 响应和根表条目给的是**物理地址**。内核先对所在页调用 `mmio::map_page`，再加 HHDM 偏移与原页内偏移：

```rust title="acpi.rs · sketch"
let page = mmio::map_page(physical & !0xfff)?;
let pointer = (page + (physical & 0xfff)) as *const u8;
```

本机实测 RSDP 位于低 1 MiB，原 HHDM 没覆盖它；直接解引用会触发 `#PF`。这与 Bite1 的 LAPIC 页缺映射是同一类问题，D17 的通用页表改造仍在 M5。

## 两层结构

RSDT/XSDT 与 MADT 都有标准 SDT 头。头部包含 4 字节 signature 和 32 位总长度。解析器先检查长度至少覆盖头部，再验证整表所有字节之和模 256 为 0。

根表在头后排列等宽地址；MADT 在 44 字节固定区域后排列**变长条目**。每条至少以 `type: u8` 和 `length: u8` 开头，所以遍历必须同时检查：

1. 至少还剩 2 字节可读。
2. `length >= 2`，保证 offset 能前进。
3. `length` 不超过本表剩余字节。
4. 读取具体字段前，条目长度覆盖该类型所需结构。

## 本课只取两种条目

| 类型 | 名称 | 本课读取 | 回答 |
|---|---|---|---|
| 1 | IOAPIC | MMIO base、GSI start | 哪块 IOAPIC、从哪个 GSI 开始 |
| 2 | Interrupt Source Override（ISO） | source IRQ、GSI、flags | ISA IRQ 实际怎样接 |

ISO 的 flags 分为两组两位：位 0..1 表示极性，位 2..3 表示触发方式。`00` 表示沿用总线默认，`11` 分别表示低电平有效与电平触发；`10` 是保留值，本课拒绝。

如果 IRQ1 没有 ISO，才使用 ISA 默认：GSI = IRQ = 1、高电平有效、边沿触发。这个默认来自 ISA 约定，不是 QEMU 硬编码。

## 与 IOAPIC 表项对上

`KeyboardRoute` 保存 `ioapic_base`、`gsi`、`active_low` 和 `level_triggered`。`ioapic::route` 用 base 映射设备页，用 GSI 选择重定向表项，再把 flags 翻译为 IOAPIC 低 32 位的位 13 和 15。

```flow LR
[MADT / ISO] -> [KeyboardRoute]: base · GSI · flags
[KeyboardRoute] -> [IOAPIC 重定向表]: 先写目标，再写向量与属性
[IOAPIC 重定向表] -> [LAPIC]: 向量 33
```

解析失败不能退回旧地址。继续写 `0xFEC00000` 可能碰一块不存在或属于别处的 MMIO；返回错误让启动停住更安全。

## 本课边界

| 能力 | 当前状态 |
|---|---|
| RSDP checksum、RSDT/XSDT 选择 | 已实现 |
| MADT 长度/checksum 与变长条目检查 | 已实现 |
| IOAPIC 基址、IRQ1 GSI 与 flags 发现 | 已实现 |
| 无硬编码回退 | 已实现 |
| 跨页 ACPI 表 | 尚未覆盖 |
| 多 IOAPIC 完整 GSI 范围管理 | 尚未覆盖 |
| 其他设备与多核路由 | 尚未覆盖 |

当前只在 QEMU 默认机实测。那里发现的结果仍是 `0xFEC00000` / GSI 1，但来源已经是固件表。

## 回到项目

- 主入口：`impl/kernel/src/acpi.rs:25-50`
- 变长条目遍历：`acpi.rs:85-136`
- 路由应用：`impl/kernel/src/ioapic.rs:22-38`
- 课文：[M3 Bite3](../m03-interrupts/b03-acpi-madt.html)
- 决定：[0005](../../decisions/0005-acpi-madt-ioapic-discovery.md)

## 自测

**1. 根表里的 `APIC` 指什么？**

- A. MADT 的 signature
- B. IOAPIC 的物理基址
- C. CPU 的 LAPIC ID
- D. IRQ1 的向量号

```html
<details><summary>第 1 题答案</summary><p><strong>A</strong>。找到它后再按 MADT 的变长条目解析。</p></details>
```

**2. 为什么 ISO flags 中的 `00` 不等于低电平或边沿？**

- A. 它表示沿用总线默认；ISA 默认才是高电平、边沿
- B. 它表示保留值
- C. 它表示屏蔽中断
- D. 它表示多核广播

```html
<details><summary>第 2 题答案</summary><p><strong>A</strong>。本课把 <code>00</code> 保留为 false，最终采用 ISA 默认。</p></details>
```

**3. 表项长度大于剩余表长时该怎么做？**

- A. 拒绝解析，避免越过本表边界读内存
- B. 继续读到 checksum 正确为止
- C. 改用硬编码地址
- D. 把长度当作 2

```html
<details><summary>第 3 题答案</summary><p><strong>A</strong>。固件数据也是不可信输入，长度必须先校验。</p></details>
```
