use super::*;

#[test]
fn exhausted_revision_crosses_cxx_as_diagnostic_without_changing_the_project()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::temp_directory::Fixture::new()?;
    let mut service = crate::project_service_new();
    let created = project_service_create_response(
        &mut service,
        fixture.root.display().to_string(),
        "Max".into(),
    );
    assert!(created.error.code.is_empty());
    let path = std::path::PathBuf::from(&created.value.path);
    let mut manifest = String::from_utf8(std::fs::read(&path)?)?;
    let revision_key = manifest
        .find("\"revision\"")
        .ok_or("revision field missing")?;
    let value_start = revision_key
        + manifest[revision_key..]
            .find(':')
            .ok_or("revision field has no value")?
        + 1;
    let value_start = value_start
        + manifest[value_start..]
            .find(|character: char| !character.is_whitespace())
            .ok_or("revision value missing")?;
    let value_end = value_start
        + manifest[value_start..]
            .find(|character: char| !character.is_ascii_digit())
            .ok_or("revision value has no terminator")?;
    manifest.replace_range(value_start..value_end, &u64::MAX.to_string());
    std::fs::write(&path, &manifest)?;
    let opened = project_service_open_response(&mut service, path.display().to_string());
    assert!(opened.error.code.is_empty());
    let result = project_service_execute_response(
        &mut service,
        bridge::ProjectCommand {
            kind: bridge::ProjectCommandKind::Rename,
            value: "Next".into(),
        },
    );
    assert_eq!(result.error.code, "project.command_invalid");
    assert_eq!(result.error.category, "validation");
    assert_eq!(result.error.detail, "project revision exhausted");
    assert!(result.value.path.is_empty());
    let current = project_service_current_response(&service);
    assert!(current.error.code.is_empty());
    assert_eq!(current.value.name, "Max");
    assert_eq!(current.value.revision, u64::MAX);
    assert!(!current.value.dirty);
    assert_eq!(std::fs::read(&path)?, manifest.as_bytes());
    Ok(())
}

fn poll_preview(service: &mut ProjectService, request: u64) -> bridge::StlPreviewPoll {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let result = project_service_finish_stl_preview_response(service, request);
        if result.ready || !result.error.code.is_empty() {
            return result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "bridge preview timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn poll_import(service: &mut ProjectService, request: u64) -> bridge::StlImportPoll {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let result = project_service_finish_stl_import_response(service, request);
        if result.ready {
            return result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "bridge import timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn confirmed_import_bridge_returns_committed_record() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::temp_directory::Fixture::new()?;
    let source = fixture.root.join("bridge.stl");
    std::fs::write(&source, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = crate::project_service_new();
    let project = project_service_create_response(
        &mut service,
        fixture.root.display().to_string(),
        "Bridge".into(),
    );
    assert!(project.error.code.is_empty());
    let preview =
        project_service_begin_stl_preview_response(&mut service, source.display().to_string());
    assert!(preview.error.code.is_empty());
    assert!(poll_preview(&mut service, preview.value).ready);
    let request = project_service_begin_stl_import_response(
        &mut service,
        source.display().to_string(),
        "solid-3d".into(),
        "millimeters".into(),
        false,
    );
    assert!(request.error.code.is_empty());
    let imported = poll_import(&mut service, request.value);
    assert!(imported.error.code.is_empty());
    assert!(imported.ready);
    assert_eq!(imported.value.id, "import-1");
    assert_eq!(imported.value.source_name, "bridge.stl");
    assert_eq!(project_service_current_response(&service).value.revision, 1);
    Ok(())
}

#[test]
fn preview_bridge_distinguishes_pending_success_failure_and_cancelled_requests()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::temp_directory::Fixture::new()?;
    let source = fixture.root.join("preview.stl");
    std::fs::write(&source, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut service = crate::project_service_new();
    let request =
        project_service_begin_stl_preview_response(&mut service, source.display().to_string());
    assert_eq!(request.error, bridge::ProjectDiagnostic::default());
    assert!(request.value > 0);
    let result = poll_preview(&mut service, request.value);
    assert!(result.ready);
    assert_eq!(result.error, bridge::ProjectDiagnostic::default());
    assert_eq!(result.value.source_name, "preview.stl");
    assert_eq!(result.value.triangle_count, 1);
    assert!(project_service_cancel_stl_preview(
        &mut service,
        request.value
    ));
    let invalid = project_service_finish_stl_preview_response(&mut service, request.value);
    assert!(!invalid.ready);
    assert_eq!(invalid.error.code, "project.command_invalid");
    assert!(invalid.value.source_name.is_empty());
    let request = project_service_begin_stl_preview_response(
        &mut service,
        fixture.root.join("missing.stl").display().to_string(),
    );
    let failed = poll_preview(&mut service, request.value);
    assert!(!failed.ready);
    assert_eq!(failed.error.code, "project.import_file_missing");
    assert_eq!(failed.error.category, "missing");
    assert!(failed.error.detail.ends_with("missing.stl"));
    assert!(failed.value.source_name.is_empty());
    Ok(())
}

#[test]
fn domain_diagnostics_preserve_codes_categories_and_verbatim_context() {
    let context = "参数: 中文 / C:\\零件.stl";
    let errors = [
        (
            ProjectError::EmptyName,
            "project.empty_name",
            "validation",
            "",
        ),
        (
            ProjectError::InvalidName(context.into()),
            "project.invalid_name",
            "validation",
            context,
        ),
        (
            ProjectError::LocationEmpty,
            "project.location_empty",
            "validation",
            "",
        ),
        (
            ProjectError::LocationNotAbsolute(context.into()),
            "project.location_not_absolute",
            "validation",
            context,
        ),
        (
            ProjectError::LocationCreateFailed(context.into()),
            "project.location_create_failed",
            "io",
            context,
        ),
        (
            ProjectError::FileMissing(context.into()),
            "project.file_missing",
            "missing",
            context,
        ),
        (
            ProjectError::InvalidFile(context.into()),
            "project.invalid_file",
            "format",
            context,
        ),
        (
            ProjectError::AlreadyExists(context.into()),
            "project.already_exists",
            "conflict",
            context,
        ),
        (
            ProjectError::ManifestInvalid(context.into()),
            "project.manifest_invalid",
            "format",
            context,
        ),
        (
            ProjectError::UnsupportedSchema(999),
            "project.unsupported_schema",
            "format",
            "999",
        ),
        (ProjectError::NoProject, "project.no_project", "state", ""),
        (
            ProjectError::CommandInvalid(context.into()),
            "project.command_invalid",
            "validation",
            context,
        ),
        (
            ProjectError::ImportFileMissing(context.into()),
            "project.import_file_missing",
            "missing",
            context,
        ),
        (
            ProjectError::ImportInvalidFile(context.into()),
            "project.import_invalid_file",
            "format",
            context,
        ),
        (
            ProjectError::ImportUnsupportedMeshType(context.into()),
            "project.import_unsupported_mesh_type",
            "format",
            context,
        ),
        (
            ProjectError::ImportUnsupportedUnits(context.into()),
            "project.import_unsupported_units",
            "format",
            context,
        ),
        (
            ProjectError::ImportParseFailed(context.into()),
            "project.import_parse_failed",
            "format",
            context,
        ),
        (
            ProjectError::ImportSourceChanged(context.into()),
            "project.import_source_changed",
            "conflict",
            context,
        ),
        (
            ProjectError::ImportAssetWriteFailed(context.into()),
            "project.import_asset_write_failed",
            "io",
            context,
        ),
        (
            ProjectError::ImportRecordMissing(context.into()),
            "project.import_record_missing",
            "missing",
            context,
        ),
        (
            ProjectError::Io(context.into()),
            "project.io",
            "io",
            context,
        ),
    ];
    for (error, code, category, detail) in errors {
        let diagnostic = bridge::ProjectDiagnostic::from(error);
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.category, category);
        assert_eq!(diagnostic.detail, detail);
    }
}

#[test]
fn all_response_shapes_distinguish_failure_from_empty_success() {
    macro_rules! check {
        ($response:ty, $value:ty) => {
            let success = <$response>::from(Ok::<$value, ProjectError>(<$value>::default()));
            assert_eq!(success.error, bridge::ProjectDiagnostic::default());
            let failure = <$response>::from(Err::<$value, _>(ProjectError::NoProject));
            assert_eq!(failure.error.code, "project.no_project");
            assert_eq!(failure.error.category, "state");
            assert!(failure.error.detail.is_empty());
        };
    }
    check!(bridge::ProjectSnapshotResult, bridge::ProjectSnapshot);
    check!(bridge::ProjectImportResult, bridge::ProjectImport);
    check!(bridge::ProjectImportsResult, Vec<bridge::ProjectImport>);
    check!(bridge::StlPreviewRequestResult, u64);
    check!(bridge::MaterialDefinitionResult, bridge::MaterialDefinition);
    check!(bridge::ProjectConfirmationResult, bool);
    check!(
        bridge::SurfaceMeshSnapshotResult,
        bridge::SurfaceMeshSnapshot
    );
    check!(bridge::ActivationAttemptResult, bridge::ActivationAttempt);
}

#[test]
fn service_responses_reject_requests_without_publishing_placeholder_values() {
    let mut service = crate::project_service_new();
    let result = project_service_create_response(&mut service, "relative".into(), "Demo".into());
    assert_eq!(result.error.code, "project.location_not_absolute");
    assert_eq!(result.error.category, "validation");
    assert_eq!(result.error.detail, "relative");
    assert!(result.value.path.is_empty());
    assert_eq!(
        project_service_current_response(&service).error.code,
        "project.no_project"
    );
    assert_eq!(
        project_service_save_response(&mut service).error.category,
        "state"
    );
    assert_eq!(
        project_service_imports_response(&service).error.code,
        "project.no_project"
    );
    let command = bridge::ProjectCommand {
        kind: bridge::ProjectCommandKind { repr: 99 },
        value: String::new(),
    };
    let result = project_service_execute_response(&mut service, command);
    assert_eq!(result.error.code, "project.command_invalid");
    assert_eq!(result.error.detail, "unknown command");
    assert_eq!(
        project_service_begin_asset_activation_response(&mut service, "import-9")
            .error
            .code,
        "project.no_project"
    );
    assert_eq!(
        project_service_mesh_snapshot_for_import_response(&service, "import-9")
            .error
            .code,
        "project.no_project"
    );
    assert_eq!(
        project_service_finish_material_confirmation_response(&mut service)
            .error
            .code,
        "project.command_invalid"
    );
    let material = default_material_response();
    assert!(material.error.code.is_empty());
    assert!(!material.value.id.is_empty());
}

#[test]
fn confirmation_responses_fail_without_a_project() {
    let mut service = crate::project_service_new();
    let opened = project_service_open_response(&mut service, "relative".into());
    assert_eq!(opened.error.code, "project.file_missing");
    assert!(opened.value.path.is_empty());

    let imported = project_service_import_stl_response(
        &mut service,
        "part.stl".into(),
        "solid-3d".into(),
        "millimeters".into(),
        false,
    );
    assert_eq!(imported.error.code, "project.no_project");
    assert!(imported.value.id.is_empty());

    let sequence = project_service_set_analysis_sequence_response(
        &mut service,
        "/tmp/missing.panta",
        1,
        "import-1",
        "sequence-1",
    );
    assert!(!sequence.error.code.is_empty());
    assert!(sequence.value.path.is_empty());

    let material = project_service_begin_material_confirmation_response(
        &mut service,
        "/tmp/missing.panta",
        1,
        "import-1",
        "material-1",
    );
    assert_eq!(material.error.code, "project.no_project");
    assert!(!material.value);

    let fill = bridge::FillSettings {
        mold_temperature_celsius: 40.0,
        melt_temperature_celsius: 240.0,
        flow_rate_cm3_per_second: 20.0,
        switch_over_volume_percent: 98.0,
        fiber_orientation: false,
        crystallization: false,
        holding_profile: Vec::new(),
    };
    let fill_begin = project_service_begin_fill_settings_confirmation_response(
        &mut service,
        "/tmp/missing.panta",
        1,
        "import-1",
        fill,
    );
    assert_eq!(fill_begin.error.code, "project.no_project");
    assert_eq!(
        project_service_finish_fill_settings_confirmation_response(&mut service)
            .error
            .code,
        "project.command_invalid"
    );

    let gate = bridge::GateLocationSettings {
        machine_id: "default".into(),
        machine_source_text: String::new(),
        mold_temperature_celsius: 40.0,
        melt_temperature_celsius: 240.0,
        algorithm_id: "missing-algorithm".into(),
        algorithm_source_text: String::new(),
        number_of_gates: 1,
    };
    let gate_begin = project_service_begin_gate_location_settings_confirmation_response(
        &mut service,
        "/tmp/missing.panta",
        1,
        "import-1",
        gate,
    );
    assert_eq!(gate_begin.error.code, "project.command_invalid");
    assert_eq!(
        project_service_finish_gate_location_settings_confirmation_response(&mut service)
            .error
            .code,
        "project.command_invalid"
    );

    let empty = bridge::ProjectDiagnostic::default();
    assert_eq!(empty.to_string(), "");
    let detailed = diagnostic("project.io", "io", "disk full".into());
    assert_eq!(detailed.to_string(), "project.io: disk full");
}
