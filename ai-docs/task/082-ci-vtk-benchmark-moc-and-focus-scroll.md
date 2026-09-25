# 082 — CI 修复：VTK benchmark moc 前置与标题条焦点滚动

- 状态：in-progress
- 阶段：验证基础
- 依赖：[048 性能基线与性能测试体系](048-performance-testing.md)、[076 Cargo 测试与质量 runner 维护](076-panta-tests-runner-maintenance.md)、[029 QML 组件库](029-qml-component-library.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-25 / 2026-09-25

## 目标与背景

commit `3af54c2` 之后 CI main 出现 4 个失败 job（run [36037663181](https://github.com/Yuki-Nagori/panta/actions/runs/36037663181)）：

1. `cargo lint (clang-tidy)` 与 `cargo lint (includes)`：`tests/cpp/visualization/viewport_gpu_benchmark.cpp` 与 `viewport_navigation_cpu_benchmark.cpp` 在编译数据库中，但其 `#include "*.moc"` 文件不存在。两个新 VTK benchmark 目标（048）声明为 `EXCLUDE_FROM_ALL`，而 `tests/src/main.rs` 的 moc 前置步骤 `build_qml_benchmark_moc` 仍只构建 QML benchmark 的两个 autogen 目标，未覆盖新目标。
2. `cargo lint (cppcheck)`：`quality/cppcheck.json` 引用 `panta_viewport_navigation_cpu_benchmark_autogen/mocs_compilation.cpp`，该文件因目标未被构建而不存在，cppcheck 加载工程失败。与 076 修复过的问题同型，触发源换成新 VTK 目标。
3. `cargo check and build (macos-latest)`：`Qml.ShellModuleLoads::loads_shell_module` 在 `shell_module_load_test.cpp:286` 失败——把 "Activate Animation (A)" 按钮文案扩到 160 字符后，聚焦的搜索框未滚回 `titleStrip` 可视区。该 commit 未改 QML；失败暴露 `HorizontalToolStrip.ensureFocusedVisible` 的真实竞态：`contentWidth` 变化后仅用两轮 `Qt.callLater` 重滚，而 RowLayout 的 polish（移动 `titleSearchGroup`）可能发生在其后，滚动用了旧几何且不再重触发。Linux/Windows 时序恰好通过，macOS 必现。

完成后的可观察行为：CI main 三平台与全部 lint job 恢复绿色；文案增长后聚焦项在下一帧内被可靠滚回可视区，不依赖窗口 resize。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)
- [QML 性能基准登记规范](078-qml-performance-benchmark-policy.md)

## 范围与非目标

- 扩展 runner 的 moc 前置生成，覆盖所有 `EXCLUDE_FROM_ALL` 且含 Q_OBJECT 的基准目标（QML 两个 + VTK 两个），并同步函数命名与注释。
- 修复 `HorizontalToolStrip` 的重触发时机：内容子项几何变化（polish 完成后）再次调度 `ensureFocusedVisible`，不再只依赖 `contentWidth` 变化。
- `shell_module_load_test.cpp` 对异步滚动契约改用 QTRY 断言（286 行一处），保持契约本身不放松。
- 不新增 CTest 或 CI 时间门禁；不把手动 benchmark 目标改回默认构建；不调整 lint 工具检查范围。

## 前置条件与待决策

- 失败根因已由 run [36037663181](https://github.com/Yuki-Nagori/panta/actions/runs/36037663181) 日志证实（见上）。
- 修复后能否在 macOS CI 复现通过需真实 run 验证；本机（Windows）无法复现 macOS 时序，竞态修复以代码证据 + CI 证据共同支撑。

## 实施步骤

1. CMake：native 顶层新增空聚合目标 `panta_benchmark_moc`（BUILD_TESTING 下）；qml 与 visualization 两个基准定义处把各自的手动基准 `_autogen` 目标 `add_dependencies` 进聚合，登记点与基准声明同文件。
2. `tests/src/main.rs`：`build_qml_benchmark_moc` 更名为 `build_benchmark_moc`，只构建 `panta_benchmark_moc` 聚合，不再维护目标名单。
3. `qml/Components/Composites/HorizontalToolStrip.qml`：为 `contentRoot.childrenRect` 增加变化连接，polish 后重调度滚动；确认不与 contentX 写入形成回环。
4. `tests/cpp/app/shell_module_load_test.cpp`：文案增长后的搜索框可视断言改 QTRY，并保留几何诊断字段。
5. 本地验证 + 推送后核对 CI 全部 job。

## 预计改动

- `native/CMakeLists.txt`、`qml/CMakeLists.txt`、`native/visualization/CMakeLists.txt`（现存）
- `tests/src/main.rs`（现存）
- `qml/Components/Composites/HorizontalToolStrip.qml`（现存）
- `qml/Components/Composites/DialogTitleBar.qml`、`qml/Dialogs/NewProjectDialog.qml`、`qml/Dialogs/ImportDialog.qml`（现存）
- `tests/cpp/app/shell_module_load_test.cpp`（现存）
- `ai-docs/task-index.md` 与本任务

## 清理与兼容例外

`build_qml_benchmark_moc` 更名后旧名不再保留（无调用方之外的引用）；无兼容例外。

## 验收标准

- [ ] `cargo lint clang-tidy --check`、`cargo lint includes --check`、`cargo lint cppcheck --check` 在未构建 benchmark 主目标的工作树上通过。
- [ ] `cargo test --locked --workspace` 通过（含 `Qml.ShellModuleLoads`）。
- [ ] CI main push run 三平台 check、四个 lint、coverage、sanitizer 全部成功。
- [ ] 任务与索引状态一致，验证证据已记录。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待填写 | 待填写 | 未执行 |

## 风险与回退

- `childrenRect` 连接若引入滚动回环或额外帧开销，回退为仅 QTRY 化测试并在本任务记录为已知竞态；以 CTest 与 qmllint 结果判断无回环。
- runner 与 CMake 聚合目标改动影响所有 lint 入口；三项 lint 单项命令逐个本地验证后再推送。

## 决策与工作记录

- 2026-09-25：创建任务。4 个失败 job 根因定位：3 个 lint 失败源于 048 新增的 EXCLUDE_FROM_ALL VTK benchmark 目标未纳入 moc 前置生成；macOS 失败暴露 `HorizontalToolStrip` 焦点滚动在 polish 前读旧几何且不重触发的竞态（该 commit 未改 QML，属既有缺陷被时序暴露）。
- 2026-09-25：review 时把 autogen 目标名单从 runner 挪进 CMake（顶层空聚合 `panta_benchmark_moc` + 各基准定义处 `add_dependencies`）。理由：runner 硬编码名单正是本次漂移的成因，按“CMake 拥有 native 构建图”的分层规则，新增手动基准时的登记点应与基准声明同文件；runner 只保留聚合名。
- 2026-09-25：顺带清理 qmllint 的既有提示（Dialogs 的 unqualified access、DialogTitleBar 未用 import）；CI 门禁对 warning 级本不放行失败，属质量卫生，经维护者要求并入本任务。本地 format --check 首轮暴露手排 CMake 与 cmake-format 输出不一致，已由修复模式重写并以 --check 复验；cmake-lint 补 `add_custom_target` 的 COMMENT 并将下载函数收敛为 3 参数（R0913）。

## 完成摘要

未完成。
