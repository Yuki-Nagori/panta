//! 格式识别和来源快照；格式专属算法由对应领域后端执行。
//! STEP / IGES 的几何读取由后续 OCCT adapter 接入。

use panta_mesh::{StlError, SurfaceMesh, SurfaceSummary, parse_stl, surface_mesh_expanded_bytes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Stl,
    Step,
    Iges,
}

#[derive(Debug)]
pub enum ImportError {
    Missing(String),
    UnsupportedFormat(String),
    Read(String),
    Parse(StlError),
    SourceChanged(String),
    UnsupportedMeshType(String),
    UnsupportedUnits(String),
    CoordinateOverflow,
}

impl Display for ImportError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ImportError {}

pub const IMPORT_RECORD_VERSION: u32 = 4;
pub const STL_IMPORT_PARSER_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportRecord {
    pub record_version: u32,
    pub parser_version: u32,
    pub id: String,
    pub display_name: String,
    pub source_name: String,
    pub source_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_digest: Option<String>,
    pub source_size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_modified_unix_ms: Option<u64>,
    pub mesh_asset_id: String,
    pub format: String,
    pub mesh_type: String,
    pub units: String,
    pub show_import_log: bool,
    pub triangle_count: u64,
    pub dimensions: [f64; 3],
}

/// 返回与来源名称分离的默认资产名；只移除该记录所声明格式的扩展名。
pub fn default_display_name(source_name: &str, format: &str) -> String {
    let extension = format.strip_prefix('.').unwrap_or(format);
    source_name
        .rsplit_once('.')
        .filter(|(_, suffix)| suffix.eq_ignore_ascii_case(extension))
        .map_or_else(|| source_name.to_owned(), |(stem, _)| stem.to_owned())
}

#[derive(Debug, Clone, PartialEq)]
pub struct StlImportPreview {
    pub source_name: String,
    pub triangle_count: u64,
    pub dimensions: [f64; 3],
    /// 未压缩 Mesh 资产的估算字节数；不包含来源缓冲、显示 DTO 或 GPU 内存。
    pub expanded_mesh_bytes: u64,
    /// 仅提醒资源成本；不限制导入，也不依赖来源文件的压缩率或字节数。
    pub large_mesh_warning: bool,
}

impl StlImportPreview {
    fn from_summary(source_name: String, summary: SurfaceSummary) -> Self {
        const LARGE_MESH_WARNING_BYTES: u64 = 2 * 1024 * 1024 * 1024;
        let expanded_mesh_bytes = surface_mesh_expanded_bytes(summary.triangle_count);
        Self {
            source_name,
            triangle_count: summary.triangle_count,
            dimensions: summary.dimensions,
            expanded_mesh_bytes,
            large_mesh_warning: expanded_mesh_bytes > LARGE_MESH_WARNING_BYTES,
        }
    }
}

/// 可用网格类型的稳定 ID 和英文源文案；Qt 仅负责翻译和展示。
#[derive(Debug, Clone, Copy)]
pub struct MeshTypeDefinition {
    pub id: &'static str,
    pub source_text: &'static str,
}

pub const DEFAULT_MESH_TYPE: &str = "dual-domain";
pub const MESH_TYPES: &[MeshTypeDefinition] = &[
    MeshTypeDefinition {
        id: "midplane",
        source_text: "Midplane",
    },
    MeshTypeDefinition {
        id: "dual-domain",
        source_text: "Dual Domain",
    },
    MeshTypeDefinition {
        id: "solid-3d",
        source_text: "Solid 3D",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct ImportOptions {
    mesh_type: &'static str,
    units: LengthUnit,
    show_import_log: bool,
}

#[derive(Debug, Clone, Copy)]
enum LengthUnit {
    Millimeters,
    Centimeters,
    Inches,
}

impl LengthUnit {
    fn parse(value: &str) -> Result<Self, ImportError> {
        match value {
            "millimeters" => Ok(Self::Millimeters),
            "centimeters" => Ok(Self::Centimeters),
            "inches" => Ok(Self::Inches),
            other => Err(ImportError::UnsupportedUnits(other.to_owned())),
        }
    }

    fn factor_mm(self) -> f64 {
        match self {
            Self::Millimeters => 1.0,
            Self::Centimeters => 10.0,
            Self::Inches => 25.4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Millimeters => "millimeters",
            Self::Centimeters => "centimeters",
            Self::Inches => "inches",
        }
    }
}

impl ImportOptions {
    pub fn new(mesh_type: &str, units: &str, show_import_log: bool) -> Result<Self, ImportError> {
        let mesh_type = MESH_TYPES
            .iter()
            .find(|entry| entry.id == mesh_type)
            .map(|entry| entry.id)
            .ok_or_else(|| ImportError::UnsupportedMeshType(mesh_type.to_owned()))?;
        Ok(Self {
            mesh_type,
            units: LengthUnit::parse(units)?,
            show_import_log,
        })
    }
}

pub struct SourceSnapshot {
    pub path: PathBuf,
    pub source_name: String,
    pub format: ImportFormat,
    pub bytes: Vec<u8>,
    pub modified_unix_ms: Option<u64>,
}

/// 为导入来源计算稳定的 SHA-256 快照标识。
pub fn source_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn detect_format(path: &Path) -> Result<ImportFormat, ImportError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    match extension.to_ascii_lowercase().as_str() {
        "stl" => Ok(ImportFormat::Stl),
        "step" | "stp" => Ok(ImportFormat::Step),
        "iges" | "igs" => Ok(ImportFormat::Iges),
        _ => Err(ImportError::UnsupportedFormat(path.display().to_string())),
    }
}

pub fn read_source(path: &Path) -> Result<SourceSnapshot, ImportError> {
    // 永不取消的同步入口：Cancelled 分支按构造不可达。
    match read_source_checked(path, || false) {
        Ok(snapshot) => Ok(snapshot),
        Err(CheckedReadError::Failed(error)) => Err(error),
        Err(CheckedReadError::Cancelled(_)) => {
            unreachable!("a read that never checks cancellation cannot be cancelled")
        }
    }
}

/// 异步激活（073）使用的可取消读取错误：取消与读失败分开表达，
/// 不混入共享的 [`ImportError`]。
#[derive(Debug)]
pub enum CheckedReadError {
    Cancelled(String),
    Failed(ImportError),
}

impl Display for CheckedReadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CheckedReadError {}

/// 每个读取块（256 KiB）后查询一次取消检查点。
const READ_CHUNK_BYTES: usize = 256 * 1024;

/// 与 [`read_source`] 相同的读取契约，但分块执行并在每个检查点查询
/// `is_cancelled`；取消与 I/O 失败分别返回，供异步激活流程在安全点停止。
pub fn read_source_checked(
    path: &Path,
    is_cancelled: impl Fn() -> bool,
) -> Result<SourceSnapshot, CheckedReadError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(CheckedReadError::Failed(ImportError::Missing(
            path.display().to_string(),
        )));
    }
    let format = detect_format(path).map_err(CheckedReadError::Failed)?;
    let source_name = source_name(path).map_err(CheckedReadError::Failed)?;
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => return Err(read_error(path, error)),
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Err(read_error(path, error)),
    };
    let expected = metadata.len();
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|duration| u64::try_from(duration.as_millis()).ok());
    let mut bytes = Vec::new();
    reserve_source_bytes(
        path,
        &mut bytes,
        usize::try_from(expected).unwrap_or(usize::MAX),
    )?;
    let mut chunk = vec![0u8; READ_CHUNK_BYTES];
    loop {
        if is_cancelled() {
            return Err(CheckedReadError::Cancelled(path.display().to_string()));
        }
        let read = match std::io::Read::read(&mut file, &mut chunk) {
            Ok(read) => read,
            Err(error) => return Err(read_error(path, error)),
        };
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    Ok(SourceSnapshot {
        path: path.to_path_buf(),
        source_name,
        format,
        bytes,
        modified_unix_ms,
    })
}

fn source_name(path: &Path) -> Result<String, ImportError> {
    path.file_name()
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
        .ok_or_else(|| ImportError::UnsupportedFormat(path.display().to_string()))
}

fn read_error(path: &Path, error: std::io::Error) -> CheckedReadError {
    CheckedReadError::Failed(ImportError::Read(format!("{}: {error}", path.display())))
}

fn reserve_source_bytes(
    path: &Path,
    bytes: &mut Vec<u8>,
    capacity: usize,
) -> Result<(), CheckedReadError> {
    match bytes.try_reserve_exact(capacity) {
        Ok(()) => Ok(()),
        Err(error) => Err(CheckedReadError::Failed(ImportError::Read(format!(
            "{}: allocation failed: {error}",
            path.display()
        )))),
    }
}

/// 唯一的 STL 解析入口；几何格式尚无 native backend 时不得走此函数。
pub fn parse_stl_snapshot(snapshot: &SourceSnapshot) -> Result<SurfaceMesh, ImportError> {
    if snapshot.format != ImportFormat::Stl {
        return Err(ImportError::UnsupportedFormat(
            snapshot.path.display().to_string(),
        ));
    }
    parse_stl(&snapshot.bytes).map_err(ImportError::Parse)
}

/// 预览与提交共享同一字节快照；提交前重新读取源文件并逐字节比较。
#[derive(Debug, Default)]
pub struct StlImportSession {
    preview: Option<PreparedStl>,
}

#[derive(Debug)]
pub struct PreparedStl {
    pub path: PathBuf,
    pub source_name: String,
    pub bytes: Vec<u8>,
    pub mesh: SurfaceMesh,
    pub modified_unix_ms: Option<u64>,
}

impl PreparedStl {
    pub fn scale_mesh_mm(&mut self, options: ImportOptions) -> Result<(), ImportError> {
        scale_mesh_mm(&mut self.mesh, options.units)
    }

    pub fn record(&self, import_number: usize, options: ImportOptions) -> ImportRecord {
        let summary = self.summary();
        ImportRecord {
            record_version: IMPORT_RECORD_VERSION,
            parser_version: STL_IMPORT_PARSER_VERSION,
            id: format!("import-{import_number}"),
            display_name: default_display_name(&self.source_name, "stl"),
            source_name: self.source_name.clone(),
            source_path: self.path.to_string_lossy().into_owned(),
            source_digest: Some(source_digest(&self.bytes)),
            source_size_bytes: u64::try_from(self.bytes.len()).unwrap_or(u64::MAX),
            source_modified_unix_ms: self.modified_unix_ms,
            mesh_asset_id: format!("mesh-import-{import_number}"),
            format: "stl".to_owned(),
            mesh_type: options.mesh_type.to_owned(),
            units: options.units.label().to_owned(),
            show_import_log: options.show_import_log,
            triangle_count: summary.triangle_count,
            dimensions: summary.dimensions,
        }
    }

    pub fn summary(&self) -> SurfaceSummary {
        self.mesh.summary()
    }
}

impl StlImportSession {
    pub fn clear(&mut self) {
        self.preview = None;
    }

    pub fn has_preview_for(&self, path: &Path) -> bool {
        self.preview
            .as_ref()
            .is_some_and(|preview| preview.path == path)
    }

    pub fn preview(&mut self, path: &Path) -> Result<StlImportPreview, ImportError> {
        match self.preview_checked(path, || false) {
            Ok(preview) => Ok(preview),
            Err(CheckedReadError::Failed(error)) => Err(error),
            Err(CheckedReadError::Cancelled(_)) => unreachable!("uncancellable preview"),
        }
    }

    /// 后台预检使用分块读取检查取消；解析不能中断，解析后的快照发布前再次检查。
    /// 取消与失败都清空本次预览，不改变源文件。
    pub fn preview_checked(
        &mut self,
        path: &Path,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<StlImportPreview, CheckedReadError> {
        self.preview = None;
        let input = read_source_checked(path, &is_cancelled)?;
        let mesh = parse_stl_snapshot(&input).map_err(CheckedReadError::Failed)?;
        if is_cancelled() {
            return Err(CheckedReadError::Cancelled(path.display().to_string()));
        }
        let summary = mesh.summary();
        let source_name = input.source_name.clone();
        self.preview = Some(PreparedStl {
            path: input.path,
            source_name: input.source_name,
            bytes: input.bytes,
            mesh,
            modified_unix_ms: input.modified_unix_ms,
        });
        Ok(StlImportPreview::from_summary(source_name, summary))
    }

    pub fn prepare(&mut self, path: &Path) -> Result<PreparedStl, ImportError> {
        match self.prepare_checked(path, || false) {
            Ok(prepared) => Ok(prepared),
            Err(CheckedReadError::Failed(error)) => Err(error),
            Err(CheckedReadError::Cancelled(_)) => {
                unreachable!("an import preparation that never checks cancellation cannot cancel")
            }
        }
    }

    /// 后台确认导入复核源快照时可在分块读取边界取消。
    /// 源快照一致后复用预检网格，避免重复解析；取消或变化不消耗原预览。
    pub fn prepare_checked(
        &mut self,
        path: &Path,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<PreparedStl, CheckedReadError> {
        let input = read_source_checked(path, &is_cancelled)?;
        if input.format != ImportFormat::Stl {
            return Err(CheckedReadError::Failed(ImportError::UnsupportedFormat(
                path.display().to_string(),
            )));
        }
        if let Some(cached) = self.preview.as_ref().filter(|cached| cached.path == path) {
            if cached.bytes != input.bytes {
                return Err(CheckedReadError::Failed(ImportError::SourceChanged(
                    path.display().to_string(),
                )));
            }
            if is_cancelled() {
                return Err(CheckedReadError::Cancelled(path.display().to_string()));
            }
            if let Some(mut confirmed) = self.preview.take() {
                confirmed.modified_unix_ms = input.modified_unix_ms;
                return Ok(confirmed);
            }
        }
        let mesh = parse_stl_snapshot(&input).map_err(CheckedReadError::Failed)?;
        if is_cancelled() {
            return Err(CheckedReadError::Cancelled(path.display().to_string()));
        }
        Ok(PreparedStl {
            path: input.path,
            source_name: input.source_name,
            bytes: input.bytes,
            mesh,
            modified_unix_ms: input.modified_unix_ms,
        })
    }
}

fn scale_mesh_mm(mesh: &mut SurfaceMesh, units: LengthUnit) -> Result<(), ImportError> {
    let factor = units.factor_mm();
    for triangle in &mut mesh.triangles {
        for point in triangle {
            for coordinate in point {
                *coordinate *= factor;
                if !coordinate.is_finite() {
                    return Err(ImportError::CoordinateOverflow);
                }
            }
        }
    }
    if mesh
        .summary()
        .dimensions
        .iter()
        .any(|size| !size.is_finite())
    {
        return Err(ImportError::CoordinateOverflow);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
