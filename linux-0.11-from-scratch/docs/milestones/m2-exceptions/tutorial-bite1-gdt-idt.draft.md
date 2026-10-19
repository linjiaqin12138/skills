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
