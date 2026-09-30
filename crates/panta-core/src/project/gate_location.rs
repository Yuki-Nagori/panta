//! 浇口定位工艺配置；目录标识、默认值和校验由领域持有。
use super::*;

/// 工程清单中的算法标识；执行引擎另行接入。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateLocatorAlgorithm {
    AdvancedGateLocator,
}

impl GateLocatorAlgorithm {
    pub fn id(self) -> &'static str {
        "advanced-gate-locator"
    }
    pub fn source_text(self) -> &'static str {
        "Advanced gate locator"
    }
    pub fn from_id(id: &str) -> Result<Self, ProjectError> {
        match id {
            "advanced-gate-locator" => Ok(Self::AdvancedGateLocator),
            _ => Err(ProjectError::CommandInvalid(
                "unknown gate locator algorithm".to_owned(),
            )),
        }
    }
}

/// 温度单位为摄氏度；设备引用当前只支持默认设备。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateLocationSettings {
    pub machine_id: String,
    pub mold_temperature_celsius: f64,
    pub melt_temperature_celsius: f64,
    pub algorithm: GateLocatorAlgorithm,
    pub number_of_gates: u32,
}

impl Default for GateLocationSettings {
    fn default() -> Self {
        Self {
            machine_id: "default-machine".to_owned(),
            mold_temperature_celsius: 40.0,
            melt_temperature_celsius: 230.0,
            algorithm: GateLocatorAlgorithm::AdvancedGateLocator,
            number_of_gates: 1,
        }
    }
}

impl GateLocationSettings {
    pub fn machine_source_text(&self) -> &'static str {
        "Default machine"
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if self.machine_id != "default-machine"
            || ![self.mold_temperature_celsius, self.melt_temperature_celsius]
                .iter()
                .all(|value| value.is_finite() && *value >= -273.15)
            || !(1..=10).contains(&self.number_of_gates)
        {
            return Err(ProjectError::CommandInvalid(
                "invalid gate location settings".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(super) fn validate_settings(
    imports: &[ImportRecord],
    settings: &BTreeMap<String, GateLocationSettings>,
) -> Result<(), ProjectError> {
    for (id, setting) in settings {
        if !imports.iter().any(|record| record.id == *id) {
            return Err(ProjectError::ManifestInvalid(format!(
                "gate location settings reference missing import {id}"
            )));
        }
        setting
            .validate()
            .map_err(|error| ProjectError::ManifestInvalid(error.to_string()))?;
    }
    Ok(())
}

impl ProjectService {
    /// 冻结工程目标，后台校验并写盘；相同已确认配置无需再次写入。
    pub fn begin_gate_location_settings_confirmation(
        &mut self,
        expected_path: &Path,
        expected_revision: u64,
        import_id: &str,
        settings: GateLocationSettings,
    ) -> Result<bool, ProjectError> {
        let state = self.checked_plan_target(expected_path, expected_revision, import_id)?;
        if state
            .analysis_sequences
            .get(import_id)
            .map_or(analysis_sequence::DEFAULT_SEQUENCE_ID, String::as_str)
            != "gate-location"
        {
            return Err(ProjectError::CommandInvalid(
                "gate location settings require Gate Location analysis sequence".to_owned(),
            ));
        }
        if state.gate_location_settings.get(import_id) == Some(&settings) {
            return Ok(false);
        }
        let mut candidate = state.clone();
        candidate
            .gate_location_settings
            .insert(import_id.to_owned(), settings);
        candidate.revision = candidate.revision.saturating_add(1);
        candidate.dirty = false;
        self.start_metadata_write(candidate, storage::MetadataWriteKind::GateLocationSettings)
    }

    /// 非阻塞消费结果；失败解除写入锁且保留旧工程配置。
    pub fn finish_gate_location_settings_confirmation(&mut self) -> Result<bool, ProjectError> {
        self.finish_metadata_write(storage::MetadataWriteKind::GateLocationSettings)
    }
}
