//! 工程模块门面：领域数据、诊断与应用服务（057、063、107）。
//!
//! 工程目录由宿主选择，工程模型和清单事务由 Rust 持有。Qt/QML 只通过
//! CXX adapter 传递 UTF-8 路径、名称和命令，不直接读写清单文件。

use panta_import::StlImportSession;
mod activation_result;
mod error;
pub use activation_result::CompletedActivation;
mod mesh_cache;
mod model;
mod repository;
mod service;
mod variable_document;
mod workspace;
pub use error::ProjectError;
use mesh_cache::SurfaceMeshCache;
pub use model::{ProjectCommand, ProjectSnapshot};
use model::{ProjectManifest, ProjectState};
pub use panta_import::{
    DEFAULT_MESH_TYPE, IMPORT_RECORD_VERSION, ImportRecord, MESH_TYPES, STL_IMPORT_PARSER_VERSION,
    StlImportPreview, default_display_name,
};
pub(crate) use service::resolve_asset_path;
pub use service::{ProjectService, VariableDocumentError};
pub use variable_document::{VariableDocumentCompletion, VariableDocumentRequest};
pub use workspace::{
    ActivationRequest as WorkspaceActivationRequest, ActivationResultKind, CloseDocumentResult,
    DocumentDiagnostic, DocumentKind, DocumentState, DocumentWorkspace, WorkspaceDocument,
    WorkspaceSnapshot, activation_error_message_key,
    format_dimensions as format_project_dimensions, project_error_message_key,
};

mod command_job;
pub use command_job::{ProjectOperation, ProjectOperationCompletion, ProjectOperationRequest};

mod analysis_sequence;
pub use analysis_sequence::{ANALYSIS_SEQUENCES, AnalysisSequenceDefinition};
mod plan_settings;
pub use plan_settings::PlanSettings;
mod process_settings;
pub use process_settings::{
    FLOW_RATE_CONSTRAINT, FillSettings, HOLDING_DURATION_CONSTRAINT, HOLDING_PRESSURE_CONSTRAINT,
    HOLDING_PROFILE_MAX_POINTS, HoldingProfilePoint, NumericConstraint,
    SWITCH_OVER_VOLUME_CONSTRAINT, TEMPERATURE_CONSTRAINT,
};
mod gate_location;
pub use gate_location::{GateLocationSettings, GateLocatorAlgorithm, NUMBER_OF_GATES_CONSTRAINT};
mod material;
pub use material::{MaterialDefinition, MaterialProperty, default_material};
mod import;
mod mesh_assets;
mod preview;
mod storage;

pub use crate::fsm::open_saved_stl::{ActivationAttempt, Outcome, OutcomeKind};

/// 当前范例工程清单的 schema 版本。
pub const PROJECT_SCHEMA_VERSION: u32 = 4;
const PROJECT_FILE_EXTENSION: &str = "panta";

#[cfg(test)]
#[path = "../../../tests/support/rust/temp_directory.rs"]
pub(crate) mod temp_directory;

#[cfg(test)]
mod tests;
