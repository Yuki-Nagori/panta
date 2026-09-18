# panta

[![CI](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

规划中的 CAE 桌面平台，采用 Qt Quick/QML、OpenCASCADE、Netgen 和 VTK；物理求解通过外部进程接入。

> **命名**：Panta 取自赫拉克利特的名言 *Panta rhei*（希腊语 **πάντα ῥεῖ**，意为"万物皆流"）。注塑过程中，聚合物熔体、温度场、压力场与材料形态始终处于动态演化之中；CAE 的本质，正是对这一"流变万物"过程的数值再现。

**当前状态：Rust workspace、Cargo 调度 native 构建、Qt Quick/C++ ViewModel 桌面骨架和 `.pa` parser/formatter CLI 已落地；完整 CAE 业务尚未实现。**

## 环境要求

必须预装：

| 平台 | 必要环境 |
|---|---|
| macOS | git、[rustup](https://rustup.rs)、Apple 命令行工具（`xcode-select --install`） |
| Linux | git、[rustup](https://rustup.rs)、C/C++ 编译器（gcc 或 clang）；发行版需 glibc ≥ 2.34（Qt 预编译包基于 RHEL 9.6） |
| Windows | git、[rustup](https://rustup.rs)、MSVC 构建工具（Visual Studio Build Tools 2022） |

Rust 工具链版本由 [rust-toolchain.toml](rust-toolchain.toml) 固定，仓库内执行 cargo 命令时按提示 `rustup toolchain install` 即可；其余依赖（CMake、Ninja、Qt、VTK、OCCT、Netgen）由构建引导自动拉取，无需预装，见 [依赖获取与主平台环境](ai-docs/standards/dependency-acquisition.md)。

## 当前可用命令

| 命令 | 当前行为 |
|---|---|
| `cargo build` | 统一构建入口：Cargo 先生成 `panta-ffi` staticlib，再经 build.rs 调度托管 CMake/Ninja 构建 Rust workspace/native（Qt、GoogleTest 等首次自动下载到 `target/`） |
| `cargo run --locked --package panta-tests -- toolchain` | 校验最近一次 Cargo build 使用了托管 LLVM 22.1.7（clang/clang++/clang-cl/clang-format/clang-tidy）、CMake/Ninja、Qt、GoogleTest，并生成 native `compile_commands.json`；CI 在跨平台 build 后执行 |
| `cargo build --no-default-features` | 诊断构建：通过 Cargo feature 裁剪 `Panta.Bridge` 静态模块，构建不依赖 ViewModel 的最小 Shell；默认构建启用该模块 |
| `cargo lint` | 根 `tests/` 入口统一执行 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、include-cleaner 和 Cppcheck；工具缺失或任一检查失败即非零。可用 `cargo lint <tool>` 单独运行 |
| `cargo audit` | 通过根 runner 按固定版本准备 cargo-deny 到 `target/panta-tools/<cargo-tool>/<version>`，执行 RustSec、许可证和依赖关系审计 |
| `cargo test` | Cargo workspace 测试入口，并由根 `tests/` package 集成测试聚合 qmllint、完整 CTest/GTest/QtTest/QML 行为套件 |
| `cargo format` | 根 `tests/` 入口检查 Rust、C++/CXX、CMake 和 QML 格式；只改 Rust 时使用官方 `cargo fmt --all -- --check` |
| `cargo coverage` | 通过根 runner 按固定版本准备 cargo-llvm-cov 到 `target/panta-tools/<cargo-tool>/<version>`，执行 Rust 覆盖率门禁；`cargo coverage native` 生成 native C++ 覆盖率报告，两个 CI job 分开运行 |
| `cargo quality` | 依次执行 `cargo format`、`cargo lint`、依赖审计、Rust 测试和 native/QML 测试 |
| `cargo run -p panta-dslc -- check resources/i18n/panta-cn.pa` | 校验 `.pa` 语言字典 |
| `cargo run -p panta-dslc -- format --check resources/i18n/panta-cn.pa` | 检查 `.pa` 是否为规范格式；写回使用 `format <file>` |
| `cargo run` | 启动 Qt Quick 主窗口（当前为骨架界面：主题、命令按钮、占位面板、错误展示入口） |

作为完整 CAE 桌面（工程树/视口/属性区等）的体验仍是目标：[构建说明](ai-docs/architecture/build-and-development.md) 列出落地条件；托管引导（CMake/Ninja 二进制自动供给）见任务 020。

native 直接诊断构建（不经 Cargo）仍可用：在 `native/` 下执行 `cmake --preset debug`、`cmake --build --preset debug`、`ctest --preset debug`、`cmake --install build/debug`。

日常命令默认使用 Cargo 的常规依赖解析。CI 和需要复现锁文件的验证会显式追加 `--locked`（例如 `cargo test --locked`）；它要求已提交的 `Cargo.lock` 与 manifests 一致，不会自动更新锁文件。根 `tests/` 目录按 `src/` 调度器、`integration/` Cargo 聚合测试、`cpp/` C++ 测试和 `qml/` QML 测试分类；Rust 单元测试仍与被测实现同文件，跨 crate 行为测试放在对应 crate 的 `tests/`。

## 开始工作

1. 从 [任务索引](ai-docs/task-index.md) 选择任务；新增工作先复制 [任务模板](ai-docs/task/_template.md)。
2. 阅读任务引用的 [架构说明](ai-docs/architecture/README.md) 与 [技术规范](ai-docs/standards/README.md)。
3. 按任务实施、验证并更新状态。具体依赖与可开始任务以[任务索引](ai-docs/task-index.md)为准。

提交前质量门禁（fmt + clippy）经 git hooks 强制，新克隆后启用一次：

```sh
git config core.hooksPath .githooks
```

国际化、跨平台路径与运行时、变量 DSL、C++/QML 模块和热重载的规划见[重要模块说明](ai-docs/modules/README.md)。

完整路由见 [AGENTS.md](AGENTS.md)。

外部边界：[[External / 非本仓库] MoldSolver](ai-docs/architecture/external-moldsolver.md) · [[External / 非本仓库] Mold Protocol](ai-docs/architecture/external-mold-protocol.md)。许可证：[Apache-2.0](LICENSE)。
