//! 工程服务的 CXX 返回值适配；不解析错误文本，不持有另一份工程状态。
use crate::{ProjectService, bridge};
use panta_core::project::ProjectError;

pub(crate) fn diagnostic(code: &str, category: &str, detail: String) -> bridge::ProjectDiagnostic {
    bridge::ProjectDiagnostic {
        code: code.to_owned(),
        category: category.to_owned(),
        detail,
    }
}

impl From<ProjectError> for bridge::ProjectDiagnostic {
    fn from(error: ProjectError) -> Self {
        diagnostic(error.code(), error.category(), error.detail())
    }
}

// 只在此处构造结果，保证成功诊断为空、失败载荷为缺省值。
macro_rules! response_conversion {
    ($response:ty, $value:ty) => {
        impl<E: Into<bridge::ProjectDiagnostic>> From<Result<$value, E>> for $response {
            fn from(result: Result<$value, E>) -> Self {
                match result {
                    Ok(value) => Self {
                        value,
                        ..Self::default()
                    },
                    Err(error) => Self {
                        error: error.into(),
                        ..Self::default()
                    },
                }
            }
        }
    };
}

response_conversion!(bridge::ProjectSnapshotResult, bridge::ProjectSnapshot);
response_conversion!(bridge::ProjectImportResult, bridge::ProjectImport);
response_conversion!(bridge::ProjectImportsResult, Vec<bridge::ProjectImport>);
response_conversion!(bridge::MaterialDefinitionResult, bridge::MaterialDefinition);
response_conversion!(bridge::ProjectConfirmationResult, bool);
response_conversion!(
    bridge::SurfaceMeshSnapshotResult,
    bridge::SurfaceMeshSnapshot
);
response_conversion!(bridge::ActivationAttemptResult, bridge::ActivationAttempt);
response_conversion!(bridge::StlPreviewRequestResult, u64);

pub(super) fn project_service_create_response(
    service: &mut ProjectService,
    location: String,
    name: String,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_create(service, location, name).into()
}

pub(super) fn project_service_open_response(
    service: &mut ProjectService,
    path: String,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_open(service, path).into()
}

pub(super) fn project_service_save_response(
    service: &mut ProjectService,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_save(service).into()
}

pub(super) fn project_service_execute_response(
    service: &mut ProjectService,
    command: bridge::ProjectCommand,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_execute(service, command).into()
}

pub(super) fn project_service_current_response(
    service: &ProjectService,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_current(service).into()
}

pub(super) fn project_service_set_analysis_sequence_response(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    sequence_id: &str,
) -> bridge::ProjectSnapshotResult {
    crate::project_service_set_analysis_sequence(
        service,
        project_path,
        revision,
        import_id,
        sequence_id,
    )
    .into()
}

pub(super) fn project_service_import_stl_response(
    service: &mut ProjectService,
    source: String,
    mesh_type: String,
    units: String,
    show_import_log: bool,
) -> bridge::ProjectImportResult {
    crate::project_service_import_stl(service, source, mesh_type, units, show_import_log).into()
}

pub(super) fn project_service_imports_response(
    service: &ProjectService,
) -> bridge::ProjectImportsResult {
    crate::project_service_imports(service).into()
}

pub(super) fn project_service_begin_stl_preview_response(
    service: &mut ProjectService,
    source: String,
) -> bridge::StlPreviewRequestResult {
    service
        .service
        .begin_stl_preview(std::path::Path::new(&source))
        .into()
}

pub(super) fn project_service_finish_stl_preview_response(
    service: &mut ProjectService,
    request: u64,
) -> bridge::StlPreviewPoll {
    match service.service.finish_stl_preview(request) {
        Ok(Some(preview)) => bridge::StlPreviewPoll {
            ready: true,
            value: crate::stl_import_preview(preview),
            ..Default::default()
        },
        Ok(None) => bridge::StlPreviewPoll::default(),
        Err(error) => bridge::StlPreviewPoll {
            error: error.into(),
            ..Default::default()
        },
    }
}

pub(super) fn project_service_cancel_stl_preview(
    service: &mut ProjectService,
    request: u64,
) -> bool {
    service.service.cancel_stl_preview(request)
}

pub(super) fn project_service_begin_stl_import_response(
    service: &mut ProjectService,
    source: String,
    mesh_type: String,
    units: String,
    show_import_log: bool,
) -> bridge::StlPreviewRequestResult {
    crate::begin_stl_import(service, source, mesh_type, units, show_import_log).into()
}

pub(super) fn project_service_finish_stl_import_response(
    service: &mut ProjectService,
    request: u64,
) -> bridge::StlImportPoll {
    match crate::project_service_finish_stl_import(service, request) {
        Ok(Some(record)) => bridge::StlImportPoll {
            ready: true,
            value: record,
            ..Default::default()
        },
        Ok(None) => bridge::StlImportPoll::default(),
        Err(error) => bridge::StlImportPoll {
            ready: true,
            error: error.into(),
            ..Default::default()
        },
    }
}

pub(super) fn project_service_cancel_stl_import_response(
    service: &mut ProjectService,
    request: u64,
) -> bool {
    crate::cancel_stl_import(service, request)
}

pub(super) fn default_material_response() -> bridge::MaterialDefinitionResult {
    crate::default_material().into()
}

pub(super) fn project_service_begin_material_confirmation_response(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    material_id: &str,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_begin_material_confirmation(
        service,
        project_path,
        revision,
        import_id,
        material_id,
    )
    .into()
}

pub(super) fn project_service_finish_material_confirmation_response(
    service: &mut ProjectService,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_finish_material_confirmation(service).into()
}

pub(super) fn project_service_begin_fill_settings_confirmation_response(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    settings: bridge::FillSettings,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_begin_fill_settings_confirmation(
        service,
        project_path,
        revision,
        import_id,
        settings,
    )
    .into()
}

pub(super) fn project_service_finish_fill_settings_confirmation_response(
    service: &mut ProjectService,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_finish_fill_settings_confirmation(service).into()
}

pub(super) fn project_service_begin_gate_location_settings_confirmation_response(
    service: &mut ProjectService,
    project_path: &str,
    revision: u64,
    import_id: &str,
    settings: bridge::GateLocationSettings,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_begin_gate_location_settings_confirmation(
        service,
        project_path,
        revision,
        import_id,
        settings,
    )
    .into()
}

pub(super) fn project_service_finish_gate_location_settings_confirmation_response(
    service: &mut ProjectService,
) -> bridge::ProjectConfirmationResult {
    crate::project_service_finish_gate_location_settings_confirmation(service).into()
}

pub(super) fn project_service_mesh_snapshot_for_import_response(
    service: &ProjectService,
    import_id: &str,
) -> bridge::SurfaceMeshSnapshotResult {
    crate::project_service_mesh_snapshot_for_import(service, import_id).into()
}

pub(super) fn project_service_begin_asset_activation_response(
    service: &mut ProjectService,
    import_id: &str,
) -> bridge::ActivationAttemptResult {
    crate::project_service_begin_asset_activation(service, import_id).into()
}

impl std::fmt::Display for bridge::ProjectDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.detail.is_empty() {
            write!(formatter, "{}", self.code)
        } else {
            write!(formatter, "{}: {}", self.code, self.detail)
        }
    }
}

impl std::error::Error for bridge::ProjectDiagnostic {}

#[cfg(test)]
mod tests;
