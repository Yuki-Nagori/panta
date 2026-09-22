//! 工程 application service（任务 057、063）。
//!
//! 工程目录由宿主选择，工程模型和清单事务由 Rust 持有。Qt/QML 只通过
//! CXX adapter 传递 UTF-8 路径、名称和命令，不直接读写清单文件。

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// 当前范例工程清单的 schema 版本。
pub const PROJECT_SCHEMA_VERSION: u32 = 2;
pub const IMPORT_RECORD_VERSION: u32 = 1;
pub const STL_IMPORT_PARSER_VERSION: u32 = 1;
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

/// 已复制到工程包中的资产及其导入解释选项，带有记录和解析器版本。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportRecord {
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
    pub dimensions: [f64; 3],
}

/// 在复制资产前供导入对话框展示的 STL 元数据。
#[derive(Debug, Clone, PartialEq)]
pub struct StlImportPreview {
    pub source_name: String,
    pub triangle_count: u64,
    pub dimensions: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StlSummary {
    triangle_count: u64,
    dimensions: [f64; 3],
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
    ImportAssetCopyFailed(String),
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
            Self::ImportAssetCopyFailed(_) => "project.import_asset_copy_failed",
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
            | Self::ImportAssetCopyFailed(name)
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
            imports: Vec::new(),
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

    /// 校验、复制并持久化当前工程包中的一次 STL 导入。
    /// 返回成功前先写入清单，重新打开工程时无需依赖后续显式保存即可恢复。
    pub fn import_stl(
        &mut self,
        source: &Path,
        mesh_type: &str,
        units: &str,
        show_import_log: bool,
    ) -> Result<ImportRecord, ProjectError> {
        if self.current.is_none() {
            return Err(ProjectError::NoProject);
        }
        let preview = self.inspect_stl(source)?;
        let mesh_type = normalize_mesh_type(mesh_type)?;
        let units = normalize_units(units)?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        let project_root = state
            .path
            .parent()
            .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
        let asset_dir = project_root.join("assets").join("imports");
        fs::create_dir_all(&asset_dir).map_err(|error| {
            ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_dir.display()))
        })?;
        let import_number = state.imports.len() + 1;
        let asset_name = format!("{import_number:04}-{}", preview.source_name);
        let asset_path = asset_dir.join(&asset_name);
        if let Err(error) = fs::copy(source, &asset_path) {
            let _ = fs::remove_file(&asset_path);
            return Err(ProjectError::ImportAssetCopyFailed(format!(
                "{}: {error}",
                source.display()
            )));
        }

        let record = ImportRecord {
            record_version: IMPORT_RECORD_VERSION,
            parser_version: STL_IMPORT_PARSER_VERSION,
            id: format!("import-{import_number}"),
            source_name: preview.source_name,
            asset: format!("assets/imports/{asset_name}"),
            format: "stl".to_owned(),
            mesh_type: mesh_type.to_owned(),
            units: units.to_owned(),
            show_import_log,
            triangle_count: preview.triangle_count,
            dimensions: preview.dimensions,
        };
        let previous_revision = state.revision;
        state.imports.push(record.clone());
        state.revision = state.revision.saturating_add(1);
        state.dirty = false;
        if let Err(error) = write_manifest(state) {
            state.imports.pop();
            state.revision = previous_revision;
            let _ = fs::remove_file(&asset_path);
            return Err(error);
        }
        Ok(record)
    }

    /// Parse STL metadata without requiring an open project or mutating disk.
    pub fn inspect_stl(&self, source: &Path) -> Result<StlImportPreview, ProjectError> {
        let summary = parse_stl(source)?;
        let source_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ProjectError::ImportInvalidFile(source.display().to_string()))?
            .to_owned();
        Ok(StlImportPreview {
            source_name,
            triangle_count: summary.triangle_count,
            dimensions: summary.dimensions,
        })
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
        imports: state.imports.clone(),
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

fn normalize_mesh_type(value: &str) -> Result<&'static str, ProjectError> {
    match value {
        "midplane" => Ok("midplane"),
        "dual-domain" => Ok("dual-domain"),
        "solid-3d" => Ok("solid-3d"),
        other => Err(ProjectError::ImportUnsupportedMeshType(other.to_owned())),
    }
}

fn normalize_units(value: &str) -> Result<&'static str, ProjectError> {
    match value {
        "millimeters" => Ok("millimeters"),
        "centimeters" => Ok("centimeters"),
        "inches" => Ok("inches"),
        other => Err(ProjectError::ImportUnsupportedUnits(other.to_owned())),
    }
}

fn parse_stl(path: &Path) -> Result<StlSummary, ProjectError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(ProjectError::ImportFileMissing(path.display().to_string()));
    }
    let is_stl = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("stl"));
    if !is_stl {
        return Err(ProjectError::ImportInvalidFile(path.display().to_string()));
    }
    let bytes = fs::read(path)
        .map_err(|error| ProjectError::ImportParseFailed(format!("{}: {error}", path.display())))?;
    if let Some(summary) = parse_binary_stl(&bytes) {
        return Ok(summary);
    }
    parse_ascii_stl(&bytes).ok_or_else(|| {
        ProjectError::ImportParseFailed(format!("{}: invalid STL data", path.display()))
    })
}

fn parse_binary_stl(bytes: &[u8]) -> Option<StlSummary> {
    if bytes.len() < 84 {
        return None;
    }
    let triangle_count = u32::from_le_bytes(bytes[80..84].try_into().ok()?) as usize;
    let expected_size = 84usize.checked_add(triangle_count.checked_mul(50)?)?;
    if triangle_count == 0 || expected_size != bytes.len() {
        return None;
    }
    let mut bounds = Bounds::default();
    for triangle in 0..triangle_count {
        let start = 84 + triangle * 50 + 12;
        for vertex in 0..3 {
            let offset = start + vertex * 12;
            let x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as f64;
            let y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?) as f64;
            let z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().ok()?) as f64;
            bounds.add(x, y, z)?;
        }
    }
    Some(StlSummary {
        triangle_count: triangle_count as u64,
        dimensions: bounds.dimensions(),
    })
}

fn parse_ascii_stl(bytes: &[u8]) -> Option<StlSummary> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut bounds = Bounds::default();
    let mut vertices = 0usize;
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let keyword = match fields.next() {
            Some(keyword) => keyword,
            None => continue,
        };
        if keyword.eq_ignore_ascii_case("vertex") {
            let x = fields.next()?.parse::<f64>().ok()?;
            let y = fields.next()?.parse::<f64>().ok()?;
            let z = fields.next()?.parse::<f64>().ok()?;
            bounds.add(x, y, z)?;
            vertices += 1;
        }
    }
    if vertices == 0 || !vertices.is_multiple_of(3) {
        return None;
    }
    Some(StlSummary {
        triangle_count: (vertices / 3) as u64,
        dimensions: bounds.dimensions(),
    })
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min: [f64; 3],
    max: [f64; 3],
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
        }
    }
}

impl Bounds {
    fn add(&mut self, x: f64, y: f64, z: f64) -> Option<()> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return None;
        }
        for (index, value) in [x, y, z].into_iter().enumerate() {
            self.min[index] = self.min[index].min(value);
            self.max[index] = self.max[index].max(value);
        }
        Some(())
    }

    fn dimensions(self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IMPORT_RECORD_VERSION, PROJECT_SCHEMA_VERSION, ProjectCommand, ProjectError,
        ProjectService, STL_IMPORT_PARSER_VERSION,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
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

    #[test]
    fn reports_empty_or_relative_locations_and_missing_project_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut service = ProjectService::new();
        assert!(matches!(service.current(), Err(ProjectError::NoProject)));
        assert!(matches!(service.save(), Err(ProjectError::NoProject)));
        assert!(matches!(
            service.execute(ProjectCommand::Rename {
                name: "Demo".to_owned(),
            }),
            Err(ProjectError::NoProject)
        ));

        assert!(matches!(
            service.create(Path::new(""), "Demo"),
            Err(ProjectError::LocationEmpty)
        ));
        assert!(matches!(
            service.create(Path::new("relative"), "Demo"),
            Err(ProjectError::LocationNotAbsolute(_))
        ));

        let fixture = Fixture::new()?;
        let file_location = fixture.root.join("location-file");
        fs::write(&file_location, b"not a directory")?;
        assert!(matches!(
            service.create(&file_location, "Demo"),
            Err(ProjectError::LocationCreateFailed(_))
        ));
        assert!(matches!(
            service.create(&file_location.join("child"), "Demo"),
            Err(ProjectError::LocationCreateFailed(_))
        ));
        Ok(())
    }

    #[test]
    fn validates_platform_names_and_command_failures() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        let mut invalid_names = vec![
            ".".to_owned(),
            "..".to_owned(),
            "bad/child".to_owned(),
            r"bad\child".to_owned(),
            "bad:child".to_owned(),
            "trailing.".to_owned(),
            "trailing ".to_owned(),
            "Demo.PANTA".to_owned(),
            "CON.txt".to_owned(),
            "LPT9.log".to_owned(),
            "line\nfeed".to_owned(),
        ];
        invalid_names.push("x".repeat(256));
        for name in invalid_names {
            assert!(matches!(
                service.create(&fixture.root, &name),
                Err(ProjectError::InvalidName(_))
            ));
        }

        let created = service.create(&fixture.root, "Demo.v1")?;
        assert_eq!(service.current()?, created);
        assert!(matches!(
            service.execute(ProjectCommand::Rename {
                name: "Demo.v1".to_owned(),
            }),
            Err(ProjectError::CommandInvalid(_))
        ));
        assert!(matches!(
            service.execute(ProjectCommand::Rename {
                name: "bad/name".to_owned(),
            }),
            Err(ProjectError::InvalidName(_))
        ));
        Ok(())
    }

    #[test]
    fn opens_invalid_manifests_and_accepts_case_insensitive_extension()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let project = fixture.root.join("Demo");
        fs::create_dir_all(&project)?;

        let mut service = ProjectService::new();
        assert!(matches!(
            service.open(Path::new("relative.panta")),
            Err(ProjectError::FileMissing(_))
        ));
        assert!(matches!(
            service.open(&fixture.root.join("missing.panta")),
            Err(ProjectError::FileMissing(_))
        ));
        let directory_with_extension = project.join("directory.panta");
        fs::create_dir(&directory_with_extension)?;
        assert!(matches!(
            service.open(&directory_with_extension),
            Err(ProjectError::FileMissing(_))
        ));

        let malformed = project.join("malformed.panta");
        fs::write(&malformed, b"not json")?;
        assert!(matches!(
            service.open(&malformed),
            Err(ProjectError::ManifestInvalid(_))
        ));

        let invalid_name = project.join("invalid-name.panta");
        fs::write(
            &invalid_name,
            br#"{"schema":1,"name":"bad/name","revision":0}"#,
        )?;
        assert!(matches!(
            service.open(&invalid_name),
            Err(ProjectError::InvalidName(_))
        ));

        let uppercase = project.join("Upper.PANTA");
        fs::write(&uppercase, br#"{"schema":1,"name":"Upper","revision":2}"#)?;
        let opened = service.open(&uppercase)?;
        assert_eq!(opened.name, "Upper");
        assert_eq!(opened.revision, 2);
        Ok(())
    }

    #[test]
    fn imports_ascii_stl_and_round_trips_record_and_asset() -> Result<(), Box<dyn std::error::Error>>
    {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("Case.STL");
        fs::write(
            &source,
            b"solid case\n facet normal 0 0 1\n  outer loop\n   vertex 1 2 3\n   vertex 5 2 3\n   vertex 1 8 6\n  endloop\n endfacet\nendsolid case\n",
        )?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Demo")?;
        let record = service.import_stl(&source, "dual-domain", "millimeters", true)?;
        assert_eq!(record.record_version, IMPORT_RECORD_VERSION);
        assert_eq!(record.parser_version, STL_IMPORT_PARSER_VERSION);
        assert_eq!(record.id, "import-1");
        assert_eq!(record.source_name, "Case.STL");
        assert_eq!(record.asset, "assets/imports/0001-Case.STL");
        assert_eq!(record.mesh_type, "dual-domain");
        assert_eq!(record.units, "millimeters");
        assert_eq!(record.triangle_count, 1);
        assert_eq!(record.dimensions, [4.0, 6.0, 3.0]);
        assert!(fixture.root.join("Demo").join(&record.asset).is_file());
        assert!(!service.current()?.dirty);

        let project_path = fixture.root.join("Demo/Demo.panta");
        let mut reopened = ProjectService::new();
        reopened.open(&project_path)?;
        assert_eq!(reopened.imports()?, vec![record]);
        Ok(())
    }

    #[test]
    fn inspects_binary_stl() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("Binary.STL");
        let vertices = [[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 4.0]];
        let mut bytes = vec![0_u8; 84 + 50];
        bytes[80..84].copy_from_slice(&1_u32.to_le_bytes());
        for (index, vertex) in vertices.into_iter().enumerate() {
            let offset = 84 + 12 + index * 12;
            for (axis, value) in vertex.into_iter().enumerate() {
                let value_offset = offset + axis * 4;
                bytes[value_offset..value_offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        fs::write(&source, bytes)?;

        let preview = ProjectService::new().inspect_stl(&source)?;
        assert_eq!(preview.source_name, "Binary.STL");
        assert_eq!(preview.triangle_count, 1);
        assert_eq!(preview.dimensions, [2.0, 3.0, 4.0]);
        Ok(())
    }

    #[test]
    fn rejects_invalid_stl_import_requests_without_project_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("case.stl");
        fs::write(&source, b"not an stl")?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Demo")?;
        let revision = service.current()?.revision;
        assert!(matches!(
            service.import_stl(&source, "unknown", "millimeters", false),
            Err(ProjectError::ImportParseFailed(_))
        ));
        assert!(matches!(
            service.import_stl(&source, "dual-domain", "unknown", false),
            Err(ProjectError::ImportParseFailed(_))
        ));
        assert!(matches!(
            service.import_stl(
                &fixture.root.join("case.obj"),
                "dual-domain",
                "millimeters",
                false
            ),
            Err(ProjectError::ImportFileMissing(_))
        ));
        assert_eq!(service.current()?.revision, revision);
        assert!(service.imports()?.is_empty());
        Ok(())
    }

    #[test]
    fn handles_manifest_revision_limits_and_save_io_failures()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let project = fixture.root.join("Max");
        fs::create_dir_all(&project)?;
        let project_file = project.join("Max.panta");
        fs::write(
            &project_file,
            format!(
                r#"{{"schema":{},"name":"Max","revision":{}}}"#,
                PROJECT_SCHEMA_VERSION,
                u64::MAX
            ),
        )?;
        let mut service = ProjectService::new();
        service.open(&project_file)?;
        let snapshot = service.execute(ProjectCommand::Rename {
            name: "MaxRenamed".to_owned(),
        })?;
        assert_eq!(snapshot.revision, u64::MAX);

        fs::remove_file(&project_file)?;
        fs::create_dir(&project_file)?;
        assert!(matches!(service.save(), Err(ProjectError::Io(_))));
        Ok(())
    }

    #[test]
    fn error_display_uses_stable_codes_and_details() {
        let errors = [
            (ProjectError::EmptyName, "project.empty_name"),
            (
                ProjectError::InvalidName("bad".to_owned()),
                "project.invalid_name: bad",
            ),
            (ProjectError::LocationEmpty, "project.location_empty"),
            (
                ProjectError::LocationNotAbsolute("relative".to_owned()),
                "project.location_not_absolute: relative",
            ),
            (
                ProjectError::LocationCreateFailed("location".to_owned()),
                "project.location_create_failed: location",
            ),
            (
                ProjectError::FileMissing("missing".to_owned()),
                "project.file_missing: missing",
            ),
            (
                ProjectError::InvalidFile("file".to_owned()),
                "project.invalid_file: file",
            ),
            (
                ProjectError::AlreadyExists("existing".to_owned()),
                "project.already_exists: existing",
            ),
            (
                ProjectError::ManifestInvalid("manifest".to_owned()),
                "project.manifest_invalid: manifest",
            ),
            (
                ProjectError::UnsupportedSchema(7),
                "project.unsupported_schema: 7",
            ),
            (ProjectError::NoProject, "project.no_project"),
            (
                ProjectError::CommandInvalid("unchanged".to_owned()),
                "project.command_invalid: unchanged",
            ),
            (
                ProjectError::ImportFileMissing("missing.stl".to_owned()),
                "project.import_file_missing: missing.stl",
            ),
            (
                ProjectError::ImportInvalidFile("mesh.obj".to_owned()),
                "project.import_invalid_file: mesh.obj",
            ),
            (
                ProjectError::ImportUnsupportedMeshType("shell".to_owned()),
                "project.import_unsupported_mesh_type: shell",
            ),
            (
                ProjectError::ImportUnsupportedUnits("unknown".to_owned()),
                "project.import_unsupported_units: unknown",
            ),
            (
                ProjectError::ImportParseFailed("invalid".to_owned()),
                "project.import_parse_failed: invalid",
            ),
            (
                ProjectError::ImportAssetCopyFailed("copy".to_owned()),
                "project.import_asset_copy_failed: copy",
            ),
            (ProjectError::Io("disk".to_owned()), "project.io: disk"),
        ];
        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }
}
