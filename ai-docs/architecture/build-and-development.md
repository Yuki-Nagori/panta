# 构建、开发与部署

[架构总览](README.md)

## 当前可用范围

Rust workspace 骨架已落地（任务 001）：根 [Cargo.toml](../../Cargo.toml)（edition 2024、resolver 3、默认成员 `panta-launcher`）、launcher、DSL 和 FFI 成员与 [rust-toolchain.toml](../../rust-toolchain.toml) 固定的 stable 1.98.1。`cargo build --locked`、`cargo test --locked --workspace`、`cargo fmt --all -- --check` 可运行。

native 构建骨架已落地（任务 003）：[native/](../../native/CMakeLists.txt) 顶层 CMakeLists、`panta_foundation` 库、`native/app` 可执行骨架与 CTest 测试；单配置 Ninja presets（`debug`/`release`），安装树可被 `find_package(panta-native)` 消费。

Cargo 调度已接通（任务 004）：Cargo manifest 通过 panta-ffi 的普通依赖 + build-dependency 双边确保 staticlib 先于 launcher build script 生成，`crates/launcher/build.rs` 再以与 presets 一致的有效配置构建 native 树（任务 041：构建树固定为 `target/native/<profile>`，presets 与 Cargo 共用；Qt 与各平台 SDK 缓存在 `target/panta-deps/`，工具在 `target/panta-tools/`，均与 launcher 哈希目录无关）；构建完成后 `cargo run` 启动 native 产物并转发参数与退出码（未知参数 64、产物缺失 69、信号终止 128+信号）。

QML 模块可裁剪：`cargo build --locked` 默认启用 `bridge-module` feature，构建并静态注册 `Panta.Bridge`；诊断时使用 `cargo build --locked --no-default-features`，由 launcher 将 feature 状态映射为 `PANTA_ENABLE_BRIDGE_MODULE=OFF`，移除该模块并使用 `Panta.Shell` 的 `AppNoBridge` 最小入口。

Qt Quick 主窗口已可用（任务 005）：`native/app` 为 Qt 入口，`native/bridge` 提供 ViewModel（GTest 信号测试），`qml/`（URI `Panta.Shell`，静态 QML 模块及 `panta_shellplugin`）承载界面；应用和测试显式链接 Shell 插件以保留字体等 C++ 单例的类型注册；Qt 6.11.2 预编译包由 `native/cmake/qt-provision.cmake` 按三平台固定清单下载到构建树。任务 006 的 `panta-ffi` 以 CXX 1.0.202 生成 Rust/C++ 桥接静态库和 native 边界测试，任务 047 的 `panta-foundation` 由该边界暴露进程级崩溃安装入口，launcher build.rs 将生成头与静态库路径传入 CMake；该最小路径已由三平台 CI 干净构建覆盖。QML/资源目录已纳入 build.rs 重建追踪（改 QML 即重跑 qmlcachegen）。`prototypes/shell/shell.js` 是图标 symbol 的正式来源，并由 QML 组件测试打包为资源；原型目录变更须纳入构建重建及测试/质量影响范围。i18n 编译链已接通（任务 034）：`resources/i18n/*.pa` 经 `panta-dsl-core` 生成构建树 TS，`native/i18n` 用 Qt 供给的锁定 `lrelease` 编 QM 并嵌入 `:/i18n/`（ctest 覆盖加载）；运行期语言切换由 022 接入。VTK 的 007 WebGPU/platform-surface 模块、适配器和创建级测试已接入构建；macOS Cocoa/Metal 与 Windows 已有真实窗口记录，但 Linux Wayland 真实窗口、跨显示器 DPR 和任务 007 列出的剩余场景仍待验收，007 当前为 deferred，不能把创建级测试视为完整窗口验收。

构建图与扩展点：Cargo → launcher 的 build.rs → CMake/Ninja → native targets，单向无环；CMake 侧不回调 Cargo。重建追踪覆盖 native 源/配置、QML、正式资源与测试使用的原型输入。Rust 库供 C++ 消费的接入点在 CMake 侧，已由任务 006 接通。

工程新建、打开、保存、STL 预检/导入、Mesh sidecar 旁置存储和按需激活已实现（任务 063/088）；当前工程清单为 schema 4，来源 STL 仅保留追溯元数据，重开从 Mesh sidecar 加载。`panta-cli` 已复用 GUI 的 Rust 应用命令提供工程检查/创建、STL 检查/导入（121–124）；`ProjectService` 已持有 Rust 运行时上下文，提供不可变变量快照、工程内只读资源解析及协作取消作用域（024）。完整 CAE 流程仍未交付：STEP 导入到工程、实际 Create Mesh、Study 与真实求解器接入仍属后续工作。桌面安装部署（013）已具备 CMake 安装规则：macOS 生成 `.app` 并使用 Qt 部署脚本收集 QML imports 和 Qt 插件；CMake runtime dependency set 收集应用实际链接的 SDK 动态库，许可、制品来源和 Qt SBOM 随安装树保存。macOS arm64 安装包已在清理开发环境变量后启动并检查真实窗口；Windows/Linux 的安装树和真实窗口仍待各平台验收，不能据此标记为受支持。当前 app 只链接 VTK；OCCT 和 Netgen 仍由几何/网格测试目标消费，没有被桌面 app 加载。工具二进制供给（020/042）已落地：受支持平台由构建引导按固定资产下载校验 CMake、Ninja 与 LLVM 22.1.7 到根 `target/panta-tools/`，CMake、Cargo CXX、clang-format 和 clang-tidy 复用同一套 LLVM；只有显式设置 `PANTA_USE_SYSTEM_TOOLS=1` 才读取 `CMAKE`、`CC`、`CXX` 或 PATH 的本机工具。Linux aarch64 无官方固定资产时需走该旁路。VTK/OCCT/Netgen 的 SDK 供给模块（031 `native/cmake/sdk-provision.cmake`）已落地并经 ctest `Build.SdkProvision` 验证，三者均有三平台固定资产；当前 app 只链接 VTK，OCCT 和 Netgen 由几何/网格目标消费。以下命令是现有构建/安装入口；跨平台安装尚未全部实测。

## 构建职责

Cargo 提供统一开发入口，负责 Rust workspace 和原生构建调度；CMake 拥有 C++、Qt、QML 和 native desktop executable 的构建图；Ninja 是当前原生构建执行器。

Qt 桌面可执行文件由 CMake 生成，使 moc、rcc、QML 模块处理和部署处在同一构建图中。应用 QML 入口在 `Panta.App` 模块中包装可复用的 `Panta.Shell` 页面，让 Qt 部署扫描从可执行目标解析传递的 QML imports。Qt 版本锁定为 6.11.2；`qt_generate_deploy_qml_app_script()` 已接入 install 规则。

`cmake` crate 可用于 Rust package 的 `build.rs` 构建原生库，但这不自动实现整个桌面程序的运行与部署。不能把示意的 `cmake::Config::new("native").build()` 当成完整工程方案：还需处理安装产物、链接目录、运行时动态库和增量构建。

## Cargo 命令

| 命令 | 当前行为 |
|---|---|
| `cargo build --locked` | 构建 FFI staticlib、Rust workspace 和 native 桌面程序。 |
| `cargo run --locked` | 启动 CMake 生成的桌面程序，并转发参数与退出码。 |
| `cargo panta-package` | 通过 launcher 调用托管 CMake 的 `--install --component Runtime`，默认输出到 `target/package/<profile>`。可在命令后附加自定义安装前缀。 |
| `cargo test --locked --workspace` | 运行 Rust 测试及已聚合的 native/QML 测试。 |

Cargo 自带的 `cargo package` 保留给 Rust crate 打包；桌面安装命令使用 `cargo panta-package` alias。该命令只安装应用运行时文件，不附带 native 开发用静态库、头文件和 CMake 导出；它生成可搬迁安装目录，不负责生成平台安装器、签名或公证。

避免构建递归：Cargo 调 CMake 时，CMake 不应再次无条件调用同一 Cargo target。Rust 库产物与 native target 的依赖顺序应显式描述。

## 工具链与依赖准备

主验证平台为 macOS arm64（Apple clang 17 + CLT，2026-09-16 记录）；获取方式为 Cargo 统一托管：开发者只安装 git、rustup 与 Apple 命令行工具，`cargo build` 触发的构建引导按固定清单供给 CMake、Ninja、Qt 6、VTK、OCCT、Netgen（[依赖获取与主平台环境](../standards/dependency-acquisition.md)）。三平台构建由 CI 覆盖，安装树与真实窗口的验证边界见本页“当前可用范围”；构建通过不代表各目标平台的安装与窗口验收完成。

首次集成时记录：构建配置、Qt 图形后端、实测依赖版本与校验信息。第三方依赖不必自身以 C++20 编写，但头文件、ABI、运行库和链接方式必须与应用兼容。

依赖路径应通过公开的构建配置传入，不硬编码个人机器路径。分别管理 Debug/Release 产物，追踪源文件与配置变动，验证连续构建和清理后构建的一致性。

## 安装与部署

在仓库根目录构建并安装到独立目录：

```sh
cargo build --locked
cmake --install target/native/debug --prefix /tmp/panta-install
```

macOS 的应用位于 `/tmp/panta-install/panta.app`；Windows/Linux 的入口为安装前缀下 `bin/panta[.exe]`。macOS arm64 可通过 `open /tmp/panta-install/panta.app` 启动；无开发环境变量的检查可用 `env -i PATH=/usr/bin:/bin HOME="$HOME" /tmp/panta-install/panta.app/Contents/MacOS/panta`。真实窗口通过 CUA 检查，不以成功构建或进程启动替代窗口验收。Windows/Linux 目标机和安装树仍需独立验证。

Qt 的运行期 QML imports、平台插件和所需 Qt 动态库由 `qt_generate_deploy_qml_app_script()` 安装；应用实际链接的 VTK 等 native 动态依赖由 CMake runtime dependency set 收集。macOS 许可、SDK provenance 和 Qt SBOM 位于 `.app/Contents/Resources/{licenses,sdk,sbom}`；其他平台位于安装前缀 `share/panta/`。安装依赖集只包含 app 的链接闭包：当前几何/网格模块由测试目标消费，OCCT/Netgen 不属于桌面 app 的运行时闭包。许可证标识与版本基线见[依赖获取规范](../standards/dependency-acquisition.md)，发布前仍须结合实际打包内容复核各上游再分发要求。
