# 064 — VTK 视口导航、笛卡尔坐标系与六面体定位

- 状态：implemented
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[063](063-stl-import-and-mesh-workspace.md)
- 关联任务：[065](065-vtk-zoom-and-cube-transition.md)（交互绑定与方向过渡）
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-10-13

## 目标

在 VTK WebGPU 原生视口中提供右下角笛卡尔坐标轴、右上角六面体方向定位器和鼠标拖拽观察旋转。用户可以识别当前空间方向、切换到标准正交视角，并围绕模型连续观察。

本任务中的“旋转”是相机围绕焦点旋转，不修改 STL 顶点、Mesh IR 或工程导入记录，不产生工程 dirty，也不将逐帧视角变化写入 `.panta`。模型变换与视角持久化不在本任务范围。

## 设计边界

- 视口使用 VTK WebGPU 原生 surface，覆盖在 Qt Quick 内容之上；坐标轴和方向定位器必须在同一 VTK render window 内绘制，不能用会被遮挡的 QML 浮层或第二套 Qt overlay。
- 当前锁定的 VTK SDK 不提供适用的现成 orientation marker、annotated cube 或 trackball style。实现使用 VTK renderer、actor 和自有输入 / 相机逻辑；所有 VTK 类型均留在 `native/visualization/src/vtk/`，公共 `CaeViewport` 接口不暴露 VTK 类型。
- 坐标轴表示世界坐标系。方向定位器首期支持六个正交面；边、角命中不属于本任务验收范围。方向标签由 VTK 适配层提供英文 ASCII 文案，不经过 QML 或翻译字典。
- 鼠标职责沿用 [065](065-vtk-zoom-and-cube-transition.md)：右键拖动旋转主相机，左键点击六面体；方向切换使用 260 ms 的平滑过渡。滚轮缩放由 065 验收，本任务继续负责活动过渡期间的真实 GPU 资源销毁覆盖。
- 拖拽只改变相机姿态。Rust 领域状态、Mesh IR、工程资产及 `.panta` manifest 不因浏览视角变化而修改。
- 原生 surface 的输入、逻辑坐标到像素坐标换算、DPR、窗口切换和资源释放由 C++ / VTK 适配层处理；QML 只消费后端中立的视口接口。

## 参考资料

- [VTK WebGPU 硬件窗口与渲染数据](../standards/vtk.md)
- [可视化、RenderScene 与原生视口](../architecture/visualization.md)
- [分层与依赖方向](../standards/layering.md)
- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)

## 范围

包含：

- 欢迎场景与已导入网格场景中的坐标轴和六面体覆盖层。
- 六面体六个面的命中和标准正交视角切换，包括连续或快速重复点击时的相机过渡。
- 鼠标拖拽相机轨道旋转及与六面体点击、窗口交互的输入协调。
- resize、DPR、隐藏 / 恢复、窗口重建和关闭时的覆盖层、interactor 与 VTK 资源生命周期。
- 后端中立接口、可重复的状态 / 命中 / 生命周期测试、任务与架构文档同步。

不包含：

- 几何编辑、模型选择、测量、拾取高亮、裁剪、触摸手势或动画时间轴。
- 把相机状态写入工程，或修改 STL、网格和资产修订。
- 引入 `QQuickVTKItem`、Qt OpenGL scenegraph 集成，或将 VTK 类型加入公共视口头文件。
- 用 offscreen 测试代替真实窗口中的 GPU 像素、输入和视觉验收。

## 实施情况

- 已实现 VTK 原生双层渲染中的方向覆盖层：右下角坐标轴、右上角六面体和面方向文案；采用射线与实际六面体求交进行方向命中。
- 已实现相机轨道状态、过渡与视口输入适配；VTK 与平台输入细节封装在 native VTK 路径中。
- 已实现按场景挂载 / 移除 Welcome renderer：欢迎场景显示字标，导入场景移除全视口欢迎层，避免它遮挡导航覆盖层。
- `WelcomeScene::sync()` 是视口与测试共用的挂载决策；native 测试覆盖欢迎 → 导入 → 欢迎 → 隐藏，并确认方向 renderer 在切换中保留。
- 已扩展真实 GPU 生命周期测试：隐藏和换宿主窗口时启动相机过渡，并断言过渡与定时器停止；另在活动过渡且刷新待执行时销毁视口，等待事件队列排空确认没有析构后回调。macOS Cocoa 下通过。
- macOS 真实窗口已复验 Welcome / 已导入网格的覆盖层和右键拖动；Cocoa / Metal GPU 生命周期测试在 Qt 配置 DPR 1.0、1.5、2.0 时检查 VTK 应用的目标像素尺寸和真实帧提交，并验证最小化 / 恢复后的过渡停止与帧重提交。实现及当前平台验收完成，仅待 Windows / Wayland 实机验收。

## 验收标准

- [x] macOS 真实窗口在本次观察尺寸下，Welcome 与已导入 STL 均显示右下角坐标轴和右上角六面体；Welcome 字样只出现在欢迎场景。
- [x] macOS 真实窗口显示 Welcome / 导入网格中的坐标轴和六面体；Cocoa / Metal GPU 测试在 Qt 配置 DPR 1.0、1.5、2.0 下校验 VTK 应用的目标像素尺寸与逻辑尺寸缩放一致，并确认真实帧提交。
- [x] 右键拖动旋转、左键点击六面体及六方向平滑过渡已由 065 的 macOS 人工验收确认（2026-09-23）。
- [x] 六个面投影命中由 `Visualization.WelcomeWordmark::picks_projected_cube_faces` 覆盖，标准方向和连续过渡由 `Visualization.ViewportNavigation` 覆盖。
- [x] 用户于 2026-10-13 确认 macOS 真实窗口右键拖动可正常旋转。该路径只更新 C++ / VTK 相机，不修改 Mesh IR 或工程状态。
- [x] VTK 输入分类测试区分视口点击、右键旋转和滚轮缩放；固定显示尺寸的投影命中测试覆盖六个方向。真实窗口下的右键旋转由用户人工确认。
- [x] `WelcomeScene::sync()` 生产挂载决策有可重复测试，覆盖欢迎场景 → 导入场景 → 欢迎场景及隐藏状态；测试断言 Welcome renderer 按状态挂载 / 移除，同时方向 renderer 保持挂载。
- [x] macOS 真实 GPU 测试验证 resize 到零并恢复；活动相机过渡时隐藏视口和切换宿主窗口均断言过渡与定时器停止，并确认旧窗口资源释放；另验证在活动过渡且刷新待执行时销毁视口不会触发析构后回调。
- [x] Cocoa / Metal GPU 生命周期测试在 Qt 配置 DPR 1.0、1.5、2.0 下校验 VTK 目标尺寸并确认帧提交；窗口最小化停止相机过渡，恢复后重新提交帧。隐藏、换宿主、resize 和析构资源释放也有真实 GPU 覆盖。
- [x] Rust / `.panta` 数据边界和 QML 公共接口不暴露 VTK 类型；`cargo format --check` 与 `cargo test --locked --workspace`（含 `qmllint` 和 native 测试）通过。
- [x] 完成 macOS 真实窗口及 GPU 生命周期验收。
- [ ] Windows / Wayland 真实窗口、DPR 和交互验收（仅待跨平台实机；不以离屏测试替代）。

## 验证记录

| 日期 | 环境 / 命令 | 结果 |
|---|---|---|
| 2026-09-22 | 锁定 VTK 9.7.0 SDK 头文件与 CMake targets | 未发现适用的现成 orientation widget 或 trackball style；确定使用 VTK 原生 renderer / actor 实现，并加入 `VTK::FiltersSources`、`VTK::FiltersGeneral`。 |
| 2026-09-22 | `cmake --build target/native/debug` | 构建通过；记录到既有重复静态库链接 warning。 |
| 2026-09-22 | C++ / CMake 格式检查、`git diff --check` | 直接 clang-format、cmake-format 与 diff 检查通过。统一 `cargo format` 的 uv / system-configuration 检查受本机 NULL system object 影响失败；不代表格式检查通过。 |
| 2026-09-23 | 定向 CTest：`I18n.CompiledQmLoads`、`Visualization.DefaultWordmark`、`Qml.FormatCheck`、`Qml.ShellModuleLoads`、`Qml.ViewportModuleLoads` | 5/5 通过；覆盖方向投影 / 命中、面姿态、渲染器生命周期消融及 QML 模块加载。 |
| 2026-09-23 | `target/native/debug/visualization/panta_default_wordmark_test` | CPU 基线：无 overlay 更新约 0.0000058 ms/次、完整 overlay 更新约 0.000094 ms/次、射线命中约 0.00088 ms/次；不含 WebGPU 提交时间。 |
| 2026-09-23 | macOS 真实窗口，见 [065](065-vtk-zoom-and-cube-transition.md) | 用户人工核对滚轮缩放、右键旋转、左键六面体点击和方向过渡通过；该记录早于本轮 renderer 生命周期改动，需回归复验。 |
| 2026-10-02 | `cargo test --locked --workspace`、`cargo format --check`、`cargo lint --check` | 与统一 1280×720 主窗口下限更新对应；Cargo 聚合测试通过，包含当时 CTest 69/69。该验证早于本轮 VTK renderer 生命周期改动。 |
| 2026-10-13 | `cargo build --locked` | 本轮 VTK renderer 生命周期改动后构建通过；构建成功不替代测试或真实窗口交互验收。 |
| 2026-10-13 | `cargo format --check`、`git diff --check`、`cargo test --locked --workspace` | 通过；Cargo 聚合测试包含 75/75 CTest。 |
| 2026-10-13 | `cargo lint --check` | 通过，8 个阶段全部完成。 |
| 2026-10-13 | macOS arm64 真实窗口：Welcome 与 `Benchmark/triangles_10000` | 欢迎字标只在 Welcome 场景显示；导入网格场景的 Welcome 字标已隐藏，六面体 FRONT 面和坐标轴可见。本次未测试六面切换、拖拽或窗口生命周期。 |
| 2026-10-13 | `PANTA_TEST_NATIVE_VIEWPORT=1 ctest ... -R '^Qml\\.ViewportModuleLoads$'` | 未能作为 GPU 验收：CTest 为该目标强制设置 `QT_QPA_PLATFORM=offscreen`，测试等待真实原生 surface 后超时。 |
| 2026-10-13 | 直接运行原生生命周期测试并设 `QT_QPA_PLATFORM=cocoa` | 运行环境没有可用 Cocoa screen，Qt 报 `Cannot create window: no screens available` 后中止；真实 GPU 生命周期测试仍未验收。 |
| 2026-10-13 | 临时 `.app` wrapper：`native_refresh_and_window_lifecycle`，Cocoa / macOS GPU | 通过，3/3；实际 GPU surface 下验证 resize 到零并恢复、活动过渡中的隐藏与宿主切换，以及旧窗口 renderer / interactor / hardware window 释放。该用例不覆盖带 pending refresh 的活动过渡析构。 |
| 2026-10-13 | 临时 `.app` wrapper：`destroys_attached_viewport_with_pending_refresh`，Cocoa / macOS GPU | 通过，3/3；真实 GPU 窗口下启动相机过渡并排入刷新后析构视口，等待 50 ms 未见析构后回调。 |
| 2026-10-13 | 临时 Cocoa `.app` wrapper：`native_refresh_and_window_lifecycle`，Qt 配置 DPR 1.0 / 1.5 / 2.0 | 三种启动配置均通过，合计 9/9；真实 GPU 窗口中校验 VTK 应用的目标尺寸并确认帧提交，也验证最小化时过渡停止、恢复后帧重提交，以及隐藏、换宿主、resize 和窗口资源释放。未读取 NSView / CAMetalLayer 实际像素尺寸，也未在运行期间切换 DPR。日志：`target/native/debug/app/viewport-tests.log`（临时产物，未提交）。 |
| 2026-10-13 | macOS 真实应用窗口与用户人工确认 | Welcome 和导入网格均显示坐标轴与六面体；用户确认右键拖动正常，六方向投影命中由 `Visualization.WelcomeWordmark::picks_projected_cube_faces` 确定性测试覆盖。 |
| 2026-10-13 | `cargo build --locked`、`cargo format --check`、`cargo test --locked --workspace` | 通过；Cargo 聚合测试 CTest 75/75。 |
| 2026-10-13 | `gpt-6.1-sol` medium 独立只读评审 | 指出过渡自然结束可能掩盖窗口切换停表、pending refresh 析构验收记录超出实际用例，以及旧测试描述过时并遗留无调用方 `set_visible()`；均已修正并复验。 |
| 2026-10-13 | `gpt-6.1-sol` medium 独立复审本轮改动 | 指出 DPR 测试只证明 VTK 目标尺寸和真实帧提交，未证明 backing surface 尺寸或动态 DPR / 实际输入坐标；已收窄验收证据并把未覆盖的原生行为列为 Windows / Wayland 实机条件，复审确认状态边界一致。 |
| — | Windows / Wayland 真实窗口、DPR 和交互 | 尚待对应平台实机验收；包含原生 surface 尺寸和实际指针坐标换算验证，不阻塞依赖任务。 |

## 决策与工作记录

- 2026-09-22：把“整体几何体旋转”定义为相机轨道旋转；坐标轴 / 六面体使用 VTK 原生 surface 内 overlay。首期采用世界坐标系和六个标准面，不持久化视角。
- 2026-09-22：核对锁定 VTK SDK 后确认缺少可直接用于 WebGPU 的现成 orientation / trackball 组件，采用第二渲染层的独立 renderer、轴线、立方体和矢量文本实现。
- 2026-09-22 至 2026-09-23：根据窗口截图多次调整 overlay 位置、标签大小与姿态；六面命中改为 renderer 显示坐标的反投影射线与实体六面体求交。方向面使用明确的正交基，避免标准视角下标签倒置；仅显示朝向当前相机的可见面。
- 2026-09-23：完成一次 VTK 导航实现评审和生命周期 / 命中测试整理；验证结果见上表。方向标签固定为 VTK 适配层内置 ASCII 文案。
- 2026-09-23：[065](065-vtk-zoom-and-cube-transition.md) 将旋转改为右键拖动，保留左键六面体点击，并加入 260 ms 方向过渡；用户在 macOS 真实窗口人工确认交互通过。活动过渡中的真实 GPU 窗口销毁仍由 064 验收。
- 2026-10-02：用户确认主窗口最小尺寸统一为 1280×720 逻辑像素。取消针对 640×512 非支持尺寸的适配；跨 DPR 验收仍属于本任务。
- 2026-10-13：切换导入网格时移除覆盖全视口的 Welcome renderer，并在 render window 初始化后挂载方向 renderer；随后将场景挂载判定集中到 `WelcomeScene::sync()`，补充共享决策测试及活动过渡下隐藏、换宿主和析构用例。使用 `.app` wrapper 在真实 Cocoa/GPU 窗口分别运行相关用例通过，详情见上表。
- 2026-10-13：完成 macOS 窗口和 GPU 生命周期回归，用户确认右键拖动正常；独立评审后将状态更新为 `implemented`，仅保留 Windows / Wayland 原生窗口验收。

## 风险与限制

- 原生 surface 与 Qt 窗口层级、DPR 和平台 interactor 有关；macOS 真实 GPU 测试覆盖多种启动 DPR 下的 VTK 目标尺寸与帧提交，但未直接读取 backing surface 尺寸或验证动态 DPR 变化；Windows / Wayland 仍需对应实机验证。
- 六面体可见性、投影命中及相机输入分别由真实窗口观察、确定性命中测试和用户人工确认覆盖；跨平台真实输入仍待验收。
- 视口隐藏、宿主窗口切换或销毁时如发现旧回调访问，优先停用对应输入并修复 VTK 对象释放顺序，不改变工程数据边界。
