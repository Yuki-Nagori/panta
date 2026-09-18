use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let result = match arguments.next().as_deref() {
        Some("quality") => quality(),
        Some("test") => test_all(),
        Some("format") => format_all(),
        Some("lint") => lint(arguments.next().as_deref()),
        Some(command) => {
            Err(format!("未知命令 '{command}'；可用：quality、lint、test、format").into())
        }
        None => Err("缺少命令；可用：quality、lint、test、format".into()),
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
    cargo("deny", ["check"])?;
    test_all()
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
    let mut command = Command::new("cargo");
    command.arg(subcommand);
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
