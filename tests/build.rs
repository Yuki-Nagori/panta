// launcher 的供给模块还包含生产构建脚本专用的 Ninja 定位逻辑。
// 测试 package 复用其中的 CMake/clang-format 定位，因此这里明确允许
// launcher 专用项目未被使用。
#[allow(dead_code)]
#[path = "../crates/launcher/src/provision.rs"]
mod provision;

use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("panta-tests build script 失败：{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../crates/launcher/src/provision.rs");

    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| "Cargo 未设置 OUT_DIR".to_string())?);
    let target_root = out_dir
        .ancestors()
        .nth(4)
        .ok_or_else(|| "无法从 tests OUT_DIR 推导 target 根".to_string())?
        .to_path_buf();
    let profile = env::var("PROFILE").map_err(|error| format!("Cargo 未设置 PROFILE：{error}"))?;
    let build_type = match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        other => return Err(format!("未知 PROFILE '{other}'，无法映射 CMAKE_BUILD_TYPE")),
    };

    let cmake = provision::resolve_cmake(&target_root)?;
    let clang_format = provision::resolve_clang_format(&target_root)?;
    let native_dir = target_root.join("native").join(&profile);

    println!(
        "cargo:rustc-env=PANTA_TEST_TARGET_DIR={}",
        target_root.display()
    );
    println!(
        "cargo:rustc-env=PANTA_TEST_NATIVE_DIR={}",
        native_dir.display()
    );
    println!("cargo:rustc-env=PANTA_TEST_BUILD_TYPE={build_type}");
    println!("cargo:rustc-env=PANTA_TEST_CMAKE={}", cmake.display());
    println!(
        "cargo:rustc-env=PANTA_TEST_CLANG_FORMAT={}",
        clang_format.display()
    );
    Ok(())
}
