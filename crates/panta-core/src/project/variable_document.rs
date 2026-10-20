//! 工程变量文档的固定路径与原子文件替换。

use super::{ProjectError, ProjectService, VariableDocumentError};
use crate::runtime::{RuntimeCommitError, RuntimeTaskScope, RuntimeValue, VariableSnapshot};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
const FILE_NAME: &str = "variables.pa";
const TEMPORARY_ATTEMPTS: usize = 32;

pub(super) fn path(project_file: &Path) -> Result<PathBuf, ProjectError> {
    let root = project_file
        .parent()
        .ok_or_else(|| ProjectError::Io("project file has no parent directory".to_owned()))?;
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| ProjectError::Io(format!("{}: {error}", root.display())))?;
    let target = root.join(FILE_NAME);

    // 读取现存文件时拒绝指向工程目录外的链接；写入会以 rename 替换目标，
    // 不会跟随链接，但同样验证它以保持读写路径语义一致。
    if target.exists() {
        let canonical_target = fs::canonicalize(&target)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", target.display())))?;
        if !canonical_target.starts_with(&canonical_root) {
            return Err(ProjectError::Io(format!(
                "variable document escapes project root: {}",
                target.display()
            )));
        }
    }
    Ok(target)
}

pub(super) fn read(path: &Path) -> Result<String, ProjectError> {
    let file = File::open(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => ProjectError::FileMissing(path.display().to_string()),
        _ => ProjectError::Io(format!("{}: {error}", path.display())),
    })?;
    let limit = panta_dsl_core::MAX_SOURCE_BYTES;
    let mut bytes = Vec::with_capacity(limit.saturating_add(1));
    file.take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| ProjectError::Io(format!("{}: {error}", path.display())))?;
    if bytes.len() > limit {
        return Err(ProjectError::ManifestInvalid(format!(
            "{}: variable source exceeds {limit} bytes",
            path.display()
        )));
    }
    String::from_utf8(bytes)
        .map_err(|error| ProjectError::ManifestInvalid(format!("{}: {error}", path.display())))
}

pub(super) fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), ProjectError> {
    write_atomically_with(path, contents, || Ok(()))
}

fn write_atomically_with(
    path: &Path,
    contents: &[u8],
    before_replace: impl FnOnce() -> Result<(), ProjectError>,
) -> Result<(), ProjectError> {
    let (temporary_path, file) = create_temporary(path)?;
    let mut guard = TemporaryFile {
        path: temporary_path.clone(),
        file: Some(file),
        committed: false,
    };
    let staged_file = guard
        .file
        .as_mut()
        .ok_or_else(|| ProjectError::Io("variable staging file closed".to_owned()))?;
    staged_file
        .write_all(contents)
        .and_then(|()| staged_file.sync_all())
        .map_err(|error| ProjectError::Io(format!("{}: {error}", temporary_path.display())))?;
    drop(guard.file.take());
    before_replace()?;
    fs::rename(&temporary_path, path)
        .map_err(|error| ProjectError::Io(format!("{}: {error}", path.display())))?;
    guard.committed = true;
    Ok(())
}

fn create_temporary(target: &Path) -> Result<(PathBuf, File), ProjectError> {
    let parent = target
        .parent()
        .ok_or_else(|| ProjectError::Io("variable document has no parent directory".to_owned()))?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ProjectError::Io("invalid variable document filename".to_owned()))?;
    for _ in 0..TEMPORARY_ATTEMPTS {
        let id = NEXT_TEMPORARY
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| ProjectError::Io("variable staging identifiers exhausted".to_owned()))?;
        let path = parent.join(format!(".{name}-{}-{id}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(ProjectError::Io(format!("{}: {error}", path.display()))),
        }
    }
    Err(ProjectError::Io(
        "variable staging collision limit reached".to_owned(),
    ))
}

struct TemporaryFile {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// 后台变量文件请求；独占 receiver，可以移到等待线程，不能在 GUI 线程阻塞等待。
#[derive(Debug)]
pub struct VariableDocumentRequest {
    identity: Arc<()>,
    scope: RuntimeTaskScope,
    receiver: mpsc::Receiver<Result<PreparedDocument, VariableDocumentError>>,
}

/// 已完成但尚未发布的变量结果；回到应用服务线程后消费。
#[derive(Debug)]
pub struct VariableDocumentCompletion {
    identity: Arc<()>,
    scope: RuntimeTaskScope,
    result: Result<PreparedDocument, VariableDocumentError>,
}

impl VariableDocumentRequest {
    /// 阻塞当前线程直到后台完成；宿主须在独立等待线程调用。
    pub fn wait(self) -> VariableDocumentCompletion {
        VariableDocumentCompletion {
            identity: self.identity,
            scope: self.scope,
            result: self.receiver.recv().unwrap_or_else(|_| {
                Err(VariableDocumentError::Project(ProjectError::Io(
                    "variable document worker disconnected".to_owned(),
                )))
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct PendingVariableDocument {
    identity: Arc<()>,
    pub(super) write: bool,
}

#[derive(Debug)]
pub(super) struct PreparedDocument {
    pub(super) path: PathBuf,
    pub(super) source: String,
    pub(super) values: BTreeMap<String, RuntimeValue>,
}

pub(super) fn prepare_save(
    scope: &RuntimeTaskScope,
    project_path: &Path,
    source: &str,
) -> Result<PreparedDocument, VariableDocumentError> {
    let values = super::service::variable_runtime_values(scope, source)?;
    let source =
        panta_dsl_core::format_source(source).map_err(VariableDocumentError::Diagnostics)?;
    if source.len() > panta_dsl_core::MAX_SOURCE_BYTES {
        return Err(VariableDocumentError::SourceTooLarge {
            size: source.len(),
            limit: panta_dsl_core::MAX_SOURCE_BYTES,
        });
    }
    scope
        .variables()
        .revision()
        .checked_add(1)
        .ok_or(VariableDocumentError::Commit(
            RuntimeCommitError::RevisionExhausted,
        ))?;
    Ok(PreparedDocument {
        path: path(project_path).map_err(VariableDocumentError::Project)?,
        source,
        values,
    })
}

impl ProjectService {
    /// 发起有界后台变量读取或保存；Some(source) 表示保存，None 表示读取。
    /// 保存冻结变量修订并拒绝冲突的工程写入/切换；读取允许切换工程，完成时拒绝迟到结果。
    /// worker 只持有输入和作用域，服务销毁不等待 worker，也不保留对服务的借用。
    pub fn begin_variable_document(
        &mut self,
        expected_generation: u64,
        expected_revision: u64,
        source: Option<String>,
    ) -> Result<VariableDocumentRequest, VariableDocumentError> {
        self.begin_variable_document_with(expected_generation, expected_revision, source, || Ok(()))
    }

    fn begin_variable_document_with(
        &mut self,
        expected_generation: u64,
        expected_revision: u64,
        source: Option<String>,
        before_replace: impl FnOnce() -> Result<(), ProjectError> + Send + 'static,
    ) -> Result<VariableDocumentRequest, VariableDocumentError> {
        let scope = self.variable_scope(expected_generation, expected_revision)?;
        if self.pending_variable.is_some() {
            return Err(VariableDocumentError::Project(
                ProjectError::CommandInvalid("variable document request pending".to_owned()),
            ));
        }
        let project_path = self
            .current
            .as_ref()
            .ok_or(VariableDocumentError::Commit(RuntimeCommitError::NoContext))?
            .path
            .clone();
        let write = source.is_some();
        let executor = if write {
            &self.execution.writes
        } else {
            &self.execution.reads
        };
        let identity = Arc::new(());
        let worker_scope = scope.clone();
        let (sender, receiver) = mpsc::channel();
        executor
            .submit(move || {
                let result = (|| {
                    if worker_scope.is_cancelled() {
                        return Err(VariableDocumentError::Commit(
                            RuntimeCommitError::StaleContext,
                        ));
                    }
                    if let Some(source) = source {
                        let prepared = prepare_save(&worker_scope, &project_path, &source)?;
                        // 原子替换前再次检查会话撤销；已进入的文件系统调用不能强行中断。
                        write_atomically_with(&prepared.path, prepared.source.as_bytes(), || {
                            if worker_scope.is_cancelled() {
                                Err(ProjectError::CommandInvalid(
                                    "variable document scope cancelled".to_owned(),
                                ))
                            } else {
                                before_replace()
                            }
                        })
                        .map_err(VariableDocumentError::Project)?;
                        Ok(prepared)
                    } else {
                        let path = path(&project_path).map_err(VariableDocumentError::Project)?;
                        let source = read(&path).map_err(VariableDocumentError::Project)?;
                        let values =
                            super::service::variable_runtime_values(&worker_scope, &source)?;
                        Ok(PreparedDocument {
                            path,
                            source,
                            values,
                        })
                    }
                })();
                let _ = sender.send(result);
            })
            .map_err(|error| {
                VariableDocumentError::Project(ProjectError::Io(format!(
                    "start variable document request: {error}"
                )))
            })?;
        self.pending_variable = Some(PendingVariableDocument {
            identity: Arc::clone(&identity),
            write,
        });
        Ok(VariableDocumentRequest {
            identity,
            scope,
            receiver,
        })
    }

    /// 在服务线程发布已完成请求。generation/revision 或请求身份过期时保留当前快照。
    pub fn finish_variable_document(
        &mut self,
        completion: VariableDocumentCompletion,
    ) -> Result<(String, VariableSnapshot), VariableDocumentError> {
        if !self
            .pending_variable
            .as_ref()
            .is_some_and(|pending| Arc::ptr_eq(&pending.identity, &completion.identity))
        {
            return Err(VariableDocumentError::Commit(
                RuntimeCommitError::StaleContext,
            ));
        }
        self.pending_variable = None;
        self.variable_scope(
            completion.scope.generation(),
            completion.scope.variables().revision(),
        )?;
        let prepared = completion.result?;
        let snapshot = self
            .commit_runtime_variables(&completion.scope, prepared.values)
            .map_err(VariableDocumentError::Commit)?;
        Ok((prepared.source, snapshot))
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectError, read, write_atomically, write_atomically_with};
    use crate::project::temp_directory::Fixture;
    use std::fs;

    #[test]
    fn failed_pre_replace_keeps_previous_file_and_removes_staging_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let target = fixture.root.join("variables.pa");
        fs::write(&target, b"previous")?;

        let result = write_atomically_with(&target, b"candidate", || {
            Err(ProjectError::Io("injected replace failure".to_owned()))
        });

        assert!(result.is_err());
        assert_eq!(fs::read(&target)?, b"previous");
        assert_eq!(fs::read_dir(&fixture.root)?.count(), 1);
        Ok(())
    }

    #[test]
    fn atomically_replaces_existing_document() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let target = fixture.root.join("variables.pa");
        fs::write(&target, b"previous")?;
        write_atomically(&target, b"updated")?;
        assert_eq!(fs::read(target)?, b"updated");
        Ok(())
    }

    #[test]
    fn variable_document_read_is_bounded_by_the_parser_source_limit()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let target = fixture.root.join("variables.pa");
        fs::write(
            &target,
            vec![b'x'; panta_dsl_core::MAX_SOURCE_BYTES.saturating_add(1)],
        )?;
        assert!(read(&target).is_err());
        Ok(())
    }

    #[test]
    fn project_variable_document_round_trips_and_rejects_stale_writes()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::ProjectService;

        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        let created = service.create(&fixture.root, "Demo")?;
        let source = "version: 1\nkind: variables\n\nvalues:\n  enabled: bool = true\n  divisions: int = 24\n  scale: real = 0.5\n  label: string = Inlet\n  mesh: resource = project:/assets/mesh.vtu\n  half: real = divisions / 2\n";
        let generation = service.variable_document_generation();
        let (saved_source, saved) = service.save_variable_document(generation, 0, source)?;
        assert_eq!(saved.revision(), 1);
        assert_eq!(
            saved.get("half"),
            Some(&crate::runtime::RuntimeValue::Real(12.0))
        );
        assert!(saved_source.contains("half: real = divisions / 2"));
        let variable_path = fixture.root.join("Demo/variables.pa");
        let original_bytes = fs::read(&variable_path)?;

        let stale = service.save_variable_document(generation, 0, source);
        assert!(stale.is_err());
        assert_eq!(fs::read(&variable_path)?, original_bytes);
        assert_eq!(
            service
                .runtime_context()
                .map(|context| context.variables().revision()),
            Some(1)
        );

        let mut reopened = ProjectService::new();
        reopened.open(std::path::Path::new(&created.path))?;
        let (loaded_source, loaded) =
            reopened.load_variable_document(reopened.variable_document_generation(), 0)?;
        assert_eq!(loaded_source, saved_source);
        assert_eq!(loaded.revision(), 1);
        assert_eq!(loaded.get("mesh"), saved.get("mesh"));
        assert_eq!(loaded.get("half"), saved.get("half"));
        Ok(())
    }

    #[test]
    fn invalid_variable_save_preserves_file_and_snapshot() -> Result<(), Box<dyn std::error::Error>>
    {
        use crate::project::ProjectService;

        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Demo")?;
        let source = "version: 1\nkind: variables\n\nvalues:\n  amount: int = 12\n";
        let generation = service.variable_document_generation();
        service.save_variable_document(generation, 0, source)?;
        let variable_path = fixture.root.join("Demo/variables.pa");
        let original_bytes = fs::read(&variable_path)?;

        let invalid = "version: 1\nkind: variables\n\nvalues:\n  amount: int = unknown-name\n";
        assert!(
            service
                .save_variable_document(generation, 1, invalid)
                .is_err()
        );
        assert_eq!(fs::read(&variable_path)?, original_bytes);
        assert_eq!(
            service
                .runtime_context()
                .and_then(|context| context.variables().get("amount")),
            Some(&crate::runtime::RuntimeValue::Integer(12))
        );
        Ok(())
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "解析 1 MiB 边界输入会让解释执行过久；普通测试仍覆盖该上限"
    )]
    fn normalized_source_over_limit_is_rejected_before_creating_file()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::{ProjectService, VariableDocumentError};

        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Large")?;
        let generation = service.variable_document_generation();
        let prefix = "version:1\nkind:variables\nvalues:\n  payload: string = \"";
        let suffix = "\"\n";
        let content_size = panta_dsl_core::MAX_SOURCE_BYTES - prefix.len() - suffix.len();
        let source = format!("{prefix}{}{suffix}", "x".repeat(content_size));
        assert_eq!(source.len(), panta_dsl_core::MAX_SOURCE_BYTES);

        let error = match service.save_variable_document(generation, 0, &source) {
            Ok(_) => return Err("oversized normalized source was accepted".into()),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            VariableDocumentError::SourceTooLarge { .. }
        ));
        assert!(!fixture.root.join("Large").join("variables.pa").exists());
        assert_eq!(
            service
                .runtime_context
                .as_ref()
                .ok_or("runtime context missing after project creation")?
                .capture_task_scope()
                .variables()
                .revision(),
            0
        );
        Ok(())
    }

    #[test]
    fn old_project_generation_cannot_write_into_new_project()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::{ProjectService, VariableDocumentError};

        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "First")?;
        let old_generation = service.variable_document_generation();
        service.create(&fixture.root, "Second")?;
        let source = "version: 1\nkind: variables\n\nvalues:\n  count: int = 7\n";

        let result = service.save_variable_document(old_generation, 0, source);

        assert!(matches!(
            result,
            Err(VariableDocumentError::Commit(
                crate::runtime::RuntimeCommitError::StaleContext
            ))
        ));
        assert!(!fixture.root.join("Second").join("variables.pa").exists());
        assert_eq!(
            service
                .runtime_context
                .as_ref()
                .ok_or("runtime context missing after project creation")?
                .capture_task_scope()
                .variables()
                .revision(),
            0
        );
        Ok(())
    }

    const VALID_SOURCE: &str = "version: 1\nkind: variables\n\nvalues:\n  count: int = 12\n";
    const UPDATED_SOURCE: &str = "version: 1\nkind: variables\n\nvalues:\n  count: int = 99\n";

    fn block_executor(
        executor: &crate::execution::Executor,
    ) -> Result<std::sync::mpsc::Sender<()>, Box<dyn std::error::Error>> {
        let (started, ready) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel();
        executor.submit(move || {
            let _ = started.send(());
            let _ = gate.recv();
        })?;
        ready.recv_timeout(std::time::Duration::from_secs(2))?;
        Ok(release)
    }

    #[test]
    fn background_save_returns_before_io_and_rejects_conflicting_mutations()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::ProjectService;
        use crate::runtime::RuntimeCommitError;
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Async")?;
        let generation = service.variable_document_generation();
        service.save_variable_document(generation, 0, VALID_SOURCE)?;
        let previous = fs::read(fixture.root.join("Async/variables.pa"))?;
        let scope = service
            .runtime_context()
            .ok_or("no runtime")?
            .capture_task_scope();
        let executor = crate::execution::Executor::new("variable-write-test", 1, 2);
        let release = block_executor(&executor)?;
        service.execution.writes = std::sync::Arc::clone(&executor);
        let request =
            service.begin_variable_document(generation, 1, Some(UPDATED_SOURCE.to_owned()))?;
        assert_eq!(
            service
                .runtime_context()
                .ok_or("no runtime")?
                .variables()
                .revision(),
            1
        );
        assert_eq!(fs::read(fixture.root.join("Async/variables.pa"))?, previous);
        assert!(service.create(&fixture.root, "Other").is_err());
        assert!(!fixture.root.join("Other").exists());
        assert_eq!(
            service.commit_runtime_variables(&scope, std::collections::BTreeMap::new()),
            Err(RuntimeCommitError::Busy)
        );
        assert!(
            service
                .save_variable_document(generation, 1, VALID_SOURCE)
                .is_err()
        );
        release.send(())?;
        let (_, snapshot) = service.finish_variable_document(request.wait())?;
        assert_eq!(snapshot.revision(), 2);
        assert_eq!(
            snapshot.get("count"),
            Some(&crate::runtime::RuntimeValue::Integer(99))
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("Async/variables.pa"))?,
            UPDATED_SOURCE
        );
        executor.close_and_wait();
        Ok(())
    }

    #[test]
    fn background_write_and_parse_failures_preserve_file_and_snapshot()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::ProjectService;
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Failure")?;
        let generation = service.variable_document_generation();
        service.save_variable_document(generation, 0, VALID_SOURCE)?;
        let previous = fs::read(fixture.root.join("Failure/variables.pa"))?;
        for source in [UPDATED_SOURCE, "invalid variable document"] {
            let request = service.begin_variable_document_with(
                generation,
                1,
                Some(source.to_owned()),
                || {
                    Err(ProjectError::Io(
                        "injected background replace failure".to_owned(),
                    ))
                },
            )?;
            assert!(service.finish_variable_document(request.wait()).is_err());
            assert_eq!(
                fs::read(fixture.root.join("Failure/variables.pa"))?,
                previous
            );
            assert_eq!(
                service
                    .runtime_context()
                    .ok_or("no runtime")?
                    .variables()
                    .revision(),
                1
            );
            assert_eq!(
                service
                    .runtime_context()
                    .ok_or("no runtime")?
                    .variables()
                    .get("count"),
                Some(&crate::runtime::RuntimeValue::Integer(12))
            );
            assert!(!fs::read_dir(fixture.root.join("Failure"))?.any(|entry| {
                entry.is_ok_and(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".variables.pa-")
                })
            }));
        }
        Ok(())
    }

    #[test]
    fn background_read_rejects_old_revision_and_generation_without_clearing_new_request()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::{ProjectService, VariableDocumentError};
        use crate::runtime::RuntimeCommitError;
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "First")?;
        let generation = service.variable_document_generation();
        service.save_variable_document(generation, 0, VALID_SOURCE)?;
        let request = service.begin_variable_document(generation, 1, None)?;
        let completion = request.wait();
        service.save_variable_document(generation, 1, UPDATED_SOURCE)?;
        assert!(matches!(
            service.finish_variable_document(completion),
            Err(VariableDocumentError::Commit(
                RuntimeCommitError::StaleRevision { .. }
            ))
        ));
        assert_eq!(
            service
                .runtime_context()
                .ok_or("no runtime")?
                .variables()
                .revision(),
            2
        );

        let executor = crate::execution::Executor::new("variable-read-test", 1, 3);
        let release = block_executor(&executor)?;
        service.execution.reads = std::sync::Arc::clone(&executor);
        let old_request = service.begin_variable_document(generation, 2, None)?;
        service.create(&fixture.root, "Second")?;
        let next_generation = service.variable_document_generation();
        service.save_variable_document(next_generation, 0, VALID_SOURCE)?;
        let next_request = service.begin_variable_document(next_generation, 1, None)?;
        release.send(())?;
        assert!(matches!(
            service.finish_variable_document(old_request.wait()),
            Err(VariableDocumentError::Commit(
                RuntimeCommitError::StaleContext
            ))
        ));
        let (_, snapshot) = service.finish_variable_document(next_request.wait())?;
        assert_eq!(snapshot.revision(), 2);
        assert_eq!(
            snapshot.get("count"),
            Some(&crate::runtime::RuntimeValue::Integer(12))
        );
        executor.close_and_wait();
        Ok(())
    }

    #[test]
    fn service_drop_does_not_wait_and_cancels_a_queued_write()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::ProjectService;
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Dropped")?;
        let generation = service.variable_document_generation();
        service.save_variable_document(generation, 0, VALID_SOURCE)?;
        let previous = fs::read(fixture.root.join("Dropped/variables.pa"))?;
        let executor = crate::execution::Executor::new("variable-drop-test", 1, 2);
        let release = block_executor(&executor)?;
        service.execution.writes = std::sync::Arc::clone(&executor);
        let request =
            service.begin_variable_document(generation, 1, Some(UPDATED_SOURCE.to_owned()))?;
        drop(service);
        release.send(())?;
        assert!(request.wait().result.is_err());
        assert_eq!(
            fs::read(fixture.root.join("Dropped/variables.pa"))?,
            previous
        );
        executor.close_and_wait();
        Ok(())
    }

    #[test]
    fn rejected_background_capacity_does_not_reserve_variable_state()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::project::ProjectService;
        let fixture = Fixture::new()?;
        let mut service = ProjectService::new();
        service.create(&fixture.root, "Capacity")?;
        let generation = service.variable_document_generation();
        let executor = crate::execution::Executor::new("variable-capacity-test", 1, 1);
        let release = block_executor(&executor)?;
        service.execution.writes = std::sync::Arc::clone(&executor);
        assert!(
            service
                .begin_variable_document(generation, 0, Some(VALID_SOURCE.to_owned()))
                .is_err()
        );
        assert!(service.pending_variable.is_none());
        assert_eq!(
            service
                .runtime_context()
                .ok_or("no runtime")?
                .variables()
                .revision(),
            0
        );
        assert!(!fixture.root.join("Capacity/variables.pa").exists());
        release.send(())?;
        executor.close_and_wait();
        service.execution.writes = crate::execution::Executor::new("variable-retry-test", 1, 1);
        let request =
            service.begin_variable_document(generation, 0, Some(VALID_SOURCE.to_owned()))?;
        assert_eq!(
            service
                .finish_variable_document(request.wait())?
                .1
                .revision(),
            1
        );
        Ok(())
    }
}
