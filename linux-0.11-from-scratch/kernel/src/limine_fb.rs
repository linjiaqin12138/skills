//! Limine Framebuffer Feature 请求/响应布局。
//! 字段与魔数对照 `reference/limine-protocol/include/limine.h` 与 PROTOCOL.md
//! "Framebuffer Feature"；禁止当依赖链接 limine.h，只按协议手写。

/// LIMINE_COMMON_MAGIC（limine.h）
const COMMON_MAGIC_0: u64 = 0xc7b1_dd30_df4c_8b88;
const COMMON_MAGIC_1: u64 = 0x0a82_e883_a194_f07b;

/// LIMINE_FRAMEBUFFER_REQUEST_ID 后半（limine.h）
const FB_REQUEST_ID_2: u64 = 0x9d58_27dc_d881_dd75;
const FB_REQUEST_ID_3: u64 = 0xa314_8604_f6fa_b11b;

/// LIMINE_FRAMEBUFFER_RGB（limine.h）
pub const MEMORY_MODEL_RGB: u8 = 1;

/// 对应 `struct limine_framebuffer_request`
#[repr(C)]
pub struct FramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut FramebufferResponse,
}

/// 对应 `struct limine_framebuffer_response`
#[repr(C)]
pub struct FramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *mut *mut Framebuffer,
}

/// 对应 `struct limine_framebuffer`（本 Bite 只用到 address/几何/mask；
/// 后面的 edid/modes 字段保留布局对齐，避免读错偏移）
#[repr(C)]
pub struct Framebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *mut u8,
    // response revision 1 起才保证有意义；本 Bite 不读
    pub mode_count: u64,
    pub modes: *mut *mut u8,
}

/// 放进 `.limine_requests`：Limine 扫描后填写 `response`
#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: [
        COMMON_MAGIC_0,
        COMMON_MAGIC_1,
        FB_REQUEST_ID_2,
        FB_REQUEST_ID_3,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

/// 取第一块 framebuffer；失败返回 None（无响应 / 数量为 0）
pub fn first_framebuffer() -> Option<&'static Framebuffer> {
    // SAFETY: Limine 在跳进内核前写好 response；之后只读
    let request = unsafe { &*&raw const FRAMEBUFFER_REQUEST };
    let response = request.response;
    if response.is_null() {
        return None;
    }
    // SAFETY: response 非空时由 bootloader 保证指向有效结构
    let response = unsafe { &*response };
    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return None;
    }
    // SAFETY: count>=1 时 framebuffers[0] 有效
    let fb_ptr = unsafe { *response.framebuffers };
    if fb_ptr.is_null() {
        return None;
    }
    Some(unsafe { &*fb_ptr })
}
