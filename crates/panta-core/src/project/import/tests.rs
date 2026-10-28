use super::{IMPORT_CANCELLED, IMPORT_COMMITTING, IMPORT_RUNNING, PendingImport, map_import_error};
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
#[cfg_attr(
    miri,
    ignore = "commits a compressed mesh sidecar through zstd native FFI"
)]
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
    assert!(service.resident_mesh_ids().is_empty());
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
    assert!(service.mesh_for_import(&record.id).is_some());
    let asset_path = service
        .current
        .as_ref()
        .and_then(|state| state.mesh_assets.get(&record.mesh_asset_id))
        .map(|asset| asset.relative_path.clone())
        .ok_or("mesh asset index entry missing")?;
    assert!(fixture.root.join("Async").join(asset_path).is_file());
    assert!(!fixture.root.join("Async/assets/imports").exists());

    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.imports()?, vec![record]);
    executor.close_and_wait();
    Ok(())
}

#[test]
#[cfg_attr(
    miri,
    ignore = "commits a compressed mesh sidecar through zstd native FFI"
)]
fn imported_mesh_does_not_depend_on_source_file_after_commit()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let source = fixture.root.join("external-part.stl");
    write_triangle(&source)?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Portable mesh")?;
    let record = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let sidecar_path = service
        .current
        .as_ref()
        .and_then(|state| state.mesh_assets.get(&record.mesh_asset_id))
        .map(|asset| asset.relative_path.clone())
        .ok_or("mesh asset index entry missing")?;
    let sidecar = fixture.root.join("Portable mesh").join(sidecar_path);
    assert!(sidecar.is_file());
    assert!(!fixture.root.join("Portable mesh/assets/imports").exists());

    fs::remove_file(&source)?;
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    let attempt = reopened.begin_asset_activation(&record.id)?;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let outcomes = reopened.drain_asset_activations();
        if let Some(outcome) = outcomes.into_iter().next() {
            assert_eq!(outcome.attempt, attempt.attempt);
            assert_eq!(
                outcome.kind,
                crate::fsm::open_saved_stl::OutcomeKind::Succeeded
            );
            assert_eq!(
                outcome
                    .mesh
                    .ok_or("activated sidecar has no mesh")?
                    .triangles
                    .len(),
                1
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "sidecar activation timed out"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

#[test]
#[cfg_attr(
    miri,
    ignore = "commits a compressed mesh sidecar through zstd native FFI"
)]
fn corrupt_mesh_sidecar_fails_activation_without_publishing_a_mesh()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let source = fixture.root.join("part.stl");
    write_triangle(&source)?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Corrupt sidecar")?;
    let record = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let asset_path = service
        .current
        .as_ref()
        .and_then(|state| state.mesh_assets.get(&record.mesh_asset_id))
        .map(|asset| {
            fixture
                .root
                .join("Corrupt sidecar")
                .join(&asset.relative_path)
        })
        .ok_or("mesh asset index entry missing")?;
    drop(service);

    let mut bytes = fs::read(&asset_path)?;
    *bytes.last_mut().ok_or("mesh sidecar is empty")? ^= 1;
    fs::write(&asset_path, bytes)?;

    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    let attempt = reopened.begin_asset_activation(&record.id)?;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let outcomes = reopened.drain_asset_activations();
        if let Some(outcome) = outcomes.into_iter().next() {
            assert_eq!(outcome.attempt, attempt.attempt);
            assert_eq!(
                outcome.kind,
                crate::fsm::open_saved_stl::OutcomeKind::Failed
            );
            assert_eq!(outcome.code, "project.asset_parse_failed");
            assert!(outcome.mesh.is_none());
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "corrupt sidecar activation timed out"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
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
    assert!(
        !fixture
            .root
            .join("Cancel/assets/meshes/import-1-r1.pmesh")
            .exists()
    );
    assert!(service.imports()?.is_empty());
    assert!(service.resident_mesh_ids().is_empty());
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
#[cfg_attr(
    miri,
    ignore = "commits a compressed mesh sidecar through zstd native FFI"
)]
fn rejected_import_admission_restores_preview_for_retry() -> Result<(), Box<dyn std::error::Error>>
{
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
