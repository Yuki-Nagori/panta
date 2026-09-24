# 构建、开发与部署

[架构总览](README.md)

## 当前可用范围

Rust workspace 骨架已落地（任务 001）：根 [Cargo.toml](../../Cargo.toml)（edition 2024、resolver 3、默认成员 `panta-launcher`）、launcher、DSL 和 FFI 成员与 [rust-toolchain.toml](../../rust-toolchain.toml) 固定的 stable 1.98.1。`cargo build --locked`、`cargo test --locked --workspace`、`cargo fmt --all -- --check` 可运行。

native 构建骨架已落地（任务 003）：[native/](../../native/CMakeLists.txt) 顶层 CMakeLists、`panta_foundation` 库、`native/app` 可执行骨架与 CTest 测试；单配置 Ninja presets（`debug`/`release`），安装树可被 `find_package(panta-native)` 消费。

Cargo 调度已接通（任务 004）：Cargo manifest 通过 panta-ffi 的普通依赖 + build-dependency 双边确保 staticlib 先于 launcher build script 生成，`crates/launcher/build.rs` 再以与 presets 一致的有效配置构建 native 树（任务 041：构建树固定为 `target/native/<profile>`，presets 与 Cargo 共用；Qt 与各平台 SDK 缓存在 `target/panta-deps/`，工具在 `target/panta-tools/`，均与 launcher 哈希目录无关）；构建完成后 `cargo run` 启动 native 产物并转发参数与退出码（未知参数 64、产物缺失 69、信号终止 128+信号）。

QML 模块可裁剪：`cargo build --locked` 默认启用 `bridge-module` feature，构建并静态注册 `Panta.Bridge`；诊断时使用 `cargo build --locked --no-default-features`，由 launcher 将 feature 状态映射为 `PANTA_ENABLE_BRIDGE_MODULE=OFF`，移除该模块并使用 `Panta.Shell` 的 `AppNoBridge` 最小入口。

Qt Quick 主窗口已可用（任务 005）：`native/app` 为 Qt 入口，`native/bridge` 提供 ViewModel（GTest 信号测试），`qml/`（URI `Panta.Shell`，NO_PLUGIN 资源模块）承载界面；Qt 6.11.2 预编译包由 `native/cmake/qt-provision.cmake` 按三平台固定清单下载到构建树。任务 006 的 `panta-ffi` 以 CXX 1.0.202 生成 Rust/C++ 桥接静态库和 native 边界测试，任务 047 的 `panta-foundation` 由该边界暴露进程级崩溃安装入口，launcher build.rs 将生成头与静态库路径传入 CMake；该最小路径已由三平台 CI 干净构建覆盖。QML/资源目录已纳入 build.rs 重建追踪（改 QML 即重跑 qmlcachegen）。i18n 编译链已接通（任务 034）：`resources/i18n/*.pa` 经 `panta-dsl-core` 生成构建树 TS，`native/i18n` 用 Qt 供给的锁定 `lrelease` 编 QM 并嵌入 `:/i18n/`（ctest 覆盖加载）；运行期语言切换由 022 接入。VTK 的 007 模块、适配器和创建级测试已接入构建；macOS 26 窗口化渲染仍 blocked，默认界面保留占位面板，不宣称 VTK 桌面视口已交付。

构建图与扩展点：Cargo → launcher 的 build.rs → CMake/Ninja → native targets，单向无环；CMake 侧不回调 Cargo。重建追踪显式列举 native 源/配置与 qml/ 目录；`resources/` 落地时追加（qt_add_resources 扩展点已在 qml/CMakeLists.txt 标注）。Rust 库供 C++ 消费的接入点在 CMake 侧，由任务 006 确定。

仍不能完成 CAE 业务流程：几何导入、网格、渲染与持久化均为后续任务；桌面分发（013）未实施。工具二进制供给（020/042）已落地：受支持平台由构建引导按固定资产下载校验 CMake、Ninja 与 LLVM 22.1.7 到根 `target/panta-tools/`，CMake、Cargo CXX、clang-format 和 clang-tidy 复用同一套 LLVM；只有显式设置 `PANTA_USE_SYSTEM_TOOLS=1` 才读取 `CMAKE`、`CC`、`CXX` 或 PATH 的本机工具。Linux aarch64 无官方固定资产时需走该旁路。VTK/OCCT/Netgen 的 SDK 供给模块（031 `native/cmake/sdk-provision.cmake`）已落地并经 ctest `Build.SdkProvision` 验证，缓存于 `target/panta-deps/sdk/`；当前仅 OCCT Windows 有固定资产，消费方任务接入前不触发下载。下面是构建契约与实施要求，不是已验证的安装教程。

## 构建职责

Cargo 提供统一开发入口，负责 Rust workspace 和原生构建调度；CMake 拥有 C++、Qt、QML 和 native desktop executable 的构建图；Ninja 作为计划采用的原生构建执行器。

Qt 桌面可执行文件由 CMake 生成，使 moc、rcc、QML 模块处理和部署处在同一构建图中。计划使用 `qt_add_qml_module()` 组织 QML；具体 Qt 版本和部署 API 的可用性在锁定版本后验证。

`cmake` crate 可用于 Rust package 的 `build.rs` 构建原生库，但这不自动实现整个桌面程序的运行与部署。不能把示意的 `cmake::Config::new("native").build()` 当成完整工程方案：还需处理安装产物、链接目录、运行时动态库和增量构建。

## 目标命令与落地条件

| 目标入口 | 预期行为 | 必须补齐的实现 |
|---|---|---|
| `cargo build` | 由 Cargo 依赖图先构建 FFI staticlib，再构建 Rust 与 native desktop | 编排入口、原生调度、依赖发现、失败码传递 |
| `cargo run` | 启动 CMake 生成的桌面程序 | Cargo 可运行 launcher、可执行文件定位、参数转发 |
| `cargo test --workspace` | 提供统一验证入口 | Rust 测试及 native/集成检查的明确调度 |

这些是目标用户体验。Cargo 默认只理解已配置的 Rust targets，不会自动运行 CTest、QML 检查或 CMake executable。若采用 launcher 或专门的调度 crate，需记录它与桌面进程之间的退出码、信号和工作目录约定。

避免构建递归：Cargo 调 CMake 时，CMake 不应再次无条件调用同一 Cargo target。Rust 库产物与 native target 的依赖顺序应显式描述。

## 工具链与依赖准备

主验证平台为 macOS arm64（Apple clang 17 + CLT，2026-09-16 记录）；获取方式为 Cargo 统一托管：开发者只安装 git、rustup 与 Apple 命令行工具，`cargo build` 触发的构建引导按固定清单供给 CMake、Ninja、Qt 6、VTK、OCCT、Netgen（[依赖获取与主平台环境](../standards/dependency-acquisition.md)）。不承诺跨平台矩阵；引导实装前的当前可用范围见本页开头。

首次集成时记录：构建配置、Qt 图形后端、实测依赖版本与校验信息。第三方依赖不必自身以 C++20 编写，但头文件、ABI、运行库和链接方式必须与应用兼容。

依赖路径应通过公开的构建配置传入，不硬编码个人机器路径。分别管理 Debug/Release 产物，追踪源文件与配置变动，验证连续构建和清理后构建的一致性。

## 打包与发布

开发机能启动不等于可分发。部署需包含 Qt plugins、QML modules、VTK/OCCT/Netgen 动态库和应用资源，并在无开发环境的目标机器上验证。`qt_generate_deploy_qml_app_script()` 是可评估的 Qt 部署机制，适用版本和平台需要确认。

发行前核对依赖许可证与再分发要求，记录资源定位、日志目录和用户数据目录。当前仓库只确定自身采用 Apache-2.0；不据此推断所有第三方依赖的许可证。
