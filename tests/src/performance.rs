//! 手动性能套件入口；辅助代码通过后，按 CPU、GPU 顺序运行，不进入常规 CTest。
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suite {
    All,
    Support,
    Cpu,
    Gpu,
}

#[derive(Debug)]
struct Options {
    suite: Suite,
    presentation: bool,
    project: Option<PathBuf>,
    samples: Option<u32>,
}

impl Options {
    fn parse(arguments: Vec<String>) -> Result<Self, String> {
        let mut options = Self {
            suite: Suite::All,
            presentation: false,
            project: None,
            samples: None,
        };
        let mut arguments = arguments.into_iter();
        let mut selected = false;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "all" | "support" | "cpu" | "gpu" if !selected => {
                    selected = true;
                    options.suite = match argument.as_str() {
                        "support" => Suite::Support,
                        "cpu" => Suite::Cpu,
                        "gpu" => Suite::Gpu,
                        _ => Suite::All,
                    };
                }
                "--presentation" if !options.presentation => options.presentation = true,
                "--project" if options.project.is_none() => {
                    options.project =
                        Some(PathBuf::from(arguments.next().ok_or("--project 缺少路径")?));
                }
                "--samples" if options.samples.is_none() => {
                    let value = arguments.next().ok_or("--samples 缺少次数")?;
                    let samples = value.parse::<u32>().map_err(|_| "--samples 必须为正整数")?;
                    if samples == 0 || samples > i32::MAX as u32 {
                        return Err("--samples 超出有效范围".into());
                    }
                    options.samples = Some(samples);
                }
                _ => return Err(format!("未知或重复参数：{argument}")),
            }
        }
        if options.presentation && matches!(options.suite, Suite::Support | Suite::Cpu) {
            return Err("--presentation 仅适用于 all / gpu".into());
        }
        if options.suite == Suite::Support
            && (options.project.is_some() || options.samples.is_some())
        {
            return Err("support 不接受 --project / --samples".into());
        }
        Ok(options)
    }
}

fn build(targets: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut command = super::cmake_build_command()?;
    command.arg("--target").args(targets);
    super::run("构建性能目标", command)
}

fn native_command(target: &str) -> Result<Command, Box<dyn Error>> {
    let executable = super::native_build_dir()
        .join("performance")
        .join(panta_build::exe_name(target));
    if !executable.is_file() {
        return Err(format!("性能目标不存在：{}", executable.display()).into());
    }
    let mut command = Command::new(executable);
    command
        .current_dir(super::repository_root()?)
        .envs(panta_build::native_test_env(
            super::target_root(),
            env!("PANTA_TEST_HOST"),
        )?);
    command.env("QT_QUICK_CONTROLS_STYLE", "Basic");
    Ok(command)
}

fn run_project(
    target: &str,
    project: &Path,
    samples: Option<u32>,
    presentation: bool,
) -> Result<(), Box<dyn Error>> {
    let mut command = native_command(target)?;
    command.arg(project);
    if let Some(samples) = samples {
        command.arg(samples.to_string());
    }
    command.env(
        "PANTA_BENCHMARK_PRESENTATION",
        if presentation { "1" } else { "0" },
    );
    super::run(target, command)
}

pub(super) fn run(arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    if arguments == ["--help"] {
        println!(
            "cargo performance [all|support|cpu|gpu] [--presentation] [--project FILE] [--samples N]\n执行顺序：辅助代码 → CPU → GPU；默认生成合成 STL 工程，真实窗口需 --presentation。"
        );
        return Ok(());
    }
    let options = Options::parse(arguments)?;
    // 性能二进制使用当前 runner 的构建配置，避免误用另一配置的旧产物。
    super::cargo("build", ["--locked", "-p", "panta-launcher"])?;
    build(&["panta_benchmark_support_test"])?;
    super::cargo(
        "test",
        [
            "--locked",
            "-p",
            "panta-tests",
            "--bin",
            "panta-tests",
            "performance::tests",
            "--",
            "--ignored",
        ],
    )?;
    super::run(
        "性能辅助代码",
        native_command("panta_benchmark_support_test")?,
    )?;
    if options.suite == Suite::Support {
        return Ok(());
    }

    let cpu = matches!(options.suite, Suite::All | Suite::Cpu);
    let gpu = matches!(options.suite, Suite::All | Suite::Gpu);
    let mut targets = vec!["panta_performance_fixture"];
    if cpu {
        targets.extend([
            "panta_mesh_ir_benchmark",
            "panta_qml_cpu_benchmark",
            "panta_viewport_navigation_cpu_benchmark",
            "panta_bridge_project_stl_activation_cpu_benchmark",
        ]);
    }
    if gpu {
        targets.extend([
            "panta_qml_gpu_benchmark",
            "panta_viewport_gpu_benchmark",
            "panta_project_stl_viewport_gpu_benchmark",
        ]);
    }
    build(&targets)?;
    let supplied_stl = std::env::var_os("PANTA_BENCH_STL").map(PathBuf::from);
    let fixture = if options.project.is_none() || (cpu && supplied_stl.is_none()) {
        let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let directory = super::target_root()
            .join("performance")
            .join(format!("fixture-{}-{id}", std::process::id()));
        let mut command = native_command("panta_performance_fixture")?;
        command.arg(&directory);
        super::run("生成合成 STL 工程", command)?;
        Some(directory)
    } else {
        None
    };
    let project = match options.project {
        Some(project) => project.canonicalize()?,
        None => fixture
            .as_ref()
            .ok_or("missing project fixture")?
            .join("Benchmark/Benchmark.panta"),
    };
    if cpu {
        super::cargo(
            "run",
            [
                "--locked",
                "--release",
                "-p",
                "panta-core",
                "--example",
                "background_execution",
            ],
        )?;
        let stl = supplied_stl
            .or_else(|| {
                fixture
                    .as_ref()
                    .map(|path| path.join("triangles_20000.stl"))
            })
            .ok_or("missing STL benchmark input")?;
        let (_, mut command) = super::cargo_command(
            "test",
            [
                "--locked",
                "--release",
                "-p",
                "panta-mesh",
                "--test",
                "mesh_performance",
                "--",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ],
        )?;
        command.env("PANTA_BENCH_STL", stl);
        super::run("Rust mesh CPU benchmarks", command)?;
        for target in [
            "panta_mesh_ir_benchmark",
            "panta_qml_cpu_benchmark",
            "panta_viewport_navigation_cpu_benchmark",
        ] {
            let mut command = native_command(target)?;
            command.env("QT_QPA_PLATFORM", "offscreen");
            super::run(target, command)?;
        }
        run_project(
            "panta_bridge_project_stl_activation_cpu_benchmark",
            &project,
            options.samples,
            false,
        )?;
    }
    if gpu {
        for target in ["panta_qml_gpu_benchmark", "panta_viewport_gpu_benchmark"] {
            let mut command = native_command(target)?;
            command.env(
                "PANTA_BENCHMARK_PRESENTATION",
                if options.presentation { "1" } else { "0" },
            );
            super::run(target, command)?;
        }
        run_project(
            "panta_project_stl_viewport_gpu_benchmark",
            &project,
            options.samples,
            options.presentation,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Options, Suite};
    #[test]
    #[ignore = "Performance CLI validation; cargo performance runs this before benchmarks"]
    fn parses_suite_and_options() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(
            ["gpu", "--presentation", "--samples", "3"]
                .map(str::to_owned)
                .to_vec(),
        )?;
        assert_eq!(options.suite, Suite::Gpu);
        assert!(options.presentation);
        assert_eq!(options.samples, Some(3));
        assert_eq!(Options::parse(vec![])?.suite, Suite::All);
        Ok(())
    }
    #[test]
    #[ignore = "Performance CLI validation; cargo performance runs this before benchmarks"]
    fn preserves_cargo_argument_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let (_, command) = crate::cargo_command("test", ["--locked", "--", "--ignored"])?;
        let arguments: Vec<_> = command.get_args().collect();
        let separator = arguments
            .iter()
            .position(|argument| *argument == "--")
            .ok_or("missing separator")?;
        let target = arguments
            .iter()
            .position(|argument| *argument == "--target-dir")
            .ok_or("missing target directory")?;
        assert!(target < separator);
        Ok(())
    }
    #[test]
    #[ignore = "Performance CLI validation; cargo performance runs this before benchmarks"]
    fn rejects_invalid_options() {
        for arguments in [
            vec!["cpu", "--presentation"],
            vec!["gpu", "cpu"],
            vec!["--samples", "0"],
            vec!["--project"],
            vec!["--samples", "bad"],
            vec!["--unknown"],
        ] {
            assert!(Options::parse(arguments.into_iter().map(str::to_owned).collect()).is_err());
        }
    }
}
