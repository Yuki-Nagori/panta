# 131 — 影响范围选择器的质量域路由

- 状态：in-progress
- 阶段：验证基础
- 依赖：[119 Rust 与 CMake 分层依赖 lint](119-rust-cmake-layering-lint.md)、[120 PR 按影响范围选择测试](120-pr-affected-test-selection.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-25 / 2026-10-25

## 目标与背景

同步评估并移植 [VibeSTEM/caret commit `e41f400`](https://github.com/VibeSTEM/caret/commit/e41f4004ecc01ae9704fd9b2789b7f04b447e2ae) 对 Rust 影响范围选择器的优化。该实现来自 Panta 的派生仓库，但两边 Cargo package、CMake 模块、测试标签和质量入口已有差异，需按 Panta 当前依赖图移植，不能直接覆盖后假设等价。

目标是让一个共享选择器分别描述测试、格式、各类 lint、分层检查和选择器自身回归测试的受影响范围。局部配置只扩大其对应检查域；未知路径、空 CI diff 和公共构建边界仍按现有安全策略扩大，防止 CI 静默漏检。

## 必读

- [测试规范](../standards/testing.md)
- [验证与评审](../standards/validation-and-review.md)
- [任务 119](119-rust-cmake-layering-lint.md)
- [任务 120](120-pr-affected-test-selection.md)
- [Cargo 测试与质量 runner](076-panta-tests-runner-maintenance.md)

## 范围与非目标

- 在 `tests/src/test_scope.rs` 中区分 Rust 测试包、Clippy 包、Rust 格式包、native/C++ lint、CMake lint、QML lint、分层检查和 selector runner 自测所需范围。
- 将选择器输出接入本地 `cargo verify-changed`、`.github/actions/changed-paths` 与 PR CI，使本地和 CI 消费同一份路径规划。
- 补充各类路径、配置文件、CI 路由文件、测试 runner 文件及未知/空输入的行为回归。
- 不改变 L1/L2 测试依赖图与覆盖率策略，不削减检查规则或引入第二套 shell/Python 路径分类。

## 涉及分组与跨组依赖

- 涉及分组：Rust 测试 runner、Rust/CMake/native 质量命令、GitHub Actions。
- 不新增产品运行时依赖；同步已有测试选择器输出与本地、CI 消费端的契约。

## 前置条件与待决策

已读取并核对 caret 提交及 Panta 当前 selector、lint/format runner 和 CI workflow。其实现只能作为行为参考：须基于 Panta 的 `panta-*` packages、native label 与工具输入重建映射。

## 实施步骤

1. 比较 caret 优化与 Panta 当前选择器，明确适用的分类、边界和依赖映射。
2. 更新 Rust 规划输出及本地质量 runner，再同步 CI 输出与 job 条件。
3. 覆盖各独立检查域、未知/空路径、公共配置、runner 自身变更的选择器行为，并验证本地与 CI 选择一致。
4. 完成受影响检查及独立只读评审，记录证据。

## 预计改动

- `tests/src/test_scope.rs`、相关 `tests/src/lint.rs` / `tests/src/format.rs` 消费端。
- `.github/actions/changed-paths/action.yml`、`.github/workflows/ci.yml` 及必要的 selector 回归测试。
- 本任务和任务索引。

## 清理与兼容例外

移除不再使用的整体质量回退输出及其环境变量传递（若逐域安全回退完整验证）；无兼容例外。

## 验收标准

- [ ] Rust 测试、Clippy、格式、native lint、CMake lint、QML lint、分层检查和 runner 自测分别由同一规划结果选择。
- [ ] 各工具配置只扩大对应工具的扫描范围；未知路径、空 CI diff 和公共构建输入不会静默漏检。
- [ ] runner、workflow 与路径分类器变更运行 selector 自身回归，不因此触发无关的全量应用测试。
- [ ] 本地 `cargo verify-changed` 与 PR CI 对相同变更路径得出一致检查范围。
- [ ] 选择器行为测试、适用格式/lint/测试、CI workflow 校验及独立只读评审通过。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-25 | `cargo test --locked --package panta-tests --bin panta-tests`、`cargo verify-changed`、`actionlint .github/workflows/ci.yml`、`git diff --check` | selector/runner 单测、L1/L2 受影响验证、CI workflow 语法及 diff 检查通过；改动范围不误升全套测试 | runner 42 项中 39 项通过、3 项既有性能专属入口忽略；`cargo verify-changed` 按当前 7 个变更文件选择 `panta-tests` L1、无 L2/native/full fallback，格式、Clippy 与 runner 测试通过；actionlint 与 diff 检查通过。selector 自身测试覆盖空 CI 输入全量回退、本地空改动 no-op、各工具配置域、CMake 全量 native lint、当前 8 个 QML 注册头文件映射；本地与 `test-plan --ci` 对同一变更列表输出相同范围。PR CI 尚未运行。 |

## 风险与回退

选择器过度缩小会造成 CI 假绿。每种精细分类都须有正反向行为测试；路径无法确认时回退到完整相关域或完整测试，不保留宽泛的隐式跳过。

## 决策与工作记录

- 2026-10-25：根据 caret commit `e41f400` 新增本地跟进任务；确认需要按 Panta 当前 package、CTest label 和工具入口迁移，不能直接复制整文件。
- 2026-10-25：移植分域选择输出，接入本地 format/lint、CI action 与 job；独立评审发现 QML 注册头映射、CMake 全量 clang-tidy 与本地空变更处理的边界缺口，已修复并补回归；`gpt-6.1-sol` medium 复核更新 diff 未发现剩余问题。PR CI 尚待验证。

## 完成摘要

尚未完成。
