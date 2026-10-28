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
mod tests;
