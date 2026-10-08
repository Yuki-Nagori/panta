//! 任务状态和日志的 CXX 映射；生命周期由 core 任务服务拥有。
use crate::bridge;

/// panta-core 任务管理器的 FFI 包装：与 `Box<TaskService>` 同生命周期，
/// 析构关闭领域状态并请求协作停止，不等待宿主线程；后台不能回调已关闭的宿主。
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
        .map_err(submit_error)
}

fn submit_error(error: panta_core::task::SubmitError) -> String {
    match error {
        panta_core::task::SubmitError::EmptyLabel => "task.empty_label".to_owned(),
        panta_core::task::SubmitError::TooLongDuration(millis) => {
            format!("task.invalid_duration: {millis}")
        }
        panta_core::task::SubmitError::CapacityExceeded => "task.capacity_exhausted".into(),
        panta_core::task::SubmitError::IdentifiersExhausted => "task.identifiers_exhausted".into(),
        panta_core::task::SubmitError::SpawnFailed(detail) => {
            format!("task.spawn_failed: {detail}")
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        bridge, task_service_cancel, task_service_drain, task_service_new,
        task_service_recent_logs, task_service_running, task_service_submit,
    };
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
    fn submission_diagnostics_distinguish_capacity_and_identifier_exhaustion() {
        assert_eq!(
            super::submit_error(panta_core::task::SubmitError::EmptyLabel),
            "task.empty_label"
        );
        assert_eq!(
            super::submit_error(panta_core::task::SubmitError::TooLongDuration(u64::MAX)),
            format!("task.invalid_duration: {}", u64::MAX)
        );
        assert_eq!(
            super::submit_error(panta_core::task::SubmitError::SpawnFailed(
                "线程: denied".into()
            )),
            "task.spawn_failed: 线程: denied"
        );
        assert_eq!(
            super::submit_error(panta_core::task::SubmitError::CapacityExceeded),
            "task.capacity_exhausted"
        );
        assert_eq!(
            super::submit_error(panta_core::task::SubmitError::IdentifiersExhausted),
            "task.identifiers_exhausted"
        );
    }
}
