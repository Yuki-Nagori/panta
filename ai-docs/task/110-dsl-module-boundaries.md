# 110 — DSL AST、解析、校验与输出模块整理

- 状态：planned
- 阶段：Rust 架构整理
- 依赖：[107](107-rust-service-boundaries-and-revisions.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

## 目标与范围

在 panta-dsl-core 内分离 AST / 诊断、Pest 解析、语义校验、格式化和 TS 输出。保持共享 grammar、FSM 模块、公共 API、CLI 与构建脚本输出；不创建第二套 parser 或无消费者的新 crate。

本任务对应 [106 架构评审](106-rust-architecture-review.md) 的后续建议；确认导入后台化仍由 063 实施。保持 UI 与正常工程 schema，不将新基础接口描述为已经实现。

## 必读

- [分层规则](../standards/layering.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [ ] AST / 诊断 / 格式化 / TS / FSM 既有回归通过，格式及生成输出保持一致，CLI 和编译期消费者通过。
- [ ] 替换实现与失效引用已删除，无新增兼容分支。
- [ ] Cargo 工作区聚合、构建、格式及完整 lint 通过；记录具体平台与未覆盖点。

## 验证计划与结果

先补定向回归定位，再执行 `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check`。涉及性能容量或真实窗口时按实际范围单独验证。本任务仍为规划，尚未执行实施验证。

## 工作记录

- 2026-10-08：按用户要求一次登记剩余 Rust 架构任务，先提交规划后实施；避免把后续建议混入已完成的 107。

## 完成摘要

未完成。
