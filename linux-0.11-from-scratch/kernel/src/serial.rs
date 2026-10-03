//! 最小 COM1 串口驱动（UART 16550），只够往外写字节。
//! 寄存器偏移与位定义是 16550 datasheet 的行业约定，
//! 亦可见 OSDev wiki 的 "Serial Ports" 词条。

use core::arch::asm;

/// PC 惯例：COM1 的 I/O 端口基址（BIOS 数据区 0x400 处可查）
const COM1: u16 = 0x3F8;

// 以下都是相对端口基址的寄存器偏移（16550 datasheet 约定）
const DATA: u16 = 0; // DLAB=0 时：写=发送保持寄存器，读=接收缓冲寄存器
const INTERRUPT_ENABLE: u16 = 1; // DLAB=0 时：中断使能寄存器
const FIFO_CONTROL: u16 = 2; // 只写：FIFO 控制寄存器
const LINE_CONTROL: u16 = 3; // 线路控制寄存器（数据格式，最高位是 DLAB）
const MODEM_CONTROL: u16 = 4; // Modem 控制寄存器
const LINE_STATUS: u16 = 5; // 线路状态寄存器

/// LINE_STATUS 第 5 位：发送保持寄存器为空，可以写入下一字节
const TRANSMIT_HOLDING_EMPTY: u8 = 1 << 5;

/// LINE_CONTROL 第 7 位：DLAB，置位后 DATA/INTERRUPT_ENABLE 变成波特率除数寄存器
const DLAB: u8 = 1 << 7;

fn outb(port: u16, value: u8) {
    // SAFETY: 只读写固定 I/O 端口，不触碰内存与栈；in/out 均不影响 EFLAGS
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: 同上
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// 16550 标准初始化序列：关中断 → 设波特率 → 定数据格式 → 开 FIFO → 拉握手线
pub fn init() {
    outb(COM1 + INTERRUPT_ENABLE, 0x00); // 关掉串口所有中断，本阶段纯轮询
    outb(COM1 + LINE_CONTROL, DLAB); // 打开 DLAB 以访问除数寄存器
    outb(COM1 + DATA, 0x01); // 除数低字节：除数 1 = 115200 baud（基准时钟 1.8432 MHz）
    outb(COM1 + INTERRUPT_ENABLE, 0x00); // 除数高字节
    outb(COM1 + LINE_CONTROL, 0x03); // 关 DLAB；0b011 = 8 数据位、无校验、1 停止位（8N1）
    outb(COM1 + FIFO_CONTROL, 0xC7); // 使能 FIFO 并清空收发队列，触发深度 14 字节
    outb(COM1 + MODEM_CONTROL, 0x0B); // DTR + RTS + OUT2：声明数据就绪、请求发送
}

pub fn write_byte(byte: u8) {
    while inb(COM1 + LINE_STATUS) & TRANSMIT_HOLDING_EMPTY == 0 {}
    outb(COM1 + DATA, byte);
}

pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}
