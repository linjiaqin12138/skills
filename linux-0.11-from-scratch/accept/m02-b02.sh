#!/usr/bin/env bash
# 在 linux-0.11-from-scratch/ 仓库根执行。
# 预期：
# 1. 内核构建并启动，M0/M1 回归输出仍在，M0 存活行恰好一次。
# 2. 自有 GDT 与 IDT 初始化成功，随后故意调用 panic!()。
# 3. panic_handler 通过 printk! 双写屏幕和串口，输出指定消息及 main.rs 的行列位置。
# 4. panic 后停机，不死循环重启；QEMU screendump 非空。
set -euo pipefail
cd "$(dirname "$0")/../impl"
make check-m2-b2
