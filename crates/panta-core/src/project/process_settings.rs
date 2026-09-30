//! Fill 工艺配置；默认值和校验由 Rust 持有，确认写盘复用工程后台事务。
use super::*;

/// 分段持续时间（秒）和填充压力百分比；时间累计用于曲线横轴。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldingProfilePoint {
    pub duration_seconds: f64,
    pub pressure_percent: f64,
}

/// Fill 的已支持控制方式：流量填充、体积百分比切换和相对压力保压。
/// 字段名携带单位，界面无需推断工程清单里的量纲。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FillSettings {
    pub mold_temperature_celsius: f64,
    pub melt_temperature_celsius: f64,
    pub flow_rate_cm3_per_second: f64,
    pub switch_over_volume_percent: f64,
    pub fiber_orientation: bool,
    pub crystallization: bool,
    pub holding_profile: Vec<HoldingProfilePoint>,
}

impl Default for FillSettings {
    fn default() -> Self {
        Self {
            mold_temperature_celsius: 40.0,
            melt_temperature_celsius: 230.0,
            flow_rate_cm3_per_second: 94.7,
            switch_over_volume_percent: 99.0,
            fiber_orientation: true,
            crystallization: false,
            holding_profile: vec![
                HoldingProfilePoint {
                    duration_seconds: 0.0,
                    pressure_percent: 100.0,
                },
                HoldingProfilePoint {
                    duration_seconds: 10.0,
                    pressure_percent: 100.0,
                },
            ],
        }
    }
}

impl FillSettings {
    fn validate(&self) -> Result<(), ProjectError> {
        let temperature_valid = |value: f64| value.is_finite() && value >= -273.15;
        let percentage_valid =
            |value: f64, max: f64| value.is_finite() && (0.0..=max).contains(&value);
        // 曲线有明确上限，避免元数据和 UI 快照随输入无限增长。
        if !temperature_valid(self.mold_temperature_celsius)
            || !temperature_valid(self.melt_temperature_celsius)
            || !self.flow_rate_cm3_per_second.is_finite()
            || self.flow_rate_cm3_per_second <= 0.0
            || !percentage_valid(self.switch_over_volume_percent, 100.0)
            || self.holding_profile.is_empty()
            || self.holding_profile.len() > 4096
            || self.holding_profile.iter().any(|point| {
                !point.duration_seconds.is_finite()
                    || point.duration_seconds < 0.0
                    || !percentage_valid(point.pressure_percent, 200.0)
            })
            || !self
                .holding_profile
                .iter()
                .map(|point| point.duration_seconds)
                .sum::<f64>()
                .is_finite()
        {
            return Err(ProjectError::CommandInvalid(
                "invalid fill process settings".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(super) fn validate_settings(
    imports: &[ImportRecord],
    settings: &BTreeMap<String, FillSettings>,
) -> Result<(), ProjectError> {
    for (id, setting) in settings {
        if !imports.iter().any(|record| record.id == *id) {
            return Err(ProjectError::ManifestInvalid(format!(
                "fill settings reference missing import {id}"
            )));
        }
        setting
            .validate()
            .map_err(|error| ProjectError::ManifestInvalid(error.to_string()))?;
    }
    Ok(())
}

impl ProjectService {
    /// 冻结目标和候选值，后台校验、写盘；返回 true 表示等待，false 表示已确认相同配置。
    /// 写入期间拒绝其他元数据命令；失败保留旧配置，结果消费前不能重新确认。
    pub fn begin_fill_settings_confirmation(
        &mut self,
        expected_path: &Path,
        expected_revision: u64,
        import_id: &str,
        settings: FillSettings,
    ) -> Result<bool, ProjectError> {
        let state = self.checked_plan_target(expected_path, expected_revision, import_id)?;
        if state
            .analysis_sequences
            .get(import_id)
            .map_or(analysis_sequence::DEFAULT_SEQUENCE_ID, String::as_str)
            != "fill"
        {
            return Err(ProjectError::CommandInvalid(
                "fill settings require Fill analysis sequence".to_owned(),
            ));
        }
        if state.fill_settings.get(import_id) == Some(&settings) {
            return Ok(false);
        }
        let mut candidate = state.clone();
        candidate
            .fill_settings
            .insert(import_id.to_owned(), settings);
        candidate.revision = candidate.revision.saturating_add(1);
        candidate.dirty = false;
        self.start_metadata_write(candidate, storage::MetadataWriteKind::FillSettings)
    }

    /// 非阻塞消费工艺确认结果；成功发布已写盘状态，失败解除写入锁。
    pub fn finish_fill_settings_confirmation(&mut self) -> Result<bool, ProjectError> {
        self.finish_metadata_write(storage::MetadataWriteKind::FillSettings)
    }
}
