//! 工程服务和 STL 导入的行为回归。
use panta_core::project::{
    IMPORT_RECORD_VERSION, PROJECT_SCHEMA_VERSION, ProjectCommand, ProjectError, ProjectService,
    STL_IMPORT_PARSER_VERSION,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("panta-project-{}-{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn create_command_save_open_and_model_command_round_trip() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let created = service.create(&fixture.root, "Demo")?;
    assert_eq!(created.name, "Demo");
    assert!(!created.dirty);
    assert_eq!(created.path, fixture.root.join("Demo/Demo.panta"));
    assert!(created.path.is_file());

    let changed = service.execute(ProjectCommand::Rename {
        name: "Renamed".to_owned(),
    })?;
    assert_eq!(changed.name, "Renamed");
    assert!(changed.dirty);
    let saved = service.save()?;
    assert!(!saved.dirty);

    let mut reopened = ProjectService::new();
    let opened = reopened.open(&created.path)?;
    assert_eq!(opened.name, "Renamed");
    assert_eq!(opened.revision, 1);
    assert!(!opened.dirty);
    Ok(())
}

#[test]
fn rejects_invalid_names_and_existing_targets() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    for name in [
        "",
        "../escape",
        "CON",
        "bad/child",
        "trailing.",
        "Demo.panta",
    ] {
        let error = match service.create(&fixture.root, name) {
            Ok(snapshot) => panic!("invalid name unexpectedly created: {snapshot:?}"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            ProjectError::EmptyName | ProjectError::InvalidName(_)
        ));
    }
    service.create(&fixture.root, "Demo")?;
    let error = match service.create(&fixture.root, "Demo") {
        Ok(snapshot) => panic!("existing target unexpectedly created: {snapshot:?}"),
        Err(error) => error,
    };
    assert!(matches!(error, ProjectError::AlreadyExists(_)));
    Ok(())
}

#[test]
fn rejects_unsupported_manifest_schema() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let project = fixture.root.join("Demo");
    fs::create_dir_all(&project)?;
    let project_file = project.join("Demo.panta");
    fs::write(
        &project_file,
        format!(
            r#"{{"schema":{},"name":"Demo","revision":0}}"#,
            PROJECT_SCHEMA_VERSION + 1
        ),
    )?;
    let error = match ProjectService::new().open(&project_file) {
        Ok(snapshot) => panic!("unsupported schema opened: {snapshot:?}"),
        Err(error) => error,
    };
    assert!(matches!(error, ProjectError::UnsupportedSchema(_)));
    Ok(())
}

#[test]
fn accepts_only_panta_project_files() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let project = fixture.root.join("Demo");
    fs::create_dir_all(&project)?;
    let other_file = project.join("Demo.json");
    fs::write(other_file, br#"{"schema":1,"name":"Demo","revision":0}"#)?;
    let error = match ProjectService::new().open(&project.join("Demo.json")) {
        Ok(snapshot) => panic!("non-panta file opened: {snapshot:?}"),
        Err(error) => error,
    };
    assert!(matches!(error, ProjectError::InvalidFile(_)));
    Ok(())
}

#[test]
fn reports_empty_or_relative_locations_and_missing_project_state()
-> Result<(), Box<dyn std::error::Error>> {
    let mut service = ProjectService::new();
    assert!(matches!(service.current(), Err(ProjectError::NoProject)));
    assert!(matches!(service.save(), Err(ProjectError::NoProject)));
    assert!(matches!(
        service.execute(ProjectCommand::Rename {
            name: "Demo".to_owned(),
        }),
        Err(ProjectError::NoProject)
    ));

    assert!(matches!(
        service.create(Path::new(""), "Demo"),
        Err(ProjectError::LocationEmpty)
    ));
    assert!(matches!(
        service.create(Path::new("relative"), "Demo"),
        Err(ProjectError::LocationNotAbsolute(_))
    ));

    let fixture = Fixture::new()?;
    let file_location = fixture.root.join("location-file");
    fs::write(&file_location, b"not a directory")?;
    assert!(matches!(
        service.create(&file_location, "Demo"),
        Err(ProjectError::LocationCreateFailed(_))
    ));
    assert!(matches!(
        service.create(&file_location.join("child"), "Demo"),
        Err(ProjectError::LocationCreateFailed(_))
    ));
    Ok(())
}

#[test]
fn validates_platform_names_and_command_failures() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let mut invalid_names = vec![
        ".".to_owned(),
        "..".to_owned(),
        "bad/child".to_owned(),
        r"bad\child".to_owned(),
        "bad:child".to_owned(),
        "trailing.".to_owned(),
        "trailing ".to_owned(),
        "Demo.PANTA".to_owned(),
        "CON.txt".to_owned(),
        "LPT9.log".to_owned(),
        "line\nfeed".to_owned(),
    ];
    invalid_names.push("x".repeat(256));
    for name in invalid_names {
        assert!(matches!(
            service.create(&fixture.root, &name),
            Err(ProjectError::InvalidName(_))
        ));
    }

    let created = service.create(&fixture.root, "Demo.v1")?;
    assert_eq!(service.current()?, created);
    assert!(matches!(
        service.execute(ProjectCommand::Rename {
            name: "Demo.v1".to_owned(),
        }),
        Err(ProjectError::CommandInvalid(_))
    ));
    assert!(matches!(
        service.execute(ProjectCommand::Rename {
            name: "bad/name".to_owned(),
        }),
        Err(ProjectError::InvalidName(_))
    ));
    Ok(())
}

#[test]
fn opens_invalid_manifests_and_accepts_case_insensitive_extension()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let project = fixture.root.join("Demo");
    fs::create_dir_all(&project)?;

    let mut service = ProjectService::new();
    assert!(matches!(
        service.open(Path::new("relative.panta")),
        Err(ProjectError::FileMissing(_))
    ));
    assert!(matches!(
        service.open(&fixture.root.join("missing.panta")),
        Err(ProjectError::FileMissing(_))
    ));
    let directory_with_extension = project.join("directory.panta");
    fs::create_dir(&directory_with_extension)?;
    assert!(matches!(
        service.open(&directory_with_extension),
        Err(ProjectError::FileMissing(_))
    ));

    let malformed = project.join("malformed.panta");
    fs::write(&malformed, b"not json")?;
    assert!(matches!(
        service.open(&malformed),
        Err(ProjectError::ManifestInvalid(_))
    ));

    let invalid_name = project.join("invalid-name.panta");
    fs::write(
        &invalid_name,
        br#"{"schema":1,"name":"bad/name","revision":0}"#,
    )?;
    assert!(matches!(
        service.open(&invalid_name),
        Err(ProjectError::InvalidName(_))
    ));

    let uppercase = project.join("Upper.PANTA");
    fs::write(&uppercase, br#"{"schema":1,"name":"Upper","revision":2}"#)?;
    let opened = service.open(&uppercase)?;
    assert_eq!(opened.name, "Upper");
    assert_eq!(opened.revision, 2);
    Ok(())
}

#[test]
fn imports_ascii_stl_and_round_trips_record_and_asset() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("Case.STL");
    fs::write(
            &source,
            b"solid case\n facet normal 0 0 1\n  outer loop\n   vertex 1 2 3\n   vertex 5 2 3\n   vertex 1 8 6\n  endloop\n endfacet\nendsolid case\n",
        )?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record = service.import_stl(&source, "dual-domain", "millimeters", true)?;
    assert_eq!(record.record_version, IMPORT_RECORD_VERSION);
    assert_eq!(record.parser_version, STL_IMPORT_PARSER_VERSION);
    assert_eq!(record.id, "import-1");
    assert_eq!(record.source_name, "Case.STL");
    assert_eq!(record.asset, "assets/imports/0001-Case.STL");
    assert_eq!(record.mesh_type, "dual-domain");
    assert_eq!(record.units, "millimeters");
    assert_eq!(record.triangle_count, 1);
    assert_eq!(record.dimensions, [4.0, 6.0, 3.0]);
    assert!(fixture.root.join("Demo").join(&record.asset).is_file());
    assert!(!service.current()?.dirty);

    let project_path = fixture.root.join("Demo/Demo.panta");
    let mut reopened = ProjectService::new();
    reopened.open(&project_path)?;
    assert_eq!(reopened.imports()?, vec![record]);
    assert_eq!(reopened.current_mesh(), service.current_mesh());
    Ok(())
}

#[test]
fn changed_preview_requires_review_and_failed_import_preserves_mesh()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("changing.stl");
    fs::write(&source, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    assert_eq!(service.inspect_stl(&source)?.dimensions, [1.0, 1.0, 0.0]);
    fs::write(&source, b"vertex 0 0 0\nvertex 2 0 0\nvertex 0 1 0\n")?;
    assert!(matches!(
        service.import_stl(&source, "solid-3d", "millimeters", false),
        Err(ProjectError::ImportSourceChanged(_))
    ));
    assert_eq!(service.current()?.revision, 0);
    assert!(service.current_mesh().is_none());

    service.inspect_stl(&source)?;
    service.import_stl(&source, "solid-3d", "centimeters", false)?;
    assert_eq!(
        service.current_mesh().map(|mesh| mesh.summary().dimensions),
        Some([20.0, 10.0, 0.0])
    );
    let committed = service.current_mesh().cloned();
    let revision = service.current()?.revision;
    fs::write(&source, b"not an stl")?;
    assert!(matches!(
        service.import_stl(&source, "solid-3d", "millimeters", false),
        Err(ProjectError::ImportParseFailed(_))
    ));
    assert_eq!(service.current()?.revision, revision);
    assert_eq!(service.current_mesh().cloned(), committed);
    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    assert_eq!(reopened.current_mesh().cloned(), committed);

    let second = fixture.root.join("second.stl");
    fs::write(&second, b"vertex 0 0 0\nvertex 3 0 0\nvertex 0 4 0\n")?;
    let second_record = service.import_stl(&second, "solid-3d", "millimeters", false)?;
    assert_eq!(
        service.current_mesh().map(|mesh| mesh.summary().dimensions),
        Some([3.0, 4.0, 0.0])
    );
    assert_eq!(service.imports()?.len(), 2);
    assert!(
        fixture
            .root
            .join("Demo/assets/imports/0001-changing.stl")
            .is_file()
    );
    assert!(
        fixture
            .root
            .join("Demo")
            .join(&second_record.asset)
            .is_file()
    );
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    assert_eq!(reopened.current_mesh(), service.current_mesh());
    Ok(())
}

#[test]
fn converted_coordinate_overflow_leaves_project_unchanged() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let source = fixture.root.join("large.stl");
    fs::write(
        &source,
        format!("vertex {} 0 0\nvertex 0 0 0\nvertex 0 1 0\n", f64::MAX),
    )?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    assert!(matches!(
        service.import_stl(&source, "solid-3d", "inches", false),
        Err(ProjectError::ImportParseFailed(_))
    ));
    assert_eq!(service.current()?.revision, 0);
    assert!(service.imports()?.is_empty());
    assert!(service.current_mesh().is_none());
    Ok(())
}

#[test]
fn inspects_binary_stl() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("Binary.STL");
    let vertices = [[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 4.0]];
    let mut bytes = vec![0_u8; 84 + 50];
    bytes[80..84].copy_from_slice(&1_u32.to_le_bytes());
    for (index, vertex) in vertices.into_iter().enumerate() {
        let offset = 84 + 12 + index * 12;
        for (axis, value) in vertex.into_iter().enumerate() {
            let value_offset = offset + axis * 4;
            bytes[value_offset..value_offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    fs::write(&source, bytes)?;

    let preview = ProjectService::new().inspect_stl(&source)?;
    assert_eq!(preview.source_name, "Binary.STL");
    assert_eq!(preview.triangle_count, 1);
    assert_eq!(preview.dimensions, [2.0, 3.0, 4.0]);
    Ok(())
}

#[test]
fn failed_manifest_commit_preserves_previous_asset_and_mesh()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let created = service.create(&fixture.root, "Demo")?;
    service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let snapshot = service.current()?;
    let mesh = service.current_mesh().cloned();
    let manifest = fs::read(&created.path)?;
    fs::create_dir(created.path.with_extension("panta.tmp"))?;
    assert!(
        service
            .import_stl(&source, "solid-3d", "millimeters", false)
            .is_err()
    );
    assert_eq!(service.current()?, snapshot);
    assert_eq!(service.current_mesh().cloned(), mesh);
    assert_eq!(service.imports()?.len(), 1);
    assert_eq!(fs::read(&created.path)?, manifest);
    assert!(
        !fixture
            .root
            .join("Demo/assets/imports/0002-part.stl")
            .exists()
    );
    Ok(())
}

#[test]
fn rejects_invalid_stl_import_requests_without_project_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("case.stl");
    fs::write(&source, b"not an stl")?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let revision = service.current()?.revision;
    assert!(matches!(
        service.import_stl(&source, "unknown", "millimeters", false),
        Err(ProjectError::ImportParseFailed(_))
    ));
    assert!(matches!(
        service.import_stl(&source, "dual-domain", "unknown", false),
        Err(ProjectError::ImportParseFailed(_))
    ));
    assert!(matches!(
        service.import_stl(
            &fixture.root.join("case.obj"),
            "dual-domain",
            "millimeters",
            false
        ),
        Err(ProjectError::ImportFileMissing(_))
    ));
    assert_eq!(service.current()?.revision, revision);
    assert!(service.imports()?.is_empty());
    Ok(())
}

#[test]
fn handles_manifest_revision_limits_and_save_io_failures() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let project = fixture.root.join("Max");
    fs::create_dir_all(&project)?;
    let project_file = project.join("Max.panta");
    fs::write(
        &project_file,
        format!(
            r#"{{"schema":{},"name":"Max","revision":{}}}"#,
            PROJECT_SCHEMA_VERSION,
            u64::MAX
        ),
    )?;
    let mut service = ProjectService::new();
    service.open(&project_file)?;
    let snapshot = service.execute(ProjectCommand::Rename {
        name: "MaxRenamed".to_owned(),
    })?;
    assert_eq!(snapshot.revision, u64::MAX);

    fs::remove_file(&project_file)?;
    fs::create_dir(&project_file)?;
    assert!(matches!(service.save(), Err(ProjectError::Io(_))));
    Ok(())
}

#[test]
fn error_display_uses_stable_codes_and_details() {
    let errors = [
        (ProjectError::EmptyName, "project.empty_name"),
        (
            ProjectError::InvalidName("bad".to_owned()),
            "project.invalid_name: bad",
        ),
        (ProjectError::LocationEmpty, "project.location_empty"),
        (
            ProjectError::LocationNotAbsolute("relative".to_owned()),
            "project.location_not_absolute: relative",
        ),
        (
            ProjectError::LocationCreateFailed("location".to_owned()),
            "project.location_create_failed: location",
        ),
        (
            ProjectError::FileMissing("missing".to_owned()),
            "project.file_missing: missing",
        ),
        (
            ProjectError::InvalidFile("file".to_owned()),
            "project.invalid_file: file",
        ),
        (
            ProjectError::AlreadyExists("existing".to_owned()),
            "project.already_exists: existing",
        ),
        (
            ProjectError::ManifestInvalid("manifest".to_owned()),
            "project.manifest_invalid: manifest",
        ),
        (
            ProjectError::UnsupportedSchema(7),
            "project.unsupported_schema: 7",
        ),
        (ProjectError::NoProject, "project.no_project"),
        (
            ProjectError::CommandInvalid("unchanged".to_owned()),
            "project.command_invalid: unchanged",
        ),
        (
            ProjectError::ImportFileMissing("missing.stl".to_owned()),
            "project.import_file_missing: missing.stl",
        ),
        (
            ProjectError::ImportInvalidFile("mesh.obj".to_owned()),
            "project.import_invalid_file: mesh.obj",
        ),
        (
            ProjectError::ImportUnsupportedMeshType("shell".to_owned()),
            "project.import_unsupported_mesh_type: shell",
        ),
        (
            ProjectError::ImportUnsupportedUnits("unknown".to_owned()),
            "project.import_unsupported_units: unknown",
        ),
        (
            ProjectError::ImportParseFailed("invalid".to_owned()),
            "project.import_parse_failed: invalid",
        ),
        (
            ProjectError::ImportAssetCopyFailed("copy".to_owned()),
            "project.import_asset_copy_failed: copy",
        ),
        (ProjectError::Io("disk".to_owned()), "project.io: disk"),
    ];
    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
    }
}
