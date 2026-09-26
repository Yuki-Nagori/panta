//! 生成 `fsm/` 下有限状态机（FSM）声明的 Rust 转移元数据（073）。
//!
//! 输入为目录发现的 `fsm/*.pa` 集合：文件 stem 必须与状态机 `name` 一致；
//! 目录为空、输入缺失、校验失败或生成失败都以 panic 中止构建，不回退到
//! 残留的旧生成文件。

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|error| panic!("CARGO_MANIFEST_DIR: {error}"));
    let fsm_dir = PathBuf::from(manifest_dir).join("fsm");
    println!("cargo:rerun-if-changed={}", fsm_dir.display());

    let inputs = collect_fsm_inputs(&fsm_dir);
    if inputs.is_empty() {
        panic!("no fsm inputs found under {}", fsm_dir.display());
    }
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|error| panic!("OUT_DIR: {error}"));
    let fsm_out_dir = PathBuf::from(out_dir).join("fsm");
    // 先整目录重建生成集合：从目录发现的输入被删除时，旧输出不再残留。
    let _ = fs::remove_dir_all(&fsm_out_dir);
    fs::create_dir_all(&fsm_out_dir)
        .unwrap_or_else(|error| panic!("create {}: {error}", fsm_out_dir.display()));

    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_else(|| panic!("fsm input has no file stem: {}", input.display()));
        let source = fs::read_to_string(&input)
            .unwrap_or_else(|error| panic!("read {}: {error}", input.display()));
        let document = panta_dsl_core::fsm::parse(&source)
            .unwrap_or_else(|diagnostics| panic!("invalid fsm {}: {diagnostics}", input.display()));
        if document.name != stem {
            panic!(
                "fsm file stem '{stem}' must match fsm name '{}'",
                document.name
            );
        }
        let code = panta_dsl_core::fsm::generate_rust(&document);
        let target = fsm_out_dir.join(format!(
            "{}.rs",
            panta_dsl_core::fsm::snake_case(&document.name)
        ));
        fs::write(&target, &code)
            .unwrap_or_else(|error| panic!("write {}: {error}", target.display()));
    }
}

/// 枚举 `fsm/*.pa`：从目录发现的输入集合，确定性排序保证生成顺序稳定。
fn collect_fsm_inputs(fsm_dir: &std::path::Path) -> Vec<PathBuf> {
    let entries =
        fs::read_dir(fsm_dir).unwrap_or_else(|error| panic!("read {}: {error}", fsm_dir.display()));
    let mut inputs = Vec::new();
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("read fsm entry: {error}"))
            .path();
        let is_fsm_input = path.extension().and_then(|value| value.to_str()) == Some("pa");
        if is_fsm_input {
            inputs.push(path);
        }
    }
    inputs.sort();
    inputs
}
