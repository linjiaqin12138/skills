//! M1 Bite2：printk!（core::fmt 格式化 + 屏幕/串口双写）+ 滚屏压测。

#![no_std]
#![no_main]

mod console;
mod fb;
mod font;
mod limine_fb;
mod serial;

use core::arch::asm;
use core::panic::PanicInfo;

use limine_fb::Framebuffer;

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

    console::init(framebuffer);

    // 回归 marker：M0 存活行 + M1 framebuffer 元数据（改走 printk!，双写屏幕+串口）
    printk!("M0: kernel alive\n");
    printk!(
        "M1: fb {}x{}x{}\n",
        framebuffer.width, framebuffer.height, framebuffer.bpp
    );
    report_fb_mask(framebuffer);

    // fmt 演示：hex 定宽补零 / 十进制 / 字符串
    printk!("M1: fmt hex={:#06x} dec={} str={}\n", 42, 42, "ok");

    // 滚屏压测：60 行远超屏幕容量（实测 qemu std vga 为 800x600，600/16 ≈ 37 行），必然触发滚屏
    for i in 0..60 {
        printk!("M1: scroll line {:02}\n", i);
    }

    halt();
}

/// fb 各颜色通道 mask 一行，供核对像素格式（排障用，断言不依赖）
fn report_fb_mask(fb: &Framebuffer) {
    printk!(
        "M1: fb mask r{}@{} g{}@{} b{}@{} model={}\n",
        fb.red_mask_size,
        fb.red_mask_shift,
        fb.green_mask_size,
        fb.green_mask_shift,
        fb.blue_mask_size,
        fb.blue_mask_shift,
        fb.memory_model,
    );
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
