//! 最小后台任务生命周期（任务 008）：状态机、取消请求、事件队列与
//! 结构化诊断，供几何导入/网格生成等服务复用。
//!
//! 契约（architecture/application-and-storage.md「命令、任务与作业」）：
//! 任务有 ID、状态与错误；开始后使用提交时的输入快照；终态不可回退，
//! 重复/迟到事件不得把任务拉回运行中。事件走拉取队列而非跨线程回调，
//! 由调用方在自有线程/事件循环中消费；工作线程不直接触碰任何 UI 对象。
//!
//! 取消是协作式：工作线程按 `TICK` 粒度检查取消与关闭请求，因此取消
//! 响应与销毁 join 的延迟上界是 `TICK` 加一次状态转换。

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// 单个模拟阶段的最长执行时间；真实长任务应拆成多个可取消阶段，
/// 不放宽此上限。
pub const MAX_TASK_DURATION: Duration = Duration::from_secs(60);
/// 工作线程检查取消/关闭请求的间隔。
const TICK: Duration = Duration::from_millis(10);
/// 结构化日志环容量；超出后丢弃最旧记录。事件队列不受此限制。
const LOG_RING_CAPACITY: usize = 256;

/// 任务事件的种类；顺序即生命周期推进方向，终态只有一种。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskEventKind {
    Started,
    /// 阶段内进度；事件携带 0-100 的 percent。
    Progress,
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskEventKind {
    fn label(&self) -> &'static str {
        match self {
            TaskEventKind::Started => "started",
            TaskEventKind::Progress => "progress",
            TaskEventKind::Succeeded => "succeeded",
            TaskEventKind::Failed => "failed",
            TaskEventKind::Cancelled => "cancelled",
        }
    }

    fn phase(&self) -> Phase {
        match self {
            TaskEventKind::Started | TaskEventKind::Progress => Phase::Running,
            TaskEventKind::Succeeded => Phase::Succeeded,
            TaskEventKind::Failed => Phase::Failed,
            TaskEventKind::Cancelled => Phase::Cancelled,
        }
    }
}

/// 一次状态转换的事件；`code` 为机器可读错误码（仅 Failed 非空），
/// 与用户可读摘要分离；`percent` 仅 Progress 事件有意义（0-100）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEvent {
    pub task_id: u64,
    pub kind: TaskEventKind,
    pub percent: u32,
    pub code: String,
    pub detail: String,
}

/// 结构化日志记录：跨语言与 UI 侧都凭 `task_id` 关联同一任务。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub task_id: u64,
    pub message: String,
}

/// 提交参数校验失败；错误码由 FFI 层映射为稳定字符串。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitError {
    EmptyLabel,
    TooLongDuration(u64),
    SpawnFailed(String),
}

impl std::fmt::Display for SubmitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmitError::EmptyLabel => write!(formatter, "empty label"),
            SubmitError::TooLongDuration(millis) => {
                write!(formatter, "duration {millis} ms exceeds the limit")
            }
            SubmitError::SpawnFailed(detail) => write!(formatter, "spawn failed: {detail}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl Phase {
    fn is_terminal(self) -> bool {
        !matches!(self, Phase::Running)
    }
}

#[derive(Debug)]
struct TaskRecord {
    phase: Phase,
    cancel_requested: bool,
}

#[derive(Debug)]
struct State {
    tasks: HashMap<u64, TaskRecord>,
    events: VecDeque<TaskEvent>,
    logs: VecDeque<LogRecord>,
}

impl State {
    fn log(&mut self, task_id: u64, message: String) {
        self.logs.push_back(LogRecord { task_id, message });
        while self.logs.len() > LOG_RING_CAPACITY {
            self.logs.pop_front();
        }
    }
}

#[derive(Debug)]
struct Inner {
    next_id: AtomicU64,
    shutdown: AtomicBool,
    state: Mutex<State>,
}

impl Inner {
    fn lock_state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }
}

/// 任务生命周期管理器。销毁时通知所有工作线程停止并 join，
/// 之后不再存在可触达的工作线程或回调。
#[derive(Debug)]
pub struct TaskManager {
    inner: Arc<Inner>,
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                next_id: AtomicU64::new(0),
                shutdown: AtomicBool::new(false),
                state: Mutex::new(State {
                    tasks: HashMap::new(),
                    events: VecDeque::new(),
                    logs: VecDeque::new(),
                }),
            }),
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 提交一个模拟慢任务（任务 008 验收用）；真实业务实现将替换
    /// 内部执行体，契约不变。`label` 非空且 `duration` 不超过
    /// [`MAX_TASK_DURATION`]；`fail` 使任务在完成后以结构化错误终止。
    pub fn submit(&self, label: &str, duration: Duration, fail: bool) -> Result<u64, SubmitError> {
        if label.is_empty() {
            return Err(SubmitError::EmptyLabel);
        }
        if duration > MAX_TASK_DURATION {
            return Err(SubmitError::TooLongDuration(duration.as_millis() as u64));
        }

        let task_id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        {
            let mut state = self.inner.lock_state();
            state.tasks.insert(
                task_id,
                TaskRecord {
                    phase: Phase::Running,
                    cancel_requested: false,
                },
            );
            state.log(task_id, format!("submitted: {label}"));
        }

        let inner = Arc::clone(&self.inner);
        let label_owned = label.to_owned();
        let spawn = thread::Builder::new()
            .name(format!("panta-task-{task_id}"))
            .spawn(move || run_task(inner, task_id, label_owned, duration, fail));
        match spawn {
            Ok(handle) => {
                self.lock_handles().push(handle);
                Ok(task_id)
            }
            // 工作线程未启动：回滚记录，任务从未进入生命周期。
            Err(error) => Err(rollback_spawn(&self.inner, task_id, &error)),
        }
    }

    /// 请求取消；仅运行中的任务接受请求并返回 true。终态任务的
    /// 迟到请求被拒绝且不产生事件。
    pub fn cancel(&self, task_id: u64) -> bool {
        let mut state = self.inner.lock_state();
        match state.tasks.get_mut(&task_id) {
            Some(record) if record.phase == Phase::Running && !record.cancel_requested => {
                record.cancel_requested = true;
                state.log(task_id, "cancel requested".to_owned());
                true
            }
            _ => false,
        }
    }

    /// 取走全部已积累事件；终态之后队列为空，重复拉取不产生内容。
    pub fn drain_events(&self) -> Vec<TaskEvent> {
        self.inner.lock_state().events.drain(..).collect()
    }

    /// 最近的结构化日志（容量 [`LOG_RING_CAPACITY`] 环）。
    pub fn recent_logs(&self) -> Vec<LogRecord> {
        self.inner.lock_state().logs.iter().cloned().collect()
    }

    /// 仍在运行的任务数；销毁前归零证明 join 完整。
    pub fn running_tasks(&self) -> usize {
        self.inner
            .lock_state()
            .tasks
            .values()
            .filter(|record| record.phase == Phase::Running)
            .count()
    }

    fn lock_handles(&self) -> MutexGuard<'_, Vec<JoinHandle<()>>> {
        self.handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for TaskManager {
    fn drop(&mut self) {
        // 先置关闭位再 join：工作线程在下一检查点以 task.shutdown 取消
        // 并退出；join 返回后不存在仍存活的工作线程。
        self.inner.shutdown.store(true, Ordering::Relaxed);
        for handle in self.lock_handles().drain(..) {
            let _ = handle.join();
        }
    }
}

fn run_task(inner: Arc<Inner>, task_id: u64, label: String, duration: Duration, fail: bool) {
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

/// 工作线程未启动时的回滚：移除任务记录并写结构化日志。独立成函数，
/// 使线程启动失败路径可不经真实 OS 故障即可测试（任务 032）。
fn rollback_spawn(inner: &Arc<Inner>, task_id: u64, error: &std::io::Error) -> SubmitError {
    let mut state = inner.lock_state();
    state.tasks.remove(&task_id);
    let message = format!("spawn failed: {error}");
    state.log(task_id, message);
    SubmitError::SpawnFailed(error.to_string())
}

fn cancel_requested(inner: &Arc<Inner>, task_id: u64) -> bool {
    let state = inner.lock_state();
    match state.tasks.get(&task_id) {
        Some(record) if record.phase == Phase::Running => record.cancel_requested,
        _ => false,
    }
}

/// 状态转换与事件/日志发布在锁内完成：仅 Running 可迁移，迟到/重复
/// 事件被拒绝，保证终态不可回退。
fn transition(
    inner: &Arc<Inner>,
    task_id: u64,
    kind: TaskEventKind,
    code: &str,
    percent: u32,
    detail: String,
) {
    let mut state = inner.lock_state();
    let Some(record) = state.tasks.get_mut(&task_id) else {
        return;
    };
    if record.phase.is_terminal() {
        return;
    }
    record.phase = kind.phase();
    state.events.push_back(TaskEvent {
        task_id,
        kind: kind.clone(),
        percent,
        code: code.to_owned(),
        detail: detail.clone(),
    });
    let reason = if detail.is_empty() { code } else { &detail };
    state.log(task_id, format!("{}: {reason}", kind.label()));
}

/// 发布运行中进度；不写日志（日志环留给状态转换与诊断）。
fn publish_progress(inner: &Arc<Inner>, task_id: u64, percent: u32) {
    let mut state = inner.lock_state();
    let Some(record) = state.tasks.get_mut(&task_id) else {
        return;
    };
    if record.phase.is_terminal() {
        return;
    }
    state.events.push_back(TaskEvent {
        task_id,
        kind: TaskEventKind::Progress,
        percent,
        code: String::new(),
        detail: String::new(),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        LOG_RING_CAPACITY, LogRecord, MAX_TASK_DURATION, SubmitError, TaskEvent, TaskEventKind,
        TaskManager, publish_progress, rollback_spawn, transition,
    };
    use std::time::{Duration, Instant};

    /// 轮询直到条件成立或超时；超时后返回最后一次条件值供断言。
    fn wait_for(condition: impl Fn() -> bool, timeout: Duration) -> bool {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if condition() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        condition()
    }

    fn submit(manager: &TaskManager, label: &str, millis: u64, fail: bool) -> u64 {
        manager
            .submit(label, Duration::from_millis(millis), fail)
            .unwrap_or_else(|error| panic!("submit '{label}' failed: {error}"))
    }

    #[test]
    fn submit_succeeds_and_emits_ordered_events() {
        let manager = TaskManager::new();
        let id = submit(&manager, "succeed-α", 30, false);

        assert!(wait_for(
            || manager.running_tasks() == 0,
            Duration::from_secs(2)
        ));
        let events = manager.drain_events();
        let kinds: Vec<TaskEventKind> = events
            .iter()
            .filter(|event| event.kind != TaskEventKind::Progress)
            .map(|event| event.kind.clone())
            .collect();
        assert_eq!(
            kinds,
            vec![TaskEventKind::Started, TaskEventKind::Succeeded]
        );
        assert!(
            events
                .iter()
                .all(|event| event.task_id == id && event.code.is_empty())
        );
        // 终态后队列保持为空：重复拉取不产生新事件。
        assert_eq!(manager.drain_events(), Vec::<TaskEvent>::new());

        let logs = manager.recent_logs();
        assert!(logs.iter().any(|record: &LogRecord| {
            record.task_id == id && record.message.contains("submitted")
        }));
        assert!(
            logs.iter()
                .any(|record| record.task_id == id && record.message.contains("succeeded"))
        );
    }

    #[test]
    fn progress_events_are_bounded_and_monotonic() {
        let manager = TaskManager::new();
        let id = submit(&manager, "progress-θ", 200, false);

        assert!(wait_for(
            || manager.running_tasks() == 0,
            Duration::from_secs(2)
        ));
        let percents: Vec<u32> = manager
            .drain_events()
            .into_iter()
            .filter(|event| event.task_id == id && event.kind == TaskEventKind::Progress)
            .map(|event| event.percent)
            .collect();
        // 10% 步进：事件量有界、单调递增且不超过 100。
        assert!(!percents.is_empty(), "no progress events");
        assert!(
            percents.len() <= 10,
            "too many progress events: {percents:?}"
        );
        for pair in percents.windows(2) {
            assert!(pair[0] < pair[1], "progress not increasing: {percents:?}");
        }
        assert!(*percents.last().unwrap_or(&0) <= 100);
    }

    #[test]
    fn simulated_failure_is_a_structured_error() {
        let manager = TaskManager::new();
        let id = submit(&manager, "fail-β", 20, true);

        assert!(wait_for(
            || manager.running_tasks() == 0,
            Duration::from_secs(2)
        ));
        let events = manager.drain_events();
        let mut failed = None;
        for event in &events {
            if event.task_id == id && event.kind == TaskEventKind::Failed {
                failed = Some(event);
            }
        }
        let failed = failed.unwrap_or_else(|| panic!("failed event missing: {events:?}"));
        assert_eq!(failed.code, "task.simulated_failure");
        assert!(failed.detail.contains("fail-β"));
    }

    #[test]
    fn cancel_running_task_and_reject_duplicate_request() {
        let manager = TaskManager::new();
        let id = submit(&manager, "cancel-γ", 30_000, false);

        assert!(manager.cancel(id));
        // 取消请求只接受一次；重复请求不产生第二个事件。
        assert!(!manager.cancel(id));
        assert!(wait_for(
            || manager.running_tasks() == 0,
            Duration::from_secs(5)
        ));

        let events = manager.drain_events();
        let mut cancelled = None;
        for event in &events {
            if event.kind == TaskEventKind::Cancelled {
                cancelled = Some(event);
            }
        }
        let cancelled = cancelled.unwrap_or_else(|| panic!("cancelled event missing: {events:?}"));
        assert_eq!(cancelled.code, "task.cancelled");
        assert!(
            !events.iter().any(|event| matches!(
                event.kind,
                TaskEventKind::Succeeded | TaskEventKind::Failed
            ))
        );
    }

    #[test]
    fn late_cancel_is_rejected_and_terminal_holds() {
        let manager = TaskManager::new();
        let id = submit(&manager, "late-δ", 20, false);

        assert!(wait_for(
            || manager.running_tasks() == 0,
            Duration::from_secs(2)
        ));
        assert!(!manager.cancel(id));
        let events = manager.drain_events();
        assert!(
            !events
                .iter()
                .any(|event| event.kind == TaskEventKind::Cancelled)
        );
        assert_eq!(manager.drain_events(), Vec::<TaskEvent>::new());
    }

    #[test]
    fn submit_rejects_invalid_input() {
        let manager = TaskManager::new();
        assert_eq!(
            manager.submit("", Duration::from_millis(1), false),
            Err(SubmitError::EmptyLabel)
        );
        assert_eq!(
            manager.submit(
                "long-ε",
                MAX_TASK_DURATION + Duration::from_millis(1),
                false
            ),
            Err(SubmitError::TooLongDuration(
                (MAX_TASK_DURATION.as_millis() + 1) as u64
            ))
        );
        assert_eq!(manager.running_tasks(), 0);
    }

    #[test]
    fn drop_joins_running_workers() {
        let started = Instant::now();
        {
            let manager = TaskManager::new();
            submit(&manager, "shutdown-ζ", 30_000, false);
            submit(&manager, "shutdown-η", 30_000, false);
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_secs(30),
            "drop 阻塞 {elapsed:?}：工作线程未被 join"
        );
    }

    #[test]
    fn submit_error_display_covers_all_variants() {
        assert_eq!(SubmitError::EmptyLabel.to_string(), "empty label");
        assert_eq!(
            SubmitError::TooLongDuration(MAX_TASK_DURATION.as_millis() as u64).to_string(),
            format!(
                "duration {} ms exceeds the limit",
                MAX_TASK_DURATION.as_millis() as u64
            )
        );
        assert_eq!(
            SubmitError::SpawnFailed("boom".to_owned()).to_string(),
            "spawn failed: boom"
        );
    }

    #[test]
    fn task_event_kind_labels_cover_all_variants() {
        let labels: Vec<&'static str> = [
            TaskEventKind::Started,
            TaskEventKind::Progress,
            TaskEventKind::Succeeded,
            TaskEventKind::Failed,
            TaskEventKind::Cancelled,
        ]
        .iter()
        .map(|kind| kind.label())
        .collect();
        assert_eq!(
            labels,
            vec!["started", "progress", "succeeded", "failed", "cancelled"]
        );
    }

    #[test]
    fn default_manager_is_usable() -> Result<(), String> {
        let manager = TaskManager::default();
        let id = manager
            .submit("默认构造", std::time::Duration::from_millis(1), false)
            .map_err(|error| error.to_string())?;
        wait_until_running_zero(&manager);
        assert!(
            manager
                .drain_events()
                .iter()
                .any(|event| event.task_id == id),
            "默认构造的管理器应有事件"
        );
        Ok(())
    }

    #[test]
    fn log_ring_capacity_evicts_oldest() -> Result<(), String> {
        let manager = TaskManager::new();
        // 提交超过环容量的任务数：每条提交至少写一条日志，最旧者被淘汰。
        for _ in 0..(LOG_RING_CAPACITY + 16) {
            manager
                .submit("环容量", std::time::Duration::from_millis(1), false)
                .map_err(|error| error.to_string())?;
        }
        wait_until_running_zero(&manager);
        let logs = manager.recent_logs();
        assert_eq!(logs.len(), LOG_RING_CAPACITY, "日志环应淘汰最旧记录");
        Ok(())
    }

    #[test]
    fn cancel_rejects_unknown_and_terminal_tasks() -> Result<(), String> {
        let manager = TaskManager::new();
        assert!(!manager.cancel(9_999), "未知任务不可取消");
        let id = manager
            .submit("终态", std::time::Duration::from_millis(1), false)
            .map_err(|error| error.to_string())?;
        wait_until_running_zero(&manager);
        assert!(!manager.cancel(id), "终态任务不可取消");
        Ok(())
    }

    #[test]
    fn late_events_for_missing_or_terminal_tasks_are_rejected() -> Result<(), String> {
        let manager = TaskManager::new();
        let inner = manager.inner.clone();
        let id = manager
            .submit("晚到", std::time::Duration::from_millis(1), false)
            .map_err(|error| error.to_string())?;
        wait_until_running_zero(&manager);
        manager.drain_events();

        // 未知任务：记录缺失，直接返回。
        publish_progress(&inner, 9_999, 50);
        transition(&inner, 9_999, TaskEventKind::Failed, "x", 0, String::new());
        // 终态任务：迟到位事件不得改写终态或追加事件。
        publish_progress(&inner, id, 50);
        transition(&inner, id, TaskEventKind::Succeeded, "", 100, String::new());

        assert!(manager.drain_events().is_empty(), "迟到位事件不得产生事件");
        Ok(())
    }

    #[test]
    fn spawn_rollback_removes_record_and_logs() -> Result<(), String> {
        let manager = TaskManager::new();
        let id = manager
            .submit("回滚", std::time::Duration::from_millis(50), false)
            .map_err(|error| error.to_string())?;
        let error = std::io::Error::other("boom");
        let submit_error = rollback_spawn(&manager.inner, id, &error);
        assert!(matches!(
            submit_error,
            SubmitError::SpawnFailed(ref detail) if detail.contains("boom")
        ));
        assert!(!manager.cancel(id), "回滚后的任务不可取消");
        assert!(
            manager
                .recent_logs()
                .iter()
                .any(|record| record.task_id == id && record.message.contains("spawn failed")),
            "回滚必须写结构化日志"
        );
        Ok(())
    }

    #[test]
    fn wait_until_returns_condition_outcome() {
        assert!(wait_for(|| true, std::time::Duration::from_millis(10)));
        assert!(!wait_for(|| false, std::time::Duration::from_millis(20)));
    }

    fn wait_until_running_zero(manager: &TaskManager) {
        assert!(wait_for(
            || manager.running_tasks() == 0,
            std::time::Duration::from_millis(2_000)
        ));
    }
}
