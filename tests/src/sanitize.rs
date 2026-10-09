//! 平台 sanitizer 矩阵、构建目录、子进程环境与已登记排除。

use std::error::Error;
use std::path::Path;
use std::process::Command;

use crate::build::run_ctest_in;
use crate::command::run;
use crate::paths::{repository_root, target_root};

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

pub(super) fn sanitize() -> Result<(), Box<dyn Error>> {
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
    let build = sanitizer_build_command(root, target, flags);
    run(&format!("sanitizer {name} build"), build)?;
    let native = target
        .join("native")
        .join(format!("debug-sanitizer-{}", flags.replace(',', "-")));
    if !native.is_dir() {
        return Err(format!("sanitizer 构建树不存在：{}", native.display()).into());
    }
    // 组合级排除（边界登记 042）：当前仅插桩 C++，TSan 无法识别 Rust std
    // 同步关系（rust-lang/rust#110485），CI 在跨线程结果交付中产生误报；
    // ProjectViewModel 异步激活、Fill 确认与 STL 预检也经 Rust 同步取结果，
    // 普通 CTest 与 ASan/UBSan 仍执行，Rust 侧由 Miri 验证。
    // QML 测试栈（Qt/glib/系统库）连续三轮仅产出第三方噪声，无自有信号。
    // Windows asan 下未插桩 Qt DLL 走 ucrt/RTL 堆而 ASan 用自有分配器，
    // QML 引擎跨模块对象生命周期触发 bad-free。
    let exclude = match (name, cfg!(windows)) {
        ("tsan", _) => Some(concat!(
            "^(TaskHost|Ffi|Qml)\\.|^ProjectViewModelTest\\.(",
            "ReopenLoadsWelcomeOnlyAndActivatesSavedRecordOnDemand|",
            "FailedLoadRetainsTabAndCloseReleasesActivationState|",
            "FillSettingsConfirmAsynchronouslyAndReopenFromRust|",
            "StructuredImportFailuresPreserveTheCommittedProjectAndViewport|",
            "PreviewsImportsAndPersistsLatestRecord|",
            "PreviewCancellationReplacementAndProjectSwitchIgnoreOldResults|",
            "PreviewNotificationsCanReplaceOrCancelTheCompletedRequest)$",
        )),
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

fn sanitizer_build_command(root: &Path, target: &Path, flags: &str) -> Command {
    let mut build = Command::new("cargo");
    build
        .current_dir(root)
        .args(["build", "--locked", "-p", "panta-launcher", "--target-dir"])
        .arg(target)
        .env("PANTA_NATIVE_SANITIZER", flags);
    build
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_preserve_platform_matrix() {
        let expected: &[(&str, &str)] = if cfg!(windows) {
            &[("asan", "address")]
        } else {
            &[("asan-ubsan", "address,undefined"), ("tsan", "thread")]
        };
        assert_eq!(sanitizer_profiles(), expected);
    }

    #[test]
    fn build_command_preserves_locked_target_and_sanitizer_flags() {
        let root = Path::new("repository with spaces");
        let target = Path::new("target with spaces");
        let command = sanitizer_build_command(root, target, "address,undefined");
        assert_eq!(command.get_program(), "cargo");
        assert_eq!(command.get_current_dir(), Some(root));
        let args = command
            .get_args()
            .map(std::ffi::OsStr::to_os_string)
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            [
                "build",
                "--locked",
                "-p",
                "panta-launcher",
                "--target-dir",
                "target with spaces"
            ]
            .map(std::ffi::OsString::from)
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            vec![(
                std::ffi::OsStr::new("PANTA_NATIVE_SANITIZER"),
                Some(std::ffi::OsStr::new("address,undefined"))
            )]
        );
    }
}
