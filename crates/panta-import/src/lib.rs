//! 格式识别和来源快照；格式专属算法由对应领域后端执行。
//! STEP / IGES 的几何读取由后续 OCCT adapter 接入。

use panta_mesh::{StlError, SurfaceMesh, SurfaceSummary, parse_stl};
use serde::{Deserialize, Serialize};
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
    let read_failed = |error: std::io::Error| {
        CheckedReadError::Failed(ImportError::Read(format!("{}: {error}", path.display())))
    };
    if !path.is_absolute() || !path.is_file() {
        return Err(CheckedReadError::Failed(ImportError::Missing(
            path.display().to_string(),
        )));
    }
    let format = detect_format(path).map_err(CheckedReadError::Failed)?;
    let source_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CheckedReadError::Failed(ImportError::UnsupportedFormat(path.display().to_string()))
        })?
        .to_owned();
    let mut file = fs::File::open(path).map_err(read_failed)?;
    let expected = file.metadata().map_err(read_failed)?.len();
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(usize::try_from(expected).unwrap_or(usize::MAX))
        .map_err(|error| {
            CheckedReadError::Failed(ImportError::Read(format!(
                "{}: allocation failed: {error}",
                path.display()
            )))
        })?;
    let mut chunk = vec![0u8; READ_CHUNK_BYTES];
    loop {
        if is_cancelled() {
            return Err(CheckedReadError::Cancelled(path.display().to_string()));
        }
        let read = std::io::Read::read(&mut file, &mut chunk).map_err(read_failed)?;
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
    })
}

/// 已提交工程资产的解析入口：STL 解析 + 记录单位换算到毫米。
/// 与 [`read_source_checked`] 组合构成 073 只读激活的加载阶段；
/// 解析本身不可中断，过期快照由调用方在完成边界释放。
pub fn parse_stl_asset(snapshot: &SourceSnapshot, units: &str) -> Result<SurfaceMesh, ImportError> {
    let mut mesh = parse_stl_snapshot(snapshot)?;
    scale_mesh_mm(&mut mesh, LengthUnit::parse(units)?)?;
    Ok(mesh)
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("panta-import-{}-{id}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root)?;
            Ok(Self { root })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

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

    #[test]
    fn import_options_records_and_unit_scaling_are_consistent()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("part.stl");
        fs::write(
            &source,
            b"solid part\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n",
        )?;

        let millimeters = ImportOptions::new("midplane", "millimeters", false)?;
        let centimeters = ImportOptions::new("dual-domain", "centimeters", true)?;
        let inches = ImportOptions::new("solid-3d", "inches", false)?;
        assert!(matches!(
            ImportOptions::new("surface", "millimeters", false),
            Err(ImportError::UnsupportedMeshType(kind)) if kind == "surface"
        ));
        assert!(matches!(
            ImportOptions::new("solid-3d", "metres", false),
            Err(ImportError::UnsupportedUnits(units)) if units == "metres"
        ));
        assert_eq!(
            ImportError::CoordinateOverflow.to_string(),
            "CoordinateOverflow"
        );

        let mut session = StlImportSession::default();
        let preview = session.preview(&source)?;
        assert_eq!(preview.source_name, "part.stl");
        assert_eq!(preview.triangle_count, 1);
        assert_eq!(preview.dimensions, [1.0, 2.0, 0.0]);

        let prepared = session.prepare(&source)?;
        assert_eq!(prepared.asset_name(7), "0007-part.stl");
        assert_eq!(prepared.summary().dimensions, preview.dimensions);
        let record = prepared.record(7, centimeters);
        assert_eq!(record.record_version, IMPORT_RECORD_VERSION);
        assert_eq!(record.parser_version, STL_IMPORT_PARSER_VERSION);
        assert_eq!(record.id, "import-7");
        assert_eq!(record.asset, "assets/imports/0007-part.stl");
        assert_eq!(record.mesh_type, "dual-domain");
        assert_eq!(record.units, "centimeters");
        assert!(record.show_import_log);
        assert_eq!(record.triangle_count, 1);
        assert_eq!(record.dimensions, [1.0, 2.0, 0.0]);

        let mut millimeter_mesh = prepared;
        millimeter_mesh.scale_mesh_mm(millimeters)?;
        assert_eq!(millimeter_mesh.summary().dimensions, [1.0, 2.0, 0.0]);
        let mut inch_mesh = session.prepare(&source)?;
        inch_mesh.scale_mesh_mm(inches)?;
        assert_eq!(inch_mesh.summary().dimensions, [25.4, 50.8, 0.0]);

        session.preview(&source)?;
        session.clear();
        let cleared = session.prepare(&source)?;
        assert_eq!(cleared.summary().dimensions, [1.0, 2.0, 0.0]);
        Ok(())
    }

    #[test]
    fn source_reads_and_previews_reject_invalid_or_changed_inputs()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("part.stl");
        let original = b"vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n";
        fs::write(&source, original)?;

        assert!(matches!(
            detect_format(Path::new("extensionless")),
            Err(ImportError::UnsupportedFormat(_))
        ));
        assert!(matches!(
            read_source(Path::new("relative.stl")),
            Err(ImportError::Missing(_))
        ));
        assert!(matches!(
            read_source(&fixture.root.join("missing.stl")),
            Err(ImportError::Missing(_))
        ));
        let unsupported = fixture.root.join("part.txt");
        fs::write(&unsupported, b"text")?;
        assert!(matches!(
            read_source(&unsupported),
            Err(ImportError::UnsupportedFormat(_))
        ));

        let non_stl = SourceSnapshot {
            path: fixture.root.join("part.step"),
            source_name: "part.step".to_owned(),
            format: ImportFormat::Step,
            bytes: original.to_vec(),
        };
        assert!(matches!(
            parse_stl_snapshot(&non_stl),
            Err(ImportError::UnsupportedFormat(_))
        ));

        let mut session = StlImportSession::default();
        session.preview(&source)?;
        fs::write(&source, b"vertex 0 0 0\nvertex 3 0 0\nvertex 0 4 0\n")?;
        assert!(matches!(
            session.prepare(&source),
            Err(ImportError::SourceChanged(_))
        ));
        session.clear();
        let updated = session.prepare(&source)?;
        assert_eq!(updated.summary().dimensions, [3.0, 4.0, 0.0]);

        fs::write(&source, b"invalid stl")?;
        assert!(matches!(
            session.preview(&source),
            Err(ImportError::Parse(_))
        ));
        fs::write(&source, original)?;
        let reread = session.prepare(&source)?;
        assert_eq!(reread.summary().dimensions, [1.0, 1.0, 0.0]);
        Ok(())
    }

    #[test]
    fn load_asset_scales_units_and_reports_coordinate_overflow()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("part.stl");
        fs::write(&source, b"vertex 0 0 0\nvertex 2 0 0\nvertex 0 3 0\n")?;
        let snapshot = read_source(&source)?;
        let mesh = parse_stl_asset(&snapshot, "centimeters")?;
        assert_eq!(mesh.summary().dimensions, [20.0, 30.0, 0.0]);
        assert!(matches!(
            parse_stl_asset(&snapshot, "yards"),
            Err(ImportError::UnsupportedUnits(units)) if units == "yards"
        ));

        fs::write(
            &source,
            format!(
                "vertex 0 0 0\nvertex {} 0 0\nvertex 0 1 0\n",
                f64::MAX / 2.0
            ),
        )?;
        let scaled_snapshot = read_source(&source)?;
        assert!(matches!(
            parse_stl_asset(&scaled_snapshot, "inches"),
            Err(ImportError::CoordinateOverflow)
        ));
        Ok(())
    }

    #[test]
    fn checked_read_supports_cancellation_checkpoints() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = Fixture::new()?;
        let source = fixture.root.join("part.stl");
        let payload = b"vertex 0 0 0\nvertex 2 0 0\nvertex 0 3 0\n";
        fs::write(&source, payload)?;
        let snapshot = read_source_checked(&source, || false).map_err(|error| error.to_string())?;
        assert_eq!(snapshot.bytes.len(), payload.len());

        assert!(matches!(
            read_source_checked(&source, || true),
            Err(CheckedReadError::Cancelled(_))
        ));
        // 不存在的文件以 Failed(Missing) 表达，不冒充取消。
        assert!(matches!(
            read_source_checked(&fixture.root.join("absent.stl"), || false),
            Err(CheckedReadError::Failed(ImportError::Missing(_)))
        ));
        // 未登记扩展名在读取前即拒绝（与 read_source 同一契约）；
        // .step 能通过格式识别，拒绝发生在解析层，不属读取边界。
        let unknown = fixture.root.join("part.obj");
        fs::write(&unknown, b"anything")?;
        assert!(matches!(
            read_source_checked(&unknown, || false),
            Err(CheckedReadError::Failed(ImportError::UnsupportedFormat(_)))
        ));
        Ok(())
    }
}
