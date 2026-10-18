# 048 — 性能基线与性能测试体系

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-21 / 2026-10-18

## 目标与背景

维护者要求在正确性/覆盖率/sanitizer 门禁之外建立性能维度。2026-10-13 调整决策：性能数据仍不作为 PR 阻断门禁；定时 CI 可以运行 CPU 基准并上传结果，观察趋势而不因共享 runner 的噪声失败。GPU 基准保留手动入口，直到有稳定可识别的硬件 runner。性能基线、剖析与回归对比由开发命令、定时 artifact 和任务记录承载。本任务先摸清"慢"的真实来源（构建/测试入口耗时测量，本轮），再逐步落地 Rust 微基准、命令级对比与 QML/C++ 剖析流程。工具矩阵与命令登记见 [性能模块](../modules/performance.md)。

## 必读

- [质量工具链](../modules/quality-tooling.md)、[性能模块](../modules/performance.md)
- [验证与评审](../standards/validation-and-review.md)、[Rust 规范](../standards/rust.md)、[C++ 规范](../standards/cpp.md)

## 范围与非目标

范围：自有构建/测试入口的耗时测量与浪费消除；Criterion 微基准（panta-dsl-core 解析/格式化热路径、panta-core 工程页签状态转换）；通过托管工具链按需获取并 SHA256 校验 hyperfine，再对比 dslc CLI 的首次进程启动与预热重复执行，并比较实际存在的 Debug/Release 构建配置（CLI 没有可切换编译参数，不虚构编译选项）；cargo-flamegraph/perf 火焰图流程（Rust 与 C++，`-g` 调试符号）；QML Profiler（渲染/JS/信号/分配）与 Massif（堆峰值）的使用流程与适用边界；分离记录 VTK CPU 场景更新与真实窗口 GPU 帧呈现基准；基线数字与回归对比规范的登记。

非目标：不把性能阈值设为 PR 门禁；不在通用 CI runner 上运行需要可见窗口或专用硬件 GPU 的基准；不剖析外部求解器进程内部（边界见 external-moldsolver.md）；不为测量而重构业务接口；不在无真实场景前给 Qt 渲染路径预设指标。

## 实施步骤

1. 测量现有入口耗时（开发环、native CTest、lint 各工具、sanitizer 矩阵），确认浪费并消除或注明原因。
2. Criterion 微基准：固定运行命令，产出可比较报告；基线数字登记进本任务验证表。
3. hyperfine 对比：通过 `cargo hyperfine` 按需安装固定版本的官方预编译包（平台 SHA256 校验），对比 dslc CLI 首次进程启动与预热执行，并比较仓库当前支持的 Debug/Release profile。dslc 不提供独立编译参数，避免编造无效对比。
4. 火焰图流程：Rust（cargo-flamegraph）与 C++（perf record -g + perf report）各登记可复现命令；macOS/Windows 侧注明可用采样器。
5. QML Profiler 与 Massif 流程：提供调试构建前提、固定触发场景与采样开销说明（Massif 10–20x，仅开发机深排）。
6. 回归对比规范：基线环境、比较口径、升级时重测流程写入模块文档。
7. VTK CPU/GPU 测量分开维护：导航 CPU 目标只测几何与相机更新；GPU 目标通过 `VtkViewport` 在真实 WebGPU 原生窗口中触发场景更新并采样帧提交间隔。VTK 当前没有跨平台 GPU 完成/屏幕呈现时间戳，因此结果代表事件调度到 WebGPU 帧提交的端到端间隔，不宣称是纯 GPU 执行时间或显示器呈现时间。两类目标均不进入默认构建或正确性门禁；纯 CPU 套件可由定时 CI 运行并上传 artifact，GPU 目标保留手动入口。

## 预计改动

`tests/performance/cpu/` Criterion 与手动基准、领域 crate Cargo bench 注册与开发依赖、`panta-build` 的 hyperfine 官方资产与按需托管入口、Cargo 命令别名、VTK CPU/GPU 手动基准、native CMake 目标、`ai-docs/modules/performance.md`、依赖获取清单、runner/构建配置的效率修复、README、任务索引。

## 清理与兼容例外

无兼容例外。基准代码不进业务依赖图（Cargo bench 特性隔离）；被证伪的"优化"（无测量支撑的改动）不保留。

## 验收标准

- [x] 入口耗时测量表进入任务记录；发现的浪费被消除或注明不处理原因。
- [x] Criterion 基准本机可运行并产出可比较报告，基线数字已登记。
- [x] 火焰图、hyperfine、QML Profiler、Massif 各有可复现命令与适用边界说明。
- [x] performance 模块文档与 task、README、quality-tooling 引用一致。
- [x] VTK 导航 CPU 基准与真实窗口 WebGPU 帧提交基准是独立手动入口，输入与测量边界明确。
- [x] 每日定时 CI 运行稳定的 CPU 基准并上传日志 artifact；不把测量波动设为门禁，也不运行依赖真实窗口或硬件 GPU 的项目。
- [x] CI 行为变更有前后对比证据，开发者手动性能入口仍可运行。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-10-18 | macOS 26.3.1 arm64、rustc/cargo 1.98.1；Criterion benches，Release/bench profile | `CRITERION_HOME="$PWD/target/criterion" cargo bench --locked -p panta-dsl-core --bench dsl_core_criterion -- --save-baseline 2026-10-18-macos-arm64` 与对应 `panta-core` 命令通过，产出 workspace `target/criterion/` 统计、命名基线和 HTML 报告。基线数值及异常样本限制见[性能模块](../modules/performance.md#rust-微基准与命令对比)。 |
| 2026-10-18 | macOS 26.3.1 arm64；`cargo hyperfine` 2.0.0 Debug/Release dslc CLI 对比 | 首次调用从官方 arm64 资产安装并通过 SHA256 校验。`cargo hyperfine --runs 15 --warmup 0` 显示首次进程受文件缓存影响（最大 421.9 ms / 358.4 ms）；暖缓存 `--runs 30 --warmup 5` 平均 Wall Time 为 Debug 1.7 ms、Release 1.5 ms，峰值内存估算 2.8 / 1.7 MiB。说明首轮不能代表稳定 CLI 延迟；hyperfine 2.0 默认报告墙钟与内存。 |
| 2026-10-18 | macOS 26.3.1 arm64；`cargo verify-changed` | 变更含 workspace manifest/lockfile，选择器按规则执行完整质量和测试范围；Rust Clippy、依赖审计、CMake 格式与分层、native build、qmllint、clang-tidy、include-cleaner、Cppcheck 均通过；Rust workspace 测试通过；native/QML CTest 77/77 通过。 |
| 2026-10-18 | GitHub Actions push CI 前后对照 | 调整前 run [38321157525](https://github.com/Yuki-Nagori/panta/actions/runs/38321157525) 含 Miri 与三平台 sanitizer jobs，关键路径从首个 job `23:15:22Z` 到最后完成 `23:24:46Z`，约 9 分 24 秒；调整后首个 push run [38407816374](https://github.com/Yuki-Nagori/panta/actions/runs/38407816374) 不再运行这些 jobs，首末 job `13:47:14Z`–`13:54:57Z`，约 7 分 43 秒，缩短约 1 分 41 秒（18%）。两次运行提交内容与共享 runner 状态不同，该数据只说明实际 job 范围和关键路径变化，不作为严格性能基准。Miri / sanitizer 已转到 nightly；nightly run [39063928969](https://github.com/Yuki-Nagori/panta/actions/runs/39063928969) 成功，CPU 性能 job 约 2 分 51 秒并上传日志 artifact。 |
| 2026-10-18 | macOS arm64；`cargo test --locked -p panta-tests --bin panta-tests`、`cargo fmt --all --check`、`git diff --check` | runner 单测 24 项通过、3 项按设计忽略；Criterion 路径选择器回归确认两个 bench 精确映射到 `panta-dsl-core` / `panta-core`，未知 Rust 性能源仍保守回退。格式与 diff 检查通过。`gpt-6.1-sol` medium 独立只读 final review 未发现剩余问题；Linux/Windows hyperfine 实机及 Linux profiling 未验证。 |
| 2026-10-18 | macOS arm64；`actionlint .github/workflows/nightly.yml`、nightly Hyperfine 命令 smoke | workflow 静态检查通过；本机 Debug/Release `dslc` 构建成功，`cargo hyperfine --runs 3 --warmup 1 --export-json ...` 成功并生成非空 JSON。独立 final review 通过；nightly 使用 30 runs / 5 warmups，尚待 PR CI 和首次 Linux nightly 运行验证。 |
| 2026-10-18 | macOS 26.3.1 arm64；`cargo performance cpu --samples 31` | 辅助代码行为测试 3/3 与 Qt benchmark-support 行为测试 7/7 通过；QML CPU benchmark 7/7、VTK 导航 CPU benchmark 3/3 通过；工程 STL 打开、激活、缓存切换、重载、关闭测量完成。链接时出现已有 Rust 静态对象目标版本 26.2 高于 native deployment 13.0 的警告，基准正常执行。 |
| 2026-10-17 | GitHub Actions nightly `CPU performance sample (ubuntu)`；`cargo performance cpu --samples 31`；run [39063928969](https://github.com/Yuki-Nagori/panta/actions/runs/39063928969) | Job 成功，上传 `cpu-performance-39063928969-1` artifact。日志确认辅助代码测试 3/3、Mesh/IR 测量测试 4/4、QML CPU benchmark 7/7、VTK navigation CPU benchmark 3/3 通过；含 50k / 500k 三角面后台预检负载、sidecar 压缩/解码、QML 构造与图标加载、VTK 导航消融，以及 31 轮工程打开/激活/缓存切换/关闭测量。该 run 首次确认定时 CPU 性能 job 与日志 artifact 均实际工作；共享 Ubuntu runner 数字作为趋势记录，不设阈值。 |
| 2026-10-13 | `actionlint .github/workflows/ci.yml .github/workflows/nightly.yml`、`git diff --check` | 配置静态检查通过。PR/push workflow 移除 Miri 与 sanitizer；每日 workflow 安排 Miri、三平台 sanitizer 和 Ubuntu CPU 性能套件，并上传带提交与 runner 信息的日志。当时实际定时运行与 artifact 尚未验证，后由 2026-10-17 nightly 记录补齐。 |
| 2026-09-21 | macOS arm64；增量开发环 `cargo build --locked -p panta-tests` | 0.04s（build.rs 无变化时不触发 native 调度），非瓶颈 |
| 2026-09-21 | macOS arm64；完整 native CTest（49 项，debug 树） | 9.66s 全过；sanitizer 树另测：asan-ubsan 约 20s、tsan 约 66s（TSan 运行时初始化为主，非代码缺陷） |
| 2026-09-21 | macOS arm64；`cargo lint cppcheck --check`（2.17.1 wheel，exhaustive） | 8.2s 墙钟（15.7s CPU，约 224%，内置并行）；EXIT=0；不加显式 `-j`，不改动已验证门禁 |
| 2026-09-21 | macOS arm64；`cargo lint clang-tidy --check`（暖缓存，分片并行） | 26.1s；EXIT=0 |
| 2026-09-21 | 业务热路径审查（panta-core 任务状态机、panta-dsl-core 解析） | 未发现代码级性能缺陷：状态/日志为 HashMap+环形队列，单元/行为测试毫秒级；无需修复项 |
| 2026-09-25 | macOS 26.3.1 arm64；CPU 目标手动运行，31×1000 次采样 | 通过；p50/p95 ns：相机插值 154.125/157.875、方向标记变化同步 2259.04/2358.42、无标记更新消融 4.958/5、稳定标记同步 92.5/94.584、六面命中 885.25/922.041、裁剪重置 1340.58/1391.75、无标记完整过渡 3223.58/3487.96、完整过渡 5665.92/5979.54 |
| 2026-09-25 | macOS 26.3.1 arm64、Qt 6.11.2、VTK 9.7.0；Cocoa 真实窗口 1000×700 逻辑像素；GPU 目标手动运行 | 30 帧预热、3×60 次更新；WebGPU 帧提交间隔 p50/p95 为 16.7598/18.1163 ms。真实窗口与 VTK context 初始化成功；这是场景更新至 VTK 帧提交日志的间隔，不代表 GPU kernel 或屏幕呈现时间 |
| 2026-09-26 | Windows 11 x64、Qt 6.11.2（MSVC 2022 ABI）、VTK 9.7.0、Debug；导航 CPU 目标手动运行，31×1000 次采样 | 通过；p50/p95 ns：相机插值 92.1/92.5、方向标记变化同步 3190.3/3423.8、无标记更新消融 4.4/4.5、稳定标记同步 84/88.1、六面命中 1296.4/1352、裁剪重置 1787.8/1878.8、无标记完整过渡 4378.5/4581.3、完整过渡 7844.6/8982.4 |
| 2026-09-26 | Windows 11 x64、Qt 6.11.2、Debug；QML CPU 构造目标手动运行（docks×场景×条目数矩阵，31 样本） | 通过；p50/p95 µs：empty 恒定约 2.8-2.9/3-3.5；tasks 约 3180-3392/3670-4115；layers 约 1130-1223/1425-1773；both 约 4352-4501/5456-6065；条目数 0→1000 对构造耗时影响平缓（模型行数增长不显著抬升 p50） |
| 2026-09-26 | Windows 11 x64、Qt 6.11.2、Debug；QML GPU 目标手动运行（Direct3D11 RHI，真实窗口，3×60 帧） | 通过；全部 13 个场景组合的帧呈现间隔 p50/p95 均落在约 5.52-5.57/5.85-6.16 ms，面板组合与导入条目数变化不改变帧间隔（vsync/合成器主导）；该数字为端到端帧间隔，非 GPU kernel 时间 |
| 2026-09-26 | Windows 11 x64、Qt 6.11.2、VTK 9.7.0、Debug；真实窗口 WebGPU 帧提交基准手动运行 | 测量通过：30 帧预热、3×60 次更新，帧提交间隔 p50/p95 = 3.037/3.8864 ms（明显快于 macOS Cocoa 的 16.76/18.12 ms，窗口机制与合成路径不同，不作跨平台结论）。缺陷：全部用例通过后进程在退出清理阶段 0xC0000005 崩溃（WebGPU/Dawn 释放顺序），登记于任务 007 资源释放验收项 |
| 2026-09-25 | `cargo test --locked --workspace` | Rust、native 与 QML 聚合测试通过；CTest 58/58 |

测量结论：无病态慢环节，无代码性能修复项；"慢"的感受来自 sanitizer/覆盖率矩阵的固有检测成本（两棵插桩树、TSan 运行时初始化）与 CI job 数量增长，属门禁设计内的代价。CTest 保持串行（既有决策），不为 TSan 树的 66s 冒并行化回归风险。

## 风险与回退

风险是把测量噪声当回归或为数字做无意义重构；所有对比固定同一机器/同一命令，波动超过阈值才立项修复。回退只撤销本任务的配置与基准目录，不影响业务代码与 CI。

## 决策与工作记录

- 2026-09-21：维护者确认性能工具矩阵（Criterion、cargo-flamegraph、hyperfine、QML Profiler、perf+火焰图、Massif）只作开发侧工作台，不配置进 CI；"当前有点慢"先按入口测量定位，代码本身的性能问题优先修复。本轮完成入口耗时测量（结论：无病态慢环节、无代码性能修复项）、runner/panta-build 重复逻辑简化（PATH 前置收敛为 `panta_build::prepend_path`，ctest 调度收敛为 runner `run_ctest_in`，行为经两棵 sanitizer 树复验不变）；Criterion/火焰图等步骤 2–6 待后续增量。
- 2026-09-25：维护者确认 VTK 性能验证按 QML CPU/GPU 基准边界拆分。导航 CPU 与 GPU 帧提交基准分别置于 `native/visualization/CMakeLists.txt`、源码位于 `tests/cpp/visualization/`，均从默认构建排除且不注册 CTest。GPU 基准通过真实 Cocoa/WebGPU `VtkViewport` 场景更新采样；VTK 暂无可移植的 GPU 完成/屏幕呈现时间戳，结果不作为纯 GPU 执行时间或显示器帧时间。
- 2026-10-13：维护者调整 CI 决策：性能数据可进入每日定时 workflow，但只作趋势 artifact、不阻断 PR；通用 runner 上先运行 CPU 套件，GPU 套件因硬件身份和图形会话要求暂留手动执行。高成本 Miri 与跨平台 sanitizer 从每次变更检查改为每日定时检查，常规 PR 保留构建、测试、覆盖率和 lint。
- 2026-10-18：依赖复核。任务 032 当前为 deferred，内容集中在测试质量门禁与覆盖率策略；性能基准本身不依赖其未完成项，因此从 task 048 依赖中移除 032，仅保留统一测试入口 011。nightly run 39063928969 成功运行 CPU 性能套件并上传 artifact，完成此前“实际定时运行与 artifact 待验收”项。
- 2026-10-18：新增 Criterion 0.8.2 开发依赖与 DSL / 工程页签状态微基准；本机两套基准均成功构建和运行。命令级对比按实际存在的 dslc Debug/Release profile 定义，不为 CLI 虚构编译开关。火焰图、QML Profiler、Massif 的工具命令、前置条件和结果边界回填至性能模块文档。
- 2026-10-18：最终 review 修正 Criterion 基准估计值口径、Linux flamegraph 链接器参数传递，并为新增 Rust bench 路径补充分层选择器映射与回归测试；独立评审确认无剩余问题。
- 2026-10-18：通过 `gh` 核实 CI 拆分前后运行。Push 关键路径由约 9:24 降至 7:43；Miri / sanitizer 在 nightly 成功运行，CPU 性能 job 已产出 artifact。由于运行提交与共享 runner 条件不同，记录为行为范围和时长参考，不作为受控性能结论。
- 2026-10-18：维护者确认将 hyperfine CLI 对比加入 nightly CPU 性能 artifact，不设性能阈值。workflow 现在构建 `dslc` Debug/Release，记录 30 轮、5 次预热的输出与 JSON；本机 actionlint 和短跑 smoke 通过，等待 PR CI。
- 2026-10-18：独立 final review 复核 nightly workflow 更新，通过；确认管道失败会传播、JSON 与日志一并上传，未把性能阈值引入 PR 检查。Ubuntu 实际执行仍待 PR 合并后的 nightly。

## 完成摘要

已完成本机实现与验收：入口耗时测量与性能浪费审查；Criterion DSL/core 可比较微基准与本机基线；hyperfine 2.0.0 三平台资产摘要登记、`cargo hyperfine` 按需安装及 macOS 实测；Rust/C++ profiling、QML Profiler、Massif 命令及边界；VTK CPU/GPU 分离；nightly CPU 性能 artifact 与 CI 拆分前后行为已核实；当前工作树聚合 CPU 性能入口、完整质量验证及独立评审通过。当前代码分支的 CI 尚未运行；性能命令仍不作为 PR 阻断门禁。
