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
response_conversion!(bridge::StlImportPreviewResult, bridge::StlImportPreview);
response_conversion!(bridge::MaterialDefinitionResult, bridge::MaterialDefinition);
response_conversion!(bridge::ProjectConfirmationResult, bool);
response_conversion!(
    bridge::SurfaceMeshSnapshotResult,
    bridge::SurfaceMeshSnapshot
);
response_conversion!(bridge::ActivationAttemptResult, bridge::ActivationAttempt);

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

pub(super) fn project_service_inspect_stl_response(
    service: &mut ProjectService,
    source: String,
) -> bridge::StlImportPreviewResult {
    crate::project_service_inspect_stl(service, source).into()
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
mod tests {
    use super::*;

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
                ProjectError::ImportAssetCopyFailed(context.into()),
                "project.import_asset_copy_failed",
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
        check!(bridge::StlImportPreviewResult, bridge::StlImportPreview);
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
        let result =
            project_service_create_response(&mut service, "relative".into(), "Demo".into());
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
}
