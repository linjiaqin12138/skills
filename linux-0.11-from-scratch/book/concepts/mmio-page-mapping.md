# map_mmio_page：把设备的一页映射进虚拟地址空间

> 概念深读（M3 Bite1 背景卡片承诺）。面向看得懂四级分页、但没见过「运行时改页表」的读者。
> 读完你应能回答：为什么 CPU 查页表用物理地址、而 Rust 指针是虚拟地址；补一个 MMIO 映射到底改了几处内存。

> 交互版（推荐）：[figures/mmio-page-map/index.html](figures/mmio-page-map/index.html)
>

## 一句话版本

`map_mmio_page` 做的事只有一件：**在当前页表里，为「设备所在的那一页物理地址」补出一条映射**。补完返回这个虚拟地址，之后访问设备寄存器就退化成普通的 load/store。

它改的内存有三处，缺一不可：

| 改什么 | 例子 | 为什么必须改 |
|---|---|---|
| 上级表里的一个表项 | `PML4[256] = 0x223003` | 让 CPU 查表时能走到下一层 |
| 叶子表项 | `PT[0] = 0xFEE0001B` | 最后一跳，指向设备的物理页 |
| （必要时）新页表本身 | `EXTRA_PAGE_TABLES[0..2]` | 缺失的层级要有表可查 |

## 为什么需要搞懂它

LAPIC 的寄存器不是普通内存。CPU 里有一块 MMIO 空间，寄存器就在那儿；它的物理地址从 MSR `IA32_APIC_BASE` 读出来，不在 RAM 里。Limine 启动时给的 HHDM（高半区直接映射）只覆盖 RAM，不覆盖这块设备区。于是「拿到物理地址」和「能用指针访问它」之间，隔着一整棵页表。

最省事的一条路由决策 [0002 按需补 LAPIC MMIO 页映射](../../decisions/0002-lapic-mmio-mapping.md) 选定。沿用 Limine 那份 CR3 和页表树，只补缺失的层级和最后一项。不做通用页分配器。

## 动画：12 步看完全过程

交互版把这件事拆成 12 步。每一步同时显示三样东西：虚拟地址侧、页表侧、物理内存侧。外加一行「本步改动了什么」，以及一份累积的改动记录。

按 <kbd>←</kbd> <kbd>→</kbd> 单步，或点播放。整条链是这样的：

```text
虚拟地址 0xFFFF8000FEE00000
   │ 拆成 L4=256 L3=3 L2=503 L1=0
   ▼
PML4[256] ──► PDPT[3] ──► PD[503] ──► PT[0] ──► 物理页 0xFEE00000（LAPIC 寄存器）
   写 0x223003   写 0x224003   写 0x225003   写 0xFEE0001B
```

### 三个关键点

**1. 页表自己也在物理内存里。** CPU 查表时手上是物理地址，Rust 指针却是虚拟地址。所以每走下一级，都要靠 HHDM 把「下一级表的物理地址」换算成能写的指针：

```rust
let entry_ptr = (hhdm_offset + table_physical + index as u64 * 8) as *mut u64;
```

`hhdm_offset` 加上物理地址，直接就是这一页在虚拟空间里的直映射位置——这也正是 HHDM 存在的意义。

**2. 中间的表项指向表，最后一个表项指向数据。** 前三级表项存「下一级页表的物理地址」，叶子表项存「目标物理页的地址」。两边都带 Present / Writable 等标志位。少写一位，下一次访问就是 `#PF`。

**3. 叶子表项要关缓存。** 设备寄存器不是内存，缓存住它的值等于对硬件撒谎。所以叶子表项额外带上 `PAGE_WRITE_THROUGH | PAGE_CACHE_DISABLE`：

```rust
physical_base | PAGE_PRESENT | PAGE_WRITABLE | PAGE_WRITE_THROUGH | PAGE_CACHE_DISABLE
```

改完还要 `invlpg [virtual_base]`，让 TLB 里可能残留的旧翻译作废，否则 CPU 会继续用「这一页没映射」的旧结论。

## 静态页表从哪来

补层级用的三张表不是动态申请来的。它们是内核 BSS 里的静态数组：

```rust
#[repr(C, align(4096))]
struct PageTable([u64; 512]);

static mut EXTRA_PAGE_TABLES: [PageTable; 3] = [const { PageTable([0; 512]) }; 3];
```

BSS 里的一张页，虚拟地址减去内核虚拟基址、加上内核物理基址，就得到它的物理地址。这是静态内核镜像里做虚拟→物理换算的标准办法：

```rust
let new_physical = table_virtual - kernel_virtual + kernel_physical;
```

## 常见误解

| 常见以为 | 实际 |
|---|---|
| 有了 HHDM 就不用管页表 | HHDM 只覆盖 RAM，设备页不在其中 |
| 映射是「开一个开关」 | 是改页表里的内存：写表项 + 可能的 `invlpg` |
| 访问 MMIO 和读内存一样 | 指令一样，但必须关缓存，否则读到的是缓存值 |
| 这段逻辑能一直用 | 决策 0002 写明：进入通用分页后应换成正式页表接口 |

## 深入阅读

- 实现：`impl/kernel/src/lapic.rs:146-191`（`map_mmio_page`）
- 决策：[0002 按需补 LAPIC MMIO 页映射](../../decisions/0002-lapic-mmio-mapping.md)
- 前置概念：[x86 的 MSR：型号特定寄存器](x86-msr.html)（LAPIC 物理基址从哪来）
- 课程：[M3 Bite1 · LAPIC 定时器](../m03-interrupts/b01-lapic-timer.html)
