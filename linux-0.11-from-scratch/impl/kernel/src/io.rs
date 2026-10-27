//! x86 端口映射 I/O（PIO）的最小封装：in / out 两条指令。
//!
//! PIT、8259 传统 PIC、16550 串口这类设备只存在于 I/O 端口空间，
//! 访问它们必须用 `in`/`out`，不能像普通内存那样读写。

use core::arch::asm;

/// 向 I/O 端口写一个字节。
pub fn outb(port: u16, value: u8) {
    // SAFETY: 只读写固定 I/O 端口，不触碰内存与栈；in/out 均不影响 EFLAGS
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

/// 从 I/O 端口读一个字节。
pub fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: 同上
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}
