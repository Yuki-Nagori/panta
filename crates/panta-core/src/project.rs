//! 最小工程 application service（任务 057）。
//!
//! 工程目录由宿主选择，工程模型和清单事务由 Rust 持有。Qt/QML 只通过
//! CXX adapter 传递 UTF-8 路径、名称和命令，不直接读写清单文件。

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// 当前范例工程清单的 schema 版本。
pub const PROJECT_SCHEMA_VERSION: u32 = 1;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectState {
    path: PathBuf,
    name: String,
    revision: u64,
    dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProjectManifest {
    schema: u32,
    name: String,
    revision: u64,
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
#[derive(Debug, Default)]
pub struct ProjectService {
    current: Option<ProjectState>,
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
        };
        if let Err(error) = write_manifest(&state) {
            if let Some(project_root) = state.path.parent() {
                let _ = fs::remove_dir_all(project_root);
            }
            return Err(error);
        }
        self.current = Some(state);
        self.snapshot()
    }

    /// 打开既有 `.panta` 主文件并读取清单；不改变 cwd，也不接受其他扩展名。
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
        if manifest.schema != PROJECT_SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedSchema(manifest.schema));
        }
        validate_name(&manifest.name)?;
        self.current = Some(ProjectState {
            path: path.to_path_buf(),
            name: manifest.name,
            revision: manifest.revision,
            dirty: false,
        });
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

fn write_manifest(state: &ProjectState) -> Result<(), ProjectError> {
    let manifest = ProjectManifest {
        schema: PROJECT_SCHEMA_VERSION,
        name: state.name.clone(),
        revision: state.revision,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ProjectError::Io(format!("serialize manifest: {error}")))?;
    let target = &state.path;
    let temporary = target.with_extension("panta.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| ProjectError::Io(format!("{}: {error}", temporary.display())))?;

    match fs::rename(&temporary, target) {
        Ok(()) => {}
        Err(error) => {
            // Windows cannot replace an existing file with rename. Keep the same
            // temp-file protocol and use the platform replacement fallback there.
            #[cfg(windows)]
            {
                let _ = error;
                fs::remove_file(&target).map_err(|remove_error| {
                    ProjectError::Io(format!("{}: {remove_error}", target.display()))
                })?;
                fs::rename(&temporary, &target).map_err(|rename_error| {
                    ProjectError::Io(format!("{}: {rename_error}", target.display()))
                })?;
            }
            #[cfg(not(windows))]
            {
                let _ = fs::remove_file(&temporary);
                return Err(ProjectError::Io(format!("{}: {error}", target.display())));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PROJECT_SCHEMA_VERSION, ProjectCommand, ProjectError, ProjectService};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("panta-project-{}-{}", std::process::id(), id));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root)?;
            Ok(Self { root })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn create_command_save_open_and_model_command_round_trip()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        let created = service.create(&fixture.root, "Demo")?;
        assert_eq!(created.name, "Demo");
        assert!(!created.dirty);
        assert_eq!(created.path, fixture.root.join("Demo/Demo.panta"));
        assert!(created.path.is_file());

        let changed = service.execute(ProjectCommand::Rename {
            name: "Renamed".to_owned(),
        })?;
        assert_eq!(changed.name, "Renamed");
        assert!(changed.dirty);
        let saved = service.save()?;
        assert!(!saved.dirty);

        let mut reopened = ProjectService::new();
        let opened = reopened.open(&created.path)?;
        assert_eq!(opened.name, "Renamed");
        assert_eq!(opened.revision, 1);
        assert!(!opened.dirty);
        Ok(())
    }

    #[test]
    fn rejects_invalid_names_and_existing_targets() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        for name in [
            "",
            "../escape",
            "CON",
            "bad/child",
            "trailing.",
            "Demo.panta",
        ] {
            let error = match service.create(&fixture.root, name) {
                Ok(snapshot) => panic!("invalid name unexpectedly created: {snapshot:?}"),
                Err(error) => error,
            };
            assert!(matches!(
                error,
                ProjectError::EmptyName | ProjectError::InvalidName(_)
            ));
        }
        service.create(&fixture.root, "Demo")?;
        let error = match service.create(&fixture.root, "Demo") {
            Ok(snapshot) => panic!("existing target unexpectedly created: {snapshot:?}"),
            Err(error) => error,
        };
        assert!(matches!(error, ProjectError::AlreadyExists(_)));
        Ok(())
    }

    #[test]
    fn rejects_unsupported_manifest_schema() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let project = fixture.root.join("Demo");
        fs::create_dir_all(&project)?;
        let project_file = project.join("Demo.panta");
        fs::write(
            &project_file,
            format!(
                r#"{{"schema":{},"name":"Demo","revision":0}}"#,
                PROJECT_SCHEMA_VERSION + 1
            ),
        )?;
        let error = match ProjectService::new().open(&project_file) {
            Ok(snapshot) => panic!("unsupported schema opened: {snapshot:?}"),
            Err(error) => error,
        };
        assert!(matches!(error, ProjectError::UnsupportedSchema(_)));
        Ok(())
    }

    #[test]
    fn accepts_only_panta_project_files() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let project = fixture.root.join("Demo");
        fs::create_dir_all(&project)?;
        let other_file = project.join("Demo.json");
        fs::write(other_file, br#"{"schema":1,"name":"Demo","revision":0}"#)?;
        let error = match ProjectService::new().open(&project.join("Demo.json")) {
            Ok(snapshot) => panic!("non-panta file opened: {snapshot:?}"),
            Err(error) => error,
        };
        assert!(matches!(error, ProjectError::InvalidFile(_)));
        Ok(())
    }
}
