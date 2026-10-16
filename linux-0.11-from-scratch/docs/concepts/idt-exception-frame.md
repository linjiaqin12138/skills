# IDT 与异常栈帧：CPU 崩溃时把现场交给了谁

> 概念深读（M2 Bite1 背景卡片承诺）。面向第一次接触 x86 中断/异常的读者。
> 读完你应能回答：异常和中断有什么区别；IDT 是什么数据结构；CPU 跳到 handler 前自动做了什么；为什么有的异常多一个 error code；interrupt gate 和 trap gate 差在哪。

## 一句话版本

**IDT（Interrupt Descriptor Table，中断描述符表）** 是内存里一张 256 行的表格。CPU 遇到异常或中断时，按"向量号"查这张表，跳到对应行登记的处理函数。跳之前，CPU 会把当前现场（rip/cs/rflags 等）自动压到栈上，交给处理函数检查。

## 异常 vs 中断：谁敲的门

x86 把 256 个向量分成几类来源：

- **异常（exception）**：CPU 执行指令时自己发现的问题。除以零是向量 0（#DE），缺页是向量 14（#PF），非法指令是向量 6（#UD）。同步发生——同一条指令重跑还会再犯。
- **外部中断（interrupt）**：CPU 之外的设备敲门。定时器、键盘、磁盘走这条路，通常从向量 32 起。异步发生——和当前执行的指令无关。
- **软件中断**：程序用 `int n` 指令主动触发。Linux 0.11 的系统调用就是 `int 0x80`。

三者走的是同一张 IDT、同一套跳转机制，区别只在向量号来源。x86 还把异常细分为 fault（可修复，rip 指向犯错指令本身，如 #PF）、trap（事后报告，rip 指向下一条，如断点 #BP）、abort（不可恢复，如 #DF）。

## 最小可运行示例：你每天都在间接使用异常

用户态程序触发 CPU 异常时，是内核的 IDT handler 接住它，再包装成信号投递给进程。下面这段 C 程序注册 SIGFPE handler 后真除零，能看到内核转交的寄存器现场：

```c
// gcc -O0 sigfpe.c -o sigfpe && ./sigfpe
#define _GNU_SOURCE
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <ucontext.h>

static void handler(int sig, siginfo_t *info, void *uctx) {
    (void)sig; (void)info;
    ucontext_t *uc = uctx;
    /* Linux 把 CPU 保存的现场装进 ucontext 交给用户态 handler */
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
    printf("%d\n", a / b);   /* 真除零：CPU 产生 #DE */
    return 0;
}
```

实测输出（rip 地址每次不同）：

```
caught SIGFPE (CPU exception #DE)
rip=0x55a2e9d052c3  rax=0x1  rdx=0
```

要点：用户态的 `sigaction` 表，在内核里的对应物就是 IDT。区别是内核自己就是最后一道——内核里发生的异常没有更内层的东西帮忙接，接不住就是三连重启。

## IDT 项：16 字节的门描述符

64 位模式下每行 IDT 项 16 字节，关键字段：

- **offset**（64 位，拆三段存）：处理函数的地址。
- **selector**：处理函数所在的代码段选择子——填自己 GDT 的内核代码段（`0x08`）。
- **type**：门类型。`0b1110` 是 interrupt gate，`0b1111` 是 trap gate。
- **P 位**：1 = 本项有效。P=0 的项是空洞，CPU 查到它等同没有 handler。

**interrupt gate 与 trap gate 的唯一差别**：interrupt gate 进门时 CPU 自动清 IF 标志（关可屏蔽中断），trap gate 不动 IF。异常处理期间不想被外部中断打断，就用 interrupt gate。

## CPU 自动压栈的帧：现场的由来

CPU 通过 IDT 跳 handler 前，自动往当前栈上压入（无特权级切换时）：

```
高地址  rflags
        cs
        rip        ← 指向犯错的那条指令（fault 类）
低地址  [error_code]  ← 只有部分向量有
```

**error code 不是每个异常都有**。向量 8、10~14、17、21 由 CPU 压入错误码（如 #PF 的错误码说明是读还是写、页不存在还是权限不够）；其余向量没有。做统一 handler 时，常见做法是给没错误码的向量由入口代码补压一个占位 0，让所有向量的栈帧布局一致。

**特权级切换时多压两项**：如果异常发生在 ring3 而 handler 在 ring0，CPU 还会压入旧的 rsp 和 ss，并切换到 TSS 指定的内核栈。同特权级异常没有这两项——handler 想知道当时的 rsp，只能从帧上 rip 字段的位置反推（rip 上方三个 u64 之后就是压栈前的栈顶）。

## 返回路径：iretq

handler 干完活想回到被打断的代码，用 `iretq`：它把栈上的 rip/cs/rflags 弹回去，CPU 从犯错指令（fault）或下一条指令（trap）继续执行。弹之前要先清掉栈上自己压的东西和 error code。缺页处理就靠这条路径：修好页表，`iretq` 回去重跑那条访存指令，这次就能命中。

本项目的 M2 Bite1 不返回——打印完现场直接 `hlt` 停机。`iretq` 路径是 M9 写时复制的待办（偏差簿 D16）。

## 没有 handler 会怎样：三连重启

CPU 查 IDT 时出错（项是空洞、表本身没装），会升级成更严重的异常：异常 → 处理异常时又出错 → **double fault**（#DF，向量 8）→ 处理 #DF 又出错 → **triple fault**，CPU 直接复位。qemu 里表现为虚拟机不断重启，串口日志反复打印开头几行。这就是"内核没有自己的 IDT 时，任何崩溃都等于整机重启"的机制原因。

## 和本项目的对应关系

| 概念 | 本项目代码 |
|---|---|
| 16 字节门描述符 | `kernel/src/idt.rs` 的 `IdtEntry` |
| interrupt gate type=0b1110 | `kernel/src/idt.rs` 的 `IdtEntry::interrupt_gate` |
| 256 项表、P=0 空洞 | `kernel/src/idt.rs` 的 `static mut IDT` 与 `IdtEntry::missing` |
| 异常名表 | `kernel/src/idt.rs` 的 `EXCEPTION_NAMES` |
| 统一栈帧（占位 0） | `kernel/src/idt.rs` 的 `exception_entry!` 宏 |
| CPU 自动压栈 + 手工压 15 寄存器 | `kernel/src/idt.rs` 的 `ExceptionFrame` 与 `common_exception_entry` |
| rsp 反推 | `kernel/src/idt.rs:268-270` |
| lidt 装表 | `kernel/src/idt.rs` 的 `init()` |

## 深入阅读

- Intel SDM Vol.3 第 6 章：6.10 IDT、6.12 异常与中断分类、6.14.1 门描述符、6.15 向量清单
- OSDev Wiki：[IDT](https://wiki.osdev.org/IDT)、[Exceptions](https://wiki.osdev.org/Exceptions)、[Triple Fault](https://wiki.osdev.org/Triple_Fault)
- 本仓库代码：`kernel/src/idt.rs`（302 行，全文可读完）
