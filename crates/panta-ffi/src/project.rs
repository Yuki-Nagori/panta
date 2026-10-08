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
