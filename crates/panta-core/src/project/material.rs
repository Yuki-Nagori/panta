//! 内置材料的展示目录和按导入记录保存的选择；不执行材料本构计算。
use super::*;
use std::sync::LazyLock;

#[derive(Deserialize)]
struct MaterialData {
    schema: String,
    id: String,
    source_text: String,
    semantics: MaterialSemantics,
    processing: Processing,
}

#[derive(Deserialize)]
struct MaterialSemantics {
    family: String,
    filler_content_percent: f64,
}

#[derive(Deserialize)]
struct Processing {
    recommended_melt_range_k: [f64; 2],
    recommended_mold_range_k: [f64; 2],
    ejection_temperature_k: f64,
    maximum_shear_stress_pa: f64,
    maximum_shear_rate_s: f64,
}

/// 只读材料摘要，包含版本 ID、显示名称和带单位的参数。
pub struct MaterialDefinition {
    pub family_source_text: String,
    pub id: String,
    pub source_text: String,
    pub properties: Vec<MaterialProperty>,
}

/// 英文源标签由 Qt 翻译；数值格式与单位由 Rust 确定。
pub struct MaterialProperty {
    pub source_text: &'static str,
    pub value: String,
}

static MATERIAL: LazyLock<Result<MaterialDefinition, String>> = LazyLock::new(|| {
    // 仅解析随程序编译的受控资源；新增资源时须核对结构和单位。
    let data: MaterialData = serde_json::from_str(include_str!(
        "../../../../resources/materials/pp-mineral-25.json"
    ))
    .map_err(|error| format!("invalid built-in material: {error}"))?;
    if data.schema != "panta.material-preview/v1" {
        return Err("unsupported built-in material schema".to_owned());
    }
    let range =
        |values: [f64; 2]| format!("{:.0}–{:.0} °C", values[0] - 273.15, values[1] - 273.15);
    Ok(MaterialDefinition {
        family_source_text: data.semantics.family,
        id: data.id,
        source_text: data.source_text,
        properties: vec![
            MaterialProperty {
                source_text: "Mineral filler",
                value: format!("{:.0}%", data.semantics.filler_content_percent),
            },
            MaterialProperty {
                source_text: "Recommended melt temperature",
                value: range(data.processing.recommended_melt_range_k),
            },
            MaterialProperty {
                source_text: "Recommended mold temperature",
                value: range(data.processing.recommended_mold_range_k),
            },
            MaterialProperty {
                source_text: "Ejection temperature",
                value: format!("{:.0} °C", data.processing.ejection_temperature_k - 273.15),
            },
            MaterialProperty {
                source_text: "Maximum shear stress",
                value: format!(
                    "{:.0} kPa",
                    data.processing.maximum_shear_stress_pa / 1000.0
                ),
            },
            MaterialProperty {
                source_text: "Maximum shear rate",
                value: format!("{:.0} s⁻¹", data.processing.maximum_shear_rate_s),
            },
        ],
    })
});

/// 返回随程序内置的默认材料，资源解析只发生一次。
/// 资源格式错误时返回可恢复错误，不向界面发布部分摘要。
pub fn default_material() -> Result<&'static MaterialDefinition, ProjectError> {
    MATERIAL
        .as_ref()
        .map_err(|error| ProjectError::ManifestInvalid(error.clone()))
}

pub(super) fn validate_materials(manifest: &ProjectManifest) -> Result<(), ProjectError> {
    for (id, material) in &manifest.materials {
        if !manifest.imports.iter().any(|record| record.id == *id)
            || material != &default_material()?.id
        {
            return Err(ProjectError::ManifestInvalid(format!(
                "invalid material for {id}"
            )));
        }
    }
    Ok(())
}

impl ProjectService {
    /// 校验并启动后台材料确认；序列化和写盘均在 Rust 工作线程执行。
    /// 返回 true 表示等待结果，false 表示相同材料已确认，无需写盘。
    /// 身份、修订、记录或材料无效，以及线程启动失败时不改变工程。
    /// 启动后不可取消；必须消费结果后才能发起其他工程写入。
    pub fn begin_material_confirmation(
        &mut self,
        expected_path: &Path,
        expected_revision: u64,
        import_id: &str,
        material_id: &str,
    ) -> Result<bool, ProjectError> {
        let state = self.checked_plan_target(expected_path, expected_revision, import_id)?;
        if material_id != default_material()?.id {
            return Err(ProjectError::CommandInvalid("unknown material".to_owned()));
        }
        if state
            .materials
            .get(import_id)
            .is_some_and(|id| id == material_id)
        {
            return Ok(false);
        }
        let mut candidate = state.clone();
        candidate
            .materials
            .insert(import_id.to_owned(), material_id.to_owned());
        candidate.revision = candidate.revision.saturating_add(1);
        candidate.dirty = false;
        self.start_metadata_write(candidate, storage::MetadataWriteKind::Material)
    }

    /// 非阻塞消费确认结果；尚未完成返回 false，成功发布已写盘状态并返回 true。
    /// 写盘或工作线程失败时解除写入锁并保留旧状态；结果只消费一次。
    pub fn finish_material_confirmation(&mut self) -> Result<bool, ProjectError> {
        self.finish_metadata_write(storage::MetadataWriteKind::Material)
    }
}
