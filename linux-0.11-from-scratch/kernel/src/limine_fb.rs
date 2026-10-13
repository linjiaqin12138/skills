//! Limine framebuffer：经官方 `limine` crate 取第一块显存。
//! 请求静态量放进 `.limine_requests_*` 段，由 linker.ld 保证 start → 请求 → end 顺序。

use limine::framebuffer::FRAMEBUFFER_RGB;
use limine::request::FramebufferRequest;
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

pub use limine::framebuffer::Framebuffer;
pub const MEMORY_MODEL_RGB: u8 = FRAMEBUFFER_RGB;

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

/// 继续请求 base revision 3（与 M0 一致；crate 最高支持 6，向下兼容）
#[used]
#[unsafe(link_section = ".limine_requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(3);

#[used]
#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

/// 取第一块 framebuffer；无响应或列表为空则 None
pub fn first_framebuffer() -> Option<&'static Framebuffer> {
    let response = FRAMEBUFFER_REQUEST.response()?;
    response.framebuffers().first().copied()
}
