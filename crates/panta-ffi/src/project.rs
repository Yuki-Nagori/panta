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
        .execute(panta_core::project::ProjectCommand::Open {
            path: std::path::PathBuf::from(path),
        })
        .map(project_snapshot)
}

pub(super) fn project_service_save(
    service: &mut ProjectService,
) -> Result<bridge::ProjectSnapshot, panta_core::project::ProjectError> {
    service.service.save().map(project_snapshot)
}

/// 独占后台 receiver；C++ 只能移交给等待线程，不能借用 ProjectService。
pub struct VariableDocumentRequest {
    request: Result<
        panta_core::project::VariableDocumentRequest,
        panta_core::project::VariableDocumentError,
    >,
    generation: u64,
}

/// 后台等待结束后移交服务线程消费的拥有式结果。
pub struct VariableDocumentCompletion {
    completion: Result<
        panta_core::project::VariableDocumentCompletion,
        panta_core::project::VariableDocumentError,
    >,
    generation: u64,
}

pub(super) fn project_service_begin_variable_document(
    service: &mut ProjectService,
    expected_generation: u64,
    expected_revision: u64,
    source: String,
    save: bool,
) -> Box<VariableDocumentRequest> {
    Box::new(VariableDocumentRequest {
        request: service.service.begin_variable_document(
            expected_generation,
            expected_revision,
            save.then_some(source),
        ),
        generation: expected_generation,
    })
}

pub(super) fn variable_document_request_accepted(request: &VariableDocumentRequest) -> bool {
    request.request.is_ok()
}

#[expect(
    clippy::boxed_local,
    reason = "CXX opaque 请求必须以 Box 移交并消费唯一所有权"
)]
pub(super) fn variable_document_wait(
    request: Box<VariableDocumentRequest>,
) -> Box<VariableDocumentCompletion> {
    Box::new(VariableDocumentCompletion {
        completion: request
            .request
            .map(panta_core::project::VariableDocumentRequest::wait),
        generation: request.generation,
    })
}

#[expect(
    clippy::boxed_local,
    reason = "CXX opaque 完成结果必须以 Box 移回 Rust 消费"
)]
pub(super) fn project_service_finish_variable_document(
    service: &mut ProjectService,
    completion: Box<VariableDocumentCompletion>,
) -> bridge::VariableDocumentResult {
    variable_document_result(
        completion
            .completion
            .and_then(|completion| service.service.finish_variable_document(completion)),
        completion.generation,
    )
}

pub(super) fn project_service_variable_document_generation(service: &ProjectService) -> u64 {
    service.service.variable_document_generation()
}

fn variable_document_result(
    result: Result<
        (String, panta_core::runtime::VariableSnapshot),
        panta_core::project::VariableDocumentError,
    >,
    generation: u64,
) -> bridge::VariableDocumentResult {
    match result {
        Ok((source, snapshot)) => bridge::VariableDocumentResult {
            source,
            generation,
            revision: snapshot.revision(),
            values: snapshot
                .iter()
                .map(|(name, value)| {
                    use panta_core::runtime::RuntimeValue;
                    let (value_type, value) = match value {
                        RuntimeValue::Boolean(value) => ("bool", value.to_string()),
                        RuntimeValue::Integer(value) => ("int", value.to_string()),
                        RuntimeValue::Real(value) => ("real", value.to_string()),
                        RuntimeValue::String(value) => ("string", value.clone()),
                        RuntimeValue::Resource(value) => ("resource", value.to_logical()),
                    };
                    bridge::VariableValue {
                        name: name.to_owned(),
                        value_type: value_type.to_owned(),
                        value,
                    }
                })
                .collect(),
            ..bridge::VariableDocumentResult::default()
        },
        Err(error) => {
            let mut response = bridge::VariableDocumentResult {
                generation,
                ..bridge::VariableDocumentResult::default()
            };
            match error {
                panta_core::project::VariableDocumentError::Diagnostics(diagnostics) => {
                    response.diagnostics = diagnostics
                        .diagnostics
                        .into_iter()
                        .map(|diagnostic| bridge::VariableDiagnostic {
                            code: diagnostic.code,
                            message: diagnostic.message,
                            start_offset: diagnostic.start_offset as u64,
                            end_offset: diagnostic.end_offset as u64,
                            line: diagnostic.line as u64,
                            column: diagnostic.column as u64,
                        })
                        .collect();
                }
                panta_core::project::VariableDocumentError::SourceTooLarge { size, limit } => {
                    response.error = panta_core::project::ProjectError::CommandInvalid(format!(
                        "normalized source is {size} bytes; limit is {limit}"
                    ))
                    .into();
                    response.error.code = "variables.source_too_large".to_owned();
                    response.error.category = "validation".to_owned();
                }
                panta_core::project::VariableDocumentError::InvalidResource {
                    name,
                    reference,
                    source_range,
                    line,
                    column,
                    error,
                } => {
                    response.diagnostics.push(bridge::VariableDiagnostic {
                        code: "variables.invalid_resource".to_owned(),
                        message: format!("{name}: {reference}: {error}"),
                        start_offset: source_range.start as u64,
                        end_offset: source_range.end as u64,
                        line: line as u64,
                        column: column as u64,
                    });
                }
                panta_core::project::VariableDocumentError::Commit(error) => {
                    let code = match error {
                        panta_core::runtime::RuntimeCommitError::NoContext => {
                            "variables.no_context"
                        }
                        panta_core::runtime::RuntimeCommitError::StaleRevision { .. } => {
                            "variables.stale_revision"
                        }
                        panta_core::runtime::RuntimeCommitError::StaleContext => {
                            "variables.stale_generation"
                        }
                        _ => "variables.commit_failed",
                    };
                    response.error =
                        panta_core::project::ProjectError::CommandInvalid(error.to_string()).into();
                    response.error.code = code.to_owned();
                    response.error.category = "state".to_owned();
                }
                panta_core::project::VariableDocumentError::Project(error) => {
                    response.error = error.into();
                }
            }
            response
        }
    }
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
        .and_then(|import| project_import(&service.service, import))
}

pub(super) fn project_service_begin_stl_import(
    service: &mut ProjectService,
    source: String,
    mesh_type: String,
    units: String,
    show_import_log: bool,
) -> Result<u64, panta_core::project::ProjectError> {
    service.service.begin_stl_import(
        std::path::Path::new(&source),
        &mesh_type,
        &units,
        show_import_log,
    )
}

pub(super) fn project_service_finish_stl_import(
    service: &mut ProjectService,
    request: u64,
) -> Result<Option<bridge::ProjectImport>, panta_core::project::ProjectError> {
    service
        .service
        .finish_stl_import(request)
        .and_then(|record| {
            record
                .map(|record| project_import(&service.service, record))
                .transpose()
        })
}

pub(super) fn project_service_cancel_stl_import(
    service: &mut ProjectService,
    request: u64,
) -> bool {
    service.service.cancel_stl_import(request)
}

pub(super) fn project_service_imports(
    service: &ProjectService,
) -> Result<Vec<bridge::ProjectImport>, panta_core::project::ProjectError> {
    service.service.imports().and_then(|imports| {
        imports
            .into_iter()
            .map(|import| project_import(&service.service, import))
            .collect()
    })
}

pub(super) fn project_service_workspace_snapshot(
    service: &ProjectService,
) -> bridge::WorkspaceSnapshot {
    let snapshot = service.service.document_workspace();
    bridge::WorkspaceSnapshot {
        documents: snapshot
            .documents
            .into_iter()
            .map(|document| bridge::WorkspaceDocument {
                id: document.id,
                kind: document.kind.as_str().to_owned(),
                state: document.state.as_str().to_owned(),
                title: document.title,
                message_key: document.message_key,
                diagnostic_code: document.diagnostic.code,
                diagnostic_category: document.diagnostic.category,
                diagnostic_detail: document.diagnostic.detail,
                display_ordinal: document.display_ordinal,
                display_count: document.display_count,
            })
            .collect(),
        active_document_id: snapshot.active_document_id,
    }
}

pub(super) fn project_service_workspace_reset(service: &mut ProjectService, welcome_title: String) {
    service.service.reset_document_workspace(&welcome_title);
}

pub(super) fn project_service_workspace_open_import(
    service: &mut ProjectService,
    import_id: &str,
    title: &str,
) {
    service.service.open_import_document(import_id, title);
}

pub(super) fn project_service_workspace_publish_import(
    service: &mut ProjectService,
    import_id: &str,
    title: &str,
) {
    service.service.publish_import_document(import_id, title);
}

pub(super) fn project_service_workspace_activate(service: &mut ProjectService, document_id: &str) {
    service.service.activate_workspace_document(document_id);
}

pub(super) fn project_service_workspace_close(service: &mut ProjectService, document_id: &str) {
    service.service.close_workspace_document(document_id);
}

pub(super) fn project_service_workspace_move(service: &mut ProjectService, from: u64, to: u64) {
    let (Ok(from), Ok(to)) = (usize::try_from(from), usize::try_from(to)) else {
        return;
    };
    service.service.move_workspace_document(from, to);
}

pub(super) fn project_error_message_key(code: &str) -> String {
    panta_core::project::project_error_message_key(code).to_owned()
}

pub(super) fn format_project_dimensions(
    size_x: f64,
    size_y: f64,
    size_z: f64,
    unit: &str,
) -> String {
    panta_core::project::format_project_dimensions(size_x, size_y, size_z, unit)
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
            process_constraints: process_constraints_dto(),
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
            process_constraints: process_constraints_dto(),
        },
    }
}

fn numeric_constraint_dto(
    constraint: panta_core::project::NumericConstraint,
) -> bridge::NumericConstraint {
    bridge::NumericConstraint {
        minimum: constraint.minimum,
        maximum: constraint.maximum,
        has_minimum: constraint.has_minimum,
        has_maximum: constraint.has_maximum,
        minimum_inclusive: constraint.minimum_inclusive,
        maximum_inclusive: constraint.maximum_inclusive,
        integer: constraint.integer,
        unit: constraint.unit.to_owned(),
    }
}

fn process_constraints_dto() -> bridge::ProcessConstraints {
    bridge::ProcessConstraints {
        temperature: numeric_constraint_dto(panta_core::project::TEMPERATURE_CONSTRAINT),
        flow_rate: numeric_constraint_dto(panta_core::project::FLOW_RATE_CONSTRAINT),
        switch_over_volume: numeric_constraint_dto(
            panta_core::project::SWITCH_OVER_VOLUME_CONSTRAINT,
        ),
        holding_duration: numeric_constraint_dto(panta_core::project::HOLDING_DURATION_CONSTRAINT),
        holding_pressure: numeric_constraint_dto(panta_core::project::HOLDING_PRESSURE_CONSTRAINT),
        number_of_gates: numeric_constraint_dto(panta_core::project::NUMBER_OF_GATES_CONSTRAINT),
        holding_profile_max_points: panta_core::project::HOLDING_PROFILE_MAX_POINTS as u64,
    }
}

pub(super) fn validate_fill_settings_draft(
    settings: bridge::FillSettings,
) -> bridge::ProcessDraftValidation {
    let errors = panta_core::project::FillSettings {
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
    }
    .validation_errors();
    bridge::ProcessDraftValidation {
        valid: errors.is_empty(),
        errors,
    }
}

pub(super) fn validate_holding_profile_draft(
    holding_profile: Vec<bridge::HoldingProfilePoint>,
) -> bridge::ProcessDraftValidation {
    let settings = panta_core::project::FillSettings {
        holding_profile: holding_profile
            .into_iter()
            .map(|point| panta_core::project::HoldingProfilePoint {
                duration_seconds: point.duration_seconds,
                pressure_percent: point.pressure_percent,
            })
            .collect(),
        ..panta_core::project::FillSettings::default()
    };
    let errors = settings
        .validation_errors()
        .into_iter()
        .filter(|key| key == "holdingProfile" || key.starts_with("holdingProfile."))
        .collect::<Vec<_>>();
    bridge::ProcessDraftValidation {
        valid: errors.is_empty(),
        errors,
    }
}

pub(super) fn validate_gate_location_settings_draft(
    mold_temperature_celsius: f64,
    melt_temperature_celsius: f64,
    number_of_gates: f64,
    machine_id: String,
) -> bridge::ProcessDraftValidation {
    let errors = panta_core::project::GateLocationSettings {
        machine_id,
        mold_temperature_celsius,
        melt_temperature_celsius,
        algorithm: panta_core::project::GateLocatorAlgorithm::AdvancedGateLocator,
        number_of_gates: 0,
    }
    .validation_errors_for_gate_count(number_of_gates);
    bridge::ProcessDraftValidation {
        valid: errors.is_empty(),
        errors,
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
        .map(activation_outcome_to_bridge)
        .collect()
}

pub(super) fn project_service_drain_workspace_activations(service: &mut ProjectService) {
    service.service.drain_workspace_activations();
}

fn activation_outcome_to_bridge(
    outcome: panta_core::project::CompletedActivation,
) -> bridge::ActivationOutcome {
    let mut result = bridge::ActivationOutcome {
        attempt: outcome.attempt,
        generation: outcome.generation,
        import_id: outcome.import_id,
        kind: activation_kind(outcome.kind),
        error: bridge::ProjectDiagnostic {
            code: outcome.code,
            category: outcome.category.to_owned(),
            detail: outcome.detail,
        },
        coordinates: Vec::new(),
    };
    if let Some(mesh) = outcome.mesh {
        match mesh_coordinates(&mesh) {
            Ok(coordinates) => result.coordinates = coordinates,
            Err(detail) => {
                // 显示格式分配失败只影响此适配结果，不撤销 core 已完成的资产激活。
                result.kind = bridge::ActivationOutcomeKind::Failed;
                result.error = project_response::diagnostic(
                    "project.mesh_allocation_failed",
                    "resource",
                    detail,
                );
            }
        }
    }
    result
}

fn activation_kind(kind: panta_core::project::OutcomeKind) -> bridge::ActivationOutcomeKind {
    match kind {
        panta_core::project::OutcomeKind::Succeeded => bridge::ActivationOutcomeKind::Succeeded,
        panta_core::project::OutcomeKind::Failed => bridge::ActivationOutcomeKind::Failed,
        panta_core::project::OutcomeKind::Cancelled => bridge::ActivationOutcomeKind::Cancelled,
        panta_core::project::OutcomeKind::Expired => bridge::ActivationOutcomeKind::Expired,
    }
}

pub(super) fn project_snapshot(
    snapshot: panta_core::project::ProjectSnapshot,
) -> bridge::ProjectSnapshot {
    bridge::ProjectSnapshot {
        path: snapshot.path.display().to_string(),
        name: snapshot.name,
        revision: snapshot.revision,
        dirty: snapshot.dirty,
    }
}

fn project_import(
    service: &panta_core::project::ProjectService,
    import: panta_core::project::ImportRecord,
) -> Result<bridge::ProjectImport, panta_core::project::ProjectError> {
    let mesh_asset_path = service.mesh_asset_path_for_import(&import.id)?;
    Ok(bridge::ProjectImport {
        record_version: import.record_version,
        parser_version: import.parser_version,
        id: import.id,
        display_name: import.display_name,
        source_name: import.source_name,
        mesh_asset_path,
        format: import.format,
        mesh_type: import.mesh_type,
        units: import.units,
        show_import_log: import.show_import_log,
        triangle_count: import.triangle_count,
        size_x: import.dimensions[0],
        size_y: import.dimensions[1],
        size_z: import.dimensions[2],
    })
}

pub(super) fn stl_import_preview(
    preview: panta_core::project::StlImportPreview,
) -> bridge::StlImportPreview {
    bridge::StlImportPreview {
        source_name: preview.source_name,
        triangle_count: preview.triangle_count,
        expanded_mesh_bytes: preview.expanded_mesh_bytes,
        large_mesh_warning: preview.large_mesh_warning,
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
mod tests;
