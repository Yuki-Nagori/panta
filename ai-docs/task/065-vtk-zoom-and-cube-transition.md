# 065 — VTK 视口缩放、右键旋转与方向过渡

- 状态：done
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[064](064-vtk-navigation-and-orientation.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-23 / 2026-09-23

## 目标与背景

现有 VTK 原生视口支持拖拽旋转和六面体方向点击，但没有缩放交互，点击方向时相机立即跳到目标姿态。本任务为视口加入滚轮缩放、将相机旋转绑定到鼠标右键拖动，并让六面体方向切换平滑过渡。六面体继续使用鼠标左键点击。交互仅改变临时相机状态，不修改几何或工程数据。

## 必读

- [VTK WebGPU 硬件窗口与渲染数据](../standards/vtk.md)
- [可视化、RenderScene 与原生视口](../architecture/visualization.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 鼠标滚轮向前 / 向后分别缩放 VTK 场景，缩放围绕相机焦点进行并限制到稳定范围。
- 鼠标右键拖动旋转主相机；左键点击仍负责选择六面体方向。
- 六面体方向点击时，在当前相机姿态和目标正交方向之间执行有时长上限的平滑动画；相机与方向标记逐帧同步。
- 连续点击时从当前过渡姿态平滑重定向；用户开始旋转或缩放时停止方向动画并保留当前相机姿态。
- 视口隐藏、原生渲染资源销毁或宿主窗口切换时停止动画，避免定时回调访问已销毁 VTK 对象。
- 补充可重复测试，覆盖缩放限幅、相机姿态插值、动画重定向 / 中断、左右键职责和无效生命周期路径。
- 增加消融与 CPU 性能基准，区分相机姿态计算、方向标记同步以及完整导航更新的成本；性能结果记录环境和输入，不设脆弱的绝对时间门槛。

不包含：

- QML 工具栏缩放按钮、键盘快捷键、平移手势、触屏交互或可配置动画时长。
- 改变相机状态持久化、几何变换或工程 dirty 行为。
- 引入常驻渲染定时器；无交互时继续按现有策略不提交帧。
- WebGPU 提交帧的跨平台 GPU 性能结论；CPU 基准不能替代真实 GPU 测量。

## 前置条件与待决策

- 复用 064 已有 VTK 原生 surface、interactor 和方向命中路径，确认锁定 SDK 的滚轮事件可由当前平台 interactor 送达。
- 相机动画只在 GUI 线程运行；缩放上下限结合当前显示几何的 bounds 计算，空 / 退化 bounds 使用有限安全值。
- 快速滚轮、拖拽与重复六面体点击的优先级以当前有效用户输入为准，动画不能覆盖后续用户操作。

## 实施步骤

1. 核实 interactor 滚轮事件在当前硬件窗口路径中的回调契约。
2. 为滚轮事件实现有界焦点缩放，并在输入时中断相机动画。
3. 为六面体相机方向切换实现有界时长、缓动插值和重定向。
4. 清理资源时停止动画并复位状态；检查注释与任务记录。
5. 为导航状态与计算边界增加行为测试、消融和 CPU 性能基准。
6. 运行 native 构建、相关测试和格式检查；同步记录用户人工核对与实际测量结果。

## 预计改动

- `native/visualization/src/vtk/vtk_viewport.cpp`：滚轮事件、相机缩放和方向过渡调度。
- `native/visualization/src/vtk/vtk_viewport.hpp`：仅当实现需要私有事件处理声明时调整。
- `native/visualization/src/vtk/navigation/`：相机状态、输入映射和方向标记的职责归类与可测试实现。
- `native/visualization/src/vtk/vtk_viewport.cpp`：GUI 事件、定时器和渲染刷新编排。
- `tests/cpp/visualization/viewport_navigation_test.cpp`：导航行为测试。
- `tests/cpp/visualization/viewport_navigation_benchmark.cpp`：独立开发侧消融与 CPU 基准；不注册 CTest 时间门禁。
- `tests/cpp/visualization/default_wordmark_test.cpp`：保留几何与方向标记覆盖，清理失效命中假设。
- `ai-docs/task-index.md`、本任务：同步状态与真实验证结果。

## 清理与兼容例外

- 六面体点击现有的瞬时相机设置逻辑由动画路径取代；不保留并行切换实现。
- 移除未挂载方向标记时按矩形象限猜测六面体方向的旧回退；未挂载 / 无效输入直接返回无命中。
- 无兼容例外，不改变公共 C++ / QML API、RenderScene 或 `.panta` 格式。

## 验收标准

- [x] 视口滚轮正反方向均可缩放，焦点和朝向保持稳定；相机距离不会变成零或无限大。
- [x] 鼠标右键拖拽可旋转视图，左键点击六面体仍可切换标准方向。
- [x] 点击六个六面体面均平滑到对应标准正交方向，方向坐标标记与主视图连续同步。
- [x] 动画过程中再次点击会平滑转向新目标；拖拽或滚轮输入可接管当前姿态。
- [x] 隐藏、窗口切换和视口销毁会停止动画回调，不产生额外后台帧循环。
- [x] 测试覆盖缩放边界、姿态插值端点 / 中间态、方向重定向与用户输入中断、左右键职责；复核隐藏 / 销毁路径停表，运行现有 pending-refresh 销毁测试。活动动画中的真实 GPU 窗口销毁与跨平台验证仍属 064 的生命周期覆盖。
- [x] 消融与性能基准覆盖相机计算、方向标记同步和完整更新路径，结果记录机器、构建类型、迭代规模与均值 / 分位数；不以性能数字作为不稳定的 CI 硬门槛。
- [x] 本机构建、相关测试与格式检查结果已记录；真实窗口行为由用户人工核对通过。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-23 | macOS ARM64，仓库根目录；`cmake --build target/native/debug --target panta_visualization --parallel 4`，VTK 9.7.0 | 编译和链接本任务涉及的 native 目标 | 通过；`panta_visualization` 编译并链接成功，退出码 0 |
| 2026-09-23 | 仓库根目录；托管 clang-format 22.1.7 `--dry-run --Werror` 检查修改的 C++ 文件；`git diff HEAD --check` | 格式与差异空白检查通过 | 通过；两项检查均退出码 0 |
| 2026-09-23 | macOS 真窗口：滚轮缩放、旋转绑定、六面体点击与方向过渡 | 交互符合验收标准 | 用户人工核对通过 |
| 2026-09-23 | 仓库根目录；`cmake --build target/native/debug --target panta_visualization panta_default_wordmark_test panta_viewport_navigation_test panta_viewport_navigation_benchmark --parallel 4` | 新增与调整目标编译通过 | 通过，退出码 0 |
| 2026-09-23 | `ctest --test-dir target/native/debug --output-on-failure -R '^Visualization\.'` | 几何 / 方向与导航行为通过 | 2/2 通过；覆盖连续缩放限幅、退化 bounds、六方向、插值、重定向与取消、事件映射 |
| 2026-09-23 | 构建 `panta_qml_viewport_module_test` 后运行 `ctest --test-dir target/native/debug --output-on-failure -R '^Qml\.ViewportModuleLoads$'` | 现有视口加载与生命周期回归通过 | 构建退出码 0，CTest 1/1 通过；不能替代活动动画中的真实 GPU 销毁验证 |
| 2026-09-23 | `cargo lint clang-tidy --check`；`cargo build --locked -p panta-launcher` | 静态检查与启动器构建通过 | 均退出码 0 |
| 2026-09-23 | `cargo lint cmake --check` | 统一 CMake 检查 | 本机 uv / system-configuration 出现 NULL object panic；改用 `target/panta-tools/python/venv/bin/cmake-lint native/visualization/CMakeLists.txt` 和同目录 `cmake-format --check`，修正格式后均退出码 0 |
| 2026-09-23 | `target/native/debug/visualization/panta_viewport_navigation_benchmark` | 分离计算、方向同步、裁剪重置与完整 CPU 更新成本 | 退出码 0，采样见下表 |

## 风险与回退

- 不同平台 interactor 对连续滚轮的事件频率可能不同；按事件步长进行倍率缩放，避免依赖固定帧率。
- WebGPU 帧仅由 GUI 事件驱动提交；动画期间使用短周期定时器，完成后立即停表。
- 若平台真实输入无法送达 VTK observer，修复限于 native surface / interactor 输入桥接，不引入 QML 覆盖层。

## 决策与工作记录

- 2026-09-23：登记任务，针对现有 VTK 拖拽旋转与六面体跳转补充滚轮缩放和相机平滑过渡。
- 2026-09-23：interactor 注册 VTK 正反向滚轮事件；以当前 bounds 对相机距离限幅，滚轮缩放保持焦点与朝向。六面体点击使用四元数球面插值和 smoothstep，在 260 ms 内转向目标并同步方向标记；重复点击重定向，右键旋转 / 滚轮缩放输入、resize、隐藏和窗口销毁停止动画计时器。旋转改由右键拖动，左键只负责六面体点击。
- 2026-09-23：用户人工核对真实窗口的滚轮缩放、旋转按键和六面体方向交互通过；后续补齐可重复行为测试、消融和性能基准。
- 2026-09-23：初版 native 目标构建、C++ 格式检查和 `git diff HEAD --check` 通过；测试与性能记录待补，任务保持 in-progress。

- 2026-09-23：完成导航职责拆分、死代码清理、独立行为测试与开发侧基准；记录最终验证和 CPU 消融结果。将活动动画真实 GPU 销毁覆盖明确归入 064，避免将无头测试误记为完整图形生命周期验证。

## 完成摘要

滚轮缩放、右键旋转与六面体平滑过渡完成；导航按相机、输入映射和方向标记拆入 navigation 文件夹，清理废弃命中回退。用户人工验收通过；新增行为测试、CPU 消融和基准完成，native 构建、相关 CTest、clang-tidy 与直接格式检查通过。跨平台真实窗口及活动动画 GPU 销毁覆盖继续由 064 跟踪；本任务不宣称已完成该覆盖。

## CPU 消融与性能记录

环境：MacBook Air，Apple M4（10 核），24 GB；macOS 26.3.1 ARM64；CMake Debug，LLVM 22.1.7，Qt 6.11.2，VTK 9.7.0。单立方体场景、32 个变化姿态，每项预热 1000 次，31 批 × 1000 次；p50 / p95 是各批平均 ns/op 的分位数，不是单帧延迟分位数。

| 路径 | p50 ns/op | p95 ns/op |
|---|---:|---:|
| 相机姿态插值 | 155.584 | 186.291 |
| 方向标记同步 | 2294.13 | 2419.50 |
| 裁剪范围重置 | 1325.13 | 1423.83 |
| 完整更新去掉方向标记同步 | 3245.04 | 3473.75 |
| 完整 CPU 过渡更新 | 5572.04 | 5966.08 |

消融采用相同变化姿态；不包含 Qt 定时调度、GPU 渲染或复杂工程场景，不能推导全帧 FPS。当前相机计算在 CPU，VTK WebGPU 承担绘制；测量未显示值得把这些轻量相机计算迁到 GPU 的 CPU 成本。
