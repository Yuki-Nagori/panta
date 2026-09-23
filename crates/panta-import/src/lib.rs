//! 格式识别和来源快照；格式专属算法由对应领域后端执行。
//! STEP / IGES 的几何读取由后续 OCCT adapter 接入。

use panta_mesh::{StlError, SurfaceMesh, SurfaceSummary, parse_stl};
use serde::{Deserialize, Serialize};
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

pub const IMPORT_RECORD_VERSION: u32 = 1;
pub const STL_IMPORT_PARSER_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportRecord {
    pub record_version: u32,
    pub parser_version: u32,
    pub id: String,
    pub source_name: String,
    pub asset: String,
    pub format: String,
    pub mesh_type: String,
    pub units: String,
    pub show_import_log: bool,
    pub triangle_count: u64,
    pub dimensions: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct StlImportPreview {
    pub source_name: String,
    pub triangle_count: u64,
    pub dimensions: [f64; 3],
}

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
        let mesh_type = match mesh_type {
            "midplane" => "midplane",
            "dual-domain" => "dual-domain",
            "solid-3d" => "solid-3d",
            other => return Err(ImportError::UnsupportedMeshType(other.to_owned())),
        };
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
    if !path.is_absolute() || !path.is_file() {
        return Err(ImportError::Missing(path.display().to_string()));
    }
    let format = detect_format(path)?;
    let source_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ImportError::UnsupportedFormat(path.display().to_string()))?
        .to_owned();
    let bytes = fs::read(path)
        .map_err(|error| ImportError::Read(format!("{}: {error}", path.display())))?;
    Ok(SourceSnapshot {
        path: path.to_path_buf(),
        source_name,
        format,
        bytes,
    })
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
}

impl PreparedStl {
    pub fn asset_name(&self, import_number: usize) -> String {
        format!("{import_number:04}-{}", self.source_name)
    }

    pub fn scale_mesh_mm(&mut self, options: ImportOptions) -> Result<(), ImportError> {
        scale_mesh_mm(&mut self.mesh, options.units)
    }

    pub fn record(&self, import_number: usize, options: ImportOptions) -> ImportRecord {
        let summary = self.summary();
        ImportRecord {
            record_version: IMPORT_RECORD_VERSION,
            parser_version: STL_IMPORT_PARSER_VERSION,
            id: format!("import-{import_number}"),
            source_name: self.source_name.clone(),
            asset: format!("assets/imports/{}", self.asset_name(import_number)),
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

    pub fn preview(&mut self, path: &Path) -> Result<StlImportPreview, ImportError> {
        self.preview = None;
        let input = read_source(path)?;
        let mesh = parse_stl_snapshot(&input)?;
        let summary = mesh.summary();
        let source_name = input.source_name.clone();
        self.preview = Some(PreparedStl {
            path: input.path,
            source_name: input.source_name,
            bytes: input.bytes,
            mesh,
        });
        Ok(StlImportPreview {
            source_name,
            triangle_count: summary.triangle_count,
            dimensions: summary.dimensions,
        })
    }

    pub fn prepare(&mut self, path: &Path) -> Result<PreparedStl, ImportError> {
        let input = read_source(path)?;
        if input.format != ImportFormat::Stl {
            return Err(ImportError::UnsupportedFormat(path.display().to_string()));
        }
        if let Some(cached) = self.preview.as_ref().filter(|cached| cached.path == path) {
            if cached.bytes != input.bytes {
                return Err(ImportError::SourceChanged(path.display().to_string()));
            }
            if let Some(confirmed) = self.preview.take() {
                return Ok(confirmed);
            }
        }
        let mesh = parse_stl_snapshot(&input)?;
        Ok(PreparedStl {
            path: input.path,
            source_name: input.source_name,
            bytes: input.bytes,
            mesh,
        })
    }
}

pub fn load_stl_asset(path: &Path, units: &str) -> Result<SurfaceMesh, ImportError> {
    let input = read_source(path)?;
    let mut mesh = parse_stl_snapshot(&input)?;
    scale_mesh_mm(&mut mesh, LengthUnit::parse(units)?)?;
    Ok(mesh)
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
mod tests {
    use super::*;

    #[test]
    fn recognizes_current_and_future_import_formats() {
        assert_eq!(
            detect_format(Path::new("a.STL")).ok(),
            Some(ImportFormat::Stl)
        );
        assert_eq!(
            detect_format(Path::new("a.stp")).ok(),
            Some(ImportFormat::Step)
        );
        assert_eq!(
            detect_format(Path::new("a.IGS")).ok(),
            Some(ImportFormat::Iges)
        );
        assert!(detect_format(Path::new("a.txt")).is_err());
    }
}
