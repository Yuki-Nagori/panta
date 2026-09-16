# panta

规划中的 CAE 桌面平台，采用 Qt Quick/QML、OpenCASCADE、Netgen 和 VTK；物理求解通过外部进程接入。

**当前仅有文档，尚无可构建或运行的应用。**

## 开始工作

1. 从 [任务索引](ai-docs/task-index.md) 选择任务；新增工作先复制 [任务模板](ai-docs/task/_template.md)。
2. 阅读任务引用的 [架构说明](ai-docs/architecture/README.md) 与 [技术规范](ai-docs/standards/README.md)。
3. 按任务实施、验证并更新状态。首项为 [001 Cargo 配置](ai-docs/task/001-cargo-config.md)。

完整路由见 [AGENTS.md](AGENTS.md)。`cargo build`、`cargo test`、`cargo run` 是后续目标入口，当前不可用；[构建说明](ai-docs/architecture/build-and-development.md) 列出落地条件。

外部边界：[[External / 非本仓库] MoldSolver](ai-docs/architecture/external-moldsolver.md) · [[External / 非本仓库] Mold Protocol](ai-docs/architecture/external-mold-protocol.md)。许可证：[Apache-2.0](LICENSE)。
