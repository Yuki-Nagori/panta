# panta

规划中的 CAE 桌面平台，采用 Qt Quick/QML、OpenCASCADE、Netgen 和 VTK；物理求解通过外部进程接入。

**当前状态：Rust workspace 骨架可构建、可测试；桌面程序与 native 构建尚未实现。**

## 当前可用命令

工具链为 stable 1.98.1，由 [rust-toolchain.toml](rust-toolchain.toml) 固定（选型依据见任务 [001](ai-docs/task/001-cargo-config.md)）。新环境先安装 [rustup](https://rustup.rs)，再执行 `rustup toolchain install 1.98.1`；之后在仓库内使用 `cargo` 会自动命中固定版本。

| 命令 | 当前行为 |
|---|---|
| `cargo build --locked` | 构建 Rust 骨架（launcher） |
| `cargo test --locked` | 运行 launcher 单元测试 |
| `cargo fmt --all -- --check` | 格式检查 |
| `cargo run` | 输出"桌面尚未接入"诊断，以退出码 69 结束；不会启动 GUI |

作为完整桌面入口（自动调度 CMake、Qt/VTK/OCCT/Netgen 并启动程序）的 `cargo build`/`cargo run`/`cargo test` 仍是目标体验：[构建说明](ai-docs/architecture/build-and-development.md) 列出落地条件，由任务 003/004/005 实施。

## 开始工作

1. 从 [任务索引](ai-docs/task-index.md) 选择任务；新增工作先复制 [任务模板](ai-docs/task/_template.md)。
2. 阅读任务引用的 [架构说明](ai-docs/architecture/README.md) 与 [技术规范](ai-docs/standards/README.md)。
3. 按任务实施、验证并更新状态。001 Cargo 配置已完成，下一项为 [002 平台与依赖基线](ai-docs/task/002-dependency-baseline.md)。

完整路由见 [AGENTS.md](AGENTS.md)。

外部边界：[[External / 非本仓库] MoldSolver](ai-docs/architecture/external-moldsolver.md) · [[External / 非本仓库] Mold Protocol](ai-docs/architecture/external-mold-protocol.md)。许可证：[Apache-2.0](LICENSE)。
