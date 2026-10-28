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
        "project.import_asset_write_failed" => {
            "The mesh asset could not be saved into the project."
        }
        _ => "The project operation could not be completed.",
    }
}

pub fn activation_error_message_key(code: &str) -> &'static str {
    match code {
        "project.asset_missing" => "The saved mesh asset is missing from the project package.",
        "project.asset_unsupported_format" => "The saved mesh asset format is not supported.",
        "project.import_unsupported_units" => "The saved mesh asset uses unsupported units.",
        "project.asset_read_failed" | "project.asset_parse_failed" => {
            "The saved mesh asset could not be read."
        }
        _ => "The saved mesh asset could not be opened.",
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
mod tests;
