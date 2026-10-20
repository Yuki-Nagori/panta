# 跨语言质量工具链

[模块导航](README.md) · [测试入口与目录](../standards/testing.md) · [验证与评审约定](../standards/validation-and-review.md) · [技术基线](../standards/baseline.md) · [任务 032](../task/032-cross-language-quality-gates.md) · [任务 042](../task/042-unified-llvm-toolchain.md)

## 当前结构

质量命令由 `panta-tests` 调度，Cargo 是统一入口，CMake 管理 native 构建与测试图。Rust、C++/Objective-C++、CXX、CMake、QML 与 `.pa` 的格式和 lint 命令，以及 L1/L2 受影响范围，统一记录在[测试规范](../standards/testing.md)；工具版本和供给方式见[技术基线](../standards/baseline.md)及[依赖获取规范](../standards/dependency-acquisition.md)。本页不重复维护命令清单或缓存细节。

PR 与 main 的检查按变更影响范围运行测试和适用 lint，公共构建边界或无法分类的路径会扩大范围。Nightly 运行完整 workspace 测试、全量 lint、Rust/native coverage 与动态检查。Rust coverage 门槛为函数/行各 85%，仅 nightly 阻断；native coverage 上传报告，不设百分比门槛。日常 CPU 性能采样也在 nightly 收集，但不设性能阈值；GPU 基准单独运行。

## 当前边界与后续

- `cargo verify-changed` 聚合受影响范围的格式、lint 与测试；`cargo quality` 用于完整本地质量检查。具体选择规则和保守回退条件见[测试规范](../standards/testing.md)。
- 分层依赖检查读取 Cargo metadata 和配置后的 CMake target 图；图检查保持完整扫描。C++ 头文件检查、跨翻译单元分析与 Objective-C++ 平台检查按各自 lint 规则运行。
- sanitizer 与 Miri 的支持范围、工具限制及 CI 矩阵以任务 042 和 workflow 为准；工具通过不替代真实窗口、GPU、ABI 或平台验收。
- 任务 032 仍为 `deferred`，不再追求 Rust 100% 覆盖率。coverage 排除与未测边界应在该任务及报告元数据中维护，不在模块文档复制实现细节。

工具链只纳入仓库当前语言和真实代码。Python 当前服务开发质量工具；未引入 Python 业务运行时或 Node 源码时，不添加对应的生产语言检查。
