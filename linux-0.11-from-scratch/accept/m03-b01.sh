#!/usr/bin/env bash
# 在 linux-0.11-from-scratch/ 仓库根执行。
# 预期：
# 1. 内核构建并启动，M0/M1 回归与 M2 GDT/IDT 初始化保留，M0 存活行恰好一次，不触发故意 panic。
# 2. LAPIC 定时器以 100Hz 投递 IRQ32；handler 保存/恢复现场并 iretq 返回，只增加原子 jiffies 并发送 EOI。
# 3. 普通内核上下文至少观察到 jiffies=100、200、300 三个连续样本，经 printk! 双写屏幕和串口，证明中断后恢复执行；固定宿主时间窗口内的样本数也须符合 100Hz 的合理误差范围。
# 4. panic 不再作为正常启动演示；QEMU screendump 非空。键盘与 console 锁不属于本 Bite。
set -euo pipefail
cd "$(dirname "$0")/../impl"
make check-m3-b1
