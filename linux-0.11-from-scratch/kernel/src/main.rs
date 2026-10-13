//! M1 Bite1：Limine framebuffer + 固定 ASCII 字符串点阵输出（无 fmt / 无滚屏）。

#![no_std]
#![no_main]

mod console;
mod fb;
mod font;
mod limine_fb;
mod serial;

use core::arch::asm;
use core::panic::PanicInfo;

use console::Console;
use limine_fb::Framebuffer;

const LINE_HELLO: &str = "M1: hello framebuffer";
const LINE_TWO: &str = "M1: console line 2";

#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    serial::init();

    if !limine_fb::BASE_REVISION.is_supported() {
        serial::write_str("M0: limine base revision 3 not supported\n");
        halt();
    }

    let Some(framebuffer) = limine_fb::first_framebuffer() else {
        serial::write_str("M1: no framebuffer\n");
        serial::write_str("M0: kernel alive\n");
        halt();
    };

    report_fb_meta(framebuffer);

    let mut cons = Console::new(framebuffer);
    cons.clear();
    cons.write_str(LINE_HELLO);
    cons.write_str("\n");
    cons.write_str(LINE_TWO);
    cons.write_str("\n");

    // 屏幕与串口同内容，供自动化断言
    serial::write_str(LINE_HELLO);
    serial::write_str("\n");
    serial::write_str(LINE_TWO);
    serial::write_str("\n");

    // 保留 M0 存活行作回归
    serial::write_str("M0: kernel alive\n");
    halt();
}

/// 串口打印 `M1: fb WxHxBPP` 与 mask，便于核对像素格式
fn report_fb_meta(fb: &Framebuffer) {
    serial::write_str("M1: fb ");
    write_u64(fb.width);
    serial::write_str("x");
    write_u64(fb.height);
    serial::write_str("x");
    write_u64(fb.bpp as u64);
    serial::write_str("\n");

    serial::write_str("M1: fb mask r");
    write_u64(fb.red_mask_size as u64);
    serial::write_str("@");
    write_u64(fb.red_mask_shift as u64);
    serial::write_str(" g");
    write_u64(fb.green_mask_size as u64);
    serial::write_str("@");
    write_u64(fb.green_mask_shift as u64);
    serial::write_str(" b");
    write_u64(fb.blue_mask_size as u64);
    serial::write_str("@");
    write_u64(fb.blue_mask_shift as u64);
    serial::write_str(" model=");
    write_u64(fb.memory_model as u64);
    serial::write_str("\n");
}

/// 无 core::fmt 的十进制输出（Bite2 才上 Write/printk!）
fn write_u64(mut value: u64) {
    if value == 0 {
        serial::write_byte(b'0');
        return;
    }
    let mut digits = [0u8; 20];
    let mut len = 0;
    while value > 0 {
        digits[len] = b'0' + (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    while len > 0 {
        len -= 1;
        serial::write_byte(digits[len]);
    }
}

fn halt() -> ! {
    loop {
        // SAFETY: hlt 只暂停 CPU
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}
