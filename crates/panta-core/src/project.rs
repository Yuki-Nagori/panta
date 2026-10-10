//! 工程模块门面：领域数据、诊断与应用服务（057、063、107）。
//!
//! 工程目录由宿主选择，工程模型和清单事务由 Rust 持有。Qt/QML 只通过
//! CXX adapter 传递 UTF-8 路径、名称和命令，不直接读写清单文件。

use panta_import::StlImportSession;
mod error;
mod mesh_cache;
mod model;
mod repository;
mod service;
pub use error::ProjectError;
use mesh_cache::SurfaceMeshCache;
pub use model::{ProjectCommand, ProjectSnapshot};
use model::{ProjectManifest, ProjectState};
pub use panta_import::{
    DEFAULT_MESH_TYPE, IMPORT_RECORD_VERSION, ImportRecord, MESH_TYPES, STL_IMPORT_PARSER_VERSION,
    StlImportPreview,
};
pub use service::ProjectService;

mod analysis_sequence;
pub use analysis_sequence::{ANALYSIS_SEQUENCES, AnalysisSequenceDefinition};
mod plan_settings;
pub use plan_settings::PlanSettings;
mod process_settings;
pub use process_settings::{FillSettings, HoldingProfilePoint};
mod gate_location;
pub use gate_location::{GateLocationSettings, GateLocatorAlgorithm};
mod material;
pub use material::{MaterialDefinition, MaterialProperty, default_material};
mod import;
mod preview;
mod storage;

pub use crate::fsm::open_saved_stl::{ActivationAttempt, Outcome, OutcomeKind};

/// 当前范例工程清单的 schema 版本。
pub const PROJECT_SCHEMA_VERSION: u32 = 2;
const MIN_SUPPORTED_PROJECT_SCHEMA_VERSION: u32 = 1;
const PROJECT_FILE_EXTENSION: &str = "panta";

#[cfg(test)]
#[path = "../../../tests/support/rust/temp_directory.rs"]
mod temp_directory;

#[cfg(test)]
mod tests {
    use super::model::validate_name;
    use super::{ProjectError, ProjectService, SurfaceMeshCache};
    use panta_mesh::SurfaceMesh;

    fn mesh(triangle_count: usize) -> SurfaceMesh {
        SurfaceMesh {
            triangles: vec![[[0.0; 3]; 3]; triangle_count],
        }
    }

    #[test]
    fn mesh_cache_evicts_the_least_recently_used_inactive_asset() {
        let triangle_bytes = std::mem::size_of::<[[f64; 3]; 3]>();
        let mut service = ProjectService::with_mesh_cache_budget(triangle_bytes * 2);
        service.mesh_cache.insert_active("a".to_owned(), mesh(1));
        service.mesh_cache.insert_active("b".to_owned(), mesh(1));
        assert!(service.mesh_cache.activate("a"));
        service.mesh_cache.insert_active("c".to_owned(), mesh(1));

        let mut residents = service.mesh_cache.resident_ids();
        residents.sort();
        assert_eq!(residents, ["a", "c"]);
        assert_eq!(service.mesh_cache.retained_bytes(), triangle_bytes * 2);
    }

    #[test]
    fn mesh_cache_keeps_one_active_mesh_even_when_it_exceeds_budget() {
        let mut cache = SurfaceMeshCache::with_budget(0);
        cache.insert_active("large".to_owned(), mesh(2));

        assert_eq!(cache.resident_ids(), ["large"]);
        assert_eq!(
            cache.retained_bytes(),
            std::mem::size_of::<[[f64; 3]; 3]>() * 2
        );
        cache.deactivate();
        assert!(cache.resident_ids().is_empty());
    }

    #[test]
    fn activating_resident_mesh_updates_the_service_current_mesh_view() {
        let mut service = ProjectService::new();
        service.cache_activated_mesh("a".to_owned(), mesh(1));
        service.cache_activated_mesh("b".to_owned(), mesh(2));

        assert!(service.activate_mesh_document("a"));
        assert_eq!(
            service.current_mesh().map(|mesh| mesh.triangles.len()),
            Some(1)
        );
    }

    #[test]
    fn activating_missing_mesh_keeps_the_current_mesh_unchanged() {
        let mut service = ProjectService::new();
        service.cache_activated_mesh("resident".to_owned(), mesh(1));

        assert!(!service.activate_mesh_document("missing"));
        assert_eq!(
            service.current_mesh().map(|mesh| mesh.triangles.len()),
            Some(1)
        );
        assert_eq!(service.resident_mesh_ids(), ["resident"]);
    }

    #[test]
    fn closing_mesh_document_releases_runtime_data_without_touching_imports() {
        let mut service = ProjectService::new();
        service
            .mesh_cache
            .insert_active("import-1".to_owned(), mesh(1));
        service.latest_mesh_id = Some("import-1".to_owned());
        service.activation_attempts.insert("import-1".to_owned(), 7);

        service.release_mesh_document("import-1");

        assert!(service.mesh_for_import("import-1").is_none());
        assert!(service.current_mesh().is_none());
        assert!(service.resident_mesh_ids().is_empty());
        assert!(!service.is_current_activation_attempt("import-1", 7));
    }

    #[test]
    fn superseded_activation_cannot_replace_or_finish_the_current_mesh() {
        let mut service = ProjectService::new();
        service.activation_attempts.insert("import-1".to_owned(), 2);

        assert!(!service.cache_activation_result("import-1", 1, mesh(1)));
        service.finish_asset_activation("import-1", 1);
        assert!(service.mesh_for_import("import-1").is_none());
        assert!(service.is_current_activation_attempt("import-1", 2));

        assert!(service.cache_activation_result("import-1", 2, mesh(2)));
        assert_eq!(
            service
                .mesh_for_import("import-1")
                .map(|mesh| mesh.triangles.len()),
            Some(2)
        );
        assert!(!service.is_current_activation_attempt("import-1", 2));
    }

    #[test]
    fn project_name_rules_accept_unicode_and_reject_platform_reserved_stems() {
        assert!(validate_name("模拟件.v1").is_ok());
        assert!(matches!(
            validate_name("COM1.log"),
            Err(ProjectError::InvalidName(_))
        ));
        assert!(matches!(
            validate_name("name. "),
            Err(ProjectError::InvalidName(_))
        ));
    }
}
