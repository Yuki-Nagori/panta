//! 每个分析目录一份网格、浇口和充填缓存。
//! 目录键先规范化再入内存表，避免 macOS 把 `/var` 解析成 `/private/var` 后拆成两份。
//! 产物路径只保存相对该目录的安全相对路径；修改时间用纳秒整数，避免 JSON 数字丢精度。
//! 读回时确认规范化后的文件仍在该目录内，并核对产物文件自身的长度和修改时间。

use crate::{DisplayMesh, read_stl, result};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

/// 相同输入的比较尺度。1e-9 覆盖 JSON 往返和界面里的双精度微调。
const RELATIVE_EPSILON: f64 = 1e-9;

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileId {
    len: u64,
    mtime_ns: i64,
}

struct MeshStage {
    source: FileId,
    artifact: FileId,
    edge: f64,
    stl: PathBuf,
    display: DisplayMesh,
}

struct GateStage {
    mesh: FileId,
    artifact: FileId,
    gates: i64,
    mold: f64,
    melt: f64,
    yaml: PathBuf,
    points: Vec<f64>,
}

struct FillStage {
    mesh: FileId,
    gate: FileId,
    mold: f64,
    melt: f64,
    flow: f64,
    switch_volume: f64,
    vtk: PathBuf,
    display: DisplayMesh,
    fill_time: f64,
    peak_pressure: f64,
}

/// 一次充填缓存写入所需的工艺标量。时间单位 s，压力单位 MPa。
pub(crate) struct FillParams {
    pub mold: f64,
    pub melt: f64,
    pub flow: f64,
    pub switch_volume: f64,
    pub fill_time: f64,
    pub peak_pressure: f64,
}

/// 给界面恢复用的已解析显示数据。坐标 mm；无充填时时间为 0。
#[derive(Default)]
pub struct CachedView {
    pub root: String,
    pub has_mesh: bool,
    pub has_gate: bool,
    pub has_fill: bool,
    pub coordinates: Vec<f64>,
    pub fill_times: Vec<f64>,
    pub pressures: Vec<f64>,
    pub gate_points: Vec<f64>,
    pub fill_time: f64,
    pub peak_pressure: f64,
}

/// 一个零件分析目录的成功产物。内存中的显示网格只在载入或新结果时解析。
pub(crate) struct PartCache {
    root: PathBuf,
    mesh: Option<MeshStage>,
    gate: Option<GateStage>,
    fill: Option<FillStage>,
}

impl PartCache {
    pub(crate) fn load(root: &Path) -> Self {
        let mut part = Self {
            root: root.to_path_buf(),
            mesh: None,
            gate: None,
            fill: None,
        };
        let Ok(bytes) = fs::read(root.join("cache.json")) else {
            return part;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
            return part;
        };
        if value.get("schema") != Some(&Value::from(1)) {
            return part;
        }
        if let Some(mesh) = value.get("mesh")
            && let Ok(stage) = load_mesh(root, mesh)
        {
            part.mesh = Some(stage);
        }
        if let Some(mesh) = &part.mesh
            && let Some(gate) = value.get("gate")
            && let Ok(stage) = load_gate(root, gate, &mesh.stl)
        {
            part.gate = Some(stage);
        }
        let fill = if let (Some(mesh), Some(gate)) = (&part.mesh, &part.gate) {
            value
                .get("fill")
                .and_then(|fill| load_fill(root, fill, &mesh.stl, &gate.yaml).ok())
        } else {
            None
        };
        part.fill = fill;
        part
    }

    pub(crate) fn has_mesh(&self) -> bool {
        self.mesh.is_some()
    }
    pub(crate) fn has_gate(&self) -> bool {
        self.gate.is_some()
    }
    pub(crate) fn mesh_path(&self) -> String {
        self.mesh
            .as_ref()
            .map(|mesh| mesh.stl.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
    pub(crate) fn gate_path(&self) -> String {
        self.gate
            .as_ref()
            .map(|gate| gate.yaml.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub(crate) fn matches_mesh(&self, source: &Path, edge: f64) -> bool {
        let Some(mesh) = &self.mesh else {
            return false;
        };
        file_id(source).is_ok_and(|id| id == mesh.source && same_number(mesh.edge, edge))
            && file_id(&mesh.stl).is_ok_and(|id| id == mesh.artifact)
    }

    pub(crate) fn matches_gate(&self, gates: i64, mold: f64, melt: f64) -> bool {
        let (Some(mesh), Some(gate)) = (&self.mesh, &self.gate) else {
            return false;
        };
        file_id(&mesh.stl).is_ok_and(|id| {
            id == gate.mesh
                && gate.gates == gates
                && same_number(gate.mold, mold)
                && same_number(gate.melt, melt)
        }) && file_id(&gate.yaml).is_ok_and(|id| id == gate.artifact)
    }

    pub(crate) fn matches_fill(&self, mold: f64, melt: f64, flow: f64, switch_volume: f64) -> bool {
        let (Some(mesh), Some(gate), Some(fill)) = (&self.mesh, &self.gate, &self.fill) else {
            return false;
        };
        let Ok(mesh_id) = file_id(&mesh.stl) else {
            return false;
        };
        let Ok(gate_id) = file_id(&gate.yaml) else {
            return false;
        };
        mesh_id == fill.mesh
            && gate_id == fill.gate
            && same_number(fill.mold, mold)
            && same_number(fill.melt, melt)
            && same_number(fill.flow, flow)
            && same_number(fill.switch_volume, switch_volume)
    }

    /// 新网格替换显示数据，并丢弃依赖旧网格的浇口和充填。
    pub(crate) fn absorb_mesh(
        &mut self,
        source: &Path,
        edge: f64,
        stl: &Path,
        display: DisplayMesh,
    ) -> Result<(), String> {
        let source_id = file_id(source)?;
        let stl = canonical_file(stl)?;
        ensure_inside(&self.root, &stl)?;
        let artifact = file_id(&stl)?;
        let previous = (self.mesh.take(), self.gate.take(), self.fill.take());
        self.mesh = Some(MeshStage {
            source: source_id,
            artifact,
            edge,
            stl,
            display,
        });
        self.restore_unless_saved(previous)
    }

    /// 新浇口替换推荐位置，并丢弃依赖旧浇口的充填。
    pub(crate) fn absorb_gate(
        &mut self,
        yaml: &Path,
        gates: i64,
        mold: f64,
        melt: f64,
        points: Vec<f64>,
    ) -> Result<(), String> {
        if points.len() != 3 || points.iter().any(|point| !point.is_finite()) {
            return Err("Invalid selected gate position".into());
        }
        let mesh_id = {
            let mesh = self.mesh.as_ref().ok_or("Generate the mesh first")?;
            file_id(&mesh.stl)?
        };
        let yaml = canonical_file(yaml)?;
        ensure_inside(&self.root, &yaml)?;
        let artifact = file_id(&yaml)?;
        let previous = (self.gate.take(), self.fill.take());
        self.gate = Some(GateStage {
            mesh: mesh_id,
            artifact,
            gates,
            mold,
            melt,
            yaml,
            points,
        });
        if let Err(error) = self.save() {
            self.gate = previous.0;
            self.fill = previous.1;
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn absorb_fill(
        &mut self,
        vtk: &Path,
        params: &FillParams,
        display: DisplayMesh,
    ) -> Result<(), String> {
        if params.fill_time <= 0.0 || !params.fill_time.is_finite() || params.peak_pressure < 0.0 {
            return Err("Non-finite or invalid result summary".into());
        }
        let (mesh_id, gate_id) = {
            let mesh = self.mesh.as_ref().ok_or("Generate the mesh first")?;
            let gate = self
                .gate
                .as_ref()
                .ok_or("Run Gate Location Analyze first")?;
            (file_id(&mesh.stl)?, file_id(&gate.yaml)?)
        };
        let vtk = canonical_file(vtk)?;
        ensure_inside(&self.root, &vtk)?;
        let previous = self.fill.take();
        self.fill = Some(FillStage {
            mesh: mesh_id,
            gate: gate_id,
            mold: params.mold,
            melt: params.melt,
            flow: params.flow,
            switch_volume: params.switch_volume,
            vtk,
            display,
            fill_time: params.fill_time,
            peak_pressure: params.peak_pressure,
        });
        if let Err(error) = self.save() {
            self.fill = previous;
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn view(&self) -> CachedView {
        let mut view = CachedView {
            root: self.root.to_string_lossy().into_owned(),
            has_mesh: self.mesh.is_some(),
            has_gate: self.gate.is_some(),
            has_fill: self.fill.is_some(),
            fill_time: self.fill.as_ref().map_or(0.0, |fill| fill.fill_time),
            peak_pressure: self.fill.as_ref().map_or(0.0, |fill| fill.peak_pressure),
            ..CachedView::default()
        };
        if let Some(fill) = &self.fill {
            view.coordinates.clone_from(&fill.display.coordinates);
            view.fill_times.clone_from(&fill.display.fill_times);
            view.pressures.clone_from(&fill.display.pressures);
        } else if let Some(mesh) = &self.mesh {
            view.coordinates.clone_from(&mesh.display.coordinates);
        }
        if let Some(gate) = &self.gate {
            view.gate_points.clone_from(&gate.points);
        }
        view
    }

    fn restore_unless_saved(
        &mut self,
        previous: (Option<MeshStage>, Option<GateStage>, Option<FillStage>),
    ) -> Result<(), String> {
        if let Err(error) = self.save() {
            self.mesh = previous.0;
            self.gate = previous.1;
            self.fill = previous.2;
            return Err(error);
        }
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        if !self.root.is_dir() {
            return Err(format!(
                "analysis directory is missing: {}",
                self.root.display()
            ));
        }
        let temp = self.root.join(".cache.json.tmp");
        let dest = self.root.join("cache.json");
        fs::write(&temp, self.to_json()?.to_string()).map_err(|error| error.to_string())?;
        fs::rename(temp, dest).map_err(|error| error.to_string())
    }

    fn to_json(&self) -> Result<Value, String> {
        let mut value = serde_json::json!({"schema": 1});
        if let Some(mesh) = &self.mesh {
            value["mesh"] = serde_json::json!({
                "sourceLen": mesh.source.len,
                "sourceMtimeNs": mesh.source.mtime_ns,
                "edgeLength": mesh.edge,
                "stlLen": mesh.artifact.len,
                "stlMtimeNs": mesh.artifact.mtime_ns,
                "stl": relative_to(&self.root, &mesh.stl)?,
            });
        }
        if let Some(gate) = &self.gate {
            value["gate"] = serde_json::json!({
                "meshLen": gate.mesh.len,
                "meshMtimeNs": gate.mesh.mtime_ns,
                "numberOfGates": gate.gates,
                "moldTemperature": gate.mold,
                "meltTemperature": gate.melt,
                "yamlLen": gate.artifact.len,
                "yamlMtimeNs": gate.artifact.mtime_ns,
                "yaml": relative_to(&self.root, &gate.yaml)?,
                "points": gate.points,
            });
        }
        if let Some(fill) = &self.fill {
            value["fill"] = serde_json::json!({
                "meshLen": fill.mesh.len,
                "meshMtimeNs": fill.mesh.mtime_ns,
                "gateLen": fill.gate.len,
                "gateMtimeNs": fill.gate.mtime_ns,
                "moldTemperature": fill.mold,
                "meltTemperature": fill.melt,
                "flowRate": fill.flow,
                "switchVolume": fill.switch_volume,
                "fillTime": fill.fill_time,
                "peakPressure": fill.peak_pressure,
                "vtk": relative_to(&self.root, &fill.vtk)?,
            });
        }
        Ok(value)
    }
}

/// 向上找到已存在的祖先并规范化，再接回尚未创建的部分。不创建目录。
pub fn normalize_root(path: &Path) -> PathBuf {
    let mut missing = Vec::new();
    let mut current = path.to_path_buf();
    while !current.as_os_str().is_empty() && !current.exists() {
        let Some(name) = current.file_name() else {
            break;
        };
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        missing.push(name.to_os_string());
        current = parent.to_path_buf();
    }
    let mut base = match current.canonicalize() {
        Ok(path) => path,
        Err(_) => current,
    };
    for name in missing.iter().rev() {
        base.push(name);
    }
    base
}

fn same_number(left: f64, right: f64) -> bool {
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= scale * RELATIVE_EPSILON
}

fn file_id(path: &Path) -> Result<FileId, String> {
    let meta = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let modified = meta
        .modified()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let since = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mtime_ns = i64::try_from(since.as_nanos())
        .map_err(|_| format!("mtime out of range: {}", path.display()))?;
    Ok(FileId {
        len: meta.len(),
        mtime_ns,
    })
}

fn canonical_file(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn ensure_inside(root: &Path, path: &Path) -> Result<(), String> {
    relative_to(root, path).map(|_| ())
}

fn relative_to(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "artifact is outside the analysis directory: {}",
            path.display()
        )
    })?;
    if !relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err("cache path escapes the analysis directory".into());
    }
    let text = relative.to_string_lossy();
    if text.is_empty() {
        return Err("cache path escapes the analysis directory".into());
    }
    Ok(text.into_owned())
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err("cache path escapes the analysis directory".into());
    }
    Ok(root.join(path))
}

fn json_u64(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Invalid {key}"))
}

fn json_i64(value: &Value, key: &str) -> Result<i64, String> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Invalid {key}"))
}

fn json_file_id(value: &Value, len_key: &str, mtime_key: &str) -> Result<FileId, String> {
    Ok(FileId {
        len: json_u64(value, len_key)?,
        mtime_ns: json_i64(value, mtime_key)?,
    })
}

fn json_f64(value: &Value, key: &str) -> Result<f64, String> {
    let number = value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("Invalid {key}"))?;
    if !number.is_finite() {
        return Err(format!("Invalid {key}"));
    }
    Ok(number)
}

fn load_mesh(root: &Path, value: &Value) -> Result<MeshStage, String> {
    let edge = json_f64(value, "edgeLength")?;
    if edge <= 0.0 {
        return Err("Invalid edgeLength".into());
    }
    let relative = value
        .get("stl")
        .and_then(Value::as_str)
        .ok_or("Invalid stl")?;
    let stl = canonical_file(&safe_join(root, relative)?)?;
    ensure_inside(root, &stl)?;
    let artifact = json_file_id(value, "stlLen", "stlMtimeNs")?;
    if file_id(&stl)? != artifact {
        return Err("mesh artifact changed".into());
    }
    let display = read_stl(&stl)?;
    if display.coordinates.is_empty() {
        return Err("Empty mesh".into());
    }
    Ok(MeshStage {
        source: FileId {
            len: json_u64(value, "sourceLen")?,
            mtime_ns: json_i64(value, "sourceMtimeNs")?,
        },
        artifact,
        edge,
        stl,
        display,
    })
}

fn load_gate(root: &Path, value: &Value, mesh_stl: &Path) -> Result<GateStage, String> {
    let mesh = FileId {
        len: json_u64(value, "meshLen")?,
        mtime_ns: json_i64(value, "meshMtimeNs")?,
    };
    if file_id(mesh_stl)? != mesh {
        return Err("mesh artifact changed".into());
    }
    let points = value
        .get("points")
        .and_then(Value::as_array)
        .ok_or("Invalid points")?;
    if points.len() != 3 {
        return Err("Invalid points".into());
    }
    let mut coords = Vec::with_capacity(3);
    for point in points {
        let number = point.as_f64().ok_or("Invalid points")?;
        if !number.is_finite() {
            return Err("Invalid points".into());
        }
        coords.push(number);
    }
    let relative = value
        .get("yaml")
        .and_then(Value::as_str)
        .ok_or("Invalid yaml")?;
    let yaml = canonical_file(&safe_join(root, relative)?)?;
    ensure_inside(root, &yaml)?;
    let artifact = json_file_id(value, "yamlLen", "yamlMtimeNs")?;
    if file_id(&yaml)? != artifact {
        return Err("gate artifact changed".into());
    }
    Ok(GateStage {
        mesh,
        artifact,
        gates: json_i64(value, "numberOfGates")?,
        mold: json_f64(value, "moldTemperature")?,
        melt: json_f64(value, "meltTemperature")?,
        yaml,
        points: coords,
    })
}

fn load_fill(
    root: &Path,
    value: &Value,
    mesh_stl: &Path,
    gate_yaml: &Path,
) -> Result<FillStage, String> {
    let mesh = FileId {
        len: json_u64(value, "meshLen")?,
        mtime_ns: json_i64(value, "meshMtimeNs")?,
    };
    let gate = FileId {
        len: json_u64(value, "gateLen")?,
        mtime_ns: json_i64(value, "gateMtimeNs")?,
    };
    if file_id(mesh_stl)? != mesh || file_id(gate_yaml)? != gate {
        return Err("cached fill inputs changed".into());
    }
    let fill_time = json_f64(value, "fillTime")?;
    let peak_pressure = json_f64(value, "peakPressure")?;
    if fill_time <= 0.0 || peak_pressure < 0.0 {
        return Err("Non-finite or invalid result summary".into());
    }
    let relative = value
        .get("vtk")
        .and_then(Value::as_str)
        .ok_or("Invalid vtk")?;
    let vtk = canonical_file(&safe_join(root, relative)?)?;
    ensure_inside(root, &vtk)?;
    let text = fs::read_to_string(&vtk).map_err(|error| format!("{}: {error}", vtk.display()))?;
    Ok(FillStage {
        mesh,
        gate,
        mold: json_f64(value, "moldTemperature")?,
        melt: json_f64(value, "meltTemperature")?,
        flow: json_f64(value, "flowRate")?,
        switch_volume: json_f64(value, "switchVolume")?,
        vtk,
        display: result::read_vtk(&text)?,
        fill_time,
        peak_pressure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const STL: &str = "solid x\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 2 0\nendsolid\n";
    const VTK: &str = "# vtk DataFile Version 4.2\npreview (length unit: mm)\nASCII\nDATASET UNSTRUCTURED_GRID\nPOINTS 3 double\n0 0 0 1 0 0 0 1 0\nCELLS 1 4\n3 2 0 1\nCELL_TYPES 1\n7\nPOINT_DATA 3\nSCALARS fill_time_s double 1\nLOOKUP_TABLE default\n0 1 NaN\nSCALARS pressure_Pa double 1\nLOOKUP_TABLE default\n1000000 2000000 NaN\n";

    fn write_case(root: &Path) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        fs::create_dir_all(root.join("mesh"))?;
        fs::create_dir_all(root.join("gate"))?;
        fs::create_dir_all(root.join("fill"))?;
        let source = root.join("source.stl");
        let stl = root.join("mesh/part_remeshed.stl");
        let yaml = root.join("gate/gate_recommend.yaml");
        fs::write(&source, STL)?;
        fs::write(&stl, STL)?;
        fs::write(&yaml, "gate: 1\n")?;
        fs::write(root.join("fill/fill_pattern.vtk"), VTK)?;
        Ok((source, stl, yaml))
    }

    #[test]
    fn missing_directory_is_not_created_and_uses_canonical_parent()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = crate::tests::TempDir::new();
        fs::create_dir_all(&temp.0)?;
        let missing = temp.0.join("analyses").join("part");
        let normalized = normalize_root(&missing);
        assert_eq!(
            normalized,
            temp.0.canonicalize()?.join("analyses").join("part")
        );
        assert!(!missing.exists());
        let view = PartCache::load(&normalized).view();
        assert!(!view.has_mesh);
        assert!(!normalized.exists());
        Ok(())
    }

    #[test]
    fn save_roundtrip_matches_inputs_and_new_mesh_clears_dependents()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = crate::tests::TempDir::new();
        let root = temp.0.join("part");
        fs::create_dir_all(&root)?;
        let canonical = root.canonicalize()?;
        let (source, stl, yaml) = write_case(&canonical)?;
        let mut part = PartCache::load(&canonical);
        part.absorb_mesh(&source, 12.0, &stl, read_stl(&stl)?)?;
        part.absorb_gate(&yaml, 1, 40.0, 230.0, vec![1.5, 2.5, 3.5])?;
        part.absorb_fill(
            &canonical.join("fill/fill_pattern.vtk"),
            &FillParams {
                mold: 40.0,
                melt: 230.0,
                flow: 94.7,
                switch_volume: 99.0,
                fill_time: 1.9,
                peak_pressure: 2.1,
            },
            result::read_vtk(VTK)?,
        )?;
        let stored = fs::read_to_string(canonical.join("cache.json"))?;
        let json: Value = serde_json::from_str(&stored)?;
        assert_eq!(json["schema"], 1);
        assert!(json["mesh"]["sourceMtimeNs"].as_i64().is_some());
        assert!(json["mesh"]["stlLen"].as_u64().is_some());
        assert!(json["gate"]["yamlLen"].as_u64().is_some());
        assert!(json["stl"].is_null());
        assert!(!json["mesh"]["stl"].as_str().unwrap_or("").starts_with('/'));

        let loaded = PartCache::load(&canonical);
        let view = loaded.view();
        assert!(view.has_mesh && view.has_gate && view.has_fill);
        assert_eq!(view.gate_points, vec![1.5, 2.5, 3.5]);
        assert_eq!(view.fill_time, 1.9);
        assert!(view.pressures.contains(&1.0));
        assert!(loaded.matches_mesh(&source, 12.0 + 1e-12));
        assert!(!loaded.matches_mesh(&source, 13.0));
        assert!(loaded.matches_gate(1, 40.0, 230.0));
        assert!(!loaded.matches_gate(1, 41.0, 230.0));
        assert!(loaded.matches_fill(40.0, 230.0, 94.7, 99.0));

        fs::write(&yaml, "gate: 2\n")?;
        assert!(!loaded.matches_gate(1, 40.0, 230.0));
        let dropped = PartCache::load(&canonical).view();
        assert!(dropped.has_mesh && !dropped.has_gate && !dropped.has_fill);

        part.absorb_gate(&yaml, 1, 40.0, 230.0, vec![1.5, 2.5, 3.5])?;
        let gated = PartCache::load(&canonical).view();
        assert!(gated.has_mesh && gated.has_gate && !gated.has_fill);

        part.absorb_mesh(&source, 8.0, &stl, read_stl(&stl)?)?;
        let cleared = PartCache::load(&canonical).view();
        assert!(cleared.has_mesh);
        assert!(!cleared.has_gate && !cleared.has_fill);
        assert!(cleared.gate_points.is_empty());
        Ok(())
    }

    #[test]
    fn escaped_and_broken_artifacts_drop_their_stage() -> Result<(), Box<dyn std::error::Error>> {
        let temp = crate::tests::TempDir::new();
        let root = temp.0.join("part");
        fs::create_dir_all(root.join("mesh"))?;
        let canonical = root.canonicalize()?;
        fs::write(canonical.join("mesh/part_remeshed.stl"), STL)?;
        let source = canonical.join("source.stl");
        fs::write(&source, STL)?;
        let mut part = PartCache::load(&canonical);
        part.absorb_mesh(
            &source,
            12.0,
            &canonical.join("mesh/part_remeshed.stl"),
            read_stl(&canonical.join("mesh/part_remeshed.stl"))?,
        )?;
        let mut json: Value =
            serde_json::from_str(&fs::read_to_string(canonical.join("cache.json"))?)?;
        json["mesh"]["stl"] = "../secret.stl".into();
        fs::write(canonical.join("cache.json"), json.to_string())?;
        assert!(!PartCache::load(&canonical).view().has_mesh);

        json["mesh"]["stl"] = "/tmp/secret.stl".into();
        fs::write(canonical.join("cache.json"), json.to_string())?;
        assert!(!PartCache::load(&canonical).view().has_mesh);

        json["mesh"]["stl"] = "mesh/missing.stl".into();
        fs::write(canonical.join("cache.json"), json.to_string())?;
        assert!(!PartCache::load(&canonical).view().has_mesh);

        // 相对路径合法，但规范化后落到分析目录外时不能读取。
        #[cfg(unix)]
        {
            let outside = temp.0.join("outside.stl");
            fs::write(&outside, STL)?;
            std::os::unix::fs::symlink(&outside, canonical.join("mesh/link.stl"))?;
            let id = file_id(&outside)?;
            json["mesh"]["stl"] = "mesh/link.stl".into();
            json["mesh"]["stlLen"] = id.len.into();
            json["mesh"]["stlMtimeNs"] = id.mtime_ns.into();
            fs::write(canonical.join("cache.json"), json.to_string())?;
            assert!(!PartCache::load(&canonical).view().has_mesh);
        }
        Ok(())
    }
}
