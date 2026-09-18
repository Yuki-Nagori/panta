//! Cargo 原生质量入口（任务 011/032）：复用 build.rs 刚完成的构建树和
//! 配置，顺序执行 qmllint 与 CTest；不回调 Cargo。工具与路径由构建脚本
//! 注入，支持自定义 target-dir、Debug/Release 和多配置生成器。

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(description: &str, command: &mut Command) -> Result<(), Box<dyn Error>> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("{description} 失败（退出码 {:?}）", status.code()).into());
    }
    Ok(())
}

#[test]
fn native_quality_suite_passes() -> Result<(), Box<dyn Error>> {
    let native_dir = Path::new(env!("PANTA_NATIVE_BUILD_DIR"));
    let build_type = env!("PANTA_NATIVE_BUILD_TYPE");
    let cmake = Path::new(env!("PANTA_CMAKE"));
    // ctest 与实际配置使用的 CMake 同源，避免 PATH 上另一版本的工具。
    let ctest = cmake.with_file_name(if cfg!(windows) { "ctest.exe" } else { "ctest" });
    assert!(
        native_dir.is_dir(),
        "native 构建树缺失：{}",
        native_dir.display()
    );
    assert!(ctest.is_file(), "ctest 缺失：{}", ctest.display());

    run(
        "qmllint",
        Command::new(cmake).arg("--build").arg(native_dir).args([
            "--config",
            build_type,
            "--target",
            "all_qmllint",
        ]),
    )?;
    // 多配置生成器必须指定 -C；空套件必须失败，不能把没有运行测试算通过。
    run(
        "ctest",
        Command::new(ctest)
            .args(["--output-on-failure", "--no-tests=error", "-C", build_type])
            .current_dir(native_dir),
    )
}

#[test]
fn cpp_format_passes() -> Result<(), Box<dyn Error>> {
    let clang_format = Path::new(env!("PANTA_CLANG_FORMAT"));
    assert!(clang_format.is_file(), "clang-format 供给缺失");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = cpp_sources(&root.join("native"))?;
    // Cargo 的 CXX 桥接也包含自有 C++，不能遗漏在格式门禁之外。
    files.extend(cpp_sources(&root.join("crates/panta-ffi/src"))?);
    files.extend(cpp_sources(&root.join("crates/panta-ffi/include"))?);
    assert!(!files.is_empty(), "C++ 源文件清单为空");
    files.sort();
    for file in files {
        run(
            &format!("clang-format {}", file.display()),
            Command::new(clang_format)
                .args(["--dry-run", "-Werror"])
                .arg(file),
        )?;
    }
    Ok(())
}

fn cpp_sources(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        // 不吞掉读取错误，否则可能静默漏检源文件。
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            result.extend(cpp_sources(&path)?);
        } else if path.extension().is_some_and(|ext| {
            ext == "cpp" || ext == "cc" || ext == "cxx" || ext == "h" || ext == "hpp"
        }) {
            result.push(path);
        }
    }
    Ok(result)
}
