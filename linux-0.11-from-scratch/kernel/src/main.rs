//! M0 内核骨架：能被 Limine 加载，并向串口打印一行存活信息。

#![no_std] // 裸机环境没有 std，只用 core
#![no_main] // 没有 C 运行时，也就没有 main；入口由我们自己定义

mod serial;

use core::arch::asm;
use core::panic::PanicInfo;

/// Limine 基础协议版本（base revision）请求，布局见 PROTOCOL.md "Base Revisions"：
/// 前两个 u64 是识别魔数，第三个是请求的协议版本。
/// bootloader 若支持该版本，会在加载时把第三项原地清零；不支持则保持原样。
#[used] // 这个 static 只被 bootloader 读，需要阻止编译器当死代码删掉
#[unsafe(link_section = ".limine_requests")] // Limine 约定在此段扫描请求
static BASE_REVISION: [u64; 3] = [0xf956_2b2d_5c95_a6c8, 0x6a7b_3849_4453_6bdc, 3];

/// 内核入口，由 Limine 直接跳转过来（`extern "C"` 保持符号名与调用约定稳定）
#[unsafe(no_mangle)] // 符号名必须原样出现在 ELF 里，与 linker.ld 的 ENTRY(_start) 对应
extern "C" fn _start() -> ! {
    serial::init();

    // 协议握手检查：第三项没被清零，说明 bootloader 不支持我们要的协议版本
    if BASE_REVISION[2] != 0 {
        serial::write_str("M0: limine base revision 3 not supported\n");
        halt();
    }

    serial::write_str("M0: kernel alive\n");
    halt();
}

/// 停机：反复执行 hlt 让 CPU 休眠，直到有中断把它唤醒（目前没有中断，即永久停机）
fn halt() -> ! {
    loop {
        // SAFETY: hlt 只让 CPU 暂停，不访问内存与栈
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}
