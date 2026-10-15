//! 各质量工具执行顺序、检查 / 修复模式及扫描诊断。

use std::error::Error;
use std::process::Command;

use crate::build::{build_benchmark_moc, build_launcher, run_qmllint};
use crate::command::{cargo, cargo_scanner, run, run_scanner};
use crate::format::run_cmake_tool;
use crate::paths::{native_build_dir, repository_root, target_root};

pub(super) fn lint(
    tool: Option<&str>,
    check: bool,
    changed: bool,
    _arguments: &[String],
) -> Result<(), Box<dyn Error>> {
    match tool {
        None if changed => lint_changed(check),
        None => {
            eprintln!("cargo lint [1/9] Rust Clippy");
            lint(Some("clippy"), check, false, &[])?;
            eprintln!("cargo lint [2/9] unused Cargo dependencies");
            lint(Some("machete"), check, false, &[])?;
            eprintln!("cargo lint [3/9] CMake format and lint");
            lint(Some("cmake"), check, false, &[])?;
            eprintln!("cargo lint [4/9] Rust/CMake layering");
            crate::layering::check()?;
            eprintln!("cargo lint [5/9] native build and QML metadata");
            build_launcher()?;
            eprintln!("cargo lint [6/9] qmllint");
            run_qmllint()?;
            build_benchmark_moc()?;
            eprintln!("cargo lint [7/9] clang-tidy");
            scan_clang_tidy(false, check, false)?;
            eprintln!("cargo lint [8/9] include-cleaner");
            scan_clang_tidy(true, check, false)?;
            eprintln!("cargo lint [9/9] cppcheck");
            scan_cppcheck()
        }
        Some("clippy") if changed => run_changed_clippy(check),
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
            lint(Some("clippy"), true, false, &[])
        }
        Some("machete") => cargo_scanner("machete", []),
        Some("cmake") if changed => run_changed_cmake_lint(),
        Some("cmake") => run_cmake_lint(),
        Some("layering") => crate::layering::run_lint(),
        Some("qmllint") => {
            build_launcher()?;
            run_qmllint()
        }
        Some("clang-tidy") => run_clang_tidy(false, check, changed),
        Some("includes") => run_clang_tidy(true, check, changed),
        Some("cppcheck") => run_cppcheck(),
        Some(other) => Err(format!(
            "未知 lint 工具 '{other}'；可用：clippy、machete、cmake、layering、qmllint、clang-tidy、\
             includes、cppcheck；--check 只检查不修复"
        )
        .into()),
    }
}

fn lint_changed(check: bool) -> Result<(), Box<dyn Error>> {
    if crate::test_scope::force_full_scope() {
        return lint(None, check, false, &[]);
    }
    let paths = crate::test_scope::changed_paths()?;
    if paths.is_empty() {
        println!("变更 lint：没有变更路径");
        return Ok(());
    }
    let rust_packages = crate::test_scope::changed_lint_packages()?;
    if !rust_packages.is_empty() {
        run_changed_clippy(check)?;
        lint(Some("machete"), check, false, &[])?;
    }

    if paths.iter().any(|path| is_layering_path(path)) {
        run_changed_cmake_lint()?;
        crate::layering::check()?;
    }

    let native_changed = paths.iter().any(|path| path.starts_with("native/"));
    let cpp_changed = paths.iter().any(|path| is_cpp_path(path));
    let native_build_configuration_changed = paths.iter().any(|path| {
        path.starts_with("native/")
            && (path.ends_with("CMakeLists.txt") || path.ends_with(".cmake"))
    });
    let qml_changed = paths
        .iter()
        .any(|path| path.starts_with("qml/") || path.starts_with("tests/qml/") || native_changed);

    if cpp_changed || native_changed || qml_changed {
        // clang-tidy、qmllint 与 cppcheck 共用一次 launcher configure/build；
        // benchmark moc 只在 C++ 扫描需要完整编译数据库时生成。
        build_launcher()?;
    }
    if cpp_changed || native_changed {
        build_benchmark_moc()?;
        scan_clang_tidy(false, check, true)?;
        scan_clang_tidy(true, check, true)?;
        if cpp_changed || native_build_configuration_changed {
            // unusedFunction 是跨翻译单元诊断。改动编译单元集合也要完整扫描。
            scan_cppcheck()?;
        }
    }

    if qml_changed {
        run_qmllint()?;
    }
    Ok(())
}

fn is_cmake_path(path: &str) -> bool {
    std::path::Path::new(path)
        .file_name()
        .is_some_and(|name| name == "CMakeLists.txt")
        || path.ends_with(".cmake")
}

fn is_layering_path(path: &str) -> bool {
    is_cmake_path(path)
        || path == "Cargo.toml"
        || path.ends_with("/Cargo.toml")
        || path == "Cargo.lock"
}

fn is_cpp_path(path: &str) -> bool {
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("cpp" | "cc" | "cxx" | "h" | "hpp")
    )
}

fn run_changed_clippy(check: bool) -> Result<(), Box<dyn Error>> {
    let packages = crate::test_scope::changed_lint_packages()?;
    if packages.is_empty() {
        println!("Clippy 变更检查：没有受影响的 Rust package");
        return Ok(());
    }

    let mut command = Command::new("cargo");
    command.args(["clippy", "--locked", "--all-targets"]);
    if !check {
        command.args(["--fix", "--allow-dirty"]);
    }
    for package in &packages {
        command.arg("--package").arg(package);
    }
    command.arg("--target-dir").arg(target_root());
    if check {
        command.args(["--", "-D", "warnings"]);
    }
    let label = packages.iter().cloned().collect::<Vec<_>>().join(", ");
    run(&format!("Clippy 变更包（{label}）"), command)?;
    if !check {
        run_changed_clippy(true)?;
    }
    Ok(())
}

fn run_cmake_lint() -> Result<(), Box<dyn Error>> {
    run_cmake_tool("cmake-lint", &[])
}

fn run_changed_cmake_lint() -> Result<(), Box<dyn Error>> {
    if crate::test_scope::force_full_scope() {
        return run_cmake_lint();
    }
    let root = repository_root()?;
    let files = crate::test_scope::changed_paths()?
        .iter()
        .map(|path| root.join(path))
        .filter(|path| {
            path.is_file()
                && (path
                    .file_name()
                    .is_some_and(|name| name == "CMakeLists.txt")
                    || path
                        .extension()
                        .is_some_and(|extension| extension == "cmake"))
                && ["native", "qml", "tools"]
                    .iter()
                    .any(|directory| path.starts_with(root.join(directory)))
        })
        .collect::<Vec<_>>();
    if files.is_empty() {
        println!("cmake-lint 变更检查：没有修改的 CMake 文件");
        return Ok(());
    }
    let mut command = panta_build::python::command(target_root(), root, "cmake-lint")?;
    command.args(&files);
    run_scanner("cmake-lint 变更文件", command)
}

fn run_clang_tidy(includes_only: bool, check: bool, changed: bool) -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    build_benchmark_moc()?;
    scan_clang_tidy(includes_only, check, changed)
}

fn scan_clang_tidy(includes_only: bool, check: bool, changed: bool) -> Result<(), Box<dyn Error>> {
    let llvm = panta_build::resolve_llvm_compilers(target_root())?;
    let tool = llvm
        .root
        .join("bin")
        .join(panta_build::exe_name("clang-tidy"));
    let directory = native_build_dir().join("quality");
    let mut commands = panta_build::database::read(&directory.join("compile_commands.json"))?;
    if commands.is_empty() {
        return Err("clang-tidy 翻译单元清单为空".into());
    }
    if changed && !crate::test_scope::force_full_scope() {
        let root = repository_root()?;
        let paths = crate::test_scope::changed_paths()?;
        let changed_sources = paths
            .iter()
            .map(|path| root.join(path))
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        let changed_headers = changed_sources.iter().any(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("h" | "hpp")
            )
        });
        let build_configuration_changed = paths.iter().any(|path| {
            path.starts_with("native/")
                && (path.ends_with("CMakeLists.txt") || path.ends_with(".cmake"))
        });
        if !changed_headers && !build_configuration_changed {
            let selected = changed_sources
                .iter()
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|extension| extension.to_str()),
                        Some("cpp" | "cc" | "cxx")
                    )
                })
                .map(|path| path.canonicalize().unwrap_or_else(|_| path.clone()))
                .collect::<std::collections::BTreeSet<_>>();
            commands.retain(|entry| {
                let source = entry
                    .source()
                    .canonicalize()
                    .unwrap_or_else(|_| entry.source());
                selected.contains(&source)
            });
            if commands.is_empty() {
                println!("clang-tidy 变更检查：没有受影响的 C++ 翻译单元");
                return Ok(());
            }
        } else {
            println!("clang-tidy 变更检查扩大到完整 native 翻译单元（头文件或 CMake 配置变更）");
        }
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

fn run_cppcheck() -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    build_benchmark_moc()?;
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
