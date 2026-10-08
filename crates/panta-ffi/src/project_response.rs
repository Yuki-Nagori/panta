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
#[path = "../../../tests/support/rust/temp_directory.rs"]
mod temp_directory;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_revision_crosses_cxx_as_diagnostic_without_changing_the_project()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
        let path = fixture.root.join("Max.panta");
        let bytes = format!(r#"{{"schema":2,"name":"Max","revision":{}}}"#, u64::MAX);
        std::fs::write(&path, &bytes)?;
        let mut service = crate::project_service_new();
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
        assert_eq!(std::fs::read_to_string(path)?, bytes);
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

    #[test]
    fn preview_bridge_distinguishes_pending_success_failure_and_cancelled_requests()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = temp_directory::Fixture::new()?;
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
}
