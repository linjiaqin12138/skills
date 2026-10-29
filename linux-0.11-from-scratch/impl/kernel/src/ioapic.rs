//! IOAPIC：把外设 IRQ 线路由成"某个向量投给某个 LAPIC"。
//!
//! 基址固定为 QEMU 默认平台的 0xFEC00000，不解析 ACPI MADT（decisions/0004）。
//! 寄存器是"间接访问"：先把寄存器编号写进 IOREGSEL，再读写 IOWIN。

use core::ptr::write_volatile;

use crate::mmio;

const IOAPIC_PHYSICAL_BASE: u64 = 0xfec0_0000;

const IOREGSEL_OFFSET: u64 = 0x00;
const IOWIN_OFFSET: u64 = 0x10;

/// 重定向表第 n 项占两个 32 位寄存器：编号 0x10 + 2n（低）与 0x11 + 2n（高）。
const REDIRECTION_TABLE_BASE: u32 = 0x10;

/// 低 32 位里除向量外全 0 即：Fixed 投递、物理目标模式、高电平有效、边沿触发、未屏蔽。
/// ISA 键盘正是这组默认值；需要电平触发或低有效的设备再加位。
const REDIRECTION_LOW_VECTOR_MASK: u32 = 0xff;
/// 高 32 位的位 24..31 是目标 LAPIC ID。
const REDIRECTION_HIGH_DESTINATION_SHIFT: u32 = 24;

/// 把 `gsi`（全局系统中断号）路由到本核 LAPIC 的 `vector`。
/// ISA IRQ 与 GSI 在 QEMU 默认机上同号（IRQ0 的 GSI2 重映射不影响键盘 IRQ1）。
pub fn route(gsi: u8, vector: u8, destination_lapic_id: u8) -> bool {
    let Some(base) = mmio::map_page(IOAPIC_PHYSICAL_BASE) else {
        return false;
    };
    let register = REDIRECTION_TABLE_BASE + 2 * gsi as u32;

    // 先写高半（目标），再写低半（含 mask 位）：低半一落笔，这一项就生效了
    write_register(base, register + 1, (destination_lapic_id as u32) << REDIRECTION_HIGH_DESTINATION_SHIFT);
    write_register(base, register, vector as u32 & REDIRECTION_LOW_VECTOR_MASK);
    true
}

fn write_register(base: u64, register: u32, value: u32) {
    // SAFETY: base 是已映射且禁缓存的 IOAPIC 页；两个窗口寄存器都是 4 字节对齐的 32 位 MMIO。
    unsafe {
        write_volatile((base + IOREGSEL_OFFSET) as *mut u32, register);
        write_volatile((base + IOWIN_OFFSET) as *mut u32, value);
    }
}
