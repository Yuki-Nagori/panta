//! 各质量工具执行顺序、检查 / 修复模式及扫描诊断。

use std::error::Error;
use std::process::Command;

use crate::build::{build_benchmark_moc, build_launcher, run_qmllint};
use crate::command::{cargo, cargo_scanner, run_scanner};
use crate::format::run_cmake_tool;
use crate::paths::{native_build_dir, repository_root, target_root};

pub(super) fn lint(tool: Option<&str>, check: bool) -> Result<(), Box<dyn Error>> {
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
            build_benchmark_moc()?;
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

fn run_cmake_lint() -> Result<(), Box<dyn Error>> {
    run_cmake_tool("cmake-lint", &[])
}

fn run_clang_tidy(includes_only: bool, check: bool) -> Result<(), Box<dyn Error>> {
    build_launcher()?;
    build_benchmark_moc()?;
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
