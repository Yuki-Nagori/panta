//! 固定 nightly 的纯 Rust UB 检查；与 native sanitizer 分开编排。

use std::error::Error;
use std::process::Command;

use crate::command::run;
use crate::paths::{repository_root, target_root};

/// Miri 只随 nightly 发布：固定日期保证可复现，升级时同步回填 032 验证表。
const MIRI_NIGHTLY: &str = "nightly-2026-09-15";
/// Miri 只解释纯 Rust crate：CXX FFI 调用与进程/构建类 crate（panta-build、
/// panta-tests、launcher）不在 Miri 语义内。
const MIRI_PACKAGES: &[&str] = &["panta-core", "panta-dsl-core", "panta-foundation"];

/// Miri 解释执行纯 Rust crate 测试：工具链是固定日期 nightly（rustup 组件），
/// 产物走独立 target/miri，不污染常规构建缓存。路径/日志类测试按设计使用
/// 真实文件系统夹具，因此关闭隔离放行宿主文件操作；UB 与数据竞争检查不受
/// 隔离开关影响。
pub(super) fn miri() -> Result<(), Box<dyn Error>> {
    let mut install = Command::new("rustup");
    install.args(["toolchain", "install", MIRI_NIGHTLY]).args([
        "--profile",
        "minimal",
        "--component",
        "miri",
        "--no-self-update",
    ]);
    run("安装 Miri 工具链", install)?;
    let mut test = Command::new("cargo");
    test.arg(format!("+{MIRI_NIGHTLY}"))
        .args(["miri", "test", "--locked"]);
    for package in MIRI_PACKAGES {
        test.args(["-p", package]);
    }
    test.env("MIRIFLAGS", "-Zmiri-disable-isolation")
        .env("CARGO_TARGET_DIR", target_root().join("miri"))
        .current_dir(repository_root()?);
    run("cargo miri test", test)
}
