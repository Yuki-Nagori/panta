# 078 — QML 性能基准登记规范

- 状态：done
- 阶段：文档维护
- 依赖：[069](069-project-docks-review-and-ablation.md)、[048](048-performance-testing.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

任务 069 已建立可扩展的 QML CPU/GPU 手动基准入口。规范需要求新增 QML 组件和会影响性能的 QML 改动同步登记基准场景，避免性能测试只覆盖既有 Dock。

## 必读

- [QML 规范](../standards/qml.md)
- [任务 069：QML 面板 review 与性能消融](069-project-docks-review-and-ablation.md)
- [任务 048：性能基线与性能测试体系](048-performance-testing.md)
- [性能模块](../modules/performance.md)
- [文档规范](../standards/documentation.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

修改 QML 规范，明确新 QML 文件/组件、delegate、页面及影响界面构造、绑定、布局、渲染或交互更新的改动要登记到现有可扩展性能基准场景，并说明 CPU/GPU 的选择和记录要求。性能基准保持开发侧手动运行，不加入 CI/CTest 门禁。

不新增 benchmark 实现，不更改 069 性能工具或 CI 配置。

## 前置条件与待决策

任务 069 已实现 `tests/qml/project_docks_cpu_benchmark.cpp` 和 `tests/qml/project_docks_gpu_benchmark.cpp` 两个手动入口；以任务 069 和性能模块现有测量边界为准。

## 实施步骤

1. 对照任务 069 和性能模块确认 CPU 构造/更新测量、GPU 真实窗口帧呈现测量的边界。
2. 在 QML 规范增加新增组件到基准场景的映射、场景覆盖与测量记录要求。
3. 更新任务和索引，检查本地文档链接、task 状态和差异。

## 预计改动

- `ai-docs/standards/qml.md`
- `ai-docs/task/078-qml-performance-benchmark-policy.md`
- `ai-docs/task-index.md`

## 清理与兼容例外

无废弃项，无兼容例外。

## 验收标准

- [x] 每个新增 QML 文件/组件及实质改变界面构造、更新或渲染行为的改动，都要求登记到合适的手动性能基准场景。
- [x] 规范区分 CPU 与 GPU 场景，要求记录代表性负载、Qt/平台、采样方式与结果；已有性能模块入口保持一致。
- [x] 性能基准不新增到 CI 或 CTest，task、规范和索引状态一致。
- [x] 本地链接、Markdown 围栏与 `git diff --check` 通过。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | 对照任务 069、048 与性能模块更新 QML 规范 | 新增 QML 的性能场景映射规则明确，手动测量边界与现状一致 | 通过：每个新增 QML 至少进入 CPU 或 GPU 手动基准；按 CPU 构造/更新、GPU 可见帧呈现、两者均受影响进行映射，并记录负载、工具环境、预热/采样和 p50/p95；没有加入 CI/CTest 门禁。 |
| 2026-09-24 | 本地文档链接、围栏、task/index 状态与 `git diff --check` | 引用有效，格式和状态一致 | 通过；具体检查命令和计数记录在任务 077。 |

## 风险与回退

规则过宽可能要求为纯静态内容维护无信息量的测量；通过要求场景实际构造/使用新组件，并按可测成本选择 CPU 或 GPU 入口，保持测量可解释。

## 决策与工作记录

- 2026-09-24：用户要求新增 QML 内容同步纳入性能测试；登记本任务并只修改文档。
- 2026-09-24：更新 `ai-docs/standards/qml.md`，为每个新增 QML 建立手动基准场景映射，并按 CPU/GPU 成本给出选择与结果记录规则；task 与索引同步完成。

## 完成摘要

已在 QML 规范要求新增 QML 文件/组件登记至 CPU 或 GPU 手动性能场景，并对实质性能改动更新场景；测量要求与任务 069/性能模块一致，未新增 CI/CTest 门禁。
