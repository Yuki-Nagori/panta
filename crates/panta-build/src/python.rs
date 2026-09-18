//! Python 仅服务开发工具；解释器、环境和缓存都位于当前 Cargo target 内。
use super::{Tool, ToolAsset, ensure_tool};
use std::path::Path;
use std::process::Command;

pub const UV_VERSION: &str = "0.8.22";
pub const PYTHON_VERSION: &str = "3.13.7";

pub(super) fn uv_asset() -> Option<ToolAsset> {
    let (filename, sha256) = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        (
            "uv-aarch64-apple-darwin.tar.gz",
            "3f61099e261e449527141dbf125629fab33ad696468c8c90cebbac40185a306c",
        )
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        (
            "uv-x86_64-unknown-linux-gnu.tar.gz",
            "741ff1f5742c5a4a25d2f829e8395355e43f7a5ae2ebc6368e9ae2df0efb69cf",
        )
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        (
            "uv-x86_64-pc-windows-msvc.zip",
            "5049375aa2a5162f132b2c1cb992e25d42d47d934cab8c174dbe6f60973dcc12",
        )
    } else {
        return None;
    };
    let url = match filename {
        "uv-aarch64-apple-darwin.tar.gz" => {
            "https://github.com/astral-sh/uv/releases/download/0.8.22/uv-aarch64-apple-darwin.tar.gz"
        }
        "uv-x86_64-unknown-linux-gnu.tar.gz" => {
            "https://github.com/astral-sh/uv/releases/download/0.8.22/uv-x86_64-unknown-linux-gnu.tar.gz"
        }
        _ => {
            "https://github.com/astral-sh/uv/releases/download/0.8.22/uv-x86_64-pc-windows-msvc.zip"
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
