# 依赖获取与主平台环境

更新日期：2026-10-20。本文维护依赖供给规则和配置入口；技术选择见[版本基线](baseline.md)，构建职责与命令见[构建说明](../architecture/build-and-development.md)，实施证据见责任任务。

## 托管原则

- Cargo 引导将固定预编译包安装到 `target/panta-tools/`、`target/panta-deps/`；普通构建不编译第三方源码，不写源码树，也不隐式回退到系统包、Python wheel 或源码。需要新平台制品时，由独立 SDK 管线生产。
- 资产必须匹配平台、架构、编译器 / 运行库 ABI、Qt 兼容范围及模块；缺包、摘要或 target 不匹配立即失败并给出诊断。个人安装的同名库不构成项目基线。
- 源码固定 tag 和 commit，归档固定 URL 与 SHA256；尽量选择相容的稳定版。升级同一提交更新实际清单、相关说明和任务，记录来源、构建选项、许可证、验证与回退；受依赖约束不能升级时记录重评条件。

## 主验证平台与宿主前置

任务 002 的初始验证环境为 macOS 26.3.1 arm64、Apple clang 17.0.0 / macOS 26.2 SDK / libc++。当前自有 C++ 使用托管 LLVM；历史主平台记录不等同于发布支持范围。

| 平台 | 开发者安装的宿主能力 |
|---|---|
| 全部 | git、rustup；Rust 版本和组件由根 [rust-toolchain.toml](../../rust-toolchain.toml) 固定。首次供给需联网，完整依赖缓存可供离线构建。 |
| macOS | Apple 命令行工具、SDK 与系统运行库；独立 Qt 源码生产还需系统 Perl。 |
| Linux | glibc/sysroot、Rust 链接用 C 编译器、OpenGL 开发文件；Qt Gui 的 WrapOpenGL 在 configure 时需要 GL。QML 运行另需 EGL、xkbcommon 和软件渲染驱动，Ubuntu 对应 `libgl-dev`、`libegl1`、`libxkbcommon0`、`libgl1-mesa-dri`。 |
| Windows | MSVC Build Tools、Windows SDK 与 CRT；项目编译器为托管 clang-cl。 |

仅显式设置 `PANTA_USE_SYSTEM_TOOLS=1` 才读取 `CMAKE`、`CC`、`CXX` 或 PATH 中的本机工具。Linux aarch64 缺少固定工具资产时需走此旁路；它只解决工具选择，不保证 Qt / native SDK 对该平台的完整支持。

## 固定清单与验证入口

下表提供检索入口；精确版本、逐平台 URL / SHA256 由链接的配置维护，避免复制后漂移。技术版本摘要见[基线](baseline.md)。下载校验、configure、链接和真实运行是不同证据，不以安装成功代替验收。

| 依赖 | 权威配置 / 制品说明 | 消费边界与验证任务 |
|---|---|---|
| CMake / Ninja / LLVM | [assets.rs](../../crates/panta-build/src/assets.rs) | 官方预编译工具；Unix 用 clang/clang++，Windows 用 clang-cl；格式与 LLVM 检查使用同一工具链。任务 020 / 042。 |
| Qt | [qt-provision.cmake](../../native/cmake/qt-provision.cmake) | 官方 qtbase / qtdeclarative / qttools / qtsvg 与 qtscxml 包；Core / Gui / Qml / Quick / QuickControls2 / Test / StateMachine、固定 lrelease 及 qsvg 插件。StateMachine 仅在启用 bridge 时查找和链接。任务 005 / 029 / 034 / 074。 |
| Linux ICU | 同 Qt 供给配置 | ICU 73.2 与 Qt Linux 工具 ABI 匹配，解包到 `qt/staging/lib`；不替换为系统 ICU、不伪造 SONAME。任务 036。 |
| VTK / Dawn | [SDK manifest](../../native/cmake/sdk-provision.cmake)、[制品说明](../../tools/sdk/releases/vtk-9.7.0.md) | `find_package(VTK CONFIG)` / `Dawn`；`VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn`。Cocoa / Wayland / Win32 hardware window；Linux 制品由 Ubuntu 24.04 / GCC 13 生产、Wayland-only。任务 031 / 038 管供给，007 管窗口。 |
| OCCT | [SDK manifest](../../native/cmake/sdk-provision.cmake)、[配对制品说明](../../tools/sdk/releases/occt-netgen-8.0.1-6.2.2604.md) | `find_package(OpenCASCADE CONFIG)`；无命名空间的 `TKernel` / `TKDESTEP` 等 targets。共享库、关闭 Draw / Visualization / DETools / FreeType / Xlib，保留 STEP；进一步裁剪由 009 决定。Windows 运行库可能位于嵌套 `win64/vc14/bin/`。 |
| Netgen | 同 OCCT 配对 manifest / 制品说明 | 必须以大写 N 调用 `find_package(Netgen CONFIG)`；targets 为 `ngcore` / `nglib`，`USE_OCC=ON`；GUI / Python / MPI / JPEG / MPEG 关闭。网格正确性见 010。 |
| GoogleTest | [SDK manifest](../../native/cmake/sdk-provision.cmake)、[制品说明](../../tools/sdk/releases/googletest-1.18.0.md) | 仅 `BUILD_TESTING=ON` 获取静态 SDK；`find_package(GTest CONFIG)`、`GTest::gtest` / `GTest::gtest_main`。上游 CMake 安装、C++17、关闭 GMock，Windows Release `/MD`。任务 019 / 070 / 071。 |

Qt 缓存为 `target/panta-deps/qt/staging`，归档指纹位于 `qt/extracted`，跨 profile 共享。官方源失败或摘要不符时可尝试 TUNA 镜像；`PANTA_QT_MIRROR` 可插入同构镜像，所有来源均强校验。SDK 使用 `panta_require_sdk`，在 `target/panta-deps/sdk/<name>/<version>/<triple>` 缓存，并用受限的 `find_package(... CONFIG ... NO_DEFAULT_PATH)` 消费。

许可证随发布包保留：CMake / VTK / GoogleTest 为 BSD 系列，Ninja 为 Apache-2.0，LLVM 带 LLVM exceptions，Qt 为 LGPL-3.0，OCCT 为 LGPL-2.1 加例外，Netgen 为 LGPL-2.1，Dawn / ICU 各按上游许可证。具体模块及再分发要求以各归档的 license / provenance 和制品说明为准，不将仓库许可证套用于依赖。

## 依赖间版本关系

- **Netgen ↔ OCCT**：Netgen 只与构建时链接的 OCCT ABI 配对；038 使用同工具链生产并作为一个 Release 成对发布，任何一侧升级都须成对重建。历史候选回退点为 OCCT `V7_9_3`，不能单独替换现有配对中的 OCCT；实际构建与几何到网格证据见 009 / 010。
- **VTK ↔ Qt**：WebGPU SDK 不依赖 Qt VTK GUI 模块；Qt Quick 承载界面，平台桥接管理 native view / layer / surface。窗口、事件及运行时加载由 007 验证，保留 WebGPU 与 Qt Quick 叠加的实机验收边界。
- **Rust / CXX**：固定编译器与 MSRV 的区别、升级规则见 [Rust 规范](rust.md)；Rust 侧版本不替代 native ABI 验证。

## CI 与干净重建

[ci.yml](../../.github/workflows/ci.yml) 在三平台构建并按影响范围执行测试及质量检查；[nightly.yml](../../.github/workflows/nightly.yml) 承担完整测试、覆盖率和长时检查。纯文档规则与代码目录内 Markdown 夹具的例外以 workflow 为准；选择器与日常验证见 AGENTS.md。CI 的 Linux 宿主前置由独立 action 安装，不属于项目 SDK；工具、Qt 与 SDK 仍经 Cargo 入口供给。

干净检出后安装上述宿主前置，在仓库内由 rustup 按工具链文件补齐组件，再执行 `cargo build --locked`、`cargo run --locked`。构建目录、无桥接诊断和安装命令统一见[构建说明](../architecture/build-and-development.md)。回退时恢复上一版配套配置并重建；删除构建树不影响源码。跨平台窗口及部署限制分别记录在 007 / 013。

## 质量工具供给与缓存

`panta-build` 的 [assets.rs](../../crates/panta-build/src/assets.rs) 与 [install.rs](../../crates/panta-build/src/install.rs) 统一固定资产、校验、加锁及安装。uv / hyperfine 与 LLVM 等工具按版本和摘要隔离在 `target/panta-tools/`；`cargo hyperfine` 按需安装，命令见[性能模块](../modules/performance.md)。Python 解释器版本见 [python.rs](../../crates/panta-build/src/python.rs)，工具锁见 [pyproject.toml](../../pyproject.toml) / [uv.lock](../../uv.lock)，禁止源码安装回退。Cppcheck 是仅开发 / CI 使用的 GPL-3.0 工具，不链接进应用；工具及解释器许可证沿分发包保留。质量入口见[质量工具链](../modules/quality-tooling.md)。

CI 仅缓存 Cargo registry/git 和可校验的工具 / SDK 资产；不缓存 `target/native` 或 Cargo 编译产物，避免旧配置、编译数据库和对象泄漏。键包含平台、架构、工具版本与供给清单；回退命中仍需目录内版本及摘要校验，Windows LLVM 安装目录同样适用。

工具下载由父进程限制含重试的总耗时 30 分钟，解包 / Windows 安装上限 20 分钟；超时回收直接子进程，不发布 `.complete`，临时目录留待下次持锁清理。安装各阶段有日志，长操作每 30 秒报告耗时。不能只用 curl `--max-time` 代替总预算：每次重试会重置该计时，`--retry-max-time` 也不截断在途传输。依据见 [curl 手册](https://curl.se/docs/manpage.html#--max-time)，实跑诊断见任务 [044](../task/044-windows-ci.md)。
