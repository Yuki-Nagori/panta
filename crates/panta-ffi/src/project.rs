//! 工程服务、方案配置与显示快照的 CXX 适配；业务校验和提交由 core 拥有。
use crate::{bridge, mesh_coordinates, project_response};

/// Rust 工程服务的 CXX 适配器：只负责 DTO 映射与结构化诊断，
/// 工程校验、清单事务及模型状态均保留在 panta-core。
pub struct ProjectService {
    pub(super) service: panta_core::project::ProjectService,
}

pub(super) fn project_service_new() -> Box<ProjectService> {
    Box::new(ProjectService {
        service: panta_core::project::ProjectService::new(),
    })
}

pub(super) fn project_service_create(
    service: &mut ProjectService,
    location: String,
    name: String,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service
        .service
        .create(std::path::Path::new(&location), &name)
        .map(project_snapshot)
}

pub(super) fn project_service_open(
    service: &mut ProjectService,
    path: String,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service
        .service
        .open(std::path::Path::new(&path))
        .map(project_snapshot)
}

pub(super) fn project_service_save(
    service: &mut ProjectService,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service.service.save().map(project_snapshot)
}

pub(super) fn project_service_execute(
    service: &mut ProjectService,
    command: bridge::ProjectCommand,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    let command = match command.kind {
        bridge::ProjectCommandKind::Rename => panta_core::project::ProjectCommand::Rename {
            name: command.value,
        },
        _ => {
            return Err(panta_core::project::ProjectError::CommandInvalid(
                "unknown command".to_owned(),
            ));
        }
    };
    service.service.execute(command).map(project_snapshot)
}

pub(super) fn project_service_current(
    service: &ProjectService,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service.service.current().map(project_snapshot)
}

pub(super) fn project_service_import_stl(
    service: &mut ProjectService,
    source: String,
    mesh_type: String,
    units: String,
    show_import_log: bool,
) -> Result<bridge::ProjectImport, panta_core::project::ProjectError> {
    service
        .service
        .import_stl(
            std::path::Path::new(&source),
            &mesh_type,
            &units,
            show_import_log,
        )
        .map(project_import)
}

pub(super) fn project_service_imports(
    service: &ProjectService,
) -> Result<Vec<bridge::ProjectImport>, panta_core::project::ProjectError> {
    service
        .service
        .imports()
        .map(|imports| imports.into_iter().map(project_import).collect())
}

pub(super) fn mesh_type_catalog() -> Vec<bridge::ChoiceDefinition> {
    panta_core::project::MESH_TYPES
        .iter()
        .map(|entry| bridge::ChoiceDefinition {
            id: entry.id.to_owned(),
            source_text: entry.source_text.to_owned(),
        })
        .collect()
}

pub(super) fn default_mesh_type() -> String {
    panta_core::project::DEFAULT_MESH_TYPE.to_owned()
}

pub(super) fn analysis_sequence_catalog() -> Vec<bridge::ChoiceDefinition> {
    panta_core::project::ANALYSIS_SEQUENCES
        .iter()
        .map(|entry| bridge::ChoiceDefinition {
            id: entry.id.to_owned(),
            source_text: entry.source_text.to_owned(),
        })
        .collect()
}

pub(super) fn default_material()
-> Result<bridge::MaterialDefinition, panta_core::project::ProjectError> {
    let material = panta_core::project::default_material()?;
    Ok(bridge::MaterialDefinition {
        family_source_text: material.family_source_text.clone(),
        id: material.id.clone(),
        source_text: material.source_text.clone(),
        properties: material
            .properties
            .iter()
            .map(|property| bridge::MaterialProperty {
                source_text: property.source_text.to_owned(),
                value: property.value.clone(),
            })
            .collect(),
    })
}

pub(super) fn project_service_plan_settings(
    service: &ProjectService,
    preferred_import_id: &str,
) -> bridge::PlanSettings {
    match service.service.plan_settings(preferred_import_id) {
        Some(settings) => bridge::PlanSettings {
            project_path: settings.project_path.display().to_string(),
            revision: settings.revision,
            import_id: settings.import_id,
            mesh_type: settings.mesh_type,
            sequence_id: settings.sequence_id,
            sequence_source_text: settings.sequence_source_text,
            material_id: settings.material_id,
            material_source_text: settings.material_source_text,
            fill_settings: fill_settings_dto(settings.fill_settings),
            fill_settings_confirmed: settings.fill_settings_confirmed,
            gate_location_settings: gate_location_settings_dto(settings.gate_location_settings),
            gate_location_settings_confirmed: settings.gate_location_settings_confirmed,
        },
        None => bridge::PlanSettings {
            project_path: String::new(),
            revision: 0,
            import_id: String::new(),
            mesh_type: String::new(),
            sequence_id: String::new(),
            sequence_source_text: String::new(),
            material_id: String::new(),
            material_source_text: String::new(),
            fill_settings: fill_settings_dto(panta_core::project::FillSettings::default()),
            fill_settings_confirmed: false,
            gate_location_settings: gate_location_settings_dto(
                panta_core::project::GateLocationSettings::default(),
            ),
            gate_location_settings_confirmed: false,
        },
    }
}

pub(super) fn project_service_begin_material_confirmation(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    material_id: &str,
) -> Result<bool, panta_core::project::ProjectError> {
    service.service.begin_material_confirmation(
        std::path::Path::new(project_path),
        revision,
        import_id,
        material_id,
    )
}

pub(super) fn project_service_finish_material_confirmation(
    service: &mut ProjectService,
) -> Result<bool, panta_core::project::ProjectError> {
    service.service.finish_material_confirmation()
}

pub(super) fn project_service_set_analysis_sequence(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    sequence_id: &str,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service
        .service
        .set_analysis_sequence(
            std::path::Path::new(project_path),
            revision,
            import_id,
            sequence_id,
        )
        .map(project_snapshot)
}

#[cfg(test)]
pub(super) fn project_service_inspect_stl(
    service: &mut ProjectService,
    source: String,
) -> Result<bridge::StlImportPreview, panta_core::project::ProjectError> {
    service
        .service
        .inspect_stl(std::path::Path::new(&source))
        .map(stl_import_preview)
}

pub(super) fn project_service_mesh_snapshot_for_import(
    service: &ProjectService,
    import_id: &str,
) -> Result<bridge::SurfaceMeshSnapshot, bridge::ProjectDiagnostic> {
    let snapshot = service
        .service
        .current()
        .map_err(bridge::ProjectDiagnostic::from)?;
    let mesh = service.service.mesh_for_import(import_id).ok_or_else(|| {
        project_response::diagnostic("project.mesh_not_resident", "state", import_id.to_owned())
    })?;
    Ok(bridge::SurfaceMeshSnapshot {
        coordinates: mesh_coordinates(mesh).map_err(|detail| {
            project_response::diagnostic("project.mesh_allocation_failed", "resource", detail)
        })?,
        revision: snapshot.revision,
    })
}

pub(super) fn project_service_activate_mesh_document(
    service: &mut ProjectService,
    import_id: &str,
) -> bool {
    service.service.activate_mesh_document(import_id)
}

pub(super) fn project_service_deactivate_mesh_document(service: &mut ProjectService) {
    service.service.deactivate_mesh_document();
}

pub(super) fn project_service_release_mesh_document(service: &mut ProjectService, import_id: &str) {
    service.service.release_mesh_document(import_id);
}

pub(super) fn project_service_resident_mesh_ids(service: &ProjectService) -> Vec<String> {
    service.service.resident_mesh_ids()
}

pub(super) fn project_service_begin_asset_activation(
    service: &mut ProjectService,
    import_id: &str,
) -> Result<bridge::ActivationAttempt, panta_core::project::ProjectError> {
    service
        .service
        .begin_asset_activation(import_id)
        .map(|attempt| bridge::ActivationAttempt {
            attempt: attempt.attempt,
            generation: attempt.generation,
        })
}

pub(super) fn project_service_cancel_asset_activation(
    service: &mut ProjectService,
    attempt: u64,
) -> bool {
    service.service.cancel_asset_activation(attempt)
}

pub(super) fn project_service_drain_asset_activations(
    service: &mut ProjectService,
) -> Vec<bridge::ActivationOutcome> {
    service
        .service
        .drain_asset_activations()
        .into_iter()
        .map(|outcome| activation_outcome_to_bridge(&mut service.service, outcome))
        .collect()
}

pub(super) fn activation_outcome_to_bridge(
    service: &mut panta_core::project::ProjectService,
    outcome: panta_core::project::Outcome,
) -> bridge::ActivationOutcome {
    let (kind, error, coordinates) = match outcome.kind {
        panta_core::project::OutcomeKind::Succeeded
            if !service.is_current_activation_attempt(&outcome.import_id, outcome.attempt) =>
        {
            // 旧 attempt 的成功结果既不进缓存，也不跨 FFI 携带大网格。
            (
                bridge::ActivationOutcomeKind::Expired,
                bridge::ProjectDiagnostic::default(),
                Vec::new(),
            )
        }
        panta_core::project::OutcomeKind::Succeeded => match outcome.mesh {
            Some(mesh) => match mesh_coordinates(&mesh) {
                Ok(coordinates) => {
                    // DTO 由当前 outcome 的 Mesh 构造；缓存提交再次校验 attempt。
                    if service.cache_activation_result(&outcome.import_id, outcome.attempt, mesh) {
                        (
                            bridge::ActivationOutcomeKind::Succeeded,
                            bridge::ProjectDiagnostic::default(),
                            coordinates,
                        )
                    } else {
                        (
                            bridge::ActivationOutcomeKind::Expired,
                            bridge::ProjectDiagnostic::default(),
                            Vec::new(),
                        )
                    }
                }
                Err(detail) => {
                    service.finish_asset_activation(&outcome.import_id, outcome.attempt);
                    (
                        bridge::ActivationOutcomeKind::Failed,
                        project_response::diagnostic(
                            "project.mesh_allocation_failed",
                            "resource",
                            detail,
                        ),
                        Vec::new(),
                    )
                }
            },
            None => {
                service.finish_asset_activation(&outcome.import_id, outcome.attempt);
                (
                    bridge::ActivationOutcomeKind::Failed,
                    project_response::diagnostic(
                        "project.mesh_missing",
                        "state",
                        outcome.import_id.clone(),
                    ),
                    Vec::new(),
                )
            }
        },
        kind => {
            service.finish_asset_activation(&outcome.import_id, outcome.attempt);
            let error = bridge::ProjectDiagnostic {
                code: outcome.code,
                category: outcome.category.to_owned(),
                detail: outcome.detail,
            };
            (activation_kind(kind), error, Vec::new())
        }
    };
    bridge::ActivationOutcome {
        attempt: outcome.attempt,
        generation: outcome.generation,
        import_id: outcome.import_id,
        kind,
        error,
        coordinates,
    }
}

fn activation_kind(kind: panta_core::project::OutcomeKind) -> bridge::ActivationOutcomeKind {
    match kind {
        panta_core::project::OutcomeKind::Succeeded => bridge::ActivationOutcomeKind::Succeeded,
        panta_core::project::OutcomeKind::Failed => bridge::ActivationOutcomeKind::Failed,
        panta_core::project::OutcomeKind::Cancelled => bridge::ActivationOutcomeKind::Cancelled,
        panta_core::project::OutcomeKind::Expired => bridge::ActivationOutcomeKind::Expired,
    }
}

fn project_snapshot(snapshot: panta_core::project::ProjectSnapshot) -> bridge::ProjectSnapshot {
    bridge::ProjectSnapshot {
        path: snapshot.path.display().to_string(),
        name: snapshot.name,
        revision: snapshot.revision,
        dirty: snapshot.dirty,
    }
}

fn project_import(import: panta_core::project::ImportRecord) -> bridge::ProjectImport {
    bridge::ProjectImport {
        record_version: import.record_version,
        parser_version: import.parser_version,
        id: import.id,
        source_name: import.source_name,
        asset: import.asset,
        format: import.format,
        mesh_type: import.mesh_type,
        units: import.units,
        show_import_log: import.show_import_log,
        triangle_count: import.triangle_count,
        size_x: import.dimensions[0],
        size_y: import.dimensions[1],
        size_z: import.dimensions[2],
    }
}

pub(super) fn stl_import_preview(
    preview: panta_core::project::StlImportPreview,
) -> bridge::StlImportPreview {
    bridge::StlImportPreview {
        source_name: preview.source_name,
        triangle_count: preview.triangle_count,
        size_x: preview.dimensions[0],
        size_y: preview.dimensions[1],
        size_z: preview.dimensions[2],
    }
}

/// CXX 枚举跨边界可能携带越界表示；未知取值按稳定错误码拒绝，不猜测。
fn gate_location_settings_dto(
    settings: panta_core::project::GateLocationSettings,
) -> bridge::GateLocationSettings {
    bridge::GateLocationSettings {
        machine_source_text: settings.machine_source_text().to_owned(),
        machine_id: settings.machine_id,
        mold_temperature_celsius: settings.mold_temperature_celsius,
        melt_temperature_celsius: settings.melt_temperature_celsius,
        algorithm_id: settings.algorithm.id().to_owned(),
        algorithm_source_text: settings.algorithm.source_text().to_owned(),
        number_of_gates: settings.number_of_gates,
    }
}

pub(super) fn project_service_begin_gate_location_settings_confirmation(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    settings: bridge::GateLocationSettings,
) -> Result<bool, panta_core::project::ProjectError> {
    let algorithm = panta_core::project::GateLocatorAlgorithm::from_id(&settings.algorithm_id)?;
    service.service.begin_gate_location_settings_confirmation(
        std::path::Path::new(project_path),
        revision,
        import_id,
        panta_core::project::GateLocationSettings {
            machine_id: settings.machine_id,
            mold_temperature_celsius: settings.mold_temperature_celsius,
            melt_temperature_celsius: settings.melt_temperature_celsius,
            algorithm,
            number_of_gates: settings.number_of_gates,
        },
    )
}

pub(super) fn project_service_finish_gate_location_settings_confirmation(
    service: &mut ProjectService,
) -> Result<bool, panta_core::project::ProjectError> {
    service.service.finish_gate_location_settings_confirmation()
}

fn fill_settings_dto(settings: panta_core::project::FillSettings) -> bridge::FillSettings {
    bridge::FillSettings {
        mold_temperature_celsius: settings.mold_temperature_celsius,
        melt_temperature_celsius: settings.melt_temperature_celsius,
        flow_rate_cm3_per_second: settings.flow_rate_cm3_per_second,
        switch_over_volume_percent: settings.switch_over_volume_percent,
        fiber_orientation: settings.fiber_orientation,
        crystallization: settings.crystallization,
        holding_profile: settings
            .holding_profile
            .into_iter()
            .map(|point| bridge::HoldingProfilePoint {
                duration_seconds: point.duration_seconds,
                pressure_percent: point.pressure_percent,
            })
            .collect(),
    }
}

pub(super) fn project_service_begin_fill_settings_confirmation(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    settings: bridge::FillSettings,
) -> Result<bool, panta_core::project::ProjectError> {
    service.service.begin_fill_settings_confirmation(
        std::path::Path::new(project_path),
        revision,
        import_id,
        panta_core::project::FillSettings {
            mold_temperature_celsius: settings.mold_temperature_celsius,
            melt_temperature_celsius: settings.melt_temperature_celsius,
            flow_rate_cm3_per_second: settings.flow_rate_cm3_per_second,
            switch_over_volume_percent: settings.switch_over_volume_percent,
            fiber_orientation: settings.fiber_orientation,
            crystallization: settings.crystallization,
            holding_profile: settings
                .holding_profile
                .into_iter()
                .map(|point| panta_core::project::HoldingProfilePoint {
                    duration_seconds: point.duration_seconds,
                    pressure_percent: point.pressure_percent,
                })
                .collect(),
        },
    )
}

pub(super) fn project_service_finish_fill_settings_confirmation(
    service: &mut ProjectService,
) -> Result<bool, panta_core::project::ProjectError> {
    service.service.finish_fill_settings_confirmation()
}

#[cfg(test)]
mod tests {
    use crate::{
        activation_outcome_to_bridge, bridge, project_service_create, project_service_current,
        project_service_execute, project_service_mesh_snapshot_for_import, project_service_new,
        project_service_open, project_service_resident_mesh_ids, project_service_save,
    };
    use std::fs;
    use std::path::PathBuf;
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
        let project =
            project_service_create(&mut service, root.display().to_string(), "Demo".into())?;
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
    fn project_service_bridge_maps_snapshots_and_errors() -> Result<(), Box<dyn std::error::Error>>
    {
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
        let missing = match crate::project_service_begin_asset_activation(&mut service, "import-99")
        {
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
        let preview =
            crate::project_service_inspect_stl(&mut service, source.display().to_string())?;
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
        assert_eq!(imported.asset, "assets/imports/0001-part.stl");
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
        let unsupported = match crate::project_service_inspect_stl(
            &mut service,
            step_source.display().to_string(),
        ) {
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
        let mut service = project_service_new();
        let outcome = panta_core::project::Outcome {
            attempt: 7,
            generation: 3,
            import_id: "import-2".into(),
            kind: panta_core::project::OutcomeKind::Failed,
            code: "project.asset_read_failed".into(),
            category: "io",
            detail: "C:\\资产: read denied".into(),
            mesh: None,
        };
        let result = activation_outcome_to_bridge(&mut service.service, outcome);
        assert!(result.kind == bridge::ActivationOutcomeKind::Failed);
        assert_eq!(result.attempt, 7);
        assert_eq!(result.generation, 3);
        assert_eq!(result.import_id, "import-2");
        assert_eq!(result.error.code, "project.asset_read_failed");
        assert_eq!(result.error.category, "io");
        assert_eq!(result.error.detail, "C:\\资产: read denied");
        assert!(result.coordinates.is_empty());
    }

    #[test]
    fn activation_without_a_current_attempt_does_not_cross_the_bridge() {
        let mut service = project_service_new();
        let outcome = |attempt, x| panta_core::project::Outcome {
            attempt,
            generation: 0,
            import_id: "import-1".to_owned(),
            kind: panta_core::project::OutcomeKind::Succeeded,
            code: String::new(),
            category: "",
            detail: String::new(),
            mesh: Some(panta_mesh::SurfaceMesh {
                triangles: vec![[[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]]],
            }),
        };

        let first = activation_outcome_to_bridge(&mut service.service, outcome(1, 1.0));
        let second = activation_outcome_to_bridge(&mut service.service, outcome(2, 10.0));

        assert!(first.kind == bridge::ActivationOutcomeKind::Expired);
        assert!(first.coordinates.is_empty());
        assert!(second.kind == bridge::ActivationOutcomeKind::Expired);
        assert!(second.coordinates.is_empty());
    }
}
