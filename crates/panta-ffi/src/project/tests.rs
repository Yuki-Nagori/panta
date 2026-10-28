use super::activation_outcome_to_bridge;
use crate::{
    bridge, project_service_create, project_service_current, project_service_execute,
    project_service_mesh_snapshot_for_import, project_service_new, project_service_open,
    project_service_plan_settings, project_service_resident_mesh_ids, project_service_save,
    validate_fill_settings_draft, validate_gate_location_settings_draft,
    validate_holding_profile_draft,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn variable_requests_transfer_owned_results_and_preserve_structured_errors()
-> Result<(), Box<dyn std::error::Error>> {
    use super::{
        project_service_begin_variable_document, project_service_finish_variable_document,
        project_service_variable_document_generation, variable_document_request_accepted,
        variable_document_wait,
    };
    let fixture = crate::temp_directory::Fixture::new()?;
    let mut service = project_service_new();
    project_service_create(
        &mut service,
        fixture.root.display().to_string(),
        "Variables".to_owned(),
    )?;
    let generation = project_service_variable_document_generation(&service);
    let source = "version: 1\nkind: variables\n\nvalues:\n  count: int = 12\n";
    let request = project_service_begin_variable_document(
        &mut service,
        generation,
        0,
        source.to_owned(),
        true,
    );
    assert!(variable_document_request_accepted(&request));
    // receiver 和结果拥有所有权；等待不借用应用服务。
    let completion = std::thread::spawn(move || variable_document_wait(request))
        .join()
        .map_err(|_| "variable wait thread panicked")?;
    let result = project_service_finish_variable_document(&mut service, completion);
    assert!(result.error.code.is_empty());
    assert_eq!(result.generation, generation);
    assert_eq!(result.revision, 1);
    assert_eq!(result.values[0].value, "12");
    let previous = fs::read(fixture.root.join("Variables/variables.pa"))?;
    let request = project_service_begin_variable_document(
        &mut service,
        generation,
        0,
        source.to_owned(),
        true,
    );
    assert!(!variable_document_request_accepted(&request));
    let stale =
        project_service_finish_variable_document(&mut service, variable_document_wait(request));
    assert_eq!(stale.error.code, "variables.stale_revision");
    let request = project_service_begin_variable_document(
        &mut service,
        generation,
        1,
        "version: 1\nkind: variables\nvalues:\n  count: int = unknown-name\n".to_owned(),
        true,
    );
    assert!(variable_document_request_accepted(&request));
    let invalid =
        project_service_finish_variable_document(&mut service, variable_document_wait(request));
    assert_eq!(invalid.diagnostics[0].code, "variables.unknown_reference");
    assert_eq!(invalid.diagnostics[0].line, 4);
    assert_eq!(
        fs::read(fixture.root.join("Variables/variables.pa"))?,
        previous
    );
    Ok(())
}

#[test]
fn plan_settings_exposes_core_constraints_and_draft_validation_errors() {
    let service = project_service_new();
    let settings = project_service_plan_settings(&service, "");
    assert_eq!(settings.process_constraints.temperature.minimum, -273.15);
    assert!(!settings.process_constraints.flow_rate.minimum_inclusive);
    assert_eq!(settings.process_constraints.holding_pressure.maximum, 200.0);
    assert!(settings.process_constraints.number_of_gates.integer);

    let mut fill = settings.fill_settings;
    fill.flow_rate_cm3_per_second = 0.0;
    let validation = validate_fill_settings_draft(fill);
    assert!(!validation.valid);
    assert_eq!(validation.errors, ["flowRate"]);

    let validation = validate_holding_profile_draft(vec![bridge::HoldingProfilePoint {
        duration_seconds: 0.0,
        pressure_percent: 201.0,
    }]);
    assert_eq!(validation.errors, ["holdingProfile.0.pressure"]);

    let validation =
        validate_gate_location_settings_draft(40.0, 230.0, 1.5, "default-machine".into());
    assert_eq!(validation.errors, ["numberOfGates"]);
}
fn wait_confirmation(
    service: &mut crate::ProjectService,
    finish: fn(&mut crate::ProjectService) -> Result<bool, panta_core::project::ProjectError>,
) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !finish(service).map_err(|error| error.to_string())? {
        if std::time::Instant::now() >= deadline {
            return Err("metadata confirmation timed out".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(())
}

#[test]
fn plan_bridge_preserves_catalogs_confirmation_and_reopened_values()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        project_service_begin_fill_settings_confirmation,
        project_service_begin_gate_location_settings_confirmation,
        project_service_begin_material_confirmation,
        project_service_finish_fill_settings_confirmation,
        project_service_finish_gate_location_settings_confirmation,
        project_service_finish_material_confirmation, project_service_plan_settings,
        project_service_set_analysis_sequence,
    };
    let root = std::env::temp_dir().join(format!("panta-ffi-plan-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    fs::write(
        &source,
        b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendsolid\n",
    )?;
    assert_eq!(crate::default_mesh_type(), "dual-domain");
    assert_eq!(crate::mesh_type_catalog().len(), 3);
    assert_eq!(crate::analysis_sequence_catalog().len(), 10);
    let material = crate::default_material()?;
    assert!(!material.properties.is_empty());
    let mut service = project_service_new();
    let empty = project_service_plan_settings(&service, "");
    assert!(empty.import_id.is_empty());
    assert_eq!(empty.fill_settings.holding_profile.len(), 2);
    assert_eq!(empty.gate_location_settings.number_of_gates, 1);
    assert!(project_service_finish_material_confirmation(&mut service).is_err());
    assert!(project_service_finish_fill_settings_confirmation(&mut service).is_err());
    assert!(project_service_finish_gate_location_settings_confirmation(&mut service).is_err());
    let project = project_service_create(&mut service, root.display().to_string(), "Demo".into())?;
    let record = crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".into(),
        "millimeters".into(),
        false,
    )?;
    let initial = project_service_plan_settings(&service, &record.id);
    assert_eq!(initial.mesh_type, "solid-3d");
    assert_eq!(initial.sequence_id, "fill");
    assert!(
        project_service_set_analysis_sequence(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            "unknown"
        )
        .is_err()
    );
    assert!(
        project_service_begin_material_confirmation(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            "unknown"
        )
        .is_err()
    );
    assert!(project_service_begin_material_confirmation(
        &mut service,
        &project.path,
        initial.revision,
        &record.id,
        &material.id
    )?);
    assert!(
        project_service_begin_fill_settings_confirmation(
            &mut service,
            &project.path,
            initial.revision,
            &record.id,
            initial.fill_settings
        )
        .is_err()
    );
    wait_confirmation(&mut service, project_service_finish_material_confirmation)?;
    let selected = project_service_plan_settings(&service, &record.id);
    assert_eq!(selected.material_id, material.id);
    assert_eq!(selected.material_source_text, material.source_text);
    assert!(!project_service_begin_material_confirmation(
        &mut service,
        &project.path,
        selected.revision,
        &record.id,
        &material.id
    )?);
    let mut fill = selected.fill_settings;
    fill.melt_temperature_celsius = 240.0;
    fill.holding_profile.push(bridge::HoldingProfilePoint {
        duration_seconds: 5.0,
        pressure_percent: 70.0,
    });
    assert!(project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        selected.revision,
        &record.id,
        fill
    )?);
    wait_confirmation(
        &mut service,
        project_service_finish_fill_settings_confirmation,
    )?;
    let saved = project_service_plan_settings(&service, &record.id);
    assert!(saved.fill_settings_confirmed);
    assert_eq!(saved.fill_settings.melt_temperature_celsius, 240.0);
    assert_eq!(
        saved.fill_settings.holding_profile[2].pressure_percent,
        70.0
    );
    let revision = saved.revision;
    assert!(!project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        revision,
        &record.id,
        saved.fill_settings
    )?);
    let mut invalid = project_service_plan_settings(&service, &record.id).fill_settings;
    invalid.flow_rate_cm3_per_second = -1.0;
    assert!(project_service_begin_fill_settings_confirmation(
        &mut service,
        &project.path,
        revision,
        &record.id,
        invalid
    )?);
    assert!(
        wait_confirmation(
            &mut service,
            project_service_finish_fill_settings_confirmation
        )
        .is_err()
    );
    assert_eq!(
        project_service_plan_settings(&service, &record.id).revision,
        revision
    );
    let changed = project_service_set_analysis_sequence(
        &mut service,
        &project.path,
        revision,
        &record.id,
        "gate-location",
    )?;
    let mut gate = project_service_plan_settings(&service, &record.id).gate_location_settings;
    gate.algorithm_id = "unknown".into();
    assert!(
        project_service_begin_gate_location_settings_confirmation(
            &mut service,
            &project.path,
            changed.revision,
            &record.id,
            gate
        )
        .is_err()
    );
    let mut gate = project_service_plan_settings(&service, &record.id).gate_location_settings;
    gate.number_of_gates = 3;
    assert!(project_service_begin_gate_location_settings_confirmation(
        &mut service,
        &project.path,
        changed.revision,
        &record.id,
        gate
    )?);
    wait_confirmation(
        &mut service,
        project_service_finish_gate_location_settings_confirmation,
    )?;
    let gate_saved = project_service_plan_settings(&service, &record.id);
    assert!(gate_saved.gate_location_settings_confirmed);
    assert_eq!(gate_saved.gate_location_settings.number_of_gates, 3);
    let mut reopened = project_service_new();
    project_service_open(&mut reopened, project.path)?;
    let snapshot = project_service_plan_settings(&reopened, &record.id);
    assert_eq!(snapshot.revision, gate_saved.revision);
    assert_eq!(snapshot.material_id, material.id);
    assert_eq!(snapshot.fill_settings.holding_profile.len(), 3);
    assert_eq!(snapshot.fill_settings.melt_temperature_celsius, 240.0);
    assert_eq!(snapshot.gate_location_settings.number_of_gates, 3);
    assert!(snapshot.fill_settings_confirmed && snapshot.gate_location_settings_confirmed);
    assert!(crate::project_service_activate_mesh_document(
        &mut service,
        &record.id
    ));
    crate::project_service_deactivate_mesh_document(&mut service);
    crate::project_service_release_mesh_document(&mut service, &record.id);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn project_service_bridge_maps_snapshots_and_errors() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-project-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;

    let mut service = project_service_new();
    let no_project = match project_service_current(&service) {
        Ok(snapshot) => panic!("empty service returned {}", snapshot.name),
        Err(error) => error,
    };
    assert_eq!(no_project.code(), "project.no_project");

    let create_error =
        match project_service_create(&mut service, "relative".to_owned(), "Demo".to_owned()) {
            Ok(snapshot) => panic!("relative project created at {}", snapshot.path),
            Err(error) => error,
        };
    assert_eq!(create_error.code(), "project.location_not_absolute");
    assert_eq!(create_error.detail(), "relative");
    let save_error = match project_service_save(&mut service) {
        Ok(snapshot) => panic!("empty service saved {}", snapshot.name),
        Err(error) => error,
    };
    assert_eq!(save_error.code(), "project.no_project");

    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    assert_eq!(created.name, "Demo");
    assert!(!created.dirty);
    let current = project_service_current(&service)?;
    assert_eq!(current.path, created.path);
    assert_eq!(current.name, "Demo");

    let changed = project_service_execute(
        &mut service,
        bridge::ProjectCommand {
            kind: bridge::ProjectCommandKind::Rename,
            value: "Renamed".to_owned(),
        },
    )?;
    assert_eq!(changed.name, "Renamed");
    assert!(changed.dirty);

    let saved = project_service_save(&mut service)?;
    assert_eq!(saved.revision, 1);
    assert!(!saved.dirty);
    let unchanged = match project_service_execute(
        &mut service,
        bridge::ProjectCommand {
            kind: bridge::ProjectCommandKind::Rename,
            value: "Renamed".to_owned(),
        },
    ) {
        Ok(snapshot) => panic!(
            "unchanged rename succeeded at revision {}",
            snapshot.revision
        ),
        Err(error) => error,
    };
    assert_eq!(unchanged.code(), "project.command_invalid");
    assert_eq!(unchanged.detail(), "name is unchanged");

    let mut reopened = project_service_new();
    let opened = project_service_open(&mut reopened, created.path.clone())?;
    assert_eq!(opened.name, "Renamed");
    assert_eq!(opened.revision, 1);
    let invalid_file = PathBuf::from(&created.path).with_extension("json");
    fs::write(&invalid_file, br#"{}"#)?;
    let invalid_file_error =
        match project_service_open(&mut reopened, invalid_file.display().to_string()) {
            Ok(snapshot) => panic!("non-panta file opened as {}", snapshot.name),
            Err(error) => error,
        };
    assert_eq!(
        invalid_file_error.detail(),
        invalid_file.display().to_string()
    );

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn project_activation_bridge_admits_cancels_and_drains_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-act-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    let original = b"solid part
vertex 0 0 0
vertex 1 0 0
vertex 0 2 0
endsolid
";
    fs::write(&source, original)?;

    let mut service = project_service_new();
    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    )?;

    // 未知记录在提交边界同步拒绝；未知 attempt 取消返回 false。
    let missing = match crate::project_service_begin_asset_activation(&mut service, "import-99") {
        Ok(attempt) => panic!("unknown record admitted as attempt {}", attempt.attempt),
        Err(error) => error,
    };
    assert_eq!(missing.code(), "project.import_record_missing");
    assert!(!crate::project_service_cancel_asset_activation(
        &mut service,
        u64::MAX
    ));

    let attempt = crate::project_service_begin_asset_activation(&mut service, "import-1")?;
    assert!(attempt.attempt > 0);
    assert!(crate::project_service_cancel_asset_activation(
        &mut service,
        attempt.attempt
    ));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let outcome = loop {
        let drained = crate::project_service_drain_asset_activations(&mut service);
        if let Some(outcome) = drained.into_iter().next() {
            break outcome;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "activation outcome did not arrive in time"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(outcome.attempt, attempt.attempt);
    assert_eq!(outcome.generation, attempt.generation);
    assert_eq!(outcome.import_id, "import-1");
    assert!(
        outcome.kind == bridge::ActivationOutcomeKind::Succeeded
            || outcome.kind == bridge::ActivationOutcomeKind::Cancelled,
        "小文件激活终态只可能是成功（先完成）或取消（先命中检查点）"
    );
    if outcome.kind == bridge::ActivationOutcomeKind::Succeeded {
        // 单三角形：9 个 f64 坐标必须完整过桥。
        assert_eq!(outcome.coordinates.len(), 9);
    } else {
        assert!(outcome.coordinates.is_empty(), "取消不得携带坐标负载");
    }
    let _ = created;
    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn project_activation_success_commits_mesh_and_coordinates()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = crate::temp_directory::Fixture::new()?;
    let source = fixture.root.join("part.stl");
    fs::write(
        &source,
        b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n",
    )?;

    let mut service = project_service_new();
    project_service_create(
        &mut service,
        fixture.root.display().to_string(),
        "Demo".to_owned(),
    )?;
    let imported = crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    )?;
    crate::project_service_release_mesh_document(&mut service, &imported.id);
    assert!(crate::project_service_resident_mesh_ids(&service).is_empty());
    let attempt = crate::project_service_begin_asset_activation(&mut service, &imported.id)?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let outcome = loop {
        if let Some(outcome) = crate::project_service_drain_asset_activations(&mut service)
            .into_iter()
            .next()
        {
            break outcome;
        }
        if std::time::Instant::now() >= deadline {
            return Err("successful activation outcome did not arrive".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };

    assert!(outcome.kind == bridge::ActivationOutcomeKind::Succeeded);
    assert_eq!(outcome.attempt, attempt.attempt);
    assert_eq!(outcome.generation, attempt.generation);
    assert_eq!(outcome.import_id, imported.id);
    assert!(outcome.error.code.is_empty());
    assert_eq!(
        outcome.coordinates,
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0]
    );
    assert_eq!(
        crate::project_service_resident_mesh_ids(&service).as_slice(),
        std::slice::from_ref(&imported.id)
    );
    let snapshot = project_service_mesh_snapshot_for_import(&service, &imported.id)?;
    assert_eq!(snapshot.coordinates, outcome.coordinates);
    Ok(())
}

#[test]
fn project_stl_bridge_preserves_preview_commit_and_snapshot_state()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("panta-ffi-stl-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let source = root.join("part.stl");
    let original = b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
    fs::write(&source, original)?;

    let mut service = project_service_new();
    let no_project = match crate::project_service_imports(&service) {
        Ok(imports) => panic!("empty project returned {} imports", imports.len()),
        Err(error) => error,
    };
    assert_eq!(no_project.code(), "project.no_project");
    let preview = crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
    assert_eq!(preview.source_name, "part.stl");
    assert_eq!(preview.triangle_count, 1);
    assert_eq!(
        [preview.size_x, preview.size_y, preview.size_z],
        [1.0, 2.0, 0.0]
    );
    let import_without_project = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("import without a project succeeded as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(import_without_project.code(), "project.no_project");

    let created =
        project_service_create(&mut service, root.display().to_string(), "Demo".to_owned())?;
    assert!(project_service_resident_mesh_ids(&service).is_empty());
    assert!(crate::project_service_imports(&service)?.is_empty());

    let imported = crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "centimeters".to_owned(),
        true,
    )?;
    assert_eq!(imported.id, "import-1");
    assert_eq!(imported.source_name, "part.stl");
    assert_eq!(
        imported.mesh_asset_path,
        "assets/meshes/mesh-import-1-r1.pmesh"
    );
    assert_eq!(imported.mesh_type, "solid-3d");
    assert_eq!(imported.units, "centimeters");
    assert!(imported.show_import_log);
    assert_eq!(imported.triangle_count, 1);
    assert_eq!(
        [imported.size_x, imported.size_y, imported.size_z],
        [1.0, 2.0, 0.0]
    );

    let imports = crate::project_service_imports(&service)?;
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].parser_version, imported.parser_version);
    let snapshot = project_service_mesh_snapshot_for_import(&service, &imported.id)?;
    assert_eq!(snapshot.revision, 1);
    assert_eq!(
        snapshot.coordinates,
        [0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 20.0, 0.0]
    );

    let mut reopened = project_service_new();
    project_service_open(&mut reopened, created.path)?;
    // 打开工程仅读取清单；已保存网格须通过只读激活按需恢复（073/080）。
    assert!(project_service_resident_mesh_ids(&reopened).is_empty());
    let unloaded = match project_service_mesh_snapshot_for_import(&reopened, &imported.id) {
        Ok(_) => panic!("unloaded mesh returned a display snapshot"),
        Err(error) => error,
    };
    assert_eq!(unloaded.code, "project.mesh_not_resident");
    assert_eq!(unloaded.detail, imported.id);

    let step_source = root.join("part.step");
    fs::write(&step_source, b"not a supported STL source")?;
    let unsupported =
        match crate::project_service_inspect_stl(&mut service, step_source.display().to_string()) {
            Ok(preview) => panic!(
                "STEP file unexpectedly parsed as {} triangles",
                preview.triangle_count
            ),
            Err(error) => error,
        };
    assert_eq!(unsupported.code(), "project.import_invalid_file");

    crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
    fs::write(&source, b"vertex 0 0 0\nvertex 3 0 0\nvertex 0 4 0\n")?;
    let changed = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("changed source was imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(changed.code(), "project.import_source_changed");
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    fs::write(&source, original)?;
    let invalid_options = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "unknown".to_owned(),
        "millimeters".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("unsupported mesh type was imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(
        invalid_options.code(),
        "project.import_unsupported_mesh_type"
    );
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    let invalid_units = match crate::project_service_import_stl(
        &mut service,
        source.display().to_string(),
        "solid-3d".to_owned(),
        "yards".to_owned(),
        false,
    ) {
        Ok(imported) => panic!("unsupported units were imported as {}", imported.id),
        Err(error) => error,
    };
    assert_eq!(invalid_units.code(), "project.import_unsupported_units");
    assert_eq!(crate::project_service_imports(&service)?.len(), 1);

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn failed_activation_transfers_structured_diagnostics_without_mesh_payload() {
    let outcome = panta_core::project::CompletedActivation {
        attempt: 7,
        generation: 3,
        import_id: "import-2".into(),
        kind: panta_core::project::OutcomeKind::Failed,
        code: "project.asset_read_failed".into(),
        category: "io",
        detail: "C:\\资产: read denied".into(),
        mesh: None,
    };
    let result = activation_outcome_to_bridge(outcome);
    assert!(result.kind == bridge::ActivationOutcomeKind::Failed);
    assert_eq!(result.attempt, 7);
    assert_eq!(result.generation, 3);
    assert_eq!(result.import_id, "import-2");
    assert_eq!(result.error.code, "project.asset_read_failed");
    assert_eq!(result.error.category, "io");
    assert_eq!(result.error.detail, "C:\\资产: read denied");
    assert!(result.coordinates.is_empty());
}
