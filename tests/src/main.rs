//! Cargo 质量命令分发；具体构建、检查和报告由各职责模块承接。

mod build;
mod command;
mod coverage;
mod format;
mod lint;
mod miri;
mod paths;
mod performance;
mod sanitize;
mod toolchain;

use std::error::Error;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let result = match arguments.next().as_deref() {
        Some("quality") => quality(),
        Some("performance") => performance::run(arguments.collect()),
        Some("test") => build::test_all(),
        Some("format") => {
            let mut rest: Vec<String> = arguments.collect();
            let check = take_check_flag(&mut rest);
            format::format_all(check, &rest)
        }
        Some("audit") => audit(),
        Some("coverage") => match arguments.next().as_deref() {
            None | Some("rust") => coverage::run(),
            Some("native") => coverage::native_coverage(),
            Some(other) => Err(format!("未知 coverage 类型：{other}").into()),
        },
        Some("sanitize") => sanitize::sanitize(),
        // 对外叫 ub-check：别名 `miri` 会遮蔽 cargo-miri 外部子命令。
        Some("ub-check") => miri::miri(),
        Some("toolchain") => toolchain::verify_toolchain(),
        Some("lint") => {
            let mut rest: Vec<String> = arguments.collect();
            let check = take_check_flag(&mut rest);
            lint::lint(rest.first().map(String::as_str), check)
        }
        Some(command) => Err(format!(
            "未知命令 '{command}'；可用：quality、test、audit、lint、format、coverage、sanitize、\
             ub-check、toolchain、performance"
        )
        .into()),
        None => Err(
            "缺少命令；可用：quality、test、audit、lint、format、coverage、sanitize、ub-check、\
                 toolchain、performance"
                .into(),
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("panta-tests: {error}");
            ExitCode::FAILURE
        }
    }
}

/// 从剩余参数摘除 `--check`（位置不限）。缺省行为是就地修复（写文件），
/// `--check` 只验证不改动；CI 与 pre-commit 一律带 `--check`。
fn take_check_flag(arguments: &mut Vec<String>) -> bool {
    let check = arguments.iter().any(|argument| argument == "--check");
    arguments.retain(|argument| argument != "--check");
    check
}

fn quality() -> Result<(), Box<dyn Error>> {
    format::format_all(true, &[])?;
    lint::lint(None, true)?;
    audit()?;
    build::test_all()
}

fn audit() -> Result<(), Box<dyn Error>> {
    command::cargo("deny", ["check"])
}

#[cfg(test)]
mod tests {
    use super::take_check_flag;

    #[test]
    fn check_flag_is_removed_without_reordering_arguments() {
        let mut arguments = vec![
            "--check".to_owned(),
            "clippy".to_owned(),
            "--check".to_owned(),
        ];
        assert!(take_check_flag(&mut arguments));
        assert_eq!(arguments, ["clippy"]);
        assert!(!take_check_flag(&mut arguments));
        assert_eq!(arguments, ["clippy"]);
    }
}
