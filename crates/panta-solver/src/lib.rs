//! Moldfill 工作区分析：后台执行网格、浇口推荐与充填，按已提交产物发布结果。
mod result;

use panta_foundation::process::run_observed;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const CASE: &str = "examples/cover/cover_fast.case.yaml";
const CPU_SETTINGS: [(&str, &str); 7] = [
    ("MOLDFILL_DEVICE", "cpu"),
    ("MOLDFILL_PRESSURE_DEVICE", "cpu"),
    ("MOLDFILL_PRESSURE_SOLVER", "direct"),
    ("MOLDFILL_FAST_SOLVER", "1"),
    ("MOLDFILL_PICARD_MAX", "4"),
    ("MOLDFILL_CFL", "0.9"),
    ("MOLDFILL_DT_GROWTH", "0.5"),
];

/// 展开的三角面坐标 mm；字段与展开顶点一一对应。
#[derive(Default)]
pub struct DisplayMesh {
    pub coordinates: Vec<f64>,
    pub fill_times: Vec<f64>,
    pub pressures: Vec<f64>,
    pub gate_points: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Progress,
    Log,
    MeshReady,
    GateReady,
    Completed,
    Failed,
    Cancelled,
}

/// Progress / Log 非终态；其他事件完成当前操作。
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
    gate: Option<PathBuf>,
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
            gate: None,
        }
    }
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

/// 单活动零件的运行服务；GUI 线程提交 / 拉取，后台线程持有外部进程。
/// 重新划网格成功后使旧浇口失效；失败和取消保留上一有效产物。
pub struct AnalysisService {
    root: PathBuf,
    remeshed: Option<PathBuf>,
    gate: Option<PathBuf>,
    worker: Option<Worker>,
}
impl AnalysisService {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            remeshed: None,
            gate: None,
            worker: None,
        }
    }

    /// 请求 JSON 为单次冻结快照；参数校验和依赖检查在启动前完成。
    pub fn start(&mut self, operation: &str, settings: &str) -> Result<(), String> {
        if self.worker.is_some() {
            return Err("An analysis is already running".into());
        }
        let mode = match operation {
            "mesh" => "remesh_only",
            "gate" => "gate_only",
            "fill" => "solve",
            _ => return Err("Unknown analysis operation".into()),
        };
        let mut request: Value = serde_json::from_str(settings).map_err(|e| e.to_string())?;
        validate_settings(&request)?;
        if mode != "remesh_only" && self.remeshed.is_none() {
            return Err("Generate the mesh first".into());
        }
        if mode == "solve" && self.gate.is_none() {
            return Err("Run Gate Location Analyze first".into());
        }
        request["mode"] = mode.into();
        request["mesh"] = self
            .remeshed
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
            .into();
        request["gate"] = self
            .gate
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
            .into();
        let root = self.root.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let (sender, events) = mpsc::channel();
        let handle = std::thread::Builder::new()
            .name("moldfill-analysis".into())
            .spawn(move || {
                let start = Instant::now();
                let result = perform(&root, request, &flag, &sender);
                let mut event = if flag.load(Ordering::Acquire) {
                    Event::new(EventKind::Cancelled, "Analysis stopped")
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
    pub fn cancel(&self) {
        if let Some(worker) = &self.worker {
            worker.cancelled.store(true, Ordering::Release);
        }
    }

    /// 只消费就绪事件；终态释放已退出线程，不阻塞 GUI 等待计算。
    pub fn drain(&mut self) -> Vec<Event> {
        let Some(worker) = &self.worker else {
            return Vec::new();
        };
        let mut events = Vec::new();
        let mut terminal = false;
        loop {
            match worker.events.try_recv() {
                Ok(event) => {
                    terminal = !matches!(event.kind, EventKind::Progress | EventKind::Log);
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
                        "Analysis worker exited unexpectedly",
                    ));
                    break;
                }
            }
        }
        for event in &events {
            if event.kind == EventKind::MeshReady {
                self.remeshed.clone_from(&event.remeshed);
                self.gate = None;
            } else if event.kind == EventKind::GateReady {
                self.gate.clone_from(&event.gate);
            }
        }
        if terminal {
            self.worker = None;
        }
        events
    }
}

fn validate_settings(request: &Value) -> Result<(), String> {
    for key in ["source", "output"] {
        if request[key].as_str().is_none_or(str::is_empty) {
            return Err(format!("Missing {key}"));
        }
    }
    for key in [
        "edgeLength",
        "moldTemperature",
        "meltTemperature",
        "flowRate",
        "switchVolume",
    ] {
        if !request[key].as_f64().is_some_and(f64::is_finite) {
            return Err(format!("Invalid {key}"));
        }
    }
    if request["edgeLength"].as_f64().unwrap_or_default() <= 0.0
        || request["flowRate"].as_f64().unwrap_or_default() <= 0.0
        || !(0.0..=100.0).contains(&request["switchVolume"].as_f64().unwrap_or(-1.0))
    {
        return Err("Invalid mesh or fill control settings".into());
    }
    if request["numberOfGates"] != 1 {
        return Err("This solver integration supports one injection gate; top_k is a recommendation count, not multiple injection gates".into());
    }
    Ok(())
}

fn read_stl(path: &Path) -> Result<DisplayMesh, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mesh = panta_mesh::parse_stl(&bytes).map_err(|e| e.to_string())?;
    Ok(DisplayMesh {
        coordinates: mesh.triangles.into_iter().flatten().flatten().collect(),
        ..DisplayMesh::default()
    })
}

fn perform(
    root: &Path,
    mut request: Value,
    cancelled: &AtomicBool,
    sender: &mpsc::Sender<Event>,
) -> Result<Event, String> {
    let solver = root.join("target/Moldfill_HITL_v1");
    let python = root.join(if cfg!(windows) {
        "target/moldfill-venv/Scripts/python.exe"
    } else {
        "target/moldfill-venv/bin/python"
    });
    if !python.is_file() || !solver.join(CASE).is_file() {
        return Err("Solver environment is missing. Follow MVP使用说明书.md to prepare it.".into());
    }
    let parent = PathBuf::from(request["output"].as_str().ok_or("Missing output")?);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let output = parent.join(format!("run-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    request["output"] = output.to_string_lossy().into_owned().into();
    request["template"] = solver.join(CASE).to_string_lossy().into_owned().into();
    let request_path = output.join("request.json");
    fs::write(&request_path, request.to_string()).map_err(|e| e.to_string())?;
    let log = File::create(output.join("console.log")).map_err(|e| e.to_string())?;
    let mut command = Command::new(python);
    command
        .current_dir(&solver)
        .arg("-u")
        .arg(root.join("tools/moldfill/driver.py"))
        .arg(request_path)
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
    let mut reader = EventReader::default();
    let mut console_offset = 0;
    let started = Instant::now();
    let mut observe = || {
        for mut event in reader.read(&output) {
            event.elapsed = started.elapsed().as_secs_f64();
            event.output_dir = output.to_string_lossy().into_owned();
            let _ = sender.send(event);
        }
        if let Ok(bytes) = fs::read(output.join("console.log"))
            && bytes.len() > console_offset
        {
            let end = bytes
                .iter()
                .rposition(|b| *b == b'\n')
                .map_or(console_offset, |p| p + 1);
            if end > console_offset {
                let _ = sender.send(Event::new(
                    EventKind::Log,
                    String::from_utf8_lossy(&bytes[console_offset..end]).into_owned(),
                ));
                console_offset = end;
            }
        }
    };
    let status = run_observed(&mut command, cancelled, &mut observe)
        .map_err(|e| format!("Cannot run solver: {e}"))?;
    observe();
    if !status.success() {
        return Err(format!(
            "Solver exited with {status}. See console.log in {}",
            output.display()
        ));
    }
    let mode = request["mode"].as_str().ok_or("Missing mode")?;
    let run = result::committed_run(&output, mode)?;
    let mut event = match mode {
        "remesh_only" => {
            let path = result::artifact(&run, "_remeshed.STL")?;
            let mut e = Event::new(EventKind::MeshReady, "Mesh generated");
            e.mesh = read_stl(&path)?;
            e.remeshed = Some(path);
            e
        }
        "gate_only" => {
            let path = result::artifact(&run, "gate_recommend.yaml")?;
            let mut e = Event::new(
                EventKind::GateReady,
                fs::read_to_string(&path).map_err(|e| e.to_string())?,
            );
            e.gate = Some(path);
            let positions: Vec<f64> = serde_json::from_slice(
                &fs::read(output.join("gate-display.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            if positions.len() != 3 || positions.iter().any(|p| !p.is_finite()) {
                return Err("Invalid selected gate position".into());
            }
            e.mesh.gate_points = positions;
            e
        }
        _ => {
            let summary = fs::read_to_string(result::artifact(&run, "result.yaml")?)
                .map_err(|e| e.to_string())?;
            let (time, pressure) = result::summary(&summary)?;
            let text = fs::read_to_string(result::artifact(&run, "fill_pattern.vtk")?)
                .map_err(|e| e.to_string())?;
            let mut e = Event::new(EventKind::Completed, "Filling complete");
            e.mesh = result::read_vtk(&text)?;
            e.fill_time = time;
            e.peak_pressure = pressure;
            e
        }
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
                Some("progress" | "stage_enter" | "artifact_written" | "warning" | "error")
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
        let mut service = AnalysisService::new(PathBuf::from("missing"));
        assert!(service.start("gate", "{}").is_err());
        assert!(service.start("fill", "{}").is_err());
    }
    #[test]
    fn invalid_settings_and_multiple_injection_gates_are_rejected() {
        let mut request = serde_json::json!({"source":"cover.STL", "output":"result",
            "edgeLength":12.0, "moldTemperature":40.0, "meltTemperature":230.0,
            "flowRate":94.7, "switchVolume":99.0, "numberOfGates":1});
        assert!(validate_settings(&request).is_ok());
        request["numberOfGates"] = 3.into();
        assert!(validate_settings(&request).is_err());
        request["numberOfGates"] = 1.into();
        request["edgeLength"] = (-1).into();
        assert!(validate_settings(&request).is_err());
    }

    /// 需要本机外部求解器和 Python 环境；不纳入普通回归的耗时门禁。
    #[test]
    #[ignore = "requires target/Moldfill_HITL_v1 and prepared Python environment"]
    fn cover_mesh_gate_and_fill_are_real_solver_outputs() -> Result<(), String> {
        cover_integration(true)
    }

    #[test]
    #[ignore = "requires the external solver; quick mesh and gate verification"]
    fn cover_gate_marker_comes_from_solver_recommendation() -> Result<(), String> {
        cover_integration(false)
    }

    fn cover_integration(include_fill: bool) -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let output = root.join("target/moldfill-integration");
        let request = serde_json::json!({"source":root.join("target/Moldfill_HITL_v1/examples/cover/cover.STL"),
            "output":output, "edgeLength":12.0, "moldTemperature":40.0, "meltTemperature":230.0,
            "flowRate":94.7, "switchVolume":99.0, "numberOfGates":1});
        let mut service = AnalysisService::new(root);
        for (operation, expected) in [
            ("mesh", EventKind::MeshReady),
            ("gate", EventKind::GateReady),
            ("fill", EventKind::Completed),
        ] {
            if operation == "fill" && !include_fill {
                break;
            }
            service.start(operation, &request.to_string())?;
            let deadline = Instant::now() + std::time::Duration::from_secs(900);
            loop {
                let mut done = false;
                for event in service.drain() {
                    if matches!(event.kind, EventKind::Progress | EventKind::Log) {
                        continue;
                    }
                    if event.kind != expected {
                        return Err(event.message);
                    }
                    println!("{operation}: {}", event.output_dir);
                    if expected == EventKind::GateReady {
                        assert_eq!(event.mesh.gate_points.len(), 3);
                        assert!(event.mesh.gate_points.iter().all(|p| p.is_finite()));
                    }
                    if expected == EventKind::Completed {
                        assert!(event.fill_time > 0.0 && event.peak_pressure > 0.0);
                        assert_eq!(
                            event.mesh.coordinates.len(),
                            event.mesh.fill_times.len() * 3
                        );
                        assert_eq!(event.mesh.pressures.len(), event.mesh.fill_times.len());
                        assert!(
                            event
                                .mesh
                                .pressures
                                .iter()
                                .any(|p| p.is_finite() && *p > 0.0)
                        );
                        println!(
                            "fill time={} s, peak pressure={} MPa",
                            event.fill_time, event.peak_pressure
                        );
                    }
                    done = true;
                }
                if done {
                    break;
                }
                if Instant::now() > deadline {
                    service.cancel();
                    return Err("Integration timed out".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        Ok(())
    }
    #[test]
    #[ignore = "requires external solver; optionally set PANTA_SOLVER_TEST_SOURCE"]
    fn imported_lowercase_stl_mesh_is_displayed() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let parent = root.join("target/moldfill-lowercase-regression");
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        let source = if let Some(path) = std::env::var_os("PANTA_SOLVER_TEST_SOURCE") {
            PathBuf::from(path)
        } else {
            let path = parent.join("cover.stl");
            fs::copy(
                root.join("target/Moldfill_HITL_v1/examples/cover/cover.STL"),
                &path,
            )
            .map_err(|e| e.to_string())?;
            path
        };
        let request = serde_json::json!({"source":source, "output":parent, "mode":"remesh_only",
            "edgeLength":12.0, "moldTemperature":40.0, "meltTemperature":230.0,
            "flowRate":94.7, "switchVolume":99.0, "numberOfGates":1});
        let (sender, _events) = mpsc::channel();
        let event = perform(&root, request, &AtomicBool::new(false), &sender)?;
        assert!(event.kind == EventKind::MeshReady);
        assert!(!event.mesh.coordinates.is_empty());
        println!(
            "{} triangles; {}",
            event.mesh.coordinates.len() / 9,
            event.output_dir
        );
        Ok(())
    }
}
