//! 073/080 只读 STL 资产激活的集成行为：成功、失败、取消、工程切换失效、
//! 去重与工程持久状态不变式。经 `ProjectService` 公共 API 驱动，worker 为
//! 真实线程；结果以 `drain_asset_activations` 轮询拉取。
use panta_core::project::{Outcome, OutcomeKind, ProjectError, ProjectService};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("panta-activation-{}-{}", std::process::id(), id));
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

const SAMPLE_STL: &[u8] = b"vertex 0 0 0\nvertex 2 0 0\nvertex 0 3 0\n";

fn import_sample_stl(
    service: &mut ProjectService,
    root: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let source = root.join("part.stl");
    fs::write(&source, SAMPLE_STL)?;
    let record = service.import_stl(&source, "solid-3d", "millimeters", false)?;
    service.save()?;
    Ok(record.id)
}

/// 大体积二进制 STL：让读取/解析窗口远大于测试线程设置取消的延迟，
/// 使取消、去重与失效用例摆脱调度时序依赖。
fn overwrite_asset_with_binary_stl(
    asset: &Path,
    triangles: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0u8; 80]);
    bytes.extend_from_slice(&(triangles as u32).to_le_bytes());
    let mut payload = [0u8; 50];
    let vertices: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    payload[12..48].copy_from_slice(&bytemuck_vertices(&vertices));
    for _ in 0..triangles {
        bytes.extend_from_slice(&payload);
    }
    let mut file = fs::File::create(asset)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn bytemuck_vertices(vertices: &[f32; 9]) -> [u8; 36] {
    // f32 与字节互转的标准安全路径；二进制 STL 载荷就是小端 f32 序列。
    let mut bytes = [0u8; 36];
    for (index, value) in vertices.iter().enumerate() {
        bytes[index * 4..(index + 1) * 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn asset_path(
    service: &ProjectService,
    record_id: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let record = service
        .imports()?
        .into_iter()
        .find(|record| record.id == record_id)
        .ok_or("import record not found")?;
    let root = service
        .current()?
        .path
        .parent()
        .ok_or("no project dir")?
        .to_path_buf();
    // 清单内相对路径使用 '/'；Path::join 在各平台均接受该分隔符。
    Ok(root.join(&record.asset))
}

fn wait_for_outcome(
    service: &mut ProjectService,
    timeout: Duration,
) -> Result<Outcome, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + timeout;
    loop {
        let outcomes = service.drain_asset_activations();
        if let Some(outcome) = outcomes.into_iter().next() {
            return Ok(outcome);
        }
        if Instant::now() >= deadline {
            return Err("activation outcome did not arrive in time".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn activation_restores_saved_asset_without_touching_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    let manifest_before = fs::read(service.current()?.path.clone())?;
    let revision_before = service.current()?.revision;
    drop(service);

    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    // 打开不再同步解析资产；网格只能经只读激活恢复。
    assert!(reopened.current_mesh().is_none());
    let attempt = reopened.begin_asset_activation(&record_id)?;
    let outcome = wait_for_outcome(&mut reopened, Duration::from_secs(10))?;
    assert_eq!(outcome.kind, OutcomeKind::Succeeded);
    assert_eq!(outcome.attempt, attempt.attempt);
    assert_eq!(outcome.generation, attempt.generation);
    assert_eq!(outcome.import_id, record_id);
    let mesh = outcome.mesh.ok_or("success must carry mesh")?;
    assert_eq!(mesh.summary().dimensions, [2.0, 3.0, 0.0]);

    // 加载路径不推进修订、不标脏、不改写清单。
    assert_eq!(reopened.current()?.revision, revision_before);
    assert!(!reopened.current()?.dirty);
    assert_eq!(fs::read(reopened.current()?.path.clone())?, manifest_before);
    Ok(())
}

#[test]
fn unknown_record_and_missing_project_fail_synchronously() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    assert!(matches!(
        service.begin_asset_activation("import-1"),
        Err(ProjectError::NoProject)
    ));
    service.create(&fixture.root, "Demo")?;
    assert!(matches!(
        service.begin_asset_activation("import-99"),
        Err(ProjectError::ImportRecordMissing(record)) if record == "import-99"
    ));
    Ok(())
}

#[test]
fn corrupt_asset_fails_with_stable_code() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    let asset = asset_path(&service, &record_id)?;
    fs::write(&asset, b"this is not a stl payload")?;
    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    reopened.begin_asset_activation(&record_id)?;
    let outcome = wait_for_outcome(&mut reopened, Duration::from_secs(10))?;
    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.code, "project.asset_parse_failed");
    Ok(())
}

#[test]
fn missing_asset_file_fails_with_stable_code() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    fs::remove_file(asset_path(&service, &record_id)?)?;
    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    reopened.begin_asset_activation(&record_id)?;
    let outcome = wait_for_outcome(&mut reopened, Duration::from_secs(10))?;
    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.code, "project.asset_missing");
    Ok(())
}

#[test]
fn cancel_request_terminates_as_cancelled() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    overwrite_asset_with_binary_stl(&asset_path(&service, &record_id)?, 350_000)?;

    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    let attempt = reopened.begin_asset_activation(&record_id)?;
    assert!(reopened.cancel_asset_activation(attempt.attempt));
    let outcome = wait_for_outcome(&mut reopened, Duration::from_secs(10))?;
    assert_eq!(outcome.kind, OutcomeKind::Cancelled);
    assert!(outcome.mesh.is_none());
    assert!(
        !reopened.cancel_asset_activation(attempt.attempt),
        "终态后取消必须返回 false"
    );
    Ok(())
}

#[test]
fn project_switch_invalidates_in_flight_results() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    overwrite_asset_with_binary_stl(&asset_path(&service, &record_id)?, 350_000)?;

    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    reopened.begin_asset_activation(&record_id)?;
    // 切换工程（新会话代次）：旧 attempt 立即失效，结果不跨会话发布。
    reopened.create(&fixture.root, "Other")?;
    assert!(reopened.drain_asset_activations().is_empty());
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        reopened.drain_asset_activations().is_empty(),
        "迟到成功不得覆盖新工程会话"
    );
    Ok(())
}

#[test]
fn duplicate_begin_reuses_in_flight_attempt() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    overwrite_asset_with_binary_stl(&asset_path(&service, &record_id)?, 350_000)?;

    let mut reopened = ProjectService::new();
    reopened.open(&fixture.root.join("Demo/Demo.panta"))?;
    let first = reopened.begin_asset_activation(&record_id)?;
    let second = reopened.begin_asset_activation(&record_id)?;
    assert_eq!(first.attempt, second.attempt, "同会话同记录必须去重");
    let outcome = wait_for_outcome(&mut reopened, Duration::from_secs(10))?;
    assert_eq!(outcome.kind, OutcomeKind::Succeeded);
    assert_eq!(outcome.attempt, first.attempt);
    assert!(
        reopened.drain_asset_activations().is_empty(),
        "去重后只有一个终态结果"
    );
    Ok(())
}

#[test]
fn manifest_asset_traversal_is_rejected_at_activation_admission()
-> Result<(), Box<dyn std::error::Error>> {
    use panta_core::project::ProjectError;

    let fixture = Fixture::new()?;
    let mut service = ProjectService::new();
    service.create(&fixture.root, "Demo")?;
    let record_id = import_sample_stl(&mut service, &fixture.root)?;
    service.save()?;
    let manifest_path = fixture.root.join("Demo/Demo.panta");
    drop(service);

    // 改写清单内的资产相对路径为越界 / 绝对引用：打开仍成功（只读清单），
    // 但激活提交边界必须拒绝，杜绝清单注入越出工程包目录。
    for evil in ["../../evil.stl", "/absolutely/external.stl"] {
        let manifest = format!(
            "{{\"schema\":2,\"name\":\"Demo\",\"revision\":1,\"imports\":[{{\"record_version\":1,\"parser_version\":2,\"id\":\"{record_id}\",\"source_name\":\"evil.stl\",\"asset\":\"{evil}\",\"format\":\"stl\",\"mesh_type\":\"solid-3d\",\"units\":\"millimeters\",\"show_import_log\":false,\"triangle_count\":1,\"dimensions\":[1.0,1.0,0.0]}}]}}"
        );
        fs::write(&manifest_path, &manifest)?;
        let mut reopened = ProjectService::new();
        reopened.open(&manifest_path)?;
        assert!(matches!(
            reopened.begin_asset_activation(&record_id),
            Err(ProjectError::ManifestInvalid(_))
        ));
    }
    Ok(())
}
