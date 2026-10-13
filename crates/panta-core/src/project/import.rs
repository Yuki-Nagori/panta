//! STL 导入的工程事务；读取、预览和解析由 panta-import 负责。
use super::model::next_revision;
use super::repository::WriteLease;
use super::{ImportRecord, ProjectError, ProjectService, StlImportPreview};
use panta_import::{CheckedReadError, PreparedStl, StlImportSession};
use panta_import::{ImportError, ImportOptions};
use std::io::Write;
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
    /// 启动确认导入事务。源复核、资产复制和清单提交都由 Rust 写入 worker 执行。
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
        self.latest_mesh_id = Some(record.id.clone());
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
    let record = prepared.record(import_number, options);
    prepared.scale_mesh_mm(options).map_err(map_import_error)?;
    candidate.imports.push(record.clone());
    let lease = super::repository::WriteLease::acquire(candidate)?;
    let project_root = candidate
        .path
        .parent()
        .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
    let asset_dir = project_root.join("assets").join("imports");
    fs::create_dir_all(&asset_dir).map_err(|error| {
        ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_dir.display()))
    })?;
    let asset_path = asset_dir.join(prepared.asset_name(import_number));
    let mut asset = write_import_asset(&prepared, &asset_path, phase)?;
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
    committed: bool,
}

impl Drop for ImportAsset {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn write_import_asset(
    prepared: &PreparedStl,
    asset_path: &Path,
    phase: &AtomicU8,
) -> Result<ImportAsset, ProjectError> {
    let mut asset = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(asset_path)
        .map_err(|error| {
            ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_path.display()))
        })?;
    let cleanup = ImportAsset {
        path: asset_path.to_path_buf(),
        committed: false,
    };
    let result = (|| {
        for chunk in prepared.bytes.chunks(256 * 1024) {
            ensure_import_running(phase)?;
            asset.write_all(chunk).map_err(|error| {
                ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_path.display()))
            })?;
        }
        ensure_import_running(phase)?;
        asset.sync_all().map_err(|error| {
            ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_path.display()))
        })
    })();
    drop(asset);
    result?;
    Ok(cleanup)
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
    /// 校验、复制并持久化当前工程包中的一次 STL 导入。
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
        let revision = next_revision(
            self.current
                .as_ref()
                .ok_or(ProjectError::NoProject)?
                .revision,
        )?;
        let mut prepared = self
            .import_session
            .prepare(source)
            .map_err(map_import_error)?;
        let options =
            ImportOptions::new(mesh_type, units, show_import_log).map_err(map_import_error)?;
        let import_number = self
            .current
            .as_ref()
            .ok_or(ProjectError::NoProject)?
            .imports
            .len()
            + 1;
        let record = prepared.record(import_number, options);
        prepared.scale_mesh_mm(options).map_err(map_import_error)?;
        let state = self.current.as_mut().ok_or(ProjectError::NoProject)?;
        let lease = WriteLease::acquire(state)?;
        let project_root = state
            .path
            .parent()
            .ok_or_else(|| ProjectError::Io("project has no package directory".to_owned()))?;
        let asset_dir = project_root.join("assets").join("imports");
        fs::create_dir_all(&asset_dir).map_err(|error| {
            ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_dir.display()))
        })?;
        let asset_name = prepared.asset_name(import_number);
        let asset_path = asset_dir.join(&asset_name);
        let mut asset = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&asset_path)
            .map_err(|error| {
                ProjectError::ImportAssetCopyFailed(format!("{}: {error}", asset_path.display()))
            })?;
        let written = asset
            .write_all(&prepared.bytes)
            .and_then(|()| asset.sync_all());
        drop(asset);
        if let Err(error) = written {
            let _ = fs::remove_file(&asset_path);
            return Err(ProjectError::ImportAssetCopyFailed(format!(
                "{}: {error}",
                asset_path.display()
            )));
        }

        let previous_revision = state.revision;
        let previous_dirty = state.dirty;
        state.imports.push(record.clone());
        state.revision = revision;
        state.dirty = false;
        if let Err(error) = lease.commit(state) {
            state.imports.pop();
            state.revision = previous_revision;
            state.dirty = previous_dirty;
            let _ = fs::remove_file(&asset_path);
            return Err(error);
        }
        self.latest_mesh_id = Some(record.id.clone());
        self.mesh_cache
            .insert_active(record.id.clone(), prepared.mesh);
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
mod tests {
    use super::{
        IMPORT_CANCELLED, IMPORT_COMMITTING, IMPORT_RUNNING, PendingImport, map_import_error,
    };
    use crate::execution::Executor;
    use crate::project::ProjectService;
    use crate::project::temp_directory;
    use panta_import::ImportError;
    use panta_mesh::StlError;
    use std::fs;
    use std::sync::{Arc, atomic::Ordering, mpsc};
    use std::time::Duration;

    fn write_triangle(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(path, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
        Ok(())
    }

    fn finish_preview(
        service: &mut ProjectService,
        request: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while service.finish_stl_preview(request)?.is_none() {
            assert!(std::time::Instant::now() < deadline, "preview timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn finish_import(
        service: &mut ProjectService,
        request: u64,
    ) -> Result<super::ImportRecord, Box<dyn std::error::Error>> {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(record) = service.finish_stl_import(request)? {
                return Ok(record);
            }
            assert!(std::time::Instant::now() < deadline, "import timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn block_executor(
        executor: &Executor,
    ) -> Result<(mpsc::Sender<()>, mpsc::Receiver<()>), Box<dyn std::error::Error>> {
        let (started, ready) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        executor.submit(move || {
            let _ = started.send(());
            let _ = gate.recv();
        })?;
        ready.recv_timeout(Duration::from_secs(2))?;
        Ok((release, ready))
    }

    #[test]
    fn import_failures_map_to_stable_project_diagnostics() {
        let cases = [
            (
                ImportError::Missing("part.stl".into()),
                "project.import_file_missing",
                "missing",
                "part.stl",
            ),
            (
                ImportError::UnsupportedFormat("part.step".into()),
                "project.import_invalid_file",
                "format",
                "part.step",
            ),
            (
                ImportError::Read("disk full".into()),
                "project.import_parse_failed",
                "format",
                "disk full",
            ),
            (
                ImportError::Parse(StlError::InvalidData),
                "project.import_parse_failed",
                "format",
                "InvalidData",
            ),
            (
                ImportError::SourceChanged("part.stl".into()),
                "project.import_source_changed",
                "conflict",
                "part.stl",
            ),
            (
                ImportError::UnsupportedMeshType("unknown".into()),
                "project.import_unsupported_mesh_type",
                "format",
                "unknown",
            ),
            (
                ImportError::UnsupportedUnits("yards".into()),
                "project.import_unsupported_units",
                "format",
                "yards",
            ),
            (
                ImportError::CoordinateOverflow,
                "project.import_parse_failed",
                "format",
                "millimeter coordinate overflow",
            ),
        ];

        for (error, code, category, detail) in cases {
            let mapped = map_import_error(error);
            assert_eq!(mapped.code(), code);
            assert_eq!(mapped.category(), category);
            assert_eq!(mapped.detail(), detail);
        }
    }

    #[test]
    fn async_import_publishes_only_after_atomic_manifest_commit()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let source = fixture.root.join("part.stl");
        write_triangle(&source)?;
        let mut service = ProjectService::new();
        let project = service.create(&fixture.root, "Async")?;
        let manifest = fs::read(&project.path)?;
        let preview = service.begin_stl_preview(&source)?;
        finish_preview(&mut service, preview)?;
        let executor = Executor::new("import-commit-test", 1, 2);
        let (release, _) = block_executor(&executor)?;
        service.execution.writes = Arc::clone(&executor);

        let request = service.begin_stl_import(&source, "solid-3d", "millimeters", false)?;
        assert_eq!(fs::read(&project.path)?, manifest);
        assert!(service.imports()?.is_empty());
        assert_eq!(service.current()?.revision, 0);
        assert!(service.current_mesh().is_none());
        assert!(
            service
                .execute(crate::project::ProjectCommand::Rename {
                    name: "Blocked".into(),
                })
                .is_err()
        );

        release.send(())?;
        let record = finish_import(&mut service, request)?;
        assert_eq!(record.id, "import-1");
        assert_eq!(service.current()?.revision, 1);
        assert_eq!(service.imports()?, vec![record.clone()]);
        assert!(service.current_mesh().is_some());
        assert!(fixture.root.join("Async").join(&record.asset).is_file());

        let mut reopened = ProjectService::new();
        reopened.open(&project.path)?;
        assert_eq!(reopened.imports()?, vec![record]);
        executor.close_and_wait();
        Ok(())
    }

    #[test]
    fn queued_async_import_can_be_cancelled_without_asset_or_manifest_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let source = fixture.root.join("part.stl");
        write_triangle(&source)?;
        let mut service = ProjectService::new();
        let project = service.create(&fixture.root, "Cancel")?;
        let manifest = fs::read(&project.path)?;
        let preview = service.begin_stl_preview(&source)?;
        finish_preview(&mut service, preview)?;
        let executor = Executor::new("import-cancel-test", 1, 2);
        let (release, _) = block_executor(&executor)?;
        service.execution.writes = Arc::clone(&executor);

        let request = service.begin_stl_import(&source, "solid-3d", "millimeters", false)?;
        assert!(service.cancel_stl_import(request));
        assert!(service.finish_stl_import(request).is_err());
        release.send(())?;
        executor.close_and_wait();
        assert_eq!(fs::read(&project.path)?, manifest);
        assert!(
            !fixture
                .root
                .join("Cancel/assets/imports/0001-part.stl")
                .exists()
        );
        assert!(service.imports()?.is_empty());
        assert!(service.current_mesh().is_none());
        Ok(())
    }

    #[test]
    fn import_cancel_is_rejected_after_the_manifest_commit_gate() {
        let phase = Arc::new(std::sync::atomic::AtomicU8::new(IMPORT_RUNNING));
        assert!(
            phase
                .compare_exchange(
                    IMPORT_RUNNING,
                    IMPORT_COMMITTING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
        );
        let pending = PendingImport {
            request: 1,
            phase,
            receiver: mpsc::channel().1,
        };
        assert!(!pending.request_cancel());
        assert_ne!(IMPORT_CANCELLED, pending.phase.load(Ordering::Acquire));
        assert_eq!(IMPORT_COMMITTING, pending.phase.load(Ordering::Acquire));
    }

    #[test]
    fn rejected_import_admission_restores_preview_for_retry()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let source = fixture.root.join("part.stl");
        write_triangle(&source)?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Retry")?;
        let preview = service.begin_stl_preview(&source)?;
        finish_preview(&mut service, preview)?;
        let executor = Executor::new("import-admission-test", 1, 1);
        let (release, _) = block_executor(&executor)?;
        service.execution.writes = Arc::clone(&executor);

        assert!(
            service
                .begin_stl_import(&source, "solid-3d", "millimeters", false)
                .is_err()
        );
        assert!(service.import_session.has_preview_for(&source));
        release.send(())?;
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let request = loop {
            match service.begin_stl_import(&source, "solid-3d", "millimeters", false) {
                Ok(request) => break request,
                Err(error) if error.detail().contains("background capacity exhausted") => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "executor did not drain"
                    );
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => return Err(error.into()),
            }
        };
        finish_import(&mut service, request)?;
        executor.close_and_wait();
        assert_eq!(service.imports()?.len(), 1);
        Ok(())
    }
}
