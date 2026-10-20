# 性能测试与剖析

[模块导航](README.md) · [实施任务 048](../task/048-performance-testing.md) · [质量工具链](quality-tooling.md)

更新日期：2026-10-20。

## 统一手动入口（任务 100）

源码集中在 `tests/performance/{cpu,gpu,support}`；性能专用文件名不重复 CPU/GPU 分类。共享功能测试辅助代码放 `tests/support`，QML 夹具仍为 `tests/fixtures/qml`。全部 native 性能目标独立登记在 `native/performance/CMakeLists.txt`，从默认构建与 CTest 排除，格式 / clang-tidy / include-cleaner / Cppcheck 仍覆盖这些源码。

工程 Mesh 视口基准先等待目标文档 DTO 就绪，再核对该 Mesh 对应的场景修订已报告帧。真实窗口使用 `VtkViewport::frameSubmitted` 确认 VTK 提交，离屏在 `WaitForCompletion` 后确认同一修订的队列完成；切换时的空帧和迟到旧帧均不能完成目标采样。帧计时包含后台显示准备及场景更新，不能解释为纯 GPU 内核耗时；确定性顺序回归纳入 Bridge 行为测试，实际 GPU 测量仍按需单独运行。

```sh
cargo performance                       # 辅助代码 → CPU → GPU；一步失败即停止
cargo performance support               # 仅辅助代码与 CLI 正确性测试
cargo performance cpu                   # 辅助代码 → CPU
cargo performance gpu                   # 辅助代码 → GPU
cargo performance gpu --presentation    # 真实窗口对照
cargo performance all --project /path/to/model.panta --samples 31
```

默认通过正式工程服务生成 10,000 / 20,000 个合成平面三角形的 STL 工程，保留在 `target/performance/fixture-*`；用于入口与生命周期重复验证，不代表生产模型。`--project` 可选择实际工程；Rust STL 单文件基准另可通过 `PANTA_BENCH_STL` 指定输入，未设置时使用合成 STL。`--samples` 只影响工程激活 / 页签切换基准，其他场景仍使用各自固定的预热 / 采样策略。

需要在工程级基准中加入真实 STL 时，可直接向夹具程序追加一个或多个输入路径；源 STL 只作为导入输入，不复制到工程：

```sh
target/native/release/performance/panta_performance_fixture \
  "$PWD/target/performance/fixture-real" /path/to/model.stl
target/native/release/performance/panta_bridge_project_stl_activation_cpu_benchmark \
  "$PWD/target/performance/fixture-real/Benchmark/Benchmark.panta" 31
```

native 基准采用当前 runner 配置（缺省 Debug），Rust 网格微基准及后台执行容量基准显式使用 Release；输出和比较须区分配置。默认命令需要完整 bridge / VTK 构建；GPU 需要硬件图形会话，初始化失败不当作通过。Qt Quick 默认渲染到硬件纹理；VTK 默认使用隐藏的原生硬件 surface，仍需要平台图形会话。

CPU 套件还运行 `panta-core` 的 `background_execution` example（源码 `tests/performance/cpu/background_execution.rs`）：合成 5 万 / 50 万三角面的 binary STL，分别提交 1 / 4 / 8 / 64 个预检请求，输出输入字节、整批耗时及请求完成延迟 p50 / p95。通过公开工程服务驱动，完成即释放来源快照；固定负载不受 `--samples` 或 `--project` 控制。可单独用 `cargo run --locked --release -p panta-core --example background_execution` 定位；容量及峰值内存证据见 [109](../task/109-bounded-background-execution.md)。

任务 [088](../task/088-mesh-asset-sidecar-storage.md) 的 Rust microbenchmark 可对比来源 STL 解析、Mesh sidecar 编码 / 解码、热文件缓存读 + 解码、同步写入和文件字节。未设置 `PANTA_BENCH_STL` 时用 100k 合成 binary STL；代表性实物应显式传入文件，并在相同机器 / Release 环境运行：

```sh
PANTA_BENCH_STL=/path/to/model.stl \
  cargo test --locked --release -p panta-mesh --test mesh_performance \
  supplied_stl_and_sidecar_costs -- --ignored --nocapture
```

该微基准不测完整工程首次激活或缓存命中；这些项目由 `panta_bridge_project_stl_activation_cpu_benchmark <project.panta> [reload_iterations]` 测量。工程级结果包括 manifest 打开 / 原子保存耗时、manifest 与 sidecar 字节、首次激活与关闭重载耗时、缓存切换、各阶段进程 CPU 时间、进程 RSS / 高水位，以及 Mesh DTO 常驻容量估算。进程 CPU 时间由平台进程 CPU 时钟测量（macOS 使用 `getrusage`，Linux 使用 `std::clock`，Windows 使用 `GetProcessTimes`）。打开计时覆盖新建 `ProjectViewModel`、读取并校验 manifest，不加载 sidecar；激活到 ready 覆盖 Rust sidecar 读盘、摘要校验、解压 / 解码、领域校验及 FFI 快照投影，不包含 VTK、GPU 上传或真实窗口呈现。RSS 是进程级采样；关闭后 Mesh 快照弱引用必须失效，RSS 不保证立即下降，因为分配器可能保留已释放内存。所有磁盘读取均使用暖 OS 页缓存，除非另有标记。Mesh sidecar v2 保留 f64 三角顶点语义并用 Zstandard 压缩 payload；压缩后大小和编解码耗时依赖具体数据。

## 定位与边界

性能阈值不作为 CI 门禁。每日定时 CI 运行 `cargo performance cpu`，并用 `cargo hyperfine` 对比 Debug/Release 的 dslc CLI；日志和 hyperfine JSON 上传为 artifact，用于观察趋势。共享 runner 的波动不阻断变更。GPU 基准要求可识别的硬件设备或真实图形会话，仍由开发侧手动运行，直到有稳定的专用 runner。测量优先于修改——没有测量支撑的"优化"不保留；对比必须固定同一机器、同一命令与同一构建配置，波动显著才立项修复。

## 工具矩阵

| 层 | 工具 | 用途 | 适用场景 |
|---|---|---|---|
| Rust | Criterion.rs | 函数级微基准，统计分析 + HTML 报告，可对比命名基线 | 量化核心逻辑（`.pa` 解析/格式化、工程页签状态转换）的变化 |
| Rust | cargo-flamegraph（底层 perf） | CPU 火焰图，展示调用栈耗时分布 | 定位 CPU 瓶颈的首选入口 |
| Rust/CLI | hyperfine | 命令级基准与统计对比 | 对比 dslc 启动/执行时间、编译选项或参数差异 |
| QML | QML Profiler（Qt Creator） | 渲染、JS 执行、信号与内存分配时间线 | 界面卡顿的第一诊断工具 |
| C++ | perf + 火焰图 | C++ 热点采样 | 编译保留 `-g` 调试符号，`perf record -g` 后 `perf report` 或转火焰图 |
| C++ | Valgrind Massif | 堆内存峰值与分配位置 | 深度排查内存增长；10–20x 运行开销，仅开发阶段按需 |

## Rust 微基准与命令对比

Criterion 基准源码放在 `tests/performance/cpu/`，通过各领域 crate 的 `[[bench]]` 注册，不进入运行时依赖或常规测试。构建后在 `target/criterion/` 生成统计结果和 HTML 报告；命名基线只保存在本地 target 目录，不提交机器相关数据。基准只包含库调用，不含 CLI 启动、文件读取或 GPU 工作。

```sh
CRITERION_HOME="$PWD/target/criterion" cargo bench --locked -p panta-dsl-core --bench dsl_core_criterion \
  -- --save-baseline <machine-and-date>
CRITERION_HOME="$PWD/target/criterion" cargo bench --locked -p panta-core --bench core_workspace_criterion \
  -- --save-baseline <machine-and-date>
# 同一机器、相同工具链和构建参数下重新运行后，与已有基线对照：
CRITERION_HOME="$PWD/target/criterion" cargo bench --locked -p panta-dsl-core --bench dsl_core_criterion \
  -- --baseline <machine-and-date>
```

DSL 套件测量解析 fixture、从源码解析并格式化、以及对已解析 AST 格式化；core 套件测量新建页签、请求激活未加载页签、以及单工程页签快照。Core 基准用 `iter_batched_ref` 在计时区间外创建工作区，避免夹具构造和析构时间混入被测操作。升级 Rust/编译器、依赖、基准 profile 或测量 fixture 后，旧基线不再作为新环境的对照；使用新环境标识重新生成命名基线，并在任务记录中注明机器、工具链、依赖版本、profile 与 fixture 变化。不要把环境升级造成的新旧差异解释为业务性能回归。

2026-10-18 本机 Criterion 基线：macOS 26.3.1 arm64、rustc/cargo 1.98.1、`bench` profile。以下为 Criterion `slope` 点估计及 95% CI：DSL fixture 解析 17.674 µs（17.351–18.172 µs）、源码 parse+format 19.513 µs（19.308–19.767 µs）、已解析文档格式化 1.894 µs（1.890–1.899 µs）；工程页签新建 57.154 ns（56.244–58.210 ns）、未加载页签激活请求 31.659 ns（29.213–33.780 ns）、单页签快照 196.732 ns（194.887–199.628 ns）。少数样本存在高位异常；本结果仅用于同机后续对比，不设性能门槛。`CRITERION_HOME` 显式固定到 workspace `target/criterion`，避免 Cargo workspace 中报告落入 crate 子目录。完整统计与图表可由上方命令重建。

命令级对比使用 `cargo hyperfine`，首次调用会从固定的官方平台资产安装 hyperfine 2.0.0 并验证 SHA256，文件缓存位于 `target/panta-tools/hyperfine/`。先编译两个 profile，避免把编译耗时混入执行耗时；无预热 run 只是首次进程启动样本，不代表清空磁盘缓存。工作区不存在可切换的 dslc 编译开关，因此 Debug/Release 是本项目实际可比较的构建配置：

```sh
cargo build --locked -p panta-dslc
cargo build --locked --release -p panta-dslc
cargo hyperfine --runs 15 --warmup 0 \
  'target/debug/panta-dslc check tests/fixtures/dsl/artifacts.pa' \
  'target/release/panta-dslc check tests/fixtures/dsl/artifacts.pa'
cargo hyperfine --runs 30 --warmup 5 \
  'target/debug/panta-dslc check tests/fixtures/dsl/artifacts.pa' \
  'target/release/panta-dslc check tests/fixtures/dsl/artifacts.pa'
```

短 CLI 测量须记录 warmup 次数、profile、机器和 hyperfine 输出；进程启动 / 调度开销不能当作 Criterion 库级成本。

## CPU 与内存剖析

Linux 使用带调试符号的 bench profile，通过 Criterion 持续采样 Rust 热路径；`--bench` 使 Criterion 以 benchmark 模式运行，`--profile-time` 控制采样时间。Rust 1.90+ 使用 lld 或 mold 链接时，把 `--no-rosegment` 作为链接器参数传入（若环境已有 `RUSTFLAGS`，请将此参数追加到现有值）：

```sh
CARGO_PROFILE_BENCH_DEBUG=true \
RUSTFLAGS='-C link-arg=-Wl,--no-rosegment' \
  cargo flamegraph \
  -p panta-dsl-core --bench dsl_core_criterion \
  -- --bench --profile-time 20
perf record -g --call-graph dwarf -o target/performance/native.data -- \
  target/native/debug/performance/panta_viewport_navigation_cpu_benchmark
perf report --input target/performance/native.data
```

macOS 使用 Instruments 的 Time Profiler 对同一 Release/Debug 场景取样；Windows 使用 Visual Studio CPU Usage 或 Windows Performance Recorder。平台采样器的调用栈和计时口径不同，不与 Linux 的绝对耗时比较。

QML Profiler 需要带 QML debugging infrastructure 的构建，且调试端口只在本机受信任环境开放。通过 Qt Creator 的 Analyze → QML Profiler 启动应用，重现一个固定交互（如打开指定页签或弹窗），停止采样后保存 `.qzt` trace；也可用 `qmlprofiler` 启动并捕获：

```sh
mkdir -p target/performance
qmlprofiler --output target/performance/panta.qzt \
  target/native/debug/app/panta \
  -qmljsdebugger=port:3768,services:CanvasFrameRate,EngineControl,DebugMessages
```

QML trace 应记录触发操作、样本时间窗与启用的服务；调试通道本身会增加少量开销，结果用于定位绑定、JS、组件创建和场景图事件，不与未启用调试的帧耗时直接对比。详见 [Qt QML profiling](https://doc.qt.io/qt-6/qtquick-profiling.html)。

Linux 堆分析使用 Valgrind Massif；在图形桌面会话中启动 Debug 应用、重现目标工程操作后退出，再读取快照。Massif 会显著放慢运行，10–20 倍属于诊断工具开销，不是应用实际性能：

```sh
valgrind --tool=massif --time-unit=ms \
  --massif-out-file=target/performance/massif.out.%p \
  target/native/debug/app/panta
ms_print target/performance/massif.out.<pid>
```

Massif 观察进程堆分配峰值，不等于 RSS；需要评估常驻集时结合 task 088/109 的 RSS 与 Mesh 容量报告。macOS 使用 Instruments Allocations / VM Tracker；Windows 使用 Windows Performance Recorder 的 Heap/Memory 分析。

## 当前 QML / 原生视口诊断

QML CPU 构造与资源加载基准由任务 [069](../task/069-project-docks-review-and-ablation.md) 的手动入口承载；图标场景比较空 source、单色 image provider 和保留 SVG 原色的 qrc 路径，测 1 / 8 / 24 个图标从创建到 `Image.Ready` 的耗时，并报告首轮样本与缓存后 31 次采样的 p50 / p95。首轮样本包含该进程和当前路径的初始化成本，计时不包括 GPU 帧呈现。该目标不注册 CTest，纳入每日 CPU 套件；手动基线见任务 081。

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

显式呈现模式保留真实窗口 `frameSwapped` 间隔，并在渲染线程采集 swapchain 已完成的 GPU 时间戳；它需要窗口可见。尚未就绪的时间戳不进入分位数，报告有效观测数；完全没有有效观测或请求帧未完整呈现均失败。099 同机比较未证明两条路径等价，故保留呈现模式；固定机器、后端、逻辑尺寸、DPR、场景和样本数，分别建基线。离屏性能不替代窗口布局及输入验收。VTK WebGPU 使用独立的共享辅助类 `tests/performance/support/offscreen_vtk_renderer.hpp`，复用产品几何转换、欢迎字样与方向标记，管理隐藏 surface、设备检查、渲染与队列同步。提交和完成计时均从 CPU 场景更新开始；VTK 错误会使基准失败，设备销毁的 INFO 日志不属于渲染失败。

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

单立方体场景，32 个循环变化的姿态；每项预热 1000 次，再采样 31 批、每批 1000 次。输出批次平均耗时的 p50 / p95，分解相机插值、变化姿态下的方向标记同步、无标记消融、稳定姿态同步、六面命中和裁剪范围重置，并比较无标记与完整 CPU 过渡。结果不包括 Qt 调度、GPU 渲染和提交延迟。机器、Debug 配置及数字见任务记录。

VTK WebGPU 基准与 CPU 目标独立维护，默认隐藏硬件 surface，预热 30 帧、正式采样 3 × 60 帧；报告 CPU 场景更新至 Render 返回和 WaitForCompletion 返回的两种耗时：

```sh
cmake --build target/native/debug --target panta_viewport_gpu_benchmark --parallel 4
target/native/debug/performance/panta_viewport_gpu_benchmark
```

`PANTA_BENCHMARK_PRESENTATION=1` 通过产品 `VtkViewport` 测量场景更新至帧提交日志的间隔。离屏完成计时包含 CPU 更新、VTK 提交与队列同步；窗口提交计时包含 GUI 调度，未等待 GPU 完成。两者均不是 GPU 内核时间或显示器呈现时间，不能混成一个基线。两目标均不注册 CTest。性能目标在所有平台统一产出到 `target/native/debug/performance/`（Windows 后缀 `.exe`），Qt/VTK 依赖 DLL 由任务 083 的部署步骤与 staging 拷贝提供，裸启即可运行。


`cargo quality` 不运行本模块工具；CI 分工与性能阈值规则见“定位与边界”。`cargo sanitize`（内存错误）与覆盖率（执行面）回答"对不对、测没测到"，本模块回答"快不快、内存峰值在哪"。启动耗时的构建侧测量（入口耗时表）登记在任务 048 验证表。

### GPU 基准共享约定（任务 100）

三个 GPU 基准统一使用 `tests/performance/support/benchmark_statistics.hpp` 的最近秩 p50/p95；空、负值、NaN 和无穷计时拒绝汇总。偶数样本的 p50 取下中位秩，与旧 Qt Quick / VTK 的上中位数略有差别，历史基线比较须注明统计规则。

VTK 功能测试和通用视口基准的窗口模式使用 `tests/support/qt/frame_submission_capture.hpp` 捕获提交日志，作用域结束恢复调用者的日志处理器与过滤规则。工程 Mesh 基准改用 `frameSubmitted(sceneRevision)` 与 `tests/support/qt/mesh_frame_tracker.hpp` 关联目标场景，其确定性回归纳入常规 Bridge 测试。性能专用辅助代码的行为测试仍由 `cargo performance` 单独执行。
