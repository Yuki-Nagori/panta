# panta

[![CI](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

> **命名**：Panta 取自赫拉克利特的名言 *Panta rhei*（希腊语 **πάντα ῥεῖ**，意为"万物皆流"）。注塑过程中，聚合物熔体、温度场、压力场与材料形态始终处于动态演化之中；CAE 的本质，正是对这一"流变万物"过程的数值再现。

Panta 是一个规划中的 CAE 桌面平台，使用 Qt Quick/QML 构建界面，以 OpenCASCADE、Netgen 和 VTK 提供几何、网格与可视化基础；物理求解通过外部进程接入。

**当前已交付：** Rust workspace、Cargo 驱动的 native 构建、Qt Quick/C++ ViewModel 桌面骨架，以及 `.pa` parser/formatter CLI。CAE 工程、导入、网格、求解和持久化流程仍在开发中。

## 快速开始

安装下表所列基础工具后，在仓库根目录运行：

```sh
cargo build --locked
cargo run --locked
```

Cargo 会按锁定清单自动获取 CMake、Ninja、Qt 和 native SDK 到 `target/`。首次构建需要网络访问。Rust 版本见 [rust-toolchain.toml](rust-toolchain.toml)，依赖来源、版本和平台限制见[依赖获取规范](ai-docs/standards/dependency-acquisition.md)。

## 系统要求

| 平台 | 需要预装 |
|---|---|
| macOS（arm64；目标基线 13.0，待 SDK 重产与验收） | git、[rustup](https://rustup.rs)、Apple 命令行工具（`xcode-select --install`） |
| Linux | git、[rustup](https://rustup.rs)、C 编译器及 OpenGL/Wayland 前置；Debian/Ubuntu 需要 `pkg-config`、`libgl-dev`、`libegl1`、`libxkbcommon0`、`libxkbcommon-dev`、`libwayland-dev`、`wayland-protocols`，运行 QML 另需 `libgl1-mesa-dri`。Qt 预编译包要求 glibc ≥ 2.34。 |
| Windows | git、[rustup](https://rustup.rs)、Visual Studio Build Tools 2022（MSVC） |

## 常用命令

| 命令 | 用途 |
|---|---|
| `cargo build --locked` | 构建默认成员 `panta-launcher` 及其依赖，并生成桌面程序 `panta`（Windows 为 `panta.exe`）。 |
| `cargo build --locked --workspace` | 构建 workspace 全部成员。 |
| `cargo run --locked` | 启动桌面窗口并转发程序参数与退出码。 |
| `cargo panta-package` | 生成当前 profile 的可搬迁安装目录，默认位于 `target/package/<profile>`。 |
| `cargo panta-package /path/to/output` | 将安装目录写到指定路径。macOS 输出 `panta.app`，Windows/Linux 输出 `bin/panta[.exe]`。 |
| `cargo test --locked --workspace` | 运行 Rust、native、QML 聚合测试。 |
| `cargo format` | 检查 Rust、C++、CMake 和 QML 格式。 |
| `cargo lint` | 运行 Clippy、依赖、CMake、QML、C++ 静态检查。 |
| `cargo quality` | 依次运行格式、lint、依赖审计和测试。 |
| `cargo coverage` | 运行 Rust 覆盖率门禁，CI 当前要求函数覆盖率 89%、行覆盖率 92%；`cargo coverage native` 生成 C++ 报告 artifact，当前不设阈值。门槛调整规划见任务 032（deferred）。 |
| `cargo performance` | 手动运行性能基准，不纳入常规测试门禁。 |
| `cargo run --locked --package panta-tests -- toolchain` | 检查 Cargo 托管的编译、格式和构建工具。 |
| `cargo run -p panta-dslc -- check resources/i18n/panta-cn.pa` | 校验 `.pa` 语言字典；`format --check` 检查格式。 |

`cargo panta-package` 通过 Qt 部署脚本收集 Qt/QML 运行时，并由 CMake 安装 app 实际链接的 native 动态依赖及许可清单；不会带上 native 开发用静态库、头文件和 CMake 导出。它生成安装目录，不生成 DMG、MSI 或 DEB 安装器，也不执行签名或公证。macOS arm64 安装目录已做独立启动和真实窗口检查；Windows/Linux 的安装目录和目标机窗口仍待验收。

GitHub Actions 的 **Package desktop builds** workflow 可手动触发，为 macOS arm64、Windows x86_64 和 Linux x86_64 分别构建测试包，并将归档作为保留 1 天的 workflow artifact 上传。下载后可转发给测试者；Linux 仍需满足[系统要求](#系统要求)。这些是便于测试的可搬迁目录，不是正式发行安装器。

Release 安装目录可用 `cargo run --locked --release --package panta-launcher -- package` 生成到 `target/package/release`。

Cargo alias 定义在 [.cargo/config.toml](.cargo/config.toml)。Cargo 自带的 `cargo package` 保持原有 Rust crate 打包语义，因此桌面安装命令使用独立名称 `cargo panta-package`。

## 构建与测试

Cargo 负责 Rust workspace 和 native 构建调度；CMake 管理 C++、Qt/QML 和桌面程序构建图。native 诊断命令也可直接在 `native/` 下运行：

```sh
cmake --preset debug
cmake --build --preset debug
ctest --preset debug
cmake --install ../target/native/debug --prefix /path/to/output
```

日常构建可按 Cargo 常规行为解析依赖；CI 和可复现验证使用 `--locked`。跨语言测试、质量门禁和任务约束见 [AGENTS.md](AGENTS.md)、[测试规范](ai-docs/standards/testing.md)和[任务索引](ai-docs/task-index.md)。

## 项目说明

项目采用 Apache-2.0，第三方依赖按各自许可分发，见[依赖获取规范](ai-docs/standards/dependency-acquisition.md)。

- [架构总览](ai-docs/architecture/README.md)
- [构建、开发与部署](ai-docs/architecture/build-and-development.md)
- [重要模块说明](ai-docs/modules/README.md)
- [实施任务索引](ai-docs/task-index.md)
- [MoldSolver 外部边界](ai-docs/architecture/external-moldsolver.md)
- [Mold Protocol 外部边界](ai-docs/architecture/external-mold-protocol.md)
