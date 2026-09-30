//! 当前零件方案快照及配置命令的目标校验。
use super::analysis_sequence::{DEFAULT_SEQUENCE_ID, definition};
use super::*;

/// 当前任务面板关联零件的轻量快照；不读取或复制网格载荷。
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSettings {
    pub project_path: PathBuf,
    pub revision: u64,
    pub import_id: String,
    pub mesh_type: String,
    pub sequence_id: String,
    pub sequence_source_text: String,
    pub material_id: String,
    pub material_source_text: String,
    pub fill_settings: FillSettings,
    pub fill_settings_confirmed: bool,
}

impl ProjectService {
    /// 活动导入记录优先；Welcome / 空页签时展示最近导入记录。
    pub fn plan_settings(&self, preferred_import_id: &str) -> Option<PlanSettings> {
        let state = self.current.as_ref()?;
        let record = state
            .imports
            .iter()
            .find(|record| record.id == preferred_import_id)
            .or_else(|| state.imports.last())?;
        let sequence_id = state
            .analysis_sequences
            .get(&record.id)
            .map_or(DEFAULT_SEQUENCE_ID, String::as_str);
        let sequence = definition(sequence_id)?;
        Some(PlanSettings {
            project_path: state.path.clone(),
            revision: state.revision,
            import_id: record.id.clone(),
            mesh_type: record.mesh_type.clone(),
            sequence_id: sequence.id.to_owned(),
            sequence_source_text: sequence.source_text.to_owned(),
            fill_settings: state
                .fill_settings
                .get(&record.id)
                .cloned()
                .unwrap_or_default(),
            fill_settings_confirmed: state.fill_settings.contains_key(&record.id),
            material_id: state.materials.get(&record.id).cloned().unwrap_or_default(),
            material_source_text: if state.materials.contains_key(&record.id) {
                default_material().ok()?.source_text.clone()
            } else {
                String::new()
            },
        })
    }

    pub(super) fn checked_plan_target(
        &self,
        expected_path: &Path,
        expected_revision: u64,
        import_id: &str,
    ) -> Result<&ProjectState, ProjectError> {
        self.ensure_project_writable()?;
        let state = self.current.as_ref().ok_or(ProjectError::NoProject)?;
        if state.path != expected_path || state.revision != expected_revision {
            return Err(ProjectError::CommandInvalid(
                "plan settings changed".to_owned(),
            ));
        }
        if !state.imports.iter().any(|record| record.id == import_id) {
            return Err(ProjectError::ImportRecordMissing(import_id.to_owned()));
        }
        Ok(state)
    }
}
