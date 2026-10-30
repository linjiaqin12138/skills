# 0004 IOAPIC 固定基址路由键盘 IRQ1→向量 33

- 状态：被取代
- 来源：手搓
- 出现于：m03-b02
- 被取代于：0005

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

键盘是外设 IRQ，不能像 LAPIC 定时器那样走本地 LVT。M3 已整片屏蔽 8259 PIC，必须经 IOAPIC 把 IRQ1 投到本核 LAPIC，再进 IDT。QEMU 默认平台上 IOAPIC 物理基址为 `0xFEC00000`，键盘为 ISA IRQ1；本 Bite 只要扫描码→ASCII 回显（D3），不要行规程。

## 备选

1. 重新启用 8259 PIC 接键盘：改动面小，但与 Bite1「屏蔽 PIC、走 APIC」路线冲突，也教不出 IOAPIC。
2. 解析 ACPI MADT / Limine 拓扑请求动态找 IOAPIC 与 GSI：可移植性好，但本 Bite 验收只跑 QEMU 默认机，解析量远超「打通回显」。
3. 固定 IOAPIC MMIO `0xFEC00000`，把 IRQ1 配成 Fixed 投递、向量 33；PS/2 数据口 `0x60`/`0x64` 读扫描码；handler 内映射 make code→ASCII 并 `printk!`，再写 LAPIC EOI。

## 决定

选方案 3。向量 33 紧挨定时器 32，IDT 再装一条可返回入口（与 timer 共享保存/恢复/`iretq` 骨架，禁止复制两套入口逻辑）。IOAPIC 页同样经现有 MMIO 补映射路径接入 HHDM。只处理通码（make），忽略断码与 E0 前缀扩展键；Shift 等修饰键本 Bite 不做。

## 放弃的成本

方案 1 少碰 IOAPIC 寄存器，上手更快；我们放弃这份捷径，换来与 Bite1 同一套 APIC 故事，并兑现概念页里「键盘走 IOAPIC」的承诺。方案 2 换到真机或非默认 QEMU 机型时更稳；我们放弃这份可移植性，把 MADT/拓扑发现留给以后设备增多时再做。

## 失效边界

目标机 IOAPIC 不在 `0xFEC00000`、键盘不在 GSI 1、或需要多 IOAPIC / 中断重定向表动态配置时，必须改为查表发现。进入行规程、修饰键、或把回显移出中断上下文时，handler 职责要重划，不在本决定范围。
