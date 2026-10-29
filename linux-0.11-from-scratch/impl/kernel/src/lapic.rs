//! LAPIC：100 Hz 定时器（PIT 校准）与 EOI。IRQ32 只累计 jiffies，EOI 由 IDT 分发层统一发送。

use core::arch::asm;
use core::hint::spin_loop;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::io::{inb, outb};
use crate::mmio;

pub const TIMER_VECTOR: u32 = 32;
const TIMER_HZ: u32 = 100;

const IA32_APIC_BASE_MSR: u32 = 0x1b; // 0b 0001_1011
const APIC_ENABLE: u64 = 1 << 11;
const APIC_BASE_MASK: u64 = 0x0000_000f_ffff_f000; // 0b 0000_0000_0000_0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_0000

const EOI: u64 = 0x0b0; // 0b 0000_1011_0000
const SPURIOUS_VECTOR: u64 = 0x0f0; // 0b 0000_1111_0000
const ID: u64 = 0x020;
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

static JIFFIES: AtomicU64 = AtomicU64::new(0);

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
    mask_legacy_pic();
    let Some(lapic) = enable_lapic() else {
        return false;
    };
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

/// 中断上下文不格式化、不打印，也不触碰 console（decisions/0001）。
pub fn timer_interrupt() {
    JIFFIES.fetch_add(1, Ordering::Relaxed);
}

/// 告诉 LAPIC 当前中断已处理完，它才会投递同级及更低优先级的下一个中断。
pub fn end_of_interrupt() {
    // SAFETY: init 在开中断前已写入有效 LAPIC 地址，此后不再修改。
    unsafe { LAPIC }.write(EOI, 0);
}

/// 本核 LAPIC ID（寄存器位 24..31），IOAPIC 路由表用它指定投递目标。
pub fn id() -> u8 {
    // SAFETY: 同 end_of_interrupt。
    (unsafe { LAPIC }.read(ID) >> 24) as u8
}

fn enable_lapic() -> Option<Lapic> {
    let apic_base = read_msr(IA32_APIC_BASE_MSR);
    if apic_base & APIC_ENABLE == 0 {
        write_msr(IA32_APIC_BASE_MSR, apic_base | APIC_ENABLE);
    }

    let lapic = Lapic {
        base: mmio::map_page(apic_base & APIC_BASE_MASK)?,
    };
    lapic.write(SPURIOUS_VECTOR, LAPIC_SOFTWARE_ENABLE | 0xff);
    Some(lapic)
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
