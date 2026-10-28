use super::super::{ProjectError, ProjectService, temp_directory};
use super::{TEMPORARY_ATTEMPTS, Temporary};
use std::fs;
use std::sync::atomic::AtomicU64;

#[test]
fn temporary_candidates_are_distinct_and_cleanup_preserves_unowned_files()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let directory = fixture.root.join("staging");
    fs::create_dir(&directory)?;
    let unowned = directory.join("foreign.tmp");
    fs::write(&unowned, "preserve")?;
    let counter = AtomicU64::new(0);
    let first = Temporary::create(directory.clone(), &counter)?;
    let second = Temporary::create(directory, &counter)?;
    assert_ne!(first.path, second.path);
    let first_path = first.path.clone();
    let second_path = second.path.clone();
    drop(first);
    assert!(!first_path.exists());
    assert!(second_path.exists());
    assert_eq!(fs::read_to_string(&unowned)?, "preserve");
    drop(second);
    assert!(!second_path.exists());
    assert_eq!(fs::read_to_string(unowned)?, "preserve");
    Ok(())
}

#[test]
fn temporary_creation_retries_collisions_without_overwriting()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let counter = AtomicU64::new(0);
    let occupied = fixture.root.join(format!("{}-0.tmp", std::process::id()));
    fs::write(&occupied, "preserve")?;
    let temporary = Temporary::create(fixture.root.clone(), &counter)?;
    assert_eq!(
        temporary.path,
        fixture.root.join(format!("{}-1.tmp", std::process::id()))
    );
    drop(temporary);
    assert_eq!(fs::read_to_string(occupied)?, "preserve");
    Ok(())
}

#[test]
fn temporary_creation_reports_a_non_directory_parent_without_overwriting()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let blocked_directory = fixture.root.join("staging");
    fs::write(&blocked_directory, "preserve")?;
    let error = Temporary::create(blocked_directory.clone(), &AtomicU64::new(0))
        .err()
        .ok_or("file unexpectedly accepted as a staging directory")?;

    assert!(matches!(error, ProjectError::Io(_)));
    assert_eq!(fs::read_to_string(blocked_directory)?, "preserve");
    Ok(())
}

#[test]
fn commit_rejects_state_that_no_longer_matches_its_lease() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    let snapshot = service.create(&fixture.root, "Demo")?;
    let manifest_before = fs::read(&snapshot.path)?;
    let mut state = service
        .current
        .take()
        .ok_or("created project state missing")?;
    let lease = super::WriteLease::acquire(&state)?;
    state.persisted_revision = Some(state.revision + 1);

    assert!(matches!(
        lease.commit(&mut state),
        Err(ProjectError::CommandInvalid(message))
            if message == "project write lease changed"
    ));
    assert_eq!(fs::read(&snapshot.path)?, manifest_before);
    Ok(())
}

#[test]
fn commit_does_not_recreate_a_manifest_removed_during_lease()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    let snapshot = service.create(&fixture.root, "Removed manifest")?;
    let mut state = service
        .current
        .take()
        .ok_or("created project state missing")?;
    let lease = super::WriteLease::acquire(&state)?;
    fs::remove_file(&snapshot.path)?;
    state.name = "Must not be written".into();
    state.revision += 1;

    assert!(matches!(lease.commit(&mut state), Err(ProjectError::Io(_))));
    assert!(!snapshot.path.exists());

    Ok(())
}

#[test]
fn commit_rejects_a_manifest_revision_changed_during_lease()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    let snapshot = service.create(&fixture.root, "Changed revision")?;
    let mut state = service
        .current
        .take()
        .ok_or("created project state missing")?;
    let lease = super::WriteLease::acquire(&state)?;
    let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&snapshot.path)?)?;
    manifest["revision"] = serde_json::Value::from(state.revision + 1);
    let external_update = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&snapshot.path, &external_update)?;
    state.name = "Must not overwrite disk".into();
    state.revision += 1;

    let error = lease
        .commit(&mut state)
        .err()
        .ok_or("stale write lease unexpectedly committed")?;

    assert!(matches!(error, ProjectError::CommandInvalid(_)));
    assert_eq!(error.code(), "project.command_invalid");
    assert_eq!(error.detail(), "project changed on disk");
    assert_eq!(fs::read(snapshot.path)?, external_update);
    Ok(())
}

#[test]
fn temporary_creation_bounds_collisions_and_identifier_exhaustion()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    for id in 0..TEMPORARY_ATTEMPTS {
        fs::write(
            fixture
                .root
                .join(format!("{}-{id}.tmp", std::process::id())),
            "preserve",
        )?;
    }
    let counter = AtomicU64::new(0);
    let error = Temporary::create(fixture.root.clone(), &counter)
        .err()
        .ok_or("colliding candidates unexpectedly succeeded")?;
    assert_eq!(error.detail(), "temporary file collision limit reached");
    assert_eq!(
        fs::read_dir(&fixture.root)?
            .collect::<Result<Vec<_>, _>>()?
            .len(),
        TEMPORARY_ATTEMPTS
    );
    let exhausted = AtomicU64::new(u64::MAX);
    let error = Temporary::create(fixture.root.clone(), &exhausted)
        .err()
        .ok_or("exhausted identifiers unexpectedly succeeded")?;
    assert_eq!(error.detail(), "temporary file identifiers exhausted");
    for id in 0..TEMPORARY_ATTEMPTS {
        assert_eq!(
            fs::read_to_string(
                fixture
                    .root
                    .join(format!("{}-{id}.tmp", std::process::id()))
            )?,
            "preserve"
        );
    }
    Ok(())
}

#[test]
fn lease_release_is_not_prolonged_by_a_duplicated_file_handle()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut service = super::super::ProjectService::new();
    let project = service.create(&fixture.root, "Duplicated")?;
    let mut state = service.current.as_ref().ok_or("missing project")?.clone();
    let lease = super::WriteLease::acquire(&state)?;
    // dup 与 fork 继承共享 open file description，控制其存活期即可复现竞态。
    let duplicated = lease.lock.try_clone()?;
    let probe = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(project.path.with_extension("panta.lock"))?;
    assert!(matches!(
        fs4::FileExt::try_lock(&probe),
        Err(fs4::TryLockError::WouldBlock)
    ));
    state.name = "Committed".into();
    state.revision += 1;
    lease.commit(&mut state)?;
    assert!(matches!(
        fs4::FileExt::try_lock(&probe),
        Err(fs4::TryLockError::WouldBlock)
    ));
    drop(lease);
    fs4::FileExt::try_lock(&probe)?;
    drop(duplicated);
    let competing = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(project.path.with_extension("panta.lock"))?;
    assert!(matches!(
        fs4::FileExt::try_lock(&competing),
        Err(fs4::TryLockError::WouldBlock)
    ));
    fs4::FileExt::unlock(&probe)?;
    let mut reopened = super::super::ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.current()?.name, "Committed");
    assert_eq!(reopened.current()?.revision, 1);
    Ok(())
}

#[test]
fn lease_acquisition_rejects_a_target_created_after_project_creation()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Raced create")?;
    let mut state = service.current.as_ref().ok_or("missing project")?.clone();
    state.persisted_revision = None;

    let error = super::WriteLease::acquire(&state)
        .err()
        .ok_or("existing project target unexpectedly acquired a create lease")?;

    assert!(matches!(error, ProjectError::AlreadyExists(_)));
    Ok(())
}

#[test]
fn lease_acquisition_rejects_a_corrupted_project_manifest() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Corrupted manifest")?;
    let state = service.current.as_ref().ok_or("missing project")?.clone();
    fs::write(&project.path, b"not a project manifest")?;

    let error = super::WriteLease::acquire(&state)
        .err()
        .ok_or("corrupted project manifest unexpectedly acquired a lease")?;

    assert!(matches!(error, ProjectError::ManifestInvalid(_)));
    Ok(())
}

#[test]
fn lease_acquisition_reports_an_unopenable_lock_path() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_directory::Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Blocked lock")?;
    let state = service.current.as_ref().ok_or("missing project")?.clone();
    let lock_path = project.path.with_extension("panta.lock");

    fs::remove_file(&lock_path)?;
    fs::create_dir(&lock_path)?;
    let error = super::WriteLease::acquire(&state)
        .err()
        .ok_or("directory unexpectedly opened as a lock file")?;

    assert!(matches!(error, ProjectError::Io(_)));
    assert!(error.detail().contains(&lock_path.display().to_string()));
    assert!(project.path.is_file());
    Ok(())
}
