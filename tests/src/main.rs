use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const TESTS_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const CARGO_DENY_VERSION: &str = "0.20.2";
const CARGO_MACHETE_VERSION: &str = "0.9.2";
const CARGO_LLVM_COV_VERSION: &str = "0.9.1";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let result = match arguments.next().as_deref() {
        Some("quality") => quality(),
        Some("test") => test_all(),
        Some("format") => format_all(),
        Some("audit") => audit(),
        Some("coverage") => match arguments.next().as_deref() {
            None | Some("rust") => coverage(),
            Some("native") => native_coverage(),
            Some(other) => Err(format!("未知 coverage 类型：{other}").into()),
        },
        Some("toolchain") => verify_toolchain(),
        Some("lint") => lint(arguments.next().as_deref()),
        Some(command) => Err(format!(
            "未知命令 '{command}'；可用：quality、audit、lint、test、format、coverage、toolchain"
        )
        .into()),
        None => {
            Err("缺少命令；可用：quality、audit、lint、test、format、coverage、toolchain".into())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("panta-tests: {error}");
            ExitCode::FAILURE
        }
    }
}

fn quality() -> Result<(), Box<dyn Error>> {
    format_all()?;
    lint(None)?;
    audit()?;
    test_all()
}

fn audit() -> Result<(), Box<dyn Error>> {
    cargo("deny", ["check"])
}

fn coverage() -> Result<(), Box<dyn Error>> {
    let tool = ensure_cargo_tool("cargo-llvm-cov", CARGO_LLVM_COV_VERSION)?;
    let target_dir = Path::new(env!("PANTA_TEST_TARGET_DIR"));
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
    let cmake = panta_build::resolve_cmake(target)?;
    let mut test = Command::new(cmake.with_file_name(executable_name("ctest")));
    test.envs(panta_build::native_test_env(
        target,
        &native,
        env!("PANTA_TEST_HOST"),
    )?);
    test.current_dir(&native)
        .args(["--output-on-failure", "--no-tests=error"])
        .env("LLVM_PROFILE_FILE", &profile_file);
    run("native coverage tests", test)?;
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
    let mut merge = Command::new(llvm.root.join("bin").join(executable_name("llvm-profdata")));
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
    let mut report = Command::new(llvm.root.join("bin").join(executable_name("llvm-cov")));
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

fn lint(tool: Option<&str>) -> Result<(), Box<dyn Error>> {
    match tool {
        None => {
            lint(Some("clippy"))?;
            lint(Some("machete"))?;
            lint(Some("cmake"))?;
            build_launcher()?;
            lint(Some("qmllint"))?;
            lint(Some("clang-tidy"))?;
            lint(Some("includes"))?;
            lint(Some("cppcheck"))
        }
        Some("clippy") => cargo(
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
        Some("machete") => cargo("machete", []),
        Some("cmake") => run_cmake_lint(),
        Some("qmllint") => {
            build_launcher()?;
            run_qmllint()
        }
        Some("clang-tidy") => run_clang_tidy(false),
        Some("includes") => run_clang_tidy(true),
        Some("cppcheck") => run_cppcheck(),
        Some(other) => Err(format!(
            "未知 lint 工具 '{other}'；可用：clippy、machete、cmake、qmllint、clang-tidy、includes、cppcheck"
        )
        .into()),
    }
}

fn format_all() -> Result<(), Box<dyn Error>> {
    cargo("fmt", ["--all", "--", "--check"])?;
    check_cpp_format()?;
    run_cmake_format()?;
    run_qml_format_check()
}

fn verify_toolchain() -> Result<(), Box<dyn Error>> {
    let target_root = target_root();
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
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
    let googletest = target_root.join("panta-deps/fetchcontent/googletest-src/CMakeLists.txt");
    let compile_database = native_dir.join("compile_commands.json");

    require_file("托管 CMake", &cmake)?;
    require_file("托管 Ninja", &ninja)?;
    require_file("托管 clang-format", clang_format)?;
    require_file(
        "托管 clang",
        &llvm_root.join("bin").join(executable_name("clang")),
    )?;
    require_file(
        "托管 clang++",
        &llvm_root.join("bin").join(executable_name("clang++")),
    )?;
    if cfg!(windows) {
        require_file(
            "托管 clang-cl",
            &llvm_root.join("bin").join(executable_name("clang-cl")),
        )?;
    }
    let version_binary = if cfg!(windows) {
        llvm_root.join("bin").join(executable_name("clang-cl"))
    } else {
        llvm_root.join("bin").join(executable_name("clang++"))
    };
    require_tool_version(&version_binary, llvm_version)?;
    require_file("Qt qmlformat", &qt_bin.join(executable_name("qmlformat")))?;
    require_file("Qt qmllint", &qt_bin.join(executable_name("qmllint")))?;
    require_file("GoogleTest FetchContent", &googletest)?;
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
        let path = llvm.root.join("bin").join(executable_name(tool));
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

fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
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
    let target_dir = Path::new(env!("PANTA_TEST_TARGET_DIR"));
    let mut command = if matches!(subcommand, "deny" | "machete") {
        let (name, version) = if subcommand == "deny" {
            ("cargo-deny", CARGO_DENY_VERSION)
        } else {
            ("cargo-machete", CARGO_MACHETE_VERSION)
        };
        Command::new(ensure_cargo_tool(name, version)?)
    } else {
        Command::new("cargo")
    };
    if !matches!(subcommand, "deny" | "machete") {
        command.arg(subcommand);
    }
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
    run(&format!("cargo {subcommand}"), command)
}

fn ensure_cargo_tool(name: &str, version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let target_root = Path::new(env!("PANTA_TEST_TARGET_DIR"));
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
        if !staging.join("bin").join(executable_name(name)).is_file() {
            return Err(format!("安装未生成 {name}"));
        }
        Ok(())
    })?;
    Ok(root.join("bin").join(executable_name(name)))
}

fn build_launcher() -> Result<(), Box<dyn Error>> {
    cargo("build", ["--locked", "-p", "panta-launcher"])
}

fn run_qmllint_and_ctest() -> Result<(), Box<dyn Error>> {
    run_qmllint()?;
    run_ctest(None)
}

fn run_qml_format_check() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let target_root = Path::new(env!("PANTA_TEST_TARGET_DIR"));
    let qmlformat = qmlformat_path(target_root);
    if !qmlformat.is_file() {
        provision_qml_format(root, target_root)?;
    }
    let qmlformat = qmlformat_path(target_root);
    if !qmlformat.is_file() {
        return Err(format!("qmlformat 不存在：{}", qmlformat.display()).into());
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

fn run_clang_tidy(includes_only: bool) -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    let llvm = panta_build::resolve_llvm_compilers(target_root())?;
    let tool = llvm.root.join("bin").join(executable_name("clang-tidy"));
    let directory = Path::new(env!("PANTA_TEST_NATIVE_DIR")).join("quality");
    let commands = panta_build::database::read(&directory.join("compile_commands.json"))?;
    if commands.is_empty() {
        return Err("clang-tidy 翻译单元清单为空".into());
    }
    let mut failures = Vec::new();
    for entry in commands {
        let file = entry.source();
        let mut command = Command::new(&tool);
        command.envs(panta_build::windows_sdk_env(env!("PANTA_TEST_HOST"))?);
        command.arg("-p").arg(&directory).arg("-quiet");
        if includes_only {
            command.arg("--checks=-*,misc-include-cleaner");
        }
        command.arg(&file);
        if let Err(error) = run(&format!("clang-tidy {}", file.display()), command) {
            failures.push(error.to_string());
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

fn run_cppcheck() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    let database = Path::new(env!("PANTA_TEST_NATIVE_DIR")).join("quality/cppcheck.json");
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
    run("cppcheck", command)
}

fn run_qmllint() -> Result<(), Box<dyn Error>> {
    let cmake = panta_build::resolve_cmake(target_root())?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let build_type = env!("PANTA_TEST_BUILD_TYPE");
    let mut command = Command::new(cmake);
    command.envs(panta_build::native_test_env(
        target_root(),
        native_dir,
        env!("PANTA_TEST_HOST"),
    )?);
    command.args(["--build"]).arg(native_dir).args([
        "--config",
        build_type,
        "--target",
        "all_qmllint",
    ]);
    run("qmllint", command)
}

fn run_ctest(regex: Option<&str>) -> Result<(), Box<dyn Error>> {
    let cmake = panta_build::resolve_cmake(target_root())?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let build_type = env!("PANTA_TEST_BUILD_TYPE");
    let ctest_name = if cfg!(windows) { "ctest.exe" } else { "ctest" };
    let ctest = cmake.with_file_name(ctest_name);
    if !ctest.is_file() {
        return Err(format!("ctest 不存在：{}", ctest.display()).into());
    }
    let mut command = Command::new(&ctest);
    command.envs(panta_build::native_test_env(
        target_root(),
        native_dir,
        env!("PANTA_TEST_HOST"),
    )?);
    command.args(["--output-on-failure", "--no-tests=error", "-C", build_type]);
    if let Some(regex) = regex {
        command.args(["-R", regex]);
    }
    command.current_dir(native_dir);
    let status = command.status()?;
    if status.success() {
        return Ok(());
    }
    Err(format!("CTest 失败（退出码 {:?}）", status.code()).into())
}

fn check_cpp_format() -> Result<(), Box<dyn Error>> {
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
    for file in files {
        let mut command = Command::new(clang_format);
        command.args(["--dry-run", "-Werror"]).arg(&file);
        run(&format!("clang-format {}", file.display()), command)?;
    }
    Ok(())
}

fn run_cmake_format() -> Result<(), Box<dyn Error>> {
    run_cmake_tool("cmake-format", &["--check"])
}

fn run_cmake_lint() -> Result<(), Box<dyn Error>> {
    run_cmake_tool("cmake-lint", &[])
}

fn run_cmake_tool(tool: &str, arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let mut command = panta_build::python::command(target_root(), root, tool)?;
    command.args(arguments).args(cmake_files(root)?);
    run(tool, command)
}

fn target_root() -> &'static Path {
    Path::new(env!("PANTA_TEST_TARGET_DIR"))
}

fn cpp_sources(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
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
    for entry in std::fs::read_dir(dir)? {
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
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{description} 失败（退出码 {:?}）", status.code()).into())
    }
}
