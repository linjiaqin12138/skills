# Phase 2 · 自底向上实现路径（v2，2026-10-03 架构升级）

> v2 变更：目标架构 i386 → **x86_64**（用户决策）。连锁影响：boot 走 Limine（直接进 64 位长模式）、跑不了原版 a.out 二进制（用户态改为自写 Rust 小程序）、段槽位机制整体退役、TSS 硬件切换不可行自动转软件切换、syscall ABI 从 int 0x80 换成 syscall 指令。

终态：我们的 Rust 内核（x86_64）在 qemu 里启动，framebuffer 图形控制台 + 键盘，挂载 hdc-0.11.img 读出原版 MINIX 文件系统的文件，跑起**我们自己用 Rust 写的 shell**（initramfs 加载），交互执行 echo/ls/cat/mkdir/重定向/管道——与原版的**能力级**功能点对齐（feature-inventory.md，Phase 4 逐项核对）。

## Phase 2 四张项目级决策卡片

### 卡片 A · 架构与 boot：x86_64 + Limine，一次跳过两个时代
- **决策点**：目标 ISA 与引导方式。
- **备选**：(a) i386 + GRUB multiboot（v1 方案）；(b) x86_64 + Limine（所选）；(c) riscv64 + OpenSBI。
- **选择理由**：用户明确要求抛弃历史包袱（2026-10-03）。Limine 把内核直接投进 64 位长模式、分页已开、framebuffer 已备好——实模式、A20、手工建初级页表全部不用写。x86_64-unknown-none 是 Rust 官方 target（连自定义 target JSON 都省了）。选 (b) 而非 (c) 是因为 qemu 的 riscv64 虚拟机没有简单的显卡和键盘，(c) 会丢掉刚定的 framebuffer 图形控制台。
- **放弃的成本**：(a) 的"跑起 1991 年原版 /bin/sh 二进制"没要到——a.out 是 i386 机器码，任何 64 位内核都执行不了，字节级 ABI 对齐这个最强验收锚点让位给能力级对齐；(c) 的概念纯度也没要到——x86_64 仍有 GDT/IDT 等怪癖，RISC-V 才是白板。
- **失效边界**：若未来要做"在真实硬件上跑"或"教学纯净化"（对标 xv6-riscv），再评估 (c)。

### 卡片 B · 用户态程序：自己用 Rust 写，内核 ABI 自定义
- **决策点**：用户程序（sh、ls、cat…）从哪来？
- **备选**：(a) 复用 hdc-0.11.img 原版 a.out 二进制（v1 方案，已被卡片 A 否决）；(b) 自写 Rust 用户态：mini-sh + coreutils-lite，no_std + 我们自己的 syscall 封装（所选）。
- **选择理由**：架构升级后 (a) 物理上不可能；(b) 让 Rust 教学延伸到用户态（同一门语言、第二个上下文），且 ABI 可以设计成干净的 x86_64 惯例（syscall 指令，rax=调用号，rdi/rsi/rdx 传参），调用号数值仍沿用原版 74 项表做对照。
- **备选补充（2026-10-03 用户问后调研）**：(c) 移植现成教学 shell——**xv6 的 sh.c**（499 行 C，kernel 依赖仅 11 个 syscall：fork/exec/pipe/open/close/read/write/dup/wait/exit/chdir，外加 7 个 libc 函数）+ 一层 ~250 行的 C 兼容 shim 翻译到我们的 ABI（open 标志位、exec 补 envp 等）。可行，但会把 C 引入 Rust 教学项目、让"顺便教 Rust"在用户态断档。**默认不选，留作 M12 的可选支线**；xv6 sh.c 本身作为我们写 mini-sh 时的设计参照读物（parsecmd/runcmd 结构是标准教材）。
- **放弃的成本**：(a) 的"1991 年的二进制在我们内核上复活"这个魔幻时刻没要到；以及由此带来的强约束验收（ABI 错一个bit就跑不起来）也一并失去，验收强度降级为能力级。
- **补偿**：hdc-0.11.img 仍然挂载——我们的 MINIX 实现必须能读出它的目录树和文件内容（"能读原版的盘"依然是硬验收），只是不执行里面的程序。
- **失效边界**：无，与卡片 A 共存亡。

### 卡片 C · 架构自主权：机制照学，模块切分重来
- 同 v1（模块按 Rust 惯例 `kernel/src/{mm,sched,fs,drv,…}`；buffer cache、请求队列、MINIX 盘上格式照学）。**例外升级**：原版的"64MB 段槽位"机制在 x86_64 上不存在（长模式废弃段机制），内存隔离改学现代做法——每进程独立页表 + 更高半核（higher-half kernel）。段槽位的故事放进概念文档讲"它当年解决什么、为什么被页表取代"，不实做。
- **放弃的成本**：逐文件对照 reference 的便利性没要到；段槽位的实做体感没要到。

### 卡片 D · 显示路径：线性 framebuffer + 自绘字体（沿用 v1）
- Limine 直接交给我们一块线性 framebuffer，比 v1 方案（GRUB/VBE 自己配）更省。串口日志通道沿用——验收自动化的地基。

## 里程碑路线图（v2）

工作量：S=一轮内，M=2~3 个 Bite，L=3~4 个 Bite。每个里程碑执行时再拆 Bite。

| # | 目标（一句话） | 前置 | 验收标准 | 量 | 类比 |
|---|---|---|---|---|---|
| M0 | 工具链+引导：x86_64-unknown-none target、Limine ISO、`qemu` 启动、**串口打印首行日志** | 环境（Rust✅/Limine✅已备，待装 qemu/xorriso/gcc/make） | `qemu -serial stdio` 捕获到内核首行日志（grep 断言） | S | 给裸机装一条能往外传话的"电报线" |
| M1 | 内核 printk：framebuffer 控制台（Limine 显存 + 8x16 字体渲染 + 滚屏）+ core::fmt | M0 | 屏幕显示多行格式化文本（截图目检）+ 串口同内容日志（grep 断言） | M | 串口是电报，屏幕是霓虹灯 |
| M2 | GDT(扁平)/IDT + CPU 异常接管 + panic 打印 | M1 | 故意除零 → 打印异常名与寄存器现场而非死机 | M | 安全气囊：摔了能报告怎么摔的 |
| M3 | 中断：LAPIC 定时器（100Hz）+ IOAPIC 接键盘 | M2 | jiffies 每秒递增上屏；敲键扫描码→ASCII 回显 | M | 接通心跳和耳朵（M3 出卡：APIC vs 老式 PIC） |
| M4 | 物理页分配器（从 Limine memory map 建 mem_map） | M3 | 分配/释放/耗尽断言全过 | S | 内存切成 4KB 格子间发号牌 |
| M5 | 分页接管：自建 PML4 四级页表 + 更高半核 + 缺页处理 | M4 | 访问未映射地址打印 page fault 而非三连重启 | M | 每个地址发通行证，没证拦下问话 |
| M6 | 进程骨架：task 表 + 软件上下文切换（保存/恢复寄存器+切栈） | M5 | 两个内核任务交替打印，切换无损 | L | CPU 分身术第一版：手动换人 |
| M7 | 调度器：时间片轮转 + 睡眠/唤醒 + LAPIC 定时器抢占 | M6 | 打印交错证明抢占生效 | M | 分身术自动化：闹钟到强制换人 |
| M8 | 用户态 ring3 + syscall/sysret 系统调用门 + 调用表 | M7 | 用户态程序 write 上屏、exit 回收 | L | 从内核自嗨到开门营业 |
| M9 | fork + 写时复制（四级页表版） | M8 | 子进程改变量父进程不变；页计数证明 COW | L | 复印机只复印目录，改哪页才印哪页 |
| M10 | ATA(PIO) 硬盘驱动 + buffer cache + 请求队列 | M9 | 读出 hdc-0.11.img 第 0 块与 hexdump 字节一致 | L | 接仓库 + 前台缓存货架 |
| M11 | MINIX 文件系统（读）：super/inode/namei/read，挂载 hdc-0.11.img | M10 | 读出镜像里 /etc/rc 内容并打印；`ls -R /` 输出镜像精确清单 | L | 仓库之上建图书索引 |
| M12 | exec + init：ELF64 加载器，initramfs 里起跑我们的 Rust shell | M11 | **屏幕上出现 shell 提示符，能跑内建命令** | L | 交房：第一个用户住进来 |
| M13 | tty 行规程：回显/退格/行缓冲 | M12 | 交互敲命令（含输错退格）全正常 | M | 电报机升级成对话窗口 |
| M14 | MINIX 写 + 管道 + 信号基础（Ctrl-C） | M13 | `echo a > f; cat f; ls \| cat`、Ctrl-C 全通 | L | 从能看到能改、能组合 |
| M15 | 收敛：coreutils-lite 补齐 + 对照功能清单逐项回归 | M14 | feature-inventory P0 全过 | M | 查漏补缺 |

### 里程碑大小自检（上限：新概念≤2 / 模块≤5 / 决策卡≤3）
- 全部满足；M6、M8、M12 最重，执行时若超上限拆 a/b。

### 直接下载/使用、不手搓的部分
- `hdc-0.11.img` 根文件系统（挂载读数据，证明 MINIX 兼容；不执行其中二进制）
- Limine 引导器二进制（已克隆 `reference/limine` v10 binary 分支）
- Rust core 库（rust-src）；用户态二进制格式用编译器自带的 ELF64（不自造格式）

### 简化项与预定收敛点（见 deviations.md 台账）
1. M0~M9 无磁盘无文件系统 → M10/M11 收敛
2. M3 键盘只做扫描码→ASCII 直显，无行规程 → M13 收敛
3. M12 前 syscall 只实现当里程碑验收所需子集 → M15 收敛（调用号对照原版 74 项表）
4. M12 用户程序从 initramfs 起步，MINIX 可写后才落盘 → M14 收敛
5. 原版 floppy / math 模拟 / serial 外设 / a.out 加载器：标 Phase 4 裁决（大概率永久豁免）
