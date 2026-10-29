#!/usr/bin/env bash
# 在 linux-0.11-from-scratch/ 仓库根执行。
# 预期：
# 1. 内核构建并启动；M0/M1/M2 回归 marker 与 M3 Bite1 的 100Hz jiffies 样本仍在。
# 2. IOAPIC 将键盘 IRQ1 路由到 IDT 向量 33；PS/2 读扫描码后在中断上下文 printk 回显 ASCII。
# 3. QEMU monitor 依次 sendkey h/e/l/l/o 后，串口出现连续子串 "hello"（证明回显通路）。
# 4. 全局 console 已用关中断临界区收敛 D15：定时器 tick 与键盘回显并存时不 panic、不重入花屏。
# 5. QEMU screendump 非空。行规程、修饰键、异常 iret、MADT 发现不在本 Bite。
set -euo pipefail
cd "$(dirname "$0")/../impl"
make check-m3-b2
