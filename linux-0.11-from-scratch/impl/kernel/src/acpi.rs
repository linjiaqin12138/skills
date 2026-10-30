//! 从 Limine 的 RSDP 响应查找 MADT，发现键盘使用的 IOAPIC 与 GSI。

use core::ptr::read_unaligned;
use limine::request::RsdpRequest;

use crate::mmio;

#[used]
#[unsafe(link_section = ".limine_requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

const SDT_HEADER_LEN: usize = 36;
const MADT_ENTRIES_OFFSET: usize = 44;
const ENTRY_IOAPIC: u8 = 1;
const ENTRY_ISO: u8 = 2;

#[derive(Clone, Copy)]
pub struct KeyboardRoute {
    pub ioapic_base: u64,
    pub gsi: u32,
    pub active_low: bool,
    pub level_triggered: bool,
}

pub fn keyboard_route() -> Option<KeyboardRoute> {
    let rsdp_physical = RSDP_REQUEST.response()?.address as u64;
    let rsdp = map_physical(rsdp_physical)?;
    // SAFETY: ACPI 页已通过 HHDM 按需映射，解析前逐项校验长度。
    unsafe {
        if !has_signature(rsdp, b"RSD PTR ") || !checksum_ok(rsdp, 20) {
            return None;
        }
        let revision = rsdp.add(15).read();
        let extended_length = if revision >= 2 {
            read_u32(rsdp, 20) as usize
        } else {
            20
        };
        if extended_length < 20 || !checksum_ok(rsdp, extended_length) {
            return None;
        }

        let (root_physical, entry_size) = if revision >= 2 {
            (read_u64(rsdp, 24), 8)
        } else {
            (read_u32(rsdp, 16) as u64, 4)
        };
        let root = map_physical(root_physical)?;
        let madt = find_madt(root, entry_size)?;
        parse_madt(madt)
    }
}

fn map_physical(physical: u64) -> Option<*const u8> {
    let page = mmio::map_page(physical & !0xfff)?;
    Some((page + (physical & 0xfff)) as *const u8)
}

unsafe fn find_madt(root: *const u8, entry_size: usize) -> Option<*const u8> {
    if unsafe { !valid_sdt(root) } {
        return None;
    }
    let length = unsafe { read_u32(root, 4) } as usize;
    let entries = unsafe { root.add(SDT_HEADER_LEN) };
    let entries_len = length - SDT_HEADER_LEN;
    if entries_len % entry_size != 0 {
        return None;
    }
    for offset in (0..entries_len).step_by(entry_size) {
        let physical = unsafe {
            if entry_size == 8 {
                read_u64(entries, offset)
            } else {
                read_u32(entries, offset) as u64
            }
        };
        let address = map_physical(physical)?;
        if unsafe { valid_sdt(address) } && unsafe { has_signature(address, b"APIC") } {
            return Some(address);
        }
    }
    None
}

unsafe fn parse_madt(madt: *const u8) -> Option<KeyboardRoute> {
    let length = unsafe { read_u32(madt, 4) } as usize;
    if length < MADT_ENTRIES_OFFSET {
        return None;
    }
    let mut ioapic_base = None;
    let mut ioapic_gsi_start = 0;
    let mut iso = None;
    let mut offset = MADT_ENTRIES_OFFSET;
    while offset < length {
        if length - offset < 2 {
            return None;
        }
        let entry_type = unsafe { madt.add(offset).read() };
        let entry_len = unsafe { madt.add(offset + 1).read() } as usize;
        if entry_len < 2 || entry_len > length - offset {
            return None;
        }
        match entry_type {
            ENTRY_IOAPIC if entry_len >= 12 => {
                let gsi_start = unsafe { read_u32(madt, offset + 8) };
                ioapic_base = Some(unsafe { read_u32(madt, offset + 4) } as u64);
                ioapic_gsi_start = gsi_start;
            }
            ENTRY_ISO if entry_len >= 12 => {
                let source = unsafe { madt.add(offset + 3).read() };
                if source == 1 {
                    let gsi = unsafe { read_u32(madt, offset + 4) };
                    let flags = unsafe { madt.add(offset + 8).read() } as u16
                        | (unsafe { madt.add(offset + 9).read() } as u16) << 8;
                    iso = Some((gsi, flags));
                }
            }
            _ => {}
        }
        offset += entry_len;
    }

    let ioapic_base = ioapic_base?;
    let (gsi, flags) = iso.unwrap_or((1, 0));
    if gsi < ioapic_gsi_start {
        return None;
    }
    let polarity = flags & 0b11;
    let trigger = (flags >> 2) & 0b11;
    if polarity == 0b10 || trigger == 0b10 {
        return None;
    }
    Some(KeyboardRoute {
        ioapic_base,
        gsi,
        active_low: polarity == 0b11,
        level_triggered: trigger == 0b11,
    })
}

unsafe fn valid_sdt(table: *const u8) -> bool {
    if table.is_null() {
        return false;
    }
    let length = unsafe { read_u32(table, 4) } as usize;
    length >= SDT_HEADER_LEN && unsafe { checksum_ok(table, length) }
}

unsafe fn has_signature(table: *const u8, signature: &[u8]) -> bool {
    signature.iter().enumerate().all(|(index, expected)| unsafe { table.add(index).read() } == *expected)
}

unsafe fn checksum_ok(bytes: *const u8, length: usize) -> bool {
    let sum = (0..length).fold(0u8, |sum, index| sum.wrapping_add(unsafe { bytes.add(index).read() }));
    sum == 0
}

unsafe fn read_u32(base: *const u8, offset: usize) -> u32 {
    unsafe { read_unaligned(base.add(offset).cast()) }
}

unsafe fn read_u64(base: *const u8, offset: usize) -> u64 {
    unsafe { read_unaligned(base.add(offset).cast()) }
}
