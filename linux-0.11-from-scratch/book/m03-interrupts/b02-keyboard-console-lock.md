---
template: doc
theme: paper
title: M3 Bite2 · IOAPIC 键盘 + console 关中断锁
subtitle: IRQ1 经 IOAPIC 进向量 33；中断里回显字符；IrqLock 挡住 tick 打印与回显的互相打断
source: linux-0.11-from-scratch · M3
---

Lead: 键盘 IRQ1 经 IOAPIC 变成向量 33。handler 读扫描码、查出 ASCII、在中断里 `printk!` 回显。全局 console 换成 `IrqLock`：进门 `cli`，出门按进入时的 IF 恢复。定时器仍 100 Hz；两条外部中断共用同一份可返回入口与统一 EOI。

## 问题

Bite1 已经有了自己到来的 tick：每秒一行 `M3: jiffies=…`。但屏幕还不能对键盘有反应。

键盘不是 LAPIC 本地源。它是外设线 IRQ1，PIC 已被整片屏蔽，所以必须经 IOAPIC 把这条线投到本核，再进 IDT。进门之后还有第二个故障：键盘 handler 要在中断上下文 `printk!` 回显，而主流程的 jiffies 打印也走同一份 console。两边同时改光标，就会花屏或 Rust 别名。

本课交付：

- IOAPIC 把 IRQ1 路由到向量 33
- 按下可打印键时，串口与屏幕出现对应 ASCII
- 全局 console 用关中断临界区；tick 与回显并存时不 panic
- 定时器与键盘共享同一份可返回入口，EOI 只发一次

## 背景

**IOAPIC 管外设线。** LAPIC 定时器不经 IOAPIC；键盘必须经它。本课约定：QEMU 默认机上 IOAPIC 物理基址写死为 `0xFEC00000`，把 GSI 1（键盘 IRQ1）配成 Fixed 投递、向量 33，目标为本核 LAPIC ID。深入见 [PIC、LAPIC 与 IOAPIC](../concepts/apic-vs-pic.html)。

**黑盒：不查 MADT。** ACPI MADT / ISO 的作用是发现 IOAPIC 基址与中断重定向。本课约定写死 `0xFEC00000`（偏差 D18）；解析过程不展开。设备增多、换机型时再回访。

**IrqLock：关中断等于独占。** 单核上，另一个能碰 console 的执行流只有 IRQ handler。`IrqLock::with` 进门先读 RFLAGS.IF 再 `cli`，出门只在「进来时 IF 为 1」才 `sti`。中断门进门时 IF 已是 0，handler 里拿锁属于嵌套关中断，释放时不得误开 IF。见 [关中断临界区](../concepts/irq-critical-section.html)。

**PS/2 只读两个端口。** `0x60` 是数据口，`0x64` 是状态口。本课只读：状态 bit 0 表示输出缓冲有字节；扫描码集 1 里 bit 7 置位是松开（break），否则是按下（make）；`0xE0` 开头的扩展键整段丢掉。见 [PS/2 键盘](../concepts/ps2-keyboard.html)。

**黑盒：不写 `0x64` 发命令。** 往 `0x64` 写能改控制器（自检、开关端口等）。本课约定只读，沿用固件与 QEMU 默认状态；需要发命令时再回访。

**两条外部中断共用骨架。** `timer_entry` / `keyboard_entry` 只压各自向量号，都跳进 `common_irq_entry`；Rust 侧 `irq_dispatch` 按向量找驱动，末尾统一写 LAPIC EOI。这是本课的设计落点，不另开概念页。

## 设计

![M3 顶层：键盘经 IOAPIC 进向量 33；定时器与键盘共享 common_irq_entry / irq_dispatch；console 经 IrqLock](arch.svg)

相对 Bite1 的模块与边：新增 `ioapic`、`keyboard`、`IrqLock`，以及共享的 `common_irq_entry` + `irq_dispatch`。对照见 [架构差异图](figures/arch-diff.svg)。

### 启动时多两步

```flow TB
[IDT 装好向量 32 与 33] -> [LAPIC 定时器 + sti]
[LAPIC 定时器 + sti] -> [排空 PS/2 输出缓冲]
[排空 PS/2 输出缓冲] -> [IOAPIC 路由 GSI1 → 向量 33]
[IOAPIC 路由 GSI1 → 向量 33] -> [report_jiffies 循环]
```

`keyboard::init` 必须在 IDT 已写入向量 33 之后调用（`impl/kernel/src/main.rs:70-74`）。残留字节不清掉，输出缓冲一直满着，控制器不会再拉新的 IRQ1。

### 一次按键怎么走

```sequence
participants: 键盘, IOAPIC, LAPIC, CPU, common_irq_entry, irq_dispatch, keyboard handler, console
键盘 -> IOAPIC: 拉起 IRQ1
IOAPIC -> LAPIC: 按表项投递向量 33
LAPIC -> CPU: 可屏蔽中断
CPU -> common_irq_entry: IDT[33] 经 keyboard_entry 进公共出口
common_irq_entry -> irq_dispatch: 传入向量号 33
irq_dispatch -> keyboard handler: 读 0x60 并 decode
keyboard handler -> console: IrqLock 内 printk
irq_dispatch -> LAPIC: 统一 EOI
common_irq_entry --> CPU: iretq
```

定时器路径同一套：`timer_entry` 压入 32，`irq_dispatch` 调 `lapic::timer_interrupt`（仍只加 jiffies），然后同一处 EOI。主流程的 `printk!("M3: jiffies=…")` 与键盘回显都进 `CONSOLE.with`；一边持锁时另一边进不来。

## 架构决定

### 决定一：console 用关中断临界区

[decisions/0003](../../decisions/0003-console-irq-safe-lock.md)

选了什么。`printk!` 与 console 初始化走同一把 `IrqLock`：拿锁前 `cli`，释放时按进入时的 IF 恢复。键盘 handler 与主流程（含 jiffies 打印）都经此路径。

为什么。本课要证明「中断上下文能安全打印」，并按承诺收敛 D15。

放弃的成本。把扫描码丢进缓冲、主流程再打印会更短中断路径、本课也可少改 console——我们没要这份简单。引入可睡眠互斥或带嵌套计数的完整自旋锁，多核以后更省事——我们没要这份超前完备。

失效边界。多核，或 console 持锁期间需要开中断等待时，关中断独占不够用。

### 决定二：IOAPIC 固定基址路由键盘

[decisions/0004](../../decisions/0004-ioapic-keyboard-irq.md)

选了什么。IOAPIC MMIO 固定 `0xFEC00000`；IRQ1 → Fixed、向量 33、目标本核 LAPIC ID。IDT 再装一条可返回入口，与 timer 共享保存 / 恢复 / `iretq` 骨架。只回显扫描码集 1 的 make code；break、`0xE0` 扩展键、修饰键本课不做。

为什么。与 Bite1「屏蔽 PIC、走 APIC」同一条故事线，并兑现「键盘走 IOAPIC」。

放弃的成本。重新启用 8259 接键盘改动面更小——我们没要这份捷径。解析 ACPI MADT / Limine 拓扑动态找 IOAPIC，换机型更稳——我们没要这份可移植性。

失效边界。IOAPIC 不在该基址、键盘不在 GSI 1、或多 IOAPIC 时，必须改为查表发现。

## 代码领读

**入口只压向量号，骨架只有一份**（`impl/kernel/src/idt.rs:293-321`）。

```rust title="idt.rs · sketch"
macro_rules! irq_entry {
    ($name:ident, $vector:expr) => { /* push 向量号; jmp common_irq_entry */ };
}
irq_entry!(timer_entry, lapic::TIMER_VECTOR);       // 32
irq_entry!(keyboard_entry, keyboard::KEYBOARD_VECTOR); // 33

extern "C" fn irq_dispatch(vector: u64) {
    match vector as u32 {
        lapic::TIMER_VECTOR => lapic::timer_interrupt(),
        keyboard::KEYBOARD_VECTOR => keyboard::interrupt(),
        _ => unreachable!(),
    }
    lapic::end_of_interrupt(); // EOI 只在这里
}
```

`common_irq_entry`（`idt.rs:245-289`）保存 15 个通用寄存器、`call irq_dispatch`、恢复、丢掉向量号、`iretq`。两个 stub 禁止各抄一套。`timer_interrupt` 不再自己写 EOI——和键盘一样交给分发层。

**Rust 语法。** `macro_rules!` 是声明宏：这里用它生成两个几乎相同的裸入口，只差向量常量。`match … as u32` 把传入的 `u64` 向量号转成与驱动常量同宽再分支。`unreachable!()` 表示「按设计进不来」；真进来会 panic。

**键盘 handler：读到空为止，再 decode**（`impl/kernel/src/keyboard.rs:45-71`）。

```rust title="keyboard.rs · sketch"
pub fn interrupt() {
    while inb(STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
        let scancode = inb(DATA_PORT);
        if let Some(ch) = decode(scancode) {
            printk!("{}", ch as char); // 中断上下文打印
        }
    }
}
```

`decode` 丢掉 `0xE0` 及其后一字节、丢掉 break（最高位 1）、查 `SCANCODE_TO_ASCII` 表；表项为 0 的键（Esc / 修饰键等）不回显。`printk!` 会进下面的锁。

**console 整次格式化都在锁内**（`impl/kernel/src/console.rs:95-113`，`irq_lock.rs:33-47`）。

```rust title="console + IrqLock · sketch"
static CONSOLE: IrqLock<Option<Console>> = IrqLock::new(None);

pub fn _print(args: fmt::Arguments) {
    CONSOLE.with(|slot| {
        if let Some(cons) = slot {
            let _ = cons.write_fmt(args);
        }
    });
}

// IrqLock::with：was_enabled = IF; cli; f(&mut value); if was_enabled { sti }
```

`IrqLock<T>` 是泛型：`T` 这里是 `Option<Console>`——还没 `init` 时是 `None`，`printk!` 静默丢弃。`with` 接受一个闭包 `f`，在关中断期间把内部 `&mut T` 借给它。**关键规则是还原 IF，不是无条件 `sti`。**

**IOAPIC：先写高半（目标），再写低半（向量）**（`impl/kernel/src/ioapic.rs:26-36`）。

```rust title="ioapic.rs · sketch"
pub fn route(gsi: u8, vector: u8, destination_lapic_id: u8) -> bool {
    let Some(base) = mmio::map_page(IOAPIC_PHYSICAL_BASE) else { return false };
    let register = REDIRECTION_TABLE_BASE + 2 * gsi as u32;
    write_register(base, register + 1, (destination_lapic_id as u32) << 24);
    write_register(base, register, vector as u32 & 0xff); // 低半一写就生效
    true
}
```

MMIO 页走与 LAPIC 同一套 `mmio::map_page`（池容量 4，够两张设备页）。间接访问：先把寄存器编号写入 IOREGSEL，再写 IOWIN。低半含 mask 位，一落笔该项就生效，所以目标要先写好。

## 验收

在仓库根目录 `linux-0.11-from-scratch/` 的**宿主机**上操作。约定：编译与打 ISO 由 `make` 自动 `docker exec` 进构建容器 `linux011-rust-build` 完成；qemu 在宿主机跑。**不必**手动 `docker exec -it` 进容器敲命令。首次若镜像不存在，先在 `impl/` 执行一次 `make toolchain`。

**1. 构建 ISO**

```sh
make -C impl build
```

**2. 图形窗口观察（推荐）**

```sh
make -C impl run
```

启动后先等串口 / 屏幕出现 `M3: jiffies=100`（约 1 秒），再在 qemu 窗口里连续敲 `h` `e` `l` `l` `o`。应看到这五个字母回显；之后继续出现 `jiffies=200`、`300`…。看够后退出 qemu。

**3. 需要用 monitor 注入按键时**

仍在宿主机、`impl/` 下启动（若 `qemu-system-x86_64` 不在 `PATH`，换成 `~/.local/bin/qemu-system-x86_64`）：

```sh
cd impl
rm -f build/serial.log build/m3-bite2.ppm
qemu-system-x86_64 -m 128M -boot d -display none -vga std -net none \
  -monitor stdio \
  -serial file:build/serial.log \
  -drive file=../upstream/hdc-0.11.img,format=raw,if=ide \
  -cdrom build/kernel.iso
```

先等串口文件里出现 `M3: jiffies=100`（可另开一个终端 `tail -f build/serial.log` 观察）。再在 qemu monitor 提示符下依次输入（每行一条）：

```text
sendkey h
sendkey e
sendkey l
sendkey l
sendkey o
```

再等几秒让 jiffies 再打一两行，然后：

```text
screendump build/m3-bite2.ppm
quit
```

查看日志：

```sh
cat build/serial.log
```

**预期**

- `M0: kernel alive` 恰好一次；保留 `M1` / `M2: gdt ok` / `M2: idt ok`
- 出现 `M3: LAPIC timer 100 Hz` 与 `M3: keyboard IRQ1 -> vector 33`
- 出现 `M3: jiffies=100`、`200`、`300`（约每秒一行）
- 串口中能找到连续子串 `hello`（若紧挨着 jiffies 行，可能看到 `helloM3: jiffies=200`，见坑）
- 无 `PANIC:` / `EXCEPTION`；若做了 screendump，`build/m3-bite2.ppm` 存在且非空

自动验收封装见 `accept/m03-b02.sh`（内部即 `make -C impl check-m3-b2`）。独立 verifier 记录：`tutor/runs/m03-b02.txt`（3/3 通过）。

## 与原版差异与坑

- **D15 已收敛。** 全局 console 换成 `IrqLock` 关中断临界区。见 [偏差簿](../../spec/deviations.md)。
- **D16 部分再收敛。** IRQ32 / 33 均经共享可返回入口；其余外部向量与异常返回路径仍待后续。
- **D17 备注更新。** MMIO 补页已抽到 `mmio.rs` 共用；池上限 4 张（本阶段两张设备页）。M5 收敛。
- **D18 新登记（永久豁免·架构）。** IOAPIC 基址与 GSI 写死，无 MADT/ISO。
- **D19 新登记。** 键盘只回显扫描码集 1 可打印键与回车；无 Esc / 退格 / Tab / 修饰键。
- **D3 仍待收敛。** 只回显，无行规程。

```callout warn sendkey 与 jiffies 打印交错
验收应先等到 `jiffies=100` 再连敲。若在 tick 打印途中注入按键，`hello` 可能被切开。回显本身不带换行时，串口常出现 `helloM3: jiffies=200`——`hello` 与下一行 jiffies 粘在一起，属预期，不是丢字符。
```

## 自测

**1. 主流程正在 `printk!("M3: jiffies=100\n")`，键盘中断到来。IrqLock 会怎样？**

- A. 主流程已 `cli`，键盘中断要等锁释放、IF 恢复后才进 handler；两边不会同时改光标
- B. 键盘 handler 抢占主流程，两份 `printk!` 同时写 console
- C. 键盘中断被丢弃，这个键永远不会回显
- D. 硬件自动把键盘中断推迟到下一个 jiffy

```html
<details><summary>第 1 题答案与解析</summary><p><strong>A</strong>。主流程持锁期间 IF=0，可屏蔽中断不会进 CPU；LAPIC 会记下待处理，等 sti 后再投递。所以不是丢键（C），也不是同时写（B）。</p></details>
```

**2. 键盘 handler 里调用 `IrqLock::with` 时，退出该不该无条件 `sti`？**

- A. 不该。中断门进门时 IF 已是 0；应按进入时的 IF 恢复，否则会在 handler 未完成时放进下一个中断
- B. 该。否则 `iretq` 回不去
- C. 该。`cli` 必须成对 `sti`
- D. 无所谓。EOI 之后 IF 会自动打开

```html
<details><summary>第 2 题答案与解析</summary><p><strong>A</strong>。决定 0003 的核心：还原，不要设置。handler 里 was_enabled=false，放锁时保持 IF=0，直到 iretq 弹出旧 RFLAGS。</p></details>
```

**3. 为什么键盘不能像定时器那样只配 LAPIC、不碰 IOAPIC？**

- A. 键盘是外设 IRQ 线，LAPIC 本地寄存器接不到；必须经 IOAPIC 路由
- B. IOAPIC 比 LAPIC 快
- C. 向量 33 只能由 IOAPIC 产生，硬件写死
- D. 屏蔽 PIC 之后 LAPIC 定时器也会失效，所以一切都要走 IOAPIC

```html
<details><summary>第 3 题答案与解析</summary><p><strong>A</strong>。LAPIC timer 是本地源；键盘 IRQ1 是外部线。C 不对——向量号是软件写入重定向表的。D 不对——Bite1 已证明屏蔽 PIC 后定时器仍可跑。</p></details>
```

**4. 一次中断里 `0x60` 读到 `0xE0`，紧接着又读到 `0x48`（上方向键）。屏幕会显示什么？**

- A. 什么都不显示；`0xE0` 置位「扩展待定」，下一字节整段丢弃
- B. 显示 `H`，因为 `0x48` 在字母表里
- C. 显示两个乱码字符
- D. 触发 panic

```html
<details><summary>第 4 题答案与解析</summary><p><strong>A</strong>。decode 对 0xE0 置 EXTENDED_PENDING，随后一字节无论是什么都返回 None。本课不做扩展键。</p></details>
```

**5. `irq_dispatch` 末尾统一写 EOI。如果改成每个 handler 各自写一次，最直接违反的是哪条项目约定？**

- A. 同一函数实现只能有一份；EOI 会变成两处重复
- B. 中断门不允许在 handler 里写 MMIO
- C. 向量 32 与 33 不能共用 EOI 寄存器
- D. `iretq` 之前禁止写 EOI

```html
<details><summary>第 5 题答案与解析</summary><p><strong>A</strong>。编码规范要求同一实现一份；本课把 end_of_interrupt 收在分发层末尾。B/C/D 都不是硬件禁止项——提前写 EOI 在 IF 已清时也不会嵌套，但会复制调用点。</p></details>
```
