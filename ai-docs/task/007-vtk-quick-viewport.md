# 007 — VTK WebGPU 硬件窗口原生视口

- 状态：in-progress
- 阶段：M0
- 依赖：[005](005-qt-qml-shell.md)（已完成：Qt Quick 主窗口与预编译 Qt 6.11.2 就绪）、[031](031-prebuilt-native-dependencies.md)（WebGPU 硬件窗口制品已登记）
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-20

## 前置条件已解除与历史阻塞

旧的 `QQuickVTKItem`/Qt OpenGL scenegraph 路线已经确认不适合作为当前实现：**窗口化渲染在 macOS 26（arm64，Qt 6.11.2 + OpenGLRhi）两种场景图循环下均崩溃**。该路线不再作为回退，也不保留兼容分支；旧适配器、Qt VTK GUI 模块和失效引用已删除。

- basic 循环（平台默认落到）：同步阶段（updatePaintNode）无当前 GL 上下文，glad 函数表加载为空（"Failed to initialize OpenGL functions" ×2），首次 `Render()` 段错误 pc=0x0——崩溃点为 dispatch_async 命令内的 Render 调用（崩溃报告 2026-09-19 18:14）。
- threaded 循环（显式 `QSG_RENDER_LOOP=threaded`）：更早崩溃——RHI 创建阶段 `NSOpenGLContext setView` 在非主线程被 AppKit 断言（SIGTRAP，崩溃报告 18:33）。
- VTK v9.7.0 源码核对：两种循环均无适配分支；加载器依赖 `QOpenGLContext::currentContext()` 在 updatePaintNode 时非空（threaded 循环契约）。

前置条件已满足：038/031 已产出并登记 `sdk-vtk-9.7.0-webgpu`，三平台包含 `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn` 和对应硬件窗口（Cocoa、Wayland、Win32）。Linux 制品明确为 `VTK_USE_X=OFF`、`VTK_USE_Wayland=ON`，纯 X11 环境不属于该资产的运行目标；若产品需要纯 X11，必须另产 X11 变体。原生 surface/view 与 Qt Quick 叠加、尺寸同步和事件协调已实现，run 36001859191 的三平台测试通过；任务仍待真实窗口下确认渲染可见性、resize、高 DPI、隐藏/恢复、关闭重开和资源生命周期。原生崩溃诊断的历史证据记录在 `~/Library/Logs/DiagnosticReports/*.ips`。

## 目标与背景

完成 M0 的关键图形集成：使用 VTK WebGPU render window 和各平台 hardware window；macOS 通过 Cocoa hardware view/layer 将 VTK 结果作为原生 Metal surface 嵌入 Qt Quick 界面，Linux 使用 Wayland surface，Windows 使用 Win32 窗口承载，提供空视口和小型测试图元。

038 生产与 031 登记已完成；实现前继续核对 Release 内实际 CMake targets、头文件和 ABI。不得在本地通过 FetchContent 或其他方式默认编译 VTK 源码。

## 必读

- [规范：vtk](../standards/vtk.md)
- [规范：qt](../standards/qt.md)
- [规范：qml](../standards/qml.md)
- [规范：cmake](../standards/cmake.md)
- [规范：cpp](../standards/cpp.md)
- [架构：visualization](../architecture/visualization.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小 WebGPU/平台硬件窗口原生视口基础设施。

非目标：不接 STEP/Netgen，不完成裁剪/标量等完整后处理，不开发自研 GPU backend，不再维护 `QQuickVTKItem` 或 Qt OpenGL scenegraph 集成。

## 前置条件与待决策

依赖 005 已完成，031/038 已登记新的 VTK WebGPU 硬件窗口制品。已核对预编译 VTK 9.7.0 SDK 的实际头文件、CMake targets、Apple/Windows/Linux ABI 和图形后端；Linux 按 Wayland 目标接入，纯 X11 另行登记变体。模块 URI 的正式自动注册仍由 026 负责，007 沿用 005 的显式注册边界。

## 实施步骤

1. 核对锁定 VTK 的 `vtkWebGPURenderWindow`、平台 hardware window API、模块和 surface 生命周期；macOS 重点核对 Cocoa/Metal layer，Linux 核对 Wayland，Windows 核对 Win32。
2. 确定 Qt Quick 窗口与原生 platform view/layer/surface 的承载、叠加、尺寸/高 DPI 同步和鼠标/触摸事件协调方式；不引入 Qt OpenGL scenegraph 集成。
3. 建立原生 `CaeViewport` 与最小 `RenderScene` 边界，在合法线程和生命周期回调中创建/更新/释放 WebGPU VTK 状态；只使用默认测试球体等小数据。
4. 保留重建所需 CPU 状态，验证 native view/layer 重建、隐藏/恢复、窗口退出和 VTK 资源释放；已删除 `QQuickVTKItem`、`GUISupportQtQuick`、`RenderingQt` 及失效 OpenGL 引用。

## 预计改动

native/bridge/viewport、native/visualization/、QML 视口组件及 CMake；并删除旧 `QQuickVTKItem`/Qt OpenGL 集成及其 SDK 依赖。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

不引入兼容层或双渲染路径。实施时必须记录删除的旧实现、配置、依赖与失效引用；完成后仓库中不得保留 `QQuickVTKItem`、`GUISupportQtQuick`、`RenderingQt` 或旧 OpenGL adapter 的死代码。禁止以 `COMPAT` 标记长期保留旧路径。

## 验收标准

- [ ] 通过 cargo run 显示空视口与默认测试球体，VTK WebGPU 渲染错误不会静默表现为“已成功”。
- [ ] resize、高 DPI、隐藏/恢复和关闭/重开可用，记录平台与图形后端。
- [ ] VTK 对象没有从 GUI/worker 任意修改，所用平台 WebGPU/hardware-window API 已对照 release 核实。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景是验收边界；表中带“历史”标记的 QQuickVTKItem/OpenGL 记录仅用于解释路线废止，不代表新实现验收。每次验证记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | 历史路线：Qt Quick `QQuickVTKItem` / OpenGL 方案，在 macOS 26 的 basic 与 threaded scenegraph 循环实测 | 两种窗口路径均崩溃，VTK/Qt API 契约无法满足；该路线删除，不作回退。原因与崩溃报告见本任务“前置条件已解除与历史阻塞”。 |
| 2026-09-19–20 | 切换 VTK 9.7 WebGPU hardware-window SDK；macOS 构建与 CTest | Release [sdk-vtk-9.7.0-webgpu](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-vtk-9.7.0-webgpu) 提供三平台 `RenderingWebGPU`/`RenderingUI`/Dawn targets；Cocoa bridge 编译，`CaeViewport` 创建与 SDK provision 测试通过，macOS CTest 29/29。 |
| 2026-09-20 | macOS surface 生命周期与覆盖层回归 | 延后到窗口可见后创建 surface，修正 AppKit flipped 坐标、Retina 逻辑尺寸和 WebGPU autoinit；用户窗口确认 VTK 可见且 QML 控件不再被遮挡。全套 resize/DPR/隐藏恢复/关闭重开验收仍未完成。 |
| 2026-09-20 | Linux/Windows bridge 与 CI 排障 | Linux 使用 Wayland `wl_subsurface`，Windows 使用 child HWND；修复 Wayland 开发依赖/模块发现及 Windows 缺 VTK DLL（`0xC0000135`）导致的 QML 测试启动问题。Windows 修复由 run [35510679878](https://github.com/Yuki-Nagori/panta/actions/runs/35510679878) 验证，CTest 27/27。 |
| 2026-09-20 | macOS 生命周期、更新策略与边界复审 | 尺寸未变或隐藏时跳过重复 render，显示恢复时补帧；VTK/Dawn 依赖收为 PRIVATE，surface 清理与 bridge 关闭构建均回归通过。此类自动化与离屏证据不替代实际窗口生命周期验收。 |
| 2026-09-24 | GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4`：三平台 native CTest | 通过：Linux 56/56、macOS ASan/UBSan 56/56、Windows 54/54。验证 SDK 消费与 QML/native 自动化回归，不代表真实图形窗口验收完成。 |
| 2026-09-26 | Windows 11 真实窗口交互验收（维护者手动操作 + SendInput 自动化双确认）：右键拖拽旋转、滚轮缩放 | 通过前发现缺陷：Windows 分支误用 `vtkGenericRenderWindowInteractor`（不接入平台消息流，子 HWND 的鼠标消息从未送达交互观察者）；改为 `vtkWin32RenderWindowInteractor` 后，子窗口（类 `vtkWin32`，1262×680）右键拖拽与滚轮缩放均改变渲染画面，维护者确认操作可用。配套运行库裸启部署见任务 083。 |
| 2026-09-26 | Windows 11；`panta_viewport_gpu_benchmark` 真实窗口 WebGPU 帧提交（30 预热 + 3×60 帧） | 测量通过（p50/p95 约 3.04-3.41/3.89-4.34 ms，登记于任务 048）；全部用例通过后退出阶段 0xC0000005。定位收敛：崩溃恒定发生在 `vtkWebGPURenderWindow::Finalize()` 内部，且仅当 `vtkWin32RenderWindowInteractor` 曾附加才触发（Generic interactor 全程干净；interactor 先析构、HWND 先销毁等排列组合均已实测不改变结果），属 VTK 内部缺陷，Release SDK 无符号需源码级排查；基准已改为 cleanupTestCase 显式拆除资源，崩溃被限定在 cleanup 段而不污染测量与后续静态析构 |
| 2026-09-26 | Windows 11；应用生命周期自动化验收（SendInput/Win32 驱动真实窗口） | 通过：resize 1000×700 与回原尺寸精确生效（1600×1000 被任务栏工作区钳制到 977 高，属系统行为）；最小化+恢复、隐藏+显示全程存活；WM_CLOSE 优雅关闭 exit 0（WebGPU "Device lost, reason=Destroyed" 正常日志）；重开再次创建主窗口，二次关闭 exit 0。高 DPI 变更需改显示器缩放，留待手动验收 |
| 2026-09-26 | Windows 11；基准退出崩溃处置 A/B 与修复 | A/B 实测：Generic interactor 退出 0、Win32 interactor 崩溃，崩溃定位在 QTEST_MAIN 标准退出经历的静态析构（应用自身优雅关闭 exit 0 不受影响，析构顺序重排无效）。基准改为自定义 main + qExec 后 `std::_Exit(status)` 跳过静态析构，退出码 0、数字不变（3.1502/3.8956 ms）；已知残留 "QDxgiVSyncService not destroyed in time" 提示为跳过析构的预期伴随 |

## 风险与回退

图形后端不匹配或 GUI 线程修改 VTK 状态可能造成黑屏/崩溃；先验证锁定版本的线程及重建路径，再增加场景功能。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：将 VTK 原生视口列入主线；最初等待适配的预编译 SDK，模块注册留给 026。
- 2026-09-19：macOS 26 上 Qt Quick `QQuickVTKItem` / OpenGL basic 与 threaded 两种循环均崩溃，废止该路线，转向 VTK WebGPU hardware window。
- 2026-09-19–20：038 发布三平台 WebGPU SDK 后，新增 Cocoa、Wayland、Win32 surface bridge 并恢复 `App.qml` 的实际视口；修复 macOS surface 生命周期、坐标/DPR 和控件层叠问题。
- 2026-09-20：确认锁定的 VTK 9.7.0 不含上游后续版本的 `GUISupportQtWebGPU`；当前继续使用硬件窗口桥接，SDK 升级再评估官方集成路线。
- 待验收：实际窗口下 resize、高 DPI、隐藏恢复、关闭重开、输入协调和资源释放；自动化 CTest 不替代这些图形验收。
- 2026-09-26：Windows 真实窗口交互验收发现并修复 interactor 选型缺陷；生命周期自动化验收通过 resize、最小化/恢复、隐藏/显示、关闭/重开与优雅退出（高 DPI 变更留待手动验收）。资源释放存在一项 VTK 内部缺陷已定位待源码级排查（见验证表），输入协调已随 065/007 验收完成。

## 完成摘要

WebGPU 原生视口与三平台 surface bridge 已实现，三平台 SDK 消费和 CTest 在 run 36001859191 通过。仍需真实图形窗口验收 VTK 内容可见性、resize、高 DPI、隐藏/恢复、关闭重开及输入/资源生命周期；CI 与 offscreen 创建测试不替代该验收。
