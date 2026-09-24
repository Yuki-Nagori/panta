# 058 — 项目包提交后的 CI 回归修复

- 状态：done
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

范围：补齐项目 service 的可达行为测试以满足现有 Rust 覆盖率门槛；修正标题栏在布局内容增长和窗口收窄时的焦点可见性/几何保证；格式化新增 C++ 测试文件、CMake 和 QML，并复跑相关门禁。

非目标：不降低覆盖率阈值，不排除项目 service 源文件，不改变项目包协议、用户可见命令或 sanitizer 矩阵，不顺手重构无关 QML。

## 实施步骤

1. 登记任务并从 main CI `35709245866` 的失败日志固定三个根因。
2. 为 `ProjectService` 补齐错误码、路径、清单、命令和失败清理路径的行为测试；仅在有明确不可达平台分支时记录排除理由。
3. 修复标题栏焦点滚动与内容宽度计算，使搜索框在 1440/640 及动态文案增长场景下保持可见。
4. 执行格式、Rust 覆盖率、项目测试、native 测试和 sanitizer 等实际存在的入口，并回填证据。

## 预计改动

- `crates/panta-core/src/project.rs`：补充项目服务行为测试。
- `crates/panta-ffi/src/lib.rs`：补充项目 service FFI 桥接快照、错误和保存/打开路径测试。
- `qml/Panels/TopChromePanel.qml` 或对应 shell 布局：修复搜索焦点可见性。
- `tests/cpp/bridge/project_view_model_test.cpp`：clang-format 排版。
- `tests/cpp/i18n/qm_load_test.cpp`、`native/CMakeLists.txt`、`native/app/CMakeLists.txt`：按仓库使用的格式工具修正排版。
- 本任务与索引：记录根因、验证和最终状态。

## 清理与兼容例外

不引入兼容分支，不降低已有质量门禁；若替换布局逻辑，删除失效实现和重复调用点。

## 验收标准

- [x] `cargo format --check` 通过。
- [x] Rust coverage functions/lines 门禁在现有 89%/92% 阈值下通过。
- [x] macOS sanitizer 的 `Qml.ShellModuleLoads` 通过，且不以跳过该测试规避回归。
- [x] 项目包既有 Rust/native 测试继续通过，`git diff --check` 通过。
- [x] 任务、索引与实际验证证据一致，无已知损坏状态。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-22 | GitHub Actions run `35709245866`（提交 `cd032d2`） | format、macOS sanitizer、Rust coverage 失败；其余 job 通过。 |
| 2026-09-22 | `cargo format --check`、`git diff --check` | 通过；Rust、C++、CMake、QML 聚合格式检查通过。 |
| 2026-09-22 | `cargo build --locked --jobs 1 --verbose` | 阻塞于本机 `proc-macro2` build script 的 macOS 动态加载阶段，未进入项目测试；已停止进程并重建 Cargo global cache。 |
| 2026-09-22 | `cargo coverage` | 通过；functions `89.46%`、lines `94.21%`，高于 `89%` / `92%` 门禁。 |
| 2026-09-22 | `cargo check --locked --workspace --all-targets`、`cargo build --locked --workspace`、`cargo run --locked --package panta-tests -- toolchain` | 通过；复用现有 `target/` 中的 LLVM、CMake、Ninja 和 Qt 工具链。 |
| 2026-09-22 | `cargo test --locked --workspace` | 通过；native 54/54、QML 格式和模块加载测试，以及 Rust/doc tests 全部通过。首次运行的 `Visualization.DefaultWordmark` 超时在重跑中通过，未修改超时配置。 |
| 2026-09-22 | GitHub Actions run `35712220845` | 该次仍对应修复提交前状态，失败项为 QML/C++/CMake 格式和覆盖率；上述本地修复已覆盖这些失败原因，待新提交触发 CI 后回填。 |
| 2026-09-24 | GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4` | 全绿：聚合格式和 Rust coverage gate 通过；macOS sanitizer 中 `Qml.ShellModuleLoads` 53/56 Passed（未跳过），macOS/Linux/Windows Cargo tests 通过，Linux native CTest 56/56、Windows CTest 54/54。CI run success，关闭回归任务。 |

## 决策与工作记录

- 2026-09-22：创建任务。CI 失败根因已由真实日志固定：`project_view_model_test.cpp:30` clang-format；`project.rs` 将 Rust coverage 降至 functions 84.06% / lines 90.28%；macOS `Qml.ShellModuleLoads` 的搜索框右边界断言失败。
- 2026-09-22：补齐 `ProjectService` 错误路径和清单边界测试，延迟两轮标题栏焦点可见性计算，并修正 C++ 测试格式；本地构建阻塞点确认在 `proc-macro2` build script 启动，而非项目代码或 Qt 依赖。
- 2026-09-22：补充 `panta-ffi` 项目 service 桥接测试，使覆盖率恢复至门禁以上；按仓库实际 formatter 修正 QML、CMake 和 C++ 排版。
- 2026-09-22：本地完整验证通过；覆盖率使用已有 `target/llvm-cov-target`，未新增 LLVM target 目录。
- 2026-09-24：run 36001859191 全绿；Rust coverage 与聚合格式门禁通过，macOS sanitizer 的 `Qml.ShellModuleLoads` 未跳过且通过，关闭本任务。

## 完成摘要

三类 CI 回归均已修复并由 run 36001859191 复验：格式检查与 Rust 覆盖率门禁通过，macOS sanitizer 的 `Qml.ShellModuleLoads` 通过，项目包 Rust/native 测试在三平台继续通过。覆盖率阈值未降低，测试未被跳过。
