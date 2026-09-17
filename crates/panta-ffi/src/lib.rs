//! 最小 CXX 双向边界：验证 DTO、opaque 句柄所有权、错误转换、C++ 实现
//! 调用，以及 Rust 拥有的任务生命周期（任务 008）。

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

    /// 任务生命周期事件种类，与 panta_core::task::TaskEventKind 一一对应。
    pub enum TaskEventKind {
        Started = 0,
        Succeeded = 1,
        Failed = 2,
        Cancelled = 3,
    }

    /// 一次状态转换的事件；`code` 为机器可读错误码（仅 Failed 非空），
    /// `detail` 为诊断上下文，面向日志/Console 而非用户摘要。
    /// CXX 共享枚举不支持自定义 derive，因此本结构不派生 Debug/Clone。
    pub struct TaskEvent {
        pub task_id: u64,
        pub kind: TaskEventKind,
        pub code: String,
        pub detail: String,
    }

    /// 结构化日志行：跨语言与 UI 侧凭 `task_id` 关联同一任务。
    pub struct TaskLogLine {
        pub task_id: u64,
        pub message: String,
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

        /// 任务生命周期管理器（panta_core::task::TaskManager 的包装）：
        /// 事件为拉取式队列，无跨语言回调；Box 析构即关闭并 join 工作线程。
        type TaskService;

        fn task_service_new() -> Box<TaskService>;
        fn task_service_submit(
            service: &TaskService,
            label: String,
            duration_ms: u64,
            fail: bool,
        ) -> Result<u64>;
        fn task_service_cancel(service: &TaskService, task_id: u64) -> bool;
        fn task_service_drain(service: &TaskService) -> Vec<TaskEvent>;
        fn task_service_recent_logs(service: &TaskService) -> Vec<TaskLogLine>;
        fn task_service_running(service: &TaskService) -> u32;

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

/// panta-core 任务管理器的 FFI 包装：与 `Box<TaskService>` 同生命周期，
/// 析构即关闭并 join 全部工作线程（关闭后不存在可触达的执行体）。
pub struct TaskService {
    manager: panta_core::task::TaskManager,
}

fn task_service_new() -> Box<TaskService> {
    Box::new(TaskService {
        manager: panta_core::task::TaskManager::new(),
    })
}

fn task_service_submit(
    service: &TaskService,
    label: String,
    duration_ms: u64,
    fail: bool,
) -> Result<u64, String> {
    let duration = std::time::Duration::from_millis(duration_ms);
    if duration > panta_core::task::MAX_TASK_DURATION {
        return Err(format!("task.invalid_duration: {duration_ms}"));
    }
    service
        .manager
        .submit(&label, duration, fail)
        .map_err(|error| match error {
            panta_core::task::SubmitError::EmptyLabel => "task.empty_label".to_owned(),
            panta_core::task::SubmitError::TooLongDuration(millis) => {
                format!("task.invalid_duration: {millis}")
            }
            panta_core::task::SubmitError::SpawnFailed(detail) => {
                format!("task.spawn_failed: {detail}")
            }
        })
}

fn task_service_cancel(service: &TaskService, task_id: u64) -> bool {
    service.manager.cancel(task_id)
}

fn task_service_drain(service: &TaskService) -> Vec<bridge::TaskEvent> {
    service
        .manager
        .drain_events()
        .into_iter()
        .map(|event| bridge::TaskEvent {
            task_id: event.task_id,
            kind: match event.kind {
                panta_core::task::TaskEventKind::Started => bridge::TaskEventKind::Started,
                panta_core::task::TaskEventKind::Succeeded => bridge::TaskEventKind::Succeeded,
                panta_core::task::TaskEventKind::Failed => bridge::TaskEventKind::Failed,
                panta_core::task::TaskEventKind::Cancelled => bridge::TaskEventKind::Cancelled,
            },
            code: event.code,
            detail: event.detail,
        })
        .collect()
}

fn task_service_recent_logs(service: &TaskService) -> Vec<bridge::TaskLogLine> {
    service
        .manager
        .recent_logs()
        .into_iter()
        .map(|record| bridge::TaskLogLine {
            task_id: record.task_id,
            message: record.message,
        })
        .collect()
}

fn task_service_running(service: &TaskService) -> u32 {
    u32::try_from(service.manager.running_tasks()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        FfiRequest, FfiResponse, MAX_LABEL_BYTES, bridge, panic_probe, process, session_close,
        session_create, session_label, session_live_count, task_service_drain, task_service_new,
        task_service_recent_logs, task_service_running, task_service_submit,
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

    #[test]
    fn task_service_drains_events_through_bridge() {
        let service = task_service_new();
        let id = match task_service_submit(&service, "桥接-θ".to_owned(), 20, false) {
            Ok(id) => id,
            Err(error) => panic!("submit failed: {error}"),
        };

        // 轮询到终态；等待上界远大于任务时长，避免偶发失败。
        for _ in 0..400 {
            if task_service_running(&service) == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(task_service_running(&service), 0);

        let events = task_service_drain(&service);
        let started = events
            .iter()
            .position(|event| matches!(event.kind, bridge::TaskEventKind::Started));
        let succeeded = events
            .iter()
            .position(|event| matches!(event.kind, bridge::TaskEventKind::Succeeded));
        match (started, succeeded) {
            (Some(started_index), Some(succeeded_index)) => {
                assert!(started_index < succeeded_index)
            }
            _ => panic!(
                "expected started+succeeded, got started={started:?} succeeded={succeeded:?}"
            ),
        }
        assert!(events.iter().all(|event| event.task_id == id));

        let logs = task_service_recent_logs(&service);
        assert!(
            logs.iter()
                .any(|line| line.task_id == id && line.message.contains("succeeded"))
        );

        // 无效输入映射为稳定错误码。
        match task_service_submit(&service, String::new(), 1, false) {
            Ok(_) => panic!("empty label accepted"),
            Err(error) => assert_eq!(error, "task.empty_label"),
        }
        match task_service_submit(&service, "x".to_owned(), 60_001, false) {
            Ok(_) => panic!("oversized duration accepted"),
            Err(error) => assert!(error.starts_with("task.invalid_duration:")),
        }
    }
}
