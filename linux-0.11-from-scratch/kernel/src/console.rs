//! 最小文本控制台：光标 + put_char / write_str。
//! 本 Bite 不做滚屏——调用方保证行数不超出屏幕。

use crate::fb::{self, COLOR_BG, COLOR_FG};
use crate::font::{self, FONT_HEIGHT, FONT_WIDTH};
use crate::limine_fb::Framebuffer;

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

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.put_char(byte);
        }
    }

    pub fn put_char(&mut self, ch: u8) {
        if ch == b'\n' {
            self.newline();
            return;
        }
        // 行尾不够放下一个字形就换行（本 Bite 固定短串，几乎走不到）
        if self.cursor_x + FONT_WIDTH > self.fb.width as usize {
            self.newline();
        }
        self.draw_glyph(ch);
        self.cursor_x += FONT_WIDTH;
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += FONT_HEIGHT;
        // 不做滚屏：超出后继续往下画会被 put_pixel 丢弃
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
