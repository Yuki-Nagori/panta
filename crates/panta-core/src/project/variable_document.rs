//! 工程变量文档的固定路径与原子文件替换。

use super::ProjectError;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
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
}
