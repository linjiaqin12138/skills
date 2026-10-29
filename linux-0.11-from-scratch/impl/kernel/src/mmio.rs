//! 设备 MMIO 页的按需映射：LAPIC、IOAPIC 共用。
//!
//! Limine 的 HHDM 不保证覆盖设备物理地址（QEMU 上实测 LAPIC 页缺失，访问即 #PF），
//! 所以这里在当前 CR3 的页树里只补缺失的层级与最后一项。
//! 这是启动期权宜之计，不是通用页分配器；M5 自建页表后收敛。见 decisions/0002。

use core::arch::asm;
use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use limine::paging::PagingMode;
use limine::request::{ExecutableAddressRequest, HhdmRequest, PagingModeRequest};

const PAGE_PRESENT: u64 = 1 << 0;
const PAGE_WRITABLE: u64 = 1 << 1;
const PAGE_WRITE_THROUGH: u64 = 1 << 3;
const PAGE_CACHE_DISABLE: u64 = 1 << 4;
const PAGE_LARGE: u64 = 1 << 7;
const PAGE_ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;

/// 静态页表池容量。第一张设备页最坏要补 3 层（PDPT/PD/PT），
/// 第二张设备页（IOAPIC 与 LAPIC 隔着不同的 2 MiB）再补 1 张 PT，共 4。
/// 再加设备要来这里加大（代理假设：M3 只有 LAPIC + IOAPIC 两张页）。
const POOL_TABLES: usize = 4;

#[used]
#[unsafe(link_section = ".limine_requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static PAGING_MODE_REQUEST: PagingModeRequest =
    PagingModeRequest::new_exact(PagingMode::X86_64_4LVL);

#[repr(C, align(4096))]
struct PageTable([u64; 512]);

static mut TABLE_POOL: [PageTable; POOL_TABLES] = [const { PageTable([0; 512]) }; POOL_TABLES];
static mut POOL_USED: usize = 0;

/// 让一页设备物理内存在 HHDM 里可访问（禁缓存），返回其虚拟地址；
/// Limine 没给出所需响应时返回 None。只能在关中断的单核启动阶段调用。
pub fn map_page(physical_base: u64) -> Option<u64> {
    let hhdm_offset = HHDM_REQUEST.response()?.offset;
    let executable = EXECUTABLE_ADDRESS_REQUEST.response()?;
    let virtual_base = physical_base + hhdm_offset;

    let cr3: u64;
    // SAFETY: 读取当前页树根地址，不修改控制寄存器。
    unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags)) };
    let mut table_physical = cr3 & PAGE_ADDRESS_MASK;

    // 自顶向下走 PML4 → PDPT → PD，缺哪层补哪层
    for shift in [39, 30, 21] {
        let entry_ptr = entry_pointer(hhdm_offset, table_physical, virtual_base, shift);
        // SAFETY: CR3 与各级描述符都指向 HHDM 可访问的页表。
        let mut entry = unsafe { read_volatile(entry_ptr) };
        if entry & PAGE_PRESENT == 0 {
            let new_physical = take_pool_table_physical(executable.physical_base, executable.virtual_base);
            entry = new_physical | PAGE_PRESENT | PAGE_WRITABLE;
            // SAFETY: entry_ptr 是当前级中尚未使用的描述符。
            unsafe { write_volatile(entry_ptr, entry) };
        }
        if entry & PAGE_LARGE != 0 {
            // 已被 1 GiB / 2 MiB 大页覆盖，无需再补
            return Some(virtual_base);
        }
        table_physical = entry & PAGE_ADDRESS_MASK;
    }

    let entry_ptr = entry_pointer(hhdm_offset, table_physical, virtual_base, 12);
    // SAFETY: 最后一级描述符指向设备物理页，PCD/PWT 禁止缓存 MMIO。
    unsafe {
        write_volatile(
            entry_ptr,
            physical_base | PAGE_PRESENT | PAGE_WRITABLE | PAGE_WRITE_THROUGH | PAGE_CACHE_DISABLE,
        );
        asm!("invlpg [{}]", in(reg) virtual_base, options(nostack));
    }
    Some(virtual_base)
}

/// 某级页表中，`virtual_base` 对应描述符的 HHDM 地址。`shift` 是该级索引在地址里的起始位。
fn entry_pointer(hhdm_offset: u64, table_physical: u64, virtual_base: u64, shift: u32) -> *mut u64 {
    let index = (virtual_base >> shift) & 0x1ff;
    (hhdm_offset + table_physical + index * 8) as *mut u64
}

/// 从静态池取一张清零页表，返回它的物理地址（内核镜像内的虚拟地址换算而来）。
fn take_pool_table_physical(kernel_physical: u64, kernel_virtual: u64) -> u64 {
    // SAFETY: 仅在关中断的单核初始化阶段访问池与计数。
    let (pool, used) = unsafe { (&mut *addr_of_mut!(TABLE_POOL), &mut *addr_of_mut!(POOL_USED)) };
    assert!(*used < POOL_TABLES, "mmio: 静态页表池用尽");
    let table = &mut pool[*used];
    *used += 1;
    table.0.fill(0);
    table.0.as_ptr() as u64 - kernel_virtual + kernel_physical
}
