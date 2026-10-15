# 119 — Rust 与 CMake 分层依赖 lint

- 状态：in-progress
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
- 在 CMake 配置/生成图中检查 native target 的直接项目内链接边，包括 `LINK_LIBRARIES`、`INTERFACE_LINK_LIBRARIES`、`INTERFACE_LINK_LIBRARIES_DIRECT` 及可解析的生成表达式/alias；第三方 imported targets 按其分类规则处理。
- 纳入质量命令与 CI，提供合法/非法 fixture 和清楚的来源、目标、允许边诊断。
- 格式与普通 lint 使用任务 120 的变更路径选择器：Rust 格式只运行直接变更包，Clippy 覆盖 L1 与 L2 受影响包；C++/CMake/QML 格式只处理变更文件。clang-tidy 对源文件变更按编译单元筛选，头文件、CMake 或工具配置变更扩大到完整 native 编译单元；未知路径和空 CI diff 触发完整质量检查。

分层图 lint 始终检查完整 Cargo/CMake 图；machete 与 cppcheck 保留工作区级扫描（后者依赖跨翻译单元信息）；qmllint 保留完整 QML target。非目标：重组 crate/target、自动修复依赖、在本任务实现 GUI/CLI 业务命令。

## 涉及分组与跨组依赖

- 涉及分组：Rust 基础、领域、工具链、FFI、native 适配器、bridge、UI、组合根及测试。
- 是否新增跨组依赖：否；只检查任务 118 的允许表。

## 验收标准

- [x] 合法现状图通过；人为加入 `panta-mesh → panta-core`、生成表达式前缀碰撞、隐藏/命名空间 alias 和 `INTERFACE_LINK_LIBRARIES_DIRECT` 等禁止 CMake target 边均失败。
- [x] 诊断标明依赖来源、目标和违反的规则；缺失或无法解析的项目 target 不被静默忽略。
- [x] `cargo lint` 单项与 CI 均执行该检查，且 lint 自身有行为测试。
- [x] `cargo format --changed` 与可安全缩小范围的 lint 只检查受影响文件/包；完整图及跨翻译单元检查保留完整扫描。
- [x] 完整适用验证、独立只读评审和 task-index 更新完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | `cargo test --locked --package panta-tests test_scope::tests -- --nocapture` | 路径选择与完整回退分支通过 | 10 项通过；覆盖未知路径、公共配置、真实 `.clang-tidy` 子目录、Rust L1/L2 闭包。 |
| 2026-10-16 | `cargo format --changed --check` | 变更文件格式通过 | 通过；首次检查发现 CMake 文件格式差异，格式化后复验通过。 |
| 2026-10-16 | `cargo lint layering --check` | Cargo/CMake 实际依赖图与违规 fixtures 检查通过 | 通过；覆盖直接违规边、前缀碰撞、隐藏/命名空间 alias、`INTERFACE_LINK_LIBRARIES_DIRECT`、生成表达式拼接及未登记 `mesh_helper`；同时验证 Qt/CTest utility target 不误报，并完成 launcher 构建。 |
| 2026-10-16 | `cargo test --locked --package panta-tests` | 分层、选择器与聚合测试通过 | 通过：22 项单元测试、3 项 native 集成测试，CTest 76/76。 |
| 2026-10-16 | `cargo format --changed --check`、`git diff --check` | 变更格式与 diff 空白检查通过 | 均通过。 |
| 2026-10-16 | `cargo verify-changed` | 运行变更范围质量与 L1/L2 测试；未知/根配置变更完整回退 | 通过；完整质量检查与 L2 workspace 测试通过，CTest 76/76。 |
| 2026-10-16 | CI（PR） | 运行受影响范围测试与质量检查 | 待 PR CI 完成。 |

## 风险与回退

生成图可能包含条件 target、别名、生成表达式或第三方传递边；只对明确归属本仓库的直接边应用规则，并以 CMake 本身生成的数据判定。若检查误报，先修正分类/提取逻辑，不通过扩大 allowlist 消音。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；任务 118 与任务 120 已完成，开始实现实际 Cargo metadata 与配置后 CMake target graph 检查。
- 2026-10-16：加入 `cargo verify-changed` 本地聚合入口与 CI 变更范围；复核发现后修复 L1 测试遗漏、完整质量回退传递、工具配置路径漏检及 CMake 生成表达式 target/alias 解析，并补充回归。
- 2026-10-16：最终独立评审发现普通仓库 target 的登记检查缺口及 changed lint 重复构建；现已拒绝未登记的可链接仓库 target、略过不参与链接图的 CMake utility target，并复用一次 launcher 构建。`gpt-6.1-sol` medium 复核完整 diff 未发现剩余可操作问题。
- 2026-10-16：全部本地验收项、独立评审及索引更新完成；任务保持 `in-progress`，等待 PR CI 通过后再标记完成。

## 完成摘要

本地分层与测试聚合验证、格式检查和独立只读评审通过；CI 验收待执行。
