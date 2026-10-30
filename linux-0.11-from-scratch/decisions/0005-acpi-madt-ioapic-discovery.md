# 0005 从 ACPI MADT / ISO 发现键盘 IOAPIC 路由

- 状态：已采纳
- 来源：手搓
- 出现于：m03-b03
- 被取代于：

## 上下文

Bite2 把 IOAPIC 基址 `0xFEC00000` 与键盘 GSI 1 写死，只在 QEMU 默认机上成立。LAPIC 能从 MSR 读基址，IOAPIC 没有对应的通用 MSR，必须读取固件提供的中断硬件描述。D18 在 Bite3 收敛。

## 备选

1. 保留 QEMU 默认值：实现最短，但换机器后没有正确性保证。
2. 通过 Limine RSDP 响应解析 ACPI：按固件表发现 IOAPIC 与 IRQ1 的实际路由，增加一个解析模块和表校验。
3. 引入完整 ACPI 库：覆盖更广，但本课只需要 RSDP、RSDT/XSDT、MADT 与 ISO，不值得增加依赖和抽象。

## 决定

选方案 2。RSDP revision 决定使用 RSDT 还是 XSDT；校验 RSDP 与 SDT 的 signature、长度和 checksum，再按 MADT 变长条目长度遍历。IOAPIC 项提供 MMIO 基址与 GSI 起点，ISO 项提供 IRQ1 的 GSI、极性和触发方式；没有 IRQ1 的 ISO 时使用 ISA 默认 GSI 1、高电平、边沿触发。

`keyboard::init` 把发现结果交给 `ioapic::route`。解析或映射失败就返回错误，不回退到 `0xFEC00000`。ACPI 物理页复用 `mmio::map_page` 按需映射；静态页表池扩到 8，D17 仍留到 M5。

## 放弃的成本

没有采用完整 ACPI 框架。当前只找键盘 IRQ1 所需条目，多 IOAPIC 的完整 GSI 范围管理、多核目标和其他设备留到实际需要时扩展。

## 失效边界

RSDT/XSDT 或 MADT 跨越多页时，当前逐页映射假设需要扩展；固件表损坏、缺少匹配 IOAPIC、保留的 polarity/trigger 编码都会使初始化失败。真机与非默认机型具备正确发现路径，但本 Bite 只在 QEMU 默认机实测。
