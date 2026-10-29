//! PS/2 键盘：IRQ1 经 IOAPIC 投到向量 33，handler 里把扫描码转 ASCII 并回显。
//!
//! 只做回显（D3）：没有行规程、没有 Shift 等修饰键。见 decisions/0004。

use core::sync::atomic::{AtomicBool, Ordering};

use crate::io::inb;
use crate::{ioapic, lapic, printk};

pub const KEYBOARD_VECTOR: u32 = 33;
/// ISA IRQ1；QEMU 默认机上 GSI 与之同号。
const KEYBOARD_GSI: u8 = 1;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
/// 状态寄存器位 0：输出缓冲区有数据可读。不先查就读 0x60 会拿到过期字节。
const STATUS_OUTPUT_FULL: u8 = 1 << 0;

/// 扩展键前缀：其后的一个字节属于方向键、右 Ctrl 等扩展键，本 Bite 忽略。
const EXTENDED_PREFIX: u8 = 0xe0;
/// 扫描码集 1 里，最高位置位表示松开（break），否则是按下（make）。
const BREAK_BIT: u8 = 1 << 7;

/// 扫描码集 1 的 make code → ASCII，下标即扫描码（0x00..=0x39），0 = 不回显。
/// Esc / Backspace / Tab / 修饰键都留 0：字体里没有它们的字形，修饰键也不在本 Bite 范围。
/// 类型标成定长数组，让编译器核对长度恰为 0x3a。
const SCANCODE_TO_ASCII: &[u8; 0x3a] =
    b"\0\x001234567890-=\0\0qwertyuiop[]\n\0asdfghjkl;'`\0\\zxcvbnm,./\0*\0 ";

/// 上一个字节是 0xE0 前缀时，下一个字节要整个丢弃。
/// 只有键盘 handler 读写它，用原子量是为了不用 `static mut`，不是为了并发。
static EXTENDED_PENDING: AtomicBool = AtomicBool::new(false);

/// 清空控制器里残留的字节，再把 IRQ1 路由到向量 33。
/// 必须在 IDT 已装好向量 33 之后调用。
pub fn init() -> bool {
    // 残留字节不清掉，输出缓冲区一直满着，控制器不会再触发新的 IRQ1
    while inb(STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
        inb(DATA_PORT);
    }
    ioapic::route(KEYBOARD_GSI, KEYBOARD_VECTOR as u8, lapic::id())
}

/// 向量 33 的 handler（由 idt 分发层调用，EOI 由分发层发）。
pub fn interrupt() {
    // 一次中断可能积了多个字节，读到缓冲区空为止
    while inb(STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
        let scancode = inb(DATA_PORT);
        if let Some(ch) = decode(scancode) {
            printk!("{}", ch as char);
        }
    }
}

/// 扫描码 → 要回显的 ASCII；松开事件、扩展键、无字形的键返回 None。
fn decode(scancode: u8) -> Option<u8> {
    if scancode == EXTENDED_PREFIX {
        EXTENDED_PENDING.store(true, Ordering::Relaxed);
        return None;
    }
    if EXTENDED_PENDING.swap(false, Ordering::Relaxed) {
        return None;
    }
    if scancode & BREAK_BIT != 0 {
        return None;
    }
    match SCANCODE_TO_ASCII.get(scancode as usize) {
        Some(&ascii) if ascii != 0 => Some(ascii),
        _ => None,
    }
}
