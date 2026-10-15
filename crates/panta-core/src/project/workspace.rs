//! 工程窗口中的运行期文档页签状态；不参与工程文件持久化。

use std::collections::HashMap;

const WELCOME_DOCUMENT_ID: &str = "welcome";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Welcome,
    Import,
}

impl DocumentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::Import => "import",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    Ready,
    Loading,
    Failed,
    Unloaded,
}

impl DocumentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Loading => "loading",
            Self::Failed => "failed",
            Self::Unloaded => "unloaded",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentDiagnostic {
    pub code: String,
    pub category: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDocument {
    pub id: String,
    pub kind: DocumentKind,
    pub state: DocumentState,
    pub title: String,
    /// Qt 翻译目录中的英文源文案；由 Qt 按当前 locale 翻译。
    pub message_key: String,
    pub diagnostic: DocumentDiagnostic,
    pub display_ordinal: u64,
    pub display_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub documents: Vec<WorkspaceDocument>,
    pub active_document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationRequest {
    Ignore,
    ActivateWelcome,
    ActivateImport(String),
    LoadImport(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseDocumentResult {
    pub cancel_attempt: Option<u64>,
    pub release_import: bool,
    pub active_changed: bool,
    pub activate_document: Option<String>,
    pub load_document: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationResultKind {
    Succeeded,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Default)]
pub struct DocumentWorkspace {
    documents: Vec<WorkspaceDocument>,
    active_document_id: Option<String>,
    activation_attempts: HashMap<String, u64>,
}

impl DocumentWorkspace {
    pub fn new(welcome_title: &str) -> Self {
        let mut workspace = Self::default();
        workspace.reset(welcome_title);
        workspace
    }

    /// 重置运行期页签；返回需由调用方请求取消的后台 attempt。
    pub fn reset(&mut self, welcome_title: &str) -> Vec<u64> {
        let attempts = self
            .activation_attempts
            .drain()
            .map(|(_, attempt)| attempt)
            .collect();
        self.documents.clear();
        self.documents.push(WorkspaceDocument {
            id: WELCOME_DOCUMENT_ID.to_owned(),
            kind: DocumentKind::Welcome,
            state: DocumentState::Ready,
            title: welcome_title.to_owned(),
            message_key: String::new(),
            diagnostic: DocumentDiagnostic::default(),
            display_ordinal: 1,
            display_count: 1,
        });
        self.active_document_id = Some(WELCOME_DOCUMENT_ID.to_owned());
        attempts
    }

    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let mut documents = self.documents.clone();
        let mut title_counts: HashMap<String, u64> = HashMap::new();
        for document in &documents {
            if document.kind == DocumentKind::Import {
                *title_counts.entry(document.title.clone()).or_default() += 1;
            }
        }
        let mut title_ordinals: HashMap<String, u64> = HashMap::new();
        for document in &mut documents {
            if document.kind == DocumentKind::Import {
                let ordinal = title_ordinals.entry(document.title.clone()).or_default();
                *ordinal += 1;
                document.display_ordinal = *ordinal;
                document.display_count = title_counts[&document.title];
            }
        }
        WorkspaceSnapshot {
            documents,
            active_document_id: self.active_document_id.clone().unwrap_or_default(),
        }
    }

    pub fn open_import(&mut self, id: &str, title: &str) -> bool {
        if self.document_index(id).is_some() {
            return false;
        }
        self.documents.push(WorkspaceDocument {
            id: id.to_owned(),
            kind: DocumentKind::Import,
            state: DocumentState::Unloaded,
            title: title.to_owned(),
            message_key: String::new(),
            diagnostic: DocumentDiagnostic::default(),
            display_ordinal: 1,
            display_count: 1,
        });
        true
    }

    pub fn open_ready_import(&mut self, id: &str, title: &str) -> bool {
        if !self.open_import(id, title) {
            return false;
        }
        if let Some(document) = self.document_mut(id) {
            document.state = DocumentState::Ready;
        }
        self.active_document_id = Some(id.to_owned());
        true
    }

    pub fn fail_document(
        &mut self,
        id: &str,
        message_key: &str,
        diagnostic: DocumentDiagnostic,
    ) -> bool {
        let Some(document) = self.document_mut(id) else {
            return false;
        };
        document.state = DocumentState::Failed;
        document.message_key = message_key.to_owned();
        document.diagnostic = diagnostic;
        true
    }

    pub fn activation_request(&mut self, id: &str) -> ActivationRequest {
        let Some(index) = self.document_index(id) else {
            return ActivationRequest::Ignore;
        };
        let document = &mut self.documents[index];
        match document.state {
            DocumentState::Loading => ActivationRequest::Ignore,
            DocumentState::Ready => match document.kind {
                DocumentKind::Welcome => {
                    self.active_document_id = Some(document.id.clone());
                    ActivationRequest::ActivateWelcome
                }
                DocumentKind::Import => ActivationRequest::ActivateImport(document.id.clone()),
            },
            DocumentState::Failed | DocumentState::Unloaded => {
                document.state = DocumentState::Loading;
                document.message_key.clear();
                document.diagnostic = DocumentDiagnostic::default();
                ActivationRequest::LoadImport(document.id.clone())
            }
        }
    }

    pub fn mark_unloaded(&mut self, id: &str) {
        if let Some(document) = self.document_mut(id)
            && document.kind == DocumentKind::Import
        {
            document.state = DocumentState::Unloaded;
            document.message_key.clear();
            document.diagnostic = DocumentDiagnostic::default();
        }
    }

    pub fn set_active(&mut self, id: &str) -> bool {
        if self.document_index(id).is_none() {
            return false;
        }
        self.active_document_id = Some(id.to_owned());
        true
    }

    pub fn active_document_id(&self) -> Option<&str> {
        self.active_document_id.as_deref()
    }

    pub fn set_activation_attempt(&mut self, id: &str, attempt: u64) -> bool {
        if self.document_index(id).is_none() {
            return false;
        }
        self.activation_attempts.insert(id.to_owned(), attempt);
        true
    }

    pub fn activation_attempt(&self, id: &str) -> Option<u64> {
        self.activation_attempts.get(id).copied()
    }

    pub fn resolve_activation(
        &mut self,
        id: &str,
        attempt: u64,
        result: ActivationResultKind,
        message_key: &str,
        diagnostic: DocumentDiagnostic,
    ) -> bool {
        if self.activation_attempt(id) != Some(attempt) {
            return false;
        }
        self.activation_attempts.remove(id);
        let Some(document) = self.document_mut(id) else {
            return false;
        };
        match result {
            ActivationResultKind::Succeeded => {
                document.state = DocumentState::Ready;
                document.message_key.clear();
                document.diagnostic = DocumentDiagnostic::default();
                self.active_document_id = Some(id.to_owned());
            }
            ActivationResultKind::Failed => {
                document.state = DocumentState::Failed;
                document.message_key = message_key.to_owned();
                document.diagnostic = diagnostic;
            }
            ActivationResultKind::Cancelled => {
                document.state = DocumentState::Unloaded;
                document.message_key.clear();
                document.diagnostic = DocumentDiagnostic::default();
            }
            ActivationResultKind::Expired => {
                document.state = DocumentState::Unloaded;
                document.message_key.clear();
                document.diagnostic = DocumentDiagnostic::default();
            }
        }
        true
    }

    pub fn sync_residency<'a>(&mut self, resident_ids: impl IntoIterator<Item = &'a str>) -> bool {
        let residents: std::collections::HashSet<&str> = resident_ids.into_iter().collect();
        let mut changed = false;
        for document in &mut self.documents {
            if document.kind != DocumentKind::Import
                || matches!(
                    document.state,
                    DocumentState::Loading | DocumentState::Failed
                )
            {
                continue;
            }
            let next = if residents.contains(document.id.as_str()) {
                DocumentState::Ready
            } else {
                DocumentState::Unloaded
            };
            if document.state != next {
                document.state = next;
                document.message_key.clear();
                document.diagnostic = DocumentDiagnostic::default();
                changed = true;
            }
        }
        changed
    }

    pub fn close(&mut self, id: &str) -> Option<CloseDocumentResult> {
        let index = self.document_index(id)?;
        let document = self.documents.remove(index);
        let cancel_attempt = self.activation_attempts.remove(id);
        let active_changed = self.active_document_id.as_deref() == Some(id);
        let mut result = CloseDocumentResult {
            cancel_attempt,
            release_import: document.kind == DocumentKind::Import,
            active_changed,
            activate_document: None,
            load_document: None,
        };
        if !active_changed {
            return Some(result);
        }

        self.active_document_id = None;
        if let Some(next) = self.neighbor(index, DocumentState::Ready) {
            self.active_document_id = Some(next.clone());
            result.activate_document = Some(next);
            return Some(result);
        }
        if let Some(next) = self.neighbor(index, DocumentState::Unloaded) {
            if let Some(next_document) = self.document_mut(&next) {
                next_document.state = DocumentState::Loading;
                next_document.message_key.clear();
                next_document.diagnostic = DocumentDiagnostic::default();
            }
            result.load_document = Some(next);
        }
        Some(result)
    }

    pub fn move_document(&mut self, from: usize, to: usize) -> bool {
        if from >= self.documents.len() || to >= self.documents.len() || from == to {
            return false;
        }
        let document = self.documents.remove(from);
        self.documents.insert(to, document);
        true
    }

    pub fn document_index(&self, id: &str) -> Option<usize> {
        self.documents.iter().position(|document| document.id == id)
    }

    fn document_mut(&mut self, id: &str) -> Option<&mut WorkspaceDocument> {
        self.documents.iter_mut().find(|document| document.id == id)
    }

    fn neighbor(&self, removed_index: usize, state: DocumentState) -> Option<String> {
        self.documents
            .iter()
            .skip(removed_index)
            .find(|document| document.state == state)
            .or_else(|| {
                self.documents
                    .iter()
                    .take(removed_index)
                    .rev()
                    .find(|document| document.state == state)
            })
            .map(|document| document.id.clone())
    }
}

/// 错误码映射为 Qt 翻译目录中的稳定英文源文案。
pub fn project_error_message_key(code: &str) -> &'static str {
    match code {
        "project.empty_name" | "project.invalid_name" => "Enter a valid project name.",
        "project.location_empty" | "project.location_not_absolute" => {
            "Choose an absolute project location."
        }
        "project.location_create_failed" => "The project location could not be created.",
        "project.file_missing" => "The selected project file does not exist.",
        "project.invalid_file" => "Choose a .panta project file.",
        "project.file_not_local" => "Choose a project file on this device.",
        "project.already_exists" => "A project with this name already exists.",
        "project.manifest_invalid" | "project.unsupported_schema" => {
            "The selected project file is not supported."
        }
        "project.no_project" => "Open or create a project first.",
        "project.command_invalid" => "The project change is not valid.",
        "project.import_file_missing" => "The selected STL file does not exist.",
        "project.import_invalid_file" => "Choose an STL file to import.",
        "project.import_unsupported_mesh_type" | "project.import_unsupported_units" => {
            "Choose valid STL import options."
        }
        "project.import_source_changed" => "The STL file changed after preview. Select it again.",
        "project.import_parse_failed" => "The selected STL file could not be read.",
        "project.import_asset_copy_failed" => "The STL file could not be copied into the project.",
        _ => "The project operation could not be completed.",
    }
}

pub fn activation_error_message_key(code: &str) -> &'static str {
    match code {
        "project.asset_missing" => "The saved STL asset is missing from the project package.",
        "project.asset_unsupported_format" => "The saved STL asset format is not supported.",
        "project.import_unsupported_units" => "The saved STL asset uses unsupported units.",
        "project.asset_read_failed" | "project.asset_parse_failed" => {
            "The saved STL asset could not be read."
        }
        _ => "The saved STL asset could not be opened.",
    }
}

/// 工程 UI 中使用的三边尺寸字符串；数值单位为调用方约定的显示单位。
pub fn format_dimensions(size_x: f64, size_y: f64, size_z: f64, unit: &str) -> String {
    let values = format!("{size_x:.2} × {size_y:.2} × {size_z:.2}");
    if unit.is_empty() {
        values
    } else {
        format!("{values} {unit}")
    }
}

#[cfg(test)]
mod tests {
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
                "project.import_asset_copy_failed",
                "The STL file could not be copied into the project.",
            ),
            ("unknown", "The project operation could not be completed."),
        ];
        for (code, expected) in cases {
            assert_eq!(project_error_message_key(code), expected, "{code}");
        }
        let activation_cases = [
            (
                "project.asset_missing",
                "The saved STL asset is missing from the project package.",
            ),
            (
                "project.asset_unsupported_format",
                "The saved STL asset format is not supported.",
            ),
            (
                "project.import_unsupported_units",
                "The saved STL asset uses unsupported units.",
            ),
            (
                "project.asset_read_failed",
                "The saved STL asset could not be read.",
            ),
            (
                "project.asset_parse_failed",
                "The saved STL asset could not be read.",
            ),
            (
                "project.asset_source_changed",
                "The saved STL asset could not be opened.",
            ),
            (
                "project.activation_spawn_failed",
                "The saved STL asset could not be opened.",
            ),
            (
                "project.activation_worker_failed",
                "The saved STL asset could not be opened.",
            ),
            ("unknown", "The saved STL asset could not be opened."),
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
}
