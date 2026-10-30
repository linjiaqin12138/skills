//! M3 Bite2：IRQ32 定时器 + IRQ33 键盘；主流程观察并打印 jiffies，键盘在中断里回显。

#![no_std]
#![no_main]

mod acpi;
mod console;
mod fb;
mod font;
mod gdt;
mod idt;
mod io;
mod ioapic;
mod irq_lock;
mod keyboard;
mod lapic;
mod limine_fb;
mod mmio;
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

    // M2 Bite1 成果保留：用自己的扁平 GDT 与 256 项 IDT 接管段与异常
    gdt::init();
    printk!("M2: gdt ok\n");
    idt::init();
    printk!("M2: idt ok\n");

    if !lapic::init() {
        printk!("M3: LAPIC initialization failed\n");
        halt();
    }
    printk!("M3: LAPIC timer 100 Hz\n");

    let Some(route) = keyboard::init() else {
        printk!("M3: keyboard MADT discovery or IOAPIC routing failed\n");
        halt();
    };
    printk!(
        "M3: MADT IOAPIC base={:#x} IRQ1 -> GSI {} polarity={} trigger={}\n",
        route.ioapic_base,
        route.gsi,
        if route.active_low { "low" } else { "high" },
        if route.level_triggered { "level" } else { "edge" },
    );
    printk!("M3: keyboard IRQ1 -> vector {}\n", keyboard::KEYBOARD_VECTOR);
    report_jiffies();
}

/// 中断只计数；这里在正常上下文读取实际值，每 100 tick 打印一次。
fn report_jiffies() -> ! {
    let mut next_report = 100;
    loop {
        let observed = lapic::jiffies();
        if observed >= next_report {
            printk!("M3: jiffies={}\n", observed);
            next_report += 100;
        } else {
            // SAFETY: 定时器周期中断会唤醒 CPU；没有调度器时无需做其他工作。
            unsafe { asm!("hlt", options(nomem, nostack)) };
        }
    }
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

/// panic_handler：把 Rust 的 panic 接到内核的 printk! 通道（屏幕 + 串口双写）。
/// 打印消息与源码位置后停机，让 panic 不再静默挂死。
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printk!("M2: panic handler entered\n");
    printk!("PANIC: {}\n", info.message());
    match info.location() {
        // 位置里的路径是 cargo 交给 rustc 的相对路径（"src/main.rs"），
        // 补上包名让它带上 "kernel/" 前缀，日志里能直接定位到仓库中的文件
        Some(loc) => printk!(
            "panicked at {}/{}:{}:{}\n",
            env!("CARGO_PKG_NAME"),
            loc.file(),
            loc.line(),
            loc.column()
        ),
        None => printk!("panicked at <unknown location>\n"),
    }
    // 回归 marker：panic 路径跑完了（与 "M2: gdt ok" 同风格）
    printk!("M2: panic ok\n");
    halt()
}
