/// CLI 端到端测试（任务 032）：以 `CARGO_BIN_EXE` 提供的真二进制驱动
/// main() 与退出码契约（成功 0、用法/校验失败 2），子进程继承覆盖率
/// 环境（LLVM_PROFILE_FILE），main() 的行覆盖随本测试并入统计。
use std::process::Command;

fn dslc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_panta-dslc"))
}

#[test]
fn no_arguments_prints_usage_and_exits_two() {
    let output = dslc()
        .output()
        .unwrap_or_else(|error| panic!("启动失败: {error}"));
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage: panta-dslc"), "{stderr}");
}

#[test]
fn check_succeeds_and_fails_with_exit_code_two() {
    let dir = std::env::temp_dir().join(format!("panta-dslc-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_else(|error| panic!("mkdir 失败: {error}"));
    let ok = dir.join("ok.pa");
    std::fs::write(&ok, "version: 1\nkind: language\nlanguage: en\nsourcelanguage: en\n\n[App]\ntitle:\n  src: Hi\n  tr: Hi\n")
        .unwrap_or_else(|error| panic!("写入失败: {error}"));

    let output = dslc()
        .args([
            "check",
            ok.to_str().unwrap_or_else(|| panic!("非 UTF-8 路径")),
        ])
        .output()
        .unwrap_or_else(|error| panic!("启动失败: {error}"));
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let bad = dir.join("bad.pa");
    std::fs::write(&bad, "version: 1\nkind: language\nlanguage: en\n")
        .unwrap_or_else(|error| panic!("写入失败: {error}"));
    let output = dslc()
        .args([
            "check",
            bad.to_str().unwrap_or_else(|| panic!("非 UTF-8 路径")),
        ])
        .output()
        .unwrap_or_else(|error| panic!("启动失败: {error}"));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("pa."));

    let _ = std::fs::remove_dir_all(&dir);
}
