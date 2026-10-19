---
template: doc
theme: paper
title: M2 Bite1 · GDT/IDT：接管 CPU 异常
subtitle: 扁平 GDT + 256 项 IDT + 异常现场打印，除零不再三连重启
source: linux-0.11-from-scratch · M2
---

Lead: 自建 GDT 接管段寄存器，再用 IDT 接管向量 0~31。故意除零后，内核打印异常名与寄存器现场并停机。`make check-m2-b1` 已通过。

## 问题

**没有自己的 IDT，一次普通异常就可能变成 triple fault。**

1. 指令触发 #PF 等异常。
2. CPU 按 IDT 查找 handler。
3. 无效 handler 引发 double fault。
4. 再次失败触发 triple fault。
5. qemu 重启并重复启动日志。

此时看不到出错地址，也不知道哪个模块出错。后续外部中断、缺页和特权级切换都需要这条诊断路径。

**本 Bite 验收标准**：自己的 GDT 和 IDT 生效；`asm div` 触发 #DE；串口和屏幕显示异常名、RIP 与通用寄存器；`M0: kernel alive` 只出现一次。

## 背景卡片：GDT 与段描述符

**GDT 是段属性表，段寄存器保存的是表索引。**

长模式的内存隔离由分页负责，但 CS 等寄存器仍需合法选择子。代码段描述符的 L 位声明 64 位代码。

最小示例：拼一个长模式代码段描述符。执行 `rustc gdt_desc.rs && ./gdt_desc`：

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

输出：

```
code segment descriptor = 0x00209b0000000000
access byte  = 0b10011011
flags nibble = 0b0010
```

本 Bite 在 `gdt.rs` 建三项扁平 GDT，并向 IDT 提供代码段选择子。

[概念深读：GDT 与长模式](../../concepts/gdt-long-mode.html)。卡片看不懂就先去读那篇，里面有描述符位图和选择子位图。

## 背景卡片：IDT、异常与栈帧

**IDT 告诉 CPU 出事时跳到哪里，栈帧保存出事时的现场。**

异常来自指令执行，外部中断来自设备。两者共用 IDT。部分异常还会多压一个 error code。

最小示例：用户态程序除零后，操作系统把异常包装成 SIGFPE。执行 `gcc -O0 sigfpe.c -o sigfpe && ./sigfpe`：

```c title="sigfpe.c · 独立示例"
#define _GNU_SOURCE
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <ucontext.h>

static void handler(int sig, siginfo_t *info, void *uctx) {
    (void)sig; (void)info;
    ucontext_t *uc = uctx;
    printf("caught SIGFPE (CPU exception #DE)\n");
    printf("rip=%#llx  rax=%#llx  rdx=%#llx\n",
           (unsigned long long)uc->uc_mcontext.gregs[REG_RIP],
           (unsigned long long)uc->uc_mcontext.gregs[REG_RAX],
           (unsigned long long)uc->uc_mcontext.gregs[REG_RDX]);
    exit(0);
}

int main(void) {
    struct sigaction sa = {0};
    sa.sa_sigaction = handler;
    sa.sa_flags = SA_SIGINFO;
    sigaction(SIGFPE, &sa, NULL);
    volatile int a = 1, b = 0;
    printf("%d\n", a / b);
    return 0;
}
```

输出（RIP 每次可能不同）：

```
caught SIGFPE (CPU exception #DE)
rip=0x55a2e9d052c3  rax=0x1  rdx=0
```

本 Bite 在 `idt.rs` 安装向量 0~31 的 handler，并打印统一栈帧。

[概念深读：IDT 与异常栈帧](../../concepts/idt-exception-frame.html)。卡片看不懂就先去读那篇，里面有门描述符和栈帧偏移图。

## 设计

**先让新 GDT 提供合法 CS，再让 IDT 把异常导向统一 handler。**

![M2 Bite1 顶层架构：GDT 与 IDT 模块边界及异常数据流](/home/jqlin/Project/skills/linux-0.11-from-scratch/docs/milestones/m2-exceptions/assets/dst/arch.svg)

分层查看：[IDT 内部结构](assets/dst/arch-idt.svg) ｜ [M1→M2 架构变动](assets/dst/arch-diff.svg)

- `gdt.rs`：建 null、内核代码段、内核数据段三项，并重载段寄存器。
- `idt.rs`：安装 32 个异常入口，统一栈帧后调用 Rust handler。
- 初始化顺序固定为 `gdt::init()` → `idt::init()`。IDT 使用新 GDT 的 `KERNEL_CODE_SEL`。

```flow LR
[CPU 执行 div] -> [IDT 向量 0]
[IDT 向量 0] -> [entry_divide_error]: interrupt gate
[entry_divide_error] -> [common_exception_entry]: 压向量号后 jmp
[common_exception_entry] -> [exception_handler]: 压 15 个寄存器后 call
[exception_handler] -> *printk!: 打印异常名与现场
[exception_handler] -> (hlt 停机)
```

```html
<details><summary>决策卡片 1：interrupt gate 还是 trap gate</summary><ul>
<li><strong>决策点</strong>：异常入口是否自动清 IF。</li>
<li><strong>备选</strong>：(a) interrupt gate 清 IF；(b) trap gate 保留 IF。</li>
<li><strong>选择理由</strong>：选 (a)，避免打印现场时被外部中断打断。</li>
<li><strong>放弃的成本</strong>：(b) 支持可屏蔽中断嵌套响应；当前选择不能直接提供这种能力。</li>
<li><strong>失效边界</strong>：M3 若需要嵌套响应，应重新评估门类型。</li>
</ul></details>
<details><summary>决策卡片 2：统一栈帧还是两套布局</summary><ul>
<li><strong>决策点</strong>：有无 error code 的向量如何共用 handler。</li>
<li><strong>备选</strong>：(a) 无错误码入口补压 0；(b) handler 按向量解析两套帧。</li>
<li><strong>选择理由</strong>：选 (a)，一个 ExceptionFrame 覆盖全部异常。</li>
<li><strong>放弃的成本</strong>：(b) 可少做一次 8 字节压栈；统一布局增加这次写入。</li>
<li><strong>失效边界</strong>：只有异常路径需要极致优化时才值得改选。</li>
</ul></details>
<details><summary>决策卡片 3：打印后 hlt 还是 iretq</summary><ul>
<li><strong>决策点</strong>：报告现场后停机，还是恢复被打断的指令。</li>
<li><strong>备选</strong>：(a) 打印后 hlt；(b) 恢复寄存器并 iretq。</li>
<li><strong>选择理由</strong>：选 (a)。本 Bite 只要求可靠诊断，除零后返回还会再次异常。</li>
<li><strong>放弃的成本</strong>：(b) 支持修复后重试指令；当前 hlt 不能恢复执行。</li>
<li><strong>失效边界</strong>：可修复缺页到来时必须增加 iretq 路径。</li>
</ul></details>
```

## 代码领读

**实现顺序与设计一致：先建表和入口，再接统一 handler。**

**GDT 描述符**（`gdt.rs:22-49`）。`const fn` 在编译期拼标志位。`0b1011` 的最低位预置 A=1，避免 CPU 回写只读描述符。

```rust src=kernel/src/gdt.rs lines=22-49 hl=32
```

**lgdt 与段寄存器重载**（`gdt.rs:64-98`）。`lgdt` 只换表。CS 通过伪造远返回帧并用 `retfq` 重载，其它段寄存器用 `mov`。

```rust src=kernel/src/gdt.rs lines=64-98 hl=71
```

**IDT 门描述符**（`idt.rs:34-62`）。`#[repr(C, packed)]` 保证 16 字节布局。`interrupt_gate` 把 handler 地址拆成三段，并写入 `KERNEL_CODE_SEL`。

```rust src=kernel/src/idt.rs lines=34-62 hl=47
```

**统一栈帧**（`idt.rs:64-90`）。字段顺序与压栈顺序互为镜像。最后压入的 r15 地址最低，因此排在结构体首位。

```rust src=kernel/src/idt.rs lines=64-90 hl=67
```

**公共入口**（`idt.rs:128-160`）。`naked_asm!` 避免编译器生成会改动 RSP 的序言。保存 15 个寄存器后，RSP 作为帧指针传给 Rust handler。

```rust src=kernel/src/idt.rs lines=128-160 hl=149
```

**宏生成 stub**（`idt.rs:162-187`）。无 error code 的入口先压 0。`define_exceptions!` 在 `idt.rs:189-204` 批量生成 32 个入口和安装语句。

```rust src=kernel/src/idt.rs lines=162-187 hl=170
```

**Rust handler**（`idt.rs:264-302`）。`idt.rs:270` 从 RIP 字段的位置反推异常前 RSP。打印后进入 hlt，不再返回。

```rust src=kernel/src/idt.rs lines=264-302 hl=270
```

**main 接线与触发**（`main.rs:52-77`）。Rust 除法自带除零检查，会走 panic。验收必须使用内联汇编 `div` 才能进入 IDT。

```rust src=kernel/src/main.rs lines=52-77 hl=72
```

## 验收

**`make check-m2-b1` 于 2026-10-18 复跑通过，exit 0。**

serial.log 尾部实测：

```
M2: gdt ok
M2: idt ok
M2: EXCEPTION #0 divide error
M2: error_code=0x0
M2: rip=0xffffffff800002a4 cs=0x0008 rflags=0x0000000000010046
M2: rax=0x0000000000000000 rbx=0x0000000000000000 rcx=0xffff800007f86ac0 rdx=0x0000000000000000
M2: rsi=0xffff800007f86a28 rdi=0x0000000000000000 rbp=0x0000000000000000 rsp=0xffff800007f86ee0
M2: r8=0x0000000000000257 r9=0x0000000000000012 r10=0x0000000000000000 r11=0x0000000000000000
M2: r12=0x0000000000000000 r13=0x0000000000000000 r14=0x0000000000000000 r15=0x0000000000000000
```

| 断言 | 证明什么 | 状态 |
|---|---|---|
| `M0: kernel alive` 恰好 1 次 | 没有三连重启 | ok 通过 |
| `M2: gdt ok` / `M2: idt ok` | 两张表完成安装 | ok 通过 |
| `EXCEPTION #0` + `divide error` | 向量 0 进入自己的 handler | ok 通过 |
| RIP 正则命中 | 现场含真实地址 | ok 通过 |
| M1 marker 全部保留 | 旧功能未回归 | ok 通过 |
| screendump 1440015 字节 | 现场同时写入屏幕 | ok 通过 |

```callout ok 现场来自真实寄存器
cs=0x0008 对应新 GDT 代码段。rax=rdx=0 对应两条 xor。rflags 的 ZF/PF 位与 xor 后的状态一致。
```

## 与原版差异、常见坑与展望

**本 Bite 保留汇编入口和打印后停机，但统一了 32 个向量的栈帧。**

【合理推断】本地没有 Linux 0.11 源码，下表按公开版本结构概述，未逐行核对。

| 维度 | Linux 0.11 | 本 Bite |
|---|---|---|
| 入口 | 汇编压栈后转 C | naked 汇编后转 Rust |
| 栈帧 | 各入口分别压栈 | 32 个入口统一布局 |
| 输出 | C 打印后死循环 | Rust `printk!` 双写后 hlt |

- **D16**：向量 32~255 仍为空洞，handler 也没有 iretq。外部中断在 M3 收敛，修复后返回在 M9 收敛。
- **D15**：当前无外部中断，console 无锁假设仍成立。M3 引入中断后换锁或临界区。

```callout err 实测坑：A=0 与只读 GDT 引发三连重启
CPU 装载段寄存器时会回写 A 位。首版 GDT 位于只读 .rodata，这次写入触发 #PF；无效 IDT 又让异常继续升级。预置 A=1 后，CPU 无需回写。
```

```flow LR
[A=0 的描述符] -> [装载段寄存器] -> [CPU 回写 A 位] -> {只读 .rodata?}
{只读 .rodata?} -> [#PF]: 是
{只读 .rodata?} -> [完成装载]: 否
[#PF] -> [IDT 不可用] -> [double fault] -> [triple fault]
```

**另一个坑**：Rust 的 `1/0` 走 panic，不会触发 #DE。Bite2 需要让 `#[panic_handler]` 接上 `printk!`，打印消息与位置。

**后续收敛**：M3 填外部中断 handler 并重估 console 并发；M9 为写时复制增加修复后的 iretq 返回路径。

## 自测

**1. 把 `idt::init()` 移到 `gdt::init()` 之前，最可能引入什么问题？**

- A. IDT 项数从 256 变成 32
- B. 门里的 `0x08` 可能仍指向旧 GDT 的其它描述符
- C. CPU 不再保存 RIP
- D. GDT 会自动迁移到可写页

```html
<details><summary>第 1 题答案与解析</summary><p><strong>B</strong>。新选择子只有在新 GDT 生效后才有预期含义。异常到来时可能进一步触发 #GP。</p></details>
```

**2. 新增一个无 error code 的异常入口，怎样接入现有统一 handler？**

- A. 只压向量号；公共入口已有统一布局
- B. 先压占位 0，再压向量号，然后 jmp 公共入口
- C. 直接 call Rust handler，不保存寄存器
- D. 把该向量的 P 位清零

```html
<details><summary>第 2 题答案与解析</summary><p><strong>B</strong>。占位 0 对齐 error code 字段，jmp 不会额外压返回地址。</p></details>
```

**3. M3 需要 handler 中继续响应定时器，应该优先重评哪个决策？**

- A. interrupt gate 的选择，因为它会清 IF
- B. GDT 的 null 项，因为它占用索引 0
- C. 预置 A=1，因为它会关闭中断
- D. `#[repr(C, packed)]`，因为它会修改 IF

```html
<details><summary>第 3 题答案与解析</summary><p><strong>A</strong>。trap gate 保留 IF，更适合需要嵌套响应的场景，但仍需评估现场安全。</p></details>
```

**4. 缺页 handler 修好页表后要重试原指令，当前代码最需要补什么？**

- A. 再打印一次现场后继续 hlt
- B. 恢复寄存器，清理入口附加字段，再 iretq
- C. 把 IDT 的 P 位全部清零
- D. 把 error code 改成 RIP 后直接 ret

```html
<details><summary>第 4 题答案与解析</summary><p><strong>B</strong>。可修复 fault 需要恢复现场并返回保存的 RIP；当前 hlt 路径不具备这个能力。</p></details>
```

## 细节展开

**两种顺序都合法，x86 没有规定 CS 与数据段谁先换。**

本工程把数据段 `mov` 排在 `retfq` 之后。原因是 `mov ss` 带一条特殊语义：装完 SS 之后的那条指令边界上，中断与部分调试异常被抑制。这条语义要求 `mov ss` 排在所有栈操作之后。

**为什么顺序可以不同**

`lgdt`（`gdt.rs:71`）只往 GDTR 写表基址和限长，六个段寄存器一个都不动。往里装选择子这一步的先后由这段代码自己安排，架构不参与裁决。

CS 与 DS/ES/FS/GS/SS 之间也没有数据依赖：CS 只决定「下一条指令从哪个地址取字节」，`mov ds` 不读 CS；反过来换 CS 也不看数据段当前装的是什么。

| 步骤 | 概念页 gdt-long-mode 的排法 | 本工程 gdt.rs:82-91 的排法 |
|---|---|---|
| 第 1 步 | lgdt | lgdt |
| 第 2 步 | mov 重载数据段 | 压入 CS 与返回地址 |
| 第 3 步 | 压入 CS 与返回地址 | retfq 换 CS |
| 第 4 步 | retfq 换 CS | mov 重载数据段 |

[GDT 与长模式](../../concepts/gdt-long-mode.html) 的时序图画的是左边那一列，boot 代码常这么排；本工程写成右边那一列。两条路径最终把同一批选择子装进同一批段寄存器，结果一致。

```html
<details><summary>逐条读这七条指令：两个 push、一个 lea、一个 retfq</summary><ul>
<li><strong>push 8</strong>（<code>gdt.rs:82</code>）：64 位模式下 <code>push imm</code> 按 64 位操作数宽度压栈，8 字节。这个槽只有低 2 字节是 CS 选择子 <code>0x0008</code>，上面 6 字节是 0；远返回读完丢弃它们，但 RSP 照样 +8。</li>
<li><strong>lea {tmp}, [rip + 2f]</strong>（<code>gdt.rs:83</code>）：<code>2f</code> 是 GNU 汇编的数字局部标签写法，指向本块里 <code>2:</code> 处那条指令（<code>gdt.rs:86</code>），也就是 <code>retfq</code> 后面的第一条。压进栈的是「CS 换完以后该接着执行的那条指令的地址」——换 CS 之后立刻由新 CS 解码的正是这条 <code>mov ds</code>。<code>lea</code> 只算地址，不读内存也不改标志位，本块声明 <code>preserves_flags</code> 成立。</li>
<li><strong>为什么用 lea 而不是 push $2f</strong>：<code>push $标签</code> 要求汇编器把标签编成绝对地址，而内核可以被链接到任意物理地址。RIP 相对寻址只在「相对当前指令」的意义上成立，与加载地址无关。</li>
<li><strong>压栈顺序</strong>（<code>gdt.rs:82-84</code>）：栈从高地址向低地址长，先压的 CS 落在高地址，后压的返回地址落在低地址。<code>retfq</code> 先弹 RIP（低地址）再弹 CS（高地址），顺序天然就对。</li>
<li><strong>retfq</strong>（<code>gdt.rs:85</code>）：64 位操作数宽度的远返回。装 CS 时 CPU 照常做选择子合法性与 DPL 检查，<code>0x08</code> 的 RPL=0、描述符 DPL=0、当前 CPL=0，检查通过。<code>f</code> 是强制远返回，64 位模式下必须写；<code>q</code> 指定操作数宽度。</li>
<li><strong>栈账</strong>：两次 push 共 16 字节，远返回也弹掉 16 字节（8 字节 RIP 加 8 字节 CS 槽），RSP 净回到原位。实测做法是把同一串指令放进一个 freestanding 程序里执行，前后各读一次 RSP，差值为 0。</li>
</ul></details>
<details><summary>mov ds, 0x10 为什么必须绕一个 16 位寄存器</summary><ul>
<li>装载段寄存器只有 <code>MOV Sreg, r/m16</code> 一种形式（opcode 8E，Intel MOV 指令文档的编码表里 Sreg 作目的只有这一条），没有 <code>mov ds, imm16</code>。同一份文档还写明 <code>MOV</code> 不能装载 CS，尝试写触发 #UD，装 CS 只能用 far JMP / CALL / RET。</li>
<li><code>{data_sel:x}</code> 的 <code>x</code> 修饰符强制分配 16 位寄存器中转。<code>push</code> 能吃立即数、<code>mov</code> 不能，这就是 <code>gdt.rs:92-93</code> 把 <code>code_sel</code> 声明成 <code>const</code>（省一个寄存器）而 <code>data_sel</code> 声明成 <code>in(reg)</code>（必须过寄存器）的理由。</li>
<li>FS 和 GS 是特例：长模式下它们的基址来自 <code>IA32_FS_BASE</code> 与 <code>IA32_GS_BASE</code> 两个 MSR，不来自描述符。<code>mov fs</code> / <code>mov gs</code> 换的是选择子与它缓存的段属性，基址不动——长模式把 FS/GS 留给 TLS 正是这个原因。这里换它们是为了让选择子和属性合法，基址保留引导器留下的原值。</li>
</ul></details>
<details><summary>mov ss 排在最后的依据：Intel MOV 指令文档原文</summary>
<blockquote><p>Loading the SS register with a MOV instruction suppresses or inhibits some debug exceptions and inhibits interrupts on the following instruction boundary. (The inhibition ends after delivery of an exception or the execution of the next instruction.) This behavior allows a stack pointer to be loaded into the ESP register with the next instruction (MOV ESP, stack-pointer value) before an event can be delivered. … Intel recommends that software use the LSS instruction to load the SS register and ESP together.</p></blockquote>
<ul>
<li>这条语义服务于 32 位的换栈竞态：SS 已指向新栈、ESP 还没指向新栈顶的那一条指令边界上，不允许事件插进来。</li>
<li>本块的 <code>push</code>、<code>push</code>、<code>retfq</code> 全靠 SS 定位栈——<code>push</code> 隐式读 SS:SP 写 SS:SP-8，远返回从 SS:SP 弹栈。所以 <code>mov ss</code> 只能排在它们之后，它是 <code>gdt.rs:91</code>，这七条指令的最后一条。</li>
<li>本工程只换段寄存器、不换 RSP，抑制窗口一直覆盖到函数返回前的第一条指令，这段窗口内没有栈操作，副作用为零。要连栈一起换时，Intel 推荐的写法是 <code>lss</code> 一条指令同时装 SS 与 ESP。</li>
</ul></details>
<details><summary>原版 Linux-0.11 的顺序，以及它的一条错注释</summary><ul>
<li>原版的 CS 确实在数据段之前就换好了，但不是在 <code>setup_gdt</code> 里换的：16 位到 32 位过渡时就已经由 <code>reference/boot/setup.s:294</code> 的 <code>ljmp $sel_cs0, $0</code> 完成。之后的 <code>setup_gdt</code>（<code>reference/boot/head.s:107-109</code>）只剩两条指令：<code>lgdt gdt_descr</code> 与 <code>ret</code>。</li>
<li>真正的一次段寄存器重装在 <code>reference/boot/head.s:27-31</code>：<code>movl $0x10,%eax</code> 之后依次 <code>mov %ax,%ds / %es / %fs / %gs</code>，紧跟 <code>reference/boot/head.s:32</code> 的 <code>lss stack_start,%esp</code>。SS 同样排在最后，而且 SS 与 ESP 由一条指令装完。</li>
<li><code>reference/boot/head.s:28-29</code> 的注释写着 “after changing gdt. CS was already reloaded in 'setup_gdt'”。<code>setup_gdt</code> 只有 <code>lgdt</code> 与 <code>ret</code>，没有重载 CS，CS 是 <code>setup.s:294</code> 的 <code>ljmp</code> 换的。这条注释在原版里就不准确。</li>
</ul></details>
<details><summary>一个尚未触发但存在的风险：out(reg) 允许与输入共用寄存器</summary><ul>
<li><code>gdt.rs:94</code> 的 <code>tmp = out(reg) _</code>（LLVM 的 <code>=r</code>）没有 <code>earlyclobber</code> 修饰。LLVM 契约允许输出与输入分到同一个通用寄存器，<code>earlyclobber</code> 的存在意义正是禁止这种共享。</li>
<li>本块里 <code>tmp</code> 被 <code>lea</code> 写在前（<code>gdt.rs:83</code>），<code>data_sel</code> 被 <code>mov ds, ax</code> 读在后（<code>gdt.rs:87-91</code>）。若两者分到同一寄存器，<code>lea</code> 会先把该寄存器覆盖成标签地址，后面的 <code>mov ds, ax</code> 装进去的就不是 <code>0x10</code>。</li>
<li>当前 release 产物没有踩到。<code>kernel/target/x86_64-unknown-none/release/build/kernel/*/out/kernel.s</code> 的实际输出：</li>
</ul>
<pre><code>	mov	ax, 16
	push	8
	lea	rcx, [rip + .Ltmp34]
	push	rcx
	retfq
.Ltmp34:
	mov	ds, ax
	mov	es, ax
	mov	fs, ax
	mov	gs, ax
	mov	ss, ax</code></pre>
<ul><li><code>tmp</code> 是 rcx、<code>data_sel</code> 是 ax，两者没有重叠。规避这个风险的写法是 <code>tmp = earlyclobber(out(reg) _)</code>。</li></ul>
</details>
```

**换完 CS 之后，为什么能保证新 CS 下 `retfq` 的下一条就是 `mov ds` 这串指令？**

这条保证不来自 CPU，来自两个地方叠在一起。汇编器保证 `2:` 标签就贴在 `retfq` 后面（`gdt.rs:86`）。长模式下 CS 又根本不参与取指地址的计算。压进栈的不是「某个段的某个偏移」，而是一条绝对线性地址。换 CS 只换特权级和代码模式，不动这条地址。

```html
<details><summary>保证一：返回地址是链接期就定死的线性地址</summary><ul>
<li><code>lea {tmp}, [rip + 2f]</code> 里的位移量 <code>2f</code> 由汇编器在生成 <code>.s</code> 时算死，<code>2:</code> 是本块内自己定义、紧跟在 <code>retfq</code> 后面那一行的数字局部标签（<code>gdt.rs:86</code>）。两者指同一条指令，这层关系在汇编期就成立，不依赖任何运行期状态。</li>
<li>所以「retfq 之后是谁」这个问题，答案由源码文本本身给出——只要标签没写歪、这段 asm 没被拆开重排（<code>options(nomem)</code> 已禁止编译器把内存访问跨过 asm 块搬移），release 产物里的实际字节就是：<code>retfq</code> 紧跟 <code>.Ltmp34:</code> 再跟 <code>mov ds, ax</code>。上面那份 <code>kernel.s</code> 就是证据。</li>
<li>唯一的人为风险点是标签位置写错：把 <code>2:</code> 挪到别处汇编照样通过，只是算出的地址指向了别的指令。这类错误只能靠读代码发现。</li>
</ul></details>
<details><summary>保证二：长模式下 CS 不改取指地址</summary><ul>
<li>64 位代码段的基址一律当 0、限长不检查（SDM Vol.3 6.4.2 与 9.5）。取指地址 = RIP 里那个值，直接就是线性地址，中间不加段基址。</li>
<li><code>kernel_code()</code>（<code>gdt.rs:29-35</code>）压根没写 base 字段，描述符里那 8 个字节除了 access 与 L 位全是 0——这就是「base=0」落地的地方。</li>
<li>把 L 位设成 0 就不一样了：32 位代码段要按 线性地址 = base + 偏移 算，base 非零就跳飞。所以 L=1 在这里不是风格选择，是这套写法成立的前提。</li>
<li>CS 换完后还管三件与地址无关的事：CPL（特权级检查与 SS 能不能写）、L 位（后续按 64 位还是 32 位解码）、装载时的描述符合法性。都不改「从哪儿取字节」。</li>
</ul></details>
<details><summary>换个 CS 失败时，是报错而不是悄悄跑错代码</summary><ul>
<li><code>retfq</code> 装 CS 时照常做选择子合法性检查与 DPL 检查：<code>0x08</code> 的 RPL=0、描述符 DPL=0、当前 CPL=0，三者相容才放行；描述符缺 P 位（不在内存 / L 位没置位）会直接 #GP。</li>
<li>也就是说，「GDT 装错」的失败形态是这条指令就抛异常，绝不会跳到某个错误上下文里执行我们的 <code>mov</code> 串。</li>
<li>本 Bite 的验收反过来印证了这点：能被 #DE handler 打印出 RIP 与寄存器现场，说明 CS 换成功了——新 CS 下的取指确实回到了这段代码里，handler 顺着内核栈走回来才拿到的现场。</li>
<li>真正会让「新 CS 下取到意外指令」的只剩与 CS 无关的两件事：栈上的返回地址被写坏，或 <code>lea</code> 的偏移算错。</li>
</ul></details>
```

**GDT 里三段的基址全是 0，CPU 靠什么找到内核代码。**

靠 RIP。长模式下段机制不参与地址翻译，那三个 base 字段不是被忽略的残留，而是这一层压根没有活干。翻译分两段，段机制是第一段，分页是第二段。第一段在长模式下被硬件阉割成恒等映射，第二段还没轮到我们做。

```html
<details><summary>第一段：段机制 → 线性地址，在长模式下被取消</summary><ul>
<li>32 位保护模式的公式是 线性 = CS.base + 偏移，取指时硬件真会加这一下。</li>
<li>64 位模式下这条加法没了：CS 的 base 硬件一律当 0，limit 对 64 位代码段也不检查（SDM Vol.3 6.4.2 / 9.5）。于是 线性 = RIP。</li>
<li>所以描述符里那 4 字节 base 的规格要求变成「必须是 0」，<code>kernel_code()</code>（<code>gdt.rs:29-35</code>）干脆不写它，正好合规。</li>
<li>反例是 L=0 的 32 位代码段：far jmp 到它时按 base + 偏移算，base 非零就跳飞。<code>gdt.rs:33</code> 那行 <code>FLAG_LONG_MODE</code> 因此不是风格选择，是这套写法成立的前提。</li>
</ul></details>
<details><summary>第二段：线性 → 物理，由页表做，眼下还不是我们的活</summary><ul>
<li>真正做翻译的是分页。M2 Bite1 时我们一行页表代码都没写，跳进来的页表是 Limine 建的。</li>
<li> Limine 交接那张时序图第二拍写的就是它：建页表、进长模式、映射高半区（<code>tutorial-bite2-iso-boot.draft.md:80</code>）。</li>
<li>眼下 RIP = 线性 成立（长模式恒等），但 RIP = 物理 不成立：Limine 把内核镜像映射到高半区链接地址，页表项指向它加载内核时用的那批物理页。</li>
<li>framebuffer 的指针值也不是恒等映射。实测 <code>framebuffer.address()</code> 打印为 <code>0xffff8000fd000000</code>：这是 Limine 的高半区直接映射（HHDM）视角，物理显存在 <code>0xfd000000</code>，偏移是 <code>0xffff800000000000</code>。协议规定所有指针默认带 HHDM 偏移（PROTOCOL.md「General Notes」）。</li>
<li>「低 4GB 恒等映射」只是 base revision 0 的遗留约定，我们请求的是 revision 3，已经没有它。所以写显存靠的是 HHDM，不是恒等。</li>
<li>实测这台机器物理内存只有 128MB（<code>Makefile:11</code> 的 <code>-m 128M</code>）。高半区那串地址只是虚拟地址空间里的一个编号，不占任何物理内存。</li>
<li>真正吃物理内存的是被映射的页：内核镜像三个 LOAD 段合计约 115KB，加上页表本身几 KB。地址空间大小与内存大小是两件事。</li>
<li>等自己做页表那一步，「段基址是 0」还省了事：不必为每个段另维护一份内存布局，段退化成纯权限标签。</li>
</ul></details>
<details><summary>那 GDT 描述符在长模式下还剩什么用</summary><ul>
<li>合法性：P 位与 type 位决定装段寄存器时是否 #GP。</li>
<li>CPL：<code>retfq</code> 装 <code>0x08</code> 时比 RPL / 描述符 DPL / 当前 CPL，决定特权级、SS 能否写、以后能否 iret 回用户态。</li>
<li>L 位：唯一开关——它决定后续按 64 位还是 32 位解码指令。「这是一段 64 位代码」这句话，全靠它。</li>
<li>FS 与 GS 是另一套：长模式下它们的基址来自 <code>IA32_FS_BASE</code> / <code>IA32_GS_BASE</code> 两个 MSR，也不来自描述符。</li>
</ul></details>
```

**framebuffer 的指针打印出来是虚拟地址还是物理地址？相等是不是分页造成的？**

实测打印的是虚拟地址：`framebuffer.address()` 在 qemu 下是 `0xffff8000fd000000`。它是 Limine 的高半区直接映射（HHDM）视角，物理显存在 `0xfd000000`。两者能对应上，确实是分页造成的——页表项是 Limine 建的，不是我们。

```html
<details><summary>协议原文：所有指针默认带 HHDM 偏移</summary><ul>
<li>Limine 协议「General Notes」原文：所有指针指向的对象都已加上 higher half direct map 偏移。</li>
<li>HHDM 是 Limine 建的一条巨型映射：整块物理地址空间搬到虚拟地址 <code>0xffff800000000000</code> 起。</li>
<li>换算：打印值 <code>0xffff8000fd000000</code> = 偏移 <code>0xffff800000000000</code> + 物理地址 <code>0xfd000000</code>。</li>
<li>想要偏移的具体值，协议里有专门的 HHDM request 可以问；我们没发，从打印值里也能读出来。</li>
</ul></details>
<details><summary>0xfd000000 这块「物理地址」其实不是内存</summary><ul>
<li>机器只有 128MB RAM（<code>Makefile:11</code> 的 <code>-m 128M</code>），物理地址空间到 <code>0x8000000</code> 为止。</li>
<li><code>0xfd000000</code> 是 qemu 把 VGA 显存放进的 PCI MMIO 空洞，在 RAM 之上。这块物理地址背后没有 DRAM，是显卡的寄存器空间。</li>
<li>所以「物理地址」这个概念的边界不是「有没有内存条」，而是「CPU 访存总线上的一个槽位」。MMIO 槽位直通设备，DRAM 槽位进内存。</li>
</ul></details>
<details><summary>这对我们自己的代码意味着什么</summary><ul>
<li>将来自己建页表时，framebuffer 的这条映射必须接过去：要么同样走 HHDM，要么单独映射一次。不建，写显存立刻 #PF。</li>
<li>现在能写成功，靠的是 Limine 的页表还挂着。我们自己的 GDT/IDT 已经接管了 CPU，但页表还没接管。</li>
<li>顺带一个容易踩的坑：「低 4GB 恒等映射」是 base revision 0 的遗留约定，我们请求 revision 3，它不存在。别以为物理地址 0x100000 就能当虚拟地址直接写。</li>
</ul></details>
```

**`mov rdi, rsp` 及后面四条指令在干什么（idt.rs:149-157）。**

`mov rdi, rsp` 是把"现场"打包交给 Rust 的那一下。前面 15 条 push 把通用寄存器全压进栈，压完后 RSP 正好指着 `ExceptionFrame` 的最低地址字段。把 RSP 拷进 rdi，等于把栈顶当成 `&ExceptionFrame` 传给 handler——rdi 是 C 调用约定的第一个参数寄存器。后面三条各管一件事：对齐栈、调用、兜底停机。

```html
<details><summary>为什么 RSP 直接就是 ExceptionFrame 的头</summary><ul>
<li>push 顺序（rax 先、r15 后，<code>idt.rs:134-148</code>）与结构体字段顺序（r15 先、rax 后，<code>idt.rs:68-82</code>）互为镜像。</li>
<li>栈向下长，最后压的 r15 地址最低。<code>#[repr(C)]</code> 保证字段按声明顺序排，最低地址就是第一个字段。</li>
<li>两边对齐后，栈内存布局 == 结构体布局，RSP 恰好是结构体头。这是零拷贝传参：不用把现场搬进变量，直接让结构体"披"在栈上。</li>
</ul></details>
<details><summary>and rsp, -16 / call / hlt 三条各管什么</summary><ul>
<li><code>and rsp, -16</code>：把 RSP 低 4 位清零，向下取整到 16 字节。调用约定要求 call 时栈 16 对齐，handler 里的 SSE 指令靠这个假设。异常可在任意对齐状态发生，这里必须自己修。</li>
<li>handler 是 <code>-> !</code> 不返回（<code>idt.rs:266</code>），所以不用恢复 RSP，对齐是单向的。</li>
<li><code>call {handler}</code>：<code>sym</code> 让编译器解析符号地址，call 把控制权交给 Rust 函数。</li>
<li><code>2: / hlt / jmp 2b</code>：类型上就到不了的保险丝。万一 handler 意外返回，CPU 走这三条停机，不会跑进别人的内存。</li>
</ul></details>
```

**`lea` 是什么指令，`2f`/`2b` 是什么语法。**

`lea` = Load Effective Address。普通 `mov` 从内存取值，`lea` 只算地址本身：把操作数算出来存进寄存器，不去读那处内存。`lea rax, [rip + 2f]` 就是「rax = 标签 2 的地址」，不读它。

`2f` 是 GNU 汇编器的数字局部标签。`2:` 定义一个局部标签；`f` 是 forward，往后找最近的 `2:`；`b` 是 backward，往前找。数字标签不是符号，不进符号表，同名可以反复定义，靠位置区分。

```html
<details><summary>数字标签的边界与动机</summary><ul>
<li>同一个文件里能写多个 <code>2:</code>：gdt.rs:86 一个、idt.rs:155 一个，各自的 <code>2f</code> / <code>2b</code> 只认自己附近那个。</li>
<li>动机是给「短命的跳转目标」起名（loop2、retry3）既费名字又污染符号表。数字标签是用完就扔的写法。</li>
<li>历史来自 PDP-11 时代的 Unix 汇编器，GNU as 继承至今。</li>
<li>代价是不能从远处引用：跨函数或跨大段代码的跳转，还是得用有名字的符号。</li>
</ul></details>
```

**FS/GS 不看 GDT/LDT 的意思吗？**

半对。装载分两步：装选择子时照样查 GDT，做合法性检查；真正寻址时，长模式下 FS/GS 的基址不来自描述符，来自 MSR。所以准确说法是：基址不看 GDT，合法性检查照样看。

```html
<details><summary>两步各干什么的完整画面</summary><ul>
<li>第一步（装载）：<code>mov fs, 0x10</code> 会去 GDT 读那条描述符，查 P 位、type、DPL，把缓存份塞进 FS 的隐藏寄存器。FS/GS 一个不少。</li>
<li>第二步（寻址）：长模式下 FS/GS 的基址来自 <code>IA32_FS_BASE</code> / <code>IA32_GS_BASE</code> 两个 MSR，描述符的 base 字段对它俩无效。</li>
<li>证据就在我们代码里：GDT 没有 FS/GS 专属条目，<code>gdt.rs:89-90</code> 装的是普通数据段描述符。若基址来自描述符（base=0），所有 FS 访问都指到地址 0，TLS 当场挂掉。它们能工作，正因为基址走 MSR。</li>
<li>推论：<code>mov fs</code> 换的是「哪个选择子合法」，不改基址。改基址得写 MSR（将来 TLS 的活儿）。</li>
</ul></details>
```

**SSE 指令是什么？为什么它逼出了 `and rsp, -16`。**

SSE（Streaming SIMD Extensions）是 Intel 1999 年随 Pentium III 加入的指令集扩展，操作一组 128 位寄存器 xmm0~xmm15。x86-64 把 SSE 和 SSE2 定为强制基础，编译器可以自由发射它们。即使你一行汇编没写，Rust 编译产物里也可能有 SSE。

```html
<details><summary>SSE 干什么的、和对齐的关系</summary><ul>
<li>SIMD = 一条指令同时算多个数据：一个 xmm 寄存器装 4 个 float 或 16 个字节，一条 addps 同时加 4 对数。</li>
<li>除了数学运算，SSE 还被编译器当搬运工用：结构体拷贝、memcpy、memcmp 这类操作常被优化成 SSE 批量读写。</li>
<li>关键在指令分两族：movaps/movdqa 这类"对齐版"要求内存地址是 16 的倍数，不对齐直接 #GP；movups/movdqu 是"不对齐版"，慢一点但安全。</li>
<li>编译器只在它能证明对齐时才发对齐版——它的证明就是调用约定里那句「call 时栈 16 对齐」。rsp 对齐了，函数开头用 movaps 往栈上存 xmm 就合法。</li>
</ul></details>
<details><summary>跟我们的 and rsp, -16 怎么接上的</summary><ul>
<li>异常打断的是任意代码，RSP 可能是任何对齐状态。裸 asm 里没有编译器生成的序言帮我们修。</li>
<li><code>idt.rs:152</code> 把 RSP 砍到 16 对齐后再 call，handler 入口看到的栈状态和正常调用约定一致，编译器在 handler 里发 SSE 就安全了。</li>
<li>不修会怎样：handler 里 printk! 的格式化路径可能含编译器生成的 SSE 拷贝，遇到不对齐的栈就 #GP——在 #GP handler 里再 #GP，直接变 double fault。</li>
<li>SSE 的后继是 AVX（256 位 ymm 寄存器）与 AVX-512；对齐要求跟着寄存器宽度涨。x86-64 基础级别只承诺到 SSE2。</li>
</ul></details>
```

**push 的都是 8 字节寄存器，栈不是天然对齐的吗？为什么要 and rsp, -16。**

对齐说的是"地址值是 16 的倍数"，不是"push 了多少字节"。而且 15 × 8 = 120，120 不是 16 的倍数——压完反而把对齐状态挪动了一次。真正的问题更根本：被打断代码的 RSP 值不受我们控制，push 只能平移，砍对齐只能靠自己。

```html
<details><summary>对齐盯的是 rsp 的数值，不是 push 的量</summary><ul>
<li>16 字节对齐 = rsp 的低 4 位全是 0，比如 <code>0x...f0</code>。这是一条关于"地址值"的断言。</li>
<li>push 一次减 8，只让 rsp mod 16 在 0 和 8 之间来回跳。跳几次之后对齐不对齐，取决于起点是 0 还是 8——而起点我们不知道。</li>
<li>15 次 push 一共减 120，120 mod 16 = 8。压完 15 个寄存器后，rsp 的对齐位恰好翻转了一次。</li>
<li>所以不能靠"凑 push 次数"来对齐：凑一次 push 是把 0 变 8、8 变 0，两个方向都可能走错。</li>
</ul></details>
<details><summary>真正的未知量：异常前的 rsp</summary><ul>
<li>异常打断的是任意位置的任意代码，那时的 RSP 可能是 <code>0x...f0</code> 也可能是 <code>0x...f8</code>，我们不掌握。</li>
<li>硬件只约定栈 8 字节对齐，16 字节是调用约定的软件约定，不是 CPU 强制。所以"入口时应该对齐"这条前提不存在。</li>
<li>有趣的是整条路径的总位移是确定的：CPU 压 3~4 个槽 + stub 压 1~2 个 + 我们 15 个，合计 160 字节，恰好是 16 的倍数。也就是说压完后 rsp 的对齐状态和异常前完全一样——状态不变，但状态本身未知。</li>
<li>所以唯一能做的就是把当前值砍到 16 的倍数：<code>and rsp, -16</code>。push 负责移动，and 负责落地。</li>
</ul></details>
```

**`and rsp, -16` 如果 rsp 余 8，砍完栈顶不就指到 r14 了吗？**

不会，砍的方向反了。`and rsp, -16` 是把 rsp 往**低地址**挪，而栈向下长：低地址是还没用过的空闲区，高地址才是压进去的数据。余 8 时砍掉 8 字节，rsp 落在 r15 槽位**下方**的一格空闲区，不是 r14——r14 在 r15 上方，碰不到。

```svg
<svg width="420" height="300" viewBox="0 0 420 300" xmlns="http://www.w3.org/2000/svg" font-family="monospace" font-size="12">
  <rect width="420" height="300" fill="#1e1e28"/>
  <text x="14" y="22" fill="#9aa4b0">↑ 高地址（先压的数据）</text>
  <rect x="120" y="34" width="170" height="30" fill="#32323e" stroke="#667"/>
  <text x="24" y="53" fill="#9aa4b0">CPU 压</text>
  <text x="205" y="53" fill="#dde2ea" text-anchor="middle">rflags / cs / rip</text>
  <rect x="120" y="64" width="170" height="30" fill="#32323e" stroke="#667"/>
  <text x="24" y="83" fill="#9aa4b0">stub 压</text>
  <text x="205" y="83" fill="#dde2ea" text-anchor="middle">error_code / vector</text>
  <rect x="120" y="94" width="170" height="30" fill="#32323e" stroke="#667"/>
  <text x="24" y="113" fill="#9aa4b0">push ×14</text>
  <text x="205" y="113" fill="#dde2ea" text-anchor="middle">rax … r14</text>
  <rect x="120" y="124" width="170" height="30" fill="#4a4030" stroke="#e0b040" stroke-width="1.5"/>
  <text x="24" y="143" fill="#9aa4b0">push ×1</text>
  <text x="205" y="143" fill="#f0d080" text-anchor="middle">r15 ← rdi 指这里</text>
  <rect x="120" y="154" width="170" height="30" fill="none" stroke="#8899aa" stroke-dasharray="5 4"/>
  <text x="24" y="173" fill="#9aa4b0">and 落地</text>
  <text x="205" y="173" fill="#a9b6c4" text-anchor="middle">8 字节填充（仅 rsp≡8 时）</text>
  <rect x="120" y="184" width="170" height="30" fill="#32323e" stroke="#667"/>
  <text x="24" y="203" fill="#9aa4b0">call 压</text>
  <text x="205" y="203" fill="#dde2ea" text-anchor="middle">返回地址</text>
  <text x="300" y="143" fill="#7fa7c9">rsp（and 前）</text>
  <text x="300" y="173" fill="#7fa7c9">rsp（and 后）</text>
  <text x="300" y="203" fill="#7fa7c9">rsp（call 后）</text>
  <text x="14" y="238" fill="#9aa4b0">↓ 低地址（空闲区）：往下砍永远碰不到上面的数据</text>
  <text x="14" y="262" fill="#8b95a3">mov rdi, rsp 在 and 之前（idt.rs:149 vs 152），</text>
  <text x="14" y="280" fill="#8b95a3">帧指针已存进 rdi，rsp 之后怎么挪都不影响 handler 读帧。</text>
</svg>
```

```html
<details><summary>那 8 字节填充去哪了，要不要清理</summary><ul>
<li>它是一格死空间。<code>call</code> 会把返回地址压进 rsp−8，正好用上；r15 槽位往上原封不动。</li>
<li>handler 类型上是 <code>-> !</code>（<code>idt.rs:266</code>），不返回，所以也不需要把这格填回来。</li>
<li>真正依赖对齐的是 handler 自己的序言：编译器假设入口时 <code>rsp ≡ 8 (mod 16)</code>。我们 <code>and</code> 到 16 的倍数后 call 再压 8 字节，入口恰好余 8，与约定严丝合缝。</li>
</ul></details>
```

**取整之后，r15 的值不就变了？r14 等也左移了吗？**

不变，也不移。`and rsp, -16` 只改 rsp 这一个寄存器里的数值，对内存一个字节都不写。r15 的值还躺在原来那格内存里，位置和内容都没动；动的只是"rsp 指到哪儿"。

```svg
<svg width="480" height="230" viewBox="0 0 480 230" xmlns="http://www.w3.org/2000/svg" font-family="monospace" font-size="12">
  <rect width="480" height="230" fill="#1e1e28"/>

  <!-- and 前 -->
  <text x="14" y="24" fill="#9aa4b0">and 前：push 完 15 个寄存器</text>
  <rect x="14" y="36" width="120" height="28" fill="#2a3a4a" stroke="#7fa7c9"/>
  <text x="74" y="54" fill="#cfe0ee" text-anchor="middle">rsp = 0x…78</text>
  <line x1="134" y1="50" x2="236" y2="50" stroke="#7fa7c9" stroke-width="1.5"/>
  <polygon points="236,50 228,45 228,55" fill="#7fa7c9"/>
  <rect x="180" y="40" width="56" height="26" fill="none" stroke="#8899aa" stroke-dasharray="4 3"/>
  <rect x="240" y="40" width="56" height="26" fill="#4a4030" stroke="#e0b040"/>
  <text x="268" y="57" fill="#f0d080" text-anchor="middle">r15</text>
  <rect x="300" y="40" width="56" height="26" fill="#32323e" stroke="#667"/>
  <text x="328" y="57" fill="#dde2ea" text-anchor="middle">r14</text>
  <rect x="360" y="40" width="56" height="26" fill="#32323e" stroke="#667"/>
  <text x="388" y="57" fill="#dde2ea" text-anchor="middle">r13</text>
  <text x="430" y="57" fill="#8899aa">…</text>
  <text x="186" y="80" fill="#77828f" font-size="10">0x…70（空闲）</text>
  <text x="252" y="80" fill="#77828f" font-size="10">0x…78</text>
  <text x="312" y="80" fill="#77828f" font-size="10">0x…80</text>
  <text x="372" y="80" fill="#77828f" font-size="10">0x…88</text>

  <!-- and 后 -->
  <text x="14" y="116" fill="#9aa4b0">and 后：只改 rsp 的数值，格子原地不动</text>
  <rect x="14" y="128" width="120" height="28" fill="#2a3a4a" stroke="#7fa7c9"/>
  <text x="74" y="146" fill="#cfe0ee" text-anchor="middle">rsp = 0x…70</text>
  <line x1="134" y1="142" x2="206" y2="142" stroke="#7fa7c9" stroke-width="1.5"/>
  <polygon points="206,142 198,137 198,147" fill="#7fa7c9"/>
  <rect x="180" y="132" width="56" height="26" fill="none" stroke="#8899aa" stroke-dasharray="4 3"/>
  <text x="208" y="149" fill="#a9b6c4" text-anchor="middle" font-size="10">填充</text>
  <rect x="240" y="132" width="56" height="26" fill="#4a4030" stroke="#e0b040"/>
  <text x="268" y="149" fill="#f0d080" text-anchor="middle">r15</text>
  <rect x="300" y="132" width="56" height="26" fill="#32323e" stroke="#667"/>
  <text x="328" y="149" fill="#dde2ea" text-anchor="middle">r14</text>
  <rect x="360" y="132" width="56" height="26" fill="#32323e" stroke="#667"/>
  <text x="388" y="149" fill="#dde2ea" text-anchor="middle">r13</text>
  <text x="430" y="149" fill="#8899aa">…</text>

  <!-- 结论 -->
  <text x="14" y="196" fill="#8b95a3">内存地址从左到右递增。两行里格子的位置与内容完全相同，</text>
  <text x="14" y="214" fill="#8b95a3">变的只有 rsp 这个寄存器里的数值（0x…78 → 0x…70）。</text>
</svg>
```

```html
<details><summary>rsp 是游标不是货架：为什么不存在"左移"</summary><ul>
<li>压栈时寄存器的值被复制进内存格，之后各归各位。rsp 只记录"栈顶是哪格"，是一个游标。</li>
<li><code>and</code> 是纯粹的寄存器运算：把 rsp 的低 4 位清零。没有访存指令执行，内存一个字节都没碰。</li>
<li>内存格子的地址是绝对的（比如 <code>0x1234_5678</code>），不是"相对 rsp 的第几格"。所以改 rsp 不产生任何移位。</li>
<li>handler 也不通过 rsp 读帧：帧指针早已存进 rdi（<code>idt.rs:149</code>），rsp 之后挪到哪儿都与读帧无关。</li>
</ul></details>
```

**函数参数不是通过栈传的吗？为什么 handler 的参数在 rdi 里。**

那是 32 位时代的 cdecl 约定。x86-64 的 System V 约定改成寄存器优先：前六个参数依次走 rdi、rsi、rdx、rcx、r8、r9，栈只在参数超过六个或放不下时才用上。`mov rdi, rsp` 就是把帧指针塞进第一个参数寄存器。

```html
<details><summary>32 位与 64 位的参数传递对比，以及栈什么时候还上场</summary><ul>
<li>32 位 cdecl：参数确实全在栈上。调用方从右往左 push，被调方按 <code>[ebp+8]</code> 这样的偏移读。以前在教材里见过的就是这种。</li>
<li>64 位 SysV：寄存器从 8 个涨到 16 个，前六个参数放寄存器，省掉一圈内存往返。栈留给第 7 个及以后的参数、按值传的大结构体、以及寄存器不够时的溢出。</li>
<li>我们的 handler 只有一个参数，正好落在 rdi，一条 <code>mov</code> 完成交接（<code>idt.rs:149</code>）。</li>
<li>这也是 <code>ExceptionFrame</code> 按引用传的原因：<code>extern "C" fn exception_handler(frame: &ExceptionFrame)</code>（<code>idt.rs:266</code>）里 <code>&T</code> 就是一个指针，编译后正好占一个 rdi，栈上的数据原地就是结构体，零拷贝。</li>
<li>同一个约定在 x86-64 Linux 用户态也一样：printf 的参数同样先走 rdi~r9。这不是内核特有规矩。</li>
</ul></details>
```

**为什么是 16 字节对齐？不对齐会怎样。**

16 是 xmm 寄存器的宽度：128 位 = 16 字节。SSE 的对齐版指令一次把整个 xmm 搬进搬出内存，硬件按「单次、不跨行」实现这条通路。允许任意地址的话，一次 16 字节读写可能横跨两条缓存行甚至两个页。硬件要为这种分裂访问另开一条路，又慢又复杂。对齐版指令干脆把「地址必须是 16 的倍数」写进规格：不满足就直接 #GP，把约束变成编译器与程序员之间的契约。

```html
<details><summary>不对齐会发生什么：不是变慢，是直接 fault</summary><ul>
<li>对齐版指令（<code>movaps</code>、<code>movdqa</code>）遇到不对齐地址，CPU 不做「拆成两次访问」的兼容处理，直接抛 #GP（向量 13）。</li>
<li>在我们的场景里这条链特别毒：#GP 发生在 exception_handler 内部 → 异常处理中再异常 → 升级为 double fault（#DF）→ #DF handler 同样要压栈写显存，栈还是不对齐 → triple fault → qemu 整机重启。</li>
<li>也就是说 <code>and rsp, -16</code> 省掉的不是一点性能，而是「异常处理本身变成异常源」这条死循环。</li>
<li>对照组是 <code>movups</code>/<code>movdqu</code>：不对齐也能跑，只是跨缓存行时多一次访问。编译器只在约定保证不了对齐时才用它。</li>
</ul></details>
<details><summary>为什么是 ABI 约定而不是 CPU 强制</summary><ul>
<li>CPU 只强制对齐版 SSE 指令的操作数对齐；普通的 <code>push</code>、<code>mov</code> 对 rsp 没有 16 字节要求。</li>
<li>「栈在 call 边界 16 对齐」是 System V ABI 写死的软件约定：每个调用方负责对齐，每个被调方才敢放心发对齐版指令。</li>
<li>ABI 停在 16 而没有跟着 AVX 涨到 32/64：约定一旦公布就成历史包袱，寄存器变宽后编译器对更宽的对齐改用动态处理（函数里自己砍 rsp）。</li>
<li>裸 asm 是我们自己的地盘，编译器不生成序言，这条约定只能手工补上——这正是 <code>idt.rs:152</code> 存在的理由。</li>
</ul></details>
```

**Bite1 的代码里哪里用到 SSE、需要 16 对齐？**

实测答案：一条都没有。`objdump -d` 整个内核，`movaps` / `movdqa` / `movups` / `movdqu` 与 `xmm` 的命中数都是 0，连 memcpy / memset 符号都没有。`and rsp, -16` 并非现有代码逼出来的，而是守 ABI 契约的防御写法。

```html
<details><summary>验证命令与"现在没有"不等于"永远不用"</summary><ul>
<li>验证：<code>objdump -d kernel/target/x86_64-unknown-none/debug/kernel | grep -cE 'xmm'</code>，输出 0。这覆盖 handler、printk! 的整条调用链。</li>
<li>什么时候会突然出现：超过 16 字节的结构体按值拷贝、memcpy / memset 大对象、release 模式的向量化、将来引入浮点的格式化路径。这些全是编译器决策，一行源码不动也可能改变。</li>
<li>因果关系是反的：不是「代码用了 SSE 所以要对齐」，而是「裸 asm 先守约定，编译器才永远安全」。遵守在前，用到在后。</li>
<li>这次 0 命中的价值正在于此：对齐的论证是纯契约论证。哪天 grep 出非零，没有 <code>and</code> 的版本会当场 #GP，不需要等到那一天。</li>
</ul></details>
```
