//! 工程服务和 STL 导入的行为回归。
use panta_core::project::{
    FillSettings, IMPORT_RECORD_VERSION, PROJECT_SCHEMA_VERSION, ProjectCommand, ProjectError,
    ProjectService, STL_IMPORT_PARSER_VERSION,
};
use std::fs;
use std::path::Path;

#[path = "../support/rust/temp_directory.rs"]
mod temp_directory;
use temp_directory::Fixture;

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
fn rejects_analysis_sequences_with_missing_imports_or_unknown_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("sequence.stl");
    fs::write(
        &source,
        b"solid sequence\n facet normal 0 0 1\n  outer loop\n   vertex 0 0 0\n   vertex 1 0 0\n   vertex 0 1 0\n  endloop\n endfacet\nendsolid sequence\n",
    )?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Invalid sequence")?;
    let imported = service.import_stl(&source, "solid-3d", "millimeters", true)?;
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&project.path)?)?;

    let mut manifest = original.clone();
    manifest["analysis_sequences"] = serde_json::json!({"missing-import": "fill"});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(matches!(
        service.open(&project.path),
        Err(ProjectError::ManifestInvalid(_))
    ));

    let mut manifest = original;
    manifest["analysis_sequences"] = serde_json::json!({imported.id: "unknown-sequence"});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(matches!(
        service.open(&project.path),
        Err(ProjectError::ManifestInvalid(_))
    ));
    Ok(())
}

#[test]
fn rejects_fill_settings_for_missing_import_without_replacing_current_project()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Invalid fill settings")?;
    let before = service.current()?;
    let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&project.path)?)?;
    manifest["fill_settings"] = serde_json::json!({
        "missing-import": FillSettings::default()
    });
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;

    let error = service
        .open(&project.path)
        .err()
        .ok_or("fill settings for a missing import were accepted")?;

    assert!(matches!(error, ProjectError::ManifestInvalid(_)));
    assert!(
        error
            .detail()
            .contains("fill settings reference missing import missing-import")
    );
    assert_eq!(service.current()?, before);
    Ok(())
}

#[test]
fn rejects_materials_with_missing_imports_or_unknown_ids() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let source = fixture.root.join("material.stl");
    fs::write(
        &source,
        b"solid material\n facet normal 0 0 1\n  outer loop\n   vertex 0 0 0\n   vertex 1 0 0\n   vertex 0 1 0\n  endloop\n endfacet\nendsolid material\n",
    )?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Invalid material")?;
    let imported = service.import_stl(&source, "solid-3d", "millimeters", true)?;
    let before = service.current()?;
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&project.path)?)?;
    let default_material_id = panta_core::project::default_material()?.id.clone();

    let mut manifest = original.clone();
    manifest["materials"] = serde_json::json!({"missing-import": default_material_id});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(matches!(
        service.open(&project.path),
        Err(ProjectError::ManifestInvalid(_))
    ));
    assert_eq!(service.current()?, before);

    let mut manifest = original;
    manifest["materials"] = serde_json::json!({imported.id: "unknown-material"});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(matches!(
        service.open(&project.path),
        Err(ProjectError::ManifestInvalid(_))
    ));
    assert_eq!(service.current()?, before);
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
    // 打开只读清单；已保存资产由只读激活按需恢复（见 activation.rs）。
    assert!(reopened.current_mesh().is_none());
    Ok(())
}

#[test]
fn import_asset_collision_does_not_overwrite_or_publish_import()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let created = service.create(&fixture.root, "Demo")?;
    let asset_dir = fixture.root.join("Demo/assets/imports");
    fs::create_dir_all(&asset_dir)?;
    let occupied_asset = asset_dir.join("0001-part.stl");
    fs::write(&occupied_asset, b"preserve this unrelated file")?;
    let before = service.current()?;
    let manifest_before = fs::read(&created.path)?;

    assert!(matches!(
        service.import_stl(&source, "solid-3d", "millimeters", false),
        Err(ProjectError::ImportAssetCopyFailed(_))
    ));
    assert_eq!(fs::read(&occupied_asset)?, b"preserve this unrelated file");
    assert_eq!(service.current()?, before);
    assert!(service.imports()?.is_empty());
    assert!(service.current_mesh().is_none());
    assert_eq!(fs::read(&created.path)?, manifest_before);
    Ok(())
}

#[test]
fn import_directory_failure_preserves_the_project() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let created = service.create(&fixture.root, "Demo")?;
    let blocking_file = fixture.root.join("Demo/assets");
    fs::write(&blocking_file, b"preserve this file")?;
    let before = service.current()?;
    let manifest_before = fs::read(&created.path)?;

    assert!(matches!(
        service.import_stl(&source, "solid-3d", "millimeters", false),
        Err(ProjectError::ImportAssetCopyFailed(_))
    ));
    assert_eq!(fs::read(&blocking_file)?, b"preserve this file");
    assert_eq!(service.current()?, before);
    assert!(service.imports()?.is_empty());
    assert!(service.current_mesh().is_none());
    assert_eq!(fs::read(&created.path)?, manifest_before);
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
    assert!(reopened.current_mesh().is_none());

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
    assert!(reopened.current_mesh().is_none());
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
    fs::remove_dir(created.path.with_extension("panta.write"))?;
    fs::write(
        created.path.with_extension("panta.write"),
        b"blocked staging directory",
    )?;
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
            u64::MAX - 1
        ),
    )?;
    let mut service = ProjectService::new();
    service.open(&project_file)?;
    let snapshot = service.execute(ProjectCommand::Rename {
        name: "MaxRenamed".to_owned(),
    })?;
    assert_eq!(snapshot.revision, u64::MAX);
    assert!(matches!(
        service.execute(ProjectCommand::Rename {
            name: "Again".to_owned()
        }),
        Err(ProjectError::CommandInvalid(_))
    ));
    assert_eq!(service.current()?, snapshot);

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

#[test]
fn plan_settings_are_per_record_and_transactional() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"solid case\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid case\n")?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Demo")?;
    let first = service.import_stl(&source, "midplane", "millimeters", false)?;
    let second = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let latest = service
        .plan_settings("welcome")
        .ok_or("missing latest plan")?;
    assert_eq!(latest.import_id, second.id);
    assert_eq!(latest.mesh_type, "solid-3d");
    let settings = service
        .plan_settings(&first.id)
        .ok_or("missing first plan")?;
    assert_eq!(settings.mesh_type, "midplane");
    assert_eq!(settings.sequence_id, "fill");
    let before = fs::read(&project.path)?;
    for (path, revision, id, sequence) in [
        (
            project.path.clone(),
            settings.revision,
            first.id.as_str(),
            "unknown",
        ),
        (
            project.path.clone(),
            settings.revision,
            "missing",
            "fill-pack",
        ),
        (
            project.path.clone(),
            settings.revision + 1,
            first.id.as_str(),
            "fill-pack",
        ),
        (
            fixture.root.join("other.panta"),
            settings.revision,
            first.id.as_str(),
            "fill-pack",
        ),
    ] {
        assert!(
            service
                .set_analysis_sequence(&path, revision, id, sequence)
                .is_err()
        );
        assert_eq!(fs::read(&project.path)?, before);
    }
    service.set_analysis_sequence(&project.path, settings.revision, &first.id, "fill-pack")?;
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(
        reopened
            .plan_settings(&first.id)
            .ok_or("missing first")?
            .sequence_id,
        "fill-pack"
    );
    assert_eq!(
        reopened
            .plan_settings(&second.id)
            .ok_or("missing second")?
            .sequence_id,
        "fill"
    );
    let selected = service.plan_settings(&first.id).ok_or("missing settings")?;
    let unchanged =
        service.set_analysis_sequence(&project.path, selected.revision, &first.id, "fill-pack")?;
    assert_eq!(unchanged.revision, selected.revision);
    // 用普通文件阻塞 staging 目录，模拟清单提交失败。
    fs::remove_dir(project.path.with_extension("panta.write"))?;
    fs::write(
        project.path.with_extension("panta.write"),
        b"blocked staging directory",
    )?;
    assert!(
        service
            .set_analysis_sequence(&project.path, selected.revision, &first.id, "cool")
            .is_err()
    );
    assert_eq!(service.plan_settings(&first.id), Some(selected));
    Ok(())
}

fn wait_metadata_confirmation(
    service: &mut ProjectService,
    finish: fn(&mut ProjectService) -> Result<bool, ProjectError>,
) -> Result<(), ProjectError> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if finish(service)? {
            return Ok(());
        }
        assert!(
            std::time::Instant::now() < deadline,
            "metadata confirmation timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn material_confirmation_blocks_competing_writes_and_recovers_from_failure()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("material.stl");
    fs::write(&source, b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Material")?;
    let first = service.import_stl(&source, "dual-domain", "millimeters", false)?;
    let second = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let initial = service.plan_settings(&first.id).ok_or("missing plan")?;
    let material = &panta_core::project::default_material()?.id;
    let original = fs::read(&project.path)?;
    assert!(initial.material_id.is_empty());
    assert!(service.finish_material_confirmation().is_err());
    for (path, revision, id, value) in [
        (
            &fixture.root,
            initial.revision,
            first.id.as_str(),
            material.as_str(),
        ),
        (
            &project.path,
            initial.revision - 1,
            first.id.as_str(),
            material.as_str(),
        ),
        (
            &project.path,
            initial.revision,
            "missing",
            material.as_str(),
        ),
        (
            &project.path,
            initial.revision,
            first.id.as_str(),
            "unknown",
        ),
    ] {
        assert!(
            service
                .begin_material_confirmation(path, revision, id, value)
                .is_err()
        );
        assert_eq!(service.plan_settings(&first.id), Some(initial.clone()));
    }

    let obstacle = project.path.with_extension("panta.write");
    fs::remove_dir(&obstacle)?;
    fs::write(&obstacle, b"blocked staging directory")?;
    assert!(service.begin_material_confirmation(
        &project.path,
        initial.revision,
        &first.id,
        material
    )?);
    // 结果未消费前锁仍有效；不依赖后台线程的耗时来命中提交窗口。
    assert_eq!(service.plan_settings(&first.id), Some(initial.clone()));
    assert!(service.create(&fixture.root, "Competing").is_err());
    assert!(service.open(&project.path).is_err());
    assert!(service.save().is_err());
    assert!(
        service
            .execute(ProjectCommand::Rename {
                name: "Competing".into()
            })
            .is_err()
    );
    assert!(
        service
            .import_stl(&source, "solid-3d", "millimeters", false)
            .is_err()
    );
    assert!(
        service
            .set_analysis_sequence(&project.path, initial.revision, &first.id, "cool")
            .is_err()
    );
    assert!(
        service
            .begin_material_confirmation(&project.path, initial.revision, &first.id, material)
            .is_err()
    );
    assert!(
        service
            .begin_fill_settings_confirmation(
                &project.path,
                initial.revision,
                &first.id,
                initial.fill_settings.clone()
            )
            .is_err()
    );
    assert!(service.finish_fill_settings_confirmation().is_err());
    assert!(
        wait_metadata_confirmation(&mut service, ProjectService::finish_material_confirmation)
            .is_err()
    );
    assert_eq!(service.plan_settings(&first.id), Some(initial.clone()));
    assert_eq!(fs::read(&project.path)?, original);
    fs::remove_file(&obstacle)?;
    service.save()?;

    assert!(service.begin_material_confirmation(
        &project.path,
        initial.revision,
        &first.id,
        material
    )?);
    wait_metadata_confirmation(&mut service, ProjectService::finish_material_confirmation)?;
    let saved = service
        .plan_settings(&first.id)
        .ok_or("missing saved plan")?;
    assert_eq!(saved.material_id, *material);
    assert_eq!(saved.revision, initial.revision + 1);
    assert!(
        service
            .plan_settings(&second.id)
            .ok_or("missing second plan")?
            .material_id
            .is_empty()
    );
    let manifest = fs::read(&project.path)?;
    assert!(!service.begin_material_confirmation(
        &project.path,
        saved.revision,
        &first.id,
        material
    )?);
    assert_eq!(service.plan_settings(&first.id), Some(saved.clone()));
    assert_eq!(fs::read(&project.path)?, manifest);
    assert!(service.finish_material_confirmation().is_err());
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.plan_settings(&first.id), Some(saved));
    assert!(
        reopened
            .plan_settings(&second.id)
            .ok_or("missing second plan")?
            .material_id
            .is_empty()
    );
    Ok(())
}

#[test]
fn fill_settings_round_trip_is_per_record_and_blocks_competing_writes()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"solid case\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid case\n")?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Demo")?;
    let first = service.import_stl(&source, "midplane", "millimeters", false)?;
    let second = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let initial = service
        .plan_settings(&first.id)
        .ok_or("missing first plan")?;
    assert!(!initial.fill_settings_confirmed);
    let mut settings = initial.fill_settings.clone();
    settings.melt_temperature_celsius = 235.0;
    settings
        .holding_profile
        .push(panta_core::project::HoldingProfilePoint {
            duration_seconds: 5.0,
            pressure_percent: 70.0,
        });
    assert!(service.begin_fill_settings_confirmation(
        &project.path,
        initial.revision,
        &first.id,
        settings.clone()
    )?);
    assert_eq!(service.plan_settings(&first.id), Some(initial.clone()));
    assert!(service.save().is_err());
    assert!(
        service
            .set_analysis_sequence(&project.path, initial.revision, &first.id, "cool")
            .is_err()
    );
    assert!(
        service
            .begin_material_confirmation(
                &project.path,
                initial.revision,
                &first.id,
                &panta_core::project::default_material()?.id
            )
            .is_err()
    );
    assert!(service.finish_material_confirmation().is_err());
    wait_metadata_confirmation(
        &mut service,
        ProjectService::finish_fill_settings_confirmation,
    )?;
    let saved = service
        .plan_settings(&first.id)
        .ok_or("missing saved plan")?;
    assert!(saved.fill_settings_confirmed);
    assert_eq!(saved.revision, initial.revision + 1);
    assert_eq!(saved.fill_settings, settings);
    assert!(
        !service
            .plan_settings(&second.id)
            .ok_or("missing second plan")?
            .fill_settings_confirmed
    );
    assert!(!service.begin_fill_settings_confirmation(
        &project.path,
        saved.revision,
        &first.id,
        settings.clone()
    )?);
    assert!(service.finish_fill_settings_confirmation().is_err());
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.plan_settings(&first.id), Some(saved.clone()));
    for (path, revision, id) in [
        (&project.path, saved.revision - 1, first.id.as_str()),
        (&fixture.root, saved.revision, first.id.as_str()),
        (&project.path, saved.revision, "missing"),
    ] {
        assert!(
            service
                .begin_fill_settings_confirmation(path, revision, id, settings.clone())
                .is_err()
        );
    }
    service.set_analysis_sequence(&project.path, saved.revision, &first.id, "cool")?;
    let other_sequence = service
        .plan_settings(&first.id)
        .ok_or("missing switched plan")?;
    assert_eq!(other_sequence.fill_settings, settings);
    assert!(
        service
            .begin_fill_settings_confirmation(
                &project.path,
                other_sequence.revision,
                &first.id,
                settings
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn fill_settings_invalid_candidates_and_io_failure_preserve_manifest_and_release_lock()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(&source, b"solid case\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid case\n")?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Demo")?;
    let record = service.import_stl(&source, "midplane", "millimeters", false)?;
    let initial = service.plan_settings(&record.id).ok_or("missing plan")?;
    let before = fs::read(&project.path)?;
    let mut invalid = Vec::new();
    let mut candidate = initial.fill_settings.clone();
    candidate.mold_temperature_celsius = -274.0;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.melt_temperature_celsius = f64::NAN;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.flow_rate_cm3_per_second = 0.0;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.switch_over_volume_percent = 101.0;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.holding_profile.clear();
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.holding_profile[0].pressure_percent = 201.0;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate.holding_profile[0].duration_seconds = -1.0;
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate
        .holding_profile
        .resize(4097, candidate.holding_profile[0].clone());
    invalid.push(candidate);
    let mut candidate = initial.fill_settings.clone();
    candidate
        .holding_profile
        .iter_mut()
        .for_each(|point| point.duration_seconds = f64::MAX);
    invalid.push(candidate);
    for (index, candidate) in invalid.into_iter().enumerate() {
        assert!(service.begin_fill_settings_confirmation(
            &project.path,
            initial.revision,
            &record.id,
            candidate
        )?);
        assert!(
            matches!(
                wait_metadata_confirmation(
                    &mut service,
                    ProjectService::finish_fill_settings_confirmation
                ),
                Err(ProjectError::ManifestInvalid(_))
            ),
            "invalid candidate {index} was not rejected by domain validation"
        );
        assert_eq!(service.plan_settings(&record.id), Some(initial.clone()));
        assert_eq!(fs::read(&project.path)?, before);
    }
    fs::remove_dir(project.path.with_extension("panta.write"))?;
    fs::write(
        project.path.with_extension("panta.write"),
        b"blocked staging directory",
    )?;
    assert!(service.begin_fill_settings_confirmation(
        &project.path,
        initial.revision,
        &record.id,
        initial.fill_settings.clone()
    )?);
    assert!(
        wait_metadata_confirmation(
            &mut service,
            ProjectService::finish_fill_settings_confirmation
        )
        .is_err()
    );
    assert_eq!(service.plan_settings(&record.id), Some(initial.clone()));
    assert_eq!(fs::read(&project.path)?, before);
    fs::remove_file(project.path.with_extension("panta.write"))?;
    assert!(service.begin_material_confirmation(
        &project.path,
        initial.revision,
        &record.id,
        &panta_core::project::default_material()?.id
    )?);
    assert!(
        service
            .begin_fill_settings_confirmation(
                &project.path,
                initial.revision,
                &record.id,
                initial.fill_settings
            )
            .is_err()
    );
    wait_metadata_confirmation(&mut service, ProjectService::finish_material_confirmation)?;
    service.save()?;
    Ok(())
}

#[test]
fn gate_location_confirmation_round_trips_and_rejects_invalid_candidates()
-> Result<(), Box<dyn std::error::Error>> {
    use panta_core::project::GateLocatorAlgorithm;
    let fixture = Fixture::new()?;
    let source = fixture.root.join("gate.stl");
    fs::write(
        &source,
        b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendsolid\n",
    )?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Gate")?;
    let imported = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let initial = service.plan_settings(&imported.id).ok_or("missing plan")?;
    assert!(!initial.gate_location_settings_confirmed);
    assert_eq!(initial.gate_location_settings.number_of_gates, 1);
    assert_eq!(
        initial.gate_location_settings.machine_source_text(),
        "Default machine"
    );
    assert_eq!(
        GateLocatorAlgorithm::from_id("advanced-gate-locator")?,
        GateLocatorAlgorithm::AdvancedGateLocator
    );
    assert!(GateLocatorAlgorithm::from_id("unknown").is_err());
    assert!(
        service
            .finish_gate_location_settings_confirmation()
            .is_err()
    );
    assert!(
        service
            .begin_gate_location_settings_confirmation(
                &project.path,
                initial.revision,
                &imported.id,
                initial.gate_location_settings.clone()
            )
            .is_err()
    );
    service.set_analysis_sequence(
        &project.path,
        initial.revision,
        &imported.id,
        "gate-location",
    )?;
    let before = service.plan_settings(&imported.id).ok_or("missing plan")?;
    let mut settings = before.gate_location_settings.clone();
    settings.number_of_gates = 3;
    settings.melt_temperature_celsius = 240.0;
    assert!(service.begin_gate_location_settings_confirmation(
        &project.path,
        before.revision,
        &imported.id,
        settings.clone()
    )?);
    assert_eq!(service.plan_settings(&imported.id), Some(before.clone()));
    assert!(service.save().is_err());
    assert!(service.finish_fill_settings_confirmation().is_err());
    wait_metadata_confirmation(
        &mut service,
        ProjectService::finish_gate_location_settings_confirmation,
    )?;
    let saved = service.plan_settings(&imported.id).ok_or("missing plan")?;
    assert!(saved.gate_location_settings_confirmed);
    assert_eq!(saved.gate_location_settings, settings);
    assert_eq!(saved.revision, before.revision + 1);
    assert!(!service.begin_gate_location_settings_confirmation(
        &project.path,
        saved.revision,
        &imported.id,
        settings.clone()
    )?);
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.plan_settings(&imported.id), Some(saved.clone()));
    for (path, revision, id) in [
        (&fixture.root, saved.revision, imported.id.as_str()),
        (&project.path, saved.revision - 1, imported.id.as_str()),
        (&project.path, saved.revision, "missing"),
    ] {
        assert!(
            service
                .begin_gate_location_settings_confirmation(path, revision, id, settings.clone())
                .is_err()
        );
    }
    let mut invalid = Vec::new();
    for number_of_gates in [0, 11] {
        let mut value = settings.clone();
        value.number_of_gates = number_of_gates;
        invalid.push(value);
    }
    let mut value = settings.clone();
    value.machine_id = "unknown".into();
    invalid.push(value);
    for temperature in [f64::NAN, f64::INFINITY, -274.0] {
        let mut value = settings.clone();
        value.mold_temperature_celsius = temperature;
        invalid.push(value);
        let mut value = settings.clone();
        value.melt_temperature_celsius = temperature;
        invalid.push(value);
    }
    let manifest = fs::read(&project.path)?;
    for value in invalid {
        assert!(service.begin_gate_location_settings_confirmation(
            &project.path,
            saved.revision,
            &imported.id,
            value
        )?);
        assert!(
            wait_metadata_confirmation(
                &mut service,
                ProjectService::finish_gate_location_settings_confirmation
            )
            .is_err()
        );
        assert_eq!(service.plan_settings(&imported.id), Some(saved.clone()));
        assert_eq!(fs::read(&project.path)?, manifest);
    }
    fs::remove_dir(project.path.with_extension("panta.write"))?;
    fs::write(
        project.path.with_extension("panta.write"),
        b"blocked staging directory",
    )?;
    settings.number_of_gates = 4;
    assert!(service.begin_gate_location_settings_confirmation(
        &project.path,
        saved.revision,
        &imported.id,
        settings
    )?);
    assert!(
        wait_metadata_confirmation(
            &mut service,
            ProjectService::finish_gate_location_settings_confirmation
        )
        .is_err()
    );
    assert_eq!(service.plan_settings(&imported.id), Some(saved));
    fs::remove_file(project.path.with_extension("panta.write"))?;
    assert!(service.save().is_ok());
    Ok(())
}

#[test]
fn gate_location_manifest_rejects_missing_import_and_invalid_settings()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Gate")?;
    let source = fixture.root.join("part.stl");
    fs::write(
        &source,
        b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendsolid\n",
    )?;
    let imported = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    let before = service.current()?;
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&project.path)?)?;
    let mut manifest = original.clone();
    manifest["gate_location_settings"] =
        serde_json::json!({"missing": panta_core::project::GateLocationSettings::default()});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(service.open(&project.path).is_err());
    assert_eq!(service.current()?, before);
    let mut manifest = original;
    let invalid = panta_core::project::GateLocationSettings {
        number_of_gates: 0,
        ..Default::default()
    };
    manifest["gate_location_settings"] = serde_json::json!({imported.id: invalid});
    fs::write(&project.path, serde_json::to_vec(&manifest)?)?;
    assert!(service.open(&project.path).is_err());
    assert_eq!(service.current()?, before);
    Ok(())
}

fn finish_preview(
    service: &mut ProjectService,
    request: u64,
) -> Result<panta_core::project::StlImportPreview, ProjectError> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(preview) = service.finish_stl_preview(request)? {
            return Ok(preview);
        }
        assert!(
            std::time::Instant::now() < deadline,
            "STL preview timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn asynchronous_preview_retains_source_snapshot_and_preserves_committed_state()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let path = fixture.root.join("零件.stl");
    fs::write(&path, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Demo")?;
    let manifest = fs::read(&project.path)?;
    let request = service.begin_stl_preview(&path)?;
    // 即使 worker 已就绪，必须先消费预检结果，不能绕过已显示的来源快照。
    assert_eq!(
        service
            .import_stl(&path, "solid-3d", "millimeters", false)
            .err()
            .map(|error| error.code()),
        Some("project.command_invalid")
    );
    let preview = finish_preview(&mut service, request)?;
    assert_eq!(preview.source_name, "零件.stl");
    assert_eq!(preview.dimensions, [1.0, 1.0, 0.0]);
    assert_eq!(fs::read(&project.path)?, manifest);
    assert!(service.imports()?.is_empty());
    fs::write(&path, "vertex 0 0 0\nvertex 2 0 0\nvertex 0 1 0\n")?;
    assert_eq!(
        service
            .import_stl(&path, "solid-3d", "millimeters", false)
            .err()
            .map(|error| error.code()),
        Some("project.import_source_changed")
    );
    assert_eq!(fs::read(&project.path)?, manifest);
    let request = service.begin_stl_preview(&path)?;
    assert_eq!(
        finish_preview(&mut service, request)?.dimensions,
        [2.0, 1.0, 0.0]
    );
    service.import_stl(&path, "solid-3d", "millimeters", false)?;
    assert_eq!(service.imports()?.len(), 1);
    Ok(())
}

#[test]
fn asynchronous_preview_rejects_replaced_cancelled_and_old_project_requests()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let path = fixture.root.join("sample.stl");
    fs::write(&path, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = ProjectService::new();
    let first = service.begin_stl_preview(&path)?;
    assert!(service.cancel_stl_preview(first));
    assert!(service.finish_stl_preview(first).is_err());
    let second = service.begin_stl_preview(&path)?;
    let third = service.begin_stl_preview(&path)?;
    assert!(third > second);
    assert!(!service.cancel_stl_preview(second));
    assert!(service.finish_stl_preview(second).is_err());
    assert_eq!(finish_preview(&mut service, third)?.triangle_count, 1);
    let previous = service.begin_stl_preview(&path)?;
    service.create(&fixture.root, "Demo")?;
    assert!(service.finish_stl_preview(previous).is_err());
    let failed = service.begin_stl_preview(&fixture.root.join("missing.stl"))?;
    let error = finish_preview(&mut service, failed)
        .err()
        .ok_or("missing STL unexpectedly succeeded")?;
    assert_eq!(error.code(), "project.import_file_missing");
    assert_eq!(error.category(), "missing");
    assert!(service.imports()?.is_empty());
    let next = service.begin_stl_preview(&path)?;
    assert_eq!(finish_preview(&mut service, next)?.triangle_count, 1);
    assert!(service.cancel_stl_preview(next));
    Ok(())
}

#[test]
fn exhausted_revision_rejects_all_mutations_without_writing_assets_or_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    use panta_core::project::{FillSettings, GateLocationSettings, default_material};
    fn exhausted<T>(result: Result<T, ProjectError>) {
        match result {
            Err(error) => {
                assert_eq!(error.code(), "project.command_invalid");
                assert_eq!(error.detail(), "project revision exhausted");
            }
            Ok(_) => panic!("exhausted revision accepted a mutation"),
        }
    }
    let fixture = Fixture::new()?;
    let source = fixture.root.join("sample.stl");
    fs::write(&source, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    for sequence in ["fill", "gate-location"] {
        let mut service = ProjectService::new();
        let created = service.create(&fixture.root, sequence)?;
        let record = service.import_stl(&source, "solid-3d", "millimeters", false)?;
        let revision = service.current()?.revision;
        service.set_analysis_sequence(&created.path, revision, &record.id, sequence)?;
        let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&created.path)?)?;
        manifest["revision"] = serde_json::Value::from(u64::MAX);
        fs::write(&created.path, serde_json::to_vec(&manifest)?)?;
        service.open(&created.path)?;
        let before = service.current()?;
        let plan = service.plan_settings(&record.id);
        let bytes = fs::read(&created.path)?;
        let assets = created
            .path
            .parent()
            .ok_or("missing package root")?
            .join("assets/imports");
        exhausted(service.execute(ProjectCommand::Rename {
            name: "Changed".into(),
        }));
        exhausted(service.import_stl(&source, "solid-3d", "millimeters", false));
        exhausted(service.set_analysis_sequence(
            &created.path,
            u64::MAX,
            &record.id,
            if sequence == "fill" {
                "gate-location"
            } else {
                "fill"
            },
        ));
        exhausted(service.begin_material_confirmation(
            &created.path,
            u64::MAX,
            &record.id,
            &default_material()?.id,
        ));
        if sequence == "fill" {
            exhausted(service.begin_fill_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                FillSettings::default(),
            ));
        } else {
            exhausted(service.begin_gate_location_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                GateLocationSettings::default(),
            ));
        }
        // 没有变更的序列命令不需要新修订，仍能返回现有快照。
        assert_eq!(
            service.set_analysis_sequence(&created.path, u64::MAX, &record.id, sequence)?,
            before
        );
        assert_eq!(service.current()?, before);
        assert_eq!(service.plan_settings(&record.id), plan);
        assert_eq!(fs::read(&created.path)?, bytes);
        assert_eq!(service.imports()?.len(), 1);
        assert_eq!(
            fs::read_dir(assets)?
                .collect::<Result<Vec<_>, std::io::Error>>()?
                .len(),
            1
        );
        // 已确认值的无操作请求在耗尽状态仍成功；实际变更必须拒绝。
        manifest["materials"] = serde_json::json!({ record.id.clone(): default_material()?.id });
        if sequence == "fill" {
            manifest["fill_settings"] =
                serde_json::json!({ record.id.clone(): FillSettings::default() });
        } else {
            manifest["gate_location_settings"] =
                serde_json::json!({ record.id.clone(): GateLocationSettings::default() });
        }
        fs::write(&created.path, serde_json::to_vec(&manifest)?)?;
        service.open(&created.path)?;
        let confirmed = fs::read(&created.path)?;
        assert!(!service.begin_material_confirmation(
            &created.path,
            u64::MAX,
            &record.id,
            &default_material()?.id
        )?);
        if sequence == "fill" {
            assert!(!service.begin_fill_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                FillSettings::default()
            )?);
            let changed = FillSettings {
                flow_rate_cm3_per_second: 95.0,
                ..FillSettings::default()
            };
            exhausted(service.begin_fill_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                changed,
            ));
        } else {
            assert!(!service.begin_gate_location_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                GateLocationSettings::default()
            )?);
            let changed = GateLocationSettings {
                number_of_gates: 2,
                ..GateLocationSettings::default()
            };
            exhausted(service.begin_gate_location_settings_confirmation(
                &created.path,
                u64::MAX,
                &record.id,
                changed,
            ));
        }
        assert_eq!(service.current()?, before);
        assert_eq!(fs::read(&created.path)?, confirmed);
    }
    Ok(())
}

#[test]
fn stale_services_and_parallel_saves_cannot_overwrite_a_committed_revision()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut owner = ProjectService::new();
    let project = owner.create(&fixture.root, "Concurrent")?;
    let mut stale = ProjectService::new();
    stale.open(&project.path)?;
    owner.execute(ProjectCommand::Rename {
        name: "Owner".into(),
    })?;
    owner.save()?;
    let committed = fs::read(&project.path)?;
    stale.execute(ProjectCommand::Rename {
        name: "Stale".into(),
    })?;
    let before = stale.current()?;
    let result = stale.save();
    assert!(matches!(result, Err(ProjectError::CommandInvalid(_))));
    assert_eq!(stale.current()?, before);
    assert_eq!(fs::read(&project.path)?, committed);
    let source = fixture.root.join("part.stl");
    fs::write(&source, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    assert!(matches!(
        stale.import_stl(&source, "solid-3d", "millimeters", false),
        Err(ProjectError::CommandInvalid(_))
    ));
    assert!(
        !project
            .path
            .parent()
            .ok_or("no package root")?
            .join("assets")
            .exists()
    );

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for name in ["First", "Second"] {
        let mut service = ProjectService::new();
        service.open(&project.path)?;
        service.execute(ProjectCommand::Rename { name: name.into() })?;
        let barrier = std::sync::Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            service.save()
        }));
    }
    let mut successes = 0;
    for handle in handles {
        match handle.join().map_err(|_| "save worker panicked")? {
            Ok(_) => successes += 1,
            Err(ProjectError::CommandInvalid(_)) => {}
            Err(error) => return Err(error.into()),
        }
    }
    assert_eq!(successes, 1);
    owner.open(&project.path)?;
    assert_eq!(owner.current()?.revision, 2);
    assert!(matches!(owner.current()?.name.as_str(), "First" | "Second"));
    assert_eq!(
        fs::read_dir(project.path.with_extension("panta.write"))?
            .collect::<Result<Vec<_>, _>>()?
            .len(),
        0
    );
    Ok(())
}

#[test]
#[cfg_attr(miri, ignore = "requires a separate OS process")]
fn project_lease_child() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = std::env::var_os("PANTA_TEST_PROJECT_LEASE") else {
        return Ok(());
    };
    let file = fs::OpenOptions::new().read(true).write(true).open(path)?;
    fs4::FileExt::try_lock(&file)?;
    use std::io::Write;
    println!("LEASE_HELD");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    // 不运行 Rust 析构，验证操作系统在进程退出时释放锁。
    std::process::exit(0);
}

#[test]
#[cfg_attr(miri, ignore = "requires a separate OS process")]
fn project_write_lock_is_nonblocking_and_released_after_process_exit()
-> Result<(), Box<dyn std::error::Error>> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    let project = service.create(&fixture.root, "Lease")?;
    service.execute(ProjectCommand::Rename {
        name: "Saved".into(),
    })?;
    let before = service.current()?;
    let mut child = Command::new(std::env::current_exe()?)
        .args(["--exact", "project_lease_child", "--nocapture"])
        .env(
            "PANTA_TEST_PROJECT_LEASE",
            project.path.with_extension("panta.lock"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("missing child stdout")?;
    let mut ready = false;
    for line in BufReader::new(stdout).lines() {
        if line? == "LEASE_HELD" {
            ready = true;
            break;
        }
    }
    assert!(ready, "child did not acquire the lease");
    match service.save() {
        Err(error) => assert_eq!(error.detail(), "project write pending"),
        Ok(_) => panic!("save succeeded while another process held the lease"),
    }
    assert_eq!(service.current()?, before);
    drop(child.stdin.take());
    assert!(child.wait()?.success());
    service.save()?;
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    assert_eq!(reopened.current()?.name, "Saved");
    Ok(())
}

#[test]
#[cfg_attr(miri, ignore = "requires a separate OS process")]
fn background_write_shutdown_child() -> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("PANTA_TEST_WRITE_SHUTDOWN") else {
        return Ok(());
    };
    let root = std::path::PathBuf::from(root);
    let mut service = ProjectService::new();
    let project = service.create(&root, "Shutdown")?;
    let record =
        service.import_stl(&root.join("shutdown.stl"), "solid-3d", "millimeters", false)?;
    let revision = service.current()?.revision;
    service.begin_material_confirmation(
        &project.path,
        revision,
        &record.id,
        &panta_core::project::default_material()?.id,
    )?;
    drop(service);
    panta_core::finish_background_writes();
    let mut reopened = ProjectService::new();
    reopened.open(&project.path)?;
    let plan = reopened.plan_settings(&record.id).ok_or("missing plan")?;
    assert_eq!(plan.revision, revision + 1);
    assert_eq!(
        plan.material_id,
        panta_core::project::default_material()?.id
    );
    Ok(())
}

#[test]
#[cfg_attr(miri, ignore = "requires a separate OS process")]
fn application_shutdown_finishes_accepted_writes_after_service_teardown()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.root.join("shutdown.stl"),
        "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n",
    )?;
    let result = std::process::Command::new(std::env::current_exe()?)
        .args(["--exact", "background_write_shutdown_child", "--nocapture"])
        .env("PANTA_TEST_WRITE_SHUTDOWN", &fixture.root)
        .output()?;
    assert!(
        result.status.success(),
        "shutdown child failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
