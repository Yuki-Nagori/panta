//! Moldfill 工作区分析：后台执行网格、浇口推荐与充填，按已提交产物发布结果。
mod cache;
mod result;

use cache::{FillParams, PartCache};
use panta_foundation::process::run_observed;
use serde_json::Value;
use std::collections::HashMap;
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
#[derive(Clone, Default)]
pub struct DisplayMesh {
    pub coordinates: Vec<f64>,
    pub fill_times: Vec<f64>,
    pub pressures: Vec<f64>,
    pub gate_points: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    Progress,
    Log,
    MeshReady,
    GateReady,
    Completed,
    Failed,
    Cancelled,
    /// 输入与已有成功产物一致，未启动求解器。
    Restored,
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
    /// 规范化后的分析目录。界面用它忽略已切走零件的迟到事件。
    pub scope: String,
    pub from_cache: bool,
    pub has_mesh: bool,
    pub has_gate: bool,
    pub has_fill: bool,
    source: Option<PathBuf>,
    artifact: Option<PathBuf>,
    edge_length: f64,
    mold: f64,
    melt: f64,
    flow: f64,
    switch_volume: f64,
    number_of_gates: i64,
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
            scope: String::new(),
            from_cache: false,
            has_mesh: false,
            has_gate: false,
            has_fill: false,
            source: None,
            artifact: None,
            edge_length: 0.0,
            mold: 0.0,
            melt: 0.0,
            flow: 0.0,
            switch_volume: 0.0,
            number_of_gates: 0,
        }
    }
}
struct Worker {
    cancelled: Arc<AtomicBool>,
    events: mpsc::Receiver<Event>,
    handle: Option<JoinHandle<()>>,
    scope: String,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// 按零件分析目录记住网格、浇口和充填。GUI 线程提交 / 拉取，后台线程持有外部进程。
/// 相同输入直接复用缓存。重新划网格成功后使旧浇口和充填失效；失败和取消保留上一有效产物。
pub struct AnalysisService {
    root: PathBuf,
    parts: HashMap<PathBuf, PartCache>,
    active: Option<PathBuf>,
    pending: Vec<Event>,
    worker: Option<Worker>,
}
impl AnalysisService {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            parts: HashMap::new(),
            active: None,
            pending: Vec::new(),
            worker: None,
        }
    }

    /// 绑定分析目录并返回已有显示数据。目录不存在时只建立空的内存项，不创建目录。
    pub fn bind(&mut self, output: &Path) -> cache::CachedView {
        let root = cache::normalize_root(output);
        let view = self
            .parts
            .entry(root.clone())
            .or_insert_with(|| PartCache::load(&root))
            .view();
        self.active = Some(root);
        view
    }

    /// 请求 JSON 为单次冻结快照；参数校验和依赖检查在启动前完成。
    /// 与缓存一致时把 `Restored` 放进待取队列，不启动求解器。
    pub fn start(&mut self, operation: &str, settings: &str) -> Result<(), String> {
        if self.worker.is_some() {
            return Err("An analysis is already running".into());
        }
        // 丢掉尚未取走的缓存命中。否则下一次 drain 只返回它，正在跑的求解器不再被轮询。
        self.pending.clear();
        let mode = match operation {
            "mesh" => "remesh_only",
            "gate" => "gate_only",
            "fill" => "solve",
            _ => return Err("Unknown analysis operation".into()),
        };
        let mut request: Value = serde_json::from_str(settings).map_err(|e| e.to_string())?;
        validate_settings(&request)?;
        let output = PathBuf::from(request["output"].as_str().ok_or("Missing output")?);
        let bound = self.bind(&output);
        request["output"] = bound.root.clone().into();
        if self.matches_request(mode, &request) {
            let event = self.restored_event();
            self.pending.push(event);
            return Ok(());
        }
        let (mesh, gate, ready) = self.dependency_paths()?;
        if mode != "remesh_only" && !ready.0 {
            return Err("Generate the mesh first".into());
        }
        if mode == "solve" && !ready.1 {
            return Err("Run Gate Location Analyze first".into());
        }
        request["mode"] = mode.into();
        request["mesh"] = mesh.into();
        request["gate"] = gate.into();
        let scope = bound.root;
        let root = self.root.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let (sender, events) = mpsc::channel();
        let worker_scope = scope.clone();
        let handle = std::thread::Builder::new()
            .name("moldfill-analysis".into())
            .spawn(move || {
                let started = Instant::now();
                let result = perform(&root, request, &flag, &sender);
                let mut event = if flag.load(Ordering::Acquire) {
                    Event::new(EventKind::Cancelled, "Analysis stopped")
                } else {
                    result.unwrap_or_else(|error| Event::new(EventKind::Failed, error))
                };
                if event.scope.is_empty() {
                    event.scope = scope;
                }
                event.elapsed = started.elapsed().as_secs_f64();
                let _ = sender.send(event);
            })
            .map_err(|e| e.to_string())?;
        self.worker = Some(Worker {
            cancelled,
            events,
            handle: Some(handle),
            scope: worker_scope,
        });
        Ok(())
    }
    pub fn cancel(&self) {
        if let Some(worker) = &self.worker {
            worker.cancelled.store(true, Ordering::Release);
        }
    }

    /// 先交还缓存命中，再消费求解器事件。终态写入对应零件的缓存；失败和取消不覆盖。
    pub fn drain(&mut self) -> Vec<Event> {
        if !self.pending.is_empty() {
            return std::mem::take(&mut self.pending);
        }
        let Some(worker) = &self.worker else {
            return Vec::new();
        };
        let scope = worker.scope.clone();
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
                    let mut event =
                        Event::new(EventKind::Failed, "Analysis worker exited unexpectedly");
                    event.scope = scope;
                    events.push(event);
                    break;
                }
            }
        }
        if terminal {
            self.worker = None;
        }
        for event in &events {
            self.absorb_success(event);
        }
        events
    }

    fn matches_request(&self, mode: &str, request: &Value) -> bool {
        let Some(part) = self.active_part() else {
            return false;
        };
        let source = Path::new(request["source"].as_str().unwrap_or(""));
        let edge = request["edgeLength"].as_f64().unwrap_or(0.0);
        let gates = request["numberOfGates"].as_i64().unwrap_or(0);
        let mold = request["moldTemperature"].as_f64().unwrap_or(0.0);
        let melt = request["meltTemperature"].as_f64().unwrap_or(0.0);
        let flow = request["flowRate"].as_f64().unwrap_or(0.0);
        let switch_volume = request["switchVolume"].as_f64().unwrap_or(0.0);
        match mode {
            "remesh_only" => part.matches_mesh(source, edge),
            "gate_only" => part.matches_gate(gates, mold, melt),
            _ => part.matches_fill(mold, melt, flow, switch_volume),
        }
    }

    fn dependency_paths(&self) -> Result<(String, String, (bool, bool)), String> {
        let part = self.active_part().ok_or("Missing analysis output")?;
        Ok((
            part.mesh_path(),
            part.gate_path(),
            (part.has_mesh(), part.has_gate()),
        ))
    }

    fn restored_event(&self) -> Event {
        let Some(part) = self.active_part() else {
            return Event::new(EventKind::Failed, "Missing analysis output");
        };
        let view = part.view();
        let mut event = Event::new(EventKind::Restored, "Restored from cache");
        event.scope.clone_from(&view.root);
        event.output_dir.clone_from(&view.root);
        event.from_cache = true;
        event.has_mesh = view.has_mesh;
        event.has_gate = view.has_gate;
        event.has_fill = view.has_fill;
        event.fill_time = view.fill_time;
        event.peak_pressure = view.peak_pressure;
        event.progress = 1.0;
        event.mesh.coordinates = view.coordinates;
        event.mesh.fill_times = view.fill_times;
        event.mesh.pressures = view.pressures;
        event.mesh.gate_points = view.gate_points;
        event
    }

    fn absorb_success(&mut self, event: &Event) {
        if event.scope.is_empty() {
            return;
        }
        let Some(part) = self.parts.get_mut(Path::new(&event.scope)) else {
            return;
        };
        let saved = match event.kind {
            EventKind::MeshReady => {
                let (Some(source), Some(stl)) = (&event.source, &event.artifact) else {
                    return;
                };
                part.absorb_mesh(source, event.edge_length, stl, event.mesh.clone())
            }
            EventKind::GateReady => {
                let Some(yaml) = &event.artifact else {
                    return;
                };
                part.absorb_gate(
                    yaml,
                    event.number_of_gates,
                    event.mold,
                    event.melt,
                    event.mesh.gate_points.clone(),
                )
            }
            EventKind::Completed => {
                let Some(vtk) = &event.artifact else {
                    return;
                };
                part.absorb_fill(
                    vtk,
                    &FillParams {
                        mold: event.mold,
                        melt: event.melt,
                        flow: event.flow,
                        switch_volume: event.switch_volume,
                        fill_time: event.fill_time,
                        peak_pressure: event.peak_pressure,
                    },
                    event.mesh.clone(),
                )
            }
            _ => return,
        };
        // 写入失败时保留上一份磁盘缓存；本次界面结果仍由事件发布。
        let _ = saved;
    }

    fn active_part(&self) -> Option<&PartCache> {
        self.active.as_ref().and_then(|key| self.parts.get(key))
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

pub(crate) fn read_stl(path: &Path) -> Result<DisplayMesh, String> {
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
    let scope = request["output"]
        .as_str()
        .ok_or("Missing output")?
        .to_owned();
    let parent = PathBuf::from(&scope);
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
    let mut console_offset: usize = 0;
    let started = Instant::now();
    let console_path = output.join("console.log");
    let mut observe = || {
        for mut event in reader.read(&output) {
            event.elapsed = started.elapsed().as_secs_f64();
            event.output_dir = output.to_string_lossy().into_owned();
            event.scope.clone_from(&scope);
            let _ = sender.send(event);
        }
        // 只读上次偏移之后的完整行，避免每次轮询重读整份控制台。
        publish_console(&console_path, &mut console_offset, sender, &scope);
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
            e.artifact = Some(path);
            e
        }
        "gate_only" => {
            let path = result::artifact(&run, "gate_recommend.yaml")?;
            let mut e = Event::new(
                EventKind::GateReady,
                fs::read_to_string(&path).map_err(|e| e.to_string())?,
            );
            e.artifact = Some(path);
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
            let vtk = result::artifact(&run, "fill_pattern.vtk")?;
            let text = fs::read_to_string(&vtk).map_err(|e| e.to_string())?;
            let mut e = Event::new(EventKind::Completed, "Filling complete");
            e.mesh = result::read_vtk(&text)?;
            e.fill_time = time;
            e.peak_pressure = pressure;
            e.artifact = Some(vtk);
            e
        }
    };
    stamp_request(&mut event, &request, &scope);
    event.progress = 1.0;
    event.output_dir = run.to_string_lossy().into_owned();
    Ok(event)
}

fn stamp_request(event: &mut Event, request: &Value, scope: &str) {
    event.scope = scope.to_owned();
    event.edge_length = request["edgeLength"].as_f64().unwrap_or(0.0);
    event.mold = request["moldTemperature"].as_f64().unwrap_or(0.0);
    event.melt = request["meltTemperature"].as_f64().unwrap_or(0.0);
    event.flow = request["flowRate"].as_f64().unwrap_or(0.0);
    event.switch_volume = request["switchVolume"].as_f64().unwrap_or(0.0);
    event.number_of_gates = request["numberOfGates"].as_i64().unwrap_or(0);
    if let Some(source) = request["source"].as_str() {
        event.source = Some(PathBuf::from(source));
    }
}

fn publish_console(path: &Path, offset: &mut usize, sender: &mpsc::Sender<Event>, scope: &str) {
    let Ok(mut file) = File::open(path) else {
        return;
    };
    let Ok(start) = u64::try_from(*offset) else {
        return;
    };
    if file.seek(SeekFrom::Start(start)).is_err() {
        return;
    }
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return;
    }
    let Some(end) = bytes.iter().rposition(|byte| *byte == b'\n') else {
        return;
    };
    let end = end + 1;
    let mut event = Event::new(
        EventKind::Log,
        String::from_utf8_lossy(&bytes[..end]).into_owned(),
    );
    event.scope = scope.to_owned();
    let _ = sender.send(event);
    *offset += end;
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

    const TRIANGLE: &str = "solid x\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
    const FILL_VTK: &str = "# vtk DataFile Version 4.2\npreview (length unit: mm)\nASCII\nDATASET UNSTRUCTURED_GRID\nPOINTS 3 double\n0 0 0 1 0 0 0 1 0\nCELLS 1 4\n3 2 0 1\nCELL_TYPES 1\n7\nPOINT_DATA 3\nSCALARS fill_time_s double 1\nLOOKUP_TABLE default\n0 1 NaN\nSCALARS pressure_Pa double 1\nLOOKUP_TABLE default\n1000000 2000000 NaN\n";

    fn request_json(source: &Path, output: &Path, edge: f64, mold: f64) -> String {
        serde_json::json!({
            "source": source, "output": output, "edgeLength": edge,
            "moldTemperature": mold, "meltTemperature": 230.0,
            "flowRate": 94.7, "switchVolume": 99.0, "numberOfGates": 1
        })
        .to_string()
    }

    fn seed_part(service: &mut AnalysisService, output: &Path) -> Result<PathBuf, String> {
        let root = PathBuf::from(service.bind(output).root);
        fs::create_dir_all(root.join("mesh")).map_err(|error| error.to_string())?;
        fs::create_dir_all(root.join("gate")).map_err(|error| error.to_string())?;
        fs::create_dir_all(root.join("fill")).map_err(|error| error.to_string())?;
        let source = root.join("source.stl");
        let stl = root.join("mesh/part_remeshed.stl");
        let yaml = root.join("gate/gate_recommend.yaml");
        let vtk = root.join("fill/fill_pattern.vtk");
        fs::write(&source, TRIANGLE).map_err(|error| error.to_string())?;
        fs::write(&stl, TRIANGLE).map_err(|error| error.to_string())?;
        fs::write(&yaml, "gate: 1\n").map_err(|error| error.to_string())?;
        fs::write(&vtk, FILL_VTK).map_err(|error| error.to_string())?;
        let key = service.active.clone().ok_or("Missing analysis output")?;
        {
            let part = service
                .parts
                .get_mut(&key)
                .ok_or("Missing analysis output")?;
            part.absorb_mesh(&source, 12.0, &stl, read_stl(&stl)?)?;
            part.absorb_gate(&yaml, 1, 40.0, 230.0, vec![1.5, 2.5, 3.5])?;
            part.absorb_fill(
                &vtk,
                &FillParams {
                    mold: 40.0,
                    melt: 230.0,
                    flow: 94.7,
                    switch_volume: 99.0,
                    fill_time: 1.9,
                    peak_pressure: 2.1,
                },
                result::read_vtk(FILL_VTK)?,
            )?;
        }
        Ok(root)
    }

    fn terminal_event(service: &mut AnalysisService) -> Result<Event, String> {
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        loop {
            for event in service.drain() {
                if !matches!(event.kind, EventKind::Progress | EventKind::Log) {
                    return Ok(event);
                }
            }
            if Instant::now() > deadline {
                return Err("timed out waiting for analysis".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn cached_parts_restore_without_the_solver_and_keep_failures() -> Result<(), String> {
        let temp = TempDir::new();
        let output = temp.0.join("analyses").join("part");
        let mut service = AnalysisService::new(temp.0.clone());
        assert!(!service.bind(&output).has_mesh);
        assert!(!output.exists());
        let root = seed_part(&mut service, &output)?;
        let source = root.join("source.stl");
        let warm = service.bind(&output);
        assert_eq!(warm.root, root.to_string_lossy());
        assert!(warm.has_fill);
        assert_eq!(warm.gate_points, vec![1.5, 2.5, 3.5]);
        let mesh_request = request_json(&source, &output, 12.0, 40.0);
        service.start("mesh", &mesh_request)?;
        let restored = service.drain();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].kind, EventKind::Restored);
        assert!(restored[0].from_cache && restored[0].has_fill);
        assert_eq!(restored[0].scope, warm.root);
        assert!(service.drain().is_empty());

        service.start("mesh", &request_json(&source, &output, 8.0, 40.0))?;
        let failed = terminal_event(&mut service)?;
        assert_eq!(failed.kind, EventKind::Failed);
        assert!(service.bind(&output).has_fill);

        service.start("gate", &mesh_request)?;
        assert_eq!(service.drain()[0].kind, EventKind::Restored);
        service.start("fill", &mesh_request)?;
        let fill = service.drain();
        assert_eq!(fill[0].kind, EventKind::Restored);
        assert!(!fill[0].mesh.coordinates.is_empty());
        assert!(fill[0].mesh.pressures.contains(&1.0));

        service.start("gate", &mesh_request)?;
        service.start("mesh", &request_json(&source, &output, 8.0, 40.0))?;
        let replaced = terminal_event(&mut service)?;
        assert_eq!(replaced.kind, EventKind::Failed);
        assert!(service.bind(&output).has_fill);

        let changed = format!("{TRIANGLE}vertex 0 0 0\nvertex 2 0 0\nvertex 0 4 0\n");
        fs::write(root.join("mesh/part_remeshed.stl"), changed)
            .map_err(|error| error.to_string())?;
        let memory = service.bind(&output);
        assert_eq!(memory.coordinates, warm.coordinates);
        service.start("mesh", &mesh_request)?;
        let missed = terminal_event(&mut service)?;
        assert_eq!(missed.kind, EventKind::Failed);
        assert!(service.bind(&output).has_fill);
        let reopened = AnalysisService::new(temp.0.clone()).bind(&root);
        assert!(!reopened.has_mesh && !reopened.has_gate && !reopened.has_fill);
        Ok(())
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
