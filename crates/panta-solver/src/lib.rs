//! 固定 cover 充填演示：工作线程管理输入、外部进程和结果，GUI 只拉取事件。
//! 默认输入及实施记录见仓库根 preview.md；不作为任意求解器的通用协议。

mod result;

use panta_foundation::process::run_observed;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const CASE: &str = "benchmarks/cover_noniso_quick.case.yaml";
// 固定示例与 GUI 只读参数必须一致；改用其他输入需要重新验证本演示。
const FIXED_INPUTS: [(&str, &str); 3] = [
    (
        "benchmarks/cover_noniso_quick.case.yaml",
        "d62605cd253c0a14d63c5d85e54ee4171afbe0200d70eacea3d88edfe2e1cbe9",
    ),
    (
        "examples/cover/cover.STL",
        "4102bfe5acccad37ca3a0eed80ddd43997a7dcf42a518cfd58398fb53a5c227f",
    ),
    (
        "materials/polyflam-ripp-3625-cs1.yaml",
        "f516ce22ed2690dd0051c256a678940a52ef0753443bfd3814db234b01281f17",
    ),
];
const CPU_SETTINGS: [(&str, &str); 7] = [
    ("MOLDFILL_DEVICE", "cpu"),
    ("MOLDFILL_PRESSURE_DEVICE", "cpu"),
    ("MOLDFILL_PRESSURE_SOLVER", "direct"),
    ("MOLDFILL_FAST_SOLVER", "1"),
    ("MOLDFILL_PICARD_MAX", "4"),
    ("MOLDFILL_CFL", "0.9"),
    ("MOLDFILL_DT_GROWTH", "0.5"),
];

/// 展开后的三角面坐标 mm；fill_times 为空或与顶点一一对应，单位 s。
#[derive(Default)]
pub struct DisplayMesh {
    pub coordinates: Vec<f64>,
    pub fill_times: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Progress,
    ModelReady,
    MeshReady,
    Completed,
    Failed,
    Cancelled,
}

/// 每个终态最多携带一次显示快照。非 Progress 事件结束当前工作线程。
pub struct Event {
    pub kind: EventKind,
    pub message: String,
    pub progress: f64,
    pub elapsed: f64,
    pub fill_time: f64,
    pub peak_pressure: f64,
    pub output_dir: String,
    pub mesh: DisplayMesh,
    remeshed: Option<PathBuf>,
}

impl Event {
    fn new(kind: EventKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            progress: -1.0,
            elapsed: 0.0,
            fill_time: 0.0,
            peak_pressure: 0.0,
            output_dir: String::new(),
            mesh: DisplayMesh::default(),
            remeshed: None,
        }
    }
}

#[derive(Clone, Copy)]
enum Operation {
    Load,
    Remesh,
    Fill,
}

struct Worker {
    cancelled: Arc<AtomicBool>,
    events: mpsc::Receiver<Event>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// 单作业演示服务。重复提交被拒绝；释放服务时停止并回收求解器。
/// 所有方法由同一宿主线程调用；只有工作线程接触进程和输入/结果文件。
pub struct PreviewService {
    root: PathBuf,
    loaded: bool,
    remeshed: Option<PathBuf>,
    worker: Option<Worker>,
}

impl PreviewService {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            loaded: false,
            remeshed: None,
            worker: None,
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        self.start(Operation::Load)
    }

    pub fn remesh(&mut self) -> Result<(), String> {
        if !self.loaded {
            return Err("Load the default example first".into());
        }
        self.start(Operation::Remesh)
    }

    pub fn fill(&mut self) -> Result<(), String> {
        if self.remeshed.is_none() {
            return Err("Generate the mesh first".into());
        }
        self.start(Operation::Fill)
    }

    pub fn cancel(&self) {
        if let Some(worker) = &self.worker {
            worker.cancelled.store(true, Ordering::Release);
        }
    }

    fn start(&mut self, operation: Operation) -> Result<(), String> {
        if self.worker.is_some() {
            return Err("A preview operation is already running".into());
        }
        let root = self.root.clone();
        let remeshed = self.remeshed.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let (sender, events) = mpsc::channel();
        let handle = std::thread::Builder::new()
            .name("moldfill-preview".into())
            .spawn(move || {
                let start = Instant::now();
                let result = perform(&root, operation, remeshed.as_deref(), &flag, &sender);
                let mut event = if flag.load(Ordering::Acquire) {
                    Event::new(EventKind::Cancelled, "Stopped. You can retry this step.")
                } else {
                    result.unwrap_or_else(|e| Event::new(EventKind::Failed, e))
                };
                event.elapsed = start.elapsed().as_secs_f64();
                let _ = sender.send(event);
            })
            .map_err(|e| e.to_string())?;
        self.worker = Some(Worker {
            cancelled,
            events,
            handle: Some(handle),
        });
        Ok(())
    }

    /// 非阻塞拉取。终态更新服务的可执行步骤，并回收已退出的工作线程。
    pub fn drain(&mut self) -> Vec<Event> {
        let Some(worker) = &self.worker else {
            return Vec::new();
        };
        let mut events = Vec::new();
        let mut terminal = false;
        loop {
            match worker.events.try_recv() {
                Ok(event) => {
                    terminal |= event.kind != EventKind::Progress;
                    events.push(event);
                    if terminal {
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    terminal = true;
                    events.push(Event::new(
                        EventKind::Failed,
                        "Preview worker exited unexpectedly",
                    ));
                    break;
                }
            }
        }
        for event in &events {
            match event.kind {
                EventKind::ModelReady => {
                    self.loaded = true;
                    self.remeshed = None;
                }
                EventKind::MeshReady => self.remeshed.clone_from(&event.remeshed),
                _ => {}
            }
        }
        if terminal {
            self.worker = None;
        }
        events
    }
}

fn read_stl(path: &Path) -> Result<DisplayMesh, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mesh = panta_mesh::parse_stl(&bytes).map_err(|e| e.to_string())?;
    Ok(DisplayMesh {
        coordinates: mesh.triangles.into_iter().flatten().flatten().collect(),
        fill_times: Vec::new(),
    })
}

fn unique_output(root: &Path) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let parent = root.join("target/moldfill-preview");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let output = parent.join(format!("gui-{}-{stamp}", std::process::id()));
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    Ok(output)
}

fn perform(
    root: &Path,
    operation: Operation,
    mesh: Option<&Path>,
    cancelled: &AtomicBool,
    sender: &mpsc::Sender<Event>,
) -> Result<Event, String> {
    let solver = root.join("target/Moldfill_HITL_v1");
    for (relative, expected) in FIXED_INPUTS {
        let bytes = fs::read(solver.join(relative))
            .map_err(|e| format!("Default input {relative}: {e}"))?;
        if format!("{:x}", Sha256::digest(bytes)) != expected {
            return Err(format!(
                "Default input changed: {relative}. Restore the validated example described in MVP使用说明书.md."
            ));
        }
    }
    if matches!(operation, Operation::Load) {
        let mut event = Event::new(EventKind::ModelReady, "Cover loaded · 202 × 6 × 152 mm");
        event.mesh = read_stl(&solver.join("examples/cover/cover.STL"))?;
        return Ok(event);
    }
    let python = root.join(if cfg!(windows) {
        "target/moldfill-venv/Scripts/python.exe"
    } else {
        "target/moldfill-venv/bin/python"
    });
    if !python.is_file() || !solver.join(CASE).is_file() {
        return Err("Solver environment is missing. Follow MVP使用说明书.md to prepare it.".into());
    }
    let output = unique_output(root)?;
    let log = File::create(output.join("console.log")).map_err(|e| e.to_string())?;
    let mut command = Command::new(python);
    command
        .current_dir(&solver)
        .args(["-u", "-m", "moldfill", "run", CASE, "--out"])
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("MOLDFILL_") {
            command.env_remove(name);
        }
    }
    command
        .envs(CPU_SETTINGS)
        .env("PYTHONPATH", solver.join("src"));
    let mode = if matches!(operation, Operation::Remesh) {
        command.arg("--remesh-only");
        "remesh_only"
    } else {
        command.arg("--mesh").arg(mesh.ok_or("No remeshed input")?);
        "solve"
    };
    let settings = serde_json::json!({"case": CASE, "environment": CPU_SETTINGS, "mode":mode});
    fs::write(output.join("preview-settings.json"), settings.to_string())
        .map_err(|e| e.to_string())?;
    let mut reader = EventReader::default();
    let started = Instant::now();
    let status = run_observed(&mut command, cancelled, || {
        for mut event in reader.read(&output) {
            event.elapsed = started.elapsed().as_secs_f64();
            event.output_dir = output.to_string_lossy().into_owned();
            let _ = sender.send(event);
        }
    })
    .map_err(|e| format!("Cannot run solver: {e}\n{}", output.display()))?;
    if !status.success() {
        let log = fs::read_to_string(output.join("console.log")).unwrap_or_default();
        let tail = log
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "Solver exited with {status}.\n{tail}\nRun: {}",
            output.display()
        ));
    }
    let run = result::committed_run(&output, mode)?;
    let mut event = if matches!(operation, Operation::Remesh) {
        let path = result::artifact(&run, "_remeshed.STL")?;
        let mut event = Event::new(
            EventKind::MeshReady,
            "Mesh ready · default material and process applied",
        );
        event.mesh = read_stl(&path)?;
        event.remeshed = Some(path);
        event
    } else {
        let summary = fs::read_to_string(result::artifact(&run, "result.yaml")?)
            .map_err(|e| e.to_string())?;
        let (time, pressure) = result::summary(&summary)?;
        let text = fs::read_to_string(result::artifact(&run, "fill_pattern.vtk")?)
            .map_err(|e| e.to_string())?;
        let mut event = Event::new(EventKind::Completed, "Filling complete · ready to replay");
        event.mesh = result::read_vtk(&text)?;
        event.fill_time = time;
        event.peak_pressure = pressure;
        event
    };
    event.progress = 1.0;
    event.output_dir = run.to_string_lossy().into_owned();
    Ok(event)
}

/// 只消费完整 JSONL 行；staging 重命名后继续使用同一偏移，不重复事件。
#[derive(Default)]
struct EventReader {
    offset: u64,
    pending: Vec<u8>,
}

impl EventReader {
    fn read(&mut self, output: &Path) -> Vec<Event> {
        let Ok(entries) = fs::read_dir(output.join("runs")) else {
            return Vec::new();
        };
        let Some(path) = entries
            .filter_map(Result::ok)
            .map(|e| e.path().join("logs/events.jsonl"))
            .find(|p| p.is_file())
        else {
            return Vec::new();
        };
        let Ok(mut file) = File::open(path) else {
            return Vec::new();
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return Vec::new();
        }
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_err() {
            return Vec::new();
        }
        self.offset += bytes.len() as u64;
        self.pending.extend(bytes);
        let mut events = Vec::new();
        while let Some(end) = self.pending.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let Ok(row) = serde_json::from_slice::<Value>(&line) else {
                continue;
            };
            if matches!(
                row["event"].as_str(),
                Some("progress" | "stage_enter" | "warning" | "error")
            ) {
                let mut event = Event::new(
                    EventKind::Progress,
                    row["message_zh"].as_str().unwrap_or("Working"),
                );
                event.progress = row["data"]["vol_frac"]
                    .as_f64()
                    .filter(|v| v.is_finite())
                    .unwrap_or(-1.0)
                    .clamp(-1.0, 1.0);
                events.push(event);
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct TempDir(pub PathBuf);
    impl TempDir {
        pub fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("panta-preview-{}-{stamp}", std::process::id())))
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn split_utf8_and_staging_rename_do_not_lose_or_duplicate_events()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new();
        let staging = temp.0.join("runs/id.staging");
        fs::create_dir_all(staging.join("logs"))?;
        let log = staging.join("logs/events.jsonl");
        let text = "{\"event\":\"progress\",\"message_zh\":\"充填\",\"data\":{\"vol_frac\":0.4}}\n";
        let bytes = text.as_bytes();
        let split = text.find('充').ok_or("missing test text")? + 1;
        fs::write(&log, &bytes[..split])?;
        let mut reader = EventReader::default();
        assert!(reader.read(&temp.0).is_empty());
        fs::write(&log, bytes)?;
        fs::rename(staging, temp.0.join("runs/id"))?;
        let events = reader.read(&temp.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].message, "充填");
        assert_eq!(events[0].progress, 0.4);
        assert!(reader.read(&temp.0).is_empty());
        Ok(())
    }

    #[test]
    fn steps_require_their_inputs() {
        let mut service = PreviewService::new(PathBuf::from("missing"));
        assert!(service.remesh().is_err());
        assert!(service.fill().is_err());
    }

    #[test]
    fn missing_source_fails_without_replacing_state() -> Result<(), String> {
        let mut service = PreviewService::new(PathBuf::from("missing"));
        service.load()?;
        assert!(service.load().is_err());
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let events = service.drain();
            if !events.is_empty() {
                assert!(events.iter().any(|e| e.kind == EventKind::Failed));
                assert!(!service.loaded);
                return Ok(());
            }
            if Instant::now() > deadline {
                return Err("Timed out".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
