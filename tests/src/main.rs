use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
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
        Some("coverage") => coverage(),
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
        .env("CARGO_TARGET_DIR", target_dir)
        .args([
            "--locked",
            "--workspace",
            "--exclude",
            "panta-launcher",
            "--summary-only",
            "--fail-under-functions",
            "89",
            "--fail-under-lines",
            "92",
        ])
        .current_dir(ROOT);
    run("cargo llvm-cov", command)
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
            lint(Some("iwyu"))?;
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
        Some("clang-tidy") => run_clang_tidy(),
        Some("iwyu") => run_iwyu(),
        Some("cppcheck") => run_cppcheck(),
        Some(other) => Err(format!(
            "未知 lint 工具 '{other}'；可用：clippy、machete、cmake、qmllint、clang-tidy、iwyu、cppcheck"
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
    let target_root = Path::new(env!("PANTA_TEST_TARGET_DIR"));
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let cmake = Path::new(env!("PANTA_TEST_CMAKE"));
    let clang_format = Path::new(env!("PANTA_TEST_CLANG_FORMAT"));
    let llvm_root = Path::new(env!("PANTA_TEST_LLVM_ROOT"));
    let llvm_version = env!("PANTA_TEST_LLVM_VERSION");
    let managed_root = target_root.join("panta-tools");
    let managed_cmake = managed_root.join("cmake");
    let managed_llvm = managed_root.join("llvm");
    let ninja = managed_root.join("ninja").join(executable_name("ninja"));
    let qt_bin = target_root.join("panta-deps/qt/staging/bin");
    let googletest = target_root.join("panta-deps/fetchcontent/googletest-src/CMakeLists.txt");
    let compile_database = native_dir.join("compile_commands.json");

    require_file("托管 CMake", cmake)?;
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
    command.current_dir(ROOT);
    run(&format!("cargo {subcommand}"), command)
}

fn ensure_cargo_tool(name: &str, version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let target_root = Path::new(env!("PANTA_TEST_TARGET_DIR"));
    let root = target_root.join("panta-tools/cargo");
    let binary = root.join("bin").join(executable_name(name));
    let marker = root.join(format!(".{name}-{version}.installed"));
    if binary.is_file()
        && fs::read_to_string(&marker)
            .map(|recorded| recorded.trim() == version)
            .unwrap_or(false)
    {
        return Ok(binary);
    }
    let mut command = Command::new("cargo");
    command
        .args(["install", "--root"])
        .arg(&root)
        .args([name, "--locked", "--version", version, "--force"])
        .env("CARGO_TARGET_DIR", root.join("build"))
        .current_dir(ROOT);
    run(&format!("安装 {name} {version}"), command)?;
    if binary.is_file() {
        fs::write(&marker, format!("{version}\n"))?;
        Ok(binary)
    } else {
        Err(format!("cargo install 成功但未生成项目工具：{}", binary.display()).into())
    }
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
    let cmake = Path::new(env!("PANTA_TEST_CMAKE"));
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
    let cmake = Path::new(env!("PANTA_TEST_CMAKE"));
    let build_dir = target_root.join("native/format-tools");
    let mut command = Command::new(cmake);
    command.args([
        "-S",
        root.join("native")
            .to_str()
            .ok_or("native 路径不是 UTF-8")?,
    ]);
    command.args([
        "-B",
        build_dir.to_str().ok_or("格式工具构建路径不是 UTF-8")?,
    ]);
    command.args(["-DBUILD_TESTING=OFF", "-DPANTA_ENABLE_BRIDGE_MODULE=OFF"]);
    command.arg(format!(
        "-DQT_PROVISION_DIR={}",
        target_root.join("panta-deps/qt").display()
    ));
    command.arg(format!(
        "-DFETCHCONTENT_BASE_DIR={}",
        target_root.join("panta-deps/fetchcontent").display()
    ));
    run("准备 qmlformat", command)
}

fn run_clang_tidy() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    let tool = required_tool("CLANG_TIDY", &["clang-tidy", "clang-tidy.exe"])?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    require_compile_database(native_dir)?;
    let root = repository_root()?;
    let mut files = cpp_sources(&root.join("native"))?;
    files.extend(cpp_sources(&root.join("tests/cpp"))?);
    files.extend(cpp_sources(&root.join("tests/qml"))?);
    files.sort();
    for file in files {
        let mut command = Command::new(&tool);
        command
            .args([
                "-p",
                native_dir.to_str().ok_or("native 路径不是 UTF-8")?,
                "-quiet",
            ])
            .arg(&file);
        run(&format!("clang-tidy {}", file.display()), command)?;
    }
    Ok(())
}

fn run_iwyu() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    let tool = required_tool(
        "IWYU_TOOL",
        &["iwyu_tool.py", "iwyu_tool", "iwyu_tool.py.exe"],
    )?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    require_compile_database(native_dir)?;
    let root = repository_root()?;
    let mut command =
        if cfg!(windows) && tool.extension().is_some_and(|extension| extension == "py") {
            let mut command = Command::new("python");
            command.arg(&tool);
            command
        } else {
            Command::new(&tool)
        };
    command.args([
        "-p",
        native_dir.to_str().ok_or("native 路径不是 UTF-8")?,
        "-j",
        "1",
    ]);
    command.args([
        root.join("native"),
        root.join("tests/cpp"),
        root.join("tests/qml"),
        root.join("crates/panta-ffi"),
    ]);
    run("include-what-you-use", command)
}

fn run_cppcheck() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    let tool = required_tool("CPPCHECK", &["cppcheck", "cppcheck.exe"])?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let database = require_compile_database(native_dir)?;
    let mut command = Command::new(&tool);
    command
        .arg(format!("--project={}", database.display()))
        .args([
            "--enable=warning,style,performance,portability,unusedFunction",
            "--error-exitcode=1",
            "--inline-suppr",
            "--suppress=missingIncludeSystem",
        ]);
    run("cppcheck", command)
}

fn require_compile_database(native_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let database = native_dir.join("compile_commands.json");
    if database.is_file() {
        Ok(database)
    } else {
        Err(format!("CMake compile database 不存在：{}", database.display()).into())
    }
}

fn required_tool(variable: &str, names: &[&str]) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(value) = std::env::var_os(variable) {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("{variable} 指向的工具不存在：{}", path.display()).into());
    }
    if variable == "CLANG_TIDY" {
        let llvm_candidate = Path::new(env!("PANTA_TEST_LLVM_ROOT"))
            .join("bin")
            .join(executable_name("clang-tidy"));
        if llvm_candidate.is_file() {
            return Ok(llvm_candidate);
        }
    }
    let path_var = std::env::var_os("PATH").ok_or("PATH 未设置")?;
    for directory in std::env::split_paths(&path_var) {
        for name in names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(format!("缺少 {variable} 工具（PATH 中未找到 {}）", names.join(", ")).into())
}

fn run_qmllint() -> Result<(), Box<dyn Error>> {
    let cmake = Path::new(env!("PANTA_TEST_CMAKE"));
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let build_type = env!("PANTA_TEST_BUILD_TYPE");
    let mut command = Command::new(cmake);
    command.args(["--build"]).arg(native_dir).args([
        "--config",
        build_type,
        "--target",
        "all_qmllint",
    ]);
    run("qmllint", command)
}

fn run_ctest(regex: Option<&str>) -> Result<(), Box<dyn Error>> {
    let cmake = Path::new(env!("PANTA_TEST_CMAKE"));
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let build_type = env!("PANTA_TEST_BUILD_TYPE");
    let ctest_name = if cfg!(windows) { "ctest.exe" } else { "ctest" };
    let ctest = cmake.with_file_name(ctest_name);
    if !ctest.is_file() {
        return Err(format!("ctest 不存在：{}", ctest.display()).into());
    }
    let mut command = Command::new(ctest);
    command.args(["--output-on-failure", "--no-tests=error", "-C", build_type]);
    if let Some(regex) = regex {
        command.args(["-R", regex]);
    }
    command.current_dir(native_dir);
    run("ctest", command)
}

fn check_cpp_format() -> Result<(), Box<dyn Error>> {
    let clang_format = Path::new(env!("PANTA_TEST_CLANG_FORMAT"));
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
    let root = repository_root()?;
    let uv = required_tool("UV", &["uv", "uv.exe"])?;
    for file in cmake_files(root)? {
        let mut command = Command::new(&uv);
        command
            .env("UV_CACHE_DIR", root.join("target/panta-tools/uv/cache"))
            .env(
                "UV_PROJECT_ENVIRONMENT",
                root.join("target/panta-tools/uv/venv"),
            )
            .args(["run", "--locked", "--project"])
            .arg(root)
            .args(["cmake-format", "--check"])
            .arg(&file);
        run(&format!("cmake-format {}", file.display()), command)?;
    }
    Ok(())
}

fn run_cmake_lint() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let uv = required_tool("UV", &["uv", "uv.exe"])?;
    for file in cmake_files(root)? {
        let mut command = Command::new(&uv);
        command
            .env("UV_CACHE_DIR", root.join("target/panta-tools/uv/cache"))
            .env(
                "UV_PROJECT_ENVIRONMENT",
                root.join("target/panta-tools/uv/venv"),
            )
            .args(["run", "--locked", "--project"])
            .arg(root)
            .args(["cmake-lint"])
            .arg(&file);
        run(&format!("cmake-lint {}", file.display()), command)?;
    }
    Ok(())
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
    Path::new(ROOT)
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
