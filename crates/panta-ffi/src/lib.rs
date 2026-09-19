//! 最小 CXX 双向边界：验证 DTO、opaque 句柄所有权、错误转换、C++ 实现
//! 调用，以及 Rust 拥有的任务生命周期（任务 008）与路径服务（任务 023）。

use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_REPEAT: u32 = 8;
/// 句柄标签按 UTF-8 字节数设界，与请求文本的有界策略一致。
const MAX_LABEL_BYTES: usize = 64;

// unsafe 由两部分组成：CXX 桥接宏生成的胶水（边界安全前提由 cxx 运行时
// 的类型检查与 ffi.hpp 签名一致性承担），以及 crash 模块（任务 047）的
// 信号处理 FFI（安全前提逐块注明）。本 crate 对外只暴露安全签名。
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
        Progress = 1,
        Succeeded = 2,
        Failed = 3,
        Cancelled = 4,
    }

    /// 一次状态转换的事件；`code` 为机器可读错误码（仅 Failed 非空），
    /// `detail` 为诊断上下文，面向日志/Console 而非用户摘要；
    /// `percent` 仅 Progress 事件有意义（0-100）。
    /// CXX 共享枚举不支持自定义 derive，因此本结构不派生 Debug/Clone。
    pub struct TaskEvent {
        pub task_id: u64,
        pub kind: TaskEventKind,
        pub percent: u32,
        pub code: String,
        pub detail: String,
    }

    /// 结构化日志行：跨语言与 UI 侧凭 `task_id` 关联同一任务。
    pub struct TaskLogLine {
        pub task_id: u64,
        pub message: String,
    }

    /// 逻辑根类别（任务 023），与 panta_core::path::RootCategory 一一对应；
    /// qrc 只读、无本机根。跨语言按数值传递。
    pub enum PathRootKind {
        Project = 0,
        UserConfig = 1,
        AppData = 2,
        Cache = 3,
        Session = 4,
        Qrc = 5,
    }

    /// 结构化逻辑资源引用；字符串形态 `scheme:/relative`（`path_ref_parse`
    /// /`path_ref_to_logical` 互逆）。相对片段只跨边界传可往返 UTF-8。
    /// CXX 共享枚举不支持自定义 derive，与 TaskEvent 同规则不派生 Debug。
    #[derive(Clone, PartialEq, Eq)]
    pub struct PathRef {
        pub kind: PathRootKind,
        pub relative: String,
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

        /// 路径服务（panta_core::path::PathService 的包装）：根由宿主注入，
        /// 解析与 cwd 无关；错误以 `path.*: detail` 文本传递，UI 只按前缀分支。
        type PathService;

        fn path_service_new() -> Box<PathService>;
        fn path_service_set_root(
            service: &mut PathService,
            kind: PathRootKind,
            utf8_root: String,
        ) -> Result<()>;
        fn path_ref_parse(reference: String) -> Result<PathRef>;
        fn path_ref_to_logical(reference: &PathRef) -> Result<String>;
        /// 纯逻辑解析：结构校验 + 根拼接，不访问文件系统。
        fn path_service_resolve(service: &PathService, reference: &PathRef) -> Result<String>;
        /// 读解析：目标必须存在，规范化后仍在根内（拒绝符号链接越界）。
        fn path_service_resolve_existing(
            service: &PathService,
            reference: &PathRef,
        ) -> Result<String>;
        /// 写目标解析：目标可不存在，最深现存祖先仍需在根内。
        fn path_service_resolve_write_target(
            service: &PathService,
            reference: &PathRef,
        ) -> Result<String>;

        /// 崩溃信号处理器安装（任务 047，panta_core::crash 的 FFI 面）：
        /// 返回日志路径；log_dir 为空时用系统临时目录。
        fn install_crash_handler(log_dir: &str) -> Result<String>;
    }
}

// 专用手写 unsafe 边界（任务 047，rust.md）：崩溃信号处理的安全前提
// 逐块注明；领域模型（panta-core）保持无 unsafe。
#[allow(unsafe_code)]
pub mod crash;

pub use bridge::{FfiRequest, FfiResponse};

/// 任务 047：错误以文本跨边界（C++ 侧 qWarning 呈现，不静默）。
fn install_crash_handler(log_dir: &str) -> Result<String, String> {
    crate::crash::install_crash_handler(log_dir)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

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
                panta_core::task::TaskEventKind::Progress => bridge::TaskEventKind::Progress,
                panta_core::task::TaskEventKind::Succeeded => bridge::TaskEventKind::Succeeded,
                panta_core::task::TaskEventKind::Failed => bridge::TaskEventKind::Failed,
                panta_core::task::TaskEventKind::Cancelled => bridge::TaskEventKind::Cancelled,
            },
            percent: event.percent,
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

/// 路径服务的 FFI 包装（任务 023）：只做枚举/DTO 与错误文本映射，
/// 规则与包含检查全部在 panta-core 实现，不在边界复制。
pub struct PathService {
    service: panta_core::path::PathService,
}

fn path_service_new() -> Box<PathService> {
    Box::new(PathService {
        service: panta_core::path::PathService::new(),
    })
}

fn path_service_set_root(
    service: &mut PathService,
    kind: bridge::PathRootKind,
    utf8_root: String,
) -> Result<(), String> {
    let category = core_category(kind)?;
    service
        .service
        .set_root(category, std::path::Path::new(&utf8_root))
        .map_err(|error| error.to_string())
}

fn path_ref_parse(reference: String) -> Result<bridge::PathRef, String> {
    let parsed = panta_core::path::ResourceRef::parse(&reference)?;
    Ok(bridge::PathRef {
        kind: bridge_kind(parsed.category),
        relative: parsed.relative,
    })
}

fn path_ref_to_logical(reference: &bridge::PathRef) -> Result<String, String> {
    let category = core_category(reference.kind)?;
    Ok(panta_core::path::ResourceRef {
        category,
        relative: reference.relative.clone(),
    }
    .to_logical())
}

fn path_service_resolve(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(service, reference, panta_core::path::PathService::resolve)
}

fn path_service_resolve_existing(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(
        service,
        reference,
        panta_core::path::PathService::resolve_existing,
    )
}

fn path_service_resolve_write_target(
    service: &PathService,
    reference: &bridge::PathRef,
) -> Result<String, String> {
    resolve_with(
        service,
        reference,
        panta_core::path::PathService::resolve_write_target,
    )
}

fn resolve_with(
    service: &PathService,
    reference: &bridge::PathRef,
    resolver: fn(
        &panta_core::path::PathService,
        &panta_core::path::ResourceRef,
    ) -> Result<std::path::PathBuf, panta_core::path::PathError>,
) -> Result<String, String> {
    let core_ref = panta_core::path::ResourceRef {
        category: core_category(reference.kind)?,
        relative: reference.relative.clone(),
    };
    resolver(&service.service, &core_ref)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

/// CXX 枚举跨边界可能携带越界表示；未知取值按稳定错误码拒绝，不猜测。
fn core_category(kind: bridge::PathRootKind) -> Result<panta_core::path::RootCategory, String> {
    match kind {
        bridge::PathRootKind::Project => Ok(panta_core::path::RootCategory::Project),
        bridge::PathRootKind::UserConfig => Ok(panta_core::path::RootCategory::UserConfig),
        bridge::PathRootKind::AppData => Ok(panta_core::path::RootCategory::AppData),
        bridge::PathRootKind::Cache => Ok(panta_core::path::RootCategory::Cache),
        bridge::PathRootKind::Session => Ok(panta_core::path::RootCategory::Session),
        bridge::PathRootKind::Qrc => Ok(panta_core::path::RootCategory::Qrc),
        _ => Err("path.invalid_kind".to_owned()),
    }
}

fn bridge_kind(category: panta_core::path::RootCategory) -> bridge::PathRootKind {
    match category {
        panta_core::path::RootCategory::Project => bridge::PathRootKind::Project,
        panta_core::path::RootCategory::UserConfig => bridge::PathRootKind::UserConfig,
        panta_core::path::RootCategory::AppData => bridge::PathRootKind::AppData,
        panta_core::path::RootCategory::Cache => bridge::PathRootKind::Cache,
        panta_core::path::RootCategory::Session => bridge::PathRootKind::Session,
        panta_core::path::RootCategory::Qrc => bridge::PathRootKind::Qrc,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FfiRequest, FfiResponse, MAX_LABEL_BYTES, bridge, panic_probe, path_ref_parse,
        path_service_new, path_service_resolve, path_service_resolve_existing,
        path_service_resolve_write_target, path_service_set_root, process, session_close,
        session_create, session_label, session_live_count, task_service_cancel, task_service_drain,
        task_service_new, task_service_recent_logs, task_service_running, task_service_submit,
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
    fn path_service_round_trips_references_and_rejects_escapes()
    -> Result<(), Box<dyn std::error::Error>> {
        let parsed = crate::path_ref_parse("project:/资产 齿轮/a.step".to_owned())?;
        assert!(matches!(parsed.kind, bridge::PathRootKind::Project));
        assert_eq!(parsed.relative, "资产 齿轮/a.step");
        let logical = match crate::path_ref_to_logical(&parsed) {
            Ok(logical) => logical,
            Err(error) => panic!("生成逻辑地址失败: {error}"),
        };
        assert_eq!(logical, "project:/资产 齿轮/a.step");
        match crate::path_ref_parse("workspace:/x".to_owned()) {
            Ok(parsed) => panic!("未知 scheme 被接受: {:?}", parsed.relative),
            Err(error) => assert_eq!(error, "path.unknown_scheme: workspace"),
        }

        let base =
            std::fs::canonicalize(std::env::temp_dir()).unwrap_or_else(|_| std::env::temp_dir());
        let root = base.join(format!("panta-ffi-path-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?; // mkdir 失败: {error}
        let mut service = crate::path_service_new();
        if let Err(error) = crate::path_service_set_root(
            &mut service,
            bridge::PathRootKind::Project,
            root.display().to_string(),
        ) {
            panic!("注入根失败: {error}");
        }
        let resolved = match crate::path_service_resolve(
            &service,
            &bridge::PathRef {
                kind: bridge::PathRootKind::Project,
                relative: "out/../a.pa".to_owned(),
            },
        ) {
            Ok(path) => path,
            Err(error) => panic!("合法引用被拒绝: {error}"),
        };
        assert_eq!(resolved, root.join("a.pa").display().to_string());

        for (relative, code) in [
            ("../escape", "path.parent_escape"),
            ("C:/win", "path.absolute_rejected"),
            ("COM1", "path.reserved_name"),
        ] {
            let error = match crate::path_service_resolve(
                &service,
                &bridge::PathRef {
                    kind: bridge::PathRootKind::Project,
                    relative: relative.to_owned(),
                },
            ) {
                Ok(path) => panic!("{relative} 意外通过: {path}"),
                Err(error) => error,
            };
            assert!(error.starts_with(code), "{relative} -> {error}");
        }
        match crate::path_service_resolve(
            &service,
            &bridge::PathRef {
                kind: bridge::PathRootKind::Qrc,
                relative: "icons/x.svg".to_owned(),
            },
        ) {
            Ok(path) => panic!("qrc 被解析为本机路径: {path}"),
            Err(error) => assert_eq!(error, "path.qrc_not_native"),
        }
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn task_service_drains_events_through_bridge() -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(())
    }

    #[test]
    fn task_service_lifecycle_maps_cancel_failed_and_cancelled_events()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = task_service_new();
        // 失败任务:Failed 事件经桥接枚举映射。
        let failed_id = task_service_submit(&service, "失败任务".to_owned(), 10, true)?;
        for _ in 0..400 {
            if task_service_running(&service) == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let events = task_service_drain(&service);
        assert!(
            events.iter().any(|event| {
                event.task_id == failed_id
                    && matches!(event.kind, bridge::TaskEventKind::Failed)
                    && !event.code.is_empty()
            }),
            "失败任务必须携带 Failed 事件与错误码"
        );
        // 取消长任务:Cancelled 事件经桥接枚举映射;cancel 返回 true。
        let long_id = task_service_submit(&service, "长任务".to_owned(), 5_000, false)?;
        assert!(
            task_service_cancel(&service, long_id),
            "运行中任务必须可取消"
        );
        for _ in 0..400 {
            if task_service_running(&service) == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let events = task_service_drain(&service);
        assert!(
            events.iter().any(|event| {
                event.task_id == long_id && matches!(event.kind, bridge::TaskEventKind::Cancelled)
            }),
            "取消必须产生 Cancelled 事件"
        );
        Ok(())
    }

    #[test]
    fn path_service_covers_existing_write_target_and_all_kinds()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::path::Path;

        let base = std::fs::canonicalize(std::env::temp_dir())?;
        let root = base.join(format!("panta-ffi-path-kinds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        let mut service = path_service_new();

        // 全类别注入(UserConfig/AppData/Cache/Session 各指临时子目录):
        // core_category 与 set_root 的全部分支被真实执行。
        for (kind, name) in [
            (bridge::PathRootKind::UserConfig, "user-config"),
            (bridge::PathRootKind::AppData, "app-data"),
            (bridge::PathRootKind::Cache, "cache"),
            (bridge::PathRootKind::Session, "session"),
        ] {
            let directory = root.join(name);
            std::fs::create_dir_all(&directory)?;
            path_service_set_root(&mut service, kind, directory.display().to_string())?;
        }

        // 每个类别都可纯逻辑解析,且 scheme 往返经 bridge_kind 全分支。
        for scheme in ["user-config", "app-data", "cache", "session"] {
            let parsed = path_ref_parse(format!("{scheme}:/配置/x.pa"))?;
            let resolved = path_service_resolve(&service, &parsed)?;
            assert!(resolved.contains(scheme), "{scheme} -> {resolved}");
        }

        // 写目标(未创建)与读解析(现存)在 cache 类别走通全链。
        let reference = bridge::PathRef {
            kind: bridge::PathRootKind::Cache,
            relative: "out/新 口袋/pocket.pa".to_owned(),
        };
        let target = path_service_resolve_write_target(&service, &reference)?;
        let parent = Path::new(&target)
            .parent()
            .ok_or_else(|| format!("目标缺少父目录: {target}"))?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&target, b"pa")?;
        path_service_resolve_existing(&service, &reference)?;
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }
}
