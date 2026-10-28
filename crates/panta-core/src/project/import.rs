//! STL 导入的工程事务；读取、预览和解析由 panta-import 负责。
use super::model::{MeshAssetRef, next_revision};
use super::{ImportRecord, ProjectError, ProjectService, StlImportPreview, mesh_assets};
use panta_import::{CheckedReadError, StlImportSession};
use panta_import::{ImportError, ImportOptions};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};
use std::{fs, path::Path};

const IMPORT_RUNNING: u8 = 0;
const IMPORT_COMMITTING: u8 = 1;
const IMPORT_CANCELLED: u8 = 2;
const IMPORT_COMMITTED: u8 = 3;

#[derive(Debug)]
pub(super) struct PendingImport {
    request: u64,
    phase: Arc<AtomicU8>,
    receiver: Receiver<
        Result<(super::ProjectState, ImportRecord, panta_mesh::SurfaceMesh), ProjectError>,
    >,
}

impl PendingImport {
    fn request_cancel(&self) -> bool {
        self.phase
            .compare_exchange(
                IMPORT_RUNNING,
                IMPORT_CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}

impl Drop for PendingImport {
    fn drop(&mut self) {
        self.request_cancel();
    }
}

impl ProjectService {
    /// 启动确认导入事务。源复核、Mesh sidecar 写入和清单提交都由 Rust 写入 worker 执行。
    /// 成功提交只有在 `finish_stl_import` 消费结果后才发布到内存模型。
    pub fn begin_stl_import(
        &mut self,
        source: &Path,
        mesh_type: &str,
        units: &str,
        show_import_log: bool,
    ) -> Result<u64, ProjectError> {
        self.ensure_project_writable()?;
        if self.pending_preview.is_some() {
            return Err(ProjectError::CommandInvalid(
                "STL preview pending".to_owned(),
            ));
        }
        if self.current.is_none() {
            return Err(ProjectError::NoProject);
        }
        if !self.import_session.has_preview_for(source) {
            return Err(ProjectError::CommandInvalid(
                "STL preview is not ready for the selected source".to_owned(),
            ));
        }
        let request = self
            .import_request
            .checked_add(1)
            .ok_or_else(|| ProjectError::CommandInvalid("import request exhausted".to_owned()))?;
        let options =
            ImportOptions::new(mesh_type, units, show_import_log).map_err(map_import_error)?;
        let import_number = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .imports
            .len()
            + 1;
        let revision = next_revision(
            self.current
                .as_ref()
                .ok_or(ProjectError::NoProject)?
                .revision,
        )?;
        let mut candidate = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .clone();
        candidate.revision = revision;
        candidate.dirty = false;

        // Keep ownership recoverable if bounded admission rejects the job.
        let session = Arc::new(Mutex::new(Some(std::mem::take(&mut self.import_session))));
        let worker_session = Arc::clone(&session);
        let phase = Arc::new(AtomicU8::new(IMPORT_RUNNING));
        let worker_phase = Arc::clone(&phase);
        let source = source.to_path_buf();
        let (sender, receiver) = mpsc::channel();
        let submitted = self.execution.writes.submit(move || {
            let session = worker_session
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
            let result = match session {
                Some(mut session) => commit_import(
                    &mut candidate,
                    &mut session,
                    &source,
                    options,
                    import_number,
                    &worker_phase,
                ),
                None => Err(ProjectError::Io(
                    "STL import session unavailable".to_owned(),
                )),
            };
            if let Err(mpsc::SendError(Ok(_))) = sender.send(result) {
                eprintln!("panta: STL import committed after service teardown");
            }
        });
        if let Err(error) = submitted {
            self.import_session = session
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take()
                .unwrap_or_default();
            return Err(ProjectError::Io(format!("start STL import: {error}")));
        }

        self.import_request = request;
        self.pending_import = Some(PendingImport {
            request,
            phase,
            receiver,
        });
        Ok(request)
    }

    /// `None` 表示仍在执行；成功结果发布已提交修订和 Mesh，错误保留旧状态。
    pub fn finish_stl_import(
        &mut self,
        request: u64,
    ) -> Result<Option<ImportRecord>, ProjectError> {
        let pending = self
            .pending_import
            .as_ref()
            .filter(|pending| pending.request == request)
            .ok_or_else(|| {
                ProjectError::CommandInvalid("no matching STL import pending".to_owned())
            })?;
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => Err(ProjectError::Io(
                "STL import worker disconnected".to_owned(),
            )),
        };
        self.pending_import = None;
        let (state, record, mesh) = result?;
        self.mesh_cache.insert_active(record.id.clone(), mesh);
        self.current = Some(state);
        Ok(Some(record))
    }

    /// 取消只在 manifest 替换前接受；到达提交阶段后调用方必须消费结果。
    pub fn cancel_stl_import(&mut self, request: u64) -> bool {
        let Some(pending) = self
            .pending_import
            .as_ref()
            .filter(|pending| pending.request == request)
        else {
            return false;
        };
        if !pending.request_cancel() {
            return false;
        }
        self.pending_import = None;
        true
    }
}

fn commit_import(
    candidate: &mut super::ProjectState,
    session: &mut StlImportSession,
    source: &Path,
    options: ImportOptions,
    import_number: usize,
    phase: &AtomicU8,
) -> Result<(super::ProjectState, ImportRecord, panta_mesh::SurfaceMesh), ProjectError> {
    let mut prepared = session
        .prepare_checked(source, || phase.load(Ordering::Acquire) == IMPORT_CANCELLED)
        .map_err(|error| match error {
            CheckedReadError::Failed(error) => map_import_error(error),
            CheckedReadError::Cancelled(_) => import_cancelled(),
        })?;
    ensure_import_running(phase)?;
    let mut record = prepared.record(import_number, options);
    drop(std::mem::take(&mut prepared.bytes));
    prepared.scale_mesh_mm(options).map_err(map_import_error)?;
    let lease = super::repository::WriteLease::acquire(candidate)?;
    let project_root = candidate
        .path
        .parent()
        .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
    let (asset_reference, asset_path, created) =
        write_import_mesh_asset(project_root, &mut record, &prepared.mesh, Some(phase))?;
    candidate
        .mesh_assets
        .insert(asset_reference.id.clone(), asset_reference);
    candidate.imports.push(record.clone());
    let mut asset = ImportAsset {
        path: asset_path,
        created,
        committed: false,
    };
    let commit_result = lease.commit_with_before_replace(candidate, || {
        phase
            .compare_exchange(
                IMPORT_RUNNING,
                IMPORT_COMMITTING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| import_cancelled())
    });
    commit_result?;
    asset.committed = true;
    phase.store(IMPORT_COMMITTED, Ordering::Release);
    Ok((candidate.clone(), record, prepared.mesh))
}

struct ImportAsset {
    path: std::path::PathBuf,
    created: bool,
    committed: bool,
}

impl Drop for ImportAsset {
    fn drop(&mut self) {
        if self.created && !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn write_import_mesh_asset(
    project_root: &Path,
    record: &mut ImportRecord,
    mesh: &panta_mesh::SurfaceMesh,
    phase: Option<&AtomicU8>,
) -> Result<(MeshAssetRef, std::path::PathBuf, bool), ProjectError> {
    let base_id = record.mesh_asset_id.clone();
    for attempt in 0..32 {
        if let Some(phase) = phase {
            ensure_import_running(phase)?;
        }
        let asset_id = if attempt == 0 {
            base_id.clone()
        } else {
            format!("{base_id}-recovery-{attempt}")
        };
        record.mesh_asset_id = asset_id;
        let mut asset_reference = new_surface_asset_ref(record, "millimeters");
        let asset_path =
            super::service::resolve_asset_path(project_root, &asset_reference.relative_path)
                .map_err(|error| ProjectError::ImportAssetWriteFailed(error.to_string()))?;
        let write_result = mesh_assets::write_surface_mesh_asset_or_reuse(&asset_path, mesh)
            .map_err(|error| ProjectError::ImportAssetWriteFailed(error.to_string()))?;
        if let Some((digest, created)) = write_result {
            asset_reference.digest = digest;
            return Ok((asset_reference, asset_path, created));
        }
    }
    record.mesh_asset_id = base_id;
    Err(ProjectError::ImportAssetWriteFailed(
        "mesh asset recovery path limit reached".to_owned(),
    ))
}

fn ensure_import_running(phase: &AtomicU8) -> Result<(), ProjectError> {
    if phase.load(Ordering::Acquire) == IMPORT_RUNNING {
        Ok(())
    } else {
        Err(import_cancelled())
    }
}

fn import_cancelled() -> ProjectError {
    ProjectError::CommandInvalid("STL import cancelled".to_owned())
}

impl ProjectService {
    /// 校验来源快照并将 Mesh sidecar 与导入记录提交到当前工程。
    /// 返回成功前先写入清单，重新打开工程时无需依赖后续显式保存即可恢复。
    pub fn import_stl(
        &mut self,
        source: &Path,
        mesh_type: &str,
        units: &str,
        show_import_log: bool,
    ) -> Result<ImportRecord, ProjectError> {
        self.ensure_project_writable()?;
        if self.pending_preview.is_some() {
            return Err(ProjectError::CommandInvalid(
                "STL preview pending".to_owned(),
            ));
        }
        if self.current.is_none() {
            return Err(ProjectError::NoProject);
        }
        let options =
            ImportOptions::new(mesh_type, units, show_import_log).map_err(map_import_error)?;
        let import_number = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .imports
            .len()
            + 1;
        let revision = next_revision(
            self.current
                .as_ref()
                .ok_or(ProjectError::NoProject)?
                .revision,
        )?;
        let mut candidate = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .clone();
        candidate.revision = revision;
        candidate.dirty = false;
        let phase = AtomicU8::new(IMPORT_RUNNING);
        let (candidate, record, mesh) = commit_import(
            &mut candidate,
            &mut self.import_session,
            source,
            options,
            import_number,
            &phase,
        )?;
        self.current = Some(candidate);
        self.mesh_cache.insert_active(record.id.clone(), mesh);
        Ok(record)
    }

    /// 同步领域入口，供无 GUI 的调用方使用；Qt 使用后台 begin / finish 入口。
    pub fn inspect_stl(&mut self, source: &Path) -> Result<StlImportPreview, ProjectError> {
        self.clear_stl_preview();
        self.import_session
            .preview(source)
            .map_err(map_import_error)
    }
}

fn new_surface_asset_ref(record: &ImportRecord, units: &str) -> MeshAssetRef {
    MeshAssetRef {
        id: record.mesh_asset_id.clone(),
        revision: 1,
        format_version: panta_mesh::MESH_ASSET_FORMAT_VERSION,
        kind: "surface".to_owned(),
        relative_path: format!("assets/meshes/{}-r1.pmesh", record.mesh_asset_id),
        digest: String::new(),
        units: units.to_owned(),
        source_revision: record.source_digest.clone(),
    }
}

pub(super) fn map_import_error(error: ImportError) -> ProjectError {
    match error {
        ImportError::Missing(path) => ProjectError::ImportFileMissing(path),
        ImportError::UnsupportedFormat(path) => ProjectError::ImportInvalidFile(path),
        ImportError::Read(detail) => ProjectError::ImportParseFailed(detail),
        ImportError::Parse(detail) => ProjectError::ImportParseFailed(detail.to_string()),
        ImportError::SourceChanged(path) => ProjectError::ImportSourceChanged(path),
        ImportError::UnsupportedMeshType(kind) => ProjectError::ImportUnsupportedMeshType(kind),
        ImportError::UnsupportedUnits(units) => ProjectError::ImportUnsupportedUnits(units),
        ImportError::CoordinateOverflow => {
            ProjectError::ImportParseFailed("millimeter coordinate overflow".to_owned())
        }
    }
}

#[cfg(test)]
mod tests;
