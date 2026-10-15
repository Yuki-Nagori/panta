# 120 — PR 按影响范围选择测试

- 状态：planned
- 阶段：验证基础
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)、[100 tests 整体整理与性能入口统一](100-tests-consolidation.md)
- 优先级：P0
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 GitHub issue #36 的 PR CI 影响范围部分。目前 CI 已按 Rust/native/QML 大类分别触发 lint，但任一代码变更仍会运行三平台 workspace 构建/测试与覆盖率，反馈时间过长。本任务是 issue #36 的首个实现优先项：先把测试拆成可解释的层级，并按变更路径和依赖图选择 Rust package（含反向依赖）、CTest label 与 QML 检查；未知影响时保守扩大范围，main 分支继续完整验收。

## 必读

- [测试规范](../standards/testing.md)
- [质量工具链](../modules/quality-tooling.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [任务 100](100-tests-consolidation.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 定义并实现 PR 测试层级：L1 为变更 crate/target 的快速局部验证；L2 运行依赖闭包中受影响的 Rust 反向依赖、native CTest 与 QML 行为测试；公共接口、构建图和无法分类的变更进入对应语言的完整测试矩阵。层级必须给出实际执行命令与选中结果，避免“分层”只成为标签。
- Rust 影响闭包不得止于 Cargo：沿 CXX/FFI staticlib 消费边继续扩展到 native targets、相关 CTest 和 QML module/行为测试。例如 `panta-mesh` 改动需考虑其 Rust 调用方以及 native mesh/CXX 消费者。
- native 影响闭包沿 CMake 项目内 link edge（含 PRIVATE）和 QML module dependency 递归扩展到消费者及其测试；仅凭 public header 或 CTest label 不能断定私有实现变更无 UI 影响。
- 变更路径转换为受影响 Rust package 及仓库内反向依赖，选择对应 package 测试；允许根据已验证的独立性拆开纯单元、native integration 与 GUI/QML 行为检查。
- 为 CTest 建立/使用稳定 labels，并映射 native 源码和测试变更；QML 变更运行 qmllint/QML 行为测试。依赖边、测试覆盖标签或变更分类无法可靠解析时，回退到受影响语言完整测试集；公共构建/协议边界不明时回退完整跨平台矩阵。
- PR 上无法分类、根配置/公共接口/依赖图变化时保守扩大范围；`main` push 保持现有完整 CI，nightly 保持当前诊断套件，不借本任务额外增加 nightly 覆盖率或常规门禁。

非目标：降低测试断言、覆盖率或 lint 门槛；跳过与变更相关的跨平台编译；性能基准纳入 PR 测试。

## 涉及分组与跨组依赖

- 涉及分组：CI、Rust workspace、native CTest、QML 测试。
- 是否新增跨组依赖：否；直接根据 Cargo metadata、CTest labels 和变更路径生成选择，不依赖任务 119 的 lint 实现。

## 验收标准

- [ ] L1/L2/完整矩阵边界、实际命令和升级条件清楚；Rust、CMake、CXX/FFI 与 QML 依赖闭包都纳入消费者测试选择。
- [ ] Rust crate 变更选择本 crate、所有受影响反向依赖及跨语言 native/QML 消费者；native 变更沿项目内 CMake/QML 消费边选择相关 target 和 CTest label；无法可靠追踪时回退对应语言完整测试集。
- [ ] 人工 fixture 覆盖单域、跨域、根配置、未知路径、重命名/删除和空 diff；未知/核心路径保守扩大。
- [ ] PR 的 CI 矩阵只运行所选范围；`main` 现有全量 build/test/lint/coverage 门禁不缩减，nightly 现有 Miri/sanitizer/CPU performance 诊断不受影响。
- [ ] CI 输出可读的路径分类和选择结果，结果可由提交与 workflow run 复核。
- [ ] 完整适用验证、独立只读评审和 task-index 更新完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待任务启动后填写 | 选择器 fixture 与 CI 验收通过 | 未执行。 |

## 风险与回退

影响范围误判会造成假绿。核心构建文件、Cargo/CMake 图、共用 FFI 和未分类路径必须触发保守范围；`main` 全量门禁与 nightly 当前诊断矩阵保持不变，不接受弱化。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；按用户优先级提升为 P0，先解决 PR 测试耗时；按实际依赖/CTest label 设计 L1/L2 与完整矩阵回退。
- 2026-10-16：独立评审指出测试闭包还需覆盖 CMake、CXX/FFI 与 QML 消费边；已将跨语言影响传播及无法解析时的保守回退加入验收条件。

## 完成摘要

未完成。
