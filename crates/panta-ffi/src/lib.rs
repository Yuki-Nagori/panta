//! 最小 CXX 双向边界：验证 DTO、opaque 句柄所有权、错误转换、C++ 实现
//! 调用，以及 Rust 拥有的任务生命周期（任务 008）、路径服务（任务 023）
//! 与工程 application service（任务 057）。

use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_REPEAT: u32 = 8;
/// 句柄标签按 UTF-8 字节数设界，与请求文本的有界策略一致。
const MAX_LABEL_BYTES: usize = 64;

// unsafe 由 CXX 桥接宏生成的胶水产生（边界安全前提由 cxx 运行时的类型
// 检查与 ffi.hpp 签名一致性承担）。崩溃设施的手写 unsafe 位于
// panta-foundation::crash，本 crate 对外只暴露安全签名。
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

    /// Rust 工程服务返回的稳定状态快照；path/name 均为可往返 UTF-8。
    #[derive(Clone, PartialEq, Eq)]
    pub struct ProjectSnapshot {
        pub path: String,
        pub name: String,
        pub revision: u64,
        pub dirty: bool,
    }

    /// 已导入网格的持久化元数据。尺寸拆成标量，避免 CXX shared struct
    /// 直接暴露 Rust 数组，便于 Qt 侧消费。
    #[derive(Clone, PartialEq)]
    pub struct ProjectImport {
        pub record_version: u32,
        pub parser_version: u32,
        pub id: String,
        pub source_name: String,
        pub asset: String,
        pub format: String,
        pub mesh_type: String,
        pub units: String,
        pub show_import_log: bool,
        pub triangle_count: u64,
        pub size_x: f64,
        pub size_y: f64,
        pub size_z: f64,
    }

    /// STL metadata shown by the import dialog before the file is copied.
    #[derive(Clone, PartialEq)]
    pub struct StlImportPreview {
        pub source_name: String,
        pub triangle_count: u64,
        pub size_x: f64,
        pub size_y: f64,
        pub size_z: f64,
    }

    /// 连续三角面坐标：每三个 f64 为一个点，每三个点为一片面；
    /// CXX Vec 独占缓冲区，调用返回后 C++ 复制到自身场景快照。
    pub struct SurfaceMeshSnapshot {
        pub coordinates: Vec<f64>,
        pub revision: u64,
    }

    /// native Netgen 转换结果的批量输入。坐标每三项一个节点；tet 每四项、
    /// boundary 每三项一个单元；区域 / 分组数组各与对应单元数一致。
    pub struct TetMeshData {
        pub nodes: Vec<f64>,
        pub tets: Vec<u32>,
        pub tet_regions: Vec<u32>,
        pub boundary: Vec<u32>,
        pub boundary_groups: Vec<u32>,
        pub region_count: u32,
        pub boundary_group_count: u32,
    }

    /// Rust 领域校验结果；诊断和有效网格总体积统一从 Rust 返回。
    pub struct TetMeshValidation {
        pub issues: Vec<String>,
        pub volume_mm3: f64,
    }

    /// 工程模型命令种类；新增值必须同步 Rust 映射与失败测试。
    pub enum ProjectCommandKind {
        Rename = 0,
    }

    /// 工程模型命令 DTO；value 的语义由 kind 决定。
    pub struct ProjectCommand {
        pub kind: ProjectCommandKind,
        pub value: String,
    }

    /// 只读资产激活终态种类（073/080）。`Expired` 仅用于可观测性，
    /// UI 不得依据它更新任何文档状态。
    pub enum ActivationOutcomeKind {
        Succeeded = 0,
        Failed = 1,
        Cancelled = 2,
        Expired = 3,
    }

    /// begin 返回的运行期相关性句柄；generation 是会话级计数（create/open
    /// 递增），不是工程 revision，也不持久化。
    pub struct ActivationAttempt {
        pub attempt: u64,
        pub generation: u64,
    }

    /// 一次激活的终态结果。coordinates 仅在 Succeeded 时非空（每三角形
    /// 9 个 f64）；C++ 复制进自身文档快照后即释放 CXX 缓冲区。
    pub struct ActivationOutcome {
        pub attempt: u64,
        pub generation: u64,
        pub import_id: String,
        pub kind: ActivationOutcomeKind,
        pub code: String,
        pub detail: String,
        pub coordinates: Vec<f64>,
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

        /// Rust 拥有的工程 application service：目录、清单和模型状态不由
        /// QML/C++ 持有；C++ 只负责 Qt 字符串/URL 适配和 ViewModel 通知。
        type ProjectService;

        fn project_service_new() -> Box<ProjectService>;
        fn project_service_create(
            service: &mut ProjectService,
            location: String,
            name: String,
        ) -> Result<ProjectSnapshot>;
        fn project_service_open(
            service: &mut ProjectService,
            path: String,
        ) -> Result<ProjectSnapshot>;
        fn project_service_save(service: &mut ProjectService) -> Result<ProjectSnapshot>;
        fn project_service_execute(
            service: &mut ProjectService,
            command: ProjectCommand,
        ) -> Result<ProjectSnapshot>;
        fn project_service_current(service: &ProjectService) -> Result<ProjectSnapshot>;
        fn project_service_import_stl(
            service: &mut ProjectService,
            source: String,
            mesh_type: String,
            units: String,
            show_import_log: bool,
        ) -> Result<ProjectImport>;
        fn project_service_imports(service: &ProjectService) -> Result<Vec<ProjectImport>>;
        fn project_service_inspect_stl(
            service: &mut ProjectService,
            source: String,
        ) -> Result<StlImportPreview>;
        fn project_service_mesh_snapshot(service: &ProjectService) -> Result<SurfaceMeshSnapshot>;

        /// 只读资产激活（073 首个 FSM 消费者）：按稳定 ImportRecord ID
        /// 异步读取并解析已提交 STL；结果只在会话仍有效时发布。
        fn project_service_begin_asset_activation(
            service: &mut ProjectService,
            import_id: &str,
        ) -> Result<ActivationAttempt>;
        fn project_service_cancel_asset_activation(
            service: &mut ProjectService,
            attempt: u64,
        ) -> bool;
        fn project_service_drain_asset_activations(
            service: &mut ProjectService,
        ) -> Vec<ActivationOutcome>;
        fn mesh_validate_tet(data: TetMeshData) -> TetMeshValidation;

        /// 崩溃信号处理器安装（任务 047，panta_foundation::crash 的 FFI 面）：
        /// 返回日志路径；log_dir 为空时用系统临时目录。
        fn install_crash_handler(log_dir: &str) -> Result<String>;
    }
}

pub use bridge::{FfiRequest, FfiResponse};

/// 任务 047：错误以文本跨边界（C++ 侧 qWarning 呈现，不静默）。
fn install_crash_handler(log_dir: &str) -> Result<String, String> {
    panta_foundation::crash::install_crash_handler(log_dir)
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

/// Rust 工程 application service 的 CXX 适配器：只负责字符串 DTO 和错误文本，
/// 工程校验、清单事务及模型状态均保留在 panta-core。
pub struct ProjectService {
    service: panta_core::project::ProjectService,
}

fn project_service_new() -> Box<ProjectService> {
    Box::new(ProjectService {
        service: panta_core::project::ProjectService::new(),
    })
}

fn project_service_create(
    service: &mut ProjectService,
    location: String,
    name: String,
) -> Result<bridge::ProjectSnapshot, String> {
    service
        .service
        .create(std::path::Path::new(&location), &name)
        .map(project_snapshot)
        .map_err(|error| error.to_string())
}

fn project_service_open(
    service: &mut ProjectService,
    path: String,
) -> Result<bridge::ProjectSnapshot, String> {
    service
        .service
        .open(std::path::Path::new(&path))
        .map(project_snapshot)
        .map_err(|error| error.to_string())
}

fn project_service_save(service: &mut ProjectService) -> Result<bridge::ProjectSnapshot, String> {
    service
        .service
        .save()
        .map(project_snapshot)
        .map_err(|error| error.to_string())
}

fn project_service_execute(
    service: &mut ProjectService,
    command: bridge::ProjectCommand,
) -> Result<bridge::ProjectSnapshot, String> {
    let command = match command.kind {
        bridge::ProjectCommandKind::Rename => panta_core::project::ProjectCommand::Rename {
            name: command.value,
        },
        _ => {
            return Err("project.command_invalid: unknown command".to_owned());
        }
    };
    service
        .service
        .execute(command)
        .map(project_snapshot)
        .map_err(|error| error.to_string())
}

fn project_service_current(service: &ProjectService) -> Result<bridge::ProjectSnapshot, String> {
    service
        .service
        .current()
        .map(project_snapshot)
        .map_err(|error| error.to_string())
}

fn project_service_import_stl(
    service: &mut ProjectService,
    source: String,
    mesh_type: String,
    units: String,
    show_import_log: bool,
) -> Result<bridge::ProjectImport, String> {
    service
        .service
        .import_stl(
            std::path::Path::new(&source),
            &mesh_type,
            &units,
            show_import_log,
        )
        .map(project_import)
        .map_err(|error| error.to_string())
}

fn project_service_imports(service: &ProjectService) -> Result<Vec<bridge::ProjectImport>, String> {
    service
        .service
        .imports()
        .map(|imports| imports.into_iter().map(project_import).collect())
        .map_err(|error| error.to_string())
}

fn project_service_inspect_stl(
    service: &mut ProjectService,
    source: String,
) -> Result<bridge::StlImportPreview, String> {
    service
        .service
        .inspect_stl(std::path::Path::new(&source))
        .map(stl_import_preview)
        .map_err(|error| error.to_string())
}

fn mesh_validate_tet(data: bridge::TetMeshData) -> bridge::TetMeshValidation {
    if !data.nodes.len().is_multiple_of(3)
        || !data.tets.len().is_multiple_of(4)
        || !data.boundary.len().is_multiple_of(3)
        || data.tet_regions.len() != data.tets.len() / 4
        || data.boundary_groups.len() != data.boundary.len() / 3
    {
        return bridge::TetMeshValidation {
            issues: vec!["invalid mesh DTO layout".to_owned()],
            volume_mm3: 0.0,
        };
    }
    let mut nodes = Vec::new();
    let mut tets = Vec::new();
    let mut boundary = Vec::new();
    if let Err(error) = nodes.try_reserve_exact(data.nodes.len() / 3) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate mesh nodes: {error}")],
            volume_mm3: 0.0,
        };
    }
    if let Err(error) = tets.try_reserve_exact(data.tets.len() / 4) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate tetrahedra: {error}")],
            volume_mm3: 0.0,
        };
    }
    if let Err(error) = boundary.try_reserve_exact(data.boundary.len() / 3) {
        return bridge::TetMeshValidation {
            issues: vec![format!("cannot allocate boundary triangles: {error}")],
            volume_mm3: 0.0,
        };
    }
    nodes.extend(
        data.nodes
            .as_chunks::<3>()
            .0
            .iter()
            .map(|point| [point[0], point[1], point[2]]),
    );
    tets.extend(
        data.tets
            .as_chunks::<4>()
            .0
            .iter()
            .zip(data.tet_regions)
            .map(|(nodes, region)| panta_mesh::Tetrahedron {
                nodes: [nodes[0], nodes[1], nodes[2], nodes[3]],
                region,
            }),
    );
    boundary.extend(
        data.boundary
            .as_chunks::<3>()
            .0
            .iter()
            .zip(data.boundary_groups)
            .map(|(nodes, group)| panta_mesh::SurfaceTriangle {
                nodes: [nodes[0], nodes[1], nodes[2]],
                group,
            }),
    );
    let mesh = panta_mesh::TetMesh {
        nodes,
        tets,
        boundary,
        region_count: data.region_count,
        boundary_group_count: data.boundary_group_count,
    };
    let report = panta_mesh::validate_tet_mesh(&mesh);
    bridge::TetMeshValidation {
        issues: report.issues,
        volume_mm3: report.volume_mm3,
    }
}

/// SurfaceMesh → 扁平坐标缓冲；分配失败以稳定文本错误返回。
fn mesh_coordinates(mesh: &panta_mesh::SurfaceMesh) -> Result<Vec<f64>, String> {
    let mut coordinates = Vec::new();
    let values = mesh
        .triangles
        .len()
        .checked_mul(9)
        .ok_or("mesh snapshot coordinate count overflow")?;
    coordinates
        .try_reserve_exact(values)
        .map_err(|error| format!("mesh snapshot allocation failed: {error}"))?;
    for triangle in &mesh.triangles {
        for point in triangle {
            coordinates.extend_from_slice(point);
        }
    }
    Ok(coordinates)
}

fn project_service_mesh_snapshot(
    service: &ProjectService,
) -> Result<bridge::SurfaceMeshSnapshot, String> {
    let snapshot = service
        .service
        .current()
        .map_err(|error| error.to_string())?;
    let coordinates = match service.service.current_mesh() {
        Some(mesh) => mesh_coordinates(mesh)?,
        None => Vec::new(),
    };
    Ok(bridge::SurfaceMeshSnapshot {
        coordinates,
        revision: snapshot.revision,
    })
}

fn project_service_begin_asset_activation(
    service: &mut ProjectService,
    import_id: &str,
) -> Result<bridge::ActivationAttempt, String> {
    service
        .service
        .begin_asset_activation(import_id)
        .map(|attempt| bridge::ActivationAttempt {
            attempt: attempt.attempt,
            generation: attempt.generation,
        })
        .map_err(|error| error.to_string())
}

fn project_service_cancel_asset_activation(service: &mut ProjectService, attempt: u64) -> bool {
    service.service.cancel_asset_activation(attempt)
}

fn project_service_drain_asset_activations(
    service: &mut ProjectService,
) -> Vec<bridge::ActivationOutcome> {
    service
        .service
        .drain_asset_activations()
        .into_iter()
        .map(|outcome| {
            // 坐标跨越所有权边界时才转换；分配失败把本次结果降级为带稳定
            // 错误码的 Failed，不发布无负载的假成功。
            let (kind, code, detail, coordinates) = match outcome.mesh {
                Some(mesh) => match mesh_coordinates(&mesh) {
                    Ok(coordinates) => (
                        bridge::ActivationOutcomeKind::Succeeded,
                        outcome.code,
                        outcome.detail,
                        coordinates,
                    ),
                    Err(detail) => (
                        bridge::ActivationOutcomeKind::Failed,
                        "project.mesh_allocation_failed".to_owned(),
                        detail,
                        Vec::new(),
                    ),
                },
                None => (
                    activation_kind(outcome.kind),
                    outcome.code,
                    outcome.detail,
                    Vec::new(),
                ),
            };
            bridge::ActivationOutcome {
                attempt: outcome.attempt,
                generation: outcome.generation,
                import_id: outcome.import_id,
                kind,
                code,
                detail,
                coordinates,
            }
        })
        .collect()
}

fn activation_kind(kind: panta_core::project::OutcomeKind) -> bridge::ActivationOutcomeKind {
    match kind {
        panta_core::project::OutcomeKind::Succeeded => bridge::ActivationOutcomeKind::Succeeded,
        panta_core::project::OutcomeKind::Failed => bridge::ActivationOutcomeKind::Failed,
        panta_core::project::OutcomeKind::Cancelled => bridge::ActivationOutcomeKind::Cancelled,
        panta_core::project::OutcomeKind::Expired => bridge::ActivationOutcomeKind::Expired,
    }
}

fn project_snapshot(snapshot: panta_core::project::ProjectSnapshot) -> bridge::ProjectSnapshot {
    bridge::ProjectSnapshot {
        path: snapshot.path.display().to_string(),
        name: snapshot.name,
        revision: snapshot.revision,
        dirty: snapshot.dirty,
    }
}

fn project_import(import: panta_core::project::ImportRecord) -> bridge::ProjectImport {
    bridge::ProjectImport {
        record_version: import.record_version,
        parser_version: import.parser_version,
        id: import.id,
        source_name: import.source_name,
        asset: import.asset,
        format: import.format,
        mesh_type: import.mesh_type,
        units: import.units,
        show_import_log: import.show_import_log,
        triangle_count: import.triangle_count,
        size_x: import.dimensions[0],
        size_y: import.dimensions[1],
        size_z: import.dimensions[2],
    }
}

fn stl_import_preview(preview: panta_core::project::StlImportPreview) -> bridge::StlImportPreview {
    bridge::StlImportPreview {
        source_name: preview.source_name,
        triangle_count: preview.triangle_count,
        size_x: preview.dimensions[0],
        size_y: preview.dimensions[1],
        size_z: preview.dimensions[2],
    }
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
        path_service_resolve_write_target, path_service_set_root, process, project_service_create,
        project_service_current, project_service_execute, project_service_new,
        project_service_open, project_service_save, session_close, session_create, session_label,
        session_live_count, task_service_cancel, task_service_drain, task_service_new,
        task_service_recent_logs, task_service_running, task_service_submit,
    };
    use std::fs;
    use std::path::PathBuf;
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
        let invalid_root = match path_service_set_root(
            &mut service,
            bridge::PathRootKind::Project,
            "relative-root".to_owned(),
        ) {
            Ok(()) => panic!("relative root unexpectedly accepted"),
            Err(error) => error,
        };
        assert!(invalid_root.starts_with("path.root_not_absolute:"));

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

    #[test]
    fn project_service_bridge_maps_snapshots_and_errors() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = std::env::temp_dir().join(format!("panta-ffi-project-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let mut service = project_service_new();
        let no_project = match project_service_current(&service) {
            Ok(snapshot) => panic!("empty service returned {}", snapshot.name),
            Err(error) => error,
        };
        assert_eq!(no_project, "project.no_project");

        let create_error =
            match project_service_create(&mut service, "relative".to_owned(), "Demo".to_owned()) {
                Ok(snapshot) => panic!("relative project created at {}", snapshot.path),
                Err(error) => error,
            };
        assert_eq!(create_error, "project.location_not_absolute: relative");
        let save_error = match project_service_save(&mut service) {
            Ok(snapshot) => panic!("empty service saved {}", snapshot.name),
            Err(error) => error,
        };
        assert_eq!(save_error, "project.no_project");

        let created =
            project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
        assert_eq!(created.name, "Demo");
        assert!(!created.dirty);
        let current = project_service_current(&service)?;
        assert_eq!(current.path, created.path);
        assert_eq!(current.name, "Demo");

        let changed = project_service_execute(
            &mut service,
            bridge::ProjectCommand {
                kind: bridge::ProjectCommandKind::Rename,
                value: "Renamed".to_owned(),
            },
        )?;
        assert_eq!(changed.name, "Renamed");
        assert!(changed.dirty);

        let saved = project_service_save(&mut service)?;
        assert_eq!(saved.revision, 1);
        assert!(!saved.dirty);
        let unchanged = match project_service_execute(
            &mut service,
            bridge::ProjectCommand {
                kind: bridge::ProjectCommandKind::Rename,
                value: "Renamed".to_owned(),
            },
        ) {
            Ok(snapshot) => panic!(
                "unchanged rename succeeded at revision {}",
                snapshot.revision
            ),
            Err(error) => error,
        };
        assert_eq!(unchanged, "project.command_invalid: name is unchanged");

        let mut reopened = project_service_new();
        let opened = project_service_open(&mut reopened, created.path.clone())?;
        assert_eq!(opened.name, "Renamed");
        assert_eq!(opened.revision, 1);
        let invalid_file = PathBuf::from(&created.path).with_extension("json");
        fs::write(&invalid_file, br#"{}"#)?;
        let invalid_file_error =
            match project_service_open(&mut reopened, invalid_file.display().to_string()) {
                Ok(snapshot) => panic!("non-panta file opened as {}", snapshot.name),
                Err(error) => error,
            };
        assert_eq!(
            invalid_file_error,
            format!("project.invalid_file: {}", invalid_file.display())
        );

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn project_activation_bridge_admits_cancels_and_drains_outcomes()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("panta-ffi-act-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        let source = root.join("part.stl");
        let original = b"solid part
vertex 0 0 0
vertex 1 0 0
vertex 0 2 0
endsolid
";
        fs::write(&source, original)?;

        let mut service = project_service_new();
        let created =
            project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
        crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "solid-3d".to_owned(),
            "millimeters".to_owned(),
            false,
        )?;

        // 未知记录在提交边界同步拒绝；未知 attempt 取消返回 false。
        let missing = match crate::project_service_begin_asset_activation(&mut service, "import-99")
        {
            Ok(attempt) => panic!("unknown record admitted as attempt {}", attempt.attempt),
            Err(error) => error,
        };
        assert!(missing.contains("project.import_record_missing"));
        assert!(!crate::project_service_cancel_asset_activation(
            &mut service,
            u64::MAX
        ));

        let attempt = crate::project_service_begin_asset_activation(&mut service, "import-1")?;
        assert!(attempt.attempt > 0);
        assert!(crate::project_service_cancel_asset_activation(
            &mut service,
            attempt.attempt
        ));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let outcome = loop {
            let drained = crate::project_service_drain_asset_activations(&mut service);
            if let Some(outcome) = drained.into_iter().next() {
                break outcome;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "activation outcome did not arrive in time"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(outcome.attempt, attempt.attempt);
        assert_eq!(outcome.generation, attempt.generation);
        assert_eq!(outcome.import_id, "import-1");
        assert!(
            outcome.kind == bridge::ActivationOutcomeKind::Succeeded
                || outcome.kind == bridge::ActivationOutcomeKind::Cancelled,
            "小文件激活终态只可能是成功（先完成）或取消（先命中检查点）"
        );
        if outcome.kind == bridge::ActivationOutcomeKind::Succeeded {
            // 单三角形：9 个 f64 坐标必须完整过桥。
            assert_eq!(outcome.coordinates.len(), 9);
        } else {
            assert!(outcome.coordinates.is_empty(), "取消不得携带坐标负载");
        }
        let _ = created;
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    #[test]
    fn project_stl_bridge_preserves_preview_commit_and_snapshot_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("panta-ffi-stl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        let source = root.join("part.stl");
        let original = b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
        fs::write(&source, original)?;

        let mut service = project_service_new();
        let no_project = match crate::project_service_imports(&service) {
            Ok(imports) => panic!("empty project returned {} imports", imports.len()),
            Err(error) => error,
        };
        assert_eq!(no_project, "project.no_project");
        let no_project_mesh = match crate::project_service_mesh_snapshot(&service) {
            Ok(snapshot) => panic!("empty project returned revision {}", snapshot.revision),
            Err(error) => error,
        };
        assert_eq!(no_project_mesh, "project.no_project");

        let preview =
            crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
        assert_eq!(preview.source_name, "part.stl");
        assert_eq!(preview.triangle_count, 1);
        assert_eq!(
            [preview.size_x, preview.size_y, preview.size_z],
            [1.0, 2.0, 0.0]
        );
        let import_without_project = match crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "solid-3d".to_owned(),
            "millimeters".to_owned(),
            false,
        ) {
            Ok(imported) => panic!("import without a project succeeded as {}", imported.id),
            Err(error) => error,
        };
        assert_eq!(import_without_project, "project.no_project");

        let created =
            project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
        let before_import = crate::project_service_mesh_snapshot(&service)?;
        assert_eq!(before_import.revision, created.revision);
        assert!(before_import.coordinates.is_empty());
        assert!(crate::project_service_imports(&service)?.is_empty());

        let imported = crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "solid-3d".to_owned(),
            "centimeters".to_owned(),
            true,
        )?;
        assert_eq!(imported.id, "import-1");
        assert_eq!(imported.source_name, "part.stl");
        assert_eq!(imported.asset, "assets/imports/0001-part.stl");
        assert_eq!(imported.mesh_type, "solid-3d");
        assert_eq!(imported.units, "centimeters");
        assert!(imported.show_import_log);
        assert_eq!(imported.triangle_count, 1);
        assert_eq!(
            [imported.size_x, imported.size_y, imported.size_z],
            [1.0, 2.0, 0.0]
        );

        let imports = crate::project_service_imports(&service)?;
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].parser_version, imported.parser_version);
        let snapshot = crate::project_service_mesh_snapshot(&service)?;
        assert_eq!(snapshot.revision, 1);
        assert_eq!(
            snapshot.coordinates,
            [0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 20.0, 0.0]
        );

        let mut reopened = project_service_new();
        project_service_open(&mut reopened, created.path)?;
        // 打开工程只读清单；已保存网格必须经只读激活按需恢复（073/080），
        // 这里验证修订投影不变且坐标为空（未激活）。
        let restored = crate::project_service_mesh_snapshot(&reopened)?;
        assert_eq!(restored.revision, snapshot.revision);
        assert!(restored.coordinates.is_empty());

        let step_source = root.join("part.step");
        fs::write(&step_source, b"not a supported STL source")?;
        let unsupported = match crate::project_service_inspect_stl(
            &mut service,
            step_source.display().to_string(),
        ) {
            Ok(preview) => panic!(
                "STEP file unexpectedly parsed as {} triangles",
                preview.triangle_count
            ),
            Err(error) => error,
        };
        assert!(unsupported.starts_with("project.import_invalid_file:"));

        crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
        fs::write(&source, b"vertex 0 0 0\nvertex 3 0 0\nvertex 0 4 0\n")?;
        let changed = match crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "solid-3d".to_owned(),
            "millimeters".to_owned(),
            false,
        ) {
            Ok(imported) => panic!("changed source was imported as {}", imported.id),
            Err(error) => error,
        };
        assert!(changed.starts_with("project.import_source_changed:"));
        assert_eq!(crate::project_service_imports(&service)?.len(), 1);

        fs::write(&source, original)?;
        let invalid_options = match crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "unknown".to_owned(),
            "millimeters".to_owned(),
            false,
        ) {
            Ok(imported) => panic!("unsupported mesh type was imported as {}", imported.id),
            Err(error) => error,
        };
        assert!(invalid_options.starts_with("project.import_unsupported_mesh_type:"));
        assert_eq!(crate::project_service_imports(&service)?.len(), 1);

        let invalid_units = match crate::project_service_import_stl(
            &mut service,
            source.display().to_string(),
            "solid-3d".to_owned(),
            "yards".to_owned(),
            false,
        ) {
            Ok(imported) => panic!("unsupported units were imported as {}", imported.id),
            Err(error) => error,
        };
        assert!(invalid_units.starts_with("project.import_unsupported_units:"));
        assert_eq!(crate::project_service_imports(&service)?.len(), 1);

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn tet_mesh_bridge_checks_layout_and_domain_data() {
        let mesh_data = || bridge::TetMeshData {
            nodes: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            tets: vec![0, 1, 2, 3],
            tet_regions: vec![0],
            boundary: vec![0, 1, 2],
            boundary_groups: vec![0],
            region_count: 1,
            boundary_group_count: 1,
        };

        let valid = crate::mesh_validate_tet(mesh_data());
        assert!(valid.issues.is_empty());
        assert_eq!(valid.volume_mm3, 1.0 / 6.0);

        let mut invalid_layout = mesh_data();
        invalid_layout.nodes.pop();
        let layout = crate::mesh_validate_tet(invalid_layout);
        assert_eq!(layout.issues, ["invalid mesh DTO layout"]);
        assert_eq!(layout.volume_mm3, 0.0);

        let mut invalid_mesh = mesh_data();
        invalid_mesh.tets[3] = 9;
        let domain = crate::mesh_validate_tet(invalid_mesh);
        assert!(!domain.issues.is_empty());
        assert_eq!(domain.volume_mm3, 0.0);
    }
}
