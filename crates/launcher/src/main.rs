//! panta 统一运行入口。
//!
//! 桌面可执行文件尚未接入：任务 004 负责定位并启动 CMake 产物，
//! 任务 005 提供 Qt 桌面程序。在此之前本入口只输出未接入诊断，
//! 不伪称启动了 GUI；参数转发属于任务 004 的范围。

use std::ffi::OsString;
use std::process::ExitCode;

/// 用法错误（收到暂不支持的参数）；取 BSD sysexits.h 的 EX_USAGE。
const EXIT_USAGE: u8 = 64;
/// 请求的能力当前不可用（桌面尚未接入）；取 BSD sysexits.h 的 EX_UNAVAILABLE。
const EXIT_DESKTOP_NOT_CONNECTED: u8 = 69;

fn main() -> ExitCode {
    run(std::env::args_os().skip(1))
}

/// 依据命令行参数输出诊断并决定退出码；与进程环境解耦以便测试。
fn run<I>(arguments: I) -> ExitCode
where
    I: Iterator<Item = OsString>,
{
    let extra = arguments.count();
    if extra > 0 {
        eprintln!(
            "panta-launcher: 暂不接受参数（收到 {extra} 个）；参数转发随桌面接入实现，见任务 004。"
        );
        return ExitCode::from(EXIT_USAGE);
    }
    eprintln!(
        "panta-launcher: 桌面可执行文件尚未接入（任务 004/005），本次未启动 GUI；当前仅 Rust 骨架可用。"
    );
    ExitCode::from(EXIT_DESKTOP_NOT_CONNECTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_desktop_not_connected_without_arguments() {
        assert_eq!(
            run(std::iter::empty()),
            ExitCode::from(EXIT_DESKTOP_NOT_CONNECTED)
        );
    }

    #[test]
    fn reports_usage_error_with_arguments() {
        let arguments = ["--gui"].map(OsString::from);
        assert_eq!(run(arguments.into_iter()), ExitCode::from(EXIT_USAGE));
    }
}
