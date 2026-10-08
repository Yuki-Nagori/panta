# 性能测试与剖析

[模块导航](README.md) · [实施任务 048](../task/048-performance-testing.md) · [质量工具链](quality-tooling.md)

## 统一手动入口（任务 100）

源码集中在 `tests/performance/{cpu,gpu,support}`；性能专用文件名不重复 CPU/GPU 分类。共享功能测试辅助代码放 `tests/support`，QML 夹具仍为 `tests/fixtures/qml`。全部 native 性能目标独立登记在 `native/performance/CMakeLists.txt`，从默认构建与 CTest 排除，格式 / clang-tidy / include-cleaner / Cppcheck 仍覆盖这些源码。

```sh
cargo performance                       # 辅助代码 → CPU → GPU；一步失败即停止
cargo performance support               # 仅辅助代码与 CLI 正确性测试
cargo performance cpu                   # 辅助代码 → CPU
cargo performance gpu                   # 辅助代码 → GPU
cargo performance gpu --presentation    # 真实窗口对照
cargo performance all --project /path/to/model.panta --samples 31
```

默认通过正式工程服务生成 10,000 / 20,000 个合成平面三角形的 STL 工程，保留在 `target/performance/fixture-*`；用于入口与生命周期重复验证，不代表生产模型。`--project` 可选择实际工程；Rust STL 单文件基准另可通过 `PANTA_BENCH_STL` 指定输入，未设置时使用合成 STL。`--samples` 只影响工程激活 / 页签切换基准，其他场景仍使用各自固定的预热 / 采样策略。

native 基准采用当前 runner 配置（缺省 Debug），Rust 网格微基准及后台执行容量基准显式使用 Release；输出和比较须区分配置。默认命令需要完整 bridge / VTK 构建；GPU 需要硬件图形会话，初始化失败不当作通过。Qt Quick 默认渲染到硬件纹理；VTK 默认使用隐藏的原生硬件 surface，仍需要平台图形会话。原生窗口模式保留独立指标。

CPU 套件还运行 `panta-core` 的 `background_execution` example（源码 `tests/performance/cpu/background_execution.rs`）：合成 5 万 / 50 万三角面的 binary STL，分别提交 1 / 4 / 8 / 64 个预检请求，输出输入字节、整批耗时及请求完成延迟 p50 / p95。通过公开工程服务驱动，完成即释放来源快照；固定负载不受 `--samples` 或 `--project` 控制。可单独用 `cargo run --locked --release -p panta-core --example background_execution` 定位；不纳入功能测试或 CI 时间门禁。容量及峰值内存证据见 [109](../task/109-bounded-background-execution.md)。

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

QML CPU 构造与资源加载基准由任务 [069](../task/069-project-docks-review-and-ablation.md) 的手动入口承载；图标场景比较空 source、单色 image provider 和保留 SVG 原色的 qrc 路径，测 1 / 8 / 24 个图标从创建到 `Image.Ready` 的耗时，并报告首轮样本与缓存后 31 次采样的 p50 / p95。首轮样本包含该进程和当前路径的初始化成本，计时不包括 GPU 帧呈现。基准不注册为 CTest 或 CI 门禁，任务 081 记录此次环境和结果。

```sh
cmake --build target/native/debug --config Debug --target panta_qml_cpu_benchmark --parallel 4
QT_QPA_PLATFORM=offscreen target/native/debug/performance/panta_qml_cpu_benchmark measures_icon_loading
```

QML GPU 基准默认使用 `QQuickRenderControl` 渲染到硬件纹理，不显示窗口；仍需要可用的 GPU 和平台插件。共享辅助类位于 `tests/performance/support/offscreen_quick_renderer.hpp`，管理纹理、深度/模板缓冲、帧完成计时与 GPU 时间戳；场景、预热和统计由基准负责。默认模式按 60Hz 逻辑帧推进动画，每场景预热 30 帧，再采样 3 × 60 帧。报告 CPU 更新至离屏帧完成的耗时，以及本帧 GPU 时间戳；不包含显示器刷新等待。默认离屏模式要求可识别的硬件设备；设备初始化失败、丢失或任一正式样本无有效 GPU 时间戳均失败，不回退到软件渲染。内容尺寸显式匹配目标逻辑尺寸；虚拟列表行数在预热后统计，并断言已实现可视行，避免用尚未完成布局的元数据描述正式负载。

实现依据：[Qt RenderControl RHI 示例](https://doc.qt.io/qt-6/qtquick-rendercontrol-rendercontrol-rhi-example.html)。`QRhi` 通过 `Qt6::GuiPrivate` 使用，按仓库 Qt 6.11 基线编译。窗口类不放入离屏控件夹具，窗口创建和行为由已有弹窗测试覆盖。

```sh
cmake --build target/native/debug --target panta_qml_gpu_benchmark --parallel 4
QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/performance/panta_qml_gpu_benchmark
PANTA_BENCHMARK_PRESENTATION=1 QT_QUICK_CONTROLS_STYLE=Basic \
  target/native/debug/performance/panta_qml_gpu_benchmark
```

显式呈现模式保留真实窗口 `frameSwapped` 间隔，并在渲染线程采集 swapchain 已完成的 GPU 时间戳；它需要窗口可见。尚未就绪的时间戳不进入分位数，报告有效观测数；完全没有有效观测或请求帧未完整呈现均失败。比较时固定机器、后端、逻辑尺寸、DPR、场景和样本数。离屏同步完成与窗口流水线呈现具有不同调度边界，不能将两种耗时混成一个基线，也不能用离屏性能代替真实窗口的布局和输入验收。099 同机比较未证明两条路径等价，故保留呈现模式。VTK WebGPU 使用独立的共享辅助类 `tests/performance/support/offscreen_vtk_renderer.hpp`，复用产品几何转换、欢迎字样与方向标记，管理隐藏 surface、设备检查、渲染与队列同步。提交和完成计时均从 CPU 场景更新开始；VTK 错误会使基准失败，设备销毁的 INFO 日志不属于渲染失败。

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
target/native/debug/performance/panta_viewport_navigation_cpu_benchmark
```

VTK WebGPU 基准与 CPU 目标独立维护，默认隐藏硬件 surface，预热 30 帧、正式采样 3 × 60 帧；报告 CPU 场景更新至 Render 返回和 WaitForCompletion 返回的两种耗时：

```sh
cmake --build target/native/debug --target panta_viewport_gpu_benchmark --parallel 4
target/native/debug/performance/panta_viewport_gpu_benchmark
```

`PANTA_BENCHMARK_PRESENTATION=1` 通过产品 `VtkViewport` 测量场景更新至帧提交日志的间隔。离屏完成计时包含 CPU 更新、VTK 提交与队列同步；窗口提交计时包含 GUI 调度，未等待 GPU 完成。两者均不是 GPU 内核时间或显示器呈现时间，不能混成一个基线。两个目标都是开发侧手动工具，不注册为 CTest 或 CI 门禁。性能目标在所有平台统一产出到 `target/native/debug/performance/`（Windows 后缀 `.exe`），Qt/VTK 依赖 DLL 由任务 083 的部署步骤与 staging 拷贝提供，裸启即可运行。

单立方体场景，32 个循环变化的姿态；每项预热 1000 次，再采样 31 批、每批 1000 次。输出批次平均耗时的 p50 / p95，分解相机插值、变化姿态下的方向标记同步、无标记消融、稳定姿态同步、六面命中和裁剪范围重置，并比较无标记与完整 CPU 过渡。结果不包括 Qt 调度、GPU 渲染和提交延迟。该 executable 不注册到 CTest，无绝对性能门槛；机器、Debug 配置及数字见任务记录。

`cargo quality` / CI 不运行本模块任何工具；`cargo sanitize`（内存错误）与覆盖率（执行面）回答"对不对、测没测到"，本模块回答"快不快、内存峰值在哪"。启动耗时的构建侧测量（入口耗时表）登记在任务 048 验证表，不进入 CI。

### GPU 基准共享约定（任务 100）

三个 GPU 基准统一使用 `tests/performance/support/benchmark_statistics.hpp` 的最近秩 p50/p95；空、负值、NaN 和无穷计时拒绝汇总。偶数样本的 p50 取下中位秩，与旧 Qt Quick / VTK 的上中位数略有差别，历史基线比较须注明统计规则。

VTK 功能测试和两个 VTK 基准的真实窗口模式使用 `tests/support/qt/frame_submission_capture.hpp` 捕获 GUI 线程提交事件；作用域结束恢复原 Qt 日志处理器与过滤器，不清空调用者的日志规则。辅助代码行为测试由 `cargo performance` 单独执行，不注册到常规 Cargo/CTest 聚合测试。
