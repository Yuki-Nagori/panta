# 058 — 项目包提交后的 CI 回归修复

- 状态：in-progress
- 阶段：验证基础
- 依赖：[057](057-new-project-dialog.md)、[032](032-cross-language-quality-gates.md)、[043](043-root-quality-runner.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

提交 `cd032d2` 引入项目包服务后，main CI 暴露三类回归：C++ 格式检查失败、Rust 覆盖率门禁因新增 `panta-core/src/project.rs` 未覆盖而下降、macOS sanitizer 下标题栏搜索框几何断言失败。修复后应恢复真实 main CI 的全绿，同时保留项目包行为和对应测试覆盖。

## 必读

- [任务 032：跨语言质量工具链与覆盖率门禁](032-cross-language-quality-gates.md)
- [任务 043：根目录质量入口与测试聚合](043-root-quality-runner.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

范围：补齐项目 service 的可达行为测试以满足现有 Rust 覆盖率门槛；修正标题栏在布局内容增长和窗口收窄时的焦点可见性/几何保证；格式化新增 C++ 测试文件并复跑相关门禁。

非目标：不降低覆盖率阈值，不排除项目 service 源文件，不改变项目包协议、用户可见命令或 sanitizer 矩阵，不顺手重构无关 QML。

## 实施步骤

1. 登记任务并从 main CI `35709245866` 的失败日志固定三个根因。
2. 为 `ProjectService` 补齐错误码、路径、清单、命令和失败清理路径的行为测试；仅在有明确不可达平台分支时记录排除理由。
3. 修复标题栏焦点滚动与内容宽度计算，使搜索框在 1440/640 及动态文案增长场景下保持可见。
4. 执行格式、Rust 覆盖率、项目测试、native 测试和 sanitizer 等实际存在的入口，并回填证据。

## 预计改动

- `crates/panta-core/src/project.rs`：补充项目服务行为测试。
- `qml/Panels/TopChromePanel.qml` 或对应 shell 布局：修复搜索焦点可见性。
- `tests/cpp/bridge/project_view_model_test.cpp`：clang-format 排版。
- 本任务与索引：记录根因、验证和最终状态。

## 清理与兼容例外

不引入兼容分支，不降低已有质量门禁；若替换布局逻辑，删除失效实现和重复调用点。

## 验收标准

- [ ] `cargo format --check` 通过。
- [ ] Rust coverage functions/lines 门禁在现有 89%/92% 阈值下通过。
- [ ] macOS sanitizer 的 `Qml.ShellModuleLoads` 通过，且不以跳过该测试规避回归。
- [ ] 项目包既有 Rust/native 测试继续通过，`git diff --check` 通过。
- [ ] 任务、索引与实际验证证据一致，无已知损坏状态。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-22 | GitHub Actions run `35709245866`（提交 `cd032d2`） | format、macOS sanitizer、Rust coverage 失败；其余 job 通过。 |
| 2026-09-22 | `cargo fmt --all`、`git diff --check` | 通过；C++ 格式、QML 焦点滚动和 Rust 覆盖率测试改动已整理。 |
| 2026-09-22 | `cargo build --locked --jobs 1 --verbose` | 阻塞于本机 `proc-macro2` build script 的 macOS 动态加载阶段，未进入项目测试；已停止进程并重建 Cargo global cache。 |
| — | Rust 项目测试、覆盖率、native/sanitizer 相关入口 | 待本地环境恢复后验证 |
| — | 修复提交后的 main CI | 待 push 后回填 |

## 决策与工作记录

- 2026-09-22：创建任务。CI 失败根因已由真实日志固定：`project_view_model_test.cpp:30` clang-format；`project.rs` 将 Rust coverage 降至 functions 84.06% / lines 90.28%；macOS `Qml.ShellModuleLoads` 的搜索框右边界断言失败。
- 2026-09-22：补齐 `ProjectService` 错误路径和清单边界测试，延迟两轮标题栏焦点可见性计算，并修正 C++ 测试格式；本地构建阻塞点确认在 `proc-macro2` build script 启动，而非项目代码或 Qt 依赖。

## 完成摘要

待完成。
