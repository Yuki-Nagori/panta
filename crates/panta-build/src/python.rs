//! Python 仅服务开发工具；解释器、环境和缓存都位于当前 Cargo target 内。
use super::{Tool, ToolAsset, ensure_tool};
use std::path::Path;
use std::process::Command;

pub const UV_VERSION: &str = "0.12.18";
pub const PYTHON_VERSION: &str = "3.14.7";

pub(super) fn uv_asset() -> Option<ToolAsset> {
    let (filename, sha256) = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        (
            "uv-aarch64-apple-darwin.tar.gz",
            "cf40e0c6a202190ccd9e0406dcfdd5b2d6668a9a5c779b17948963df32aafe5b",
        )
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        (
            "uv-x86_64-unknown-linux-gnu.tar.gz",
            "89eadd7c76fc063887959510d5ba0ab1264dfd5f1143b925ddb73021a40acf16",
        )
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        (
            "uv-x86_64-pc-windows-msvc.zip",
            "cae6a3bc25239f83dffb467a4b180508d9da23986c04639ebfa44e43e6a84bff",
        )
    } else {
        return None;
    };
    let url = match filename {
        "uv-aarch64-apple-darwin.tar.gz" => {
            "https://releases.astral.sh/github/uv/releases/download/0.12.18/uv-aarch64-apple-darwin.tar.gz"
        }
        "uv-x86_64-unknown-linux-gnu.tar.gz" => {
            "https://releases.astral.sh/github/uv/releases/download/0.12.18/uv-x86_64-unknown-linux-gnu.tar.gz"
        }
        _ => {
            "https://releases.astral.sh/github/uv/releases/download/0.12.18/uv-x86_64-pc-windows-msvc.zip"
        }
    };
    Some(ToolAsset { url, sha256 })
}

/// uv 自身按 SHA256 安装；其固定版本携带 Python 下载清单及校验和，禁止选择系统 Python。
pub fn command(target: &Path, project: &Path, tool: &str) -> Result<Command, String> {
    let uv = ensure_tool(Tool::Uv, target, None)?;
    let root = target.join("panta-tools/python");
    let mut command = Command::new(uv);
    command
        .current_dir(project)
        .env("UV_CACHE_DIR", root.join("cache"))
        .env("UV_PYTHON_INSTALL_DIR", root.join("interpreters"))
        .env("UV_PYTHON_BIN_DIR", root.join("bin"))
        .env("UV_PROJECT_ENVIRONMENT", root.join("venv"))
        .env("UV_TOOL_DIR", root.join("tools"))
        .args([
            "run",
            "--locked",
            "--managed-python",
            "--python",
            PYTHON_VERSION,
            "--no-build",
            "--project",
        ])
        .arg(project)
        .arg(tool);
    Ok(command)
}
