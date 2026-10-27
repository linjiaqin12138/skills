//! LAPIC 定时器：PIT 校准出 100 Hz 周期，IRQ32 只累计 jiffies 并发送 EOI。

use core::arch::asm;
use core::hint::spin_loop;
use core::ptr::{addr_of_mut, read_volatile, write_volatile};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::io::{inb, outb};

use limine::paging::PagingMode;
use limine::request::{ExecutableAddressRequest, HhdmRequest, PagingModeRequest};

const TIMER_VECTOR: u32 = 32;
const TIMER_HZ: u32 = 100;

const IA32_APIC_BASE_MSR: u32 = 0x1b; // 0b 0001_1011
const APIC_ENABLE: u64 = 1 << 11;
const APIC_BASE_MASK: u64 = 0x0000_000f_ffff_f000; // 0b 0000_0000_0000_0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_0000

const EOI: u64 = 0x0b0; // 0b 0000_1011_0000
const SPURIOUS_VECTOR: u64 = 0x0f0; // 0b 0000_1111_0000
const LVT_TIMER: u64 = 0x320;
const INITIAL_COUNT: u64 = 0x380;
const CURRENT_COUNT: u64 = 0x390;
const DIVIDE_CONFIG: u64 = 0x3e0;

const LAPIC_SOFTWARE_ENABLE: u32 = 1 << 8;
const TIMER_MASKED: u32 = 1 << 16;
const TIMER_PERIODIC: u32 = 1 << 17;
const DIVIDE_BY_16: u32 = 0b0011;

const PIT_FREQUENCY: u32 = 1_193_182;
const PIT_COUNT_FOR_10_MS: u16 = (PIT_FREQUENCY / TIMER_HZ) as u16;
const PIT_CHANNEL_2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;
const PIT_GATE: u16 = 0x61;
const PIT_CHANNEL_2_OUTPUT: u8 = 1 << 5;

const PAGE_PRESENT: u64 = 1 << 0;
const PAGE_WRITABLE: u64 = 1 << 1;
const PAGE_WRITE_THROUGH: u64 = 1 << 3;
const PAGE_CACHE_DISABLE: u64 = 1 << 4;
const PAGE_LARGE: u64 = 1 << 7;
const PAGE_ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;

static JIFFIES: AtomicU64 = AtomicU64::new(0);

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

static mut EXTRA_PAGE_TABLES: [PageTable; 3] = [const { PageTable([0; 512]) }; 3];

#[derive(Clone, Copy)]
struct Lapic {
    base: u64,
}

impl Lapic {
    fn read(self, offset: u64) -> u32 {
        // SAFETY: base 是 HHDM 中有效的 LAPIC MMIO 页，offset 均为对齐的寄存器地址。
        unsafe { read_volatile((self.base + offset) as *const u32) }
    }

    fn write(self, offset: u64, value: u32) {
        // SAFETY: 同 read；volatile 防止编译器省略或合并 MMIO 写入。
        unsafe { write_volatile((self.base + offset) as *mut u32, value) }
    }
}

static mut LAPIC: Lapic = Lapic { base: 0 };

/// 配置 LAPIC 与 PIT，校准成功后开启 CPU 可屏蔽中断。
pub fn init() -> bool {
    let Some(hhdm) = HHDM_REQUEST.response() else {
        return false;
    };
    let Some(executable) = EXECUTABLE_ADDRESS_REQUEST.response() else {
        return false;
    };

    mask_legacy_pic();
    let lapic = enable_lapic(
        hhdm.offset,
        executable.physical_base,
        executable.virtual_base,
    );
    // SAFETY: 单核启动阶段只写一次；开中断后 handler 只读该值。
    unsafe { LAPIC = lapic };

    lapic.write(DIVIDE_CONFIG, DIVIDE_BY_16);
    lapic.write(LVT_TIMER, TIMER_MASKED | TIMER_VECTOR);
    lapic.write(INITIAL_COUNT, 0);

    let ticks_per_jiffy = calibrate(lapic);
    if ticks_per_jiffy == 0 {
        return false;
    }

    lapic.write(INITIAL_COUNT, ticks_per_jiffy);
    lapic.write(LVT_TIMER, TIMER_PERIODIC | TIMER_VECTOR);

    // SAFETY: IDT、LAPIC 与 handler 均已就绪，才允许 IRQ32 进入。
    unsafe { asm!("sti", options(nomem, nostack)) };
    true
}

pub fn jiffies() -> u64 {
    JIFFIES.load(Ordering::Relaxed)
}

/// 中断上下文不格式化、不打印，也不触碰 console。
pub extern "C" fn timer_interrupt() {
    JIFFIES.fetch_add(1, Ordering::Relaxed);
    // SAFETY: init 在开中断前已写入有效 LAPIC 地址，此后不再修改。
    unsafe { LAPIC }.write(EOI, 0);
}

fn enable_lapic(hhdm_offset: u64, kernel_physical: u64, kernel_virtual: u64) -> Lapic {
    let apic_base = read_msr(IA32_APIC_BASE_MSR);
    if apic_base & APIC_ENABLE == 0 {
        write_msr(IA32_APIC_BASE_MSR, apic_base | APIC_ENABLE);
    }

    let physical_base = apic_base & APIC_BASE_MASK;
    let lapic = Lapic {
        base: map_mmio_page(physical_base, hhdm_offset, kernel_physical, kernel_virtual),
    };
    lapic.write(SPURIOUS_VECTOR, LAPIC_SOFTWARE_ENABLE | 0xff);
    lapic
}

/// Limine 的 HHDM 不保证包含 LAPIC MMIO；只在缺失时补这一张 4 KiB 页。
fn map_mmio_page(
    physical_base: u64,
    hhdm_offset: u64,
    kernel_physical: u64,
    kernel_virtual: u64,
) -> u64 {
    let virtual_base = physical_base + hhdm_offset;
    let cr3: u64;
    // SAFETY: 读取当前页树根地址，不修改控制寄存器。
    unsafe { asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags)) };
    let mut table_physical = cr3 & PAGE_ADDRESS_MASK;
    // SAFETY: 仅在关中断的单核初始化阶段使用；三个静态页分别供三层页表补缺。
    let extra_tables = unsafe { &mut *addr_of_mut!(EXTRA_PAGE_TABLES) };

    for (level, shift) in [39, 30, 21].into_iter().enumerate() {
        let index = ((virtual_base >> shift) & 0x1ff) as usize;
        let entry_ptr = (hhdm_offset + table_physical + index as u64 * 8) as *mut u64;
        // SAFETY: CR3 与各级描述符都指向 HHDM 可访问的页表。
        let mut entry = unsafe { read_volatile(entry_ptr) };
        if entry & PAGE_PRESENT == 0 {
            let table = &mut extra_tables[level];
            table.0.fill(0);
            let table_virtual = table.0.as_ptr() as u64;
            let new_physical = table_virtual - kernel_virtual + kernel_physical;
            entry = new_physical | PAGE_PRESENT | PAGE_WRITABLE;
            // SAFETY: entry_ptr 是当前级中尚未使用的描述符。
            unsafe { write_volatile(entry_ptr, entry) };
        }
        if entry & PAGE_LARGE != 0 {
            return virtual_base;
        }
        table_physical = entry & PAGE_ADDRESS_MASK;
    }

    let index = ((virtual_base >> 12) & 0x1ff) as usize;
    let entry_ptr = (hhdm_offset + table_physical + index as u64 * 8) as *mut u64;
    // SAFETY: 最后一级描述符指向 LAPIC 物理页，PCD/PWT 禁止缓存 MMIO。
    unsafe {
        write_volatile(
            entry_ptr,
            physical_base | PAGE_PRESENT | PAGE_WRITABLE | PAGE_WRITE_THROUGH | PAGE_CACHE_DISABLE,
        );
        asm!("invlpg [{}]", in(reg) virtual_base, options(nostack));
    }
    virtual_base
}

/// 用 PIT channel 2 的 10 ms 输出信号测量 LAPIC 在 divider=16 下的计数差。
fn calibrate(lapic: Lapic) -> u32 {
    let gate = inb(PIT_GATE);
    outb(PIT_GATE, (gate & !0x03) | 0x01);
    outb(PIT_COMMAND, 0xb0);
    outb(PIT_CHANNEL_2, PIT_COUNT_FOR_10_MS as u8);
    outb(PIT_CHANNEL_2, (PIT_COUNT_FOR_10_MS >> 8) as u8);

    lapic.write(INITIAL_COUNT, u32::MAX);
    while inb(PIT_GATE) & PIT_CHANNEL_2_OUTPUT == 0 {
        spin_loop();
    }
    let elapsed = u32::MAX - lapic.read(CURRENT_COUNT);
    lapic.write(INITIAL_COUNT, 0);
    elapsed
}

fn mask_legacy_pic() {
    outb(0x21, 0xff);
    outb(0xa1, 0xff);
}

fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: 只读取固定的 APIC base MSR；rdmsr 的结果由 EDX:EAX 返回。
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack));
    }
    ((high as u64) << 32) | low as u64
}

fn write_msr(msr: u32, value: u64) {
    // SAFETY: 调用者仅在 APIC 全局使能位为 0 时置位该位，其他位保持不变。
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") value as u32,
            in("edx") (value >> 32) as u32,
            options(nomem, nostack),
        );
    }
}
