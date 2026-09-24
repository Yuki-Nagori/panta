# 性能测试与剖析

[模块导航](README.md) · [实施任务 048](../task/048-performance-testing.md) · [质量工具链](quality-tooling.md)

## 定位与边界

性能工具是**开发侧工作台，不配置进 CI 门禁**（维护者决策，2026-09-21）：CI 只承担格式、lint、审计、测试、覆盖率与 sanitizer 矩阵等正确性门禁；性能基线、剖析和回归对比由本模块登记的命令与任务记录承载。测量优先于修改——没有测量支撑的"优化"不保留；对比必须固定同一机器、同一命令与同一构建配置，波动显著才立项修复。

## 工具矩阵

| 层 | 工具 | 用途 | 适用场景 |
|---|---|---|---|
| Rust | Criterion.rs | 函数级微基准，统计分析 + 报告，可检测性能回归 | 量化核心逻辑（`.pa` 解析、任务状态机）的长期变化 |
| Rust | cargo-flamegraph（底层 perf） | CPU 火焰图，展示调用栈耗时分布 | 定位 CPU 瓶颈的首选入口 |
| Rust/CLI | hyperfine | 命令级基准与统计对比 | 对比 dslc 启动/执行时间、编译选项或参数差异 |
| QML | QML Profiler（Qt Creator） | 渲染、JS 执行、信号与内存分配时间线 | 界面卡顿的第一诊断工具 |
| C++ | perf + 火焰图 | C++ 热点采样 | 编译保留 `-g` 调试符号，`perf record -g` 后 `perf report` 或转火焰图 |
| C++ | Valgrind Massif | 堆内存峰值与分配位置 | 深度排查内存增长；10–20x 运行开销，仅开发阶段按需 |

## 命令与流程（任务 048 落地后回填）

各工具的可复现命令、基线数字与结果解读在实施任务 048 各步骤完成时回填到本节；登记口径：命令、机器环境、构建配置（Debug/Release）、数字与日期。命令尚未落地前不在此占位假命令。

## 当前 QML / 原生视口诊断

任务 [056](../task/056-qml-native-review.md) 已提供按需诊断：`QT_LOGGING_RULES='panta.viewport.debug=true'` 输出原生区域同步与实际提交帧；默认关闭。`PANTA_TEST_NATIVE_VIEWPORT=1` 启用既有 `panta_qml_viewport_module_test` 中的真实窗口用例，覆盖连续更新合帧、相同外观不重绘、祖先移动、隐藏/零尺寸恢复及跨窗口重建。必须在实际桌面和正确平台插件下运行，不设 `offscreen`；默认无头测试明确跳过该用例，另行验证带窗口归属的条目析构。

macOS 示例（仓库根目录，先 `cargo build --locked`）：

```sh
PANTA_TEST_NATIVE_VIEWPORT=1 QT_QPA_PLATFORM=cocoa \
  target/native/debug/app/panta_qml_viewport_module_test
```

056 的 CPU 字样生成对比是开发侧 Release 微基准，检查生成前后的顶点、颜色及面索引完全相同；原始脚本与数据放在忽略的 `artifacts/056/`，结果及环境登记在任务中，不进入 CI 时间门禁，不用于推断应用启动时间或帧率。

## 与质量门禁的关系

任务 [065](../task/065-vtk-zoom-and-cube-transition.md) 提供 VTK 导航 CPU 基准（不创建图形窗口或提交 GPU 帧）：

```sh
cmake --build target/native/debug --target panta_viewport_navigation_cpu_benchmark --parallel 4
target/native/debug/visualization/panta_viewport_navigation_cpu_benchmark
```

VTK WebGPU GPU-backed 基准与 CPU 目标并列维护，要求真实图形会话；测量场景状态更新至下一次 WebGPU 帧提交日志的间隔：

```sh
cmake --build target/native/debug --target panta_viewport_gpu_benchmark --parallel 4
target/native/debug/visualization/panta_viewport_gpu_benchmark
```

真实 VTK WebGPU 原生窗口的 GPU 基准与该 CPU 微基准保持独立，由任务 [048](../task/048-performance-testing.md) 维护；GPU 基准通过 `VtkViewport` 触发场景更新，测量事件调度到 VTK 帧提交的间隔。VTK 当前没有跨平台 GPU 完成或屏幕呈现时间戳，所以该指标包含 CPU 调度与 VTK 提交开销，不等同于 GPU 内核执行时间或实际显示器呈现间隔。两个目标都是开发侧手动工具，不注册为 CTest 或 CI 门禁。

单立方体场景，32 个循环变化的姿态；每项预热 1000 次，再采样 31 批、每批 1000 次。输出批次平均耗时的 p50 / p95，分解相机插值、变化姿态下的方向标记同步、无标记消融、稳定姿态同步、六面命中和裁剪范围重置，并比较无标记与完整 CPU 过渡。结果不包括 Qt 调度、GPU 渲染和提交延迟。该 executable 不注册到 CTest，无绝对性能门槛；机器、Debug 配置及数字见任务记录。

`cargo quality` / CI 不运行本模块任何工具；`cargo sanitize`（内存错误）与覆盖率（执行面）回答"对不对、测没测到"，本模块回答"快不快、内存峰值在哪"。启动耗时的构建侧测量（入口耗时表）登记在任务 048 验证表，不进入 CI。
