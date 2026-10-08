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

#[cfg(test)]
mod tests {
    use crate::{
        FfiRequest, FfiResponse, MAX_LABEL_BYTES, install_crash_handler, panic_probe, process,
        session_close, session_create, session_label, session_live_count,
    };
    use std::sync::{Mutex, MutexGuard};
    /// 存活计数是进程级共享状态；触碰它的测试先取锁串行化，避免并行互扰。
    fn session_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn round_trip_calls_cpp_and_preserves_unicode() -> Result<(), Box<dyn std::error::Error>> {
        let response = process(&FfiRequest {
            text: "界".to_owned(),
            repeat: 2,
        })?; // valid request failed: {error}
        assert_eq!(
            response,
            FfiResponse {
                value: "ffi:界界".to_owned(),
                repeat: 2,
            }
        );
        Ok(())
    }

    #[test]
    fn empty_text_is_a_structured_error() {
        let error = match process(&FfiRequest {
            text: String::new(),
            repeat: 1,
        }) {
            Ok(response) => panic!("empty input unexpectedly succeeded: {response:?}"),
            Err(error) => error,
        };
        assert_eq!(error, "ffi.empty_input");
    }

    #[test]
    fn repeat_bounds_are_rejected() {
        for repeat in [0, 9] {
            let error = match process(&FfiRequest {
                text: "x".to_owned(),
                repeat,
            }) {
                Ok(response) => panic!("invalid repeat unexpectedly succeeded: {response:?}"),
                Err(error) => error,
            };
            assert!(error.starts_with("ffi.invalid_repeat:"));
        }
    }

    #[test]
    #[should_panic(expected = "ffi.panic_probe")]
    fn panic_probe_panics_with_boundary_code() {
        panic_probe();
    }

    #[test]
    fn sessions_create_use_and_release_balance() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = session_lock();
        assert_eq!(session_live_count(), 0);

        let alpha = match session_create("会话-α".to_owned()) {
            Ok(session) => session,
            Err(error) => panic!("valid label rejected: {error}"),
        };
        let beta = match session_create("会话-β".to_owned()) {
            Ok(session) => session,
            Err(error) => panic!("valid label rejected: {error}"),
        };
        assert_eq!(session_live_count(), 2);
        assert_eq!(session_label(&alpha), "会话-α");
        assert_eq!(session_label(&beta), "会话-β");

        // 逆序释放：证明各 Box 独立持有，释放顺序不影响计数平衡。
        assert_eq!(session_close(beta), 1);
        assert_eq!(session_close(alpha), 0);
        assert_eq!(session_live_count(), 0);
        Ok(())
    }

    #[test]
    fn session_rejects_invalid_labels_without_leaking() {
        let _guard = session_lock();
        match session_create(String::new()) {
            Ok(_) => panic!("empty label unexpectedly accepted"),
            Err(error) => assert_eq!(error, "ffi.empty_label"),
        }
        match session_create("x".repeat(MAX_LABEL_BYTES + 1)) {
            Ok(_) => panic!("oversized label unexpectedly accepted"),
            Err(error) => assert!(error.starts_with("ffi.invalid_label:")),
        }
        assert_eq!(session_live_count(), 0);
    }

    #[test]
    fn session_drop_releases_live_count() {
        let _guard = session_lock();
        let session = match session_create("drop".to_owned()) {
            Ok(session) => session,
            Err(error) => panic!("valid label rejected: {error}"),
        };
        assert_eq!(session_live_count(), 1);
        // 不经 session_close 的 Box 析构走同一 Drop 路径。
        drop(session);
        assert_eq!(session_live_count(), 0);
    }

    #[test]
    fn crash_handler_rejects_a_file_used_as_log_directory() -> std::io::Result<()> {
        let path =
            std::env::temp_dir().join(format!("panta-ffi-crash-not-dir-{}", std::process::id()));
        std::fs::write(&path, b"not a directory")?;
        let text = path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "临时路径不是 UTF-8")
        })?;
        let error = install_crash_handler(text)
            .err()
            .ok_or_else(|| std::io::Error::other("文件路径不应成为日志目录"))?;
        assert!(!error.is_empty());
        std::fs::remove_file(path)?;
        Ok(())
    }
}
