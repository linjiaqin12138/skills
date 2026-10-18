# M2 验收档案（GDT/IDT + CPU 异常接管）

## Bite1：GDT 扁平接管 + IDT + CPU 异常接管（2026-10-16 首次通过，2026-10-18 复跑通过）

**验收命令**：`make check-m2-b1`（exit 0）

**涉及代码文件**：
- `kernel/src/gdt.rs`（扁平 GDT 三项，lgdt + retfq 重载段寄存器；描述符预置 A=1 避免 CPU 回写只读 .rodata 触发 #PF）
- `kernel/src/idt.rs`（256 项 IDT，向量 0~31 interrupt gate；宏生成 32 个 naked stub 统一帧布局；common_exception_entry 压 15 寄存器；exception_handler 打印现场后 hlt）
- `kernel/src/main.rs`（gdt::init → idt::init → asm `div` 触发 #DE；M0/M1 打印保留）
- `Makefile`（`check-m2-b1`）

**2026-10-18 实测输出**（serial.log 尾部）：

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

**断言**：`M0: kernel alive` 恰好 1 次（无三连重启）；`M2: gdt ok`/`M2: idt ok`；`EXCEPTION #0`+`divide error`；`grep -E 'M2: rip=0x[0-9a-f]+'` 命中；M1 全部回归 marker 保留；screendump 尺寸 1440015 字节。

**现场真实性交叉验证**：cs=0x0008 正是新 GDT 代码段选择子；rax=rdx=0 对应 asm 里的 xor；rflags=0x10046 的 ZF/PF 位与 xor 后状态吻合。

**回归**：`make check-m1-b2` 于 2026-10-16 通过（coder 实测）。

**架构图**：`assets/dst/arch.svg`（顶层权威图）、`assets/dst/arch-idt.svg`（IDT 分层子图）、`assets/dst/arch-diff.svg`（M1→M2 变动）。对应图源为 `assets/src/arch*.json`；fireworks validate / render / check 均通过，Chrome 目检确认文字、箭头与分层关系清晰。

**教程与概念页复验（2026-10-18）**：tutorial 以 `--style strict` 重渲，STE 0 警告；共 8 个面板，含 4 道应用型选择题和折叠解析。`gdt-long-mode.html` 与 `idt-exception-frame.html` 均为 8 个面板、4 道选择题；浏览器确认图片非空且无空白面板。
