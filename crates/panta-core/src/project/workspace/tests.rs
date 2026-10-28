use super::*;

fn add_import(workspace: &mut DocumentWorkspace, id: &str, title: &str, attempt: u64) {
    assert!(workspace.open_import(id, title));
    assert_eq!(
        workspace.activation_request(id),
        ActivationRequest::LoadImport(id.to_owned())
    );
    assert!(workspace.set_activation_attempt(id, attempt));
}

#[test]
fn newly_opened_import_requests_its_first_activation() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    assert!(workspace.open_import("first", "part"));

    assert_eq!(
        workspace.activation_request("first"),
        ActivationRequest::LoadImport("first".into())
    );
    assert_eq!(
        workspace.snapshot().documents[1].state,
        DocumentState::Loading
    );
}

#[test]
fn ready_documents_activate_and_reorder_preserves_active_identity() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "a", "part", 1);
    assert!(workspace.resolve_activation(
        "a",
        1,
        ActivationResultKind::Succeeded,
        "",
        DocumentDiagnostic::default()
    ));
    assert_eq!(
        workspace.activation_request("a"),
        ActivationRequest::ActivateImport("a".into())
    );
    assert_eq!(workspace.active_document_id(), Some("a"));
    assert!(workspace.move_document(1, 0));
    assert_eq!(workspace.snapshot().documents[0].id, "a");
    assert_eq!(workspace.active_document_id(), Some("a"));
}

#[test]
fn closing_active_document_prefers_ready_right_then_ready_left() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "left", "left", 1);
    add_import(&mut workspace, "active", "active", 2);
    add_import(&mut workspace, "right", "right", 3);
    for (id, attempt) in [("left", 1), ("active", 2), ("right", 3)] {
        workspace.resolve_activation(
            id,
            attempt,
            ActivationResultKind::Succeeded,
            "",
            Default::default(),
        );
    }
    workspace.set_active("active");
    assert_eq!(
        workspace
            .close("active")
            .and_then(|result| result.activate_document),
        Some("right".into())
    );
    workspace.set_active("right");
    assert_eq!(
        workspace
            .close("right")
            .and_then(|result| result.activate_document),
        Some("left".into())
    );
}

#[test]
fn close_without_ready_neighbor_loads_right_unloaded_before_left() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    // 移除 Welcome，使本场景两侧仅包含未驻留导入文档。
    workspace.close(WELCOME_DOCUMENT_ID);
    add_import(&mut workspace, "left", "left", 1);
    add_import(&mut workspace, "active", "active", 2);
    add_import(&mut workspace, "right", "right", 3);
    for id in ["left", "active", "right"] {
        workspace.mark_unloaded(id);
    }
    workspace.set_active("active");
    assert_eq!(
        workspace
            .close("active")
            .and_then(|result| result.load_document),
        Some("right".into())
    );
    assert_eq!(
        workspace.activation_request("right"),
        ActivationRequest::Ignore
    );
}

#[test]
fn residency_sync_preserves_loading_and_failed_states() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "loading", "loading", 1);
    add_import(&mut workspace, "failed", "failed", 2);
    workspace.open_import("resident", "resident");
    workspace.mark_unloaded("resident");
    workspace.resolve_activation(
        "failed",
        2,
        ActivationResultKind::Failed,
        "Could not load.",
        DocumentDiagnostic {
            code: "asset.failed".into(),
            category: "io".into(),
            detail: "disk".into(),
        },
    );
    assert!(workspace.sync_residency(["loading", "failed", "resident"].iter().copied()));
    let snapshot = workspace.snapshot();
    assert_eq!(snapshot.documents[1].state, DocumentState::Loading);
    assert_eq!(snapshot.documents[2].state, DocumentState::Failed);
    assert_eq!(snapshot.documents[3].state, DocumentState::Ready);
}

#[test]
fn stale_and_replaced_activation_results_cannot_mutate_reopened_tabs() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "same", "part", 1);
    assert_eq!(
        workspace
            .close("same")
            .and_then(|result| result.cancel_attempt),
        Some(1)
    );
    add_import(&mut workspace, "same", "part", 2);
    assert!(!workspace.resolve_activation(
        "same",
        1,
        ActivationResultKind::Succeeded,
        "",
        Default::default()
    ));
    assert!(workspace.resolve_activation(
        "same",
        2,
        ActivationResultKind::Failed,
        "failed",
        Default::default()
    ));
    assert_eq!(
        workspace.snapshot().documents[1].state,
        DocumentState::Failed
    );
}

#[test]
fn draining_multiple_results_updates_each_document_and_uses_last_success_as_active() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "first", "first", 10);
    add_import(&mut workspace, "second", "second", 20);
    for (id, attempt) in [("first", 10), ("second", 20)] {
        assert!(workspace.resolve_activation(
            id,
            attempt,
            ActivationResultKind::Succeeded,
            "",
            DocumentDiagnostic::default()
        ));
    }
    assert_eq!(workspace.active_document_id(), Some("second"));
    assert_eq!(
        workspace.snapshot().documents[1].state,
        DocumentState::Ready
    );
    assert_eq!(
        workspace.snapshot().documents[2].state,
        DocumentState::Ready
    );
}

#[test]
fn current_expired_attempt_leaves_loading_state_and_releases_polling() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    add_import(&mut workspace, "expired", "part", 1);
    assert!(workspace.resolve_activation(
        "expired",
        1,
        ActivationResultKind::Expired,
        "",
        DocumentDiagnostic::default()
    ));
    assert_eq!(
        workspace.snapshot().documents[1].state,
        DocumentState::Unloaded
    );
    assert_eq!(
        workspace.activation_request("expired"),
        ActivationRequest::LoadImport("expired".into())
    );
}

#[test]
fn duplicate_titles_receive_stable_ordinals_after_reordering() {
    let mut workspace = DocumentWorkspace::new("Welcome");
    workspace.open_import("first", "part");
    workspace.open_import("second", "part");
    let initial = workspace.snapshot();
    assert_eq!(
        (
            initial.documents[1].display_ordinal,
            initial.documents[1].display_count
        ),
        (1, 2)
    );
    assert!(workspace.move_document(1, 2));
    let reordered = workspace.snapshot();
    assert_eq!(reordered.documents[2].display_ordinal, 2);
}

#[test]
fn project_error_codes_map_to_stable_translation_sources() {
    let cases = [
        ("project.empty_name", "Enter a valid project name."),
        ("project.invalid_name", "Enter a valid project name."),
        (
            "project.location_empty",
            "Choose an absolute project location.",
        ),
        (
            "project.location_not_absolute",
            "Choose an absolute project location.",
        ),
        (
            "project.location_create_failed",
            "The project location could not be created.",
        ),
        (
            "project.file_missing",
            "The selected project file does not exist.",
        ),
        ("project.invalid_file", "Choose a .panta project file."),
        (
            "project.file_not_local",
            "Choose a project file on this device.",
        ),
        (
            "project.already_exists",
            "A project with this name already exists.",
        ),
        (
            "project.manifest_invalid",
            "The selected project file is not supported.",
        ),
        (
            "project.unsupported_schema",
            "The selected project file is not supported.",
        ),
        ("project.no_project", "Open or create a project first."),
        (
            "project.command_invalid",
            "The project change is not valid.",
        ),
        (
            "project.import_file_missing",
            "The selected STL file does not exist.",
        ),
        (
            "project.import_invalid_file",
            "Choose an STL file to import.",
        ),
        (
            "project.import_unsupported_mesh_type",
            "Choose valid STL import options.",
        ),
        (
            "project.import_unsupported_units",
            "Choose valid STL import options.",
        ),
        (
            "project.import_source_changed",
            "The STL file changed after preview. Select it again.",
        ),
        (
            "project.import_parse_failed",
            "The selected STL file could not be read.",
        ),
        (
            "project.import_asset_write_failed",
            "The mesh asset could not be saved into the project.",
        ),
        ("unknown", "The project operation could not be completed."),
    ];
    for (code, expected) in cases {
        assert_eq!(project_error_message_key(code), expected, "{code}");
    }
    let activation_cases = [
        (
            "project.asset_missing",
            "The saved mesh asset is missing from the project package.",
        ),
        (
            "project.asset_unsupported_format",
            "The saved mesh asset format is not supported.",
        ),
        (
            "project.import_unsupported_units",
            "The saved mesh asset uses unsupported units.",
        ),
        (
            "project.asset_read_failed",
            "The saved mesh asset could not be read.",
        ),
        (
            "project.asset_parse_failed",
            "The saved mesh asset could not be read.",
        ),
        (
            "project.asset_source_changed",
            "The saved mesh asset could not be opened.",
        ),
        (
            "project.activation_spawn_failed",
            "The saved mesh asset could not be opened.",
        ),
        (
            "project.activation_worker_failed",
            "The saved mesh asset could not be opened.",
        ),
        ("unknown", "The saved mesh asset could not be opened."),
    ];
    for (code, expected) in activation_cases {
        assert_eq!(activation_error_message_key(code), expected, "{code}");
    }
}

#[test]
fn dimension_format_matches_existing_precision_and_optional_unit() {
    assert_eq!(
        format_dimensions(1.234, 0.0, 99.999, ""),
        "1.23 × 0.00 × 100.00"
    );
    assert_eq!(
        format_dimensions(1.0, 2.0, 3.0, "mm"),
        "1.00 × 2.00 × 3.00 mm"
    );
}
