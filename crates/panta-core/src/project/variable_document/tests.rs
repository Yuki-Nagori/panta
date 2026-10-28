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
fn invalid_variable_save_preserves_file_and_snapshot() -> Result<(), Box<dyn std::error::Error>> {
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
fn old_project_generation_cannot_write_into_new_project() -> Result<(), Box<dyn std::error::Error>>
{
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
        let request =
            service.begin_variable_document_with(generation, 1, Some(source.to_owned()), || {
                Err(ProjectError::Io(
                    "injected background replace failure".to_owned(),
                ))
            })?;
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
fn service_drop_does_not_wait_and_cancels_a_queued_write() -> Result<(), Box<dyn std::error::Error>>
{
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
    let request = service.begin_variable_document(generation, 0, Some(VALID_SOURCE.to_owned()))?;
    assert_eq!(
        service
            .finish_variable_document(request.wait())?
            .1
            .revision(),
        1
    );
    Ok(())
}
