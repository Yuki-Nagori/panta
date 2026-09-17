//! panta 统一运行入口。
//!
//! 启动 CMake 构建的 native 产物并转发参数与退出码（任务 004）；桌面能力
//! 由任务 005 在 native/app 接入。产物路径由 build.rs 在编译期注入
//! （PANTA_NATIVE_BIN），定位不依赖启动时的当前目录。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// 托管引导（任务 020）的单元测试挂在本 crate 的测试构建上；生产二进制
// 不编译该模块，build.rs 经 #[path] 复用同一实现文件。测试构建只调用
// 测试函数，其余条目由 build.rs 使用，故放开 dead_code。
#[cfg(test)]
#[allow(dead_code)]
mod provision;

/// native 产物缺失或无法启动；取 BSD sysexits.h 的 EX_UNAVAILABLE。
const EXIT_PRODUCT_UNAVAILABLE: u8 = 69;
/// 子进程被信号终止且平台无法给出信号号时的回退退出码（对应 SIGINT 约定）。
const EXIT_SIGNAL_FALLBACK: u8 = 130;

fn main() -> ExitCode {
    let product = native_product();
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    match run(product.as_deref(), &arguments) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("panta-launcher: {message}");
            ExitCode::from(EXIT_PRODUCT_UNAVAILABLE)
        }
    }
}

/// 构建脚本注入的 native 产物绝对路径；含空格也安全（编译期字符串，运行期
/// 以参数数组传递，不经 shell）。
fn native_product() -> Option<PathBuf> {
    option_env!("PANTA_NATIVE_BIN").map(PathBuf::from)
}

/// 启动 native 产物：stdout/stderr 直接继承，退出码原样转发。
/// product 缺失或启动失败时返回诊断，由调用方以固定退出码报告。
fn run(product: Option<&Path>, arguments: &[OsString]) -> Result<ExitCode, String> {
    let Some(product) = product else {
        return Err("构建配置异常：未注入 PANTA_NATIVE_BIN（应总是由 build.rs 提供）".to_string());
    };
    if !product.exists() {
        return Err(format!(
            "native 产物不存在：{}；请先执行 cargo build（诊断见 ai-docs/architecture/build-and-development.md）",
            product.display()
        ));
    }

    let mut command = std::process::Command::new(product);
    command.args(arguments);
    match command.status() {
        Ok(status) => Ok(exit_code_of(&status)),
        Err(error) => Err(format!("无法启动 {}：{error}", product.display())),
    }
}

/// 退出码转发规则：平台退出码取低 8 位（POSIX 语义；Windows 上超出 255 的
/// 系统码会被截断，桌面化时由 005 重新审视）；信号终止映射为 128+信号号，
/// 无信号号信息时回退到 SIGINT 约定的 130。
fn exit_code_of(status: &std::process::ExitStatus) -> ExitCode {
    if let Some(code) = status.code() {
        return ExitCode::from((code & 0xFF) as u8);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return ExitCode::from((128 + signal).min(u8::MAX.into()) as u8);
        }
    }
    ExitCode::from(EXIT_SIGNAL_FALLBACK)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn missing_product_reports_unavailable() {
        let missing = Path::new("/panta/definitely/missing/panta-native");
        let error = run(Some(missing), &[]).unwrap_err();
        assert!(error.contains("native 产物不存在"));
    }

    #[test]
    fn absent_injection_reports_configuration_error() {
        let error = run(None, &[]).unwrap_err();
        assert!(error.contains("PANTA_NATIVE_BIN"));
    }

    #[cfg(unix)]
    #[test]
    fn forwards_child_exit_code() {
        let code = run(Some(Path::new("/usr/bin/false")), &[]).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[cfg(unix)]
    #[test]
    fn forwards_child_success() {
        let code = run(Some(Path::new("/usr/bin/true")), &[]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
