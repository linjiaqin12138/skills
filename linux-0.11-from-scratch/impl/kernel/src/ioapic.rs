//! IOAPIC：把外设 IRQ 线路由成"某个向量投给某个 LAPIC"。
//!
//! 基址、GSI 与触发属性来自 ACPI MADT / ISO（decisions/0005）。
//! 寄存器是"间接访问"：先把寄存器编号写进 IOREGSEL，再读写 IOWIN。

use core::ptr::write_volatile;

use crate::{acpi::KeyboardRoute, mmio};

const IOREGSEL_OFFSET: u64 = 0x00;
const IOWIN_OFFSET: u64 = 0x10;

/// 重定向表第 n 项占两个 32 位寄存器：编号 0x10 + 2n（低）与 0x11 + 2n（高）。
const REDIRECTION_TABLE_BASE: u32 = 0x10;

const REDIRECTION_POLARITY_LOW: u32 = 1 << 13;
const REDIRECTION_TRIGGER_LEVEL: u32 = 1 << 15;
/// 高 32 位的位 24..31 是目标 LAPIC ID。
const REDIRECTION_HIGH_DESTINATION_SHIFT: u32 = 24;

/// 把 ACPI 发现的路由投到本核 LAPIC 的 `vector`。
pub fn route(keyboard: KeyboardRoute, vector: u8, destination_lapic_id: u8) -> bool {
    let Some(base) = mmio::map_page(keyboard.ioapic_base) else {
        return false;
    };
    let register = REDIRECTION_TABLE_BASE + 2 * keyboard.gsi;
    let mut low = vector as u32;
    if keyboard.active_low {
        low |= REDIRECTION_POLARITY_LOW;
    }
    if keyboard.level_triggered {
        low |= REDIRECTION_TRIGGER_LEVEL;
    }

    // 先写高半（目标），再写低半（含 mask 位）：低半一落笔，这一项就生效了
    write_register(
        base,
        register + 1,
        (destination_lapic_id as u32) << REDIRECTION_HIGH_DESTINATION_SHIFT,
    );
    write_register(base, register, low);
    true
}

fn write_register(base: u64, register: u32, value: u32) {
    // SAFETY: base 是已映射且禁缓存的 IOAPIC 页；两个窗口寄存器都是 4 字节对齐的 32 位 MMIO。
    unsafe {
        write_volatile((base + IOREGSEL_OFFSET) as *mut u32, register);
        write_volatile((base + IOWIN_OFFSET) as *mut u32, value);
    }
}
