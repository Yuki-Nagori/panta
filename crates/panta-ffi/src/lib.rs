//! 最小 CXX 双向边界：验证 DTO、opaque 句柄所有权、错误转换、C++ 实现
//! 调用，以及路径服务（任务 023）与工程 application service（任务 057）。

mod project_response;

#[cfg(test)]
#[path = "../../../tests/support/rust/temp_directory.rs"]
mod temp_directory;

use project_response::{
    default_material_response, project_service_begin_asset_activation_response,
    project_service_begin_fill_settings_confirmation_response,
    project_service_begin_gate_location_settings_confirmation_response,
    project_service_begin_material_confirmation_response,
    project_service_begin_stl_import_response, project_service_begin_stl_preview_response,
    project_service_cancel_stl_import_response, project_service_cancel_stl_preview,
    project_service_create_response, project_service_current_response,
    project_service_execute_response, project_service_finish_fill_settings_confirmation_response,
    project_service_finish_gate_location_settings_confirmation_response,
    project_service_finish_material_confirmation_response,
    project_service_finish_stl_import_response, project_service_finish_stl_preview_response,
    project_service_import_stl_response, project_service_imports_response,
    project_service_mesh_snapshot_for_import_response, project_service_open_response,
    project_service_save_response, project_service_set_analysis_sequence_response,
};
mod support;
pub use support::Session;
use support::{
    finish_background_writes, install_crash_handler, panic_probe, process, session_close,
    session_create, session_label, session_live_count,
};
mod theme;
pub use theme::ThemeService;
use theme::{
    theme_service_current, theme_service_new, theme_service_select, theme_service_theme_ids,
};
mod settings;
pub use settings::UserSettingsService;
use settings::{
    settings_service_candidate_recent_project, settings_service_commit, settings_service_current,
    settings_service_new, settings_service_supported_languages, settings_service_validate,
};
mod path;
pub use path::PathService;
use path::{
    path_ref_parse, path_ref_to_logical, path_service_new, path_service_resolve,
    path_service_resolve_existing, path_service_resolve_write_target, path_service_set_root,
};
mod project;
#[cfg(test)]
use project::project_service_inspect_stl;
pub use project::{ProjectService, VariableDocumentCompletion, VariableDocumentRequest};
use project::{
    analysis_sequence_catalog, default_material, default_mesh_type, format_project_dimensions,
    mesh_type_catalog, project_error_message_key, project_service_activate_mesh_document,
    project_service_begin_asset_activation, project_service_begin_fill_settings_confirmation,
    project_service_begin_gate_location_settings_confirmation,
    project_service_begin_material_confirmation,
    project_service_begin_stl_import as begin_stl_import, project_service_begin_variable_document,
    project_service_cancel_asset_activation,
    project_service_cancel_stl_import as cancel_stl_import, project_service_create,
    project_service_current, project_service_deactivate_mesh_document,
    project_service_drain_asset_activations, project_service_drain_workspace_activations,
    project_service_execute, project_service_finish_fill_settings_confirmation,
    project_service_finish_gate_location_settings_confirmation,
    project_service_finish_material_confirmation, project_service_finish_stl_import,
    project_service_finish_variable_document, project_service_import_stl, project_service_imports,
    project_service_mesh_snapshot_for_import, project_service_new, project_service_open,
    project_service_plan_settings, project_service_release_mesh_document,
    project_service_resident_mesh_ids, project_service_save, project_service_set_analysis_sequence,
    project_service_variable_document_generation, project_service_workspace_activate,
    project_service_workspace_close, project_service_workspace_move,
    project_service_workspace_open_import, project_service_workspace_publish_import,
    project_service_workspace_reset, project_service_workspace_snapshot, stl_import_preview,
    validate_fill_settings_draft, validate_gate_location_settings_draft,
    validate_holding_profile_draft, variable_document_request_accepted, variable_document_wait,
};
mod project_operation;
pub use project_operation::{ProjectOperationCompletion, ProjectOperationRequest};
use project_operation::{
    project_operation_accepted, project_operation_wait, project_service_begin_operation,
    project_service_finish_operation,
};
mod display;
pub use display::MeshDisplayInput;
use display::{prepare_mesh_display, project_service_mesh_display_input};
mod mesh;
use mesh::{mesh_coordinates, mesh_validate_tet};

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
    /// CXX 共享枚举不支持自定义 derive，因此不派生 Debug。
    #[derive(Clone, PartialEq, Eq)]
    pub struct PathRef {
        pub kind: PathRootKind,
        pub relative: String,
    }

    /// Rust 工程服务返回的稳定状态快照；path/name 均为可往返 UTF-8。
    #[derive(Default, Clone, PartialEq, Eq)]
    pub struct ProjectSnapshot {
        pub path: String,
        pub name: String,
        pub revision: u64,
        pub dirty: bool,
    }

    /// 已验证主题的 QML 属性值；类型标签由 Rust schema 决定，Qt 仅转换 QVariant。
    #[derive(Clone, PartialEq)]
    pub struct ThemeToken {
        pub qml_name: String,
        pub value_type: String,
        pub value: String,
    }

    #[derive(Clone, PartialEq)]
    pub struct ThemeSnapshot {
        pub id: String,
        pub tokens: Vec<ThemeToken>,
    }

    /// Rust 校验后的应用偏好快照；窗口尺寸是 Qt 逻辑像素，不包含屏幕坐标。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UserSettingsSnapshot {
        pub schema_version: u32,
        pub theme_id: String,
        pub language: String,
        pub window_width: u32,
        pub window_height: u32,
        pub window_maximized: bool,
        pub recent_project_paths: Vec<String>,
    }

    /// 已导入网格的持久化元数据。尺寸拆成标量，避免 CXX shared struct
    /// 直接暴露 Rust 数组，便于 Qt 侧消费。
    #[derive(Default, Clone, PartialEq)]
    pub struct ProjectImport {
        pub record_version: u32,
        pub parser_version: u32,
        pub id: String,
        pub display_name: String,
        pub source_name: String,
        pub mesh_asset_path: String,
        pub format: String,
        pub mesh_type: String,
        pub units: String,
        pub show_import_log: bool,
        pub triangle_count: u64,
        pub size_x: f64,
        pub size_y: f64,
        pub size_z: f64,
    }

    /// Rust 工程文档工作区的批量显示快照；文档顺序与活动 ID 由 Rust 决定。
    #[derive(Default)]
    pub struct WorkspaceDocument {
        pub id: String,
        pub kind: String,
        pub state: String,
        pub title: String,
        pub message_key: String,
        pub diagnostic_code: String,
        pub diagnostic_category: String,
        pub diagnostic_detail: String,
        pub display_ordinal: u64,
        pub display_count: u64,
    }

    #[derive(Default)]
    pub struct WorkspaceSnapshot {
        pub documents: Vec<WorkspaceDocument>,
        pub active_document_id: String,
    }

    struct ChoiceDefinition {
        id: String,
        source_text: String,
    }

    struct MaterialProperty {
        source_text: String,
        value: String,
    }

    #[derive(Default)]
    struct MaterialDefinition {
        family_source_text: String,
        id: String,
        source_text: String,
        properties: Vec<MaterialProperty>,
    }

    struct HoldingProfilePoint {
        duration_seconds: f64,
        pressure_percent: f64,
    }

    struct FillSettings {
        mold_temperature_celsius: f64,
        melt_temperature_celsius: f64,
        flow_rate_cm3_per_second: f64,
        switch_over_volume_percent: f64,
        fiber_orientation: bool,
        crystallization: bool,
        holding_profile: Vec<HoldingProfilePoint>,
    }

    struct GateLocationSettings {
        machine_id: String,
        machine_source_text: String,
        mold_temperature_celsius: f64,
        melt_temperature_celsius: f64,
        algorithm_id: String,
        algorithm_source_text: String,
        number_of_gates: u32,
    }

    struct NumericConstraint {
        minimum: f64,
        maximum: f64,
        has_minimum: bool,
        has_maximum: bool,
        minimum_inclusive: bool,
        maximum_inclusive: bool,
        integer: bool,
        unit: String,
    }

    struct ProcessConstraints {
        temperature: NumericConstraint,
        flow_rate: NumericConstraint,
        switch_over_volume: NumericConstraint,
        holding_duration: NumericConstraint,
        holding_pressure: NumericConstraint,
        number_of_gates: NumericConstraint,
        holding_profile_max_points: u64,
    }

    struct ProcessDraftValidation {
        valid: bool,
        errors: Vec<String>,
    }

    struct PlanSettings {
        project_path: String,
        revision: u64,
        import_id: String,
        mesh_type: String,
        sequence_id: String,
        sequence_source_text: String,
        material_id: String,
        material_source_text: String,
        fill_settings: FillSettings,
        fill_settings_confirmed: bool,
        gate_location_settings: GateLocationSettings,
        gate_location_settings_confirmed: bool,
        process_constraints: ProcessConstraints,
    }

    #[derive(Default)]
    enum ProjectOperationKind {
        #[default]
        Create,
        Open,
        Save,
        AnalysisSequence,
    }
    #[derive(Default)]
    struct ProjectOperationInput {
        kind: ProjectOperationKind,
        path: String,
        name: String,
        revision: u64,
        import_id: String,
        sequence_id: String,
    }

    /// 源文件尚未复制时供导入弹窗展示的 STL 元数据。
    #[derive(Default, Clone, PartialEq)]
    pub struct StlImportPreview {
        pub expanded_mesh_bytes: u64,
        pub large_mesh_warning: bool,
        pub source_name: String,
        pub triangle_count: u64,
        pub size_x: f64,
        pub size_y: f64,
        pub size_z: f64,
    }

    /// 连续三角面坐标：每三个 f64 为一个点，每三个点为一片面；
    /// CXX Vec 独占缓冲区，调用返回后 C++ 复制到自身场景快照。
    #[derive(Default)]
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
    #[derive(Default)]
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
        pub error: ProjectDiagnostic,
        pub coordinates: Vec<f64>,
    }

    /// 所有工程结果遵循同一契约：code 为空表示成功，其余诊断字段为空；
    /// 失败时 value 为缺省占位，不可消费，调用方保留上一次有效状态。
    /// detail 保留原始上下文供诊断，界面仅按稳定 code 选择本地化摘要。
    #[derive(Default, Debug, Clone, PartialEq, Eq)]
    pub struct ProjectDiagnostic {
        pub code: String,
        pub category: String,
        pub detail: String,
    }

    /// 工程操作的快照结果。
    #[derive(Default)]
    pub struct ProjectSnapshotResult {
        pub value: ProjectSnapshot,
        pub error: ProjectDiagnostic,
    }

    /// 一次已提交导入的结果。
    #[derive(Default)]
    pub struct ProjectImportResult {
        pub value: ProjectImport,
        pub error: ProjectDiagnostic,
    }

    /// 当前工程导入记录列表；空列表也可为成功。
    #[derive(Default)]
    pub struct ProjectImportsResult {
        pub value: Vec<ProjectImport>,
        pub error: ProjectDiagnostic,
    }

    /// 已校验的变量快照条目；值使用文本传输，`value_type` 保留其领域类型。
    pub struct VariableValue {
        pub name: String,
        pub value_type: String,
        pub value: String,
    }

    /// DSL 源码诊断，范围为 UTF-8 字节偏移，行列从 1 开始。
    pub struct VariableDiagnostic {
        pub code: String,
        pub message: String,
        pub start_offset: u64,
        pub end_offset: u64,
        pub line: u64,
        pub column: u64,
    }

    /// 变量文件载入或保存的结果；失败时调用方保留之前的值快照。
    #[derive(Default)]
    pub struct VariableDocumentResult {
        pub source: String,
        pub generation: u64,
        pub revision: u64,
        pub values: Vec<VariableValue>,
        pub diagnostics: Vec<VariableDiagnostic>,
        pub error: ProjectDiagnostic,
    }

    /// 只读预检的运行期请求编号；失败 value 不可消费。
    #[derive(Default)]
    pub struct StlPreviewRequestResult {
        pub value: u64,
        pub error: ProjectDiagnostic,
    }

    /// ready=false 且诊断为空表示尚未完成；只有 ready=true 才能消费 value。
    #[derive(Default)]
    pub struct StlPreviewPoll {
        pub ready: bool,
        pub value: StlImportPreview,
        pub error: ProjectDiagnostic,
    }

    /// 确认导入的终态轮询；ready=false 且诊断为空表示仍在写入。
    #[derive(Default)]
    pub struct StlImportPoll {
        pub ready: bool,
        pub value: ProjectImport,
        pub error: ProjectDiagnostic,
    }

    /// 内置材料摘要结果。
    #[derive(Default)]
    pub struct MaterialDefinitionResult {
        pub value: MaterialDefinition,
        pub error: ProjectDiagnostic,
    }

    /// 确认请求 / 轮询结果；成功 value=false 分别表示无需写入 / 尚未完成。
    #[derive(Default)]
    pub struct ProjectConfirmationResult {
        pub value: bool,
        pub error: ProjectDiagnostic,
    }

    /// 驻留 Mesh 的显示快照结果。
    #[derive(Default)]
    pub struct SurfaceMeshSnapshotResult {
        pub value: SurfaceMeshSnapshot,
        pub error: ProjectDiagnostic,
    }

    /// 资产激活请求的相关性句柄结果。
    #[derive(Default)]
    pub struct ActivationAttemptResult {
        pub value: ActivationAttempt,
        pub error: ProjectDiagnostic,
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

        /// Rust 拥有用户偏好 schema、默认值和候选校验；Qt adapter 负责持久化。
        type UserSettingsService;

        fn settings_service_new() -> Box<UserSettingsService>;
        fn settings_service_current(service: &UserSettingsService) -> UserSettingsSnapshot;
        fn settings_service_supported_languages() -> Vec<String>;
        fn settings_service_candidate_recent_project(
            service: &UserSettingsService,
            path: String,
        ) -> Result<UserSettingsSnapshot>;
        fn settings_service_validate(
            service: &UserSettingsService,
            candidate: &UserSettingsSnapshot,
        ) -> Result<UserSettingsSnapshot>;
        fn settings_service_commit(
            service: &mut UserSettingsService,
            candidate: &UserSettingsSnapshot,
        ) -> Result<UserSettingsSnapshot>;

        /// 主题基线、主题定义与 token 校验由 Rust 服务拥有；此服务不读写设置。
        type ThemeService;

        fn theme_service_new(
            gui_source: String,
            theme_ids: Vec<String>,
            theme_sources: Vec<String>,
            default_theme_id: String,
        ) -> Result<Box<ThemeService>>;
        fn theme_service_theme_ids(service: &ThemeService) -> Vec<String>;
        fn theme_service_current(service: &ThemeService) -> ThemeSnapshot;
        fn theme_service_select(service: &mut ThemeService, id: String) -> Result<ThemeSnapshot>;

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

        type ProjectOperationRequest;
        type ProjectOperationCompletion;
        type MeshDisplayInput;
        fn project_service_begin_operation(
            service: &mut ProjectService,
            input: ProjectOperationInput,
        ) -> Box<ProjectOperationRequest>;
        fn project_operation_accepted(request: &ProjectOperationRequest) -> bool;
        fn project_operation_wait(
            request: Box<ProjectOperationRequest>,
        ) -> Box<ProjectOperationCompletion>;
        fn project_service_finish_operation(
            service: &mut ProjectService,
            completion: Box<ProjectOperationCompletion>,
        ) -> ProjectSnapshotResult;
        fn project_service_mesh_display_input(
            service: &ProjectService,
            import_id: &str,
        ) -> Box<MeshDisplayInput>;
        fn prepare_mesh_display(input: Box<MeshDisplayInput>) -> SurfaceMeshSnapshotResult;

        fn project_service_new() -> Box<ProjectService>;
        #[rust_name = "project_service_create_response"]
        fn project_service_create(
            service: &mut ProjectService,
            location: String,
            name: String,
        ) -> ProjectSnapshotResult;
        #[rust_name = "project_service_open_response"]
        fn project_service_open(
            service: &mut ProjectService,
            path: String,
        ) -> ProjectSnapshotResult;
        #[rust_name = "project_service_save_response"]
        fn project_service_save(service: &mut ProjectService) -> ProjectSnapshotResult;
        fn project_service_variable_document_generation(service: &ProjectService) -> u64;
        type VariableDocumentRequest;
        type VariableDocumentCompletion;
        /// 发起有界后台读写；save=false 忽略 source。请求独占所有权，可移交等待线程。
        fn project_service_begin_variable_document(
            service: &mut ProjectService,
            expected_generation: u64,
            expected_revision: u64,
            source: String,
            save: bool,
        ) -> Box<VariableDocumentRequest>;
        fn variable_document_request_accepted(request: &VariableDocumentRequest) -> bool;
        /// 仅在独立等待线程调用；拒绝准入的请求立即返回，不借用服务。
        fn variable_document_wait(
            request: Box<VariableDocumentRequest>,
        ) -> Box<VariableDocumentCompletion>;
        /// 仅在服务拥有线程消费完成结果并映射 DTO。
        fn project_service_finish_variable_document(
            service: &mut ProjectService,
            completion: Box<VariableDocumentCompletion>,
        ) -> VariableDocumentResult;
        #[rust_name = "project_service_execute_response"]
        fn project_service_execute(
            service: &mut ProjectService,
            command: ProjectCommand,
        ) -> ProjectSnapshotResult;
        #[rust_name = "project_service_current_response"]
        fn project_service_current(service: &ProjectService) -> ProjectSnapshotResult;
        #[rust_name = "project_service_import_stl_response"]
        fn project_service_import_stl(
            service: &mut ProjectService,
            source: String,
            mesh_type: String,
            units: String,
            show_import_log: bool,
        ) -> ProjectImportResult;
        #[rust_name = "project_service_imports_response"]
        fn project_service_imports(service: &ProjectService) -> ProjectImportsResult;
        fn project_service_workspace_snapshot(service: &ProjectService) -> WorkspaceSnapshot;
        fn project_service_workspace_reset(service: &mut ProjectService, welcome_title: String);
        fn project_service_workspace_open_import(
            service: &mut ProjectService,
            import_id: &str,
            title: &str,
        );
        fn project_service_workspace_publish_import(
            service: &mut ProjectService,
            import_id: &str,
            title: &str,
        );
        fn project_service_workspace_activate(service: &mut ProjectService, document_id: &str);
        fn project_service_workspace_close(service: &mut ProjectService, document_id: &str);
        fn project_service_workspace_move(service: &mut ProjectService, from: u64, to: u64);
        fn project_error_message_key(code: &str) -> String;
        fn format_project_dimensions(size_x: f64, size_y: f64, size_z: f64, unit: &str) -> String;
        fn analysis_sequence_catalog() -> Vec<ChoiceDefinition>;
        fn mesh_type_catalog() -> Vec<ChoiceDefinition>;
        fn default_mesh_type() -> String;
        #[rust_name = "default_material_response"]
        fn default_material() -> MaterialDefinitionResult;
        #[rust_name = "project_service_begin_material_confirmation_response"]
        fn project_service_begin_material_confirmation(
            service: &mut ProjectService,
            project_path: &str,
            revision: u64,
            import_id: &str,
            material_id: &str,
        ) -> ProjectConfirmationResult;
        #[rust_name = "project_service_finish_material_confirmation_response"]
        fn project_service_finish_material_confirmation(
            service: &mut ProjectService,
        ) -> ProjectConfirmationResult;
        #[rust_name = "project_service_begin_fill_settings_confirmation_response"]
        fn project_service_begin_fill_settings_confirmation(
            service: &mut ProjectService,
            project_path: &str,
            revision: u64,
            import_id: &str,
            settings: FillSettings,
        ) -> ProjectConfirmationResult;
        #[rust_name = "project_service_finish_fill_settings_confirmation_response"]
        fn project_service_finish_fill_settings_confirmation(
            service: &mut ProjectService,
        ) -> ProjectConfirmationResult;
        #[rust_name = "project_service_begin_gate_location_settings_confirmation_response"]
        fn project_service_begin_gate_location_settings_confirmation(
            service: &mut ProjectService,
            project_path: &str,
            revision: u64,
            import_id: &str,
            settings: GateLocationSettings,
        ) -> ProjectConfirmationResult;
        #[rust_name = "project_service_finish_gate_location_settings_confirmation_response"]
        fn project_service_finish_gate_location_settings_confirmation(
            service: &mut ProjectService,
        ) -> ProjectConfirmationResult;
        fn project_service_plan_settings(
            service: &ProjectService,
            preferred_import_id: &str,
        ) -> PlanSettings;
        fn validate_fill_settings_draft(settings: FillSettings) -> ProcessDraftValidation;
        fn validate_holding_profile_draft(
            holding_profile: Vec<HoldingProfilePoint>,
        ) -> ProcessDraftValidation;
        fn validate_gate_location_settings_draft(
            mold_temperature_celsius: f64,
            melt_temperature_celsius: f64,
            number_of_gates: f64,
            machine_id: String,
        ) -> ProcessDraftValidation;
        #[rust_name = "project_service_set_analysis_sequence_response"]
        fn project_service_set_analysis_sequence(
            service: &mut ProjectService,
            project_path: &str,
            revision: u64,
            import_id: &str,
            sequence_id: &str,
        ) -> ProjectSnapshotResult;

        #[rust_name = "project_service_begin_stl_preview_response"]
        fn project_service_begin_stl_preview(
            service: &mut ProjectService,
            source: String,
        ) -> StlPreviewRequestResult;
        #[rust_name = "project_service_finish_stl_preview_response"]
        fn project_service_finish_stl_preview(
            service: &mut ProjectService,
            request: u64,
        ) -> StlPreviewPoll;
        fn project_service_cancel_stl_preview(service: &mut ProjectService, request: u64) -> bool;
        #[rust_name = "project_service_begin_stl_import_response"]
        fn project_service_begin_stl_import(
            service: &mut ProjectService,
            source: String,
            mesh_type: String,
            units: String,
            show_import_log: bool,
        ) -> StlPreviewRequestResult;
        #[rust_name = "project_service_finish_stl_import_response"]
        fn project_service_finish_stl_import(
            service: &mut ProjectService,
            request: u64,
        ) -> StlImportPoll;
        #[rust_name = "project_service_cancel_stl_import_response"]
        fn project_service_cancel_stl_import(service: &mut ProjectService, request: u64) -> bool;
        /// 已驻留 Mesh 的一次性显示快照；只由文档激活路径调用，不用于逐帧读取。
        #[rust_name = "project_service_mesh_snapshot_for_import_response"]
        fn project_service_mesh_snapshot_for_import(
            service: &ProjectService,
            import_id: &str,
        ) -> SurfaceMeshSnapshotResult;
        /// 活动页签 pin/unpin 与关闭释放属于粗粒度文档状态变化。
        fn project_service_activate_mesh_document(
            service: &mut ProjectService,
            import_id: &str,
        ) -> bool;
        fn project_service_deactivate_mesh_document(service: &mut ProjectService);
        fn project_service_release_mesh_document(service: &mut ProjectService, import_id: &str);
        fn project_service_resident_mesh_ids(service: &ProjectService) -> Vec<String>;

        /// 只读资产激活（073 首个 FSM 消费者）：按稳定 ImportRecord ID
        /// 异步读取并解析已提交 STL；结果只在会话仍有效时发布。
        #[rust_name = "project_service_begin_asset_activation_response"]
        fn project_service_begin_asset_activation(
            service: &mut ProjectService,
            import_id: &str,
        ) -> ActivationAttemptResult;
        fn project_service_cancel_asset_activation(
            service: &mut ProjectService,
            attempt: u64,
        ) -> bool;
        fn project_service_drain_asset_activations(
            service: &mut ProjectService,
        ) -> Vec<ActivationOutcome>;
        fn project_service_drain_workspace_activations(service: &mut ProjectService);
        fn mesh_validate_tet(data: TetMeshData) -> TetMeshValidation;

        /// QML 服务销毁、事件循环退出后等待已接受写入；禁止在写入 worker 调用。
        fn finish_background_writes();

        /// 崩溃信号处理器安装（任务 047，panta_foundation::crash 的 FFI 面）：
        /// 返回日志路径；log_dir 为空时用系统临时目录。
        fn install_crash_handler(log_dir: &str) -> Result<String>;
    }
}

pub use bridge::{FfiRequest, FfiResponse};
