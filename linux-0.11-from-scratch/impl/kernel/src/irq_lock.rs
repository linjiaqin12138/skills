//! 关中断独占锁：单核下，"不被中断打断"就等于"独占"。
//!
//! 全局 console 既被主流程的 `printk!` 用，也被键盘 handler 用。
//! 若主流程打印到一半被键盘中断打断，handler 里的 `printk!` 会改乱光标，
//! 所以进入临界区前先清 IF（`cli`），出来时再按需恢复。
//! 见 decisions/0003。多核或临界区内需要等待（睡眠、磁盘）时，这把锁不够用。

use core::arch::asm;
use core::cell::UnsafeCell;

/// RFLAGS 的 IF 位（bit 9）：置位表示 CPU 接受可屏蔽外部中断（SDM Vol.1 3.4.3）。
const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;

pub struct IrqLock<T> {
    value: UnsafeCell<T>,
}

// SAFETY: 仅用于单核；`with` 全程关中断，同一时刻不会有第二个访问者。
unsafe impl<T: Send> Sync for IrqLock<T> {}

impl<T> IrqLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    /// 在关中断状态下访问内部值。
    ///
    /// 退出时按"进来时的 IF"恢复，而不是无条件 `sti`：
    /// 中断门进门时 CPU 已清 IF，handler 内再拿锁属于嵌套关中断，
    /// 若无条件开中断，会在 handler 还没做完时放进下一个中断。
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let was_enabled = interrupts_enabled();
        // SAFETY: cli 只清 IF；不加 nomem，让它同时充当编译器屏障，防止临界区内的访存被挪出去。
        unsafe { asm!("cli", options(nostack)) };

        // SAFETY: IF 已清，单核上没有别的执行流能同时拿到这个引用。
        // 同一执行流内不可嵌套调用 `with` 访问同一把锁（会产生两个 &mut）。
        let result = f(unsafe { &mut *self.value.get() });

        if was_enabled {
            // SAFETY: 恢复进入前的状态，临界区已结束。
            unsafe { asm!("sti", options(nostack)) };
        }
        result
    }
}

fn interrupts_enabled() -> bool {
    let rflags: u64;
    // SAFETY: pushfq/pop 只读 RFLAGS，栈净变化为零。
    unsafe { asm!("pushfq", "pop {}", out(reg) rflags) };
    rflags & RFLAGS_INTERRUPT_ENABLE != 0
}
