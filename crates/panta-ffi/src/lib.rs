//! 最小 CXX 双向边界：验证 DTO、opaque 句柄所有权、错误转换和 C++ 实现调用。

use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_REPEAT: u32 = 8;
/// 句柄标签按 UTF-8 字节数设界，与请求文本的有界策略一致。
const MAX_LABEL_BYTES: usize = 64;

// unsafe 只由 CXX 桥接宏生成（胶水 extern/函数/块）；边界安全前提由 cxx
// 运行时的类型检查与 ffi.hpp 签名一致性承担，本 crate 对外只暴露安全签名。
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "panta::ffi")]
pub mod bridge {
    /// 只跨边界传递 UTF-8 文本和有界整数，不暴露 Qt/CAE 类型布局。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FfiRequest {
        pub text: String,
        pub repeat: u32,
    }

    /// 成功响应保留机器可检查的重复次数，避免调用方解析展示文本。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FfiResponse {
        pub value: String,
        pub repeat: u32,
    }

    unsafe extern "C++" {
        include!("panta/ffi.hpp");

        fn cpp_prefix() -> String;
    }

    extern "Rust" {
        /// Rust 侧拥有的 opaque 句柄：C++ 经 `rust::Box` 持有唯一所有权，
        /// 释放只能把 Box move 回 Rust；不暴露指针或复制语义。
        type Session;

        fn session_create(label: String) -> Result<Box<Session>>;
        fn session_label(session: &Session) -> String;
        /// 显式释放并返回剩余存活句柄数，供两侧断言释放真实发生。
        fn session_close(session: Box<Session>) -> u32;
        fn session_live_count() -> u32;

        fn process(request: &FfiRequest) -> Result<FfiResponse>;
        fn panic_probe();
    }
}

pub use bridge::{FfiRequest, FfiResponse};

fn process(request: &FfiRequest) -> Result<FfiResponse, String> {
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
fn panic_probe() {
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

fn session_create(label: String) -> Result<Box<Session>, String> {
    if label.is_empty() {
        return Err("ffi.empty_label".to_owned());
    }
    if label.len() > MAX_LABEL_BYTES {
        return Err(format!("ffi.invalid_label: {} bytes", label.len()));
    }
    LIVE_SESSIONS.fetch_add(1, Ordering::Relaxed);
    Ok(Box::new(Session { label }))
}

fn session_label(session: &Session) -> String {
    session.label.clone()
}

/// 显式释放并返回剩余存活句柄数；Box 析构（不经此函数）走同一 Drop 路径。
fn session_close(session: Box<Session>) -> u32 {
    drop(session);
    session_live_count()
}

fn session_live_count() -> u32 {
    u32::try_from(LIVE_SESSIONS.load(Ordering::Relaxed)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        FfiRequest, FfiResponse, MAX_LABEL_BYTES, panic_probe, process, session_close,
        session_create, session_label, session_live_count,
    };
    use std::sync::{Mutex, MutexGuard};

    /// 存活计数是进程级共享状态；触碰它的测试先取锁串行化，避免并行互扰。
    fn session_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn round_trip_calls_cpp_and_preserves_unicode() {
        let response = process(&FfiRequest {
            text: "界".to_owned(),
            repeat: 2,
        })
        .unwrap_or_else(|error| panic!("valid request failed: {error}"));
        assert_eq!(
            response,
            FfiResponse {
                value: "ffi:界界".to_owned(),
                repeat: 2,
            }
        );
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
    fn sessions_create_use_and_release_balance() {
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
}
