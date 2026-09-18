//! 托管引导（任务 020）：CMake/Ninja 预编译二进制的定位、下载、校验与缓存。
//!
//! 原则（standards/dependency-acquisition.md）：
//! - 默认只使用工作区 `target/` 托管缓存 → 按固定清单下载；绝不静默
//!   采用 PATH 中的系统工具或源码编译。
//! - 不支持固定资产的平台可显式设置 `PANTA_USE_SYSTEM_TOOLS=1`，再用
//!   `CMAKE` 或 PATH 提供宿主工具；CI 和受支持平台不走该旁路。
//! - 只消费带 SHA256 的官方 release 资产；哈希不符立即删除归档并报错，
//!   不进入构建图；禁止覆盖已校验的不同版本资产（marker 不符先清场）。
//! - 缓存位于根 `target/panta-tools/`，与 profile 无关；marker 与二进制
//!   同时存在即可离线重复构建。
//! - 下载用平台自带 `curl`，解包用平台自带 `tar`（CMake 压缩包）与
//!   `cmake -E tar`（Ninja zip 需要 libarchive，三平台零额外工具）；
//!   这些都是 OS 自带组件，符合"开发者只装 rustup 与平台编译器"。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// 单个工具的官方资产：URL 与首次下载实测的 SHA256（升级时同步回填
/// standards/dependency-acquisition.md）。
struct ToolAsset {
    url: &'static str,
    sha256: &'static str,
}

const CMAKE_VERSION: &str = "4.4.3";
const NINJA_VERSION: &str = "1.13.2";

fn cmake_asset() -> Option<ToolAsset> {
    // macOS 资产为 universal（arm64/x86_64）；Linux/Windows 官方仅 x86_64，
    // 其他架构返回 None 走诊断（见 resolve_cmake）。
    if cfg!(target_os = "macos") {
        Some(ToolAsset {
            url: "https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3-macos-universal.tar.gz",
            sha256: "0c5d65251c14cc884bfa16bdbed3c263ce5bffe2e21c0d0d00962cb0610464fa",
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3-linux-x86_64.tar.gz",
            sha256: "d6c83076c575bc00b823522ac974bda66d0af05d6ddc30e739c12385cf32c6cc",
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3-windows-x86_64.zip",
            sha256: "4d52ebab7193a698651639ed80d8d04fd903358843572cf44c7fd234cb7c26ab",
        })
    } else {
        None
    }
}

/// clang-format 独立二进制(任务 032):LLVM 官方不发布独立资产,采用
/// muttleyxd/clang-tools-static-binaries 对 LLVM 20.1.0 源码的静态构建
/// (公开构建脚本，下载资产 SHA256 固定；仅用于格式检查)。
fn clang_format_asset() -> Option<ToolAsset> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Some(ToolAsset {
            url: "https://github.com/muttleyxd/clang-tools-static-binaries/releases/download/master-796e77c/clang-format-20_macos-arm-arm64",
            sha256: "fe6b8450a8cf83de3f517e3b9a9b1bb925613e5fb59145d6d24ccca5fe17d442",
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/muttleyxd/clang-tools-static-binaries/releases/download/master-796e77c/clang-format-20_linux-amd64",
            sha256: "e900c1e520b6c9b9c99e43c0f45ccd12927838741cfc60c077a33dec69bb60cc",
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/muttleyxd/clang-tools-static-binaries/releases/download/master-796e77c/clang-format-20_windows-amd64.exe",
            sha256: "44011742f30b2ebfd9013aa2b07d802b1b474186fb7904a2f773296f27ff15f9",
        })
    } else {
        None
    }
}

fn ninja_asset() -> Option<ToolAsset> {
    if cfg!(target_os = "macos") {
        Some(ToolAsset {
            url: "https://github.com/ninja-build/ninja/releases/download/v1.13.2/ninja-mac.zip",
            sha256: "c99048673aa765960a99cf10c6ddb9f1fad506099ff0a0e137ad8960a88f321b",
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/ninja-build/ninja/releases/download/v1.13.2/ninja-linux.zip",
            sha256: "5749cbc4e668273514150a80e387a957f933c6ed3f5f11e03fb30955e2bbead6",
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/ninja-build/ninja/releases/download/v1.13.2/ninja-win.zip",
            sha256: "07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65",
        })
    } else {
        None
    }
}

/// 解析 clang-format（任务 032）：缓存于 `panta-tools/clang-format/`，
/// 资产即单文件二进制（无解包步骤），SHA256 校验后置可执行位。
pub fn resolve_clang_format(target_root: &Path) -> Result<PathBuf, String> {
    let Some(asset) = clang_format_asset() else {
        return Err(format!(
            "本平台（{}/{}）没有固定的 clang-format 资产；请登记后另立供给任务",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    };
    // 版本和摘要进入缓存键，升级不能复用旧二进制。
    let dir = target_root
        .join("panta-tools")
        .join("clang-format")
        .join("20.1.0")
        .join(asset.sha256);
    let binary_name = if cfg!(windows) {
        "clang-format.exe"
    } else {
        "clang-format"
    };
    let binary = dir.join(binary_name);

    if binary.is_file() {
        let actual = sha256_file(&binary)?;
        if actual != asset.sha256 {
            return Err(format!(
                "clang-format 缓存校验失败：{}；请删除该文件后重试",
                binary.display()
            ));
        }
        return Ok(binary);
    }

    fs::create_dir_all(&dir).map_err(|error| format!("创建 {} 失败：{error}", dir.display()))?;
    let staging = dir.join("clang-format-20.download");
    if staging.exists() {
        let actual = sha256_file(&staging)?;
        if actual != asset.sha256 {
            let _ = fs::remove_file(&staging);
        }
    }
    if !staging.exists() {
        download(&asset, &staging, "clang-format", "20.1.0")?;
    }
    let actual = sha256_file(&staging)?;
    if actual != asset.sha256 {
        let _ = fs::remove_file(&staging);
        return Err(format!(
            "clang-format SHA256 不符：预期 {}，实际 {}；已删除，重试将重新下载",
            asset.sha256, actual
        ));
    }
    set_executable(&staging)?;
    fs::rename(&staging, &binary)
        .map_err(|error| format!("落位 {} 失败：{error}", binary.display()))?;
    Ok(binary)
}

/// 解析 CMake：默认使用托管缓存/下载；只有显式开启系统工具旁路，或
/// 当前平台没有固定资产时，才读取 `CMAKE`/PATH。
pub fn resolve_cmake(target_root: &Path) -> Result<PathBuf, String> {
    if use_system_tools() || cmake_asset().is_none() {
        if let Some(path) = std::env::var_os("CMAKE") {
            let path = PathBuf::from(path);
            if path.is_file() {
                return Ok(path);
            }
            return Err(format!(
                "CMAKE 环境变量指向的可执行文件不存在：{}",
                path.display()
            ));
        }
        if let Some(path) = find_on_path("cmake") {
            return Ok(path);
        }
    }
    ensure_tool(Tool::Cmake, target_root, None)
}

/// 解析 Ninja：默认使用托管缓存/下载。Ninja zip 需要 libarchive 解包，
/// 复用上一步解析到的 cmake（系统或托管）。只有显式开启系统工具旁路，
/// 或当前平台没有固定资产时，才读取 PATH。
pub fn resolve_ninja(target_root: &Path, cmake: &Path) -> Result<PathBuf, String> {
    if (use_system_tools() || ninja_asset().is_none())
        && let Some(path) = find_on_path("ninja")
    {
        return Ok(path);
    }
    ensure_tool(Tool::Ninja, target_root, Some(cmake))
}

fn use_system_tools() -> bool {
    matches!(
        std::env::var("PANTA_USE_SYSTEM_TOOLS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

#[derive(Clone, Copy)]
enum Tool {
    Cmake,
    Ninja,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Cmake => "cmake",
            Tool::Ninja => "ninja",
        }
    }

    fn version(self) -> &'static str {
        match self {
            Tool::Cmake => CMAKE_VERSION,
            Tool::Ninja => NINJA_VERSION,
        }
    }

    fn asset(self) -> Option<ToolAsset> {
        match self {
            Tool::Cmake => cmake_asset(),
            Tool::Ninja => ninja_asset(),
        }
    }

    /// 解包后二进制的定位：CMake 在 `<top>/bin/`（macOS 额外嵌套
    /// CMake.app/Contents），Ninja 在解包根。
    fn locate_installed(self, root: &Path) -> Option<PathBuf> {
        let exe = exe_name(self.name());
        match self {
            Tool::Ninja => {
                let candidate = root.join(&exe);
                candidate.is_file().then_some(candidate)
            }
            Tool::Cmake => find_cmake_binary(root, &exe, 0),
        }
    }
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

fn tools_root(target_root: &Path) -> PathBuf {
    target_root.join("panta-tools")
}

/// 深度受限地查找 `bin/cmake`；返回第一个命中的路径。
fn find_cmake_binary(dir: &Path, exe: &str, depth: usize) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }
    let bin_dir = dir.join("bin");
    let candidate = bin_dir.join(exe);
    if candidate.is_file() {
        return Some(candidate);
    }
    let entries = fs::read_dir(dir).ok()?;
    let mut paths = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            paths.push(path);
        }
    }
    paths.sort();
    paths
        .iter()
        .find_map(|path| find_cmake_binary(path, exe, depth + 1))
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exe = exe_name(name);
    std::env::split_paths(&path)
        .map(|directory| directory.join(&exe))
        .find(|candidate| candidate.is_file())
}

/// 定位 → 缺失时按清单获取 → 校验 → 解包 → 写 marker。`cmake` 仅在
/// 解压 Ninja zip 时使用。
fn ensure_tool(tool: Tool, target_root: &Path, cmake: Option<&Path>) -> Result<PathBuf, String> {
    let name = tool.name();
    let Some(asset) = tool.asset() else {
        return Err(format!(
            "本平台（{}/{}/托管清单）没有固定的 {name} 预编译资产；\
             请登记固定资产，或显式设置 PANTA_USE_SYSTEM_TOOLS=1 后用 CMAKE/PATH 提供本机工具；\
             供给资产登记见 ai-docs/standards/dependency-acquisition.md",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    };

    let root = tools_root(target_root);
    let tool_dir = root.join(name);
    let marker = root.join(format!("{name}.sha256"));
    let archives = root.join("archives");
    let archive = archives.join(format!("{}-{}.archive", name, tool.version()));

    // 命中缓存：marker 哈希一致且二进制存在即可离线复用。
    if fs::read_to_string(&marker)
        .map(|recorded| recorded.trim() == asset.sha256)
        .unwrap_or(false)
        && let Some(binary) = tool.locate_installed(&tool_dir)
    {
        return Ok(binary);
    }

    // marker 缺失或不一致 = 首次供给或版本升级：清场后重建，不覆盖已校验资产。
    let _ = fs::remove_dir_all(&tool_dir);
    let _ = fs::remove_file(&marker);
    fs::create_dir_all(&tool_dir)
        .map_err(|error| format!("创建 {} 失败：{error}", tool_dir.display()))?;
    fs::create_dir_all(&archives)
        .map_err(|error| format!("创建 {} 失败：{error}", archives.display()))?;

    // 归档命中预期哈希则跳过下载（支持离线重建）；不符或残缺立即删除重下。
    let archive_ready = match fs::metadata(&archive) {
        Ok(_) => {
            let actual = sha256_file(&archive)?;
            if actual == asset.sha256 {
                true
            } else {
                let _ = fs::remove_file(&archive);
                false
            }
        }
        Err(_) => false,
    };
    if !archive_ready {
        download(&asset, &archive, name, tool.version())?;
    }

    let actual = sha256_file(&archive)?;
    if actual != asset.sha256 {
        let _ = fs::remove_file(&archive);
        return Err(format!(
            "{name} 归档 SHA256 不符，已拒绝进入构建：预期 {}，实际 {}；\
             归档已删除，重试将重新下载（{asset_url}）",
            asset.sha256,
            actual,
            asset_url = asset.url
        ));
    }

    extract(tool, &archive, &tool_dir, cmake)?;
    let binary = tool.locate_installed(&tool_dir).ok_or_else(|| {
        format!(
            "{name} 解包完成但在 {} 下未找到二进制；目录布局可能变化，\
             请核对资产并更新供给脚本",
            tool_dir.display()
        )
    })?;
    set_executable(&binary)?;

    fs::write(&marker, format!("{}\n", asset.sha256))
        .map_err(|error| format!("写入 marker {} 失败：{error}", marker.display()))?;
    Ok(binary)
}

fn download(
    asset: &ToolAsset,
    destination: &Path,
    name: &str,
    version: &str,
) -> Result<(), String> {
    let mut command = Command::new("curl");
    command
        .args(["-fSL", "--retry", "3", "--create-dirs", "-o"])
        .arg(destination)
        .arg(asset.url);
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!(
            "下载 {name} {version} 失败（curl 退出码 {}）：{}；\
             可手动下载后放置到 {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "信号".to_owned()),
            asset.url,
            destination.display()
        )),
        Err(error) => Err(format!(
            "无法执行 curl：{error}。{name} {version} 需经 {} 下载；\
             平台应自带 curl，受支持平台不使用本机 CMake/Ninja 旁路",
            asset.url
        )),
    }
}

fn extract(
    tool: Tool,
    archive: &Path,
    destination: &Path,
    cmake: Option<&Path>,
) -> Result<(), String> {
    let result = match tool {
        // CMake 压缩包由平台自带 tar 解开（macOS/Windows 为 bsdtar，可直接
        // 读 zip；Linux 为 GNU tar，自动识别 gzip）。
        Tool::Cmake => Command::new("tar")
            .arg("-xf")
            .arg(archive)
            .arg("-C")
            .arg(destination)
            .status(),
        Tool::Ninja => {
            let Some(cmake) = cmake else {
                return Err("内部错误：解包 Ninja 需要 cmake 路径".to_owned());
            };
            Command::new(cmake)
                .args(["-E", "tar", "xf"])
                .arg(archive)
                .current_dir(destination)
                .status()
        }
    };
    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!(
            "{name} 解包失败（退出码 {}）",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "信号".to_owned()),
            name = tool.name()
        )),
        Err(error) => Err(format!("无法执行解包工具：{error}")),
    }
}

fn set_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)
            .map_err(|error| format!("读取 {} 失败：{error}", path.display()))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .map_err(|error| format!("设置 {} 可执行位失败：{error}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        // Windows zip 内的 .exe 自带可执行语义，无需额外处理。
        let _ = path;
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("打开 {} 失败：{error}", path.display()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|error| format!("读取 {} 失败：{error}", path.display()))?;
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{cmake_asset, exe_name, find_cmake_binary, hex, ninja_asset, sha256_file};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn hex_encodes_lowercase_fixed_width() {
        assert_eq!(hex(&[]), "");
        assert_eq!(hex(&[0x0f, 0xa0]), "0fa0");
        assert_eq!(hex(&[255; 32]).len(), 64);
    }

    #[test]
    fn host_platform_has_well_formed_assets() {
        for asset in [cmake_asset(), ninja_asset()] {
            let asset = match asset {
                Some(asset) => asset,
                None => panic!("宿主平台应有固定资产"),
            };
            assert!(asset.url.starts_with("https://"));
            assert_eq!(asset.sha256.len(), 64);
            assert!(asset.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn sha256_file_matches_known_vector() {
        let dir = std::env::temp_dir().join(format!("panta-provision-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap_or_else(|error| panic!("mkdir failed: {error}"));
        let file = dir.join("sample.txt");
        fs::write(&file, b"panta").unwrap_or_else(|error| panic!("write failed: {error}"));
        // printf 'panta' | shasum -a 256
        assert_eq!(
            sha256_file(&file).unwrap_or_else(|error| panic!("hash failed: {error}")),
            "40fb6f5cbab6a2ac4a8da4171e991f5b3474b296a259d78fb12111ffce243d56"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn finds_cmake_binary_in_nested_layout() {
        let dir = std::env::temp_dir().join(format!("panta-provision-tree-{}", std::process::id()));
        let nested = dir
            .join("cmake-4.4.3-macos-universal")
            .join("CMake.app")
            .join("Contents")
            .join("bin");
        fs::create_dir_all(&nested).unwrap_or_else(|error| panic!("mkdir failed: {error}"));
        let binary = nested.join(exe_name("cmake"));
        fs::write(&binary, b"MZ").unwrap_or_else(|error| panic!("write failed: {error}"));

        let found = find_cmake_binary(&dir, &exe_name("cmake"), 0);
        assert_eq!(found, Some(binary));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_binary_returns_none() {
        let dir =
            std::env::temp_dir().join(format!("panta-provision-empty-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap_or_else(|error| panic!("mkdir failed: {error}"));
        assert_eq!(find_cmake_binary(&dir, exe_name("cmake").as_str(), 0), None);
        assert_eq!(
            find_cmake_binary(
                &PathBuf::from("/nonexistent-panta"),
                exe_name("cmake").as_str(),
                0
            ),
            None
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
