# 测试目录与运行入口

| 目录 | 职责 |
|---|---|
| `rust/` | Rust 公共 API 行为回归，由对应 crate 的 Cargo test target 登记 |
| `cpp/` | C++ 功能测试，按模块分组 |
| `qml/` | QML 组件与交互功能测试 |
| `integration/` | Cargo 的 native/QML 聚合入口 |
| `support/` | 被多种功能 / 性能测试实际复用的辅助代码，按 Rust、Qt、QML 分组 |
| `fixtures/` | 固定回归输入；来源、单位和预期随夹具记录 |
| `performance/` | 手动性能基准、专用辅助代码及其正确性测试 |
| `src/`、`build.rs` | Cargo 质量与性能命令的编排实现 |

常规功能回归：`cargo test --locked --workspace`。性能套件：`cargo performance`，先验证辅助代码，再串行执行 CPU、GPU；不纳入常规测试或 CI 时间门禁。详见[性能模块](../ai-docs/modules/performance.md)。

新增文件须同步 Cargo / CMake 注册、源码质量清单与路径引用；单个测试内部的局部辅助函数不为了分层而单独拆文件。共享辅助代码不持有应用业务状态，不复制领域规则。
