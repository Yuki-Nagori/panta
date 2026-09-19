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

    let status = Command::new(cmake)
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

    let status = Command::new(ctest)
        .envs(panta_build::native_test_env(
            target_root,
            env!("PANTA_TEST_HOST"),
        )?)
        .args(["--output-on-failure", "--no-tests=error", "-C", build_type])
        .current_dir(native_dir)
        .status()?;
    if !status.success() {
        return Err(format!("CTest 失败（退出码 {:?}）", status.code()).into());
    }
    Ok(())
}
