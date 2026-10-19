---
template: doc
theme: paper
title: GDT 与长模式：分段还剩什么用
subtitle: 看懂段描述符、选择子、lgdt 与 retfq
source: x86-64 概念深读
---

Lead: GDT 是 CPU 查找段属性的表。长模式取消了分段隔离，但仍用描述符声明代码类型和特权级。

## 一句话版本

**GDT 是一张固定格式的表，每一行描述一个代码段或数据段。**

每行 8 字节，称为段描述符。段寄存器保存 16 位选择子。选择子指出要查哪一行。

长模式把内存隔离交给分页。GDT 仍回答两个问题：当前代码如何译码，当前特权级是什么。

## 为什么还需要它
**长模式退役的是 base/limit 检查，不是整张 GDT。**

| 机制 | 长模式中的作用 |
|---|---|
| base 与 limit | 代码段和数据段通常忽略 |
| 分页 | 负责地址转换与内存隔离 |
| 描述符 L 位 | 声明 64 位代码段 |
| 描述符 DPL | 保存特权级 |
| 段选择子 | 仍须指向有效描述符 |

CS、SS、DS 等寄存器仍要装合法选择子。装载非法选择子时 CPU 抛哪个异常，取决于错在哪一步——并非一律是 #GP。

```callout info 背景卡片：#XX 是什么
`#` 是 Intel 对 exception 的命名约定，编号 0–31。exception 指 CPU 执行指令时自己检测到的错误，与外部设备发来的硬件中断共用同一张 IDT，只是来源不同。
```

| 向量 | 缩写 | 全称 | 典型触发 |
|---|---|---|---|
| 0 | #DE | Divide Error | 除零 |
| 1 | #DB | Debug | 单步或调试寄存器断点 |
| 3 | #BP | Breakpoint | `int3` |
| 6 | #UD | Invalid Opcode | 非法或未启用的指令 |
| 7 | #NM | Device Not Available | 用了未打开的 SSE 状态 |
| 8 | #DF | Double Fault | 处理异常时又出异常，通常直接重启 |
| 9 | #TS | Invalid TSS | 任务切换时 TSS 非法 |
| 10 | #NP | Segment Not Present | 描述符 P=0 |
| 11 | #SS | Stack-Segment Fault | 栈段本身有问题 |
| 12 | #GP | General Protection | 段机制、特权级、指令权限的兜底错误 |
| 13 | #PF | Page Fault | 缺页，或页权限位不符 |
| 17 | #AC | Alignment Check | 未对齐访问 |

装段选择子出错，实际会落到这几种：

| 情况 | 例子 | 异常 |
|---|---|---|
| 索引越过 GDT 限长 | GDT 只有 3 项，却 `mov ds, 0x1234` | #GP，error code 带 ID=1 |
| 索引合法但 P=0 | `mov ds, 0x18`，而该项没有 present | #NP |
| 描述符类型不对 | 把代码段选择子装进 DS | #GP |
| 特权级不够 | CPL=0 时装 DPL=3 的选择子 | #GP |

#GP 的 error code 低几位用来区分细节：bit 0–1 是选择子还是 IDT 索引，bit 2 是查 GDT 还是 LDT，bit 4–11 是索引本身。#TS / #NP / #SS 被单拎出来单独报告，是因为它们几乎只出现在任务切换和栈访问这两条路径上，值得更精确的诊断。

**#PF 和 #GP 的分工**：GDT 本身是一段内存，CPU 连那一页都读不到，才是 #PF；读到了但内容不对，才是 #GP 或 #NP。下面「A 位回写」那张流程图走的正是前一种。

```callout warn boot 阶段的额外代价
重载段寄存器那几条 mov 只要有一条写错，后果就不只是一条普通的 panic。IDT 还没建好的启动早期会把它退化成 #DF，甚至三连重启。逐条重载的价值，就是让这个暴露窗口短到只剩几条指令。
```
## 段描述符位域

**真正影响长模式行为的字段集中在 access byte 和 flags。**

![64 位段描述符、access byte 与 flags 位域](/home/jqlin/Project/skills/linux-0.11-from-scratch/docs/concepts/assets/gdt-long-mode/segment-descriptor.svg)

access byte 的 P 位表示本项有效，DPL 表示特权级，S 位区分代码/数据段和系统段。

代码段的低 4 位是 E、C、R、A。A 位表示 CPU 已经装载过这个段。

flags 中的 L=1 表示 64 位代码段。D/B 位必须按代码类型设置，不能与 L 位随意组合。

## 选择子位域

**选择子由表索引、表类型和请求特权级组成。**

![16 位段选择子的 index、TI 与 RPL 位域](/home/jqlin/Project/skills/linux-0.11-from-scratch/docs/concepts/assets/gdt-long-mode/selector.svg)

计算规则是 `selector = (index << 3) | (TI << 2) | RPL`。

GDT 第 0 项必须是 null 描述符。因此索引 1 对应选择子 `0x08`，索引 2 对应 `0x10`。

## GDTR 位布局与伪描述符

**GDTR 只装两个字段：一个 16 位 limit 和一个 64 位 base，加起来正好 10 字节。**

![GDTR 的 limit 与 base 两段字段，以及 lgdt 读入的 10 字节内存伪描述符](/home/jqlin/Project/skills/linux-0.11-from-scratch/docs/concepts/assets/gdt-long-mode/gdtr.svg)

| 字段 | 宽度 | 内存伪描述符里的位置 | 含义 |
|---|---|---|---|
| limit | 16 位 | byte 0–1 | 表最后一个字节相对 base 的偏移 |
| base | 64 位 | byte 2–9 | 表第一个字节的线性地址 |

Intel 在 LGDT/LIDT 的指令参考里直接写死了这个切分：

```
ELSE IF 64-bit Operand Size (* In 64-bit Mode *)
    THEN
        GDTR(Limit) := SRC[0:15];
        GDTR(Base)  := SRC[16:79];
```

`SRC` 是那个 10 字节内存操作数，低地址的字节排在前面。所以 GDTR 整体只有 80 位——所谓「64 位 base」说的是字段宽度，不是寄存器宽度。

**limit 是最后一个字节的偏移，不是表长。** 把它加到 base 上，得到的就是最后一个有效字节的地址。GDT 有 n 项、每项 8 字节，limit = n × 8 − 1。3 项的表占 24 字节，limit 写 23，也就是 `0x17`。Intel 的原话是 GDT limit 必须写成 8N − 1。

**limit 只有 16 位，这是硬上限。** 最大 `0xFFFF` 对应 65536 字节，也就是 8192 项。Intel 明确写了长模式下描述符表最多容纳 8192 个 8 字节描述符。

**长模式不检查段的 limit，但表 limit 仍然有效。** 代码段和数据段的 base、limit 在长模式被忽略；GDTR.limit 不同，CPU 仍然拿它挡掉指向表外的选择子。

```callout warn limit 写成表长，会短掉一个字节
3 项的表把 limit 填成 24 = 0x18，最后一项描述符的末字节就落在 limit 之外。【合理推断】访问那一个字节可能被算成越过表 limit，从而抛 #GP；Intel 只规定了 8N − 1 的写法，没有逐条描述这种偏差的后果。
```

base 是 GDT 首字节的线性地址。长模式下这个字段占满 64 位，但取的值必须是 canonical 地址。低 4GB 的任何地址都满足这个条件，GDT 一般就放在低 4GB。

`sgdt` 是镜像操作：按同样的 10 字节布局把 GDTR 写回内存，用来查看 CPU 读到的 GDTR。IDTR 是同一套结构（16 位 limit 加 64 位 base，一对指令是 `lidt` 和 `sidt`），本篇不展开。

## 换表、换 CS 与 A 位
**lgdt 只更换表地址，不会自动更新所有段寄存器。**

DS、ES、FS、GS、SS 可用 `mov` 重载。CS 没有对应的 `mov` 指令，常用 `retfq` 构造远返回来切换。

```sequence
内核 -> GDTR: lgdt 写入新表地址和限长
内核 -> 数据段寄存器: mov 重载选择子
内核 -> 栈: 压入新 CS 选择子和下一条指令地址
内核 -> CS: retfq 弹出并切换
```

装载段选择子时，CPU 会把描述符 A 位置 1。只读页中的描述符应预置 A=1，避免这次隐式回写。

```flow LR
[装载段选择子] -> [CPU 检查 A 位]
[CPU 检查 A 位] -> {A 已为 1?}
{A 已为 1?} -> [无需回写]: 是
{A 已为 1?} -> [尝试写 A=1]: 否
[尝试写 A=1] -> {页面可写?}
{页面可写?} -> [继续执行]: 是
{页面可写?} -> [#PF]: 否
```

```callout info 背景卡片：为什么换 CS 只能用 retfq
这个形状是 8086 留下的，不是可以绕开的取巧写法。
```

**第一层，8086 逼出来的。** 实模式下指令地址就是 `CS:IP` 这个二元组。换 CS 意味着下一条要解码的字节位置也跟着变了。所以「换 CS」在语义上不可能是「照常往下执行」。它必须与一次控制转移绑定：`jmp far` / `call far` / `ret far` / `iret` / 中断与异常入口，就这五处。

**第二层，386 也没能改掉。** 保护模式下段基址已是描述符里的显式字段，「CS 参与地址计算」的原始理由消失了，本可以补一条 `mov cs, r16`。但 CS 还承载特权级语义：装 CS 是一次安全边界操作，和几乎无副作用的 `mov ds` 不是一类指令；补进来还会让「哪些指令会改变当前特权级」这条属性判断变得含糊。Intel 选择不补，于是「CS 只随控制转移装载」被写进架构契约。

**第三层，长模式只改了后果，没改规则。** CS 不再贡献基址和限长，但选择子合法性校验和 DPL 检查一条没少。Linux 的 `lgdt; push %cs; push $ret; lretq`、GRUB、以及所有写内核启动代码的人都停在同一堵墙前，于是这个写法成了事实标准。

两个容易卡住的小点：

- `retfq` 里的 `q` 只指定 64 位操作数宽度，等价于 `lretq`；`retf` 的 `f` 是「强制远返回」，64 位模式下仍必须写上。
- 用两次 push 加 `retfq`，而不是 `jmp far`：far jmp 的目标要塞 8 字节偏移加 2 字节选择子，写 immediate 很别扭，走内存操作数又多一次内存读。push 两次再 `retfq` 全程走寄存器，是最短路径。

代价是它要借用栈：执行前栈上需要 16 字节连续可写空间。boot 代码的栈由引导程序准备好，这条通常自动满足；自己搭环境时，它往往是第一个要确认的条件。
## 最小可运行示例

**一个长模式代码段描述符，主要是几组标志位的按位或。**

保存为 `gdt_desc.rs`，执行 `rustc gdt_desc.rs && ./gdt_desc`：

```rust title="gdt_desc.rs · 独立示例"
fn main() {
    const PRESENT: u64 = 1 << 47;
    const S_CODE_OR_DATA: u64 = 1 << 44;
    const TYPE_EXEC_READ_ACCESSED: u64 = 0b1011 << 40;
    const FLAG_LONG_MODE: u64 = 1 << 53;
    let desc = PRESENT | S_CODE_OR_DATA | TYPE_EXEC_READ_ACCESSED | FLAG_LONG_MODE;
    println!("code segment descriptor = {:#018x}", desc);
    println!("access byte  = {:#010b}", (desc >> 40) as u8);
    println!("flags nibble = {:#06b}", ((desc >> 52) as u8) & 0xF);
}
```

预期输出：

```
code segment descriptor = 0x00209b0000000000
access byte  = 0b10011011
flags nibble = 0b0010
```

`0b1011` 同时声明可执行、可读和 A=1。这里的 A=1 用来避免 CPU 回写描述符。

## 公开资料

**先对照位布局，再用运行示例验证自己的理解。**

- OSDev Wiki：[GDT](https://wiki.osdev.org/GDT)
- OSDev Wiki：[GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial)
- Intel SDM Vol.3：段描述符、GDTR 和长模式分段章节

## 自测

**1. 选择子 `0x18` 指向 GDT 第几项？RPL 是多少？**

- A. 第 2 项，RPL=3
- B. 第 3 项，RPL=0
- C. 第 4 项，RPL=0
- D. 第 6 项，RPL=1

```html
<details><summary>第 1 题答案与解析</summary><p><strong>B</strong>。`0x18 >> 3 = 3`，低两位为 0。</p></details>
```

**2. GDT 位于只读页，描述符 A=0。装载它的选择子会发生什么？**

- A. CPU 自动忽略 A 位
- B. CPU 把描述符复制到 LDT
- C. CPU 尝试回写 A 位，可能触发 #PF
- D. CPU 自动把 L 位改成 1

```html
<details><summary>第 2 题答案与解析</summary><p><strong>C</strong>。装载选择子会触发 A 位回写；只读页会拒绝这次写。</p></details>
```

**3. lgdt 已执行，但 CS 仍缓存旧选择子。接下来应怎么做？**

- A. 再执行一次 lgdt
- B. 用 mov 直接写 CS
- C. 用远返回机制重载 CS，并重载其它段寄存器
- D. 把 GDT 的 base 改成当前 RIP

```html
<details><summary>第 3 题答案与解析</summary><p><strong>C</strong>。lgdt 不换 CS；`retfq` 是常用的 CS 重载方法。</p></details>
```

**4. 要让描述符声明 64 位代码段，应优先检查哪个字段？**

- A. flags 的 L 位
- B. base 的低 16 位
- C. limit 的高 4 位
- D. access byte 的 R 位

```html
<details><summary>第 4 题答案与解析</summary><p><strong>A</strong>。L=1 声明 64 位代码段；base 和 limit 通常被忽略。</p></details>
```

**5. GDT 建了 3 项（null、64 位代码段、数据段），limit 该填多少？**

- A. 24，直接写表长
- B. 23，也就是 `0x17`
- C. 32，按每项 16 字节的系统描述符算
- D. 22，把第 0 项从表长里扣掉后再减 1

```html
<details><summary>第 5 题答案与解析</summary><p><strong>B</strong>。limit 是末字节偏移，3 × 8 − 1 = 23 = 0x17。A 多算了 1 个字节；C 错在 8 字节描述符没有 16 字节那档；D 错在第 0 项仍占 8 字节，limit 只减 1。</p></details>
```
