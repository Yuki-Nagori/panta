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

// 供给逻辑与 build.rs 共享同一文件；单元测试经 src/main.rs 的 cfg(test)
// 模块运行（cargo test 不执行 build script 内的测试）。
#[path = "src/provision.rs"]
mod provision;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::SystemTime;

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
    "../panta-ffi",
    "../panta-ffi/include",
    "../panta-ffi/src",
    // panta-ffi 静态链接 panta-core；其源码变化刷新 staticlib 内容，
    // 需触发 CMake 重新链接。
    "../panta-core",
    "../panta-core/src",
    "../../native/ffi",
    "../../qml",
    "../../qml/Themes",
    "../../qml/Panels",
    // 扩展点：resources/ 与更多 QML 子目录落地时在此追加（任务 005 起）。
];

/// 影响配置结果的环境变量，变更即重建。CMAKE 可指定 cmake 可执行文件路径。
const RERUN_ENVS: &[&str] = &[
    "CMAKE",
    "CXX",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_PREFIX_PATH",
];

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

    // 保留标准绝对路径表示，不调用 canonicalize：Windows 上它会把盘符路径
    // 转成 `//?/D:` 长路径前缀，MinGW 会将其错误解析为 POSIX 根路径。路径可能
    // 含空格，全程走参数数组；CMake 会在读取 -S 时消解 `..`。
    let native_dir = PathBuf::from(manifest_dir).join("../../native");
    if !native_dir.is_dir() {
        return Err(format!("native 目录不可达：{}", native_dir.display()));
    }
    let binary_dir = PathBuf::from(&out_dir).join("native-build");
    // OUT_DIR = <target>/<profile>/build/<hash>/out；ancestors 跳过 out、
    // hash、build、profile 四层得到 target 根，托管工具缓存与 profile 无关。
    let target_root = PathBuf::from(&out_dir)
        .ancestors()
        .nth(4)
        .ok_or_else(|| format!("无法从 launcher OUT_DIR 推导 target 根：{out_dir}"))?
        .to_path_buf();

    // 托管引导（任务 020）：定位 → 缺失时按固定资产下载并校验。
    let cmake = provision::resolve_cmake(&target_root)?;
    let generator = std::env::var("CMAKE_GENERATOR").unwrap_or_else(|_| "Ninja".to_string());
    let ninja = if generator.to_ascii_lowercase().contains("ninja") {
        Some(provision::resolve_ninja(&target_root, &cmake)?)
    } else {
        None
    };

    // Cargo 的默认 feature 是唯一用户入口；把 feature 状态转换成 CMake
    // 选项，避免开发者在日常命令中重复维护两套开关。
    let bridge_module = if std::env::var_os("CARGO_FEATURE_BRIDGE_MODULE").is_some() {
        "ON"
    } else {
        "OFF"
    };
    let (ffi_include, ffi_staticlib) = ffi_artifacts(&PathBuf::from(&out_dir))?;

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
        .arg(format!("-DCMAKE_BUILD_TYPE={build_type}"))
        .arg(format!("-DPANTA_ENABLE_BRIDGE_MODULE={bridge_module}"))
        .arg("-DPANTA_ENABLE_FFI_TEST=ON")
        .arg(format!("-DPANTA_FFI_INCLUDE_DIR={}", ffi_include.display()))
        .arg(format!(
            "-DPANTA_FFI_STATIC_LIB={}",
            ffi_staticlib.display()
        ));
    if let Some(ninja) = &ninja {
        // 托管供给的 Ninja 不依赖 PATH；系统 Ninja 传显式路径同样无害。
        configure.arg(format!("-DCMAKE_MAKE_PROGRAM={}", ninja.display()));
    }
    run_step("configure", &mut configure)?;

    let mut build = Command::new(&cmake);
    // 单配置生成器会忽略 --config，多配置生成器（如 Windows Visual Studio）
    // 则必须显式选择与 Cargo profile 对应的配置。
    build
        .arg("--build")
        .arg(&binary_dir)
        .arg("--config")
        .arg(build_type);
    run_step("build", &mut build)?;

    let exe_name = if cfg!(windows) {
        "panta-native.exe"
    } else {
        "panta-native"
    };
    locate_product(&binary_dir, build_type, exe_name)
}

fn locate_product(
    binary_dir: &std::path::Path,
    build_type: &str,
    exe_name: &str,
) -> Result<PathBuf, String> {
    let base = binary_dir.join("app");
    // Ninja 等单配置生成器把产物直接放在 app/；Visual Studio 等多配置生成器
    // 可能放在构建树根部或 app/<Config>/。同时检查三种布局，避免生成器选择
    // 泄漏到 launcher。
    let candidates = [
        binary_dir.join(build_type).join(exe_name),
        base.join(exe_name),
        base.join(build_type).join(exe_name),
    ];
    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .ok_or_else(|| {
            let expected = candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" 或 ");
            format!(
                "CMake 构建成功但未找到产物（尝试：{expected}）；检查 native/app 的 OUTPUT_NAME 与生成器配置"
            )
        })
}

fn ffi_artifacts(out_dir: &Path) -> Result<(PathBuf, PathBuf), String> {
    let include = std::env::var_os("DEP_PANTA_FFI_INCLUDE")
        .map(PathBuf::from)
        .ok_or_else(|| "Cargo 未提供 DEP_PANTA_FFI_INCLUDE，panta-ffi 桥接头不可用".to_owned())?;
    if !include.join("panta_ffi.h").is_file() {
        return Err(format!(
            "panta-ffi 生成头不存在：{}",
            include.join("panta_ffi.h").display()
        ));
    }

    // build.rs 的 OUT_DIR 位于 target/<profile>/build/<pkg-hash>/out；Cargo
    // 将 staticlib 放在同一 profile 根目录（例如 target/debug/libpanta_ffi.a），
    // 而不是 deps/。只选当前 package 名称，避免把 rlib 或其它 profile 的旧产物
    // 传进 CMake；保留 deps/作为对旧版 Cargo/平台布局的兜底。
    let profile_dir = out_dir.ancestors().nth(3).ok_or_else(|| {
        format!(
            "无法从 launcher OUT_DIR 推导 profile：{}",
            out_dir.display()
        )
    })?;
    let deps_dir = profile_dir.join("deps");
    let mut candidates = Vec::new();
    for directory in [profile_dir, deps_dir.as_path()] {
        let entries = fs::read_dir(directory)
            .map_err(|error| format!("读取 panta-ffi 产物目录 {}：{error}", directory.display()))?;
        candidates.extend(
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                        return false;
                    };
                    let is_library = path
                        .extension()
                        .is_some_and(|extension| extension == "a" || extension == "lib");
                    is_library
                        && (name.starts_with("libpanta_ffi") || name.starts_with("panta_ffi"))
                }),
        );
    }
    let staticlib = candidates
        .iter()
        .find(|path| path.parent() == Some(profile_dir))
        .cloned()
        .or_else(|| {
            candidates.sort_by_key(|path| {
                fs::metadata(path)
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            });
            candidates.into_iter().next_back()
        })
        .ok_or_else(|| {
            format!(
                "未找到 panta-ffi staticlib（检查 {} 与 {}）",
                profile_dir.display(),
                deps_dir.display()
            )
        })?;
    Ok((include, staticlib))
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
