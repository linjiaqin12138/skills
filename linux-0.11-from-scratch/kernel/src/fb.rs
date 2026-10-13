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

/// 写单个像素；越界静默丢弃（本 Bite 控制打印行数，不应越界）
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

/// 整屏填成纯色
pub fn clear(fb: &Framebuffer, r: u8, g: u8, b: u8) {
    let w = fb.width as usize;
    let h = fb.height as usize;
    // 32bpp 时按行用同一像素字快填，避免逐像素循环的开销
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
