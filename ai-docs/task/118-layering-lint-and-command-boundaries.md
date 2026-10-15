# 118 — 实际依赖图与应用命令层设计

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[106 Rust 架构复审](106-rust-architecture-review.md)、[111 构建与质量模块边界](111-build-and-quality-module-boundaries.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 [GitHub issue #36](https://github.com/Yuki-Nagori/panta/issues/36)。issue 同时覆盖现状架构盘点、Rust/CMake 分层 lint、CI 影响范围选择和 GUI/CLI 共享命令层，跨度较大。本任务完成依赖图核对、架构与命令边界设计、相关文档和模板字段；自动 lint、CI 选择器及命令实现分别拆为任务 119–121。Rust 与 Native 分别按实际依赖图定义 L1、L2 等层级，跨语言边界单独标注；不强求两张图共用层号或层数。

## 必读

- [分层规则](../standards/layering.md)
- [测试规范](../standards/testing.md)
- [质量工具链](../modules/quality-tooling.md)
- [仓库布局](../architecture/repository-layout.md)
- [架构导航](../architecture/README.md)
- [模块导航](../modules/README.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

- 以真实 manifest/target 依赖盘点 Rust crate、native target、QML 模块、bridge、组合根及测试；更新分层标准、仓库布局、模块导航和任务模板。
- 对照 Cargo manifest、CMake targets 和 QML module 核对当前依赖，形成分组及允许依赖表。
- 更新分层标准、仓库布局、模块导航、质量工具链设计和任务模板字段。
- 明确 Rust 应用命令/查询层与 GUI、CLI 的适配边界，记录共享命令落地任务。
- 将自动分层 lint、PR 影响范围 CI、共享命令实现拆成独立 task，标明依赖和验收范围。

非目标：实现自动依赖 lint、修改 CI 测试选择器、实现 CLI/GUI 命令；按理想架构重组 crate/target；实施 issue #34/#35 的具体迁移内容；调整第三方依赖获取方式或缩减 main 分支现有完整门禁及 nightly 诊断套件。

## 涉及分组与跨组依赖

- 涉及分组：Rust 基础/工具、Rust 领域、Rust 工具链、应用服务、FFI、native 适配器、bridge、UI、组合根、测试。
- 是否新增跨组依赖：本任务只记录当前边界，不新增代码依赖。任务 121 选定共享命令后，再按该依赖图决定所需边。

## 前置条件与待决策

- 当前依赖图来自 `cargo metadata`、各 `CMakeLists.txt` 的 `target_link_libraries` 及 QML module 声明；实施时须重新采集并以仓库现状为准。
- 如 GUI/CLI 没有可共享的实际业务行为，先评估最窄的现有命令/查询，不为满足验收新造平行业务逻辑或空 CLI。

## 实施步骤

1. 盘点并记录当前 crate/target/module 依赖，按 Rust 与 Native 依赖图定义 L1、L2 等层级及允许边；跨语言边单独注明。
2. 更新分层、仓库布局、命令层设计、质量工具链说明与任务模板。
3. 将 lint、影响范围 CI、共享命令拆分为任务 119–121，并串好依赖。
4. 核验 issue #34/#35 的既有任务链接和边界，运行文档检查并完成独立评审。

## 预计改动

`ai-docs/standards/layering.md`、`ai-docs/architecture/repository-layout.md`、`ai-docs/architecture/README.md`、`ai-docs/modules/README.md`、`ai-docs/modules/quality-tooling.md`、新增命令层设计说明、`ai-docs/task/_template.md`、任务索引及任务 119–121。

## 清理与兼容例外

无代码替换，不涉及废弃实现或兼容例外；现有 main/nightly 验证行为由任务 120 明确保留。

## 验收标准

- [x] Rust 与 Native 的 L1、L2 等层级准确对应当前 Cargo、CMake、QML 图；跨语言边单独列出，文档明确已实现与规划边界。
- [x] 当前 Rust/CMake/QML 分组及允许依赖方向经代码核对；未实现的命令入口标明为规划。
- [x] 任务模板包含涉及分组与是否新增跨组依赖字段。
- [x] lint、影响范围 CI、共享命令各有独立任务，依赖、范围和验收不重复也无遗漏；测试分层任务 120 为 P0，优先于依赖 lint 和命令层实现。
- [x] #34/#35 后续职责链接已核实并在相关文档中清晰引用。
- [x] 文档内部链接有效；独立评审完成且发现已处理；任务和索引同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | `cargo metadata --locked --format-version 1 --no-deps` 与 native CMake target 依赖盘点 | 建立可追溯的当前依赖图 | Rust workspace 依赖清单与 native target 直接链接边已采集；标准文档按 Rust/Native 两张图列出 L1 及以上层级，跨语言边另行注明。 |
| 2026-10-16 | Markdown 本地链接检查、`git diff --check HEAD`、`gpt-6.1-sol` medium 独立只读评审 | 链接有效、无空白错误，评审发现已处理 | 13 个修改 Markdown 文件的本地链接均可解析；diff 检查通过。评审提出的 nightly 说明和跨语言测试消费者闭包问题已修复并复核，无剩余可操作问题。功能测试未运行，本任务只改文档。 |

## 风险与回退

- 依赖 lint 若只按包名匹配可能漏掉 target alias、条件依赖或 PRIVATE/PUBLIC 传播；从生成的 metadata/真实 CMake target graph 获取信息，并用受控 fixture 验证违规与合法边。
- 影响范围选择若漏掉反向依赖或共享测试入口会造成假绿；规则需默认保守，无法准确判定时回退对应语言的完整测试；main 分支保留现有全量门禁，nightly 保留现有诊断套件。
- 共享命令若把 UI 状态或 Qt 类型带入 Rust 会越界；命令只接受领域输入并返回领域结果，GUI/CLI 各自在边界做适配。

## 决策与工作记录

- 2026-10-16：issue #36 对应编号 036 已被既有任务占用，按任务索引规则使用新编号 118；在专用分支开始实施。
- 2026-10-16：通过 `gh` 核实 issue #34/#35 均已关闭；其本地实施任务分别为 [116](116-rust-process-parameter-constraints.md) 与 [117](117-rust-project-document-workspace.md)，均已完成。#36 原文已有这两个后续 issue 链接，不重复创建。

## 完成摘要

已完成设计范围：核对并记录 Rust/Native 的 L1+ 依赖层级与跨语言边界，整理 GUI/CLI 命令层职责，创建 119–121 后续任务。issue #36 的实现工作仍由后续任务跟进；测试影响范围分离任务 120 优先级最高（P0）。
