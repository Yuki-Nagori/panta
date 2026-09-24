# 048 — 性能基线与性能测试体系

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[032](032-cross-language-quality-gates.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-21 / 2026-09-25

## 目标与背景

维护者要求在正确性/覆盖率/sanitizer 门禁之外建立性能维度，并确认**性能工具不配置进 CI**：CI 只承担已有正确性门禁；性能基线、剖析与回归对比由开发侧命令和任务记录承载。本任务先摸清"慢"的真实来源（构建/测试入口耗时测量，本轮），再逐步落地 Rust 微基准、命令级对比与 QML/C++ 剖析流程。工具矩阵与命令登记见 [性能模块](../modules/performance.md)。

## 必读

- [质量工具链](../modules/quality-tooling.md)、[性能模块](../modules/performance.md)
- [验证与评审](../standards/validation-and-review.md)、[Rust 规范](../standards/rust.md)、[C++ 规范](../standards/cpp.md)

## 范围与非目标

范围：自有构建/测试入口的耗时测量与浪费消除；Criterion 微基准（panta-dsl-core 解析/格式化热路径、panta-core 任务状态机）；hyperfine 命令级对比（dslc CLI、编译选项）；cargo-flamegraph/perf 火焰图流程（Rust 与 C++，`-g` 调试符号）；QML Profiler（渲染/JS/信号/分配）与 Massif（堆峰值）的使用流程与适用边界；分离记录 VTK CPU 场景更新与真实窗口 GPU 帧呈现基准；基线数字与回归对比规范的登记。

非目标：不接入 CI 门禁；不剖析外部求解器进程内部（边界见 external-moldsolver.md）；不为测量而重构业务接口；不在无真实场景前给 Qt 渲染路径预设指标。

## 实施步骤

1. 测量现有入口耗时（开发环、native CTest、lint 各工具、sanitizer 矩阵），确认浪费并消除或注明原因。
2. Criterion 微基准：`benches/` 固定运行命令，产出可比较报告；基线数字登记进本任务验证表。
3. hyperfine 对比：dslc CLI 冷/热执行与编译选项差异各一条命令。
4. 火焰图流程：Rust（cargo-flamegraph）与 C++（perf record -g + 火焰图转换）各登记可复现命令；macOS 侧注明工具替代（Instruments）。
5. QML Profiler 与 Massif 流程：触发场景、采样开销说明（Massif 10–20x，仅开发机深排）。
6. 回归对比规范：基线环境、比较口径、升级时重测流程写入模块文档。
7. VTK CPU/GPU 测量分开维护：导航 CPU 目标只测几何与相机更新；GPU 目标通过 `VtkViewport` 在真实 WebGPU 原生窗口中触发场景更新并采样帧提交间隔。VTK 当前没有跨平台 GPU 完成/屏幕呈现时间戳，因此结果代表事件调度到 WebGPU 帧提交的端到端间隔，不宣称是纯 GPU 执行时间或显示器呈现时间。两个目标手动构建运行，均不进入默认构建或 CI。

## 预计改动

`crates/panta-dsl-core/benches/`、`crates/panta-core/benches/`（Cargo dev-dependency）、`tests/cpp/visualization/` VTK CPU/GPU 手动基准、native CMake 目标、`ai-docs/modules/performance.md`、runner/构建配置的效率修复、README、任务索引。

## 清理与兼容例外

无兼容例外。基准代码不进业务依赖图（Cargo bench 特性隔离）；被证伪的"优化"（无测量支撑的改动）不保留。

## 验收标准

- [ ] 入口耗时测量表进入任务记录；发现的浪费被消除或注明不处理原因。
- [ ] Criterion 基准本机可运行并产出可比较报告，基线数字已登记。
- [ ] 火焰图、hyperfine、QML Profiler、Massif 各有可复现命令与适用边界说明。
- [ ] performance 模块文档与 task、README、quality-tooling 引用一致。
- [x] VTK 导航 CPU 基准与真实窗口 WebGPU 帧提交基准是独立手动入口，输入与测量边界明确。
- [ ] CI 行为与本任务实施前一致（无新增门禁），有前后对比证据。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-21 | macOS arm64；增量开发环 `cargo build --locked -p panta-tests` | 0.04s（build.rs 无变化时不触发 native 调度），非瓶颈 |
| 2026-09-21 | macOS arm64；完整 native CTest（49 项，debug 树） | 9.66s 全过；sanitizer 树另测：asan-ubsan 约 20s、tsan 约 66s（TSan 运行时初始化为主，非代码缺陷） |
| 2026-09-21 | macOS arm64；`cargo lint cppcheck --check`（2.17.1 wheel，exhaustive） | 8.2s 墙钟（15.7s CPU，约 224%，内置并行）；EXIT=0；不加显式 `-j`，不改动已验证门禁 |
| 2026-09-21 | macOS arm64；`cargo lint clang-tidy --check`（暖缓存，分片并行） | 26.1s；EXIT=0 |
| 2026-09-21 | 业务热路径审查（panta-core 任务状态机、panta-dsl-core 解析） | 未发现代码级性能缺陷：状态/日志为 HashMap+环形队列，单元/行为测试毫秒级；无需修复项 |
| 2026-09-25 | macOS 26.3.1 arm64；CPU 目标手动运行，31×1000 次采样 | 通过；p50/p95 ns：相机插值 154.125/157.875、方向标记变化同步 2259.04/2358.42、无标记更新消融 4.958/5、稳定标记同步 92.5/94.584、六面命中 885.25/922.041、裁剪重置 1340.58/1391.75、无标记完整过渡 3223.58/3487.96、完整过渡 5665.92/5979.54 |
| 2026-09-25 | macOS 26.3.1 arm64、Qt 6.11.2、VTK 9.7.0；Cocoa 真实窗口 1000×700 逻辑像素；GPU 目标手动运行 | 30 帧预热、3×60 次更新；WebGPU 帧提交间隔 p50/p95 为 16.7598/18.1163 ms。真实窗口与 VTK context 初始化成功；这是场景更新至 VTK 帧提交日志的间隔，不代表 GPU kernel 或屏幕呈现时间 |
| 2026-09-25 | `cargo test --locked --workspace` | Rust、native 与 QML 聚合测试通过；CTest 58/58 |

测量结论：无病态慢环节，无代码性能修复项；"慢"的感受来自 sanitizer/覆盖率矩阵的固有检测成本（两棵插桩树、TSan 运行时初始化）与 CI job 数量增长，属门禁设计内的代价。CTest 保持串行（既有决策），不为 TSan 树的 66s 冒并行化回归风险。

## 风险与回退

风险是把测量噪声当回归或为数字做无意义重构；所有对比固定同一机器/同一命令，波动超过阈值才立项修复。回退只撤销本任务的配置与基准目录，不影响业务代码与 CI。

## 决策与工作记录

- 2026-09-21：维护者确认性能工具矩阵（Criterion、cargo-flamegraph、hyperfine、QML Profiler、perf+火焰图、Massif）只作开发侧工作台，不配置进 CI；"当前有点慢"先按入口测量定位，代码本身的性能问题优先修复。本轮完成入口耗时测量（结论：无病态慢环节、无代码性能修复项）、runner/panta-build 重复逻辑简化（PATH 前置收敛为 `panta_build::prepend_path`，ctest 调度收敛为 runner `run_ctest_in`，行为经两棵 sanitizer 树复验不变）；Criterion/火焰图等步骤 2–6 待后续增量。
- 2026-09-25：维护者确认 VTK 性能验证按 QML CPU/GPU 基准边界拆分。导航 CPU 与 GPU 帧提交基准分别置于 `native/visualization/CMakeLists.txt`、源码位于 `tests/cpp/visualization/`，均从默认构建排除且不注册 CTest。GPU 基准通过真实 Cocoa/WebGPU `VtkViewport` 场景更新采样；VTK 暂无可移植的 GPU 完成/屏幕呈现时间戳，结果不作为纯 GPU 执行时间或显示器帧时间。

## 完成摘要

未完成：入口耗时测量与浪费消除已交付；任务 069 另外建立了可扩展的手动 QML CPU/GPU 基准并完成面板消融；本次建立了独立的手动 VTK 导航 CPU 与真实 WebGPU 帧提交基准。任务 048 仍需 Criterion 可比较基线、hyperfine 与火焰图流程、QML Profiler/Massif 使用说明、性能文档引用收敛及 CI 行为前后对照。
