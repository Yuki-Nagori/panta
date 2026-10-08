//! 进程崩溃设施转发及 CXX 边界验收句柄；不持有工程状态。
use crate::{FfiRequest, FfiResponse, MAX_LABEL_BYTES, MAX_REPEAT, bridge};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 任务 047：错误以文本跨边界（C++ 侧 qWarning 呈现，不静默）。
pub(super) fn install_crash_handler(log_dir: &str) -> Result<String, String> {
    panta_foundation::crash::install_crash_handler(log_dir)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

pub(super) fn process(request: &FfiRequest) -> Result<FfiResponse, String> {
    if request.text.is_empty() {
        return Err("ffi.empty_input".to_owned());
    }
    if request.repeat == 0 || request.repeat > MAX_REPEAT {
        return Err(format!("ffi.invalid_repeat: {}", request.repeat));
    }

    let value = format!(
        "{}{}",
        bridge::cpp_prefix(),
        request.text.repeat(request.repeat as usize)
    );
    Ok(FfiResponse {
        value,
        repeat: request.repeat,
    })
}

/// 边界验收专用：验证 Rust panic 在 CXX 胶水中被中止而非以异常穿越到 C++，
/// 与 `Result` 错误的可恢复路径区分；不承载业务功能。
pub(super) fn panic_probe() {
    panic!("ffi.panic_probe");
}

/// 存活 Session 数：证明创建/释放跨边界真实平衡，两侧测试共同断言。
static LIVE_SESSIONS: AtomicUsize = AtomicUsize::new(0);

/// 最小 opaque 句柄：只持有标签，验证 `rust::Box` 的创建/释放所有权语义。
pub struct Session {
    label: String,
}

impl Drop for Session {
    fn drop(&mut self) {
        LIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(super) fn session_create(label: String) -> Result<Box<Session>, String> {
    if label.is_empty() {
        return Err("ffi.empty_label".to_owned());
    }
    if label.len() > MAX_LABEL_BYTES {
        return Err(format!("ffi.invalid_label: {} bytes", label.len()));
    }
    LIVE_SESSIONS.fetch_add(1, Ordering::Relaxed);
    Ok(Box::new(Session { label }))
}

pub(super) fn session_label(session: &Session) -> String {
    session.label.clone()
}

/// 显式释放并返回剩余存活句柄数；Box 析构（不经此函数）走同一 Drop 路径。
pub(super) fn session_close(session: Box<Session>) -> u32 {
    drop(session);
    session_live_count()
}

pub(super) fn session_live_count() -> u32 {
    u32::try_from(LIVE_SESSIONS.load(Ordering::Relaxed)).unwrap_or(u32::MAX)
}
