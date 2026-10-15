# 119 — Rust 与 CMake 分层依赖 lint

- 状态：planned
- 阶段：Rust 架构整理
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 GitHub issue #36 的自动边界检查部分。将任务 118 定义并核对过的 Rust crate 与 native CMake target 允许依赖表转成可重复执行的 lint，避免依赖方向回退。检查真实 Cargo metadata 和 CMake target link graph，不用易漏项的人工 grep 代替。

## 必读

- [分层标准](../standards/layering.md)
- [质量工具链](../modules/quality-tooling.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 读取 workspace Cargo metadata，拒绝未登记的仓库内 crate 依赖边。
- 在 CMake 配置/生成图中检查 native target 的直接项目内链接边；第三方 imported targets 按其分类规则处理。
- 纳入质量命令与 CI，提供合法/非法 fixture 和清楚的来源、目标、允许边诊断。

非目标：重组 crate/target、自动修复依赖、在本任务实现 GUI/CLI 业务命令或 CI 影响范围选择。

## 涉及分组与跨组依赖

- 涉及分组：Rust 基础、领域、工具链、FFI、native 适配器、bridge、UI、组合根及测试。
- 是否新增跨组依赖：否；只检查任务 118 的允许表。

## 验收标准

- [ ] 合法现状图通过；人为加入 `panta-mesh → panta-core` 和禁止的 CMake target 边均失败。
- [ ] 诊断标明依赖来源、目标和违反的规则；缺失或无法解析的项目 target 不被静默忽略。
- [ ] `cargo lint` 单项与 CI 均执行该检查，且 lint 自身有行为测试。
- [ ] 完整适用验证、独立只读评审和 task-index 更新完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待任务启动后填写 | 违规图失败、当前图通过 | 未执行。 |

## 风险与回退

生成图可能包含条件 target、别名、生成表达式或第三方传递边；只对明确归属本仓库的直接边应用规则，并以 CMake 本身生成的数据判定。若检查误报，先修正分类/提取逻辑，不通过扩大 allowlist 消音。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；等待任务 118 的依赖规则落定。

## 完成摘要

未完成。
