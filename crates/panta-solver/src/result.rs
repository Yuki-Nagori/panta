//! 固定 Moldfill 演示版本的结果读取；先验证终态与哈希，再发布显示数据。

use crate::DisplayMesh;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// 一个操作只允许一个已提交 run，避免读取上次结果或尚未提交的 staging。
pub fn committed_run(output: &Path, mode: &str) -> Result<PathBuf, String> {
    let entries = fs::read_dir(output.join("runs")).map_err(|e| e.to_string())?;
    let mut runs = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() && !path.to_string_lossy().ends_with(".staging") {
            runs.push(path);
        }
    }
    if runs.len() != 1 {
        return Err("Expected one committed solver run".into());
    }
    let path = runs.remove(0);
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if manifest["schema"] != "moldfill.run-manifest/v1"
        || manifest["status"] != "SUCCEEDED"
        || manifest["mode"] != mode
    {
        return Err(format!("Solver did not commit a successful {mode} run"));
    }
    Ok(path)
}

pub fn artifact(run: &Path, suffix: &str) -> Result<PathBuf, String> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(run.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let entries = manifest["artifacts"]
        .as_array()
        .ok_or("Missing artifacts")?;
    let matches: Vec<_> = entries
        .iter()
        .filter(|item| item["path"].as_str().is_some_and(|s| s.ends_with(suffix)))
        .collect();
    if matches.len() != 1 {
        return Err(format!("Missing or ambiguous artifact: {suffix}"));
    }
    let item = matches[0];
    let relative = Path::new(item["path"].as_str().ok_or("Invalid artifact path")?);
    if !relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("Artifact path escapes run directory".into());
    }
    let path = run.join(relative);
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if item["sha256"].as_str() != Some(actual.as_str()) {
        return Err(format!("Artifact checksum mismatch: {suffix}"));
    }
    Ok(path)
}

/// 只读取该版本由 safe_dump 写出的顶层标量；不推断任意 YAML 结构。
pub fn summary(text: &str) -> Result<(f64, f64), String> {
    let scalar = |key: &str| -> Result<&str, String> {
        let prefix = format!("{key}: ");
        let mut values = text.lines().filter_map(|line| line.strip_prefix(&prefix));
        let value = values
            .next()
            .ok_or_else(|| format!("Missing result field: {key}"))?;
        if values.next().is_some() {
            return Err(format!("Duplicate result field: {key}"));
        }
        Ok(value.trim())
    };
    if scalar("schema")? != "moldfill.result/v2-surfacepair" || scalar("length_unit")? != "mm" {
        return Err("Unsupported result schema or length unit".into());
    }
    if scalar("filled")? != "true" {
        return Err("The cavity did not fill; inspect this run's diagnostics".into());
    }
    let time = scalar("t_fill_s")?
        .parse::<f64>()
        .map_err(|e| e.to_string())?;
    let pressure = scalar("p_peak_mpa")?
        .parse::<f64>()
        .map_err(|e| e.to_string())?;
    if !time.is_finite() || time <= 0.0 || !pressure.is_finite() || pressure < 0.0 {
        return Err("Non-finite or invalid result summary".into());
    }
    Ok((time, pressure))
}

fn token<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, String> {
    tokens.next().ok_or_else(|| "Truncated VTK result".into())
}

fn expect<'a>(tokens: &mut impl Iterator<Item = &'a str>, expected: &str) -> Result<(), String> {
    if token(tokens)? != expected {
        return Err(format!("Expected VTK token: {expected}"));
    }
    Ok(())
}

fn count<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<usize, String> {
    let value = token(tokens)?.parse::<usize>().map_err(|e| e.to_string())?;
    if value > 4_000_000 {
        return Err("VTK result exceeds preview size limit".into());
    }
    Ok(value)
}

fn number<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<f64, String> {
    token(tokens)?
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())
}

/// 解析 Moldfill 的 ASCII 三角面结果；坐标 mm，时间 s，NaN 保留为未充填。
/// 三角面展开时同步映射字段，避免原 STL、重划网格和结果网格节点序混用。
pub fn read_vtk(text: &str) -> Result<DisplayMesh, String> {
    let mut lines = text.lines();
    if lines.next() != Some("# vtk DataFile Version 4.2") {
        return Err("Unsupported VTK version".into());
    }
    if !lines.next().is_some_and(|s| s.contains("length unit: mm")) || lines.next() != Some("ASCII")
    {
        return Err("Expected millimetre ASCII VTK".into());
    }
    let mut tokens = lines.flat_map(|line| line.split_whitespace());
    expect(&mut tokens, "DATASET")?;
    expect(&mut tokens, "UNSTRUCTURED_GRID")?;
    expect(&mut tokens, "POINTS")?;
    let nn = count(&mut tokens)?;
    expect(&mut tokens, "double")?;
    let mut points = Vec::new();
    for _ in 0..nn {
        let point = [
            number(&mut tokens)?,
            number(&mut tokens)?,
            number(&mut tokens)?,
        ];
        if point.iter().any(|v| !v.is_finite()) {
            return Err("Non-finite VTK coordinate".into());
        }
        points.push(point);
    }
    expect(&mut tokens, "CELLS")?;
    let ne = count(&mut tokens)?;
    if ne == 0 || count(&mut tokens)? != ne * 4 {
        return Err("Expected triangular VTK cells".into());
    }
    let mut cells = Vec::new();
    for _ in 0..ne {
        expect(&mut tokens, "3")?;
        let ids = [
            count(&mut tokens)?,
            count(&mut tokens)?,
            count(&mut tokens)?,
        ];
        if ids.iter().any(|&id| id >= nn) {
            return Err("VTK cell index out of bounds".into());
        }
        cells.push(ids);
    }
    expect(&mut tokens, "CELL_TYPES")?;
    if count(&mut tokens)? != ne {
        return Err("VTK cell type count mismatch".into());
    }
    for _ in 0..ne {
        // 当前引擎导出三节点 VTK_POLYGON(7)，也接受标准 VTK_TRIANGLE(5)。
        if !matches!(token(&mut tokens)?, "7" | "5") {
            return Err("Unsupported VTK cell type".into());
        }
    }
    expect(&mut tokens, "POINT_DATA")?;
    if count(&mut tokens)? != nn {
        return Err("VTK field size mismatch".into());
    }
    for expected in [
        "SCALARS",
        "fill_time_s",
        "double",
        "1",
        "LOOKUP_TABLE",
        "default",
    ] {
        expect(&mut tokens, expected)?;
    }
    let mut times = Vec::new();
    for _ in 0..nn {
        let time = number(&mut tokens)?;
        if time.is_infinite() || time < 0.0 {
            return Err("Invalid filling time".into());
        }
        times.push(time);
    }
    if !times.iter().any(|v| v.is_finite()) {
        return Err("VTK contains no filled nodes".into());
    }
    let mut mesh = DisplayMesh::default();
    for ids in cells {
        for id in ids {
            mesh.coordinates.extend_from_slice(&points[id]);
            mesh.fill_times.push(times[id]);
        }
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VTK: &str = "# vtk DataFile Version 4.2\npreview (length unit: mm)\nASCII\nDATASET UNSTRUCTURED_GRID\nPOINTS 3 double\n0 0 0 1 0 0 0 1 0\nCELLS 1 4\n3 2 0 1\nCELL_TYPES 1\n7\nPOINT_DATA 3\nSCALARS fill_time_s double 1\nLOOKUP_TABLE default\n0 1 NaN\n";

    #[test]
    fn staging_failed_runs_and_damaged_artifacts_are_rejected()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = crate::tests::TempDir::new();
        let run = temp.0.join("runs/test.staging");
        fs::create_dir_all(run.join("artifacts"))?;
        let bytes = b"validated payload";
        fs::write(run.join("artifacts/result.yaml"), bytes)?;
        let mut manifest = serde_json::json!({
            "schema":"moldfill.run-manifest/v1", "mode":"solve", "status":"SUCCEEDED",
            "artifacts":[{"path":"artifacts/result.yaml", "sha256":format!("{:x}",Sha256::digest(bytes))}]
        });
        fs::write(run.join("manifest.json"), manifest.to_string())?;
        assert!(committed_run(&temp.0, "solve").is_err());
        let committed = temp.0.join("runs/test");
        fs::rename(run, &committed)?;
        assert_eq!(committed_run(&temp.0, "solve")?, committed);
        assert!(committed_run(&temp.0, "remesh_only").is_err());
        assert!(artifact(&committed, "result.yaml").is_ok());
        fs::write(committed.join("artifacts/result.yaml"), b"corrupted")?;
        assert!(artifact(&committed, "result.yaml").is_err());
        manifest["status"] = "FAILED".into();
        manifest["artifacts"][0]["path"] = "../result.yaml".into();
        fs::write(committed.join("manifest.json"), manifest.to_string())?;
        assert!(committed_run(&temp.0, "solve").is_err());
        assert!(artifact(&committed, "result.yaml").is_err());
        Ok(())
    }

    #[test]
    fn field_follows_cell_indices_and_preserves_unfilled() -> Result<(), String> {
        let mesh = read_vtk(VTK)?;
        assert_eq!(&mesh.coordinates[..3], &[0.0, 1.0, 0.0]);
        assert!(mesh.fill_times[0].is_nan());
        assert_eq!(&mesh.fill_times[1..], &[0.0, 1.0]);
        Ok(())
    }

    #[test]
    fn rejects_invalid_indices_units_and_partial_arrays() {
        for bad in [
            VTK.replace("3 2 0 1", "3 3 0 1"),
            VTK.replace("unit: mm", "unit: m"),
            VTK.replace("0 1 NaN", "0 1"),
            VTK.replace("0 1 NaN", "0 inf NaN"),
        ] {
            assert!(read_vtk(&bad).is_err());
        }
    }

    #[test]
    fn summary_rejects_short_shot_and_nonfinite_values() {
        let valid = "schema: moldfill.result/v2-surfacepair\nlength_unit: mm\nfilled: true\nt_fill_s: 1.9\np_peak_mpa: 2.1\n";
        assert_eq!(summary(valid), Ok((1.9, 2.1)));
        assert!(summary(&valid.replace("filled: true", "filled: false")).is_err());
        assert!(summary(&valid.replace("1.9", "NaN")).is_err());
    }
}
