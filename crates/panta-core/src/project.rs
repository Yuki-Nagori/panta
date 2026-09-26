//! 工程 application service（任务 057、063）。
//!
//! 工程目录由宿主选择，工程模型和清单事务由 Rust 持有。Qt/QML 只通过
//! CXX adapter 传递 UTF-8 路径、名称和命令，不直接读写清单文件。

use panta_import::StlImportSession;
pub use panta_import::{
    IMPORT_RECORD_VERSION, ImportRecord, STL_IMPORT_PARSER_VERSION, StlImportPreview,
};
use panta_mesh::SurfaceMesh;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

mod import;
mod storage;
use std::fs;
use std::path::{Path, PathBuf};
use storage::write_manifest;

pub use crate::fsm::open_saved_stl::{ActivationAttempt, Outcome, OutcomeKind};

/// 当前范例工程清单的 schema 版本。
pub const PROJECT_SCHEMA_VERSION: u32 = 2;
const MIN_SUPPORTED_PROJECT_SCHEMA_VERSION: u32 = 1;
const PROJECT_FILE_EXTENSION: &str = "panta";

/// 可被工程模型接受的命令；扩展命令时必须增加对应验证和持久化测试。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectCommand {
    Rename { name: String },
}

/// 提供给 UI/FFI 的工程状态快照，不暴露内部路径或 serde 类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshot {
    pub path: PathBuf,
    pub name: String,
    pub revision: u64,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ProjectState {
    path: PathBuf,
    name: String,
    revision: u64,
    dirty: bool,
    imports: Vec<ImportRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ProjectManifest {
    schema: u32,
    name: String,
    revision: u64,
    #[serde(default)]
    imports: Vec<ImportRecord>,
}

/// 工程 service 的可恢复错误；`Display` 的前缀是跨语言稳定错误码。
#[derive(Debug)]
pub enum ProjectError {
    EmptyName,
    InvalidName(String),
    LocationEmpty,
    LocationNotAbsolute(String),
    LocationCreateFailed(String),
    FileMissing(String),
    InvalidFile(String),
    AlreadyExists(String),
    ManifestInvalid(String),
    UnsupportedSchema(u32),
    NoProject,
    CommandInvalid(String),
    ImportFileMissing(String),
    ImportInvalidFile(String),
    ImportUnsupportedMeshType(String),
    ImportUnsupportedUnits(String),
    ImportParseFailed(String),
    ImportSourceChanged(String),
    ImportAssetCopyFailed(String),
    ImportRecordMissing(String),
    Io(String),
}

impl ProjectError {
    fn code(&self) -> &'static str {
        match self {
            Self::EmptyName => "project.empty_name",
            Self::InvalidName(_) => "project.invalid_name",
            Self::LocationEmpty => "project.location_empty",
            Self::LocationNotAbsolute(_) => "project.location_not_absolute",
            Self::LocationCreateFailed(_) => "project.location_create_failed",
            Self::FileMissing(_) => "project.file_missing",
            Self::InvalidFile(_) => "project.invalid_file",
            Self::AlreadyExists(_) => "project.already_exists",
            Self::ManifestInvalid(_) => "project.manifest_invalid",
            Self::UnsupportedSchema(_) => "project.unsupported_schema",
            Self::NoProject => "project.no_project",
            Self::CommandInvalid(_) => "project.command_invalid",
            Self::ImportFileMissing(_) => "project.import_file_missing",
            Self::ImportInvalidFile(_) => "project.import_invalid_file",
            Self::ImportUnsupportedMeshType(_) => "project.import_unsupported_mesh_type",
            Self::ImportUnsupportedUnits(_) => "project.import_unsupported_units",
            Self::ImportParseFailed(_) => "project.import_parse_failed",
            Self::ImportSourceChanged(_) => "project.import_source_changed",
            Self::ImportAssetCopyFailed(_) => "project.import_asset_copy_failed",
            Self::ImportRecordMissing(_) => "project.import_record_missing",
            Self::Io(_) => "project.io",
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::EmptyName | Self::LocationEmpty | Self::NoProject => String::new(),
            Self::InvalidName(name)
            | Self::LocationNotAbsolute(name)
            | Self::LocationCreateFailed(name)
            | Self::FileMissing(name)
            | Self::InvalidFile(name)
            | Self::AlreadyExists(name)
            | Self::ManifestInvalid(name)
            | Self::CommandInvalid(name)
            | Self::ImportFileMissing(name)
            | Self::ImportInvalidFile(name)
            | Self::ImportUnsupportedMeshType(name)
            | Self::ImportUnsupportedUnits(name)
            | Self::ImportParseFailed(name)
            | Self::ImportSourceChanged(name)
            | Self::ImportAssetCopyFailed(name)
            | Self::ImportRecordMissing(name)
            | Self::Io(name) => name.clone(),
            Self::UnsupportedSchema(schema) => schema.to_string(),
        }
    }
}

impl Display for ProjectError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let detail = self.detail();
        if detail.is_empty() {
            write!(formatter, "{}", self.code())
        } else {
            write!(formatter, "{}: {detail}", self.code())
        }
    }
}

impl std::error::Error for ProjectError {}

/// 当前工程 service。其所有状态都由 Rust 拥有，C++ 仅持有 opaque Box。
/// `path` 是稳定的 `.panta` 主文件身份；重命名命令只更新清单名称，不移动
/// 工程目录或文件，避免把资产路径变更混入最小模型命令。
///
/// `generation` 是会话级运行期计数（create/open 各递增一次，不持久化），
/// 与 `ImportRecord.id` / attempt 一起构成只读 FSM 激活的相关性键（073/080）。
#[derive(Debug, Default)]
pub struct ProjectService {
    current: Option<ProjectState>,
    import_session: StlImportSession,
    current_mesh: Option<SurfaceMesh>,
    activation: Arc<crate::fsm::open_saved_stl::ActivationCoordinator>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建 `location/name/name.panta` 目录包和初始清单；目标已存在时拒绝覆盖。
    pub fn create(&mut self, location: &Path, name: &str) -> Result<ProjectSnapshot, ProjectError> {
        validate_name(name)?;
        if location.as_os_str().is_empty() {
            return Err(ProjectError::LocationEmpty);
        }
        if !location.is_absolute() {
            return Err(ProjectError::LocationNotAbsolute(
                location.display().to_string(),
            ));
        }
        if !location.exists() {
            fs::create_dir_all(location).map_err(|error| {
                ProjectError::LocationCreateFailed(format!("{}: {error}", location.display()))
            })?;
        }
        if !location.is_dir() {
            return Err(ProjectError::LocationCreateFailed(
                location.display().to_string(),
            ));
        }

        let project_root = location.join(name);
        if project_root.exists() {
            return Err(ProjectError::AlreadyExists(
                project_root.display().to_string(),
            ));
        }
        fs::create_dir(&project_root)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", project_root.display())))?;

        let project_path = project_root.join(format!("{name}.{PROJECT_FILE_EXTENSION}"));

        let state = ProjectState {
            path: project_path,
            name: name.to_owned(),
            revision: 0,
            dirty: false,
            imports: Vec::new(),
        };
        if let Err(error) = write_manifest(&state) {
            if let Some(project_root) = state.path.parent() {
                let _ = fs::remove_dir_all(project_root);
            }
            return Err(error);
        }
        self.current = Some(state);
        self.current_mesh = None;
        self.activation.advance_generation();
        self.snapshot()
    }

    /// 打开既有 `.panta` 主文件并读取清单；不改变 cwd，也不接受其他扩展名。
    ///
    /// 打开只读清单，不解析任何导入资产：已保存 STL 的网格由
    /// [`Self::begin_asset_activation`] 异步按需重建（080 文档页签），
    /// 避免保留同步与异步两条重复加载路径。
    pub fn open(&mut self, path: &Path) -> Result<ProjectSnapshot, ProjectError> {
        if !path.is_absolute() || !path.is_file() {
            return Err(ProjectError::FileMissing(path.display().to_string()));
        }
        let is_panta_file = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case(PROJECT_FILE_EXTENSION));
        if !is_panta_file {
            return Err(ProjectError::InvalidFile(path.display().to_string()));
        }
        let bytes = fs::read(path)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", path.display())))?;
        let manifest: ProjectManifest = serde_json::from_slice(&bytes).map_err(|error| {
            ProjectError::ManifestInvalid(format!("{}: {error}", path.display()))
        })?;
        if !(MIN_SUPPORTED_PROJECT_SCHEMA_VERSION..=PROJECT_SCHEMA_VERSION)
            .contains(&manifest.schema)
        {
            return Err(ProjectError::UnsupportedSchema(manifest.schema));
        }
        validate_name(&manifest.name)?;
        self.current = Some(ProjectState {
            path: path.to_path_buf(),
            name: manifest.name,
            revision: manifest.revision,
            dirty: false,
            imports: manifest.imports,
        });
        self.current_mesh = None;
        self.import_session.clear();
        self.activation.advance_generation();
        self.snapshot()
    }

    /// 将当前模型以同目录临时文件写入后替换 `.panta` 主文件。
    pub fn save(&mut self) -> Result<ProjectSnapshot, ProjectError> {
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        write_manifest(state)?;
        state.dirty = false;
        self.snapshot()
    }

    /// 执行一个有校验的模型命令，并标记工程 dirty。
    pub fn execute(&mut self, command: ProjectCommand) -> Result<ProjectSnapshot, ProjectError> {
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        match command {
            ProjectCommand::Rename { name } => {
                validate_name(&name)?;
                if name == state.name {
                    return Err(ProjectError::CommandInvalid("name is unchanged".to_owned()));
                }
                state.name = name;
                state.revision = state.revision.saturating_add(1);
                state.dirty = true;
            }
        }
        self.snapshot()
    }

    pub fn current(&self) -> Result<ProjectSnapshot, ProjectError> {
        self.snapshot()
    }

    /// 返回当前工程中已持久化导入记录的副本。
    pub fn imports(&self) -> Result<Vec<ImportRecord>, ProjectError> {
        self.current
            .as_ref()
            .map(|state| state.imports.clone())
            .ok_or(ProjectError::NoProject)
    }

    /// 按 `ImportRecord.id` 发起只读资产激活（073 首个 FSM 消费者）。
    ///
    /// 记录不存在或无工程时同步失败；同会话同记录的在飞 attempt 去重复用。
    /// 结果经 [`Self::drain_asset_activations`] 拉取；加载不修改
    /// revision / dirty，也不写任何存储。
    pub fn begin_asset_activation(
        &mut self,
        import_id: &str,
    ) -> Result<ActivationAttempt, ProjectError> {
        let state = self.current.as_ref().ok_or(ProjectError::NoProject)?;
        let record = state
            .imports
            .iter()
            .find(|record| record.id == import_id)
            .ok_or_else(|| ProjectError::ImportRecordMissing(import_id.to_owned()))?;
        let root = state
            .path
            .parent()
            .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
        let asset_path = resolve_asset_path(root, &record.asset)?;
        Ok(self.activation.begin(
            &record.id,
            crate::fsm::open_saved_stl::ActivationRequest {
                asset_path,
                units: record.units.clone(),
            },
        ))
    }

    /// 请求取消一次在飞激活；`Cancelled` 终态由 worker 在安全检查点确认。
    pub fn cancel_asset_activation(&mut self, attempt: u64) -> bool {
        self.activation.cancel(attempt)
    }

    /// 拉取当前会话已完成的激活结果；旧代次结果已在 Rust 侧释放。
    pub fn drain_asset_activations(&mut self) -> Vec<Outcome> {
        self.activation.drain()
    }

    fn snapshot(&self) -> Result<ProjectSnapshot, ProjectError> {
        self.current
            .as_ref()
            .map(|state| ProjectSnapshot {
                path: state.path.clone(),
                name: state.name.clone(),
                revision: state.revision,
                dirty: state.dirty,
            })
            .ok_or(ProjectError::NoProject)
    }
}

/// 校验清单内的工程相对资产引用并解析为绝对路径；拒绝绝对路径与
/// `..` 等非普通分量，防止清单注入越出工程包目录。
fn resolve_asset_path(root: &Path, relative: &str) -> Result<PathBuf, ProjectError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(ProjectError::ManifestInvalid(
            "invalid import asset path".to_owned(),
        ));
    }
    Ok(root.join(relative_path))
}

fn validate_name(name: &str) -> Result<(), ProjectError> {
    if name.is_empty() {
        return Err(ProjectError::EmptyName);
    }
    if name.len() > 255
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.ends_with('.')
        || name.ends_with(' ')
        || name.to_ascii_lowercase().ends_with(".panta")
        || name.chars().any(|character| character.is_control())
    {
        return Err(ProjectError::InvalidName(name.to_owned()));
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.contains(&stem.as_str()) {
        return Err(ProjectError::InvalidName(name.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ProjectError, validate_name};

    #[test]
    fn project_name_rules_accept_unicode_and_reject_platform_reserved_stems() {
        assert!(validate_name("模拟件.v1").is_ok());
        assert!(matches!(
            validate_name("COM1.log"),
            Err(ProjectError::InvalidName(_))
        ));
        assert!(matches!(
            validate_name("name. "),
            Err(ProjectError::InvalidName(_))
        ));
    }
}
