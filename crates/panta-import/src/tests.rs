use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("panta-import-{}-{id}", std::process::id()));
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
fn display_name_strips_only_the_declared_format_extension() {
    assert_eq!(default_display_name("part.stl", "stl"), "part");
    assert_eq!(default_display_name("Case.STL", "stl"), "Case");
    assert_eq!(default_display_name("零件.模型.stl", "stl"), "零件.模型");
    assert_eq!(default_display_name("part", "stl"), "part");
    assert_eq!(default_display_name("part.stl.bak", "stl"), "part.stl.bak");
    assert_eq!(default_display_name("part.step", "step"), "part");
    assert_eq!(default_display_name("part.stl", "step"), "part.stl");
}

#[test]
fn preview_reports_expanded_asset_cost_without_rejecting_large_counts() {
    let preview = |triangle_count| {
        StlImportPreview::from_summary(
            "model.stl".to_owned(),
            SurfaceSummary {
                triangle_count,
                dimensions: [1.0; 3],
            },
        )
    };
    assert_eq!(preview(1).expanded_mesh_bytes, 136);
    let below = preview(29_826_160);
    assert_eq!(below.expanded_mesh_bytes, 2_147_483_584);
    assert!(!below.large_mesh_warning);
    let above = preview(29_826_161);
    assert_eq!(above.expanded_mesh_bytes, 2_147_483_656);
    assert!(above.large_mesh_warning);
    let overflowing = preview(u64::MAX);
    assert_eq!(overflowing.expanded_mesh_bytes, u64::MAX);
    assert!(overflowing.large_mesh_warning);
    assert_eq!(overflowing.triangle_count, u64::MAX);
}

#[test]
fn preview_cancellation_before_read_and_after_parse_does_not_publish_a_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let path = fixture.root.join("preview.stl");
    fs::write(&path, "vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n")?;
    let mut session = StlImportSession::default();
    assert!(matches!(
        session.preview_checked(&path, || true),
        Err(CheckedReadError::Cancelled(_))
    ));
    assert!(session.preview.is_none());
    let checks = AtomicUsize::new(0);
    // 小文件经过两次读检查（数据、EOF），第三次位于解析后发布前。
    assert!(matches!(
        session.preview_checked(&path, || checks.fetch_add(1, Ordering::Relaxed) >= 2),
        Err(CheckedReadError::Cancelled(_))
    ));
    assert!(session.preview.is_none());
    assert_eq!(session.preview(&path)?.triangle_count, 1);
    Ok(())
}

#[test]
fn import_options_records_and_unit_scaling_are_consistent() -> Result<(), Box<dyn std::error::Error>>
{
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
    assert_eq!(preview.expanded_mesh_bytes, 136);
    assert!(!preview.large_mesh_warning);

    let prepared = session.prepare(&source)?;
    assert_eq!(prepared.summary().dimensions, preview.dimensions);
    let record = prepared.record(7, centimeters);
    assert_eq!(record.record_version, IMPORT_RECORD_VERSION);
    assert_eq!(record.parser_version, STL_IMPORT_PARSER_VERSION);
    assert_eq!(record.id, "import-7");
    assert_eq!(record.display_name, "part");
    assert_eq!(
        record.source_digest,
        Some(format!("sha256:{:x}", Sha256::digest(fs::read(&source)?)))
    );
    assert_eq!(record.mesh_asset_id, "mesh-import-7");
    assert_eq!(record.source_path, source.to_string_lossy());
    assert_eq!(record.source_size_bytes, fs::metadata(&source)?.len());
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
        modified_unix_ms: None,
    };
    assert!(matches!(
        parse_stl_snapshot(&non_stl),
        Err(ImportError::UnsupportedFormat(_))
    ));
    let non_stl_path = fixture.root.join("part.step");
    fs::write(&non_stl_path, original)?;
    assert!(matches!(
        StlImportSession::default().prepare(&non_stl_path),
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
    let centimeters = ImportOptions::new("dual-domain", "centimeters", false)?;
    let mut mesh = StlImportSession::default().prepare(&source)?;
    mesh.scale_mesh_mm(centimeters)?;
    assert_eq!(mesh.summary().dimensions, [20.0, 30.0, 0.0]);
    assert!(matches!(
        ImportOptions::new("dual-domain", "yards", false),
        Err(ImportError::UnsupportedUnits(units)) if units == "yards"
    ));

    fs::write(
        &source,
        format!(
            "vertex 0 0 0\nvertex {} 0 0\nvertex 0 1 0\n",
            f64::MAX / 2.0
        ),
    )?;
    let mut scaled_mesh = StlImportSession::default().prepare(&source)?;
    let inches = ImportOptions::new("dual-domain", "inches", false)?;
    assert!(matches!(
        scaled_mesh.scale_mesh_mm(inches),
        Err(ImportError::CoordinateOverflow)
    ));

    // 有限坐标本身仍可能让包围盒的 max - min 溢出。
    let mut wide_mesh = SurfaceMesh {
        triangles: vec![[[-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0], [0.0, 1.0, 0.0]]],
    };
    assert!(matches!(
        scale_mesh_mm(&mut wide_mesh, LengthUnit::Millimeters),
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
    let snapshot = read_source_checked(&source, || false)?;
    assert_eq!(snapshot.bytes.len(), payload.len());

    assert!(matches!(
        read_source_checked(&source, || true),
        Err(CheckedReadError::Cancelled(_))
    ));
    let large_source = fixture.root.join("large.stl");
    fs::write(&large_source, vec![b' '; READ_CHUNK_BYTES + 1])?;
    let checkpoints = AtomicUsize::new(0);
    assert!(matches!(
        read_source_checked(&large_source, || {
            checkpoints.fetch_add(1, Ordering::Relaxed) == 1
        }),
        Err(CheckedReadError::Cancelled(path)) if path == large_source.display().to_string()
    ));
    // 第二次检查发生在首个 256 KiB 数据块读取后。
    assert_eq!(checkpoints.load(Ordering::Relaxed), 2);
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

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        let non_utf8_name = std::ffi::OsString::from_vec(b"part-\xff.stl".to_vec());
        let non_utf8 = fixture.root.join(non_utf8_name);
        match fs::write(&non_utf8, payload) {
            Ok(()) => {
                let source = read_source_checked(&non_utf8, || false)?;
                assert_eq!(source.source_name, "part-\u{fffd}.stl");
                fs::remove_file(non_utf8)?;
            }
            // Some filesystems reject non-UTF-8 names before they can be read.
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    || error.kind() == std::io::ErrorKind::InvalidInput
                    || (cfg!(target_os = "macos") && error.raw_os_error() == Some(92)) =>
            {
                assert_eq!(source_name(&non_utf8)?, "part-\u{fffd}.stl");
            }
            Err(error) => return Err(error.into()),
        }
    }

    assert!(matches!(
        read_error(&source, std::io::Error::other("fixture error")),
        CheckedReadError::Failed(ImportError::Read(message))
            if message.contains("part.stl") && message.contains("fixture error")
    ));
    let mut unreserved = Vec::new();
    assert!(matches!(
        reserve_source_bytes(&source, &mut unreserved, usize::MAX),
        Err(CheckedReadError::Failed(ImportError::Read(message)))
            if message.contains("part.stl") && message.contains("allocation failed")
    ));

    let cancelled = CheckedReadError::Cancelled(source.display().to_string());
    assert!(cancelled.to_string().contains("Cancelled"));
    let failed = CheckedReadError::Failed(ImportError::Missing(source.display().to_string()));
    assert!(failed.to_string().contains("Missing"));
    Ok(())
}
