//! M2 Bite1：GDT 扁平接管 + IDT + CPU 异常接管（除零异常验收）。

#![no_std]
#![no_main]

mod console;
mod fb;
mod font;
mod gdt;
mod idt;
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

    // M2 Bite1：用自己的扁平 GDT 与 256 项 IDT 接管段与异常
    gdt::init();
    printk!("M2: gdt ok\n");
    idt::init();
    printk!("M2: idt ok\n");

    // 验收动作：故意触发 #DE 除零异常，预期打印现场后停机（不再三连重启）
    trigger_divide_error();

    halt();
}

/// 用内联汇编的 div 指令真触发 #DE。
/// 注意不能写 Rust 的 `0 / 0`：除法运算符除零只走 panic!，不进 IDT
fn trigger_divide_error() {
    // SAFETY: 故意制造 0/0；div 不读内存不依赖栈，标志位本就由 div 置为未定义
    unsafe {
        asm!(
            "xor edx, edx",
            "xor eax, eax",
            "div eax",
            options(nomem, nostack),
        );
    }
    // div 必然触发异常，永远走不到这里；真走到也说明异常接管失败，halt 兜底即可
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
