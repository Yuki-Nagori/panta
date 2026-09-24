# 076 — Cargo 测试与质量 runner 维护

- 状态：done
- 阶段：验证基础
- 依赖：[011 统一测试与质量入口](011-test-quality-entrypoints.md)、[043 根目录质量入口与测试聚合](043-root-quality-runner.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

对 `tests/src/main.rs` 做整体代码审查和必要优化，保持 Cargo 测试、质量、工具链、覆盖率与 sanitizer 命令行为不变，降低重复分支和错误诊断不一致。只实施能通过结构清晰度或错误路径证据说明收益的调整；不以重排整文件或新增抽象层为目标。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [011 统一测试与质量入口](011-test-quality-entrypoints.md)
- [043 根目录质量入口与测试聚合](043-root-quality-runner.md)

## 范围与非目标

- 审查 `tests/src/main.rs` 的命令路由、工具准备、环境构造、错误传播及可复用流程。
- 仅收敛职责重复或不一致的实现，并保留命令名、参数、退出码、输出语义与 CI 工作流。
- 与任务 075 的 Qt moc lint 前置步骤保持一致，不将 Windows SDK ABI 修复混入本 task。
- 不新增依赖，不更改 Cargo/CTest 的质量门禁阈值或任务覆盖范围。

## 前置条件与待决策

- 现有命令接口和平台差异以 `tests/src/main.rs`、Cargo workspace 与对应 workflow 为准。
- 若审查发现需要改变外部命令行为或质量范围，先更新本任务的范围与验收，再实施。

## 实施步骤

1. 阅读整个 runner 并列出重复及复杂度较高的命令流程。
2. 对照调用处和构建脚本确认语义，优先采用最小的局部重构。
3. 运行相关 Cargo 聚合验证和 lint，检查命令接口未变化。
4. 记录未处理项及原因；无明确收益的代码保持原样。

## 预计改动

- `tests/src/main.rs`
- `ai-docs/task-index.md` 与本任务

## 清理与兼容例外

不增加兼容路径；没有废弃项时保持“不适用”。

## 验收标准

- [x] `tests/src/main.rs` 全文件经过审查；仅保留可说明收益的优化。
- [x] 命令/参数路由、平台差异、失败退出行为和验证范围保持一致。
- [x] Cargo format、相关 lint 与 `cargo test --locked --workspace` 通过。
- [x] 任务记录本次调整及真实验证证据，索引状态一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | macOS：`cargo format --check`、`cargo build`、`cargo lint --check` | 格式、常规构建与八阶段质量聚合通过 | 全部通过；lint 包含 qmllint、clang-tidy、include-cleaner 与 cppcheck |
| 2026-09-24 | macOS：`cargo test --locked --workspace` | Rust、native CTest 与 QML 聚合测试通过 | 通过；native CTest 56/56，workspace Rust 测试及 doc-tests 均通过 |
| 2026-09-24 | macOS：`actionlint .github/workflows/sdk-googletest.yml` | GoogleTest SDK workflow 语法检查通过 | 通过 |

## 风险与回退

runner 负责编排跨平台构建与测试，错误合并默认值可能改变 CI 覆盖面。优化限定为调用语义可逐项对照的局部重构；出现行为偏差时恢复对应局部实现，不变更用户测试数据或构建缓存。

## 决策与工作记录

- 2026-09-24：维护者要求在 CI 修复期间整体审查 `tests/src/main.rs`；独立登记，避免与 Windows SDK ABI 修复混为一项。
- 2026-09-24：统一 CTest 参数装配、CMake build 命令和 native 路径；C++ 格式化从每文件启动一次工具改为单次批量运行，子进程启动失败补充操作上下文。
- 2026-09-24：clang-tidy / include-cleaner 前显式生成排除默认构建的 Qt benchmark moc；聚合 lint 共享一次 launcher 与 moc 准备，同时让单项 lint 自行准备所需产物。

## 完成摘要

完成。`tests/src/main.rs` 的流程已完整审查；在保持 CLI、检查覆盖范围和失败退出语义不变的前提下，合并重复的命令准备流程、减少聚合 lint 重复构建、修复 Qt benchmark moc 前置生成，并统一启动错误上下文。`cargo format --check`、`cargo build`、`cargo lint --check`、`cargo test --locked --workspace` 和 workflow `actionlint` 全部通过。
