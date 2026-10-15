//! 线性 framebuffer：清屏与按 RGB mask 写像素。

use crate::limine_fb::{self, Framebuffer};

/// 深色背景（近似 #0a0a12）——拍脑袋，只求可读对比度
pub const COLOR_BG: (u8, u8, u8) = (0x0a, 0x0a, 0x12);
/// 浅色前景（近似 #e8e8f0）——拍脑袋
pub const COLOR_FG: (u8, u8, u8) = (0xe8, 0xe8, 0xf0);

/// 把 (r,g,b) 按 framebuffer 的 mask 折成一个像素字
fn pack_rgb(fb: &Framebuffer, r: u8, g: u8, b: u8) -> u32 {
    // 用 mask size 截断再移位；常见 qemu+Limine 为 8:8:8 @ shift 16/8/0（x8r8g8b8）
    let r_bits = (r as u32) >> (8u32.saturating_sub(fb.red_mask_size as u32));
    let g_bits = (g as u32) >> (8u32.saturating_sub(fb.green_mask_size as u32));
    let b_bits = (b as u32) >> (8u32.saturating_sub(fb.blue_mask_size as u32));
    (r_bits << fb.red_mask_shift)
        | (g_bits << fb.green_mask_shift)
        | (b_bits << fb.blue_mask_shift)
}

/// 写单个像素；越界静默丢弃（正常路径不越界，这里是兜底）
pub fn put_pixel(fb: &Framebuffer, x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= fb.width as usize || y >= fb.height as usize {
        return;
    }
    if fb.memory_model != limine_fb::MEMORY_MODEL_RGB {
        return;
    }

    let pixel = pack_rgb(fb, r, g, b);
    let bpp = fb.bpp as usize;
    let offset = y * fb.pitch as usize + x * (bpp / 8);
    // SAFETY: address() 是 Limine 映射的 FB；pitch/bpp 来自协议响应
    let base = fb.address() as *mut u8;
    unsafe {
        match bpp {
            32 => {
                let ptr = base.add(offset) as *mut u32;
                core::ptr::write_volatile(ptr, pixel);
            }
            24 => {
                // 少见路径：按小端写 3 字节 
                // Rust 没有 u24，没法一次写「三字节整数」，只能拆成三个 u8，并手动按小端顺序写（低字节先）。
                let ptr = base.add(offset);
                core::ptr::write_volatile(ptr, (pixel & 0xff) as u8);
                core::ptr::write_volatile(ptr.add(1), ((pixel >> 8) & 0xff) as u8);
                core::ptr::write_volatile(ptr.add(2), ((pixel >> 16) & 0xff) as u8);
            }
            16 => {
                let ptr = base.add(offset) as *mut u16;
                core::ptr::write_volatile(ptr, pixel as u16);
            }
            _ => {}
        }
    }
}

/// 整屏内容上移 `rows` 像素行，底部腾出的区域填 (r,g,b)。
/// `core::ptr::copy` 语义同 memmove，源/目标区域重叠也安全。
pub fn scroll_up(fb: &Framebuffer, rows: usize, r: u8, g: u8, b: u8) {
    let height = fb.height as usize;
    if rows == 0 {
        return;
    }
    if rows >= height {
        clear(fb, r, g, b);
        return;
    }

    let pitch = fb.pitch as usize;
    let shift = rows * pitch;
    let total = height * pitch;
    let base = fb.address() as *mut u8;
    // SAFETY: base 是 Limine 映射的整块 FB；[shift, total) → [0, total-shift) 均在界内
    unsafe {
        core::ptr::copy(base.add(shift), base, total - shift);
    }

    // 底部逐像素填背景色：put_pixel 自带越界与像素格式检查，兼容 16/24/32 bpp
    for y in (height - rows)..height {
        for x in 0..fb.width as usize {
            put_pixel(fb, x, y, r, g, b);
        }
    }
}

/// 整屏填成纯色
pub fn clear(fb: &Framebuffer, r: u8, g: u8, b: u8) {
    let w = fb.width as usize;
    let h = fb.height as usize;
    // 快路径：32bpp 时按行填同一像素字。与下面 put_pixel 慢路径完全等效，
    // 纯属优化且实测差距很小（release 下 ~20%，一次清屏 µs 级）——
    // 留作"同语义换实现"的示例，删掉行为不变。慢路径兜住 16/24bpp。
    if fb.bpp == 32 && fb.memory_model == limine_fb::MEMORY_MODEL_RGB {
        let pixel = pack_rgb(fb, r, g, b);
        let pitch = fb.pitch as usize;
        // SAFETY: 同上，整块 FB 可写
        unsafe {
            for y in 0..h {
                let row = (fb.address() as *mut u8).add(y * pitch) as *mut u32;
                for x in 0..w {
                    core::ptr::write_volatile(row.add(x), pixel);
                }
            }
        }
        return;
    }
    for y in 0..h {
        for x in 0..w {
            put_pixel(fb, x, y, r, g, b);
        }
    }
}
