//! 文本控制台：光标 + fmt::Write（屏幕/串口双写）+ 滚屏。
//!
//! 全局实例放在关中断锁里：主流程与键盘 IRQ handler 都会 printk!，
//! 不加锁的话，打印到一半被打断会让两边互相改乱光标。见 decisions/0003。

use core::fmt;

use crate::fb::{self, COLOR_BG, COLOR_FG};
use crate::irq_lock::IrqLock;
use crate::font::{self, FONT_HEIGHT, FONT_WIDTH};
use crate::limine_fb::Framebuffer;
use crate::serial;

pub struct Console {
    fb: &'static Framebuffer,
    /// 下一字符左上角的像素列
    cursor_x: usize,
    /// 下一字符左上角的像素行
    cursor_y: usize,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}

impl Console {
    pub fn new(fb: &'static Framebuffer) -> Self {
        Self {
            fb,
            cursor_x: 0,
            cursor_y: 0,
            fg: COLOR_FG,
            bg: COLOR_BG,
        }
    }

    pub fn clear(&mut self) {
        fb::clear(self.fb, self.bg.0, self.bg.1, self.bg.2);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn put_char(&mut self, ch: u8) {
        if ch == b'\n' {
            self.newline();
            return;
        }
        // 行尾不够放下一个字形就换行
        if self.cursor_x + FONT_WIDTH > self.fb.width as usize {
            self.newline();
        }
        self.draw_glyph(ch);
        self.cursor_x += FONT_WIDTH;
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += FONT_HEIGHT;
        // 光标触底：整屏上移一行字符的高度，光标留在最后一行
        if self.cursor_y + FONT_HEIGHT > self.fb.height as usize {
            fb::scroll_up(self.fb, FONT_HEIGHT, self.bg.0, self.bg.1, self.bg.2);
            self.cursor_y -= FONT_HEIGHT;
        }
    }

    fn draw_glyph(&self, ch: u8) {
        let rows = font::glyph(ch);
        for (row_i, &bits) in rows.iter().enumerate() {
            let py = self.cursor_y + row_i;
            for col in 0..FONT_WIDTH {
                // bit7 = 最左像素（VGA 惯例，见 font.rs 说明）
                let on = (bits & (0x80 >> col)) != 0;
                let (r, g, b) = if on { self.fg } else { self.bg };
                fb::put_pixel(self.fb, self.cursor_x + col, py, r, g, b);
            }
        }
    }
}

// SAFETY: Framebuffer 里的裸指针指向显存，让编译器默认认为它不能跨线程。
// 本内核单核、没有线程，Console 只经 CONSOLE 的关中断锁访问，不会同时被两个执行流碰到。
unsafe impl Send for Console {}

/// 屏幕和串口是同一个控制台的两个出口，双写点收在这里：
/// 每个字节既上屏也镜像到串口，两处内容始终一致。
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.put_char(byte);
            serial::write_byte(byte);
        }
        Ok(())
    }
}

/// 未初始化（无 framebuffer 的降级路径）时为 None，printk! 静默丢弃。
static CONSOLE: IrqLock<Option<Console>> = IrqLock::new(None);

/// 初始化全局控制台并清屏；只在 _start 早期、拿到 framebuffer 后调用一次
pub fn init(fb: &'static Framebuffer) {
    let mut cons = Console::new(fb);
    cons.clear();
    CONSOLE.with(|slot| *slot = Some(cons));
}

/// printk! 的后端。整次格式化都在锁内完成，所以一次 printk! 的输出不会被别的 printk! 切开。
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    CONSOLE.with(|slot| {
        if let Some(cons) = slot {
            let _ = cons.write_fmt(args);
        }
    });
}

/// 格式化打印到全局控制台（屏幕 + 串口），用法同 std 的 print!
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*))
    };
}
