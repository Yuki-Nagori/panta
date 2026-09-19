//! 托管引导（任务 020/042）：LLVM、CMake/Ninja 预编译二进制的定位、下载、
//! 校验与缓存。
//!
//! 固定资产按版本/摘要隔离；安装持有 OS 文件锁，失败保留旧版本，成功才发布。
//! 运行期也可复用此模块，读取工具路径时才准备该工具，不在 runner 编译期下载。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use fs2::FileExt;
use sha2::{Digest, Sha256};

pub mod database;
pub mod python;

/// 单个工具的官方资产：URL 与首次下载实测的 SHA256（升级时同步回填
/// standards/dependency-acquisition.md）。
struct ToolAsset {
    url: &'static str,
    sha256: &'static str,
}

const CMAKE_VERSION: &str = "4.4.3";
const NINJA_VERSION: &str = "1.13.2";
pub const LLVM_VERSION: &str = "22.1.7";

/// 两条 C++ 构建链必须消费同一份 LLVM 目录中的工具。
#[derive(Clone, Debug)]
pub struct LlvmCompilers {
    pub root: PathBuf,
    pub clang: PathBuf,
    pub clangxx: PathBuf,
    pub clang_cl: Option<PathBuf>,
    pub clang_format: PathBuf,
}

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

/// LLVM 官方发布资产。Windows 使用官方 NSIS 安装包：它包含 clang-cl、lld-link
/// 与 clang-format，静默安装到 Cargo 的托管目录，不写入用户系统目录。
fn llvm_asset() -> Option<ToolAsset> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Some(ToolAsset {
            url: "https://github.com/llvm/llvm-project/releases/download/llvmorg-22.1.7/LLVM-22.1.7-macOS-ARM64.tar.xz",
            sha256: "4177245188b0a30a6539c96b361dea56f253485756bfd8927a6a59e7301e7806",
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/llvm/llvm-project/releases/download/llvmorg-22.1.7/LLVM-22.1.7-Linux-X64.tar.xz",
            sha256: "edb0522b41e261819c06ea437d249f9b8acfa413d3805bc9920eec6fb76ff830",
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some(ToolAsset {
            url: "https://github.com/llvm/llvm-project/releases/download/llvmorg-22.1.7/LLVM-22.1.7-win64.exe",
            sha256: "e091fcf965ce589c83c0f7c5356b2fcf3e658a8ec990bfcf79cce4389a0d1eb3",
        })
    } else {
        None
    }
}

/// 解析 Cargo CXX 与 CMake 共用的固定 LLVM 工具链。
pub fn resolve_llvm_compilers(target_root: &Path) -> Result<LlvmCompilers, String> {
    if use_system_tools() {
        return resolve_system_llvm();
    }

    let primary = ensure_tool(Tool::Llvm, target_root, None)?;
    let bin_dir = primary
        .parent()
        .ok_or_else(|| format!("LLVM 编译器路径没有父目录：{}", primary.display()))?;
    let root = bin_dir
        .parent()
        .ok_or_else(|| format!("LLVM bin 目录没有安装根：{}", bin_dir.display()))?
        .to_path_buf();
    let clang = find_binary(&root, "clang")
        .ok_or_else(|| format!("LLVM {} 缺少 clang：{}", LLVM_VERSION, root.display()))?;
    let clangxx = find_binary(&root, "clang++")
        .ok_or_else(|| format!("LLVM {} 缺少 clang++：{}", LLVM_VERSION, root.display()))?;
    let clang_format = find_binary(&root, "clang-format").ok_or_else(|| {
        format!(
            "LLVM {} 缺少 clang-format：{}",
            LLVM_VERSION,
            root.display()
        )
    })?;
    let clang_cl = find_binary(&root, "clang-cl");
    for path in [&clang, &clangxx, &clang_format] {
        set_executable(path)?;
    }
    if let Some(path) = &clang_cl {
        set_executable(path)?;
    }
    Ok(LlvmCompilers {
        root,
        clang,
        clangxx,
        clang_cl,
        clang_format,
    })
}

fn resolve_system_llvm() -> Result<LlvmCompilers, String> {
    let cxx_name = if cfg!(windows) { "clang-cl" } else { "clang++" };
    let c_name = if cfg!(windows) { "clang-cl" } else { "clang" };
    let clangxx = env_or_path("CXX", cxx_name)?;
    let clang = env_or_path("CC", c_name)?;
    let clang_format = env_or_path("CLANG_FORMAT", "clang-format")?;
    let root = clangxx
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let clang_cl = cfg!(windows).then_some(clangxx.clone());
    Ok(LlvmCompilers {
        root,
        clang,
        clangxx,
        clang_cl,
        clang_format,
    })
}

fn env_or_path(variable: &str, name: &str) -> Result<PathBuf, String> {
    if let Some(value) = std::env::var_os(variable) {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "{variable} 环境变量指向的编译器不存在：{}",
            path.display()
        ));
    }
    find_on_path(name).ok_or_else(|| format!("系统工具旁路需要 {variable} 或 PATH 中的 {name}"))
}

/// 解析 CMake：仅在显式开启系统工具旁路后读取 `CMAKE`/PATH。
pub fn resolve_cmake(target_root: &Path) -> Result<PathBuf, String> {
    if use_system_tools() {
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
/// 才读取 PATH。
pub fn resolve_ninja(target_root: &Path, cmake: &Path) -> Result<PathBuf, String> {
    if use_system_tools()
        && let Some(path) = find_on_path("ninja")
    {
        return Ok(path);
    }
    ensure_tool(Tool::Ninja, target_root, Some(cmake))
}

pub fn use_system_tools() -> bool {
    matches!(
        std::env::var("PANTA_USE_SYSTEM_TOOLS").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

#[derive(Clone, Copy)]
enum Tool {
    Cmake,
    Ninja,
    Llvm,
    Uv,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Cmake => "cmake",
            Tool::Ninja => "ninja",
            Tool::Llvm => "llvm",
            Tool::Uv => "uv",
        }
    }

    fn version(self) -> &'static str {
        match self {
            Tool::Cmake => CMAKE_VERSION,
            Tool::Ninja => NINJA_VERSION,
            Tool::Llvm => LLVM_VERSION,
            Tool::Uv => python::UV_VERSION,
        }
    }

    fn asset(self) -> Option<ToolAsset> {
        match self {
            Tool::Cmake => cmake_asset(),
            Tool::Ninja => ninja_asset(),
            Tool::Llvm => llvm_asset(),
            Tool::Uv => python::uv_asset(),
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
            Tool::Uv => find_binary(root, "uv"),
            Tool::Cmake => find_cmake_binary(root, &exe, 0),
            Tool::Llvm => {
                let compiler = if cfg!(windows) { "clang-cl" } else { "clang++" };
                find_binary(root, compiler)
            }
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

/// 在 LLVM 解包目录中查找工具；官方压缩包在顶层带有版本目录，Windows
/// 归档和 Unix 归档的布局因此统一按相对路径处理。
fn find_binary(dir: &Path, name: &str) -> Option<PathBuf> {
    let exe = exe_name(name);
    let mut directories = vec![(dir.to_owned(), 0usize)];
    while let Some((directory, depth)) = directories.pop() {
        if depth > 6 {
            continue;
        }
        for candidate in [directory.join(&exe), directory.join("bin").join(&exe)] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        let entries = fs::read_dir(&directory).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                directories.push((path, depth + 1));
            }
        }
    }
    None
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
    // 覆盖率使用独立对象目录，但复用主 target 已校验的编译工具。
    let cache_root = std::env::var_os("PANTA_TOOL_CACHE_ROOT").map(PathBuf::from);
    let target_root = cache_root.as_deref().unwrap_or(target_root);
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

    let identity = format!("{}-{}", tool.version(), asset.sha256);
    let directory = install_directory(target_root, name, &identity, |staging| {
        let archives = tools_root(target_root).join("archives");
        fs::create_dir_all(&archives).map_err(|e| e.to_string())?;
        let suffix = if cfg!(windows) && matches!(tool, Tool::Llvm) {
            "exe"
        } else {
            "archive"
        };
        let archive = archives.join(format!("{name}-{identity}.{suffix}"));
        eprintln!("[panta-tools] {name}：检查归档缓存 {}", archive.display());
        if !archive.is_file() || sha256_file(&archive)? != asset.sha256 {
            let partial = archives.join(format!("{name}-{identity}.partial"));
            download(&asset, &partial, name, tool.version())?;
            eprintln!("[panta-tools] {name}：校验下载归档 SHA256");
            let actual = sha256_file(&partial)?;
            if actual != asset.sha256 {
                fs::remove_file(&partial).map_err(|e| e.to_string())?;
                return Err(format!(
                    "{name} SHA256 不符：期望 {}，实际 {actual}",
                    asset.sha256
                ));
            }
            if archive.exists() {
                fs::remove_file(&archive).map_err(|e| e.to_string())?;
            }
            fs::rename(&partial, &archive).map_err(|e| e.to_string())?;
        }
        extract(tool, &archive, staging, cmake)?;
        let binary = tool
            .locate_installed(staging)
            .ok_or_else(|| format!("{name} 解包后缺少可执行文件"))?;
        set_executable(&binary)
    })?;
    tool.locate_installed(&directory).ok_or_else(|| {
        format!(
            "托管 {name} 缓存损坏：{}；清理该版本目录后重试",
            directory.display()
        )
    })
}

/// OS 锁在进程退出时自动释放；锁文件不能删除，否则等待者可能锁住不同 inode。
pub fn install_lock(target_root: &Path, name: &str) -> Result<fs::File, String> {
    let locks = tools_root(target_root).join("locks");
    fs::create_dir_all(&locks).map_err(|e| e.to_string())?;
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(locks.join(format!("{name}.lock")))
        .map_err(|e| e.to_string())?;
    eprintln!("[panta-tools] {name}：等待安装锁");
    file.lock_exclusive()
        .map_err(|e| format!("获取 {name} 安装锁失败：{e}"))?;
    eprintln!("[panta-tools] {name}：已获得安装锁");
    Ok(file)
}

/// 在互斥区内构建临时目录，写入完成标记后原子重命名；中断残留只在下次持锁时清理。
pub fn install_directory(
    target_root: &Path,
    name: &str,
    identity: &str,
    install: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    let _lock = install_lock(target_root, name)?;
    let parent = tools_root(target_root).join(name);
    let destination = parent.join(identity);
    if fs::read_to_string(destination.join(".complete"))
        .ok()
        .as_deref()
        == Some(identity)
    {
        eprintln!("[panta-tools] {name}：命中已完成缓存");
        return Ok(destination);
    }
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let staging = parent.join(format!(".{identity}.staging"));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    install(&staging)?;
    fs::write(staging.join(".complete"), identity).map_err(|e| e.to_string())?;
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }
    fs::rename(&staging, &destination).map_err(|e| e.to_string())?;
    eprintln!("[panta-tools] {name}：安装完成 {}", destination.display());
    Ok(destination)
}

/// Ninja 使用 Visual Studio 的 SDK/CRT 搜索环境，但编译器始终显式指定托管 clang-cl。
pub fn windows_sdk_env(
    target: &str,
) -> Result<Vec<(std::ffi::OsString, std::ffi::OsString)>, String> {
    if !target.contains("windows-msvc") {
        return Ok(Vec::new());
    }
    cc::windows_registry::find_tool(target, "cl.exe")
        .map(|tool| tool.env().to_vec())
        .ok_or_else(|| "未找到 MSVC Build Tools / Windows SDK；请安装平台 SDK 后重试".to_owned())
}

/// 为 native 测试补齐托管 Qt DLL 的运行时搜索路径。
pub fn native_test_env(
    target_root: &Path,
    native_dir: &Path,
    target: &str,
) -> Result<Vec<(std::ffi::OsString, std::ffi::OsString)>, String> {
    let mut environment = windows_sdk_env(target)?;
    if !cfg!(windows) {
        return Ok(environment);
    }

    let qt_bin = target_root
        .join("panta-deps")
        .join("qt")
        .join("staging")
        .join("bin");
    if !qt_bin.is_dir() {
        return Err(format!("托管 Qt 运行库目录不存在：{}", qt_bin.display()));
    }
    let qt_root = qt_bin
        .parent()
        .ok_or_else(|| format!("托管 Qt bin 目录没有安装根：{}", qt_bin.display()))?;
    let qt_qml = qt_root.join("qml");
    if !qt_qml.is_dir() {
        return Err(format!("托管 Qt QML 模块目录不存在：{}", qt_qml.display()));
    }
    let current_path = environment
        .iter()
        .find(|(key, _)| key == std::ffi::OsStr::new("PATH"))
        .map(|(_, value)| value.clone())
        .or_else(|| std::env::var_os("PATH"))
        .unwrap_or_default();
    let native_qml = native_dir.join("qml");
    let native_app = native_dir.join("app");
    let mut paths = vec![qt_bin, native_qml, native_app, native_dir.to_path_buf()];
    paths.extend(std::env::split_paths(&current_path));
    let path =
        std::env::join_paths(paths).map_err(|error| format!("拼接 native 测试 PATH：{error}"))?;
    if let Some((_, value)) = environment
        .iter_mut()
        .find(|(key, _)| key == std::ffi::OsStr::new("PATH"))
    {
        *value = path;
    } else {
        environment.push((std::ffi::OsString::from("PATH"), path));
    }
    let import_path = std::env::join_paths([qt_qml, native_dir.to_path_buf()])
        .map_err(|error| format!("拼接 native 测试 QML 导入路径：{error}"))?;
    for key in ["QML2_IMPORT_PATH", "QML_IMPORT_PATH"] {
        environment.push((std::ffi::OsString::from(key), import_path.clone()));
    }
    Ok(environment)
}

/// 官方 LLVM 不替代 Apple SDK；显式 sysroot 保证 CXX 和 CMake 使用同一套平台头文件。
pub fn macos_sdk() -> Result<Option<PathBuf>, String> {
    if !cfg!(target_os = "macos") {
        return Ok(None);
    }
    let output = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-path"])
        .output()
        .map_err(|e| format!("无法定位 Apple SDK：{e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    if !path.is_dir() {
        return Err(format!("Apple SDK 不存在：{}", path.display()));
    }
    Ok(Some(path))
}

/// OUT_DIR 可能包含显式 --target 的 triple 层；拒绝未验证的跨目标构建。
pub fn target_root(out: &Path) -> Result<PathBuf, String> {
    let host = std::env::var("HOST").map_err(|e| e.to_string())?;
    let target = std::env::var("TARGET").map_err(|e| e.to_string())?;
    if host != target {
        return Err(format!("尚未支持交叉编译：{host} -> {target}"));
    }
    let mut root = out.ancestors().nth(4).ok_or("无法解析 OUT_DIR")?;
    if root.file_name().is_some_and(|name| name == target.as_str()) {
        root = root.parent().ok_or("target 根缺失")?;
    }
    Ok(root.to_path_buf())
}

fn download(
    asset: &ToolAsset,
    destination: &Path,
    name: &str,
    version: &str,
) -> Result<(), String> {
    let mut command = Command::new("curl");
    command
        // 速度护栏针对 CI 实测的传输停滞：60 秒均值低于 1 KiB/s 即中止并
        // 随 --retry 重试。curl 的 --max-time 会在每次重试时重置，
        // 因而额外用父进程限制整个下载（包括重试）最多 30 分钟。
        .args([
            "-fSL",
            "--retry",
            "3",
            "--retry-delay",
            "5",
            "--connect-timeout",
            "30",
            "--speed-limit",
            "1024",
            "--speed-time",
            "60",
            "--max-time",
            "1800",
            "--create-dirs",
            "-o",
        ])
        .arg(destination)
        .arg(asset.url);
    run_tool_command(
        &mut command,
        &format!("下载 {name} {version}：{}", asset.url),
        Duration::from_secs(30 * 60),
    )
}

fn extract(
    tool: Tool,
    archive: &Path,
    destination: &Path,
    cmake: Option<&Path>,
) -> Result<(), String> {
    if cfg!(windows) && matches!(tool, Tool::Llvm) {
        // Windows 的 LLVM tar.xz 归档由系统 tar 解包极慢（CI 实测超过
        // 20 分钟）；官方 NSIS 安装包包含同一套 clang-cl/lld/format，
        // 支持静默安装和自定义目录，避免依赖 runner 上额外的 7-Zip。
        let mut command = Command::new(archive);
        command
            .arg("/S")
            .arg(format!("/D={}", destination.display()));
        return run_tool_command(
            &mut command,
            &format!("安装 LLVM {}：{}", tool.version(), archive.display()),
            Duration::from_secs(20 * 60),
        );
    }
    let mut command = match tool {
        // CMake 压缩包由平台自带 tar 解开（macOS/Windows 为 bsdtar，可直接
        // 读 zip；Linux 为 GNU tar，自动识别 gzip）。
        Tool::Cmake | Tool::Llvm | Tool::Uv => {
            let mut command = Command::new("tar");
            command.arg("-xf").arg(archive).arg("-C").arg(destination);
            command
        }
        Tool::Ninja => {
            let Some(cmake) = cmake else {
                return Err("内部错误：解包 Ninja 需要 cmake 路径".to_owned());
            };
            let mut command = Command::new(cmake);
            command
                .args(["-E", "tar", "xf"])
                .arg(archive)
                .current_dir(destination);
            command
        }
    };
    run_tool_command(
        &mut command,
        &format!("解包 {}：{}", tool.name(), archive.display()),
        Duration::from_secs(20 * 60),
    )
}

/// 保留工具实时输出，并限制整个子进程的墙钟时间；超时后终止并回收子进程。
fn run_tool_command(command: &mut Command, stage: &str, timeout: Duration) -> Result<(), String> {
    eprintln!("[panta-tools] {stage}：开始（上限 {timeout:?}）");
    let started = Instant::now();
    let mut child = command
        .stdin(Stdio::null())
        .spawn()
        .map_err(|error| format!("{stage}：无法启动 {:?}：{error}", command.get_program()))?;
    let mut next_progress = Duration::from_secs(30);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    eprintln!("[panta-tools] {stage}：完成（{:?}）", started.elapsed());
                    return Ok(());
                }
                return Err(format!("{stage}：失败（{status}）"));
            }
            Ok(None) => {}
            Err(error) => {
                if child.kill().is_ok() {
                    let _ = child.wait();
                }
                return Err(format!("{stage}：无法查询子进程状态：{error}"));
            }
        }
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            child.kill().map_err(|error| {
                format!(
                    "{stage}：超时（上限 {timeout:?}），无法终止 PID {}：{error}",
                    child.id()
                )
            })?;
            let status = child
                .wait()
                .map_err(|error| format!("{stage}：超时后无法回收子进程：{error}"))?;
            return Err(format!(
                "{stage}：超时（上限 {timeout:?}），子进程已终止并回收（{status}）"
            ));
        }
        if elapsed >= next_progress {
            eprintln!(
                "[panta-tools] {stage}：仍在运行（{elapsed:?}，PID {}）",
                child.id()
            );
            next_progress = elapsed + Duration::from_secs(30);
        }
        std::thread::sleep(Duration::from_millis(100).min(timeout - elapsed));
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
    use super::{
        cmake_asset, exe_name, find_cmake_binary, hex, llvm_asset, ninja_asset, sha256_file,
    };
    use super::{install_directory, python};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn tool_process_fixture() {
        match std::env::var("PANTA_TEST_TOOL_PROCESS").as_deref() {
            Ok("fail") => std::process::exit(23),
            Ok("stall") => std::thread::sleep(std::time::Duration::from_secs(60)),
            _ => {}
        }
    }

    #[test]
    fn tool_process_propagates_failure_and_terminates_stalls() -> Result<(), String> {
        use std::process::Command;
        use std::time::{Duration, Instant};
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let command = |mode| {
            let mut command = Command::new(&executable);
            command
                .args(["--exact", "tests::tool_process_fixture"])
                .env("PANTA_TEST_TOOL_PROCESS", mode);
            command
        };
        super::run_tool_command(&mut command("success"), "fixture", Duration::from_secs(10))?;
        let error = super::run_tool_command(
            &mut command("fail"),
            "fixture failure",
            Duration::from_secs(10),
        )
        .err()
        .ok_or("非零退出应失败")?;
        assert!(
            error.contains("fixture failure") && error.contains("23"),
            "{error}"
        );
        let started = Instant::now();
        let error = super::run_tool_command(
            &mut command("stall"),
            "fixture timeout",
            Duration::from_millis(300),
        )
        .err()
        .ok_or("停滞进程应超时")?;
        assert!(
            error.contains("fixture timeout") && error.contains("已终止并回收"),
            "{error}"
        );
        assert!(started.elapsed() < Duration::from_secs(10));
        let error = super::run_tool_command(
            &mut Command::new(executable.join("missing")),
            "fixture missing",
            Duration::from_secs(10),
        )
        .err()
        .ok_or("缺少可执行文件应失败")?;
        assert!(
            error.contains("fixture missing") && error.contains("无法启动"),
            "{error}"
        );
        Ok(())
    }

    #[test]
    fn hex_encodes_lowercase_fixed_width() {
        assert_eq!(hex(&[]), "");
        assert_eq!(hex(&[0x0f, 0xa0]), "0fa0");
        assert_eq!(hex(&[255; 32]).len(), 64);
    }

    #[test]
    fn host_platform_has_well_formed_assets() {
        for asset in [
            cmake_asset(),
            ninja_asset(),
            llvm_asset(),
            python::uv_asset(),
        ] {
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
    #[test]
    fn concurrent_installers_publish_once_and_failed_upgrade_preserves_previous()
    -> Result<(), String> {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let root = std::env::temp_dir().join(format!("panta-lock-test-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
        }
        let count = Arc::new(AtomicUsize::new(0));
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let root = root.clone();
                let count = count.clone();
                std::thread::spawn(move || {
                    install_directory(&root, "fixture", "v1", |staging| {
                        count.fetch_add(1, Ordering::SeqCst);
                        fs::write(staging.join("binary"), "v1").map_err(|e| e.to_string())
                    })
                })
            })
            .collect();
        for worker in workers {
            worker.join().map_err(|_| "安装线程 panic")??;
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(
            install_directory(&root, "fixture", "v2", |staging| {
                fs::write(staging.join("partial"), "incomplete").map_err(|e| e.to_string())?;
                Err("模拟安装中断".into())
            })
            .is_err()
        );
        let original = root.join("panta-tools/fixture/v1/binary");
        assert_eq!(
            fs::read_to_string(&original).map_err(|e| e.to_string())?,
            "v1"
        );
        assert!(!root.join("panta-tools/fixture/v2").exists());
        let upgraded = install_directory(&root, "fixture", "v2", |staging| {
            assert!(!staging.join("partial").exists());
            fs::write(staging.join("binary"), "v2").map_err(|e| e.to_string())
        })?;
        assert_eq!(
            fs::read_to_string(upgraded.join("binary")).map_err(|e| e.to_string())?,
            "v2"
        );
        install_directory(&root, "fixture", "v1", |_| Err("旧版本不应重新安装".into()))?;
        fs::remove_dir_all(root).map_err(|e| e.to_string())
    }
    #[test]
    fn crash_during_install() -> Result<(), String> {
        if let Some(root) = std::env::var_os("PANTA_TEST_INSTALL_CRASH") {
            install_directory(
                std::path::Path::new(&root),
                "crash-fixture",
                "v1",
                |staging| {
                    fs::write(staging.join("partial"), "partial").map_err(|e| e.to_string())?;
                    // 不执行析构函数，模拟进程中断；OS 必须释放安装锁。
                    std::process::exit(23);
                },
            )?;
        }
        Ok(())
    }

    #[test]
    fn process_exit_releases_install_lock_and_retry_discards_partial_files() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!("panta-crash-test-{}", std::process::id()));
        let status =
            std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .args(["--exact", "tests::crash_during_install"])
                .env("PANTA_TEST_INSTALL_CRASH", &root)
                .status()
                .map_err(|e| e.to_string())?;
        assert_eq!(status.code(), Some(23));
        let directory = install_directory(&root, "crash-fixture", "v1", |staging| {
            assert!(!staging.join("partial").exists());
            fs::write(staging.join("binary"), "complete").map_err(|e| e.to_string())
        })?;
        assert!(directory.join("binary").is_file());
        fs::remove_dir_all(root).map_err(|e| e.to_string())
    }
}
