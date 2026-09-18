//! Cargo→CMake 调度（任务 004）：以与 native presets 相同的有效配置构建
//! native 树，并把可执行产物路径在编译期注入 launcher（PANTA_NATIVE_BIN）。
//!
//! 图约束：Cargo → 本脚本 → CMake/Ninja → native targets；CMake 侧不回调
//! Cargo，无递归。未来 C++ 消费 Rust 库时，接入点是 CMake 侧导入 Rust 产物
//! （任务 006 定），不得从本脚本再次触发 Cargo。
//!
//! 产物和依赖缓存位于 Cargo target；原生诊断继承到 Cargo 输出。

use panta_build as provision;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::SystemTime;

/// Cargo 递归追踪源码目录；第三方与生成文件不在监听范围内。
const RERUN_PATHS: &[&str] = &[
    "build.rs",
    "../../native",
    "../../tests/cpp",
    "../../tests/qml",
    "../panta-ffi",
    "../panta-core",
    "../../qml",
    "../../resources/i18n",
];

/// 影响配置结果的环境变量，变更即重建。系统工具旁路必须显式开启。
const RERUN_ENVS: &[&str] = &[
    "CMAKE",
    "PANTA_USE_SYSTEM_TOOLS",
    "PANTA_TOOL_CACHE_ROOT",
    "CC",
    "CXX",
    "CLANG_FORMAT",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_PREFIX_PATH",
    "PANTA_NATIVE_COVERAGE",
    "DEVELOPER_DIR",
    "MACOSX_DEPLOYMENT_TARGET",
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

    // 路径可能含空格，全程走参数数组；CMake 在读取 -S 时消解 `..`。
    let native_dir = PathBuf::from(&manifest_dir).join("../../native");
    if !native_dir.is_dir() {
        return Err(format!("native 目录不可达：{}", native_dir.display()));
    }
    let target_root = provision::target_root(Path::new(&out_dir))?;
    let native_profile = if std::env::var("PANTA_NATIVE_COVERAGE").as_deref() == Ok("1") {
        format!("{profile}-coverage")
    } else {
        profile.clone()
    };
    let binary_dir = target_root.join("native").join(native_profile);
    let deps_root = target_root.join("panta-deps");

    // i18n（任务 034）：解析 resources/i18n/*.pa 并把语言字典写成构建树
    // TS，供 CMake 侧锁定 lrelease 编 QM。解析失败立即终止构建，旧 TS 保持
    // 不动；与 panta-dslc CLI 共享 panta-dsl-core，无第二套 parser。
    let i18n_dir = PathBuf::from(&manifest_dir).join("../../resources/i18n");
    let ts_dir = binary_dir.join("i18n");
    emit_translation_sources(&i18n_dir, &ts_dir)?;
    // 托管引导（任务 020）：定位 → 缺失时按固定资产下载并校验。
    let cmake = provision::resolve_cmake(&target_root)?;
    let llvm = provision::resolve_llvm_compilers(&target_root)?;
    if std::env::var("CMAKE_GENERATOR").is_ok_and(|value| value != "Ninja") {
        return Err("统一构建仅支持 Ninja，以保证三平台导出真实 compile_commands.json".into());
    }
    let ninja = provision::resolve_ninja(&target_root, &cmake)?;
    let sdk_env = provision::windows_sdk_env(&required_var("TARGET")?)?;

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
        .arg("Ninja")
        .arg(format!("-DCMAKE_BUILD_TYPE={build_type}"))
        .arg(format!("-DPANTA_ENABLE_BRIDGE_MODULE={bridge_module}"))
        .arg("-DPANTA_ENABLE_FFI_TEST=ON")
        .arg(format!("-DPANTA_FFI_INCLUDE_DIR={}", ffi_include.display()))
        .arg(format!(
            "-DPANTA_FFI_STATIC_LIB={}",
            ffi_staticlib.display()
        ))
        .arg(format!(
            "-DQT_PROVISION_DIR={}",
            deps_root.join("qt").display()
        ))
        .arg(format!(
            "-DFETCHCONTENT_BASE_DIR={}",
            deps_root.join("fetchcontent").display()
        ))
        // VTK/OCCT/Netgen 预编译 SDK 缓存根（任务 031）：与 Qt/ googletest
        // 一样跨 profile 共享；presets 直接 configure 时默认构建树内。
        .arg(format!(
            "-DPANTA_SDK_PROVISION_DIR={}",
            deps_root.join("sdk").display()
        ))
        .arg(format!("-DPANTA_I18N_TS_DIR={}", ts_dir.display()));
    let cxx_compiler = if cfg!(windows) {
        llvm.clang_cl.as_ref().ok_or("LLVM 缺少 clang-cl")?
    } else {
        &llvm.clangxx
    };
    let c_compiler = if cfg!(windows) {
        cxx_compiler
    } else {
        &llvm.clang
    };
    configure
        .envs(sdk_env.clone())
        .arg(format!("-DCMAKE_C_COMPILER={}", c_compiler.display()))
        .arg(format!("-DCMAKE_CXX_COMPILER={}", cxx_compiler.display()))
        .arg(format!("-DCMAKE_MAKE_PROGRAM={}", ninja.display()))
        .arg(format!("-DPANTA_LLVM_VERSION={}", provision::LLVM_VERSION))
        // 每次显式传 ON/OFF，避免上次覆盖率或系统旁路污染 CMakeCache。
        .arg(format!(
            "-DPANTA_USE_SYSTEM_TOOLS={}",
            if provision::use_system_tools() {
                "ON"
            } else {
                "OFF"
            }
        ))
        .arg(format!(
            "-DPANTA_ENABLE_COVERAGE={}",
            if std::env::var("PANTA_NATIVE_COVERAGE").as_deref() == Ok("1") {
                "ON"
            } else {
                "OFF"
            }
        ));
    if let Some(sdk) = provision::macos_sdk()? {
        configure.arg(format!("-DCMAKE_OSX_SYSROOT={}", sdk.display()));
        let deployment = match std::env::var("MACOSX_DEPLOYMENT_TARGET") {
            Ok(version) => version,
            Err(_) => {
                let output = Command::new("xcrun")
                    .args(["--sdk", "macosx", "--show-sdk-version"])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !output.status.success() {
                    return Err("无法查询 Apple SDK 版本".into());
                }
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            }
        };
        configure.arg(format!("-DCMAKE_OSX_DEPLOYMENT_TARGET={deployment}"));
    }
    run_step("configure", &mut configure)?;

    let mut build = Command::new(&cmake);
    build
        .envs(sdk_env)
        .arg("--build")
        .arg(&binary_dir)
        .arg("--config")
        .arg(build_type);
    run_step("build", &mut build)?;

    let ffi_database = PathBuf::from(required_var("DEP_PANTA_FFI_COMPILE_DATABASE")?);
    let mut commands = provision::database::read(&binary_dir.join("compile_commands.json"))?;
    commands.extend(provision::database::read(&ffi_database)?);
    let root = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(Path::parent)
        .ok_or("launcher 必须位于 crates/launcher")?
        .to_path_buf();
    let quality_dir = binary_dir.join("quality");
    fs::create_dir_all(&quality_dir).map_err(|e| e.to_string())?;
    let owned = provision::database::owned(commands.clone(), &root);
    provision::database::write(&quality_dir.join("compile_commands.json"), &owned)?;
    // 未使用函数分析需要看到 moc/CXX 生成的调用边；生成代码本身不作为告警对象。
    let mut cppcheck = owned;
    cppcheck.extend(commands.into_iter().filter(|entry| {
        entry
            .file
            .file_name()
            .is_some_and(|name| name == "mocs_compilation.cpp" || name == "lib.rs.cc")
    }));
    provision::database::write(&quality_dir.join("cppcheck.json"), &cppcheck)?;

    let exe_name = if cfg!(windows) {
        "panta-native.exe"
    } else {
        "panta-native"
    };
    let product = binary_dir.join("app").join(exe_name);
    if !product.is_file() {
        return Err(format!("native 产物不存在：{}", product.display()));
    }
    Ok(product)
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

fn emit_translation_sources(i18n_dir: &Path, ts_dir: &Path) -> Result<(), String> {
    let entries = fs::read_dir(i18n_dir)
        .map_err(|error| format!("读取 i18n 目录 {} 失败：{error}", i18n_dir.display()))?;
    let mut sources = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pa"))
        .collect::<Vec<_>>();
    sources.sort();
    if sources.is_empty() {
        return Err(format!(
            "{} 下没有 .pa 字典；语言字典缺失时构建不支持继续",
            i18n_dir.display()
        ));
    }

    fs::create_dir_all(ts_dir)
        .map_err(|error| format!("创建 TS 目录 {} 失败：{error}", ts_dir.display()))?;
    for source in &sources {
        let bytes =
            fs::read(source).map_err(|error| format!("读取 {} 失败：{error}", source.display()))?;
        let text = String::from_utf8(bytes).map_err(|error| {
            format!(
                "pa.invalid_utf8 at byte {} in {}",
                error.utf8_error().valid_up_to(),
                source.display()
            )
        })?;
        let document = panta_dsl_core::parse(&text)
            .map_err(|error| format!("{}：{error}", source.display()))?;
        if document.kind != panta_dsl_core::Kind::Language {
            // theme/variables 字典在此只做校验；其消费方（025/030）接入前不
            // 生成任何产物。
            continue;
        }
        let locale = document.language.clone().ok_or_else(|| {
            format!(
                "{}：语言字典缺少 language 头，无法生成 TS",
                source.display()
            )
        })?;
        let catalog = document
            .catalog
            .clone()
            .unwrap_or_else(|| "panta".to_owned());
        let ts_name = locale.replace('-', "_");
        let ts = panta_dsl_core::emit_ts(&document, &locale)
            .map_err(|error| format!("{}：{error}", source.display()))?;
        let destination = ts_dir.join(format!("{catalog}_{ts_name}.ts"));
        // 临时文件 + rename：写一半失败不破坏上一份有效 TS。
        let temporary = destination.with_extension("ts.tmp");
        fs::write(&temporary, ts.as_bytes())
            .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
        fs::rename(&temporary, &destination)
            .map_err(|error| format!("替换 {} 失败：{error}", destination.display()))?;
    }
    Ok(())
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
            "无法执行 {:?}：{error}。请检查托管工具缓存与平台 SDK；参见 ai-docs/standards/dependency-acquisition.md",
            command.get_program()
        )),
    }
}

fn required_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("Cargo 未提供环境变量 {name}，构建脚本运行环境异常"))
}
