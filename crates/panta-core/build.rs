//! 从 `fsm/` 声明生成 Rust 转移元数据（073）。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|error| panic!("CARGO_MANIFEST_DIR: {error}"));
    let fsm_dir = PathBuf::from(manifest_dir).join("fsm");
    println!("cargo:rerun-if-changed={}", fsm_dir.display());

    let out_dir = env::var("OUT_DIR").unwrap_or_else(|error| panic!("OUT_DIR: {error}"));
    generate_outputs(&fsm_dir, Path::new(&out_dir))
        .unwrap_or_else(|error| panic!("generate FSM metadata: {error}"));
}

/// 从当前输入集合重新生成输出；任何失败都会先移除上一轮生成物。
pub(crate) fn generate_outputs(fsm_dir: &Path, out_dir: &Path) -> Result<(), String> {
    let fsm_out_dir = out_dir.join("fsm");
    if fsm_out_dir.exists() {
        fs::remove_dir_all(&fsm_out_dir)
            .map_err(|error| format!("remove {}: {error}", fsm_out_dir.display()))?;
    }
    fs::create_dir_all(&fsm_out_dir)
        .map_err(|error| format!("create {}: {error}", fsm_out_dir.display()))?;

    let inputs = collect_fsm_inputs(fsm_dir)?;
    if inputs.is_empty() {
        return Err(format!("no fsm inputs found under {}", fsm_dir.display()));
    }

    let mut output_names = Vec::with_capacity(inputs.len());
    let mut documents = Vec::with_capacity(inputs.len());
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("fsm input has no file stem: {}", input.display()))?;
        let source = fs::read_to_string(&input)
            .map_err(|error| format!("read {}: {error}", input.display()))?;
        let document = panta_dsl_core::fsm::parse(&source)
            .map_err(|diagnostics| format!("invalid fsm {}: {diagnostics}", input.display()))?;
        if document.name != stem {
            return Err(format!(
                "fsm file stem '{stem}' must match fsm name '{}'",
                document.name
            ));
        }

        let output_name = format!("{}.rs", panta_dsl_core::fsm::snake_case(&document.name));
        register_output_name(&mut output_names, output_name)?;
        documents.push((input, document));
    }

    for ((input, document), output_name) in documents.into_iter().zip(output_names) {
        let target = fsm_out_dir.join(output_name);
        let code = panta_dsl_core::fsm::generate_rust(&document);
        fs::write(&target, code).map_err(|error| {
            format!(
                "write {} (from {}): {error}",
                target.display(),
                input.display()
            )
        })?;
    }

    Ok(())
}

/// 注册输出文件名，防止多个声明覆盖同一个生成文件。
pub(crate) fn register_output_name(names: &mut Vec<String>, name: String) -> Result<(), String> {
    if names.iter().any(|existing| existing == &name) {
        return Err(format!("multiple fsm inputs map to output '{name}'"));
    }
    names.push(name);
    Ok(())
}

/// 枚举 `fsm/*.pa`：目录变化负责增删追踪，文件变化负责内容重建。
fn collect_fsm_inputs(fsm_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries =
        fs::read_dir(fsm_dir).map_err(|error| format!("read {}: {error}", fsm_dir.display()))?;
    let mut inputs = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| format!("read fsm entry: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) == Some("pa") {
            inputs.push(path);
        }
    }
    inputs.sort();
    Ok(inputs)
}
