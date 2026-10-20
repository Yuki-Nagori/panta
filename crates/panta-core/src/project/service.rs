//! 工程会话与应用服务编排；格式解析、存储及缓存各有专用模块。
use super::model::{MeshAssetRef, next_revision, validate_name};
use super::storage::write_manifest;
use super::{
    ActivationAttempt, ActivationResultKind, DocumentDiagnostic, DocumentWorkspace,
    IMPORT_RECORD_VERSION, ImportRecord, ProjectCommand, ProjectError, ProjectSnapshot,
    ProjectState, StlImportSession, SurfaceMeshCache, WorkspaceActivationRequest,
    WorkspaceSnapshot, activation_error_message_key, preview, storage,
};
use panta_mesh::SurfaceMesh;
use std::collections::{BTreeMap, HashMap};
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
    pub(super) pending_project: Option<super::command_job::PendingProjectOperation>,
    pub(super) pending_metadata: Option<storage::PendingMetadataWrite>,
    pub(super) pending_variable: Option<super::variable_document::PendingVariableDocument>,
    pub(super) pending_preview: Option<preview::PendingPreview>,
    pub(super) preview_request: u64,
    pub(super) pending_import: Option<super::import::PendingImport>,
    pub(super) import_request: u64,
    pub(super) execution: crate::execution::ProjectExecution,
    pub(super) workspace: DocumentWorkspace,
    pub(super) runtime_context: Option<crate::runtime::RuntimeContext>,
}

impl ProjectService {
    pub fn new() -> Self {
        let mut service = Self::default();
        service.workspace = DocumentWorkspace::new("Welcome");
        service
    }

    // 后台候选已冻结；消费结果前，任何工程写入都会与候选清单竞争。
    pub(super) fn ensure_project_writable(&self) -> Result<(), ProjectError> {
        if self.pending_project.is_some() {
            return Err(ProjectError::CommandInvalid(
                "project operation pending".to_owned(),
            ));
        }
        if self
            .pending_variable
            .as_ref()
            .is_some_and(|pending| pending.write)
        {
            return Err(ProjectError::CommandInvalid(
                "variable document save pending".to_owned(),
            ));
        }
        if self.pending_metadata.is_some() {
            return Err(ProjectError::CommandInvalid(
                "metadata confirmation pending".to_owned(),
            ));
        }
        if self.pending_import.is_some() {
            return Err(ProjectError::CommandInvalid(
                "STL import pending".to_owned(),
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn with_mesh_cache_budget(budget_bytes: usize) -> Self {
        let mut service = Self::default();
        service.mesh_cache = SurfaceMeshCache::with_budget(budget_bytes);
        service
    }

    /// 同步工程入口供 CLI 使用；GUI 通过后台工程操作执行同一准备流程。
    pub fn create(&mut self, location: &Path, name: &str) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
        let prepared = super::command_job::prepare_create(location, name)?;
        self.publish_project_replacement(prepared)
    }

    /// 打开清单，不加载 Mesh sidecar；GUI 使用后台工程操作。
    pub fn open(&mut self, path: &Path) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
        let prepared = super::command_job::prepare_open(path)?;
        self.publish_project_replacement(prepared)
    }

    pub(super) fn publish_project_replacement(
        &mut self,
        prepared: super::command_job::PreparedReplacement,
    ) -> Result<ProjectSnapshot, ProjectError> {
        self.current = Some(prepared.state);
        self.clear_stl_preview();
        self.latest_mesh_id = None;
        self.mesh_cache.clear();
        self.activation_attempts.clear();
        let generation = self.activation.advance_generation();
        self.replace_runtime_context(Some(crate::runtime::RuntimeContext::new(
            generation,
            prepared.paths,
        )));
        self.reset_document_workspace("Welcome");
        self.snapshot()
    }

    /// 当前工程的 Rust 运行时上下文；未打开工程时为空。
    pub fn runtime_context(&self) -> Option<&crate::runtime::RuntimeContext> {
        self.runtime_context.as_ref()
    }

    /// 以预期修订原子提交变量快照；未打开工程或上下文失效时拒绝提交。
    pub fn commit_runtime_variables(
        &mut self,
        scope: &crate::runtime::RuntimeTaskScope,
        values: BTreeMap<String, crate::runtime::RuntimeValue>,
    ) -> Result<crate::runtime::VariableSnapshot, crate::runtime::RuntimeCommitError> {
        if self.pending_project.is_some() {
            return Err(crate::runtime::RuntimeCommitError::Busy);
        }
        if self
            .pending_variable
            .as_ref()
            .is_some_and(|pending| pending.write)
        {
            return Err(crate::runtime::RuntimeCommitError::Busy);
        }
        let context = self
            .runtime_context
            .as_mut()
            .ok_or(crate::runtime::RuntimeCommitError::NoContext)?;
        context.commit_task_variables(scope, values).cloned()
    }

    /// 解析变量 DSL、校验逻辑资源引用并按捕获修订原子提交快照。
    pub fn commit_variable_document(
        &mut self,
        scope: &crate::runtime::RuntimeTaskScope,
        source: &str,
    ) -> Result<crate::runtime::VariableSnapshot, VariableDocumentError> {
        let values = variable_runtime_values(scope, source)?;
        self.commit_runtime_variables(scope, values)
            .map_err(VariableDocumentError::Commit)
    }

    /// 从工程固定的 `variables.pa` 读取、校验并提交变量快照。
    pub fn load_variable_document(
        &mut self,
        expected_generation: u64,
        expected_revision: u64,
    ) -> Result<(String, crate::runtime::VariableSnapshot), VariableDocumentError> {
        let scope = self.variable_scope(expected_generation, expected_revision)?;
        let project_path = self
            .current
            .as_ref()
            .ok_or(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::NoContext,
            ))?
            .path
            .clone();
        let path = super::variable_document::path(&project_path)
            .map_err(VariableDocumentError::Project)?;
        let source =
            super::variable_document::read(&path).map_err(VariableDocumentError::Project)?;
        let snapshot = self.commit_variable_document(&scope, &source)?;
        Ok((source, snapshot))
    }

    /// 规范化并原子保存变量源码；文件替换成功后发布预先验证的快照。
    pub fn save_variable_document(
        &mut self,
        expected_generation: u64,
        expected_revision: u64,
        source: &str,
    ) -> Result<(String, crate::runtime::VariableSnapshot), VariableDocumentError> {
        let scope = self.variable_scope(expected_generation, expected_revision)?;
        let project_path = self
            .current
            .as_ref()
            .ok_or(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::NoContext,
            ))?
            .path
            .clone();
        let prepared = super::variable_document::prepare_save(&scope, &project_path, source)?;
        super::variable_document::write_atomically(&prepared.path, prepared.source.as_bytes())
            .map_err(VariableDocumentError::Project)?;
        let snapshot = self
            .commit_runtime_variables(&scope, prepared.values)
            .map_err(VariableDocumentError::Commit)?;
        Ok((prepared.source, snapshot))
    }

    pub(super) fn variable_scope(
        &self,
        expected_generation: u64,
        expected_revision: u64,
    ) -> Result<crate::runtime::RuntimeTaskScope, VariableDocumentError> {
        if self.pending_project.is_some() {
            return Err(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::Busy,
            ));
        }
        if self
            .pending_variable
            .as_ref()
            .is_some_and(|pending| pending.write)
        {
            return Err(VariableDocumentError::Project(
                ProjectError::CommandInvalid("variable document save pending".to_owned()),
            ));
        }
        let scope = self
            .runtime_context
            .as_ref()
            .ok_or(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::NoContext,
            ))?
            .capture_task_scope();
        if scope.generation() != expected_generation {
            return Err(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::StaleContext,
            ));
        }
        if scope.variables().revision() != expected_revision {
            return Err(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::StaleRevision {
                    expected: expected_revision,
                    current: scope.variables().revision(),
                },
            ));
        }
        Ok(scope)
    }

    fn replace_runtime_context(&mut self, context: Option<crate::runtime::RuntimeContext>) {
        if let Some(previous) = &mut self.runtime_context {
            previous.close();
        }
        self.runtime_context = context;
        self.pending_variable = None;
    }

    /// 将当前模型以同目录临时文件写入后替换 `.panta` 主文件。
    pub fn save(&mut self) -> Result<ProjectSnapshot, ProjectError> {
        self.ensure_project_writable()?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        write_manifest(state)?;
        state.dirty = false;
        self.snapshot()
    }

    /// 执行工程命令；打开命令复用工程读取流程，模型修改命令更新内存修订。
    pub fn execute(&mut self, command: ProjectCommand) -> Result<ProjectSnapshot, ProjectError> {
        match command {
            ProjectCommand::Open { path } => self.open(&path),
            ProjectCommand::Rename { name } => {
                self.ensure_project_writable()?;
                let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
                validate_name(&name)?;
                if name == state.name {
                    return Err(ProjectError::CommandInvalid("name is unchanged".to_owned()));
                }
                let revision = next_revision(state.revision)?;
                state.name = name;
                state.revision = revision;
                state.dirty = true;
                self.snapshot()
            }
        }
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
    /// 路径包含关系、符号链接和文件读取由 worker 校验，失败经终态返回。
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
        let asset_ref = state
            .mesh_assets
            .get(&record.mesh_asset_id)
            .ok_or_else(|| {
                ProjectError::ManifestInvalid("mesh asset index entry is missing".to_owned())
            })?;
        if asset_ref.revision == 0
            || asset_ref.format_version != panta_mesh::MESH_ASSET_FORMAT_VERSION
            || asset_ref.kind != "surface"
            || asset_ref.source_revision != record.source_digest
        {
            return Err(ProjectError::ManifestInvalid(
                "mesh asset index entry is unsupported".to_owned(),
            ));
        }
        let project_root = root.to_owned();
        let relative_path = asset_ref.relative_path.clone();
        let expected_digest = asset_ref.digest.clone();
        let import_id = record.id.clone();
        let attempt = self
            .activation
            .begin(
                &import_id,
                crate::fsm::open_saved_stl::ActivationRequest {
                    project_root,
                    relative_path,
                    expected_digest,
                },
                Arc::clone(&self.execution.reads),
            )
            .map_err(|detail| ProjectError::CommandInvalid(detail.to_owned()))?;
        self.activation_attempts.insert(import_id, attempt.attempt);
        Ok(attempt)
    }

    /// 仅投影已提交 sidecar 的相对路径，不访问文件系统或构成资产身份。
    /// 包目录包含关系与符号链接校验由激活 worker 在实际读取前执行。
    pub fn mesh_asset_path_for_import(&self, import_id: &str) -> Result<String, ProjectError> {
        let state = self.current.as_ref().ok_or(ProjectError::NoProject)?;
        let record = state
            .imports
            .iter()
            .find(|record| record.id == import_id)
            .ok_or_else(|| ProjectError::ImportRecordMissing(import_id.to_owned()))?;
        let asset = state
            .mesh_assets
            .get(&record.mesh_asset_id)
            .ok_or_else(|| {
                ProjectError::ManifestInvalid("mesh asset index entry is missing".to_owned())
            })?;
        Ok(asset.relative_path.clone())
    }

    /// 请求取消一次在飞激活；`Cancelled` 终态由 worker 在安全检查点确认。
    pub fn cancel_asset_activation(&mut self, attempt: u64) -> bool {
        self.activation.cancel(attempt)
    }

    /// 消费终态并完成过期校验、缓存发布和工作区更新；消费者只读取最终结果。
    pub fn drain_asset_activations(&mut self) -> Vec<super::CompletedActivation> {
        self.activation
            .drain()
            .into_iter()
            .map(|outcome| self.complete_asset_activation(outcome))
            .collect()
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

    /// 重置运行期页签，并取消其未完成的激活任务。
    pub fn reset_document_workspace(&mut self, welcome_title: &str) {
        let old_documents = self.workspace.snapshot().documents;
        let attempts = self.workspace.reset(welcome_title);
        for attempt in attempts {
            self.cancel_asset_activation(attempt);
        }
        for document in old_documents {
            if document.kind == super::DocumentKind::Import {
                self.release_mesh_document(&document.id);
            }
        }
        self.deactivate_mesh_document();
    }

    pub fn document_workspace(&self) -> WorkspaceSnapshot {
        let mut snapshot = self.workspace.snapshot();
        if let Some(state) = &self.current {
            for document in &mut snapshot.documents {
                if document.kind != super::DocumentKind::Import {
                    continue;
                }
                let Some(import_index) = state
                    .imports
                    .iter()
                    .position(|record| record.id == document.id)
                else {
                    continue;
                };
                let display_name = &state.imports[import_index].display_name;
                document.display_count = state
                    .imports
                    .iter()
                    .filter(|record| record.display_name == *display_name)
                    .count() as u64;
                document.display_ordinal = state.imports[..=import_index]
                    .iter()
                    .filter(|record| record.display_name == *display_name)
                    .count() as u64;
            }
        }
        snapshot
    }

    /// 打开或重试一个 STL 文档；异步读取仍由现有 ProjectService 执行。
    pub fn open_import_document(&mut self, import_id: &str, title: &str) {
        self.workspace.open_import(import_id, title);
        let request = self.workspace.activation_request(import_id);
        self.apply_workspace_activation_request(request);
    }

    /// 将刚导入且已在缓存中的网格登记为活动文档。
    pub fn publish_import_document(&mut self, import_id: &str, title: &str) {
        if self.workspace.open_ready_import(import_id, title) {
            self.activate_mesh_document(import_id);
            self.sync_workspace_residency();
        }
    }

    pub fn activate_workspace_document(&mut self, document_id: &str) {
        let request = self.workspace.activation_request(document_id);
        self.apply_workspace_activation_request(request);
    }

    pub fn close_workspace_document(&mut self, document_id: &str) {
        let Some(result) = self.workspace.close(document_id) else {
            return;
        };
        if let Some(attempt) = result.cancel_attempt {
            self.cancel_asset_activation(attempt);
        }
        if result.release_import {
            self.release_mesh_document(document_id);
        }
        if !result.active_changed {
            return;
        }
        if let Some(next) = result.activate_document {
            self.activate_workspace_document(&next);
        } else if let Some(next) = result.load_document {
            self.begin_workspace_activation(&next);
        } else {
            self.deactivate_mesh_document();
        }
        self.sync_workspace_residency();
    }

    pub fn move_workspace_document(&mut self, from: usize, to: usize) {
        self.workspace.move_document(from, to);
    }

    pub fn sync_workspace_residency(&mut self) -> bool {
        let resident_ids = self.mesh_cache.resident_ids();
        self.workspace
            .sync_residency(resident_ids.iter().map(String::as_str))
    }

    /// 激活完成由服务校验 attempt 并更新工作区，不让桥接层合并业务状态。
    pub fn resolve_workspace_activation(
        &mut self,
        import_id: &str,
        attempt: u64,
        result: ActivationResultKind,
        diagnostic: DocumentDiagnostic,
    ) -> bool {
        let message_key = if result == ActivationResultKind::Failed {
            activation_error_message_key(&diagnostic.code)
        } else {
            ""
        };
        let changed =
            self.workspace
                .resolve_activation(import_id, attempt, result, message_key, diagnostic);
        if changed && result == ActivationResultKind::Succeeded {
            self.activate_mesh_document(import_id);
        }
        if changed {
            self.sync_workspace_residency();
        }
        changed
    }

    fn apply_workspace_activation_request(&mut self, request: WorkspaceActivationRequest) {
        match request {
            WorkspaceActivationRequest::Ignore => {}
            WorkspaceActivationRequest::ActivateWelcome => {
                self.deactivate_mesh_document();
                self.sync_workspace_residency();
            }
            WorkspaceActivationRequest::ActivateImport(import_id) => {
                if self.activate_mesh_document(&import_id) {
                    self.workspace.set_active(&import_id);
                    self.sync_workspace_residency();
                } else {
                    self.workspace.mark_unloaded(&import_id);
                    let request = self.workspace.activation_request(&import_id);
                    self.apply_workspace_activation_request(request);
                }
            }
            WorkspaceActivationRequest::LoadImport(import_id) => {
                self.begin_workspace_activation(&import_id);
            }
        }
    }

    fn begin_workspace_activation(&mut self, import_id: &str) {
        match self.begin_asset_activation(import_id) {
            Ok(attempt) => {
                self.workspace
                    .set_activation_attempt(import_id, attempt.attempt);
            }
            Err(error) => {
                self.workspace.fail_document(
                    import_id,
                    activation_error_message_key(error.code()),
                    DocumentDiagnostic {
                        code: error.code().to_owned(),
                        category: error.category().to_owned(),
                        detail: error.detail(),
                    },
                );
            }
        }
    }

    /// 最近一次成功导入或激活的网格；供 Rust 工程服务测试读取。
    pub fn current_mesh(&self) -> Option<&SurfaceMesh> {
        self.latest_mesh_id
            .as_deref()
            .and_then(|id| self.mesh_cache.get(id))
    }

    /// 当前工程运行时会话的代次；工程切换会使先前的编辑请求失效。
    pub fn variable_document_generation(&self) -> u64 {
        self.runtime_context
            .as_ref()
            .map_or(0, crate::runtime::RuntimeContext::generation)
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

/// 变量文档解析、资源引用转换或运行时提交失败。
#[derive(Debug)]
pub enum VariableDocumentError {
    Diagnostics(panta_dsl_core::Diagnostics),
    SourceTooLarge {
        size: usize,
        limit: usize,
    },
    InvalidResource {
        name: String,
        reference: String,
        source_range: std::ops::Range<usize>,
        line: usize,
        column: usize,
        error: crate::path::PathError,
    },
    Commit(crate::runtime::RuntimeCommitError),
    Project(ProjectError),
}

impl std::fmt::Display for VariableDocumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnostics(error) => write!(formatter, "{error}"),
            Self::SourceTooLarge { size, limit } => {
                write!(
                    formatter,
                    "normalized variable source is {size} bytes; limit is {limit}"
                )
            }
            Self::InvalidResource {
                name,
                reference,
                error,
                ..
            } => write!(
                formatter,
                "invalid resource for variable '{name}' ({reference}): {error}"
            ),
            Self::Commit(error) => write!(formatter, "{error}"),
            Self::Project(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for VariableDocumentError {}

pub(super) fn variable_runtime_values(
    scope: &crate::runtime::RuntimeTaskScope,
    source: &str,
) -> Result<BTreeMap<String, crate::runtime::RuntimeValue>, VariableDocumentError> {
    let evaluated =
        panta_dsl_core::evaluate_variables(source).map_err(VariableDocumentError::Diagnostics)?;
    let mut values = BTreeMap::new();
    for (name, value) in evaluated.iter() {
        let value = match value {
            panta_dsl_core::EvaluatedValue::Bool(value) => {
                crate::runtime::RuntimeValue::Boolean(*value)
            }
            panta_dsl_core::EvaluatedValue::Int(value) => {
                crate::runtime::RuntimeValue::Integer(*value)
            }
            panta_dsl_core::EvaluatedValue::Real(value) => {
                crate::runtime::RuntimeValue::Real(*value)
            }
            panta_dsl_core::EvaluatedValue::String(value) => {
                crate::runtime::RuntimeValue::String(value.clone())
            }
            panta_dsl_core::EvaluatedValue::Resource(reference) => {
                let resource = crate::path::ResourceRef::parse(reference).map_err(|error| {
                    let source_range = evaluated.source_range(name).unwrap_or_default();
                    let (line, column) = source_position(source, source_range.start);
                    VariableDocumentError::InvalidResource {
                        name: name.to_owned(),
                        reference: reference.to_owned(),
                        source_range,
                        line,
                        column,
                        error,
                    }
                })?;
                scope
                    .validate_reference(&resource)
                    .map_err(|error| match error {
                        crate::runtime::RuntimeResourceError::Path(error) => {
                            let source_range = evaluated.source_range(name).unwrap_or_default();
                            let (line, column) = source_position(source, source_range.start);
                            VariableDocumentError::InvalidResource {
                                name: name.to_owned(),
                                reference: reference.to_owned(),
                                source_range,
                                line,
                                column,
                                error,
                            }
                        }
                        _ => VariableDocumentError::Commit(
                            crate::runtime::RuntimeCommitError::StaleContext,
                        ),
                    })?;
                crate::runtime::RuntimeValue::Resource(resource)
            }
        };
        values.insert(name.to_owned(), value);
    }
    Ok(values)
}

fn source_position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(1, |last| last.chars().count() + 1);
    (line, column)
}

pub(super) fn validate_schema_four_mesh_assets(
    imports: &[ImportRecord],
    mesh_assets: &std::collections::BTreeMap<String, MeshAssetRef>,
) -> Result<(), ProjectError> {
    let mut import_ids = std::collections::BTreeSet::new();
    let mut asset_ids = std::collections::BTreeSet::new();
    if imports.iter().any(|import| {
        import.id.is_empty()
            || import.record_version != IMPORT_RECORD_VERSION
            || import.display_name.is_empty()
            || import.mesh_asset_id.is_empty()
            || !import_ids.insert(import.id.as_str())
            || !asset_ids.insert(import.mesh_asset_id.as_str())
            || import
                .source_digest
                .as_deref()
                .is_some_and(|digest| !is_sha256_digest(digest))
            || !mesh_assets.contains_key(&import.mesh_asset_id)
    }) {
        return Err(ProjectError::ManifestInvalid(
            "schema 4 imports require a mesh asset index entry and display name".to_owned(),
        ));
    }
    for (id, asset) in mesh_assets {
        if id != &asset.id
            || asset.revision == 0
            || asset.format_version != panta_mesh::MESH_ASSET_FORMAT_VERSION
            || !is_sha256_digest(&asset.digest)
            || asset.units != "millimeters"
            || !matches!(asset.kind.as_str(), "surface" | "volume")
        {
            return Err(ProjectError::ManifestInvalid(
                "schema 4 mesh asset index contains an invalid entry".to_owned(),
            ));
        }
    }
    Ok(())
}

fn is_sha256_digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// 校验清单内的工程相对资产引用并解析为绝对路径；拒绝绝对路径与
/// `..` 等非普通分量，防止清单注入越出工程包目录。
pub(crate) fn resolve_asset_path(root: &Path, relative: &str) -> Result<PathBuf, ProjectError> {
    let mut paths = crate::path::PathService::new();
    paths
        .set_root(crate::path::RootCategory::Project, root)
        .map_err(|error| ProjectError::ManifestInvalid(error.to_string()))?;
    paths
        .resolve_write_target(&crate::path::ResourceRef {
            category: crate::path::RootCategory::Project,
            relative: relative.to_owned(),
        })
        .map_err(|error| ProjectError::ManifestInvalid(error.to_string()))
}

impl Drop for ProjectService {
    fn drop(&mut self) {
        // 读取取消及过期结果释放不等待解析；已接受的写入由执行通道持有候选。
        self.clear_stl_preview();
        self.replace_runtime_context(None);
        self.activation.advance_generation();
        self.activation.drain();
    }
}
