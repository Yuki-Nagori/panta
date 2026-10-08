//! 最小模拟执行体；任务记录、准入与终态发布由 task 管理。
use super::{Inner, TaskEventKind, cancel_requested, publish_progress, transition};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_millis(10);

pub(super) fn run_task(
    inner: Arc<Inner>,
    task_id: u64,
    label: String,
    duration: Duration,
    fail: bool,
) {
    let started = Instant::now();
    transition(
        &inner,
        task_id,
        TaskEventKind::Started,
        "",
        0,
        label.clone(),
    );

    let mut last_percent: u32 = 0;
    loop {
        if inner.is_shutdown() {
            transition(
                &inner,
                task_id,
                TaskEventKind::Cancelled,
                "task.shutdown",
                0,
                label,
            );
            return;
        }
        if cancel_requested(&inner, task_id) {
            transition(
                &inner,
                task_id,
                TaskEventKind::Cancelled,
                "task.cancelled",
                0,
                label,
            );
            return;
        }
        if started.elapsed() >= duration {
            break;
        }
        thread::sleep(TICK);
        // 进度按 10% 步进发布，控制事件量；只增不减。
        let percent =
            u32::try_from(started.elapsed().as_millis() * 100 / duration.as_millis().max(1))
                .unwrap_or(100)
                .min(100);
        if percent >= last_percent + 10 {
            last_percent = percent;
            publish_progress(&inner, task_id, percent);
        }
    }

    if fail {
        transition(
            &inner,
            task_id,
            TaskEventKind::Failed,
            "task.simulated_failure",
            0,
            format!("{label}: simulated failure"),
        );
    } else {
        transition(&inner, task_id, TaskEventKind::Succeeded, "", 0, label);
    }
}
