use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const TESTS_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const CARGO_DENY_VERSION: &str = "0.20.2";
const CARGO_MACHETE_VERSION: &str = "0.9.2";
const CARGO_LLVM_COV_VERSION: &str = "0.9.1";
/// Miri 只随 nightly 发布：固定日期保证可复现，升级时同步回填 032 验证表。
const MIRI_NIGHTLY: &str = "nightly-2026-09-15";
/// Miri 只解释纯 Rust crate：CXX FFI 调用与进程/构建类 crate（panta-build、
/// panta-tests、launcher）不在 Miri 语义内。
const MIRI_PACKAGES: &[&str] = &["panta-core", "panta-dsl-core", "panta-foundation"];

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let result = match arguments.next().as_deref() {
        Some("quality") => quality(),
        Some("test") => test_all(),
        Some("format") => {
            let mut rest: Vec<String> = arguments.collect();
            let check = take_check_flag(&mut rest);
            format_all(check, &rest)
        }
        Some("audit") => audit(),
        Some("coverage") => match arguments.next().as_deref() {
            None | Some("rust") => coverage(),
            Some("native") => native_coverage(),
            Some(other) => Err(format!("未知 coverage 类型：{other}").into()),
        },
        Some("sanitize") => sanitize(),
        // 对外叫 ub-check：别名 `miri` 会遮蔽 cargo-miri 外部子命令。
        Some("ub-check") => miri(),
        Some("toolchain") => verify_toolchain(),
        Some("lint") => {
            let mut rest: Vec<String> = arguments.collect();
            let check = take_check_flag(&mut rest);
            lint(rest.first().map(String::as_str), check)
        }
        Some(command) => Err(format!(
            "未知命令 '{command}'；可用：quality、test、audit、lint、format、coverage、sanitize、\
             ub-check、toolchain"
        )
        .into()),
        None => Err(
            "缺少命令；可用：quality、test、audit、lint、format、coverage、sanitize、ub-check、\
                 toolchain"
                .into(),
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("panta-tests: {error}");
            ExitCode::FAILURE
        }
    }
}

/// 从剩余参数摘除 `--check`（位置不限）。缺省行为是就地修复（写文件），
/// `--check` 只验证不改动；CI 与 pre-commit 一律带 `--check`。
fn take_check_flag(arguments: &mut Vec<String>) -> bool {
    let check = arguments.iter().any(|argument| argument == "--check");
    arguments.retain(|argument| argument != "--check");
    check
}

fn quality() -> Result<(), Box<dyn Error>> {
    format_all(true, &[])?;
    lint(None, true)?;
    audit()?;
    test_all()
}

fn audit() -> Result<(), Box<dyn Error>> {
    cargo("deny", ["check"])
}

fn coverage() -> Result<(), Box<dyn Error>> {
    let tool = ensure_cargo_tool("cargo-llvm-cov", CARGO_LLVM_COV_VERSION)?;
    let target_dir = target_root();
    let mut command = Command::new(tool);
    command
        // cargo-llvm-cov 的直接调用仍要求 Cargo 子命令名作为第一个参数。
        .arg("llvm-cov")
        .env("CARGO_TARGET_DIR", target_dir)
        .env("PANTA_TOOL_CACHE_ROOT", target_dir)
        .args([
            "--locked",
            "--workspace",
            "--exclude",
            "panta-launcher",
            // 聚合器依赖 native 构建树；由独立 native CI 验证，避免冷启动误失败。
            "--exclude",
            "panta-tests",
            // 构建支持从 launcher 迁出，保持原业务覆盖率口径；另有安装/数据库回归测试。
            "--exclude",
            "panta-build",
            "--summary-only",
            "--fail-under-functions",
            "89",
            "--fail-under-lines",
            "92",
        ])
        .current_dir(repository_root()?);
    run("cargo llvm-cov", command)
}

/// 覆盖率构建使用独立 CMake 树，与普通构建共享已校验的依赖缓存。
fn native_coverage() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let target = target_root();
    let report_dir = target.join("native-coverage");
    let profiles = report_dir.join("raw");
    if profiles.exists() {
        fs::remove_dir_all(&profiles)?;
    }
    fs::create_dir_all(&profiles)?;
    let profile_file = profiles.join("%m-%p.profraw");
    let mut build = Command::new("cargo");
    build
        .current_dir(root)
        .args(["build", "--locked", "-p", "panta-launcher", "--target-dir"])
        .arg(target)
        .env("PANTA_NATIVE_COVERAGE", "1");
    run("native coverage build", build)?;
    let native = target.join("native/debug-coverage");
    let mut envs = panta_build::native_test_env(target, env!("PANTA_TEST_HOST"))?;
    envs.push((
        std::ffi::OsString::from("LLVM_PROFILE_FILE"),
        profile_file.into_os_string(),
    ));
    run_ctest_in(&native, envs, "native coverage tests", None, None, None)?;
    let llvm = panta_build::resolve_llvm_compilers(target)?;
    let raw = fs::read_dir(&profiles)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "profraw"))
        .collect::<Vec<_>>();
    if raw.is_empty() {
        return Err("native 测试未生成覆盖率数据".into());
    }
    let merged = report_dir.join("native.profdata");
    let mut merge = Command::new(
        llvm.root
            .join("bin")
            .join(panta_build::exe_name("llvm-profdata")),
    );
    merge
        .args(["merge", "-sparse"])
        .args(raw)
        .arg("-o")
        .arg(&merged);
    run("native coverage merge", merge)?;
    let mut binaries = Vec::new();
    collect_test_binaries(&native, &mut binaries)?;
    binaries.sort();
    let first = binaries.first().ok_or("没有找到 native 覆盖率测试产物")?;
    let mut report = Command::new(
        llvm.root
            .join("bin")
            .join(panta_build::exe_name("llvm-cov")),
    );
    report
        .arg("report")
        .arg(first)
        .arg(format!("-instr-profile={}", merged.display()));
    for binary in binaries.iter().skip(1) {
        report.arg("-object").arg(binary);
    }
    report.args([
        "--ignore-filename-regex=(/target/|googletest|/usr/)",
        "--show-region-summary=false",
    ]);
    let output = report.output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    fs::write(report_dir.join("summary.txt"), &output.stdout)?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn collect_test_binaries(directory: &Path, result: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_test_binaries(&path, result)?;
        } else if path
            .file_stem()
            .is_some_and(|name| name.to_string_lossy().ends_with("_test"))
            && (path.extension().is_none() || path.extension().is_some_and(|ext| ext == "exe"))
        {
            result.push(path);
        }
    }
    Ok(())
}

/// sanitizer 矩阵（任务 042）：ASan+UBSan 是三平台主组合；TSan 与其他
/// sanitizer 运行库互斥，只能独立构建，且官方支持平台不含 Windows；clang-cl
/// 仅支持部分 UBSan 检查，Windows 矩阵只验证 ASan。官方依据见任务 042。
fn sanitizer_profiles() -> &'static [(&'static str, &'static str)] {
    if cfg!(windows) {
        &[("asan", "address")]
    } else {
        &[("asan-ubsan", "address,undefined"), ("tsan", "thread")]
    }
}

fn sanitize() -> Result<(), Box<dyn Error>> {
    for (name, flags) in sanitizer_profiles() {
        sanitize_profile(name, flags)?;
    }
    Ok(())
}

/// 每个组合构建独立的插桩 CMake 树并完整执行 CTest；UBSan 以
/// `-fno-sanitize-recover=undefined` 保证错误即非零退出，sanitizer 缺省
/// 退出码（ASan/UBSan 非零、TSan 66）经 CTest 原样失败。目录名由 flags
/// 派生，与 launcher build.rs 的构建树命名规则保持一致。
fn sanitize_profile(name: &str, flags: &str) -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let target = target_root();
    let mut build = Command::new("cargo");
    build
        .current_dir(root)
        .args(["build", "--locked", "-p", "panta-launcher", "--target-dir"])
        .arg(target)
        .env("PANTA_NATIVE_SANITIZER", flags);
    run(&format!("sanitizer {name} build"), build)?;
    let native = target
        .join("native")
        .join(format!("debug-sanitizer-{}", flags.replace(',', "-")));
    if !native.is_dir() {
        return Err(format!("sanitizer 构建树不存在：{}", native.display()).into());
    }
    // 组合级排除（边界登记 042）：tsan 下 Rust std 的 futex 锁不可见
    // （rust-lang/rust#110485），经 FFI 驱动 Rust 线程的测试只会确定性误报；
    // QML 测试栈（Qt/glib/系统库）连续三轮仅产出第三方噪声，无自有信号。
    // Windows asan 下未插桩 Qt DLL 走 ucrt/RTL 堆而 ASan 用自有分配器，
    // QML 引擎跨模块对象生命周期触发 bad-free。
    let exclude = match (name, cfg!(windows)) {
        ("tsan", _) => Some("^(TaskHost|Ffi|Qml)\\."),
        ("asan", true) => Some("^Qml\\."),
        _ => None,
    };
    run_ctest_in(
        &native,
        sanitizer_test_env(target, name)?,
        &format!("sanitizer {name} ctest"),
        None,
        None,
        exclude,
    )
}

/// sanitizer 测试环境：托管 LLVM bin 前置到 PATH，让运行时报告用配套
/// llvm-symbolizer 符号化；Windows 的 ASan 动态运行库 DLL 由编译器资源
/// 目录解析。LeakSanitizer 在 macOS 默认关闭，显式开启；第三方 SDK/系统
/// 运行时的已知问题按 `tests/lsan-suppressions.txt`（泄漏）与
/// `tests/tsan-suppressions.txt`（竞态，仅 tsan 组合）抑制，自有代码的
/// 问题报告仍然阻断。
fn sanitizer_test_env(
    target: &Path,
    profile: &str,
) -> Result<Vec<(std::ffi::OsString, std::ffi::OsString)>, Box<dyn Error>> {
    let environment = panta_build::native_test_env(target, env!("PANTA_TEST_HOST"))?;
    let llvm = panta_build::resolve_llvm_compilers(target)?;
    let mut prepend = vec![llvm.root.join("bin")];
    if cfg!(windows) {
        prepend.push(panta_build::compiler_rt_dll_dir(&llvm)?);
    }
    let mut environment = panta_build::prepend_path(environment, prepend)?;
    if !cfg!(windows) {
        let suppressions = repository_root()?.join("tests/lsan-suppressions.txt");
        environment.push((
            std::ffi::OsString::from("ASAN_OPTIONS"),
            std::ffi::OsString::from("detect_leaks=1"),
        ));
        environment.push((
            std::ffi::OsString::from("LSAN_OPTIONS"),
            std::ffi::OsString::from(format!(
                "suppressions={}",
                suppressions.to_str().ok_or("抑制清单路径不是 UTF-8")?
            )),
        ));
    }
    if profile == "tsan" {
        let suppressions = repository_root()?.join("tests/tsan-suppressions.txt");
        environment.push((
            std::ffi::OsString::from("TSAN_OPTIONS"),
            std::ffi::OsString::from(format!(
                "suppressions={}",
                suppressions.to_str().ok_or("抑制清单路径不是 UTF-8")?
            )),
        ));
    }
    Ok(environment)
}

/// Miri 解释执行纯 Rust crate 测试：工具链是固定日期 nightly（rustup 组件），
/// 产物走独立 target/miri，不污染常规构建缓存。路径/日志类测试按设计使用
/// 真实文件系统夹具，因此关闭隔离放行宿主文件操作；UB 与数据竞争检查不受
/// 隔离开关影响。
fn miri() -> Result<(), Box<dyn Error>> {
    let mut install = Command::new("rustup");
    install.args(["toolchain", "install", MIRI_NIGHTLY]).args([
        "--profile",
        "minimal",
        "--component",
        "miri",
        "--no-self-update",
    ]);
    run("安装 Miri 工具链", install)?;
    let mut test = Command::new("cargo");
    test.arg(format!("+{MIRI_NIGHTLY}"))
        .args(["miri", "test", "--locked"]);
    for package in MIRI_PACKAGES {
        test.args(["-p", package]);
    }
    test.env("MIRIFLAGS", "-Zmiri-disable-isolation")
        .env("CARGO_TARGET_DIR", target_root().join("miri"))
        .current_dir(repository_root()?);
    run("cargo miri test", test)
}

fn lint(tool: Option<&str>, check: bool) -> Result<(), Box<dyn Error>> {
    match tool {
        None => {
            eprintln!("cargo lint [1/8] Rust Clippy");
            lint(Some("clippy"), check)?;
            eprintln!("cargo lint [2/8] unused Cargo dependencies");
            lint(Some("machete"), check)?;
            eprintln!("cargo lint [3/8] CMake format and lint");
            lint(Some("cmake"), check)?;
            eprintln!("cargo lint [4/8] native build and QML metadata");
            build_launcher()?;
            eprintln!("cargo lint [5/8] qmllint");
            run_qmllint()?;
            build_qml_benchmark_moc()?;
            eprintln!("cargo lint [6/8] clang-tidy");
            scan_clang_tidy(false, check)?;
            eprintln!("cargo lint [7/8] include-cleaner");
            scan_clang_tidy(true, check)?;
            eprintln!("cargo lint [8/8] cppcheck");
            scan_cppcheck()
        }
        Some("clippy") if check => cargo(
            "clippy",
            [
                "--locked",
                "--workspace",
                "--all-targets",
                "--target-dir",
                "--",
                "-D",
                "warnings",
            ],
        ),
        // 修复模式先应用机器可修复建议，再落回检查模式确认剩余问题。
        Some("clippy") => {
            cargo(
                "clippy",
                [
                    "--fix",
                    "--locked",
                    "--workspace",
                    "--all-targets",
                    "--allow-dirty",
                ],
            )?;
            lint(Some("clippy"), true)
        }
        Some("machete") => cargo_scanner("machete", []),
        Some("cmake") => run_cmake_lint(),
        Some("qmllint") => {
            build_launcher()?;
            run_qmllint()
        }
        Some("clang-tidy") => run_clang_tidy(false, check),
        Some("includes") => run_clang_tidy(true, check),
        Some("cppcheck") => run_cppcheck(),
        Some(other) => Err(format!(
            "未知 lint 工具 '{other}'；可用：clippy、machete、cmake、qmllint、clang-tidy、\
             includes、cppcheck；--check 只检查不修复"
        )
        .into()),
    }
}

fn format_all(check: bool, rest: &[String]) -> Result<(), Box<dyn Error>> {
    if let Some(unexpected) = rest.first() {
        return Err(format!(
            "format 不接受位置参数 '{unexpected}'；修复为缺省行为，\
                            --check 只验证不改动"
        )
        .into());
    }
    if check {
        cargo("fmt", ["--all", "--", "--check"])?;
    } else {
        cargo("fmt", ["--all"])?;
    }
    check_cpp_format(check)?;
    run_cmake_format(check)?;
    run_qml_format(check)?;
    if check {
        println!("format 检查通过");
    } else {
        println!("format 完成");
    }
    Ok(())
}

fn verify_toolchain() -> Result<(), Box<dyn Error>> {
    let target_root = target_root();
    let native_dir = native_build_dir();
    let cmake = panta_build::resolve_cmake(target_root)?;
    let llvm = panta_build::resolve_llvm_compilers(target_root)?;
    let clang_format = &llvm.clang_format;
    let llvm_root = &llvm.root;
    let llvm_version = panta_build::LLVM_VERSION;
    let managed_root = target_root.join("panta-tools");
    let managed_cmake = managed_root.join("cmake");
    let managed_llvm = managed_root.join("llvm");
    let ninja = panta_build::resolve_ninja(target_root, &cmake)?;
    let qt_bin = target_root.join("panta-deps/qt/staging/bin");
    let googletest_triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "macos-arm64",
        ("linux", "x86_64") => "linux-x86_64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => return Err(format!("不支持的 GoogleTest SDK 平台：{os}/{arch}").into()),
    };
    let googletest = target_root
        .join("panta-deps/sdk/googletest/1.18.0")
        .join(googletest_triple)
        .join("lib/cmake/GTest/GTestConfig.cmake");
    let compile_database = native_dir.join("compile_commands.json");

    require_file("托管 CMake", &cmake)?;
    require_file("托管 Ninja", &ninja)?;
    require_file("托管 clang-format", clang_format)?;
    require_file(
        "托管 clang",
        &llvm_root.join("bin").join(panta_build::exe_name("clang")),
    )?;
    require_file(
        "托管 clang++",
        &llvm_root.join("bin").join(panta_build::exe_name("clang++")),
    )?;
    if cfg!(windows) {
        require_file(
            "托管 clang-cl",
            &llvm_root
                .join("bin")
                .join(panta_build::exe_name("clang-cl")),
        )?;
    }
    let version_binary = if cfg!(windows) {
        llvm_root
            .join("bin")
            .join(panta_build::exe_name("clang-cl"))
    } else {
        llvm_root.join("bin").join(panta_build::exe_name("clang++"))
    };
    require_tool_version(&version_binary, llvm_version)?;
    require_file(
        "Qt qmlformat",
        &qt_bin.join(panta_build::exe_name("qmlformat")),
    )?;
    require_file("Qt qmllint", &qt_bin.join(panta_build::exe_name("qmllint")))?;
    require_file("GoogleTest SDK", &googletest)?;
    require_file("native compile_commands.json", &compile_database)?;
    if !cmake.starts_with(&managed_cmake) {
        return Err(format!(
            "CMake 未使用 Cargo 托管资产：{}（期望位于 {}）",
            cmake.display(),
            managed_cmake.display()
        )
        .into());
    }
    if !llvm_root.starts_with(&managed_llvm) || !clang_format.starts_with(&managed_llvm) {
        return Err(format!(
            "LLVM 工具未使用 Cargo 托管资产：root={} clang-format={}（期望位于 {}）",
            llvm_root.display(),
            clang_format.display(),
            managed_llvm.display()
        )
        .into());
    }
    for tool in ["clang-tidy", "llvm-cov", "llvm-profdata"] {
        let path = llvm.root.join("bin").join(panta_build::exe_name(tool));
        require_file(tool, &path)?;
        require_tool_version(&path, llvm_version)?;
    }
    let cache = fs::read_to_string(native_dir.join("CMakeCache.txt"))?;
    for (key, expected) in [
        ("CMAKE_CXX_COMPILER", version_binary.as_path()),
        (
            "CMAKE_C_COMPILER",
            if cfg!(windows) {
                version_binary.as_path()
            } else {
                llvm.clang.as_path()
            },
        ),
        ("CMAKE_COMMAND", cmake.as_path()),
        ("CMAKE_MAKE_PROGRAM", ninja.as_path()),
    ] {
        let actual = cache
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once('=')?;
                (name.split_once(':')?.0 == key).then_some(value)
            })
            .ok_or_else(|| format!("CMakeCache 缺少 {key}"))?;
        if Path::new(actual).canonicalize()? != expected.canonicalize()? {
            return Err(format!("{key} 实际使用 {actual}，期望 {}", expected.display()).into());
        }
    }
    let commands = panta_build::database::read(&native_dir.join("quality/compile_commands.json"))?;
    panta_build::database::verify(&commands, &version_binary)?;
    if !commands.iter().any(|entry| {
        entry
            .source()
            .ends_with("crates/panta-ffi/src/ffi_support.cc")
    }) {
        return Err("编译数据库缺少手写 CXX adapter".into());
    }
    println!(
        "Cargo 工具链已就绪：LLVM={} CMake={} Ninja={} Qt={} GoogleTest={} compile_commands={}",
        llvm_root.display(),
        cmake.display(),
        ninja.display(),
        qt_bin.display(),
        googletest.display(),
        compile_database.display()
    );
    Ok(())
}

fn require_tool_version(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|error| format!("运行 {} 失败：{error}", path.display()))?;
    if !output.status.success() {
        return Err(format!("{} --version 失败：{}", path.display(), output.status).into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.contains(expected) && !stderr.contains(expected) {
        return Err(format!(
            "{} 版本不匹配：期望 LLVM {}，实际 {}{}",
            path.display(),
            expected,
            stdout.trim(),
            stderr.trim()
        )
        .into());
    }
    Ok(())
}

fn require_file(label: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{label} 不存在：{}", path.display()).into())
    }
}

fn test_all() -> Result<(), Box<dyn Error>> {
    cargo(
        "test",
        [
            "--locked",
            "--workspace",
            "--exclude",
            "panta-tests",
            "--target-dir",
        ],
    )?;
    build_launcher()?;
    run_qmllint_and_ctest()
}

fn cargo(
    subcommand: &str,
    args: impl IntoIterator<Item = &'static str>,
) -> Result<(), Box<dyn Error>> {
    let (description, command) = cargo_command(subcommand, args)?;
    run(&description, command)
}

/// 扫描类工具的静默变体：通过时不转发输出，失败时原样附上全部诊断。
fn cargo_scanner(
    subcommand: &str,
    args: impl IntoIterator<Item = &'static str>,
) -> Result<(), Box<dyn Error>> {
    let (description, command) = cargo_command(subcommand, args)?;
    run_scanner(&description, command)
}

fn cargo_command(
    subcommand: &str,
    args: impl IntoIterator<Item = &'static str>,
) -> Result<(String, Command), Box<dyn Error>> {
    let target_dir = target_root();
    // deny/machete 走托管的独立二进制；其余是 cargo 子命令。
    let managed_scanner = matches!(subcommand, "deny" | "machete");
    let mut command = if managed_scanner {
        let (name, version) = if subcommand == "deny" {
            ("cargo-deny", CARGO_DENY_VERSION)
        } else {
            ("cargo-machete", CARGO_MACHETE_VERSION)
        };
        Command::new(ensure_cargo_tool(name, version)?)
    } else {
        let mut command = Command::new("cargo");
        // -q 去掉 Compiling/Finished 状态行；构建脚本与诊断仍透传。
        command.arg("-q").arg(subcommand);
        command
    };
    if env!("PANTA_TEST_BUILD_TYPE") == "Release"
        && matches!(subcommand, "build" | "test" | "clippy")
    {
        command.arg("--release");
    }
    let mut inserted_target_dir = false;
    for arg in args {
        if arg == "--target-dir" {
            command.arg(arg).arg(target_dir);
            inserted_target_dir = true;
        } else {
            command.arg(arg);
        }
    }
    if (subcommand == "build" || subcommand == "test" || subcommand == "clippy")
        && !inserted_target_dir
    {
        command.arg("--target-dir").arg(target_dir);
    }
    command.current_dir(repository_root()?);
    Ok((format!("cargo {subcommand}"), command))
}

/// 扫描类工具的静默运行：通过时零输出，失败时原样转发全部捕获输出。
fn run_scanner(description: &str, mut command: Command) -> Result<(), Box<dyn Error>> {
    let output = command
        .output()
        .map_err(|error| format!("{description} 启动失败：{error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "{description} 失败（退出码 {:?}）\n{stdout}{stderr}",
            output.status.code()
        )
        .into())
    }
}

fn ensure_cargo_tool(name: &str, version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let target_root = target_root();
    let root = panta_build::install_directory(target_root, name, version, |staging| {
        let mut command = Command::new("cargo");
        command
            .args(["install", "--root"])
            .arg(staging)
            .args([name, "--locked", "--version", &format!("={version}")])
            .env(
                "CARGO_TARGET_DIR",
                target_root.join("panta-tools/cargo-build").join(name),
            );
        run(&format!("安装 {name} {version}"), command).map_err(|e| e.to_string())?;
        if !staging
            .join("bin")
            .join(panta_build::exe_name(name))
            .is_file()
        {
            return Err(format!("安装未生成 {name}"));
        }
        Ok(())
    })?;
    Ok(root.join("bin").join(panta_build::exe_name(name)))
}

fn build_launcher() -> Result<(), Box<dyn Error>> {
    cargo("build", ["--locked", "-p", "panta-launcher"])
}

fn run_qmllint_and_ctest() -> Result<(), Box<dyn Error>> {
    run_qmllint()?;
    run_ctest(None)
}

/// 在指定 CMake 树执行完整 CTest；环境由调用方准备（常规树、覆盖率或
/// sanitizer 各有差异），参数与 `cargo coverage native` 保持一致。
fn run_ctest_in(
    native_dir: &Path,
    envs: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    description: &str,
    configuration: Option<&str>,
    regex: Option<&str>,
    exclude: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let cmake = panta_build::resolve_cmake(target_root())?;
    let ctest = cmake.with_file_name(panta_build::exe_name("ctest"));
    if !ctest.is_file() {
        return Err(format!("ctest 不存在：{}", ctest.display()).into());
    }
    let mut test = Command::new(ctest);
    test.envs(envs);
    test.current_dir(native_dir)
        .args(["--output-on-failure", "--no-tests=error"]);
    if let Some(configuration) = configuration {
        test.args(["-C", configuration]);
    }
    if let Some(regex) = regex {
        test.args(["-R", regex]);
    }
    if let Some(exclude) = exclude {
        test.args(["-E", exclude]);
    }
    run(description, test)
}

fn run_qml_format(check: bool) -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let target_root = target_root();
    let qmlformat = qmlformat_path(target_root);
    if !qmlformat.is_file() {
        provision_qml_format(root, target_root)?;
    }
    let qmlformat = qmlformat_path(target_root);
    if !qmlformat.is_file() {
        return Err(format!("qmlformat 不存在：{}", qmlformat.display()).into());
    }
    if !check {
        // 修复模式：直接就地格式化；文件清单与检查脚本（check-qml-format.cmake
        // 按 QML_DIR 递归）保持同一来源。
        let mut files = qml_sources(&root.join("qml"))?;
        files.sort();
        for file in files {
            let mut command = Command::new(&qmlformat);
            command.arg("-i").arg(&file);
            run(&format!("qmlformat {}", file.display()), command)?;
        }
        return Ok(());
    }
    let cmake = panta_build::resolve_cmake(target_root)?;
    let script = root.join("native/cmake/tests/check-qml-format.cmake");
    let mut command = Command::new(cmake);
    command.arg(format!(
        "-DQMLFMT={}",
        qmlformat.to_str().ok_or("qmlformat 路径不是 UTF-8")?
    ));
    command.arg(format!(
        "-DQML_DIR={}",
        root.join("qml").to_str().ok_or("QML 路径不是 UTF-8")?
    ));
    command.args(["-P", script.to_str().ok_or("QML 格式脚本路径不是 UTF-8")?]);
    run("qmlformat", command)
}

fn qml_sources(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            files.extend(qml_sources(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "qml") {
            files.push(path);
        }
    }
    Ok(files)
}

fn qmlformat_path(target_root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "qmlformat.exe"
    } else {
        "qmlformat"
    };
    target_root.join("panta-deps/qt/staging/bin").join(name)
}

fn provision_qml_format(root: &Path, target_root: &Path) -> Result<(), Box<dyn Error>> {
    let cmake = panta_build::resolve_cmake(target_root)?;
    let mut command = Command::new(cmake);
    command
        .arg(format!(
            "-DQT_PROVISION_DIR={}",
            target_root.join("panta-deps/qt").display()
        ))
        .arg("-P")
        .arg(root.join("native/cmake/qt-provision.cmake"));
    run("准备 qmlformat", command)
}

fn run_clang_tidy(includes_only: bool, check: bool) -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    build_qml_benchmark_moc()?;
    scan_clang_tidy(includes_only, check)
}

fn scan_clang_tidy(includes_only: bool, check: bool) -> Result<(), Box<dyn Error>> {
    let llvm = panta_build::resolve_llvm_compilers(target_root())?;
    let tool = llvm
        .root
        .join("bin")
        .join(panta_build::exe_name("clang-tidy"));
    let directory = native_build_dir().join("quality");
    let commands = panta_build::database::read(&directory.join("compile_commands.json"))?;
    if commands.is_empty() {
        return Err("clang-tidy 翻译单元清单为空".into());
    }
    let env = panta_build::native_test_env(target_root(), env!("PANTA_TEST_HOST"))?;
    // TU 之间相互独立、--fix 也只写各自源文件，按核数分片并行缩短最重
    // 门禁（单遍串行要数分钟）；单 TU 失败不短路，聚合后原样转发完整
    // 诊断。
    let workers = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .clamp(1, 8);
    let chunk_size = commands.len().div_ceil(workers);
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = commands
            .chunks(chunk_size)
            .map(|chunk| {
                let tool = &tool;
                let env = &env;
                let directory = &directory;
                scope.spawn(move || -> Vec<String> {
                    let mut local = Vec::new();
                    for entry in chunk {
                        let file = entry.source();
                        let mut command = Command::new(tool);
                        command.envs(env.clone());
                        command.arg("-p").arg(directory).arg("-quiet");
                        if includes_only {
                            command.arg("--checks=-*,misc-include-cleaner");
                        }
                        if !check {
                            command.arg("--fix");
                        }
                        command.arg(&file);
                        if let Err(error) =
                            run_scanner(&format!("clang-tidy {}", file.display()), command)
                        {
                            local.push(error.to_string());
                        }
                    }
                    local
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| vec!["clang-tidy 工作线程异常退出".to_owned()])
            })
            .collect()
    });
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

/// QML benchmark 默认不构建，但仍在 clang-tidy 编译数据库中且包含 moc 输出。
fn build_qml_benchmark_moc() -> Result<(), Box<dyn Error>> {
    let mut command = cmake_build_command()?;
    command.args([
        "--target",
        "panta_qml_cpu_benchmark_autogen",
        "panta_qml_gpu_benchmark_autogen",
        "--parallel",
    ]);
    run("生成 QML benchmark moc", command)
}

fn run_cppcheck() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    scan_cppcheck()
}

fn scan_cppcheck() -> Result<(), Box<dyn Error>> {
    let database = native_build_dir().join("quality/cppcheck.json");
    let mut entries = panta_build::database::read(&database)?;
    let root = repository_root()?;
    // Qt 6.11 的编译器宏超出 Cppcheck 解析能力；使用 Qt 库模型，LLVM 负责真实头文件。
    for entry in &mut entries {
        entry.exclude_include_root(&target_root().join("panta-deps/qt"))?;
    }
    let database = database.with_file_name("cppcheck-configured.json");
    panta_build::database::write(&database, &entries)?;
    let mut command = panta_build::python::command(target_root(), repository_root()?, "cppcheck")?;
    command
        .arg(format!("--project={}", database.display()))
        // 只补充跨翻译单元的未使用函数；常规诊断由 clang-tidy 负责。
        .args([
            "--enable=unusedFunction",
            "--error-exitcode=1",
            "--inline-suppr",
            "--quiet",
            "--check-level=exhaustive",
            "--suppress=missingIncludeSystem",
            // CXX Slice 迭代器的类型擦除被 Cppcheck 误判；仅排除其固定生成头。
            "--suppress=eraseDereference:*cxxbridge/include/rust/cxx.h",
        ]);
    command.arg(format!(
        "--library=qt,googletest,{}",
        root.join("tests/cppcheck-qt.cfg").display()
    ));
    command.arg(if cfg!(windows) {
        "--platform=win64"
    } else {
        "--platform=unix64"
    });
    command.arg(format!(
        "--suppress=unusedFunction:{}/*",
        target_root().display()
    ));
    command.arg(format!(
        "--suppress-xml={}",
        repository_root()?
            .join("tests/cppcheck-suppressions.xml")
            .display()
    ));
    run_scanner("cppcheck", command)
}

fn cmake_build_command() -> Result<Command, Box<dyn Error>> {
    let native_dir = native_build_dir();
    let mut command = Command::new(panta_build::resolve_cmake(target_root())?);
    command
        .envs(panta_build::native_test_env(
            target_root(),
            env!("PANTA_TEST_HOST"),
        )?)
        .current_dir(native_dir)
        .args(["--build"])
        .arg(native_dir)
        .args(["--config", env!("PANTA_TEST_BUILD_TYPE")]);
    Ok(command)
}

fn run_qmllint() -> Result<(), Box<dyn Error>> {
    let mut command = cmake_build_command()?;
    command.args(["--target", "all_qmllint"]);
    // 通过时静默：Qt 生成的 no-op qmllint 目标会回显 "Nothing to do"，
    // 失败时原样转发全部输出（含 qmllint 诊断）。
    run_scanner("qmllint", command)
}

fn run_ctest(regex: Option<&str>) -> Result<(), Box<dyn Error>> {
    let native_dir = native_build_dir();
    let envs = panta_build::native_test_env(target_root(), env!("PANTA_TEST_HOST"))?;
    run_ctest_in(
        native_dir,
        envs,
        "ctest",
        Some(env!("PANTA_TEST_BUILD_TYPE")),
        regex,
        None,
    )
}

fn check_cpp_format(check: bool) -> Result<(), Box<dyn Error>> {
    let llvm = panta_build::resolve_llvm_compilers(target_root())?;
    let clang_format = &llvm.clang_format;
    if !clang_format.is_file() {
        return Err(format!("clang-format 不存在：{}", clang_format.display()).into());
    }
    let root = repository_root()?;
    let mut files = cpp_sources(&root.join("native"))?;
    files.extend(cpp_sources(&root.join("tests/cpp"))?);
    files.extend(cpp_sources(&root.join("tests/qml"))?);
    files.extend(cpp_sources(&root.join("crates/panta-ffi/src"))?);
    files.extend(cpp_sources(&root.join("crates/panta-ffi/include"))?);
    if files.is_empty() {
        return Err("C++ 源文件清单为空".into());
    }
    files.sort();
    let mut command = Command::new(clang_format);
    if check {
        command.args(["--dry-run", "-Werror"]);
    } else {
        command.arg("-i");
    }
    command.args(files.iter());
    run(&format!("clang-format（{} 个文件）", files.len()), command)
}

fn run_cmake_format(check: bool) -> Result<(), Box<dyn Error>> {
    if check {
        run_cmake_tool("cmake-format", &["--check"])
    } else {
        // cmake-format 缺省把格式化结果打印到 stdout，必须显式 --in-place。
        run_cmake_tool("cmake-format", &["--in-place"])
    }
}

fn run_cmake_lint() -> Result<(), Box<dyn Error>> {
    run_cmake_tool("cmake-lint", &[])
}

fn run_cmake_tool(tool: &str, arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let mut command = panta_build::python::command(target_root(), root, tool)?;
    command.args(arguments).args(cmake_files(root)?);
    run_scanner(tool, command)
}

fn target_root() -> &'static Path {
    Path::new(env!("PANTA_TEST_TARGET_DIR"))
}

fn native_build_dir() -> &'static Path {
    Path::new(env!("PANTA_TEST_NATIVE_DIR"))
}

fn cpp_sources(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            files.extend(cpp_sources(&path)?);
        } else if path.extension().is_some_and(|extension| {
            matches!(extension.to_str(), Some("cpp" | "cc" | "cxx" | "h" | "hpp"))
        }) {
            files.push(path);
        }
    }
    Ok(files)
}

fn cmake_sources(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            files.extend(cmake_sources(&path)?);
        } else if path
            .file_name()
            .is_some_and(|name| name == "CMakeLists.txt")
            || path
                .extension()
                .is_some_and(|extension| extension == "cmake")
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn cmake_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for directory in ["native", "qml", "tools"] {
        files.extend(cmake_sources(&root.join(directory))?);
    }
    if files.is_empty() {
        return Err("CMake 源文件清单为空".into());
    }
    files.sort();
    Ok(files)
}

fn repository_root() -> Result<&'static Path, Box<dyn Error>> {
    Path::new(TESTS_MANIFEST_DIR)
        .parent()
        .ok_or_else(|| "tests package 必须位于仓库根目录下".into())
}

fn run(description: &str, mut command: Command) -> Result<(), Box<dyn Error>> {
    let status = command
        .status()
        .map_err(|error| format!("{description} 启动失败：{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{description} 失败（退出码 {:?}）", status.code()).into())
    }
}
