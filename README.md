# panta

[![CI](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml/badge.svg)](https://github.com/Yuki-Nagori/panta/actions/workflows/ci.yml)

规划中的 CAE 桌面平台，采用 Qt Quick/QML、OpenCASCADE、Netgen 和 VTK；物理求解通过外部进程接入。

**当前状态：Rust workspace 骨架可构建、可测试；桌面程序与 native 构建尚未实现。**

## 环境要求

必须预装：

| 平台 | 必要环境 |
|---|---|
| macOS | git、[rustup](https://rustup.rs)、Apple 命令行工具（`xcode-select --install`） |
| Linux | git、[rustup](https://rustup.rs)、C/C++ 编译器（gcc 或 clang） |
| Windows | git、[rustup](https://rustup.rs)、MSVC 构建工具（Visual Studio Build Tools） |

Rust 工具链版本由 [rust-toolchain.toml](rust-toolchain.toml) 固定，仓库内执行 cargo 命令时按提示 `rustup toolchain install` 即可；其余依赖（CMake、Ninja、Qt、VTK、OCCT、Netgen）由构建引导自动拉取，无需预装，见 [依赖获取与主平台环境](ai-docs/standards/dependency-acquisition.md)。

## 当前可用命令

| 命令 | 当前行为 |
|---|---|
| `cargo build --locked` | 构建 Rust workspace 并经 build.rs 调度 CMake/Ninja 构建 native 骨架（构建树在 `target/` 内） |
| `cargo test --locked` | 运行 launcher 单元测试（native 测试经 `native/` 下 ctest 运行） |
| `cargo fmt --all -- --check` | Rust 格式检查（C++ 格式检查见任务 003） |
| `cargo run` | 启动 native 骨架 `panta-native`：打印版本并以 0 退出；参数原样转发（未知参数 → 64），产物缺失 → 69 |

作为完整桌面入口（自动托管 Qt/VTK/OCCT/Netgen 并启动 GUI 程序）的体验仍是目标：[构建说明](ai-docs/architecture/build-and-development.md) 列出落地条件；托管引导（CMake/Ninja 二进制自动供给）见任务 020，Qt 桌面见任务 005。

native 直接诊断构建（不经 Cargo）仍可用：在 `native/` 下执行 `cmake --preset debug`、`cmake --build --preset debug`、`ctest --preset debug`、`cmake --install build/debug`。

## 开始工作

1. 从 [任务索引](ai-docs/task-index.md) 选择任务；新增工作先复制 [任务模板](ai-docs/task/_template.md)。
2. 阅读任务引用的 [架构说明](ai-docs/architecture/README.md) 与 [技术规范](ai-docs/standards/README.md)。
3. 按任务实施、验证并更新状态。001、002 已完成，CI（018）已建立；下一项为 [003 CMake/Ninja 原生构建骨架](ai-docs/task/003-cmake-native-skeleton.md)。

完整路由见 [AGENTS.md](AGENTS.md)。

外部边界：[[External / 非本仓库] MoldSolver](ai-docs/architecture/external-moldsolver.md) · [[External / 非本仓库] Mold Protocol](ai-docs/architecture/external-mold-protocol.md)。许可证：[Apache-2.0](LICENSE)。
