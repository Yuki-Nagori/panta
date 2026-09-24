# 069 — 工程 / Tasks / Layers 面板整体 Review 与性能消融

- 状态：done
- 阶段：应用平台扩展
- 依赖：[068](068-qml-project-and-layers-docks.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

对任务 068 的工程树、Tasks 面板、Layers 面板及 ProjectViewModel 适配路径做整体代码 review，修复有证据支持的问题并精简失效或误导性注释。用开发侧、可复现的 Qt/QML 性能基准比较 0、1 和多个导入零件场景，并对工程树、Layers 页签行和两者同时启用做消融。性能结果用于判断热点，不设 CI 门槛。

## 必读

- [任务 068](068-qml-project-and-layers-docks.md)
- [Qt / 原生领域边界](../architecture/native-domain-boundaries.md)
- [QML 规范](../standards/qml.md)、[C++ 规范](../standards/cpp.md)、[注释规范](../standards/comments.md)
- [性能测试任务](048-performance-testing.md)、[性能模块](../modules/performance.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

包含：审阅并导入列表、工程树、Tasks / Layers Dock 可见状态有关的 QML、ViewModel 和测试代码；仅在 review 或测量证明存在缺陷/开销后优化；把 `tests/qml/project_docks_cpu_benchmark.cpp` 建成仓库通用的 CPU QML 性能入口，并提供独立 GPU 帧呈现入口 `tests/qml/project_docks_gpu_benchmark.cpp`。两个入口都用可扩展的场景组，后续持续补充其他 QML 场景。

不包含：更改工程 / STL 领域语义、图层业务行为、其他 Dock 或整体架构；CI 性能阈值；为追求微小数字增加复杂缓存或领域状态副本。

## 前置条件与待决策

- 任务 068 已实现且相关工作树可构建。
- CPU 基准使用离屏 Qt Test / QML runtime，测对象和模型委托构造 / 更新；GPU 基准只在真实图形窗口中运行，采样 Qt Quick 帧呈现间隔。公开跨平台接口测得的是端到端帧时间（受合成器 / 垂直同步影响），不称为 GPU 内核纯耗时。两者都不是 CI 门禁。
- 采样采用预热后重复批次并报告中位数及 p95；比较组除被消融部件外保持一致。若现有 QML harness 无法隔离组件成本，应报告限制，不制造虚假的“纯 QML”结论。

## 实施步骤

1. 检查相关 QML / C++ / 测试实现及注释，记录可复现的问题与所有权 / 更新频率 / 控件生命周期风险。
2. 把 CPU / GPU 基准各自的场景注册、预热、采样、分位数汇总和结果输出抽成可复用入口；以相同输入数量和操作构造当前面板消融场景。
3. 在同一构建与设备下重复采样，比较 0、1、100、1000 项输入和可分离的组件成本；先测后改。
4. 只实施有代码证据或测量支持的优化，删除过时、重复或误导的注释；重跑功能测试与基准。
5. 记录真实环境、工具版本、采样数据、结论和仍未覆盖的限制。

## 预计改动

任务索引及本文件；可能修改 `qml/Panels/TasksPanel.qml`、`qml/Panels/LayersPanel.qml`、`qml/App.qml`、`native/bridge/src/project_view_model.*` 和相关测试；新增手动性能基准源文件及最小 CMake target。以实际 review 结果为准，不预设业务代码必须变更。

## 清理与兼容例外

无预设废弃项或兼容例外。若优化后产生未引用属性、测试 helper 或注释，应同步删除。

## 验收标准

- [x] Review 覆盖 QML 状态绑定、导入列表更新、ViewModel 边界、错误显示以及新增测试，不留未处置的高风险问题。
- [x] 注释符合仓库注释规范，只保留解释约束 / 原因 / 所有权的注释。
- [x] CPU / GPU 两个入口均可手动运行；CPU 提供 0、1、100、1000 个零件的构造结果，GPU 在本机硬件窗口提供对应帧呈现结果，并比较工程树 / Layers / 组合消融。
- [x] 两个性能入口都不注册进 CTest，且设为 `EXCLUDE_FROM_ALL`，常规构建和 CI 不会构建或运行它们；GPU 基准在本机 Metal 图形会话完成。
- [x] 优化前后功能行为由既有正确性测试覆盖；性能结论基于同机、同 Qt 版本、同输入的重复测量，帧数据受垂直同步影响并如实注明。
- [x] 适用的 Cargo 聚合质量入口通过，task 与索引记录一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | Review：Tasks / Layers QML、ProjectViewModel、现有导入与 Dock 测试 | 检查状态、所有权、生命周期和更新成本 | 发现 `Repeater` 为所有导入项创建 delegate；1000 项时 CPU 构造 p50 约 51 ms。改用可复用 `ListView` 后 1000 条模型数据只实例化可见 delegate。补显式滚动条对象、`ComponentBehavior: Bound`；移除 Layers 未使用的 Controls import，并补齐 C++ / Qt 直接头文件。 |
| 2026-09-24 | `cargo build --locked`、`cargo format --check`、`cargo lint --check` | 构建、格式与八阶段静态检查通过 | 全部通过；lint 包含 Clippy、cargo-machete、CMake、native/QML metadata、qmllint、Clang-Tidy、include-cleaner、Cppcheck。 |
| 2026-09-24 | `cargo test --locked --workspace` | Rust 与 native/QML 行为回归通过 | 通过：Rust workspace 测试通过，CTest 56/56；`Qml.ShellModuleLoads` 覆盖导入 1/2 个 STL、空工程与重开工程后的 Tasks / Layers 可见状态。 |
| 2026-09-24 | 手动构建：`cmake --build target/native/debug --config Debug --target panta_qml_cpu_benchmark panta_qml_gpu_benchmark` | 显式构建被排除在默认目标外的 benchmark | 两个目标编译通过；`EXCLUDE_FROM_ALL` 保证普通构建与 CI 不编译这两个手动工具，目标不注册为 CTest。 |
| 2026-09-24 | CPU 手动基准：`QT_QPA_PLATFORM=offscreen target/native/debug/qml/panta_qml_cpu_benchmark` | 0 / 1 / 100 / 1000 个导入项，Tasks / Layers / 两者，预热后 31 次采样 | 通过。环境：macOS 26.3.1 arm64、Qt 6.11.2；完整 p50/p95 见下表。离屏窗口仅用于构造 / delegate CPU 统计。 |
| 2026-09-24 | GPU 手动基准：`target/native/debug/qml/panta_qml_gpu_benchmark`，经临时 app wrapper 在 computer use 图形会话启动 | 真实窗口预热 30 帧，每场景 3 × 60 帧；Tasks / Layers / 两者和 0 / 1 / 100 / 1000 项 | 通过：窗口由 computer use 绑定并检查，Qt Quick renderer 为 Metal；完整帧间隔见下表。所有场景 p50 约 16.65–16.70 ms，接近 60 Hz 垂直同步，p95 波动未显示可区分的组件成本；数据为端到端帧间隔，不代表 GPU 内核时间。临时 wrapper 与日志已清理。 |

### CPU 构造消融

单位为毫秒，格式为 p50 / p95。修改前与修改后在同一机器、Qt 版本、面板尺寸和 benchmark harness 下采样；修改前 Tasks 使用完整 `Repeater`，修改后使用虚拟化 `ListView`。

| 面板 | 导入项 | 修改前 p50 / p95 | 修改后 p50 / p95 |
|---|---:|---:|---:|
| Tasks | 0 | 1.479 / 1.910 | 1.492 / 1.779 |
| Tasks | 1 | 1.526 / 1.831 | 1.483 / 1.807 |
| Tasks | 100 | 6.224 / 8.274 | 1.506 / 1.989 |
| Tasks | 1000 | 51.016 / 57.054 | 1.479 / 2.221 |
| Layers | 0 | 0.640 / 0.897 | 0.618 / 0.966 |
| Layers | 1 | 0.607 / 1.069 | 0.615 / 0.895 |
| Layers | 100 | 0.610 / 1.095 | 0.630 / 0.885 |
| Layers | 1000 | 0.619 / 1.101 | 0.619 / 0.916 |
| 两者 | 0 | 1.995 / 2.549 | 2.089 / 2.599 |
| 两者 | 1 | 2.127 / 2.692 | 2.128 / 2.458 |
| 两者 | 100 | 6.129 / 6.831 | 2.204 / 2.820 |
| 两者 | 1000 | 51.102 / 52.004 | 2.157 / 2.600 |

1000 项 Tasks 的 p50 从 51.016 ms 降至 1.479 ms（约 34.5 倍）；组合场景从 51.102 ms 降至 2.157 ms（约 23.7 倍）。0 / 1 项场景基本持平，收益集中于长导入列表，未引入缓存或第二份领域状态。

### GPU 帧呈现消融

单位为毫秒 / 帧，格式为 p50 / p95。窗口为 macOS 原生图形会话中的 QQuickWindow，渲染后端 Metal；测量包含合成器和垂直同步。

| 场景 | 导入项 | p50 / p95 |
|---|---:|---:|
| 空窗口 | 0 | 16.668 / 17.858 |
| Tasks | 0 | 16.676 / 17.233 |
| Layers | 0 | 16.677 / 17.210 |
| 两者 | 0 | 16.667 / 18.416 |
| Tasks | 1 | 16.696 / 17.700 |
| Layers | 1 | 16.662 / 17.766 |
| 两者 | 1 | 16.669 / 17.839 |
| Tasks | 100 | 16.675 / 17.635 |
| Layers | 100 | 16.641 / 17.579 |
| 两者 | 100 | 16.665 / 17.468 |
| Tasks | 1000 | 16.647 / 17.589 |
| Layers | 1000 | 16.683 / 17.651 |
| 两者 | 1000 | 16.651 / 17.795 |

## 风险与回退

基准可能被离屏组件生命周期或平台渲染差异影响，因此只把声明清楚的 CPU 更新路径用于比较，不跨机器比较绝对时间。若优化影响更新语义或正确性，回退相应局部代码，保留原功能测试和测量工具。

## 决策与工作记录

- 2026-09-24：创建任务；遵循用户说明，自动化测试与基准用于常规验证，computer use 留给具体行为问题的现场排查。

## 完成摘要

Review 修复了长导入列表的 delegate 全量创建，并建立独立、手动运行的 CPU / GPU QML 基准入口。性能数据表明大列表 CPU 构造是主要可优化成本；GPU 帧间隔受 60 Hz 垂直同步限制，本次没有发现需要针对 Layers 或组合面板继续优化的可测 GPU 热点。
