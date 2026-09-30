//! 方案分析序列的目录、显示快照和持久化命令。
use super::*;

/// 稳定领域 ID 与英文源文案；翻译由 Qt 展示层处理。
#[derive(Debug, Clone, Copy)]
pub struct AnalysisSequenceDefinition {
    pub id: &'static str,
    pub source_text: &'static str,
}

/// 当前序列目录；执行阶段与适用性规则由实际求解消费者扩展。
pub const ANALYSIS_SEQUENCES: &[AnalysisSequenceDefinition] = &[
    AnalysisSequenceDefinition {
        id: "fill",
        source_text: "Fill",
    },
    AnalysisSequenceDefinition {
        id: "fill-pack",
        source_text: "Fill + Pack",
    },
    AnalysisSequenceDefinition {
        id: "fast-fill",
        source_text: "Fast Fill",
    },
    AnalysisSequenceDefinition {
        id: "fill-pack-warp",
        source_text: "Fill + Pack + Warp",
    },
    AnalysisSequenceDefinition {
        id: "cool",
        source_text: "Cool",
    },
    AnalysisSequenceDefinition {
        id: "cool-fill-pack-warp",
        source_text: "Cool + Fill + Pack + Warp",
    },
    AnalysisSequenceDefinition {
        id: "molding-window",
        source_text: "Molding Window",
    },
    AnalysisSequenceDefinition {
        id: "gate-location",
        source_text: "Gate Location",
    },
    AnalysisSequenceDefinition {
        id: "cool-fem",
        source_text: "Cool (FEM)",
    },
    AnalysisSequenceDefinition {
        id: "cool-fem-fill-pack-warp",
        source_text: "Cool (FEM) + Fill + Pack + Warp",
    },
];
const DEFAULT_SEQUENCE_ID: &str = "fill";

/// 当前任务面板关联零件的轻量快照；不读取或复制网格载荷。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSettings {
    pub project_path: PathBuf,
    pub revision: u64,
    pub import_id: String,
    pub mesh_type: String,
    pub sequence_id: String,
    pub sequence_source_text: String,
}

fn definition(id: &str) -> Option<&'static AnalysisSequenceDefinition> {
    ANALYSIS_SEQUENCES.iter().find(|entry| entry.id == id)
}

pub(super) fn validate_sequences(manifest: &ProjectManifest) -> Result<(), ProjectError> {
    for (id, sequence) in &manifest.analysis_sequences {
        if !manifest.imports.iter().any(|record| record.id == *id) || definition(sequence).is_none()
        {
            return Err(ProjectError::ManifestInvalid(format!(
                "invalid analysis sequence for {id}"
            )));
        }
    }
    Ok(())
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
        })
    }

    /// 校验弹窗打开时的工程身份和修订；先写候选清单，成功后发布内存状态。
    pub fn set_analysis_sequence(
        &mut self,
        expected_path: &Path,
        expected_revision: u64,
        import_id: &str,
        sequence_id: &str,
    ) -> Result<ProjectSnapshot, ProjectError> {
        let state = self.current.as_ref().ok_or(ProjectError::NoProject)?;
        if state.path != expected_path || state.revision != expected_revision {
            return Err(ProjectError::CommandInvalid(
                "plan settings changed".to_owned(),
            ));
        }
        if !state.imports.iter().any(|record| record.id == import_id) {
            return Err(ProjectError::ImportRecordMissing(import_id.to_owned()));
        }
        if definition(sequence_id).is_none() {
            return Err(ProjectError::CommandInvalid(
                "unknown analysis sequence".to_owned(),
            ));
        }
        let current = state
            .analysis_sequences
            .get(import_id)
            .map_or(DEFAULT_SEQUENCE_ID, String::as_str);
        if current == sequence_id {
            return self.snapshot();
        }
        let mut candidate = state.clone();
        if sequence_id == DEFAULT_SEQUENCE_ID {
            candidate.analysis_sequences.remove(import_id);
        } else {
            candidate
                .analysis_sequences
                .insert(import_id.to_owned(), sequence_id.to_owned());
        }
        candidate.revision = candidate.revision.saturating_add(1);
        candidate.dirty = false;
        write_manifest(&candidate)?;
        self.current = Some(candidate);
        self.snapshot()
    }
}
