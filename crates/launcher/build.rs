//! Cargo→CMake 调度（任务 004）：以与 native presets 相同的有效配置构建
//! native 树，并把可执行产物路径在编译期注入 launcher（PANTA_NATIVE_BIN）。
//!
//! 图约束：Cargo → 本脚本 → CMake/Ninja → native targets；CMake 侧不回调
//! Cargo，无递归。未来 C++ 消费 Rust 库时，接入点是 CMake 侧导入 Rust 产物
//! （任务 006 定），不得从本脚本再次触发 Cargo。
//!
//! 重建追踪：Cargo 对 rerun-if-changed 的目录不递归，故下方清单显式列举到
//! 含源文件的层级；新增模块目录时功能变更必然触及已追踪的 CMakeLists，
//! 但仍应同步本清单。qml/、resources/ 落地时（任务 005）在此追加——这是
//! 重建追踪的扩展点。
//!
//! 构建脚本产物只写 OUT_DIR；失败时继承子进程输出并原样退出，不掩盖
//! 编译器/CMake 诊断。

use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// 需要 CMake 增量感知的仓库路径（相对本 package 根；目录不递归）。
const RERUN_PATHS: &[&str] = &[
    "build.rs",
    "../../native/CMakeLists.txt",
    "../../native/CMakePresets.json",
    "../../native/cmake",
    "../../native/app",
    "../../native/bridge",
    "../../native/bridge/src",
    "../../native/bridge/tests",
    "../../native/foundation",
    "../../native/foundation/src",
    "../../native/foundation/tests",
    "../../native/foundation/include/panta/foundation",
    "../../qml",
    "../../qml/Themes",
    "../../qml/Panels",
    // 扩展点：resources/ 与更多 QML 子目录落地时在此追加（任务 005 起）。
];

/// 影响配置结果的环境变量，变更即重建。CMAKE 可指定 cmake 可执行文件路径。
const RERUN_ENVS: &[&str] = &["CMAKE", "CXX", "CMAKE_GENERATOR", "CMAKE_PREFIX_PATH"];

fn main() -> ExitCode {
    for path in RERUN_PATHS {
        println!("cargo:rerun-if-changed={path}");
    }
    for key in RERUN_ENVS {
        println!("cargo:rerun-if-env-changed={key}");
    }

    match orchestrate() {
        Ok(product) => {
            println!("cargo:rustc-env=PANTA_NATIVE_BIN={}", product.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("panta-launcher 构建脚本失败：{message}");
            ExitCode::FAILURE
        }
    }
}

fn orchestrate() -> Result<PathBuf, String> {
    let manifest_dir = required_var("CARGO_MANIFEST_DIR")?;
    let out_dir = required_var("OUT_DIR")?;
    let profile = required_var("PROFILE")?;

    let build_type = match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        other => return Err(format!("未知 PROFILE '{other}'，无法映射 CMAKE_BUILD_TYPE")),
    };

    let native_dir = PathBuf::from(manifest_dir).join("../../native");
    // canonicalize 消除 ../ 与符号链接差异；路径可能含空格，全程走参数数组。
    let native_dir = native_dir
        .canonicalize()
        .map_err(|error| format!("native 目录不可达：{error}"))?;
    let binary_dir = PathBuf::from(out_dir).join("native-build");

    let cmake = std::env::var_os("CMAKE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cmake"));
    let generator = std::env::var("CMAKE_GENERATOR").unwrap_or_else(|_| "Ninja".to_string());

    // 有效配置与 native/CMakePresets.json 一致：差异项只有构建类型，
    // 其余默认值（安装前缀、compile_commands 导出）由 native/CMakeLists.txt 统一。
    let mut configure = Command::new(&cmake);
    configure
        .arg("-S")
        .arg(&native_dir)
        .arg("-B")
        .arg(&binary_dir)
        .arg("-G")
        .arg(&generator)
        .arg(format!("-DCMAKE_BUILD_TYPE={build_type}"));
    run_step("configure", &mut configure)?;

    let mut build = Command::new(&cmake);
    build.arg("--build").arg(&binary_dir);
    run_step("build", &mut build)?;

    let exe_name = if cfg!(windows) {
        "panta-native.exe"
    } else {
        "panta-native"
    };
    let product = binary_dir.join("app").join(exe_name);
    if !product.exists() {
        return Err(format!(
            "CMake 构建成功但未找到产物 {}；检查 native/app 的 OUTPUT_NAME 是否与 build.rs 定位约定一致",
            product.display()
        ));
    }
    Ok(product)
}

fn run_step(name: &str, command: &mut Command) -> Result<(), String> {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(match status.code() {
            Some(code) => format!("CMake {name} 失败（退出码 {code}），诊断见上方输出"),
            None => format!("CMake {name} 被信号终止，诊断见上方输出"),
        }),
        // program not found 等启动错误在此浮出，并给出可定位的处置指引。
        Err(error) => Err(format!(
            "无法执行 {:?}：{error}。请安装 CMake（Ninja 需在 PATH），或用 CMAKE 环境变量指定可执行文件；参见 ai-docs/standards/dependency-acquisition.md",
            command.get_program()
        )),
    }
}

fn required_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("Cargo 未提供环境变量 {name}，构建脚本运行环境异常"))
}
