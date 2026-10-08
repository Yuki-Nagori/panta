//! 任务状态和日志的 CXX 映射；生命周期由 core 任务服务拥有。
use crate::bridge;

/// panta-core 任务管理器的 FFI 包装：与 `Box<TaskService>` 同生命周期，
/// 析构即关闭并 join 全部工作线程（关闭后不存在可触达的执行体）。
pub struct TaskService {
    manager: panta_core::task::TaskManager,
}

pub(super) fn task_service_new() -> Box<TaskService> {
    Box::new(TaskService {
        manager: panta_core::task::TaskManager::new(),
    })
}

pub(super) fn task_service_submit(
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

pub(super) fn task_service_cancel(service: &TaskService, task_id: u64) -> bool {
    service.manager.cancel(task_id)
}

pub(super) fn task_service_drain(service: &TaskService) -> Vec<bridge::TaskEvent> {
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

pub(super) fn task_service_recent_logs(service: &TaskService) -> Vec<bridge::TaskLogLine> {
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

pub(super) fn task_service_running(service: &TaskService) -> u32 {
    u32::try_from(service.manager.running_tasks()).unwrap_or(u32::MAX)
}
