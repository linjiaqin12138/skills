# GDT 与长模式：分段还剩什么用

> 概念深读（M2 Bite1 背景卡片承诺）。面向第一次接触 x86 分段的读者。
> 读完你应能回答：GDT 是什么数据结构；64 位模式下分段还剩哪两件事；为什么 CS 要用 retfq 换；描述符 Accessed 位为什么会让内核三连重启。

## 一句话版本

**GDT（Global Descriptor Table，全局描述符表）** 是内存里一张固定格式的表格。每行 8 字节，叫一个**段描述符**，描述一个内存段的属性。CPU 的段寄存器（CS/DS/SS 等）里装的不是段本身，而是一个 16 位编号——**选择子**，指明"我现在用这张表的第几行"。

## 为什么需要搞懂它

历史上的分段机制负责内存隔离：每个段有基址、限长、权限，CPU 逐次访问检查。64 位长模式下这套检查基本被废弃——代码/数据段的基址和限长被 CPU 直接忽略，内存隔离改由分页负责。

但分段没有彻底消失，它还剩两件事：

1. **段寄存器必须指向合法描述符**。CS/SS/DS 等寄存器里仍需装一个指向有效 GDT 项的选择子，否则 CPU 报 #GP。
2. **代码段描述符的 L 位声明"这是 64-bit 代码段"**。L=1 告诉 CPU 当前执行的是 64 位代码，指令译码按长模式走。

另外，特权级（ring0/ring3）的判定也挂在段描述符的 DPL 字段上——M8 做用户态时会回来用到它。

内核从 Limine 引导器手里接管后，Limine 建的 GDT 不归我们管。自己的内核要用自己的表，这是接管的第一步。

## 最小可运行示例：一个描述符就是几个标志位的拼装

下面这段 `rustc` 直接编译运行。它按 Intel SDM 的位布局拼出一个 64 位代码段描述符：

```rust
// rustc gdt_desc.rs && ./gdt_desc
fn main() {
    const PRESENT: u64 = 1 << 47;              // P=1：描述符有效
    const S_CODE_OR_DATA: u64 = 1 << 44;       // S=1：代码/数据段（非系统段）
    const TYPE_EXEC_READ_ACCESSED: u64 = 0b1011 << 40;
    const FLAG_LONG_MODE: u64 = 1 << 53;       // L=1：64-bit 代码段
    let desc = PRESENT | S_CODE_OR_DATA | TYPE_EXEC_READ_ACCESSED | FLAG_LONG_MODE;
    println!("code segment descriptor = {:#018x}", desc);
    println!("access byte  = {:#010b}", (desc >> 40) as u8);
    println!("flags nibble = {:#06b}", ((desc >> 52) as u8) & 0xF);
}
```

实测输出：

```
code segment descriptor = 0x00209b0000000000
access byte  = 0b10011011
flags nibble = 0b0010
```

要点：

1. 长模式下 base 和 limit 字段被忽略，所以 64 位里只需要拼 access byte（位 40~47）和 flags 高半字节（位 52~55）。
2. `0x00209b0000000000` 是所有 x86-64 内核代码段描述符的标准长相。
3. access byte 的最低位（位 40）是 Accessed 位——下面专门讲它，它是本项目 M2 实测踩过的坑。

## 选择子：段寄存器里的 16 位编号

选择子的结构：`索引（位 3~15）| TI（位 2）| RPL（位 0~1）`。

- **索引**：GDT 里的行号。索引 1 → 字节偏移 8 → 选择子 `0x08`。
- **TI**：0 = 查 GDT，1 = 查 LDT（本项目不用 LDT）。
- **RPL**：请求的特权级，ring0 就是 0。

所以 `KERNEL_CODE_SEL = 0x08` 的意思是：GDT 第 1 行、GDT 表、ring0。GDT 第 0 行按规范必须是全零的 null 描述符。

## 接管的两步：lgdt 换表，retfq 换 CS

`lgdt` 指令把新表的地址和限长装进 GDTR 寄存器，从此 CPU 查新表。但有一个细节：**段寄存器里仍是旧选择子的缓存值**，`lgdt` 不会自动重载它们。

DS/ES/FS/GS/SS 可以用 `mov` 直接写。CS 不行——x86 没有 `mov cs, ...` 指令。标准做法是伪造一个远返回帧：把新代码段选择子和下一条指令的地址压栈，执行 `retfq`（64 位远返回），CPU"返回"时顺带换掉 CS，执行流恰好落回原地。

## 坑：Accessed 位的隐式回写

CPU 每次把选择子装进段寄存器时，会把内存中对应描述符的 **A 位（位 40）置 1**，标记"这个段被用过"（SDM Vol.3 3.4.5.1）。这是一次 CPU 对内存的隐式写。

如果 GDT 放在只读页（Rust 的 `static` 数组默认进 .rodata），且描述符 A 位初始为 0：装载段寄存器的那一刻，CPU 向只读页回写 A 位 → 触发 #PF 缺页异常。此时如果还没有可用的 IDT 接住异常，连锁升级成 #DF、triple fault，CPU 复位——表现就是 qemu 里内核反复重启，一行日志都打不出。

修法只有一个字节的差别：**建表时预置 A=1**。CPU 发现位已是 1，无需回写。这就是本项目描述符 type 字段用 `0b1011`（EXEC|READ|ACCESSED）而不是 `0b1010` 的原因。

## 和本项目的对应关系

| 概念 | 本项目代码 |
|---|---|
| 段描述符位拼装 | `kernel/src/gdt.rs` 的 `SegmentDescriptor::kernel_code/kernel_data` |
| 三项扁平 GDT | `kernel/src/gdt.rs` 的 `static GDT`（null + 内核代码 + 内核数据） |
| 选择子 0x08 / 0x10 | `kernel/src/gdt.rs` 的 `KERNEL_CODE_SEL` / `KERNEL_DATA_SEL` |
| lgdt 换表 | `kernel/src/gdt.rs` 的 `init()` |
| retfq 换 CS | `kernel/src/gdt.rs` 的 `reload_segment_registers()` |
| A 位预置 | `kernel/src/gdt.rs:45-47` 的注释与 `0b1011` |

本项目不建 TSS（任务状态段）。TSS 是特权级切换时换栈用的，M8 做 ring3 时才需要。

## 深入阅读

- Intel SDM Vol.3：3.4.5 段描述符、3.5.1 GDTR/LDTR、3.2.4 长模式下的分段
- OSDev Wiki：[GDT](https://wiki.osdev.org/GDT) 与 [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial)
- 本仓库代码：`kernel/src/gdt.rs`（96 行，全文可读完）
