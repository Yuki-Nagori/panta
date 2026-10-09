//! Rust / C++ / CMake / QML 格式入口及源码清单。

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::command::{cargo, run, run_scanner};
use crate::paths::{repository_root, target_root};

pub(super) fn format_all(check: bool, rest: &[String]) -> Result<(), Box<dyn Error>> {
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
        files.extend(qml_sources(&root.join("tests/qml"))?);
        files.extend(qml_sources(&root.join("tests/fixtures/qml"))?);
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
    files.extend(cpp_sources(&root.join("tests/performance"))?);
    files.extend(cpp_sources(&root.join("tests/support"))?);
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

pub(super) fn run_cmake_tool(tool: &str, arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let mut command = panta_build::python::command(target_root(), root, tool)?;
    command.args(arguments).args(cmake_files(root)?);
    run_scanner(tool, command)
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
