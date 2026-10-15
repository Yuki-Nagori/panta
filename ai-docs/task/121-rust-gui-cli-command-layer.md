# 121 — GUI 与 CLI 共享 Rust 命令入口

- 状态：planned
- 阶段：Rust 架构整理
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)、[107 Rust 工程服务边界](107-rust-service-boundaries-and-revisions.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 GitHub issue #36 的统一命令/查询入口部分。GUI 与未来 CLI 不应各自复制领域校验或流程编排；Rust 应用层拥有命令输入、领域校验、执行结果和可诊断错误，Qt/C++ 与 CLI 仅将各自输入适配为同一接口并呈现结果。

## 必读

- [分层标准](../standards/layering.md)
- [仓库服务与修订边界](../architecture/application-and-storage.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [任务 107](107-rust-service-boundaries-and-revisions.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 选择一个最小且有用户价值的真实命令/查询，并为 GUI 与 CLI 实现两个消费者。
- 命令输入/输出使用 Qt 无关的 Rust 类型；错误保持稳定、可翻译或可供 CLI 诊断。
- 测试命令行为和两个调用入口；文档说明未来新增命令的归属。

非目标：把 QML 控件状态、Qt 类型、文件选择器或显示逻辑迁入 Rust；一次性建立大型命令总线/插件系统；修改工程格式。

## 涉及分组与跨组依赖

- 涉及分组：Rust 领域/应用服务、CLI、FFI、Qt bridge/UI。
- 是否新增跨组依赖：由应用服务提供命令；GUI 和 CLI 依赖应用服务，不允许 Rust 反向依赖 Qt/bridge。

## 验收标准

- [ ] 一条用户可观察的真实命令/查询由 Rust 应用入口定义并实现。
- [ ] GUI 经 FFI/bridge 与 CLI 都调用同一应用入口，行为测试证明两侧共用执行逻辑。
- [ ] 输入校验、错误语义和结果结构不依赖 Qt；调用侧只做适配与呈现。
- [ ] 架构文档与 crate 依赖方向同步；完整适用验证、独立只读评审和索引更新完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待任务启动后填写 | Rust 命令测试和 GUI/CLI 共用入口验证通过 | 未执行。 |

## 风险与回退

当前 `panta-cli` 尚未成为既有产品入口；不要伪称已有 CLI。选定命令须能从当前 GUI/服务真实复用，并使用最窄 CLI consumer；若只在启动器/build script 共用库函数，不满足本任务验收。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；先消费任务 118 的边界设计，再由本任务根据现有应用服务选择共享命令。

## 完成摘要

未完成。
