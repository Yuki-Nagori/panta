//! 工程包提交租约：协作式排他、磁盘修订复核与独立临时文件。
use super::{PROJECT_SCHEMA_VERSION, ProjectError, ProjectManifest, ProjectState};
use fs4::{FileExt, TryLockError};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
const TEMPORARY_ATTEMPTS: usize = 32;

/// 租约覆盖资产写入、清单提交及失败回滚；销毁时先显式解锁再关闭句柄。
/// 稳定锁文件不能删除，否则另一个等待者可能持有不同 inode。
pub(super) struct WriteLease {
    target: PathBuf,
    expected_revision: Option<u64>,
    lock: File,
}

impl WriteLease {
    pub(super) fn acquire(state: &ProjectState) -> Result<Self, ProjectError> {
        let path = state.path.with_extension("panta.lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", path.display())))?;
        FileExt::try_lock(&lock).map_err(|error| match error {
            TryLockError::WouldBlock => {
                ProjectError::CommandInvalid("project write pending".to_owned())
            }
            TryLockError::Error(error) => ProjectError::Io(format!("{}: {error}", path.display())),
        })?;
        let lease = Self {
            target: state.path.clone(),
            expected_revision: state.persisted_revision,
            lock,
        };
        lease.check_disk_revision()?;
        Ok(lease)
    }

    fn check_disk_revision(&self) -> Result<(), ProjectError> {
        let Some(expected) = self.expected_revision else {
            return if self.target.exists() {
                Err(ProjectError::AlreadyExists(
                    self.target.display().to_string(),
                ))
            } else {
                Ok(())
            };
        };
        let bytes = fs::read(&self.target)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", self.target.display())))?;
        let manifest: ProjectManifest = serde_json::from_slice(&bytes).map_err(|error| {
            ProjectError::ManifestInvalid(format!("{}: {error}", self.target.display()))
        })?;
        if manifest.revision != expected {
            return Err(ProjectError::CommandInvalid(
                "project changed on disk".to_owned(),
            ));
        }
        Ok(())
    }

    /// rename 成功就是提交点；其后不再执行会改变提交结果的可失败操作。
    /// sync_all 保证文件内容同步，不承诺各平台断电后的目录项持久性。
    pub(super) fn commit(&self, state: &mut ProjectState) -> Result<(), ProjectError> {
        self.commit_with_before_replace(state, || Ok(()))
    }

    /// 在同目录候选已写入并同步后、原子替换前执行提交裁决。
    /// 取消调用方可在此处关闭取消窗口，避免提交成功但服务丢弃结果。
    pub(super) fn commit_with_before_replace(
        &self,
        state: &mut ProjectState,
        before_replace: impl FnOnce() -> Result<(), ProjectError>,
    ) -> Result<(), ProjectError> {
        if state.path != self.target || state.persisted_revision != self.expected_revision {
            return Err(ProjectError::CommandInvalid(
                "project write lease changed".to_owned(),
            ));
        }
        self.check_disk_revision()?;
        let manifest = ProjectManifest {
            schema: PROJECT_SCHEMA_VERSION,
            name: state.name.clone(),
            revision: state.revision,
            imports: state.imports.clone(),
            mesh_assets: state.mesh_assets.clone(),
            analysis_sequences: state.analysis_sequences.clone(),
            materials: state.materials.clone(),
            fill_settings: state.fill_settings.clone(),
            gate_location_settings: state.gate_location_settings.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| ProjectError::Io(format!("serialize manifest: {error}")))?;
        let mut temporary =
            Temporary::create(self.target.with_extension("panta.write"), &NEXT_TEMPORARY)?;
        let file = temporary
            .file
            .as_mut()
            .ok_or_else(|| ProjectError::Io("temporary file closed".to_owned()))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| ProjectError::Io(format!("{}: {error}", temporary.path.display())))?;
        drop(temporary.file.take());
        before_replace()?;
        fs::rename(&temporary.path, &self.target)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", self.target.display())))?;
        temporary.committed = true;
        state.persisted_revision = Some(state.revision);
        Ok(())
    }
}

impl Drop for WriteLease {
    fn drop(&mut self) {
        // fork / dup 可保留同一 open file description，单独 close 不能及时释放锁。
        // 提交点已由 rename 裁定，解锁失败仅记录诊断；不把已提交状态改报失败。
        if let Err(error) = FileExt::unlock(&self.lock) {
            let _ = writeln!(
                std::io::stderr().lock(),
                "panta: release project write lease {}: {error}",
                self.target.display()
            );
        }
    }
}

struct Temporary {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl Temporary {
    fn create(directory: PathBuf, counter: &AtomicU64) -> Result<Self, ProjectError> {
        fs::create_dir_all(&directory)
            .map_err(|error| ProjectError::Io(format!("{}: {error}", directory.display())))?;
        // create_new 不覆盖遗留文件；碰撞重试有上限，避免在途租约无限等待。
        for _ in 0..TEMPORARY_ATTEMPTS {
            let id = counter
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .map_err(|_| ProjectError::Io("temporary file identifiers exhausted".to_owned()))?;
            let path = directory.join(format!("{}-{id}.tmp", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                        committed: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(ProjectError::Io(format!("{}: {error}", path.display()))),
            }
        }
        Err(ProjectError::Io(
            "temporary file collision limit reached".to_owned(),
        ))
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests;
