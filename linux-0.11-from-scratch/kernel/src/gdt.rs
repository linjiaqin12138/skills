//! 扁平 GDT：null + 内核代码段 + 内核数据段，从 Limine 的 GDT 接管段寄存器。
//!
//! 长模式下分段已基本失效（基址/限长被忽略），GDT 只承担两件事：
//! 给 CS/SS/DS 等段寄存器一个合法的描述符可指，以及用 L 位声明"这是 64-bit 代码段"。
//! 不建 TSS：那是 M8 特权级切换才需要的东西。

use core::arch::asm;

/// 内核代码段选择子：GDT 索引 1，TI=0（GDT），RPL=0
pub const KERNEL_CODE_SEL: u16 = 0x08;
/// 内核数据段选择子：GDT 索引 2
pub const KERNEL_DATA_SEL: u16 = 0x10;

/// 段描述符（Intel SDM Vol.3 3.4.5，8 字节）。
/// 64-bit 代码/数据段的 base 与 limit 被 CPU 忽略，因此只构造
/// access byte（位 40~47）与 flags 高半字节（位 52~55），其余全零。
#[repr(transparent)]
struct SegmentDescriptor(u64);

impl SegmentDescriptor {
    const fn null() -> Self {
        Self(0)
    }

    /// 内核代码段：access = P|DPL0|S|type(EXEC|READ|ACCESSED)，flags = L
    /// （SDM Vol.3 3.4.5.1；L=1 时 D/B 位必须为 0）
    const fn kernel_code() -> Self {
        const PRESENT: u64 = 1 << 47;
        const S_CODE_OR_DATA: u64 = 1 << 44;
        const TYPE_EXEC_READ_ACCESSED: u64 = 0b1011 << 40;
        const FLAG_LONG_MODE: u64 = 1 << 53;
        Self(PRESENT | S_CODE_OR_DATA | TYPE_EXEC_READ_ACCESSED | FLAG_LONG_MODE)
    }

    /// 内核数据段：access = P|DPL0|S|type(WRITE|ACCESSED)。
    /// 长模式下 CPU 不检查数据段的 L 位，flags 全零即可
    const fn kernel_data() -> Self {
        const PRESENT: u64 = 1 << 47;
        const S_CODE_OR_DATA: u64 = 1 << 44;
        const TYPE_DATA_WRITE_ACCESSED: u64 = 0b0011 << 40;
        Self(PRESENT | S_CODE_OR_DATA | TYPE_DATA_WRITE_ACCESSED)
    }
}

/// 描述符的 Accessed 位预先置 1：CPU 每次装载段寄存器都会把内存中描述符的
/// A 位置 1（SDM Vol.3 3.4.5.1），而我们的 GDT 在只读的 .rodata 里——
/// 若 A 初始为 0，这次写会触发 #PF。预置 A=1 后 CPU 无需回写。
static GDT: [SegmentDescriptor; 3] = [
    SegmentDescriptor::null(),
    SegmentDescriptor::kernel_code(),
    SegmentDescriptor::kernel_data(),
];

/// lgdt 的操作数内存布局（SDM Vol.3 3.5.1）：2 字节限长 + 8 字节基址
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

/// 加载我们自己的 GDT，并重载全部段寄存器完成接管
pub fn init() {
    let gdtr = GdtPointer {
        limit: (size_of_val(&GDT) - 1) as u16,
        base: GDT.as_ptr() as u64,
    };
    // SAFETY: gdtr 指向合法的 GDT；栈上的 gdtr 在 lgdt 读完后即可失效（值已进 GDTR 寄存器）
    unsafe {
        asm!("lgdt [{}]", in(reg) &gdtr, options(nostack));
    }
    reload_segment_registers();
}

/// lgdt 只换了 GDT 表，段寄存器里仍是 Limine 的旧选择子，必须逐个重载：
/// CS 不能直接用 mov 写，标准做法是构造一个远返回帧，retfq 顺带换 CS。
fn reload_segment_registers() {
    // SAFETY: KERNEL_CODE_SEL/KERNEL_DATA_SEL 刚随新 GDT 生效，均为合法 ring0 选择子
    unsafe {
        asm!(
            "push {code_sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ds, {data_sel:x}",
            "mov es, {data_sel:x}",
            "mov fs, {data_sel:x}",
            "mov gs, {data_sel:x}",
            "mov ss, {data_sel:x}",
            code_sel = const KERNEL_CODE_SEL as u64,
            data_sel = in(reg) KERNEL_DATA_SEL,
            tmp = out(reg) _,
            options(nomem, preserves_flags),
        );
    }
}
