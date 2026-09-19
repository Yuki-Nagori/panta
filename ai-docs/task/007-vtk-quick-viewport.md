# 007 — VTK WebGPU/Cocoa 原生视口

- 状态：blocked
- 阶段：M0
- 依赖：[005](005-qt-qml-shell.md)（已完成：Qt Quick 主窗口与预编译 Qt 6.11.2 就绪）、[031](031-prebuilt-native-dependencies.md)（WebGPU/Cocoa 制品待生产）
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-19

## 阻塞（WebGPU/Cocoa SDK 待生产）

旧的 `QQuickVTKItem`/Qt OpenGL scenegraph 路线已经确认不适合作为当前实现：**窗口化渲染在 macOS 26（arm64，Qt 6.11.2 + OpenGLRhi）两种场景图循环下均崩溃**。该路线不再作为回退，也不保留兼容分支；待新的 WebGPU/Cocoa SDK 产出后，直接删除旧适配器、Qt VTK GUI 模块和失效引用。

- basic 循环（平台默认落到）：同步阶段（updatePaintNode）无当前 GL 上下文，glad 函数表加载为空（"Failed to initialize OpenGL functions" ×2），首次 `Render()` 段错误 pc=0x0——崩溃点为 dispatch_async 命令内的 Render 调用（崩溃报告 2026-09-19 18:14）。
- threaded 循环（显式 `QSG_RENDER_LOOP=threaded`）：更早崩溃——RHI 创建阶段 `NSOpenGLContext setView` 在非主线程被 AppKit 断言（SIGTRAP，崩溃报告 18:33）。
- VTK v9.7.0 源码核对：两种循环均无适配分支；加载器依赖 `QOpenGLContext::currentContext()` 在 updatePaintNode 时非空（threaded 循环契约）。

解除条件：038/031 产出并登记新的 VTK 9.7.0 WebGPU 制品；所有平台至少包含 `VTK::RenderingWebGPU`、`VTK::RenderingUI` 和 `vtkWebGPURenderWindow`，macOS arm64 资产另须包含 `vtkCocoaHardwareWindow` 与 `vtkCocoaHardwareView`。随后在 007 内完成原生 Cocoa view/Metal layer 与 Qt Quick 界面的叠加、尺寸同步和事件协调。诊断盲区：原生 SIGSEGV/SIGTRAP 在控制台零输出，历史证据只能取自 `~/Library/Logs/DiagnosticReports/*.ips`。

## 目标与背景

完成 M0 的关键图形集成：使用 VTK WebGPU render window 和 macOS Cocoa hardware view/layer，将 VTK 结果作为原生 Metal surface 嵌入 Qt Quick 界面，提供空视口和小型测试图元。

当前任务等待 038 生产、031 登记新的 VTK WebGPU/Cocoa SDK；实现前核对其 CMake targets、头文件和 ABI。不得在本地通过 FetchContent 或其他方式默认编译 VTK 源码。

## 必读

- [规范：vtk](../standards/vtk.md)
- [规范：qt](../standards/qt.md)
- [规范：qml](../standards/qml.md)
- [规范：cmake](../standards/cmake.md)
- [规范：cpp](../standards/cpp.md)
- [架构：visualization](../architecture/visualization.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小 WebGPU/Cocoa 原生视口基础设施。

非目标：不接 STEP/Netgen，不完成裁剪/标量等完整后处理，不开发自研 GPU backend，不再维护 `QQuickVTKItem` 或 Qt OpenGL scenegraph 集成。

## 前置条件与待决策

依赖 005 已完成，031/038 正在切换到新的 VTK WebGPU/Cocoa 制品。实施前核对预编译 VTK 9.7.0 SDK 的实际头文件、CMake targets、Apple/Windows/Linux ABI 和图形后端；缺少匹配包时保持 blocked，不启动本地源码构建。模块 URI 的正式自动注册仍由 026 负责，007 先沿用 005 的显式注册边界。

## 实施步骤

1. 核对锁定 VTK 的 `vtkWebGPURenderWindow`、`vtkCocoaHardwareWindow`、`vtkCocoaHardwareView` API、模块和 Metal surface 生命周期。
2. 确定 Qt Quick 窗口与原生 Cocoa view/layer 的承载、叠加、尺寸/高 DPI 同步和鼠标/触摸事件协调方式；不引入 Qt OpenGL scenegraph 集成。
3. 建立原生 `CaeViewport` 与最小 `RenderScene` 边界，在合法线程和生命周期回调中创建/更新/释放 WebGPU VTK 状态；只使用测试三角形等小数据。
4. 保留重建所需 CPU 状态，验证 native view/layer 重建、隐藏/恢复、窗口退出和 VTK 资源释放；完成后删除 `QQuickVTKItem`、`GUISupportQtQuick`、`RenderingQt` 及失效 OpenGL 引用。

## 预计改动

native/bridge/viewport、native/visualization/、QML 视口组件及 CMake；并删除旧 `QQuickVTKItem`/Qt OpenGL 集成及其 SDK 依赖。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

不引入兼容层或双渲染路径。实施时必须记录删除的旧实现、配置、依赖与失效引用；完成后仓库中不得保留 `QQuickVTKItem`、`GUISupportQtQuick`、`RenderingQt` 或旧 OpenGL adapter 的死代码。禁止以 `COMPAT` 标记长期保留旧路径。

## 验收标准

- [ ] 通过 cargo run 显示空视口与测试图元，VTK WebGPU 渲染错误不会静默表现为“已成功”。
- [ ] resize、高 DPI、隐藏/恢复和关闭/重开可用，记录平台与图形后端。
- [ ] VTK 对象没有从 GUI/worker 任意修改，所用 WebGPU/Cocoa API 已对照 release 核实。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。当前表中带“历史”标记的 QQuickVTKItem/OpenGL 记录仅用于解释路线废止，不代表新实现验收。新 SDK 产出后记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | 历史：锁定版本核对：`target/panta-deps/sdk/vtk/9.7.0/*/include/vtk-9.7/QQuickVTKItem.h` 与动态库清单 | QQuickVTKItem 存在且签名与 nightly 资料有差异；该 SDK 只支持旧 Qt/OpenGL 原型，不作为 WebGPU/Cocoa 路线输入 |
| 2026-09-19 | 历史：`cargo build --locked`（macOS arm64，managed LLVM/Qt/VTK） | 通过旧 QQuickVTKItem 原型编译链接；新 SDK 切换后需重新验证 |
| 2026-09-19 | 历史：窗口化 `./target/debug/panta-launcher`（macOS arm64，8 秒采样） | QML 加载无错误；VTK OpenGL 初始化输出 2 条告警，随后命中旧路径崩溃，故不作为当前路线验收 |
| 2026-09-19 | 历史：offscreen `panta-launcher -- -platform offscreen`（软件 scenegraph） | 旧 VTK OpenGL 路线显式报不支持并 abort；新 WebGPU/Cocoa 路线仍需另行定义无头测试边界 |
| 2026-09-19 | `cargo test --locked`（macOS arm64） | 全量通过（含新增 `Qml.ViewportModuleLoads`：Panta.Visualization 注册与 CaeViewport 可创建） |
| 2026-09-19 | run 35436415890 CI（三平台） | 供给消费与跨平台编译链接通过；macOS Qml.ShellModuleLoads/ViewportModuleLoads 失败：`module "Panta.Visualization" is not installed`——静态模块消费方二进制未导入 plugin；已修复（shell 测试导入/链接，后续 App.qml 回退后仅 viewport 测试消费） |
| 2026-09-19 | macOS 26 窗口化崩溃取证（.ips 分析 + QSG_INFO + VTK v9.7.0 源码核对） | basic 循环：updatePaintNode 无当前上下文 → glad 空表 → dispatch 内 Render 段错误（pc=0x0）；threaded 循环：RHI 创建期 NSOpenGLContext setView SIGTRAP。两种循环均崩溃，任务转 blocked（解除条件见上） |
| 2026-09-19 | macOS arm64；`cargo build --locked` 后直接运行 `target/native/debug/app/panta_qml_viewport_module_test`（Qt 6.11.2，offscreen） | 通过，3/3；`Panta.Visualization` 静态模块可加载并创建 `CaeViewport`。该创建级结果不替代已记录的窗口化渲染阻塞 |
| 2026-09-19 | WebGPU/Cocoa 路线与 SDK 生产契约核对 | 旧 `sdk-vtk-9.7.0` 只满足 `GUISupportQtQuick`/`RenderingQt`；新制品配置改为 `VTK_ENABLE_WEBGPU=ON`、`RenderingWebGPU` + `RenderingUI`，发布 tag 为 `sdk-vtk-9.7.0-webgpu`，待维护者 dispatch 后补 URL/SHA256 与实际头文件验证 |

## 风险与回退

图形后端不匹配或 GUI 线程修改 VTK 状态可能造成黑屏/崩溃；先验证锁定版本的线程及重建路径，再增加场景功能。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：开始核对主线视口 API，原候选为 VTK 9.7.0 `GUISupportQtQuick` 的 `QQuickVTKItem`；用户决定 native 第三方库预编译优先，因此暂停实现，等待匹配 SDK。正式模块化注册留给 026。
- 2026-09-19：旧 Release `sdk-vtk-9.7.0` 的 Qt/OpenGL 制品完成历史消费验证，但 macOS 26 窗口化渲染双循环崩溃，不能作为当前集成路线。
- 2026-09-19（实施批次）：Panta.Visualization 静态模块 + ViewportBackend 适配层 + src/vtk/ 收敛（详见提交 f55c14f）；CI 三平台供给消费与编译链接通过。
- 2026-09-19（转 blocked）：macOS 26 窗口化渲染双循环崩溃（证据与解除条件见"阻塞"节）；App.qml 集成临时回退为占位面板，模块/注册/创建级测试保留（CI 绿）。App 恢复不崩溃。
- 2026-09-19（路线切换）：放弃 Qt OpenGL/QQuickVTKItem 场景图集成，改为 VTK WebGPU render window + `vtkCocoaHardwareWindow`/`vtkCocoaHardwareView`，通过原生 Metal surface 与 Qt Quick 界面叠加；038 先生产 `sdk-vtk-9.7.0-webgpu`，031 在取得 URL/SHA256 后再切换消费 manifest。
- 待记录：新 SDK 实际头文件/targets、原生 view/layer 嵌入方式、事件协调和 macOS 视觉验证。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
