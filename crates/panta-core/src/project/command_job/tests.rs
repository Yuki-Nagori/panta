use super::*;
use crate::execution::Executor;
use crate::project::ProjectCommand;
use crate::project::temp_directory::Fixture;
use std::time::Duration;

#[test]
fn create_publishes_only_after_wait_and_rejects_conflicting_commands()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let before = service.create(&fixture.root, "Old")?;
    let scope = service
        .runtime_context()
        .ok_or("no context")?
        .capture_task_scope();
    let request = service.begin_project_operation(ProjectOperation::Create {
        location: fixture.root.clone(),
        name: "New".into(),
    })?;
    assert_eq!(service.current()?, before);
    assert!(service.save().is_err());
    assert!(service.open(&before.path).is_err());
    assert!(
        service
            .execute(ProjectCommand::Rename {
                name: "Wrong".into()
            })
            .is_err()
    );
    assert!(
        service
            .begin_project_operation(ProjectOperation::Save)
            .is_err()
    );
    assert!(
        service
            .begin_variable_document(scope.generation(), 0, None)
            .is_err()
    );
    assert_eq!(
        service.commit_runtime_variables(&scope, BTreeMap::new()),
        Err(crate::runtime::RuntimeCommitError::Busy)
    );
    let completion = std::thread::spawn(move || request.wait())
        .join()
        .map_err(|_| "wait panicked")?;
    assert_eq!(service.current()?, before);
    let created = service.finish_project_operation(completion)?;
    assert_eq!(created.name, "New");
    assert!(scope.is_cancelled());
    assert_eq!(
        service.variable_document_generation(),
        scope.generation() + 1
    );
    assert!(service.pending_project.is_none());
    Ok(())
}

#[test]
fn failed_open_preserves_current_project_and_runtime_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let before = service.create(&fixture.root, "Kept")?;
    let scope = service
        .runtime_context()
        .ok_or("no context")?
        .capture_task_scope();
    let completion = service
        .begin_project_operation(ProjectOperation::Open {
            path: fixture.root.join("missing.panta"),
        })?
        .wait();
    assert!(service.finish_project_operation(completion).is_err());
    assert_eq!(service.current()?, before);
    assert!(!scope.is_cancelled());
    assert!(service.pending_project.is_none());
    service.save()?;
    Ok(())
}

#[test]
fn open_reuses_schema_and_asset_checks_and_replaces_only_after_finish()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut source = ProjectService::new();
    let saved = source.create(&fixture.root, "Opened")?;
    let mut service = ProjectService::new();
    let before = service.create(&fixture.root, "Before")?;
    let completion = service
        .begin_project_operation(ProjectOperation::Open {
            path: saved.path.clone(),
        })?
        .wait();
    assert_eq!(service.current()?, before);
    assert_eq!(service.finish_project_operation(completion)?, saved);
    fs::write(&saved.path, "{\"schema\": 3}")?;
    let completion = service
        .begin_project_operation(ProjectOperation::Open { path: saved.path })?
        .wait();
    assert!(matches!(
        service.finish_project_operation(completion),
        Err(ProjectError::UnsupportedSchema(3))
    ));
    Ok(())
}

#[test]
fn save_failure_retains_dirty_candidate_and_previous_file() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Save")?;
    let original = fs::read(&project.path)?;
    service.execute(ProjectCommand::Rename {
        name: "Dirty".into(),
    })?;
    let before = service.current()?;
    let staging = project.path.with_extension("panta.write");
    fs::remove_dir(&staging)?;
    fs::write(&staging, "blocked")?;
    let completion = service
        .begin_project_operation(ProjectOperation::Save)?
        .wait();
    assert!(service.finish_project_operation(completion).is_err());
    assert_eq!(service.current()?, before);
    assert_eq!(fs::read(&project.path)?, original);
    assert!(service.pending_project.is_none());
    fs::remove_file(staging)?;
    let completion = service
        .begin_project_operation(ProjectOperation::Save)?
        .wait();
    assert!(!service.finish_project_operation(completion)?.dirty);
    let mut opened = ProjectService::new();
    assert_eq!(opened.open(&project.path)?.name, "Dirty");
    Ok(())
}

#[test]
fn foreign_completion_does_not_unlock_matching_request_and_stale_target_is_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut first = ProjectService::new();
    first.create(&fixture.root, "First")?;
    let mut second = ProjectService::new();
    let before = second.create(&fixture.root, "Second")?;
    let foreign = first
        .begin_project_operation(ProjectOperation::Save)?
        .wait();
    let local = second
        .begin_project_operation(ProjectOperation::Save)?
        .wait();
    assert!(second.finish_project_operation(foreign).is_err());
    assert!(second.pending_project.is_some());
    assert_eq!(second.finish_project_operation(local)?, before);
    let local = second
        .begin_project_operation(ProjectOperation::Save)?
        .wait();
    second.current.as_mut().ok_or("missing state")?.revision += 1;
    assert!(second.finish_project_operation(local).is_err());
    assert_eq!(second.current()?.revision, before.revision + 1);
    assert!(second.pending_project.is_none());
    Ok(())
}

#[test]
fn capacity_or_closed_executor_rejects_without_sticking_pending_state()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let before = service.create(&fixture.root, "Capacity")?;
    let executor = Executor::new("project-command-capacity", 1, 1);
    let (started, ready) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    executor.submit(move || {
        let _ = started.send(());
        let _ = gate.recv();
    })?;
    ready.recv_timeout(Duration::from_secs(2))?;
    service.execution.writes = Arc::clone(&executor);
    assert!(
        service
            .begin_project_operation(ProjectOperation::Save)
            .is_err()
    );
    assert!(service.pending_project.is_none());
    assert_eq!(service.current()?, before);
    release.send(())?;
    executor.close_and_wait();
    assert!(
        service
            .begin_project_operation(ProjectOperation::Save)
            .is_err()
    );
    assert!(service.pending_project.is_none());
    Ok(())
}

#[test]
fn accepted_save_finishes_after_service_drop_without_joining_gui()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Drop")?;
    service.execute(ProjectCommand::Rename {
        name: "Saved".into(),
    })?;
    let executor = Executor::new("project-command-drop", 1, 2);
    let (started, ready) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    executor.submit(move || {
        let _ = started.send(());
        let _ = gate.recv();
    })?;
    ready.recv_timeout(Duration::from_secs(2))?;
    service.execution.writes = executor;
    let request = service.begin_project_operation(ProjectOperation::Save)?;
    drop(service);
    release.send(())?;
    assert!(request.wait().result.is_ok());
    let mut opened = ProjectService::new();
    assert_eq!(opened.open(&project.path)?.name, "Saved");
    Ok(())
}

#[test]
fn sequence_validates_frozen_identity_and_persists_in_worker()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Sequence")?;
    // 序列命令只需要既有导入身份；资产读取规则由打开/激活测试覆盖。
    service
        .current
        .as_mut()
        .ok_or("no state")?
        .imports
        .push(super::super::ImportRecord {
            record_version: super::super::IMPORT_RECORD_VERSION,
            parser_version: 1,
            id: "part".into(),
            display_name: "part".into(),
            source_name: "part.stl".into(),
            source_path: String::new(),
            source_digest: None,
            source_size_bytes: 0,
            source_modified_unix_ms: None,
            mesh_asset_id: "mesh".into(),
            format: "stl".into(),
            mesh_type: "midplane".into(),
            units: "millimeters".into(),
            show_import_log: false,
            triangle_count: 1,
            dimensions: [1.0, 1.0, 0.0],
        });
    for (revision, sequence, valid) in [
        (99, "cool", false),
        (0, "unknown", false),
        (0, "cool", true),
        (1, "cool", true),
    ] {
        let before = service.current()?;
        let completion = service
            .begin_project_operation(ProjectOperation::SetAnalysisSequence {
                expected_path: project.path.clone(),
                expected_revision: revision,
                import_id: "part".into(),
                sequence_id: sequence.into(),
            })?
            .wait();
        assert_eq!(service.current()?, before);
        let result = service.finish_project_operation(completion);
        assert_eq!(result.is_ok(), valid);
    }
    assert_eq!(service.current()?.revision, 1);
    let manifest: ProjectManifest = serde_json::from_slice(&fs::read(project.path)?)?;
    assert_eq!(
        manifest.analysis_sequences.get("part").map(String::as_str),
        Some("cool")
    );
    Ok(())
}

#[test]
fn variable_save_blocks_project_admission_and_read_completion_cannot_publish_during_switch()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Variables")?;
    let generation = service.variable_document_generation();
    let source = "version: 1\nkind: variables\nvalues:\n  count: int = 1\n";
    let variable = service.begin_variable_document(generation, 0, Some(source.into()))?;
    assert!(
        service
            .begin_project_operation(ProjectOperation::Save)
            .is_err()
    );
    service.finish_variable_document(variable.wait())?;
    let variable = service.begin_variable_document(generation, 1, None)?;
    let project = service.begin_project_operation(ProjectOperation::Create {
        location: fixture.root.clone(),
        name: "Replacement".into(),
    })?;
    assert!(service.finish_variable_document(variable.wait()).is_err());
    service.finish_project_operation(project.wait())?;
    assert!(
        service
            .runtime_context()
            .ok_or("no context")?
            .variables()
            .is_empty()
    );
    Ok(())
}

#[test]
fn disconnected_worker_clears_admission_without_changing_the_project()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let before = service.create(&fixture.root, "Disconnect")?;
    let identity = Arc::new(());
    let (sender, receiver) = mpsc::channel();
    drop(sender);
    service.pending_project = Some(PendingProjectOperation {
        identity: Arc::clone(&identity),
        expected: Some(before.clone()),
    });
    let completion = ProjectOperationRequest { identity, receiver }.wait();
    let error = service
        .finish_project_operation(completion)
        .err()
        .ok_or("missing error")?;
    assert_eq!(error.detail(), "project operation worker disconnected");
    assert_eq!(service.current()?, before);
    assert!(service.pending_project.is_none());
    service.save()?;
    Ok(())
}

#[test]
fn absent_project_or_failed_creation_never_replaces_current_state()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    assert!(matches!(
        service.begin_project_operation(ProjectOperation::Save),
        Err(ProjectError::NoProject)
    ));
    assert!(service.pending_project.is_none());
    let completion = service
        .begin_project_operation(ProjectOperation::Create {
            location: fixture.root.clone(),
            name: "invalid/name".into(),
        })?
        .wait();
    assert!(service.finish_project_operation(completion).is_err());
    assert!(matches!(service.current(), Err(ProjectError::NoProject)));
    assert!(service.runtime_context().is_none());
    let before = service.create(&fixture.root, "Existing")?;
    let manifest = fs::read(&before.path)?;
    let completion = service
        .begin_project_operation(ProjectOperation::Create {
            location: fixture.root.clone(),
            name: "Existing".into(),
        })?
        .wait();
    assert!(matches!(
        service.finish_project_operation(completion),
        Err(ProjectError::AlreadyExists(_))
    ));
    assert_eq!(service.current()?, before);
    assert_eq!(fs::read(before.path)?, manifest);
    Ok(())
}
