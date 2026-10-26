---
template: doc
theme: paper
title: M2 Bite2 · panic!：让 Rust 错误可见
subtitle: panic_handler 接入 printk!，双写消息与源码位置后停机
source: linux-0.11-from-scratch · M2
---

Lead: `panic!` 现在会进入自己的 `panic_handler`。消息和源码位置同时到达 framebuffer 与串口，随后 CPU 停机。验收已通过，exit code 为 0。

## 问题

**内核发生 Rust panic 时，必须留下能定位源码的现场。**

GDT 和 IDT 初始化后，一次 `panic!` 说明内核遇到了无法继续执行的错误。若处理入口只让 CPU 停机，屏幕和串口都没有错误内容。读者只能看到启动中断，无法知道消息、文件和行号。

CPU 异常与 Rust panic 也不是同一个入口。Bite1 的 #DE 由 CPU 查询 IDT；本课的 `panic!` 由 Rust 转入 `panic_handler`。

## 设计

```flow LR
[panic!] -> *panic_handler
panic_handler -> [PanicInfo]
PanicInfo -> [info.message()] & [info.location()]
info.message() -> [printk!]
info.location() -> [printk!]
printk! -> [framebuffer] & [serial]
framebuffer -> (halt)
serial -> (halt)
```

**一个 handler 提取两类信息，再复用 M1 的输出通道。**

- `info.message()` 提供错误消息。
- `info.location()` 尝试提供源码位置。
- `printk!` 把格式化文本同时送到两个出口。
- `halt()` 让处理停在这里，不返回出错位置。

这条路径与 Bite1 共用输出后端。差别只在入口：CPU 异常走 IDT，Rust panic 走 `panic_handler`。

## 代码领读

**触发点放在 GDT 和 IDT 就绪之后**（`impl/kernel/src/main.rs:52-61`）。

```rust src=impl/kernel/src/main.rs lines=52-61 hl=59
```

`gdt::init()` 和 `idt::init()` 是模块中的函数调用。两条成功日志之后，`panic!("...")` 携带消息并触发 panic。这里的 `!` 表示宏调用。

**handler 读取现场并停机**（`impl/kernel/src/main.rs:76-106`）。

```rust src=impl/kernel/src/main.rs lines=76-106 hl=87
```

`#[panic_handler]` 把函数登记为 `no_std` 内核的 panic 入口。`info: &PanicInfo` 是只读借用，函数借此读取本次 panic 数据。

消息与位置分开处理，因为 `info.message()` 给出可显示的消息，`info.location()` 却返回 `Option`。`Option` 表示值可能存在：

- `Some(loc)`：位置存在，`loc` 是取出的位置对象。
- `None`：位置缺失，改打 `<unknown location>`。
- `match`：要求处理这两种情况，不能漏掉 `None`。

`loc.file()`、`loc.line()` 和 `loc.column()` 分别提供文件、行和列。`env!("CARGO_PKG_NAME")` 在编译期读取 Cargo 包名。本包名是 `kernel`，所以 `kernel/` 加上 `src/main.rs`，得到仓库相对路径。

函数末尾的 `-> !` 是永不返回类型。它承诺控制流不会回到调用者。`halt()` 也返回 `!`，所以能直接作为函数末尾表达式。

`loop` 建立无限循环。`unsafe` 表示内联汇编的安全性由代码保证，`asm!("hlt")` 暂停 CPU。`options(nomem, nostack)` 声明这条指令不访问内存，也不修改栈。

**`printk!` 在一个出口点完成双写**（`impl/kernel/src/console.rs:78-88`）。

```rust src=impl/kernel/src/console.rs lines=78-88 hl=83
```

`impl fmt::Write for Console` 给控制台实现格式化写入接口。`&mut self` 允许写字符时更新光标。`for byte in s.bytes()` 逐字节处理文本。

每个字节先交给 `put_char` 绘制，再交给串口发送。因此 handler 只调用 `printk!`，无需分别维护两套输出。`Ok(())` 表示写入成功，且不携带结果值。

**串口出口最终写入 COM1**（`impl/kernel/src/serial.rs:51-60`）。

```rust src=impl/kernel/src/serial.rs lines=51-60 hl=53
```

`u8` 是一个字节。`while` 循环等待硬件发送槽空闲，再由 `outb` 写入字节。`pub fn` 让 `console` 模块可以调用这两个函数。

framebuffer 是屏幕像素区域，串口是独立硬件出口。串口保留完整日志，屏幕则直接显示当前控制台内容。

## 验收

在仓库根目录执行：

```bash
bash accept/m02-b02.sh
```

`accept/m02-b02.sh:3-7` 给出四项预期：保留旧输出、初始化 GDT/IDT 后 panic、双写消息与位置、停机并生成非空截图。

**实测 PASS，exit code 为 0，四项预期全部满足。**

- `M0: kernel alive` 恰好出现一次。
- 实测消息是 `PANIC: M2 Bite2 deliberate panic demonstration`。
- 实测位置是 `panicked at kernel/src/main.rs:59:5`。
- 800x600 screendump 非空，未出现重复启动。

实测记录还手工解码了 PPM。panic 文本确实到达 framebuffer，不只出现在串口。完整证据见 [m02-b02 验收记录](../../tutor/runs/m02-b02.txt)。

## 坑

```callout warn Cargo 路径缺少 kernel/ 前缀
只打印 loc.file() 会得到 src/main.rs。Cargo 从 kernel 包构建源码，这个路径不含仓库中的 kernel/ 前缀。实现用 env!("CARGO_PKG_NAME") 补上包名，日志才能直接定位 kernel/src/main.rs。
```

## 自测

**1. 把 `panic!("new failure")` 移到另一个函数，诊断应怎样变化？**

- A. 消息不变，位置固定为 `main.rs:59:5`
- B. 消息改变，位置指向新的 `panic!` 调用点
- C. 位置改变，但消息仍来自旧调用点
- D. 不再进入 `panic_handler`，改走 IDT

```html
<details><summary>第 1 题答案与解析</summary><p><strong>B</strong>。PanicInfo 来自本次调用。message() 读取新消息，location() 指向新的源码调用点。</p></details>
```

**2. 删除 `env!("CARGO_PKG_NAME")` 和对应斜杠后，当前日志会失去什么？**

- A. panic 消息
- B. 行号和列号
- C. 仓库相对路径中的 `kernel/` 前缀
- D. framebuffer 输出

```html
<details><summary>第 2 题答案与解析</summary><p><strong>C</strong>。loc.file() 仍提供 src/main.rs，行列号也保留。缺少包名后，日志不再直接对应仓库相对路径。</p></details>
```

**3. 某次 `info.location()` 返回 `None`，现有 handler 会怎么做？**

- A. 跳过消息并立即重启
- B. 打印未知位置，保留消息，然后停机
- C. 解引用空指针并触发 #DE
- D. 自动从 IDT 猜测位置

```html
<details><summary>第 3 题答案与解析</summary><p><strong>B</strong>。消息已经单独打印。match 的 None 分支输出 unknown location，随后仍执行 halt()。</p></details>
```

**4. 为什么 `panic` 和 `halt` 都能声明返回类型 `!`？**

- A. 它们一定触发 CPU 异常
- B. 它们没有参数
- C. 它们不会把控制流返回调用者
- D. 它们会自动展开栈并返回

```html
<details><summary>第 4 题答案与解析</summary><p><strong>C</strong>。! 是永不返回类型。panic 最后进入 halt，halt 持续执行循环，两条路径都没有正常返回。</p></details>
```

**5. 把故意 `panic!` 换成内联汇编除零，主要入口会变成哪一个？**

- A. `panic_handler`
- B. `console::init`
- C. IDT 中的 #DE handler
- D. `serial::write_str`

```html
<details><summary>第 5 题答案与解析</summary><p><strong>C</strong>。汇编除零由 CPU 检测并经 IDT 分发。Rust panic! 才进入本课的 panic_handler。</p></details>
```
