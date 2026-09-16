# 构建、开发与部署

[架构总览](README.md)

## 当前可用范围

Rust workspace 骨架已落地（任务 001）：根 [Cargo.toml](../../Cargo.toml)（edition 2024、resolver 3）、唯一成员 `crates/launcher` 与 [rust-toolchain.toml](../../rust-toolchain.toml) 固定的 stable 1.98.1。`cargo build --locked`、`cargo test --locked`、`cargo fmt --all -- --check` 可运行；`cargo run` 只输出"桌面尚未接入"诊断并以退出码 69 结束，不会启动 GUI。

仍不能构建或启动桌面应用：仓库尚无 `build.rs`、`native/CMakeLists.txt` 或 CI，原生调度由任务 003/004 接入。下面是构建契约与实施要求，不是已验证的安装教程。

## 构建职责

Cargo 提供统一开发入口，负责 Rust workspace 和原生构建调度；CMake 拥有 C++、Qt、QML 和 native desktop executable 的构建图；Ninja 作为计划采用的原生构建执行器。

Qt 桌面可执行文件由 CMake 生成，使 moc、rcc、QML 模块处理和部署处在同一构建图中。计划使用 `qt_add_qml_module()` 组织 QML；具体 Qt 版本和部署 API 的可用性在锁定版本后验证。

`cmake` crate 可用于 Rust package 的 `build.rs` 构建原生库，但这不自动实现整个桌面程序的运行与部署。不能把示意的 `cmake::Config::new("native").build()` 当成完整工程方案：还需处理安装产物、链接目录、运行时动态库和增量构建。

## 目标命令与落地条件

| 目标入口 | 预期行为 | 必须补齐的实现 |
|---|---|---|
| `cargo build` | 构建 Rust 与 native desktop | 原生调度、依赖发现、失败码传递 |
| `cargo run` | 启动 CMake 生成的桌面程序 | Cargo 可运行 launcher、可执行文件定位、参数转发 |
| `cargo test` | 提供统一验证入口 | Rust 测试及 native/集成检查的明确调度 |

这些是目标用户体验。Cargo 默认只理解已配置的 Rust targets，不会自动运行 CTest、QML 检查或 CMake executable。若采用 launcher 或专门的调度 crate，需记录它与桌面进程之间的退出码、信号和工作目录约定。

避免构建递归：Cargo 调 CMake 时，CMake 不应再次无条件调用同一 Cargo target。Rust 库产物与 native target 的依赖顺序应显式描述。

## 工具链与依赖准备

主验证平台为 macOS arm64（Apple clang 17 + CLT，2026-09-16 记录）；获取方式为 Cargo 统一托管：开发者只安装 git、rustup 与 Apple 命令行工具，`cargo build` 触发的构建引导按固定清单供给 CMake、Ninja、Qt 6、VTK、OCCT、Netgen（[依赖获取与主平台环境](../standards/dependency-acquisition.md)）。不承诺跨平台矩阵；引导实装前的当前可用范围见本页开头。

首次集成时记录：构建配置、Qt 图形后端、实测依赖版本与校验信息。第三方依赖不必自身以 C++20 编写，但头文件、ABI、运行库和链接方式必须与应用兼容。

依赖路径应通过公开的构建配置传入，不硬编码个人机器路径。分别管理 Debug/Release 产物，追踪源文件与配置变动，验证连续构建和清理后构建的一致性。

## 打包与发布

开发机能启动不等于可分发。部署需包含 Qt plugins、QML modules、VTK/OCCT/Netgen 动态库和应用资源，并在无开发环境的目标机器上验证。`qt_generate_deploy_qml_app_script()` 是可评估的 Qt 部署机制，适用版本和平台需要确认。

发行前核对依赖许可证与再分发要求，记录资源定位、日志目录和用户数据目录。当前仓库只确定自身采用 Apache-2.0；不据此推断所有第三方依赖的许可证。
