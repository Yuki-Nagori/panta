# 构建、开发与部署

[架构总览](README.md) · [常用命令](../../README.md#常用命令) · [依赖获取](../standards/dependency-acquisition.md)

更新日期：2026-10-20。本页说明构建职责与产物；产品能力和实施状态以 README、任务索引为准。

## 构建职责与产物

调用方向为 `Cargo → launcher build.rs → CMake / Ninja → native targets`，CMake 不回调 Cargo。Rust 工具链由根 [rust-toolchain.toml](../../rust-toolchain.toml) 锁定，workspace 默认成员为 `panta-launcher`；完整 Rust 构建需显式 `--workspace`。

| 内容 | 责任与路径 |
|---|---|
| Rust 与 CXX | Cargo 构建 Rust；`panta-ffi` 生成桥接头和 staticlib，launcher 将路径交给 CMake。普通依赖与 build-dependency 保证库先于 launcher 构建脚本就绪。 |
| Qt / QML / C++ | CMake 管理 native 编译、moc、rcc、qmlcachegen、静态 QML 插件和桌面可执行文件；Ninja 执行构建图。 |
| 构建目录 | `target/native/<profile>`，Cargo 与 presets 共用有效配置，Debug / Release 分开。 |
| 固定工具与 SDK | 工具位于 `target/panta-tools/`，Qt 与 native SDK 位于 `target/panta-deps/`；版本与摘要见[依赖规范](../standards/dependency-acquisition.md)。 |
| 增量重建 | 跟踪 native 源码/配置、QML、正式资源及测试使用的原型输入；`prototypes/shell/shell.js` 是图标 symbol 来源，不能因位于原型目录而跳过。 |

Qt 入口为 `native/app`；`Panta.App` 包装可复用的 `Panta.Shell`，使部署扫描能解析传递的 QML imports。应用和测试显式链接静态插件，保留类型注册。i18n 的 `.pa → TS → QM` 产物均在构建树生成，QM 随资源打包；运行期切换见任务 022。

默认启用 `bridge-module`。诊断时可用 `cargo build --locked --no-default-features`，launcher 映射为 `PANTA_ENABLE_BRIDGE_MODULE=OFF` 并使用 `AppNoBridge` 入口。Rust 库由 CMake target 消费的规则见[职责边界](native-domain-boundaries.md)，不以示例构建 API 代替真实安装、链接和运行库部署。

## Cargo 入口

| 命令 | 行为 |
|---|---|
| `cargo build --locked` | 构建默认桌面入口、FFI 及其 native 依赖；`--workspace` 扩大到全部 Rust 成员。 |
| `cargo run --locked` | 启动 native 桌面程序并转发参数、退出码及信号结果。 |
| `cargo verify-changed` | 日常变更范围内的格式、lint 和 L1/L2 回归；选择规则见 AGENTS.md。 |
| `cargo test --locked --workspace` | 完整 Rust / native / QML 聚合测试，用于无法缩小范围的验收和 nightly。 |
| `cargo panta-package [安装前缀]` | 通过托管 CMake 安装 Runtime 组件，默认输出到 `target/package/<profile>`。 |

`cargo package` 保留给 Rust crate 打包。桌面安装入口不附带开发用静态库、头文件或 CMake 导出，也不生成平台安装器、签名或公证。

## 环境与依赖

各平台前置条件、固定版本、下载校验、离线缓存和显式本机工具旁路，统一见[依赖获取规范](../standards/dependency-acquisition.md)。构建配置不得硬编码个人路径。第三方库自身不要求使用 C++20，但其头文件、ABI、运行库和链接方式必须兼容自有代码。

三平台构建由 CI 覆盖；它不代替目标机安装、真实窗口、跨显示器 DPR 或 Wayland 验收。图形与部署的剩余范围分别见任务 007、013。

## 安装与部署

```sh
cargo build --locked
cargo panta-package /tmp/panta-install
```

macOS 应用为 `/tmp/panta-install/panta.app`，Windows / Linux 入口为安装前缀下 `bin/panta[.exe]`。macOS 可用 `open /tmp/panta-install/panta.app` 启动；无开发环境变量的启动和真实窗口检查按 AGENTS.md 执行。

Qt 部署脚本收集运行期 QML imports、平台插件和 Qt 动态库；CMake runtime dependency set 收集应用实际链接的 SDK 动态依赖。当前 app 链接 VTK，OCCT / Netgen 由几何和网格目标消费，不属于 app 运行时闭包。

许可、SDK provenance 与 Qt SBOM 在 macOS 的 `.app/Contents/Resources/{licenses,sdk,sbom}`，其他平台在安装前缀的 `share/panta/`。发布前按实际包内容复核再分发要求；验证结果和未完成的平台验收统一记录在任务 013。
