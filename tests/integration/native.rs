use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;
use std::process::Command;

#[test]
fn native_and_qml_suite_passes() -> Result<(), Box<dyn Error>> {
    let target_root = Path::new(env!("PANTA_TEST_TARGET_DIR"));
    let cmake = panta_build::resolve_cmake(Path::new(env!("PANTA_TEST_TARGET_DIR")))?;
    let native_dir = Path::new(env!("PANTA_TEST_NATIVE_DIR"));
    let build_type = env!("PANTA_TEST_BUILD_TYPE");
    let ctest_name = if cfg!(windows) { "ctest.exe" } else { "ctest" };
    let ctest = cmake.with_file_name(ctest_name);
    if !native_dir.is_dir() {
        return Err(format!("native 构建树不存在：{}", native_dir.display()).into());
    }
    if !ctest.is_file() {
        return Err(format!("ctest 不存在：{}", ctest.display()).into());
    }

    let run_qml_lint = std::env::var("PANTA_TEST_QML_LINT")
        .map(|value| value != "false")
        .unwrap_or(true);
    if run_qml_lint {
        let status = Command::new(&cmake)
            .envs(panta_build::native_test_env(
                target_root,
                env!("PANTA_TEST_HOST"),
            )?)
            .args(["--build"])
            .arg(native_dir)
            .args(["--config", build_type, "--target", "all_qmllint"])
            .status()?;
        if !status.success() {
            return Err(format!("qmllint 失败（退出码 {:?}）", status.code()).into());
        }
    }

    let labels = std::env::var("PANTA_TEST_CTEST_LABELS")
        .ok()
        .filter(|labels| !labels.is_empty());
    // all_qmllint 可能触发 CMake 重新配置，清除未重建 GTest target 的
    // POST_BUILD discovery 文件。先检查未筛选清单并重建占位 target，避免
    // 后续按 label 过滤时把无标签的 *_NOT_BUILT 项静默排除。
    let missing_discovery = unbuilt_test_targets(&ctest, native_dir, build_type)?;
    if !missing_discovery.is_empty() {
        let status = Command::new(&cmake)
            .envs(panta_build::native_test_env(
                target_root,
                env!("PANTA_TEST_HOST"),
            )?)
            .args(["--build"])
            .arg(native_dir)
            .args(["--config", build_type, "--target"])
            .args(&missing_discovery)
            .status()?;
        if !status.success() {
            return Err(format!(
                "恢复 GTest discovery 清单失败（退出码 {:?}）",
                status.code()
            )
            .into());
        }
    }

    let targets = selected_test_targets(&ctest, native_dir, build_type, labels.as_deref())?;
    let mut build_command = Command::new(cmake);
    build_command
        .envs(panta_build::native_test_env(
            target_root,
            env!("PANTA_TEST_HOST"),
        )?)
        .args(["--build"])
        .arg(native_dir)
        .args(["--config", build_type]);
    if labels.is_some() {
        if !targets.is_empty() {
            build_command.arg("--target").args(&targets);
        }
    } else {
        // 未筛选时保持开发者完整聚合测试行为。
        build_command.args(["--target", "all"]);
    }
    let status = build_command.status()?;
    if !status.success() {
        return Err(format!("native test build 失败（退出码 {:?}）", status.code()).into());
    }

    let mut ctest_command = Command::new(ctest);
    ctest_command
        .envs(panta_build::native_test_env(
            target_root,
            env!("PANTA_TEST_HOST"),
        )?)
        .args(["--output-on-failure", "--no-tests=error", "-C", build_type]);
    if let Some(labels) = labels {
        // PR CI 只运行影响闭包内的标签；本地未设置时仍执行完整 CTest。
        ctest_command.arg("-L").arg(labels);
    }
    let status = ctest_command.current_dir(native_dir).status()?;
    if !status.success() {
        return Err(format!("CTest 失败（退出码 {:?}）", status.code()).into());
    }
    Ok(())
}

/// 从未筛选的 CTest 清单找出尚未生成 discovery 文件的 GoogleTest target。
fn unbuilt_test_targets(
    ctest: &Path,
    native_dir: &Path,
    build_type: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut command = Command::new(ctest);
    command
        .args(["--show-only=json-v1", "-C", build_type])
        .current_dir(native_dir);
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "读取未筛选 CTest 清单失败（退出码 {:?}）：{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let tests = report["tests"]
        .as_array()
        .ok_or("CTest JSON 缺少 tests 数组")?;
    let mut targets = BTreeSet::new();
    for test in tests {
        let Some(name) = test["name"].as_str() else {
            continue;
        };
        if let Some(target) = unbuilt_target_from_name(name)? {
            targets.insert(target);
        }
    }
    Ok(targets.into_iter().collect())
}

fn unbuilt_target_from_name(name: &str) -> Result<Option<String>, String> {
    let Some(target) = name.strip_suffix("_NOT_BUILT") else {
        return Ok(None);
    };
    if target.starts_with("panta_")
        && target.ends_with("_test")
        && target
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        Ok(Some(target.to_owned()))
    } else {
        Err(format!("无法识别 CTest 未构建占位项：{name}"))
    }
}

/// 从 CTest 的已筛选清单提取实际可执行目标，避免为局部验证构建全部 native 测试。
fn selected_test_targets(
    ctest: &Path,
    native_dir: &Path,
    build_type: &str,
    labels: Option<&str>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let Some(labels) = labels else {
        return Ok(Vec::new());
    };
    let mut command = Command::new(ctest);
    command
        .args(["--show-only=json-v1", "-C", build_type, "-L", labels])
        .current_dir(native_dir);
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "读取受影响 CTest 清单失败（退出码 {:?}）：{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let tests = report["tests"]
        .as_array()
        .ok_or("CTest JSON 缺少 tests 数组")?;
    let mut targets = BTreeSet::new();
    for test in tests {
        let Some(arguments) = test["command"].as_array() else {
            continue;
        };
        for argument in arguments {
            let Some(executable) = argument.as_str() else {
                continue;
            };
            let executable = Path::new(executable);
            if !executable.starts_with(native_dir) {
                continue;
            }
            let Some(name) = executable.file_stem().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("panta_")
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                targets.insert(name.to_owned());
            }
        }
    }
    Ok(targets.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::unbuilt_target_from_name;

    #[test]
    fn recognizes_google_test_discovery_placeholders() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            unbuilt_target_from_name("panta_mesh_netgen_test_NOT_BUILT")?,
            Some("panta_mesh_netgen_test".into())
        );
        assert_eq!(unbuilt_target_from_name("Mesh.ValidatesInput")?, None);
        Ok(())
    }

    #[test]
    fn rejects_unknown_discovery_placeholder_names() {
        assert!(unbuilt_target_from_name("unknown_NOT_BUILT").is_err());
    }
}
