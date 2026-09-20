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

前置条件已满足：038/031 已产出并登记 `sdk-vtk-9.7.0-webgpu`，三平台包含 `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn` 和对应硬件窗口（Cocoa、Wayland、Win32）。Linux 制品明确为 `VTK_USE_X=OFF`、`VTK_USE_Wayland=ON`，纯 X11 环境不属于该资产的运行目标；若产品需要纯 X11，必须另产 X11 变体。当前进入 007 的原生 surface/view 与 Qt Quick 界面叠加、尺寸同步和事件协调实现阶段。诊断盲区：原生 SIGSEGV/SIGTRAP 在控制台零输出，历史证据只能取自 `~/Library/Logs/DiagnosticReports/*.ips`。

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
| 2026-09-19 | 历史：锁定版本核对：`target/panta-deps/sdk/vtk/9.7.0/*/include/vtk-9.7/QQuickVTKItem.h` 与动态库清单 | QQuickVTKItem 存在且签名与 nightly 资料有差异；该 SDK 只支持旧 Qt/OpenGL 原型，不作为当前 WebGPU 硬件窗口路线输入 |
| 2026-09-19 | 历史：`cargo build --locked`（macOS arm64，managed LLVM/Qt/VTK） | 通过旧 QQuickVTKItem 原型编译链接；新 SDK 切换后需重新验证 |
| 2026-09-19 | 历史：窗口化 `./target/debug/panta-launcher`（macOS arm64，8 秒采样） | QML 加载无错误；VTK OpenGL 初始化输出 2 条告警，随后命中旧路径崩溃，故不作为当前路线验收 |
| 2026-09-19 | 历史：offscreen `panta-launcher -- -platform offscreen`（软件 scenegraph） | 旧 VTK OpenGL 路线显式报不支持并 abort；新 WebGPU 硬件窗口路线仍需另行定义无头测试边界 |
| 2026-09-19 | `cargo test --locked`（macOS arm64） | 全量通过（含新增 `Qml.ViewportModuleLoads`：Panta.Visualization 注册与 CaeViewport 可创建） |
| 2026-09-19 | run 35436415890 CI（三平台） | 供给消费与跨平台编译链接通过；macOS Qml.ShellModuleLoads/ViewportModuleLoads 失败：`module "Panta.Visualization" is not installed`——静态模块消费方二进制未导入 plugin；已修复（shell 测试导入/链接，后续 App.qml 回退后仅 viewport 测试消费） |
| 2026-09-19 | macOS 26 窗口化崩溃取证（.ips 分析 + QSG_INFO + VTK v9.7.0 源码核对） | basic 循环：updatePaintNode 无当前上下文 → glad 空表 → dispatch 内 Render 段错误（pc=0x0）；threaded 循环：RHI 创建期 NSOpenGLContext setView SIGTRAP。两种循环均崩溃，任务转 blocked（解除条件见上） |
| 2026-09-19 | macOS arm64；`cargo build --locked` 后直接运行 `target/native/debug/app/panta_qml_viewport_module_test`（Qt 6.11.2，offscreen） | 通过，3/3；`Panta.Visualization` 静态模块可加载并创建 `CaeViewport`。该创建级结果不替代已记录的窗口化渲染阻塞 |
| 2026-09-20 | Release [sdk-vtk-9.7.0-webgpu](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-vtk-9.7.0-webgpu) 三平台归档核对 | 通过：`panta-sdk.json` 确认 C++20/WebGPU、Cocoa/Wayland/Win32、`VTK::RenderingWebGPU`/`VTK::RenderingUI`/`dawn::webgpu_dawn` 与对应 hardware window；031 已回填 URL/SHA256，前置阻塞解除 |
| 2026-09-20 | macOS arm64；`cmake --preset debug` + `cmake --build target/native/debug --target panta_native_app panta_qml_viewport_module_test` | 通过：新 SDK 被 native 工程消费，ObjC++ Cocoa/Metal bridge 编译，`App.qml` 实际创建 `CaeViewport`；链接无 `GUISupportQtQuick`/OpenGL VTK 依赖 |
| 2026-09-20 | `ctest --test-dir target/native/debug -R '^Qml\\.ViewportModuleLoads$' --output-on-failure` 与 `-R '^Build\\.SdkProvision$'` | 通过：QML 类型创建 1/1、SDK provision 1/1；offscreen 实际启动不再进入原生 surface，输出明确告警而无崩溃 |
| 2026-09-20 | macOS arm64；`cmake --build target/native/debug -j4` + `ctest --test-dir target/native/debug --output-on-failure` | 通过：29/29；Shell 模块消费方补齐 `Panta.Visualization` 静态 plugin 导入/链接，`App.qml` 的真实视口调用可加载；默认球体半径 0.5、颜色 `(0.18, 0.58, 0.95)`、相机位置 `(0, 0, 4.5)`、视角 30°、裁剪范围 `0.1–100` 已固定；VTK 浅色背景为 `(232, 238, 247)`，与 QML 背景区分 |
| 2026-09-20 | macOS 窗口首帧时序 review；`VtkViewport` 暴露前后的初始化路径 | 已修复：不再在 QML `componentComplete()` 的未显示窗口阶段创建 WebGPU surface；监听 `QWindow::visibleChanged` 并在事件循环后重试，确保原生 view 已进入可显示窗口后才初始化/首帧渲染。当前本机无可用屏幕，真实球体可见性待有屏幕 macOS 环境复验 |
| 2026-09-20 | macOS 真实窗口回归；用户运行 `cargo run` | 首次同步 attach 在窗口可见性切换栈内触发 `attach_native_surface` SIGSEGV；改为可见性事件排队到下一轮事件循环，并以 `QQuickWindow::sceneGraphInitialized` 作为补充触发，避免过早访问 Cocoa/Metal view，同时不因 `isExposed()` 没有对应信号而漏掉初始化；用户复验 VTK 已显示 |
| 2026-09-20 | macOS arm64；对照 VTK 9.7 SDK `VTK-vtk-module-properties.cmake` | 发现 `VTK::RenderingWebGPU` 标记 `INTERFACE_vtk_module_needs_autoinit=1`，消费侧缺少 `vtk_module_autoinit` 会出现“设备/帧提交成功但没有几何体”；已补齐 `RenderingWebGPU`、`RenderingUI`、`RenderingCore`、`FiltersSources` 自动初始化 |
| 2026-09-20 | macOS Cocoa surface frame review；`cmake --build target/native/debug -j4` + `ctest --test-dir target/native/debug --output-on-failure` | 通过：29/29；`vtkCocoaHardwareWindow::SetSize()` 改用逻辑点尺寸并在恢复 `QQuickItem` frame 前调用，关闭原生 view 自动伸缩，避免 Retina backing pixel 尺寸和 VTK frame 重置导致原生 view 覆盖整个 Qt Quick 窗口；真实窗口叠加效果仍需在有屏幕的 macOS 环境复验 |
| 2026-09-20 | 三端 surface 结构 review；VTK 9.7 Wayland hardware-window API 对照 | 代码已实现：macOS 使用固定尺寸 Cocoa view，Windows 使用固定尺寸 child HWND，Linux 不再把 Qt 顶层 `wl_surface` 直接交给 VTK，改为创建带位置/尺寸同步的 `wl_subsurface`；Linux consumer 增加 `libwayland-dev` 与 `wayland-client` 链接。macOS 29/29 通过，Linux/Windows 编译与真实窗口运行待对应 CI/目标环境验证 |
| 2026-09-20 | macOS；`cargo build --locked --no-default-features --package panta-launcher` + `ctest --test-dir target/native/debug --output-on-failure` | 通过：无 Bridge 消融 9/9；修复 app 进程级 crash FFI 依赖未公共链接、视口测试在 Bridge 关闭时仍无条件链接的问题；随后恢复默认 Bridge 构建并复验 29/29 |
| 2026-09-20 | macOS；`cargo test --locked --workspace` | 通过：Rust 单元/集成、CXX/Qt native suite 和 QML lint 全部通过；crash handler 测试按预期产生 SIGSEGV/UNKNOWN 诊断日志并通过 |
| 2026-09-20 | CI run 35490867113 取证（ubuntu 全部 native job 失败于 native configure） | `FindWAYLAND.cmake:56`（038 捆绑进 VTK SDK）经 pkg-config 报缺 `wayland-protocols`，宿主前置只装了 `libwayland-dev`；`linux-gl-prereqs` 已补装该包。宿主 CI 环境无法本地复跑，待 push 后 CI 复验 |
| 2026-09-20 | 同 run Windows 侧取证：`cargo check` 阶段 bridge 三测试 POST_BUILD discovery 链接后立即启动测试可执行文件，退出码 0xc0000135（构建期子进程 PATH 无托管 Qt runtime）；本地另证 PATH 上 ctest 4.3.3 无法消费 PRE_TEST 清单（`discover_tests` 为 CMake 4.4 内建命令），证实 a7e57ed 弃用 PRE_TEST 的动机 | 修复：launcher 构建脚本对 CMake build 子进程注入 `native_test_env`（Windows 前置托管 Qt runtime PATH，其余平台零变化），POST_BUILD 发现在统一入口下恢复可用且测试清单保留任意 ctest 可消费的纯 `add_test` 形式；bridge CMake 补注释固定约束。本地 macOS `cmake --build` + 托管 ctest 4.4.3 与 PATH ctest 4.3.3 均 29/29 通过；Windows 待 CI 复验 |
| 2026-09-20 | Review 修复批 1（健壮性）；`cmake --build target/native/debug -j8` + `ctest --output-on-failure` | 通过 29/29：`itemChange` 补 `ItemDevicePixelRatioHasChanged` 分支（跨显示器/缩放变化重算像素尺寸，033 前置）；`ensure_render_window` 告警改每实例一次（offscreen/失败重试不再刷屏）；`ItemSceneChange` 改用文档化载荷 `value.window` 的语义并延后到事件循环（与窗口信号共用合并后的 `schedule_refresh`，重复信号折叠为一次尝试）；`apply_state` 守卫补齐 `primitive_actor`；attach 失败清理 `native_surface` 残留；冗余空指针条件去除 |
| 2026-09-20 | Review 修复批 2（渲染提交策略）；`cmake --build target/native/debug -j8` + `ctest --output-on-failure` | 通过 29/29：`sync_native_surface` 缓存最近提交的像素尺寸，尺寸未变跳过 `SetSize`+`Render`（创建路径的双重渲染随之消除）；隐藏条目不再做渲染提交（`apply_state` 与 sync 同策略），重新显示时由可见分支强制补一帧；`CaeViewport` 宿主条目位置在构造时固定为原点，移除每次几何变化的重复 `setPosition` 与恒真的 backend 判空 |
| 2026-09-20 | Review 修复批 3（边界与去重）；`cmake --build target/native/debug -j8` + `ctest --output-on-failure` + `cargo lint cppcheck`/`includes` | 通过 29/29，lint 通过：`panta_visualization` 的 VTK/Dawn 依赖改为 PRIVATE 链接——公共头只有 Qt 类型，不再向消费方泄漏第三方 include 路径（静态库符号经 LINK_ONLY 传播，shell/viewport 测试与 app 链接验证通过），`Qt6::Quick` 因公共头基类保持 PUBLIC；三平台重复的删除器函数收敛为 `vtk_native_surface.hpp` 的 `HardwareWindowDeleter` functor + `NativeHardwareWindow` 别名（消除函数指针删除器与空删除器隐患；首版头内联自由函数被 cppcheck unusedFunction 误报，改成员函数后通过）；Wayland 重映射对 `Render()` 的依赖补注释固定，列入真实环境验证项 |
| 2026-09-20 | 修复系列全量门禁；macOS `cargo test --locked --workspace` | 通过：18 个测试目标全 ok，含 native CTest 29/29（shell/viewport 模块加载、QML 行为、bridge/foundation/ffi）；每个提交均通过 pre-commit 的 format/clippy/machete/cmake/qmllint/clang-tidy/includes/cppcheck 门禁。Linux（wayland-protocols）与 Windows（构建期 discovery PATH）行为待 push 后 CI 复验 |
| 2026-09-20 | macOS 真实窗口用户反馈"只有 VTK 可见，QML 其余内容缺失"；临时插桩打印 item/scene/host/frame 实测值（验证后移除） | 根因：Qt 的 content view（`QWindow::winId`）是 flipped 坐标系（原点左上），桥接却按 AppKit 默认底部原点做了 y 翻转，原生 view 被放到窗口顶部、盖住 label/button——实测 frame(24,24,1422,636) 而非 (24,149,1422,636)。修复按 `[host isFlipped]` 自适应：flipped 宿主直接使用 QML scene y。修复后实测 frame=(24,149 1422x636)，与 item 区域一致；`cmake --build` + ctest 29/29 通过，视觉叠加待用户窗口复验 |
| 2026-09-20 | 二遍 review 清理；`cmake --build` + ctest 29/29 + `cargo lint includes`/`cppcheck` | 通过：`ensure_render_window` 的 device 初始化失败告警并入 `warn_once`（重试不再逐次输出）；删除 `destroy_render_window` 恒假的 `impl_` 判空死防御；`SceneRevision` 收敛为 `std::uint64_t`；visualization 模块内英文解释性注释按 comments.md 统一为中文（文件头与局部原因注释，标识符保留原文） |
| 2026-09-20 | CI run 35502271608 Linux 侧：configure 依赖修复确认生效（`Found wayland-protocols 1.45`、xkbcommon 命中），Linux 分支 `vtk_native_surface.cpp` 首次参与编译报 2 个错误 | 修复：补 `#include <vtkObjectFactory.h>`（`vtkStandardNewMacro` 未定义）；`nativeInterface<QWaylandApplication>()` 的兼容约束按接收者静态类型解析、`QCoreApplication*` 不满足，落点改为 `QGuiApplication*`；去除 vtkTypeMacro 多余分号（-Wextra-semi）。本机无 Linux 工具链无法本地编译该 TU，对照 clang 诊断逐条修复，待 CI 复验 |
| 2026-09-20 | 同 run Windows 侧：`cargo check/build` 通过（构建期 discovery PATH 修复生效）；Test 步 `Qml.ShellModuleLoads`/`Qml.ViewportModuleLoads` 各超时 1500s，进程零输出（QTest 横幅未出现，同环境主题 QML 测试通过）——挂点在静态初始化或 QGuiApplication 构造，且只出现在链接 Panta.Visualization 静态 plugin 的测试 | 两个测试改为显式 main 逐阶段冲刷 stderr（reached main / QGuiApplication ready / finished，TODO(task 007) 根因确认后移除）；`panta_add_qml_test` 加 TIMEOUT 120 兜底，挂起不再拖 50 分钟。下一轮 CI 取证定位挂点后修根因 |
| 2026-09-20 | CI 取证梳理：run 35502271608（09:25）实际 head 为 3eca8fd（不含 Linux 修复），其 Linux 编译错误属旧代码；29be64b 的 run 35503727247 跑 5 分钟即被取消，修复与诊断标记从未在 CI 跑完 | 待推：细化诊断粒度——两个测试加 TU 静态初始化探针、engine/组件创建阶段标记、平台名与 QPA 环境打印；cae_viewport/vtk_viewport 构造与 componentComplete 加模块内标记。macOS 本地验证标记链完整（TU static init → main → QGuiApplication → engine → ctor → created → finished），29/29 通过。Windows 下一轮 CI 按最后一条标记定位挂点 |
| 2026-09-20 | CI run 35505448992 取证：两测试各 120s 超时且 `--output-on-failure` 下零输出——TU 静态初始化探针（main 之前、链接序最前）也未执行，挂点先于任何自有代码；Linux 侧新增 includes/clang-tidy 失败集中在 `vtk_native_surface.cpp`（Linux 分支首次进编译数据库） | 字节级核查 Windows SDK 归档：webgpu_dawn.lib 981 个成员及各 VTK 库均无 `.CRT$XCU` 静态初始化段，Dawn 接口/内嵌 DEFAULTLIB 仅系统 DLL（user32/ole32/advapi32/dbghelp 等）——排除第三方静态初始化死循环与缺 DLL。行动：移除全部探针与诊断 main（恢复 QTEST_MAIN、删 `<cstdio>`）；CI Windows job 构建前关闭 Defender 实时监控，Test 前直接拉起两个 exe 取退出码与 stderr（0xc0000135/0xc0000005/挂起三分类）；`vtk_native_surface.cpp` 按 include-cleaner 拆 wayland 子头并补 `cstdint`/`vtkSetGet.h`/`vtkWindow.h`，`set_geometry` 参数收敛为 QPointF/QSize，VTK 宏 NewInstance 遮蔽与刻意跳过父类 SetSize 以 NOLINT 注明惯例豁免。macOS 29/29、format/includes/clang-tidy/cppcheck 通过；Linux 段头映射与 Windows 缓解待 CI 复验 |
| 2026-09-20 | CI run 35508804535 取证（轮 4）：Defender 豁免未解除挂起（Qml.ShellModuleLoads 仍 120s 超时）——Defender 理论证伪；取证步修复 exe 路径后首次产出关键数据：Windows 上 exe 位于构建根（非 `app/` 子目录），直接拉起 **60ms 内静默退出**（stdout/stderr 零字节，退出码因 Start-Process 缺省 ShellExecute 拿不到句柄未捕获），而 ctest 拉起同一 exe 却永久挂死——同一 exe 两种结局，差异聚焦 stdio 形态与进程句柄获取方式 | Defender 豁免保留但改注释为构建加速用途；取证步升级：llvm-readobj 导出 PE 导入表、控制台/管道 stdio 矩阵（.NET Process 真实句柄 + 有界 + 杀树回读部分输出）、WER 事件按 ProviderName 过滤（Id 过滤只命中 WMI 噪音） |
| 2026-09-20 | CI run 35509372887/35509979762 取证（轮 5/6，根因确认）：Windows 测试 exe 实际位于构建根且仅 1.6MB/105KB（本轮新链接，非陈旧产物）——不含任何 VTK/Dawn 静态代码；CTestTestfile 注册路径即该 exe；管道 stdio 直接拉起退出码 -1073741515 = 0xC0000135 STATUS_DLL_NOT_FOUND | 根因：038 的 Windows VTK SDK 是共享库构建（bin/vtk*-9.7.dll，Dawn 静态编入 VTK DLL），macOS/Linux 同为共享但消费方经构建 rpath 解析，Windows 只认 PATH——测试进程 PATH 无 SDK bin 目录，加载器缺 DLL；交互会话的 missing-DLL 硬错误使进程不初始化即被卡住，表现为 ctest 超时零输出（历轮全部观察自此闭环）。修复：`native_test_env` 在 Windows 把 `panta-deps/sdk/*/*/windows-*/bin` 前置到 PATH（附单元测试，与 Qt staging 同一机制）；QML 测试 TIMEOUT 120→20；取证步新增导入完整性自检（readobj 导入表逐 DLL 核对 PATH+System32）并按修复后环境直接拉起注册 exe。macOS panta-build 测试/format/clippy 通过；Windows 行为待下轮 CI 复验。031 待记录：Windows SDK 为 DLL 形态，产品 app 运行时需要同样的 DLL 分发策略 |

## 风险与回退

图形后端不匹配或 GUI 线程修改 VTK 状态可能造成黑屏/崩溃；先验证锁定版本的线程及重建路径，再增加场景功能。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：开始核对主线视口 API，原候选为 VTK 9.7.0 `GUISupportQtQuick` 的 `QQuickVTKItem`；用户决定 native 第三方库预编译优先，因此暂停实现，等待匹配 SDK。正式模块化注册留给 026。
- 2026-09-19：旧 Release `sdk-vtk-9.7.0` 的 Qt/OpenGL 制品完成历史消费验证，但 macOS 26 窗口化渲染双循环崩溃，不能作为当前集成路线。
- 2026-09-19（实施批次）：Panta.Visualization 静态模块 + ViewportBackend 适配层 + src/vtk/ 收敛（详见提交 f55c14f）；CI 三平台供给消费与编译链接通过。
- 2026-09-19（历史阻塞）：macOS 26 窗口化渲染双循环崩溃（证据见"前置条件已解除与历史阻塞"节）；App.qml 集成临时回退为占位面板，模块/注册/创建级测试保留（CI 绿）。App 恢复不崩溃。
- 2026-09-19（路线切换）：放弃 Qt OpenGL/QQuickVTKItem 场景图集成，改为 VTK WebGPU render window + 平台 hardware window；macOS 使用 `vtkCocoaHardwareWindow`/`vtkCocoaHardwareView` 通过原生 Metal surface 与 Qt Quick 界面叠加，Linux 使用 Wayland，Windows 使用 Win32。
- 2026-09-20：038 完成 `sdk-vtk-9.7.0-webgpu` 三平台 Release，031 已登记新资产；007 转入原生 surface/view 嵌入与事件协调实现。
- 2026-09-20：native/visualization 删除旧 QQuickVTKItem 适配器，新增 WebGPU render window 与 macOS Cocoa、Windows Win32、Linux Wayland surface bridge；`App.qml` 恢复实际 CaeViewport 调用。macOS 无屏幕环境的启动验证发现并修复 offscreen surface 误用导致的 SIGSEGV，崩溃日志路径按任务 047 记录。
- 2026-09-20（Windows 挂起取证轮 3）：探针全链零输出且第三方静态库无静态初始化段 ⇒ 挂点在 CRT 启动之前，不属于本仓库代码路径；维护者无 Windows 机器，缓解与继续取证转 CI 侧（Defender 实时监控豁免 + 绕过 ctest 直接拉起 exe 按退出码三分类）。若下轮 Windows 仍挂起，以诊断步输出为准继续，不再加进程内探针。
- 2026-09-20（官方集成路线核对）：上游 VTK 新增 `VTK::GUISupportQtWebGPU`（`QVTKWebGPUWidget`：widget 自身设原生窗口，直接以 `winId()` 构造 `WGPUSurfaceSourceWindowsHWND`/CAMetalLayer/Wayland surface 描述符经 `SetCustomSurfaceDescriptor` 交给 render window，不用 hardware window/子 HWND/几何同步）——但该模块与 API 在 v9.7.0 tag 均不存在，且本仓库 SDK 未编译任何 GUISupportQt* 模块；`QQuickVTKItem`（Qt Quick，OpenGL2）上游仍在但 9.7 SDK 未含，官方也没有 Qt Quick + WebGPU 类。当前实现符合 9.7 hardware-window 官方契约；是否升级 SDK 转官方 widget 路线列为 031/007 待决策项（Qt Quick 侧仍需自建 QQuickItem 包装，官方只提供 QWidget）。
- 待记录：目标平台真实窗口下的 resize、高 DPI、隐藏/恢复、输入协调和重开验证；Windows 桥接中 `hardware->SetSize()` 与 `SetWindowPos` 的宽高双写是否冗余（本机仅 macOS SDK，无法核对 `vtkWin32HardwareWindow` 头）。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
