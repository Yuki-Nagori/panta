//! 只读 STL 资产激活（073 首个 FSM 消费者；080 视口文档页签的数据源）。
//!
//! 职责分界：`fsm/open-saved-stl.pa` 经生成元数据声明状态图；本文件实现
//! 事件决策（guard）、工作线程、取消与结果相关性。结果只有在工程记录与运
//! 行期会话仍有效时才发布；过期快照在 worker 或 drain 中就地释放，不跨
//! FFI。加载不修改工程 revision / dirty，也不写存储。

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, PoisonError};

use panta_import::{CheckedReadError, ImportError, parse_stl_asset, read_source_checked};
use panta_mesh::SurfaceMesh;

use super::generated as fsm;

/// begin 返回的运行期相关性句柄；generation 是会话级计数，非工程 revision。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationAttempt {
    pub attempt: u64,
    pub generation: u64,
}

/// 一次激活请求的工作数据；提交后归 worker 线程所有。
pub(crate) struct ActivationRequest {
    pub asset_path: PathBuf,
    pub units: String,
}

/// 每个进程内 attempt 至多发布一次的终态结果；`Expired` 无网格负载。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeKind {
    Succeeded,
    Failed,
    Cancelled,
    Expired,
}

/// 穿越所有权的终态结果：`mesh` 由接收方释放，drain 过滤的过期快照就地释放。
pub struct Outcome {
    pub attempt: u64,
    pub generation: u64,
    pub import_id: String,
    pub kind: OutcomeKind,
    pub code: String,
    pub detail: String,
    pub mesh: Option<SurfaceMesh>,
}

/// 手写事件：携带 payload，穷尽映射到生成的 `EventKind`。
pub(crate) enum Event {
    OpenRequested,
    AssetRead,
    ParseSucceeded { mesh: SurfaceMesh },
    Fail { code: &'static str, detail: String },
    CancelAcknowledged,
    GenerationInvalidated,
}

impl Event {
    fn kind(&self) -> fsm::EventKind {
        match self {
            Self::OpenRequested => fsm::EventKind::OpenRequested,
            Self::AssetRead => fsm::EventKind::AssetRead,
            Self::ParseSucceeded { .. } => fsm::EventKind::ParseSucceeded,
            Self::Fail { .. } => fsm::EventKind::Fail,
            Self::CancelAcknowledged => fsm::EventKind::CancelAcknowledged,
            Self::GenerationInvalidated => fsm::EventKind::GenerationInvalidated,
        }
    }
}

/// guard 上下文：当前一致快照上的纯谓词输入。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GuardContext {
    record_valid: bool,
    session_current: bool,
}

/// 手写决策结果；guard 为假表示拒绝事件（原状态不变、不执行动作）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Enter(fsm::State),
    GuardRejected,
    Illegal,
}

/// 手写 guard 求值：对生成的 `Guard` 穷尽匹配，不用默认真分支。
fn evaluate(guard: fsm::Guard, context: &GuardContext) -> bool {
    match guard {
        fsm::Guard::RecordValid => context.record_valid,
        fsm::Guard::SessionCurrent => context.session_current,
    }
}

/// 手写转移决策：状态图运行期的唯一实现，与生成 `TRANSITIONS` 的一致性由
/// `generated_transitions_match_handwritten_decisions` 双向结构测试保证。
fn decide(state: fsm::State, event: fsm::EventKind, guards: &GuardContext) -> Decision {
    match (state, event) {
        (fsm::State::Idle, fsm::EventKind::OpenRequested)
            if evaluate(fsm::Guard::RecordValid, guards) =>
        {
            Decision::Enter(fsm::State::LoadingAsset)
        }
        (fsm::State::Idle, fsm::EventKind::OpenRequested) => Decision::GuardRejected,
        (fsm::State::Idle, fsm::EventKind::Fail) => Decision::Enter(fsm::State::Failed),
        (fsm::State::Idle, fsm::EventKind::CancelAcknowledged) => {
            Decision::Enter(fsm::State::Cancelled)
        }
        (fsm::State::LoadingAsset, fsm::EventKind::AssetRead) => {
            Decision::Enter(fsm::State::Parsing)
        }
        (fsm::State::LoadingAsset, fsm::EventKind::Fail) => Decision::Enter(fsm::State::Failed),
        (fsm::State::LoadingAsset, fsm::EventKind::CancelAcknowledged) => {
            Decision::Enter(fsm::State::Cancelled)
        }
        (fsm::State::LoadingAsset, fsm::EventKind::GenerationInvalidated) => {
            Decision::Enter(fsm::State::Expired)
        }
        (fsm::State::Parsing, fsm::EventKind::ParseSucceeded)
            if evaluate(fsm::Guard::SessionCurrent, guards) =>
        {
            Decision::Enter(fsm::State::Ready)
        }
        (fsm::State::Parsing, fsm::EventKind::ParseSucceeded) => Decision::GuardRejected,
        (fsm::State::Parsing, fsm::EventKind::Fail) => Decision::Enter(fsm::State::Failed),
        (fsm::State::Parsing, fsm::EventKind::CancelAcknowledged) => {
            Decision::Enter(fsm::State::Cancelled)
        }
        (fsm::State::Parsing, fsm::EventKind::GenerationInvalidated) => {
            Decision::Enter(fsm::State::Expired)
        }
        _ => Decision::Illegal,
    }
}

/// dispatch 的终态产出；payload 在未消费时随事件就地释放。
enum DispatchOutcome {
    Entered,
    GuardRejected,
    Illegal,
}

struct AttemptEntry {
    state: fsm::State,
    generation: u64,
    import_id: String,
    cancel_requested: Arc<AtomicBool>,
}

#[derive(Default)]
struct CoordinatorState {
    next_attempt: u64,
    attempts: HashMap<u64, AttemptEntry>,
    outcomes: VecDeque<Outcome>,
}

/// 激活协调器：attempt 状态、取消标志与结果队列的唯一所有者。
/// ProjectService（FFI/GUI 线程）与 worker 线程共享 `Arc<Self>`；所有
/// attempt 状态转移都在入口锁内串行，worker 与工程切换不会交错写状态。
#[derive(Default)]
pub struct ActivationCoordinator {
    state: Mutex<CoordinatorState>,
    generation: Arc<AtomicU64>,
}

impl std::fmt::Debug for ActivationCoordinator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 不展开 attempt 与结果队列（含网格负载），只暴露会话代次。
        formatter
            .debug_struct("ActivationCoordinator")
            .field("generation", &self.generation.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl ActivationCoordinator {
    fn lock(&self) -> std::sync::MutexGuard<'_, CoordinatorState> {
        // 锁中毒只可能来自 panic 传播；恢复内层数据继续串行语义。
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 提交一次激活。同会话同记录的在飞 attempt 直接复用（去重）；
    /// `record-valid` guard 已由调用方查证记录存在，仍经决策函数进入
    /// `LoadingAsset`。worker 启动失败走声明的 `spawn-failed` 边。
    pub(crate) fn begin(
        self: &Arc<Self>,
        import_id: &str,
        request: ActivationRequest,
    ) -> ActivationAttempt {
        self.begin_with(import_id, request, spawn_activation_worker)
    }

    /// begin 的可注入入口：spawner 可替换以覆盖 worker 启动失败。
    pub(crate) fn begin_with(
        self: &Arc<Self>,
        import_id: &str,
        request: ActivationRequest,
        spawn: impl FnOnce(Arc<Self>, u64, ActivationRequest, Arc<AtomicBool>) -> std::io::Result<()>,
    ) -> ActivationAttempt {
        let mut state = self.lock();
        let generation = self.generation.load(Ordering::SeqCst);
        let existing = state.attempts.iter().find_map(|(attempt, entry)| {
            (entry.generation == generation && entry.import_id == import_id).then_some(*attempt)
        });
        if let Some(attempt) = existing {
            return ActivationAttempt {
                attempt,
                generation,
            };
        }
        state.next_attempt = state.next_attempt.wrapping_add(1);
        let attempt = state.next_attempt;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        state.attempts.insert(
            attempt,
            AttemptEntry {
                state: fsm::INITIAL_STATE,
                generation,
                import_id: import_id.to_owned(),
                cancel_requested: Arc::clone(&cancel_requested),
            },
        );
        // 提交边界已确认记录存在：record_valid 恒真，GuardRejected/Illegal
        // 在契约内不可达；兜底收敛为启动失败终态，不留悬挂 attempt。
        if let Some(detail) = match dispatch(
            &mut state,
            &self.generation,
            attempt,
            Event::OpenRequested,
            true,
        ) {
            DispatchOutcome::Entered => None,
            DispatchOutcome::GuardRejected => Some("open request rejected at admission".to_owned()),
            DispatchOutcome::Illegal => Some("attempt vanished after admission".to_owned()),
        } {
            dispatch_admission_failure(&mut state, &self.generation, attempt, detail);
        }
        if let Err(error) = spawn(Arc::clone(self), attempt, request, cancel_requested) {
            dispatch_admission_failure(&mut state, &self.generation, attempt, error.to_string());
        }
        ActivationAttempt {
            attempt,
            generation,
        }
    }

    /// 请求取消：只置位标志；`Cancelled` 终态由 worker 在安全检查点确认。
    pub(crate) fn cancel(&self, attempt: u64) -> bool {
        let state = self.lock();
        match state.attempts.get(&attempt) {
            Some(entry) => {
                entry.cancel_requested.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    /// 推进会话代次并使所有旧代次在飞 attempt 失效（create/open 调用）。
    pub(crate) fn advance_generation(&self) -> u64 {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let mut state = self.lock();
        let stale: Vec<u64> = state
            .attempts
            .iter()
            .filter(|(_, entry)| entry.generation != generation)
            .map(|(attempt, _)| *attempt)
            .collect();
        for attempt in stale {
            dispatch(
                &mut state,
                &self.generation,
                attempt,
                Event::GenerationInvalidated,
                false,
            );
        }
        generation
    }

    /// 取出当前代次的终态结果；旧代次残留结果（含快照）就地释放，不跨 FFI。
    pub(crate) fn drain(&self) -> Vec<Outcome> {
        let generation = self.generation.load(Ordering::SeqCst);
        let mut state = self.lock();
        let mut results = Vec::new();
        for outcome in state.outcomes.drain(..) {
            if outcome.generation == generation {
                results.push(outcome);
            }
        }
        results
    }

    /// worker 完成事件入口：guard 在锁内按当前代次求值。
    pub(crate) fn complete(&self, attempt: u64, event: Event) {
        let mut state = self.lock();
        dispatch(&mut state, &self.generation, attempt, event, false);
    }
}

/// 准入或 worker 启动失败的兜底终态：走声明的 `spawn-failed` 边发布一次
/// `Failed` 结果，调用方据此把对应页签置为失败态。
fn dispatch_admission_failure(
    state: &mut CoordinatorState,
    generation: &AtomicU64,
    attempt: u64,
    detail: String,
) {
    dispatch(
        state,
        generation,
        attempt,
        Event::Fail {
            code: "project.activation_spawn_failed",
            detail,
        },
        false,
    );
}

/// 单次事件的状态转移与终态发布；事件 payload 在未消费分支自动释放。
fn dispatch(
    state: &mut CoordinatorState,
    generation: &AtomicU64,
    attempt: u64,
    event: Event,
    record_valid: bool,
) -> DispatchOutcome {
    let Some(entry) = state.attempts.get(&attempt) else {
        return DispatchOutcome::Illegal;
    };
    let guards = GuardContext {
        record_valid,
        session_current: generation.load(Ordering::SeqCst) == entry.generation,
    };
    let decision = decide(entry.state, event.kind(), &guards);
    match decision {
        Decision::Enter(next) => {
            if let Some(entry) = state.attempts.get_mut(&attempt) {
                entry.state = next;
            }
            if fsm::TERMINAL_STATES.contains(&next)
                && let Some(entry) = state.attempts.remove(&attempt)
            {
                state
                    .outcomes
                    .push_back(outcome_for(&entry, next, attempt, event));
            }
            DispatchOutcome::Entered
        }
        Decision::GuardRejected => DispatchOutcome::GuardRejected,
        Decision::Illegal => DispatchOutcome::Illegal,
    }
}

fn outcome_for(entry: &AttemptEntry, state: fsm::State, attempt: u64, event: Event) -> Outcome {
    let mut outcome = Outcome {
        attempt,
        generation: entry.generation,
        import_id: entry.import_id.clone(),
        kind: OutcomeKind::Expired,
        code: String::new(),
        detail: String::new(),
        mesh: None,
    };
    match (state, event) {
        (fsm::State::Ready, Event::ParseSucceeded { mesh }) => {
            outcome.kind = OutcomeKind::Succeeded;
            outcome.mesh = Some(mesh);
        }
        (fsm::State::Failed, Event::Fail { code, detail }) => {
            outcome.kind = OutcomeKind::Failed;
            outcome.code = code.to_owned();
            outcome.detail = detail;
        }
        (fsm::State::Cancelled, _) => outcome.kind = OutcomeKind::Cancelled,
        (fsm::State::Expired, _) => outcome.kind = OutcomeKind::Expired,
        // decide() 保证终态只由配对事件进入；其余组合不可达。
        _ => unreachable!("terminal state reached with mismatched event"),
    }
    outcome
}

/// worker 入口：分块读取（可取消）→ 解析（不可中断，完成边界校验代次）。
/// GUI 线程永不 join 本线程；终止只经由状态图终态发布结果。
fn run_attempt(
    coordinator: Arc<ActivationCoordinator>,
    attempt: u64,
    request: ActivationRequest,
    cancel_requested: Arc<AtomicBool>,
) {
    let cancelled = || cancel_requested.load(Ordering::SeqCst);
    if cancelled() {
        coordinator.complete(attempt, Event::CancelAcknowledged);
        return;
    }
    let snapshot = match read_source_checked(&request.asset_path, cancelled) {
        Ok(snapshot) => snapshot,
        Err(CheckedReadError::Cancelled(_)) => {
            coordinator.complete(attempt, Event::CancelAcknowledged);
            return;
        }
        Err(CheckedReadError::Failed(error)) => {
            let (code, detail) = activation_failure(&error);
            coordinator.complete(attempt, Event::Fail { code, detail });
            return;
        }
    };
    coordinator.complete(attempt, Event::AssetRead);
    if cancelled() {
        coordinator.complete(attempt, Event::CancelAcknowledged);
        return;
    }
    match parse_stl_asset(&snapshot, &request.units) {
        Ok(mesh) => coordinator.complete(attempt, Event::ParseSucceeded { mesh }),
        Err(error) => {
            let (code, detail) = activation_failure(&error);
            coordinator.complete(attempt, Event::Fail { code, detail });
        }
    }
}

/// 默认 spawner：worker 线程分离运行，JoinHandle 直接丢弃（不 join）。
fn spawn_activation_worker(
    coordinator: Arc<ActivationCoordinator>,
    attempt: u64,
    request: ActivationRequest,
    cancel_requested: Arc<AtomicBool>,
) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name(format!("panta-activation-{attempt}"))
        .spawn(move || run_attempt(coordinator, attempt, request, cancel_requested))
        .map(drop)
}

fn activation_failure(error: &ImportError) -> (&'static str, String) {
    match error {
        ImportError::Missing(path) => ("project.asset_missing", path.clone()),
        ImportError::UnsupportedFormat(path) => ("project.asset_unsupported_format", path.clone()),
        ImportError::Read(detail) => ("project.asset_read_failed", detail.clone()),
        ImportError::Parse(error) => ("project.asset_parse_failed", error.to_string()),
        ImportError::SourceChanged(path) => ("project.asset_source_changed", path.clone()),
        ImportError::UnsupportedUnits(units) => ("project.import_unsupported_units", units.clone()),
        ImportError::UnsupportedMeshType(kind) => {
            ("project.import_unsupported_mesh_type", kind.clone())
        }
        ImportError::CoordinateOverflow => (
            "project.asset_coordinate_overflow",
            "millimeter coordinate overflow".to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 双向结构校验（073）：生成转移表 × 手写决策全对全比较。声明边在
    /// guard 为真时必须 Enter(to)、为假时必须 GuardRejected；未声明组合
    /// 在任意 guard 组合下必须 Illegal。
    #[test]
    fn generated_transitions_match_handwritten_decisions() {
        for &state in fsm::ALL_STATES {
            for &event in fsm::ALL_EVENTS {
                let declared: Vec<_> = fsm::TRANSITIONS
                    .iter()
                    .filter(|transition| transition.from == state && transition.on == event)
                    .collect();
                assert!(
                    declared.len() <= 1,
                    "状态 {:?} 事件 {:?} 存在重复声明边",
                    state.name(),
                    event.name()
                );
                match declared.first() {
                    None => {
                        for record_valid in [true, false] {
                            for session_current in [true, false] {
                                assert!(
                                    matches!(
                                        decide(
                                            state,
                                            event,
                                            &GuardContext {
                                                record_valid,
                                                session_current,
                                            }
                                        ),
                                        Decision::Illegal
                                    ),
                                    "未声明组合 {:?} x {:?} 必须 Illegal",
                                    state.name(),
                                    event.name()
                                );
                            }
                        }
                    }
                    Some(transition) => match transition.guard {
                        None => {
                            for record_valid in [true, false] {
                                for session_current in [true, false] {
                                    assert!(
                                        matches!(
                                            decide(
                                                state,
                                                event,
                                                &GuardContext {
                                                    record_valid,
                                                    session_current,
                                                }
                                            ),
                                            Decision::Enter(entered) if entered == transition.to
                                        ),
                                        "无边 guard 的 {:?} x {:?} 必须进入 {:?}",
                                        state.name(),
                                        event.name(),
                                        transition.to.name()
                                    );
                                }
                            }
                        }
                        Some(fsm::Guard::RecordValid) => {
                            assert!(
                                matches!(
                                    decide(
                                        state,
                                        event,
                                        &GuardContext {
                                            record_valid: true,
                                            session_current: false,
                                        }
                                    ),
                                    Decision::Enter(entered) if entered == transition.to
                                ),
                                "record-valid 为真必须进入 {:?}",
                                transition.to.name()
                            );
                            assert!(
                                matches!(
                                    decide(
                                        state,
                                        event,
                                        &GuardContext {
                                            record_valid: false,
                                            session_current: true,
                                        }
                                    ),
                                    Decision::GuardRejected
                                ),
                                "record-valid 为假必须拒绝"
                            );
                        }
                        Some(fsm::Guard::SessionCurrent) => {
                            assert!(
                                matches!(
                                    decide(
                                        state,
                                        event,
                                        &GuardContext {
                                            record_valid: false,
                                            session_current: true,
                                        }
                                    ),
                                    Decision::Enter(entered) if entered == transition.to
                                ),
                                "session-current 为真必须进入 {:?}",
                                transition.to.name()
                            );
                            assert!(
                                matches!(
                                    decide(
                                        state,
                                        event,
                                        &GuardContext {
                                            record_valid: true,
                                            session_current: false,
                                        }
                                    ),
                                    Decision::GuardRejected
                                ),
                                "session-current 为假必须拒绝"
                            );
                        }
                    },
                }
            }
        }
        // 声明完备性：状态、事件与声明边数量与 open-saved-stl 流程一致；
        // 增减声明而未更新手写决策时，上面的全对全比较会先失败。
        assert_eq!(fsm::ALL_STATES.len(), 7);
        assert_eq!(fsm::ALL_EVENTS.len(), 6);
        assert_eq!(fsm::TRANSITIONS.len(), 11);
    }

    fn coordinator_with_request() -> (Arc<ActivationCoordinator>, ActivationAttempt) {
        let coordinator = Arc::new(ActivationCoordinator::default());
        let attempt = coordinator.begin_with(
            "import-1",
            ActivationRequest {
                asset_path: PathBuf::from("unused.stl"),
                units: "millimeters".to_owned(),
            },
            |_coordinator, _attempt, _request, _cancel| Ok(()),
        );
        (coordinator, attempt)
    }

    #[test]
    fn success_publishes_snapshot_once_and_consumes_attempt() {
        let (coordinator, attempt) = coordinator_with_request();
        coordinator.complete(attempt.attempt, Event::AssetRead);
        coordinator.complete(
            attempt.attempt,
            Event::ParseSucceeded {
                mesh: SurfaceMesh {
                    triangles: Vec::new(),
                },
            },
        );
        // 终态后重复完成是迟到结果：Illegal，不重复发布。
        coordinator.complete(
            attempt.attempt,
            Event::ParseSucceeded {
                mesh: SurfaceMesh {
                    triangles: Vec::new(),
                },
            },
        );
        let outcomes = coordinator.drain();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, OutcomeKind::Succeeded);
        assert_eq!(outcomes[0].import_id, "import-1");
        assert!(outcomes[0].mesh.is_some());
        assert!(coordinator.drain().is_empty(), "结果只发布一次");
        assert!(!coordinator.cancel(attempt.attempt), "终态后不可取消");
    }

    #[test]
    fn fail_and_cancel_paths_publish_distinct_outcomes() {
        let (coordinator, attempt) = coordinator_with_request();
        coordinator.complete(
            attempt.attempt,
            Event::Fail {
                code: "project.asset_missing",
                detail: "part.stl".to_owned(),
            },
        );
        let outcomes = coordinator.drain();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
        assert_eq!(outcomes[0].code, "project.asset_missing");

        let (coordinator, attempt) = coordinator_with_request();
        coordinator.complete(attempt.attempt, Event::CancelAcknowledged);
        let outcomes = coordinator.drain();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, OutcomeKind::Cancelled);
    }

    #[test]
    fn generation_invalidation_drops_late_snapshot() {
        let (coordinator, attempt) = coordinator_with_request();
        coordinator.advance_generation();
        // 旧代次结果（含无负载的 Expired）不得跨会话发布：drain 为空，
        // 且 attempt 已终态化（不可再取消）。
        assert!(coordinator.drain().is_empty());
        assert!(!coordinator.cancel(attempt.attempt));
        // 迟到的解析成功在 Expired 之后到达：Illegal，快照就地释放。
        coordinator.complete(
            attempt.attempt,
            Event::ParseSucceeded {
                mesh: SurfaceMesh {
                    triangles: Vec::new(),
                },
            },
        );
        assert!(coordinator.drain().is_empty(), "迟到成功不得发布");
    }

    #[test]
    fn generation_advance_without_attempts_is_a_noop() {
        let coordinator = Arc::new(ActivationCoordinator::default());
        assert_eq!(coordinator.advance_generation(), 1);
        assert!(coordinator.drain().is_empty());
        assert_eq!(coordinator.advance_generation(), 2);
        assert!(coordinator.drain().is_empty());
    }

    #[test]
    fn drain_drops_results_queued_for_stale_generations() {
        let (coordinator, attempt) = coordinator_with_request();
        coordinator.complete(attempt.attempt, Event::AssetRead);
        coordinator.complete(
            attempt.attempt,
            Event::ParseSucceeded {
                mesh: SurfaceMesh {
                    triangles: Vec::new(),
                },
            },
        );
        // 成功结果已入队；随后代次推进（工程切换）：旧代次结果在 drain
        // 边界被过滤，坐标（含网格负载）就地释放，不跨会话发布。
        coordinator.advance_generation();
        assert!(coordinator.drain().is_empty());
    }

    #[test]
    fn duplicate_begin_reuses_in_flight_attempt() {
        let coordinator = Arc::new(ActivationCoordinator::default());
        let request = || ActivationRequest {
            asset_path: PathBuf::from("unused.stl"),
            units: "millimeters".to_owned(),
        };
        let first = coordinator.begin_with("import-1", request(), |_c, _a, _r, _cancel| Ok(()));
        let second = coordinator.begin_with("import-1", request(), |_c, _a, _r, _cancel| Ok(()));
        assert_eq!(first.attempt, second.attempt);
        assert_eq!(first.generation, second.generation);
        // 不同记录仍创建独立 attempt。
        let other = coordinator.begin_with("import-2", request(), |_c, _a, _r, _cancel| Ok(()));
        assert_ne!(first.attempt, other.attempt);
    }

    #[test]
    fn spawn_failure_terminates_with_declared_edge() {
        let coordinator = Arc::new(ActivationCoordinator::default());
        let attempt = coordinator.begin_with(
            "import-1",
            ActivationRequest {
                asset_path: PathBuf::from("unused.stl"),
                units: "millimeters".to_owned(),
            },
            |_coordinator, _attempt, _request, _cancel| {
                Err(std::io::Error::other("thread pool exhausted"))
            },
        );
        let outcomes = coordinator.drain();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
        assert_eq!(outcomes[0].code, "project.activation_spawn_failed");
        assert_eq!(outcomes[0].attempt, attempt.attempt);
    }

    #[test]
    fn asset_read_advances_and_parse_failure_terminates() {
        let (coordinator, attempt) = coordinator_with_request();
        coordinator.complete(attempt.attempt, Event::AssetRead);
        // 资产读取后的中间态没有可观察结果。
        assert!(coordinator.drain().is_empty());
        coordinator.complete(
            attempt.attempt,
            Event::Fail {
                code: "project.asset_parse_failed",
                detail: "invalid stl".to_owned(),
            },
        );
        let outcomes = coordinator.drain();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].kind, OutcomeKind::Failed);
        assert_eq!(outcomes[0].code, "project.asset_parse_failed");
    }
}
