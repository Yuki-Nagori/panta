//! 工程会话与应用服务编排；格式解析、存储及缓存各有专用模块。
use super::model::{next_revision, validate_name};
use super::storage::write_manifest;
use super::{
    ActivationAttempt, ImportRecord, MIN_SUPPORTED_PROJECT_SCHEMA_VERSION, Outcome,
    PROJECT_FILE_EXTENSION, PROJECT_SCHEMA_VERSION, ProjectCommand, ProjectError, ProjectManifest,
    ProjectSnapshot, ProjectState, StlImportSession, SurfaceMeshCache, analysis_sequence,
    gate_location, material, preview, process_settings, storage,
};
use panta_mesh::SurfaceMesh;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 当前工程 service。其所有状态都由 Rust 拥有，C++ 仅持有 opaque Box。
/// `path` 是稳定的 `.panta` 主文件身份；重命名命令只更新清单名称，不移动
/// 工程目录或文件，避免把资产路径变更混入最小模型命令。
///
/// `generation` 是会话级运行期计数（create/open 各递增一次，不持久化），
/// 与 `ImportRecord.id` / attempt 一起构成只读 FSM 激活的相关性键（073/080）。
#[derive(Debug, Default)]
pub struct ProjectService {
    pub(super) current: Option<ProjectState>,
    pub(super) import_session: StlImportSession,
    pub(super) latest_mesh_id: Option<String>,
    pub(super) mesh_cache: SurfaceMeshCache,
    pub(super) activation_attempts: HashMap<String, u64>,
    pub(super) activation: Arc<crate::fsm::open_saved_stl::ActivationCoordinator>,
    // 候选提交期间锁住工程元数据；Mesh 驻留和只读显示操作仍可继续。
    pub(super) pending_metadata: Option<storage::PendingMetadataWrite>,
    pub(super) pending_preview: Option<preview::PendingPreview>,
    pub(super) preview_request: u64,
}

impl ProjectService {
    pub fn new() -> Self {
        Self::default()
    }

    // 后台候选已冻结；消费结果前，任何工程写入都会与候选清单竞争。
    pub(super) fn ensure_project_writable(&self) -> Result<(), ProjectError> {
        if self.pending_metadata.is_some() {
            return Err(ProjectError::CommandInvalid(
                "metadata confirmation pending".to_owned(),
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn with_mesh_cache_budget(budget_bytes: usize) -> Self {
        Self {
            mesh_cache: SurfaceMeshCache::with_budget(budget_bytes),
            ..Self::default()
        }
    }

    /// 创建 `location/name/name.panta` 目录包和初始清单；目标已存在时拒绝覆盖。
    pub fn create(&mut self, location: &Path, name: &str) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
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
            analysis_sequences: BTreeMap::new(),
            materials: BTreeMap::new(),
            fill_settings: BTreeMap::new(),
            gate_location_settings: BTreeMap::new(),
        };
        if let Err(error) = write_manifest(&mut state) {
            if let Some(project_root) = state.path.parent() {
                let _ = fs::remove_dir_all(project_root);
            }
            return Err(error);
        }
        self.current = Some(state);
        self.clear_stl_preview();
        self.latest_mesh_id = None;
        self.mesh_cache.clear();
        self.activation_attempts.clear();
        self.activation.advance_generation();
        self.snapshot()
    }

    /// 打开既有 `.panta` 主文件并读取清单；不改变 cwd，也不接受其他扩展名。
    ///
    /// 打开只读清单，不解析任何导入资产：已保存 STL 的网格由
    /// [`Self::begin_asset_activation`] 异步按需重建（080 文档页签），
    /// 避免保留同步与异步两条重复加载路径。
    pub fn open(&mut self, path: &Path) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
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
        analysis_sequence::validate_sequences(&manifest)?;
        material::validate_materials(&manifest)?;
        process_settings::validate_settings(&manifest.imports, &manifest.fill_settings)?;
        gate_location::validate_settings(&manifest.imports, &manifest.gate_location_settings)?;
        self.current = Some(ProjectState {
            path: path.to_path_buf(),
            name: manifest.name,
            revision: manifest.revision,
            persisted_revision: Some(manifest.revision),
            dirty: false,
            imports: manifest.imports,
            analysis_sequences: manifest.analysis_sequences,
            materials: manifest.materials,
            fill_settings: manifest.fill_settings,
            gate_location_settings: manifest.gate_location_settings,
        });
        self.latest_mesh_id = None;
        self.mesh_cache.clear();
        self.activation_attempts.clear();
        self.clear_stl_preview();
        self.activation.advance_generation();
        self.snapshot()
    }

    /// 将当前模型以同目录临时文件写入后替换 `.panta` 主文件。
    pub fn save(&mut self) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        write_manifest(state)?;
        state.dirty = false;
        self.snapshot()
    }

    /// 执行一个有校验的模型命令，并标记工程 dirty。
    pub fn execute(&mut self, command: ProjectCommand) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        match command {
            ProjectCommand::Rename { name } => {
                validate_name(&name)?;
                if name == state.name {
                    return Err(ProjectError::CommandInvalid("name is unchanged".to_owned()));
                }
                let revision = next_revision(state.revision)?;
                state.name = name;
                state.revision = revision;
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
        let import_id = record.id.clone();
        let units = record.units.clone();
        let attempt = self.activation.begin(
            &import_id,
            crate::fsm::open_saved_stl::ActivationRequest { asset_path, units },
        );
        self.activation_attempts.insert(import_id, attempt.attempt);
        Ok(attempt)
    }

    /// 请求取消一次在飞激活；`Cancelled` 终态由 worker 在安全检查点确认。
    pub fn cancel_asset_activation(&mut self, attempt: u64) -> bool {
        self.activation.cancel(attempt)
    }

    /// 拉取当前会话已完成的激活结果；旧代次结果已在 Rust 侧释放。
    pub fn drain_asset_activations(&mut self) -> Vec<Outcome> {
        self.activation.drain()
    }

    /// 抛弃已关闭页签的 attempt，避免其迟到成功结果进入 Mesh 缓存。
    pub fn forget_asset_activation(&mut self, import_id: &str) {
        self.activation_attempts.remove(import_id);
    }

    /// 仅接受该记录最近一次仍有效的激活结果，防止同 ID 页签重开后的旧结果覆盖新数据。
    pub fn is_current_activation_attempt(&self, import_id: &str, attempt: u64) -> bool {
        self.activation_attempts.get(import_id) == Some(&attempt)
    }

    /// 激活终态已消费；旧 attempt 不得清理同 ID 的较新 attempt。
    pub fn finish_asset_activation(&mut self, import_id: &str, attempt: u64) {
        if self.is_current_activation_attempt(import_id, attempt) {
            self.activation_attempts.remove(import_id);
        }
    }

    /// 将仍有效的成功激活产物移交给工程运行期缓存。
    pub fn cache_activation_result(
        &mut self,
        import_id: &str,
        attempt: u64,
        mesh: SurfaceMesh,
    ) -> bool {
        if !self.is_current_activation_attempt(import_id, attempt) {
            return false;
        }
        self.finish_asset_activation(import_id, attempt);
        self.latest_mesh_id = Some(import_id.to_owned());
        self.mesh_cache.insert_active(import_id.to_owned(), mesh);
        true
    }

    /// 将新导入成功提交的 Mesh 移交给工程运行期缓存，并标记为活动项。
    pub fn cache_activated_mesh(&mut self, import_id: String, mesh: SurfaceMesh) {
        self.latest_mesh_id = Some(import_id.clone());
        self.mesh_cache.insert_active(import_id, mesh);
    }

    /// 选择 Rust 缓存中的活动网格；未驻留时由调用方启动异步资产激活。
    pub fn activate_mesh_document(&mut self, import_id: &str) -> bool {
        if !self.mesh_cache.activate(import_id) {
            return false;
        }
        self.latest_mesh_id = Some(import_id.to_owned());
        true
    }

    /// 返回一个导入记录的只读网格，不复制领域数据。
    pub fn mesh_for_import(&self, import_id: &str) -> Option<&SurfaceMesh> {
        self.mesh_cache.get(import_id)
    }

    /// 当前 Rust 运行期 Mesh 驻留集合，供 Qt ViewModel 投影页签状态。
    pub fn resident_mesh_ids(&self) -> Vec<String> {
        self.mesh_cache.resident_ids()
    }

    /// 文档关闭后释放对应 Mesh；工程资产与 ImportRecord 保留。
    pub fn release_mesh_document(&mut self, import_id: &str) {
        self.forget_asset_activation(import_id);
        self.mesh_cache.remove(import_id);
        self.mesh_cache.trim();
        if self.latest_mesh_id.as_deref() == Some(import_id) {
            self.latest_mesh_id = None;
        }
    }

    /// 活动文档切到 Welcome / 空白时取消 Mesh pin，再按预算回收缓存。
    pub fn deactivate_mesh_document(&mut self) {
        self.mesh_cache.deactivate();
    }

    /// 最近一次成功导入或激活的网格；供 Rust 工程服务测试读取。
    pub fn current_mesh(&self) -> Option<&SurfaceMesh> {
        self.latest_mesh_id
            .as_deref()
            .and_then(|id| self.mesh_cache.get(id))
    }

    pub(super) fn snapshot(&self) -> Result<ProjectSnapshot, ProjectError> {
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
