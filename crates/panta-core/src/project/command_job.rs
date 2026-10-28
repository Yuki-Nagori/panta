//! 后台工程命令只准备拥有式结果，服务线程校验请求身份后发布。

use super::model::validate_name;
use super::service::{resolve_asset_path, validate_schema_four_mesh_assets};
use super::storage::write_manifest;
use super::{
    PROJECT_FILE_EXTENSION, PROJECT_SCHEMA_VERSION, ProjectError, ProjectManifest, ProjectService,
    ProjectSnapshot, ProjectState, analysis_sequence, gate_location, material, process_settings,
};
use crate::path::{PathService, RootCategory};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

/// GUI 工程用例；路径检查、清单解析与原子写入全部在后台执行。
#[derive(Debug)]
pub enum ProjectOperation {
    Create {
        location: PathBuf,
        name: String,
    },
    Open {
        path: PathBuf,
    },
    Save,
    SetAnalysisSequence {
        expected_path: PathBuf,
        expected_revision: u64,
        import_id: String,
        sequence_id: String,
    },
}

/// 唯一结果接收者；宿主须在独立线程等待，不借用活跃服务。
#[derive(Debug)]
pub struct ProjectOperationRequest {
    identity: Arc<()>,
    receiver: mpsc::Receiver<Result<PreparedOperation, ProjectError>>,
}

/// 后台已完成的候选，不会自动改变当前工程。
#[derive(Debug)]
pub struct ProjectOperationCompletion {
    identity: Arc<()>,
    result: Result<PreparedOperation, ProjectError>,
}

impl ProjectOperationRequest {
    pub fn wait(self) -> ProjectOperationCompletion {
        ProjectOperationCompletion {
            identity: self.identity,
            result: self.receiver.recv().unwrap_or_else(|_| {
                Err(ProjectError::Io(
                    "project operation worker disconnected".to_owned(),
                ))
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct PendingProjectOperation {
    identity: Arc<()>,
    expected: Option<ProjectSnapshot>,
}

#[derive(Debug)]
pub(super) struct PreparedReplacement {
    pub(super) state: ProjectState,
    pub(super) paths: PathService,
}

#[derive(Debug)]
enum PreparedOperation {
    Replace(PreparedReplacement),
    Update(ProjectState),
}

impl ProjectService {
    /// 准入失败不冻结工程；已接受写入在宿主销毁后仍由执行通道完成。
    pub fn begin_project_operation(
        &mut self,
        operation: ProjectOperation,
    ) -> Result<ProjectOperationRequest, ProjectError> {
        self.ensure_project_writable()?;
        let state = match operation {
            ProjectOperation::Save | ProjectOperation::SetAnalysisSequence { .. } => Some(
                self.current
                    .as_ref()
                    .ok_or(ProjectError::NoProject)?
                    .clone(),
            ),
            _ => None,
        };
        let executor = if matches!(operation, ProjectOperation::Open { .. }) {
            &self.execution.reads
        } else {
            &self.execution.writes
        };
        let expected = self.current().ok();
        let identity = Arc::new(());
        let (sender, receiver) = mpsc::channel();
        executor
            .submit(move || {
                let result = prepare_operation(operation, state);
                if let Err(mpsc::SendError(Err(error))) = sender.send(result) {
                    eprintln!("panta: project operation failed after service teardown: {error}");
                }
            })
            .map_err(|error| ProjectError::Io(format!("start project operation: {error}")))?;
        self.pending_project = Some(PendingProjectOperation {
            identity: Arc::clone(&identity),
            expected,
        });
        Ok(ProjectOperationRequest { identity, receiver })
    }

    /// 只发布本服务准入的当前候选；外来或旧请求不能解锁其他在途命令。
    pub fn finish_project_operation(
        &mut self,
        completion: ProjectOperationCompletion,
    ) -> Result<ProjectSnapshot, ProjectError> {
        let pending = self
            .pending_project
            .as_ref()
            .filter(|pending| Arc::ptr_eq(&pending.identity, &completion.identity))
            .ok_or_else(|| ProjectError::CommandInvalid("stale project operation".to_owned()))?;
        let expected = pending.expected.clone();
        self.pending_project = None;
        if self.current().ok() != expected {
            return Err(ProjectError::CommandInvalid(
                "project operation target changed".to_owned(),
            ));
        }
        match completion.result? {
            PreparedOperation::Replace(prepared) => self.publish_project_replacement(prepared),
            PreparedOperation::Update(state) => {
                self.current = Some(state);
                self.snapshot()
            }
        }
    }
}

fn prepare_operation(
    operation: ProjectOperation,
    state: Option<ProjectState>,
) -> Result<PreparedOperation, ProjectError> {
    match operation {
        ProjectOperation::Create { location, name } => {
            prepare_create(&location, &name).map(PreparedOperation::Replace)
        }
        ProjectOperation::Open { path } => prepare_open(&path).map(PreparedOperation::Replace),
        ProjectOperation::Save => {
            let mut state = state.ok_or(ProjectError::NoProject)?;
            write_manifest(&mut state)?;
            state.dirty = false;
            Ok(PreparedOperation::Update(state))
        }
        ProjectOperation::SetAnalysisSequence {
            expected_path,
            expected_revision,
            import_id,
            sequence_id,
        } => {
            let state = state.ok_or(ProjectError::NoProject)?;
            let candidate = analysis_sequence::prepare_sequence(
                &state,
                &expected_path,
                expected_revision,
                &import_id,
                &sequence_id,
            )?;
            Ok(PreparedOperation::Update(candidate))
        }
    }
}

fn prepare_paths(root: &Path) -> Result<PathService, ProjectError> {
    let mut paths = PathService::new();
    paths
        .set_root(RootCategory::Project, root)
        .map_err(|error| ProjectError::Io(error.to_string()))?;
    Ok(paths)
}

pub(super) fn prepare_create(
    location: &Path,
    name: &str,
) -> Result<PreparedReplacement, ProjectError> {
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

    let mut state = ProjectState {
        path: project_path,
        name: name.to_owned(),
        revision: 0,
        persisted_revision: None,
        dirty: false,
        imports: Vec::new(),
        mesh_assets: BTreeMap::new(),
        analysis_sequences: BTreeMap::new(),
        materials: BTreeMap::new(),
        fill_settings: BTreeMap::new(),
        gate_location_settings: BTreeMap::new(),
    };
    let prepared = (|| {
        let paths = prepare_paths(&project_root)?;
        write_manifest(&mut state)?;
        Ok(paths)
    })();
    let paths = match prepared {
        Ok(paths) => paths,
        Err(error) => {
            if let Some(project_root) = state.path.parent() {
                let _ = fs::remove_dir_all(project_root);
            }
            return Err(error);
        }
    };
    Ok(PreparedReplacement { state, paths })
}

pub(super) fn prepare_open(path: &Path) -> Result<PreparedReplacement, ProjectError> {
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
    let bytes =
        fs::read(path).map_err(|error| ProjectError::Io(format!("{}: {error}", path.display())))?;
    // 先只读取版本号，确保旧 schema 不会因其记录形状与当前模型不同而
    // 被误报为当前清单损坏。
    #[derive(serde::Deserialize)]
    struct SchemaEnvelope {
        schema: u32,
    }

    let envelope: SchemaEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| ProjectError::ManifestInvalid(format!("{}: {error}", path.display())))?;
    if envelope.schema != PROJECT_SCHEMA_VERSION {
        return Err(ProjectError::UnsupportedSchema(envelope.schema));
    }
    let manifest: ProjectManifest = serde_json::from_slice(&bytes)
        .map_err(|error| ProjectError::ManifestInvalid(format!("{}: {error}", path.display())))?;
    validate_schema_four_mesh_assets(&manifest.imports, &manifest.mesh_assets)?;
    validate_name(&manifest.name)?;
    analysis_sequence::validate_sequences(&manifest)?;
    material::validate_materials(&manifest)?;
    process_settings::validate_settings(&manifest.imports, &manifest.fill_settings)?;
    gate_location::validate_settings(&manifest.imports, &manifest.gate_location_settings)?;
    let project_root = path
        .parent()
        .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
    for asset in manifest.mesh_assets.values() {
        resolve_asset_path(project_root, &asset.relative_path)?;
    }
    let state = ProjectState {
        path: path.to_path_buf(),
        name: manifest.name,
        revision: manifest.revision,
        persisted_revision: Some(manifest.revision),
        dirty: false,
        imports: manifest.imports,
        mesh_assets: manifest.mesh_assets,
        analysis_sequences: manifest.analysis_sequences,
        materials: manifest.materials,
        fill_settings: manifest.fill_settings,
        gate_location_settings: manifest.gate_location_settings,
    };
    let paths = prepare_paths(project_root)?;
    Ok(PreparedReplacement { state, paths })
}

#[cfg(test)]
mod tests;
