//! IDT：256 项 64-bit interrupt gate，接管向量 0~31 的全部 CPU 异常。
//!
//! 异常发生时的栈帧（低地址 → 高地址，即压栈逆序）：
//!   r15..rax（15 个通用寄存器，common_exception_entry 压入）
//!   vector（stub 压入的向量号）
//!   error_code（有错误码的向量由 CPU 压入；无错误码的由 stub 压占位 0）
//!   rip / cs / rflags（CPU 自动压入；无特权级切换，故无 rsp/ss）
//! handler 打印现场后 hlt 停机，本 Bite 不需要从异常返回。

use core::arch::{asm, naked_asm};
use core::ptr::addr_of_mut;

use crate::gdt::KERNEL_CODE_SEL;
use crate::printk;

/// IDT 项数：x86-64 固定 256 个向量（SDM Vol.3 6.10）
const IDT_ENTRIES: usize = 256;

/// 门描述符（SDM Vol.3 6.14.1，16 字节）。
/// 只实现 interrupt gate：它进门时自动清 IF，
/// 保证异常处理期间不被外部中断打断（本 Bite 本来也没开外部中断）。
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,  // 处理函数地址位 0..15
    selector: u16,    // 目标代码段选择子
    ist: u8,          // 位 0..2 是 IST 索引，0 = 不用 IST（本 Bite 无独立异常栈）
    type_attrs: u8,   // P | DPL | 0 | gate type
    offset_mid: u16,  // 地址位 16..31
    offset_high: u32, // 地址位 32..63
    reserved: u32,    // 必须为 0（SDM 规定）
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attrs: 0, // P=0：CPU 视该项为空洞
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    const fn interrupt_gate(handler: u64) -> Self {
        const PRESENT: u8 = 1 << 7;
        const DPL_RING0: u8 = 0;
        // SDM Vol.3 6.14.1：type=0b1110 即 64-bit interrupt gate
        const GATE_INTERRUPT_64: u8 = 0b1110;
        Self {
            offset_low: handler as u16,
            selector: KERNEL_CODE_SEL,
            ist: 0,
            type_attrs: PRESENT | DPL_RING0 | GATE_INTERRUPT_64,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

/// 异常现场：与 common_exception_entry 的压栈顺序一一对应，
/// 改任何一边都必须同步另一边
#[repr(C)]
struct ExceptionFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
}

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry::missing(); IDT_ENTRIES];

/// 向量 0~31 的异常助记名（SDM Vol.3 6.15，按向量号索引）
const EXCEPTION_NAMES: [&str; 32] = [
    "divide error",                // 0  #DE
    "debug",                       // 1  #DB
    "nmi",                         // 2
    "breakpoint",                  // 3  #BP
    "overflow",                    // 4  #OF
    "bound range exceeded",        // 5  #BR
    "invalid opcode",              // 6  #UD
    "device not available",        // 7  #NM
    "double fault",                // 8  #DF
    "coprocessor segment overrun", // 9  80286 遗留，现代 CPU 不产生
    "invalid tss",                 // 10 #TS
    "segment not present",         // 11 #NP
    "stack segment fault",         // 12 #SS
    "general protection",          // 13 #GP
    "page fault",                  // 14 #PF
    "reserved",                    // 15
    "x87 fpu error",               // 16 #MF
    "alignment check",             // 17 #AC
    "machine check",               // 18 #MC
    "simd exception",              // 19 #XM
    "virtualization",              // 20 #VE
    "control protection",          // 21 #CP
    "reserved",                    // 22
    "reserved",                    // 23
    "reserved",                    // 24
    "reserved",                    // 25
    "reserved",                    // 26
    "reserved",                    // 27
    "reserved",                    // 28
    "reserved",                    // 29
    "reserved",                    // 30
    "reserved",                    // 31
];

/// 公共出口：保存全部通用寄存器，把帧指针交给 Rust handler。
/// 各 stub 用 jmp 进来（不是 call），因此栈顶仍是 stub 留下的 vector 字段。
#[unsafe(naked)]
unsafe extern "C" fn common_exception_entry() {
    naked_asm!(
        // 压栈顺序与 ExceptionFrame 字段互为镜像（r15 最后压，地址最低）
        "push rax",
        "push rcx",
        "push rdx",
        "push rbx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov rdi, rsp",
        // 异常可发生在任意对齐状态，而 Rust 代码可能用到 SSE 指令；
        // 调用 handler 前把栈对齐到 16 字节（handler 不返回，无需恢复）
        "and rsp, -16",
        "call {handler}",
        // 兜底：handler 设计上不返回，万一返回就停机
        "2:",
        "hlt",
        "jmp 2b",
        handler = sym exception_handler,
    );
}

/// 生成一个异常入口 stub 并注册进 IDT 安装函数。
/// 无错误码的向量先压占位 0，使所有异常的栈帧布局统一
/// （错误码向量清单见 SDM Vol.3 6.15：8、10~14、17、21）。
macro_rules! exception_entry {
    ($name:ident, $vector:literal, no_error_code) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                "push 0",
                concat!("push ", $vector),
                "jmp {common}",
                common = sym common_exception_entry,
            );
        }
    };
    ($name:ident, $vector:literal, has_error_code) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                concat!("push ", $vector),
                "jmp {common}",
                common = sym common_exception_entry,
            );
        }
    };
}

/// 批量定义 32 个异常入口，并生成对应的 IDT 安装语句
macro_rules! define_exceptions {
    ($( $vector:literal => $name:ident : $err:ident ),* $(,)?) => {
        $( exception_entry!($name, $vector, $err); )*

        fn install_exception_entries(idt: &mut [IdtEntry; IDT_ENTRIES]) {
            // 先收敛成函数指针再取地址：直接 `$name as usize` 会触发
            // function_casts_as_integer lint（裸 fn item 强转整数可读性差）
            $( idt[$vector] = IdtEntry::interrupt_gate(entry_addr($name)); )*
        }

        fn entry_addr(entry: unsafe extern "C" fn()) -> u64 {
            entry as usize as u64
        }
    };
}

define_exceptions! {
    0  => entry_divide_error                : no_error_code,
    1  => entry_debug                       : no_error_code,
    2  => entry_nmi                         : no_error_code,
    3  => entry_breakpoint                  : no_error_code,
    4  => entry_overflow                    : no_error_code,
    5  => entry_bound_range_exceeded        : no_error_code,
    6  => entry_invalid_opcode              : no_error_code,
    7  => entry_device_not_available        : no_error_code,
    8  => entry_double_fault                : has_error_code,
    9  => entry_coprocessor_segment_overrun : no_error_code,
    10 => entry_invalid_tss                 : has_error_code,
    11 => entry_segment_not_present         : has_error_code,
    12 => entry_stack_segment_fault         : has_error_code,
    13 => entry_general_protection          : has_error_code,
    14 => entry_page_fault                  : has_error_code,
    15 => entry_reserved_15                 : no_error_code,
    16 => entry_x87_fpu_error               : no_error_code,
    17 => entry_alignment_check             : has_error_code,
    18 => entry_machine_check               : no_error_code,
    19 => entry_simd_exception              : no_error_code,
    20 => entry_virtualization              : no_error_code,
    21 => entry_control_protection          : has_error_code,
    22 => entry_reserved_22                 : no_error_code,
    23 => entry_reserved_23                 : no_error_code,
    24 => entry_reserved_24                 : no_error_code,
    25 => entry_reserved_25                 : no_error_code,
    26 => entry_reserved_26                 : no_error_code,
    27 => entry_reserved_27                 : no_error_code,
    28 => entry_reserved_28                 : no_error_code,
    29 => entry_reserved_29                 : no_error_code,
    30 => entry_reserved_30                 : no_error_code,
    31 => entry_reserved_31                 : no_error_code,
}

/// 填写 IDT 并 lidt，完成异常接管
pub fn init() {
    // SAFETY: 单核无外部中断阶段，IDT 只在此处写入一次
    let idt = unsafe { &mut *addr_of_mut!(IDT) };
    install_exception_entries(idt);

    let idtr = IdtPointer {
        limit: (size_of_val(idt) - 1) as u16,
        base: idt.as_ptr() as u64,
    };
    // SAFETY: idtr 指向刚填好的 IDT
    unsafe {
        asm!("lidt [{}]", in(reg) &idtr, options(nostack));
    }
}

/// lidt 的操作数内存布局（SDM Vol.3 6.10）：2 字节限长 + 8 字节基址
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// 所有异常的统一出口：打印现场后停机。
/// 打印格式是 Makefile 验收断言的依据，改动需同步 check-m2-b1。
extern "C" fn exception_handler(frame: &ExceptionFrame) -> ! {
    let vector = frame.vector as usize;
    // 同特权级异常时 CPU 不自动保存 rsp，但可以从帧反推：
    // CPU 压入 rip/cs/rflags 之前的栈顶，就是 rip 字段上方三个 u64 的末尾
    let exception_rsp = &raw const frame.rip as u64 + 3 * size_of::<u64>() as u64;
    printk!("M2: EXCEPTION #{} {}\n", vector, EXCEPTION_NAMES[vector]);
    printk!("M2: error_code={:#x}\n", frame.error_code);
    printk!(
        "M2: rip={:#018x} cs={:#06x} rflags={:#018x}\n",
        frame.rip, frame.cs, frame.rflags,
    );
    printk!(
        "M2: rax={:#018x} rbx={:#018x} rcx={:#018x} rdx={:#018x}\n",
        frame.rax, frame.rbx, frame.rcx, frame.rdx,
    );
    printk!(
        "M2: rsi={:#018x} rdi={:#018x} rbp={:#018x} rsp={:#018x}\n",
        frame.rsi, frame.rdi, frame.rbp, exception_rsp,
    );
    printk!(
        "M2: r8={:#018x} r9={:#018x} r10={:#018x} r11={:#018x}\n",
        frame.r8, frame.r9, frame.r10, frame.r11,
    );
    printk!(
        "M2: r12={:#018x} r13={:#018x} r14={:#018x} r15={:#018x}\n",
        frame.r12, frame.r13, frame.r14, frame.r15,
    );

    // 本 Bite 不从异常返回：停机等待人工检查现场。
    // interrupt gate 进门时已自动清 IF，hlt 不会被可屏蔽中断唤醒
    loop {
        // SAFETY: hlt 只暂停 CPU
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}
