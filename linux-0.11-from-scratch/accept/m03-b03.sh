#!/usr/bin/env bash
# 在 linux-0.11-from-scratch/ 仓库根执行。
# 预期：
# 1. 从 Limine RSDP 响应找到 RSDT/XSDT，再解析 MADT；串口报告实际 IOAPIC 基址与 IRQ1 的 GSI/极性/触发方式。
# 2. ISO 存在时应用其 GSI 与 flags；不存在时使用 ISA 默认。失败不得回退到硬编码地址。
# 3. 保留 Bite2 的键盘回显、100Hz jiffies 与 console 关中断锁；sendkey hello 后串口出现连续子串。
# 4. QEMU screendump 非空，无 MADT 发现失败、panic 或异常。
set -euo pipefail
cd "$(dirname "$0")/../impl"
make check-m3-b3
