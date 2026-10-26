//! 文本控制台：光标 + fmt::Write（屏幕/串口双写）+ 滚屏。
//!
//! 全局实例用 `static mut`：现阶段单核，且 IRQ handler 不访问 console。
//! 待收敛项：中断上下文打印或引入多核前，必须换成合适的锁封装。

use core::fmt;
use core::ptr::addr_of_mut;

use crate::fb::{self, COLOR_BG, COLOR_FG};
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

static mut CONSOLE: Option<Console> = None;

/// 初始化全局控制台并清屏；只在 _start 早期、拿到 framebuffer 后调用一次
pub fn init(fb: &'static Framebuffer) {
    let mut cons = Console::new(fb);
    cons.clear();
    // SAFETY: 单核关中断的启动阶段，且全局只在此处写入一次
    unsafe {
        *addr_of_mut!(CONSOLE) = Some(cons);
    }
}

fn global() -> Option<&'static mut Console> {
    // SAFETY: 单核且 IRQ 不访问 console，不存在并发访问；
    // 经 addr_of_mut! 拿裸指针再解引用，避免 static_mut_refs lint 的中间引用
    unsafe { (*addr_of_mut!(CONSOLE)).as_mut() }
}

/// printk! 的后端；未初始化（无 framebuffer 的降级路径）时静默丢弃
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    if let Some(cons) = global() {
        let _ = cons.write_fmt(args);
    }
}

/// 格式化打印到全局控制台（屏幕 + 串口），用法同 std 的 print!
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*))
    };
}
