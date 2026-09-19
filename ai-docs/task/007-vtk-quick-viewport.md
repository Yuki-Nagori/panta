# 007 — VTK 原生 Qt Quick 视口

- 状态：in-progress
- 阶段：M0
- 依赖：[005](005-qt-qml-shell.md)（已完成：Qt Quick 主窗口与预编译 Qt 6.11.2 就绪）、[031](031-prebuilt-native-dependencies.md)（供给侧已就绪）
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-19

## 目标与背景

完成 M0 的关键图形集成：QML 中的空视口和小型测试图元。

当前任务等待预编译 VTK SDK 供给；实现前先由 031 固定平台包、CMake package 和 ABI 匹配。不得在本地通过 FetchContent 或其他方式默认编译 VTK 源码。

## 必读

- [规范：vtk](../standards/vtk.md)
- [规范：qt](../standards/qt.md)
- [规范：qml](../standards/qml.md)
- [规范：cmake](../standards/cmake.md)
- [规范：cpp](../standards/cpp.md)
- [架构：visualization](../architecture/visualization.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不接 STEP/Netgen，不完成裁剪/标量等完整后处理，不开发自研 GPU backend。

## 前置条件与待决策

依赖 005 已完成，031 尚未完成。实施前核对预编译 VTK 9.7.0 SDK 的实际头文件、CMake targets、Qt 6.11.2 ABI 和图形后端；缺少匹配包时保持 planned，不启动本地源码构建。模块 URI 的正式自动注册仍由 026 负责，007 先沿用 005 的显式注册边界。

## 实施步骤

1. 核对锁定 VTK 的 QQuickVTKItem API、模块与 Qt 图形后端，记录采用或替代方案。
2. 确定创建应用对象前的图形初始化顺序，建立原生 CaeViewport 与最小 RenderScene 边界。
3. 通过合法渲染线程回调创建/更新/释放 VTK 状态；只使用测试三角形等小数据。
4. 保留重建所需 CPU 状态，验证场景节点销毁/重建及窗口退出时的排队命令。

## 预计改动

native/bridge/viewport、native/visualization/、QML 视口组件及 CMake。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] 通过 cargo run 显示空视口与测试图元，渲染错误不会静默表现为“已成功”。
- [ ] resize、高 DPI、隐藏/恢复和关闭/重开可用，记录平台与图形后端。
- [ ] VTK 对象没有从 GUI/worker 任意修改，所用 API 已对照 release 核实。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | 锁定版本核对：`target/panta-deps/sdk/vtk/9.7.0/*/include/vtk-9.7/QQuickVTKItem.h` 与动态库清单 | QQuickVTKItem 存在且签名与 nightly 资料有差异：`vtkUserData = vtkSmartPointer<vtkObject>`、图形 API 入口为 `setGraphicsApi()`（运行期错误文本仍写 setupGraphicsApi，保留字）；`initializeVTK/destroyingVTK/dispatch_async` 契约与规范一致，按实际头文件实现 |
| 2026-09-19 | `cargo build --locked`（macOS arm64，managed LLVM/Qt/VTK） | 通过：native/visualization 模块（Panta.Visualization 静态 QML 模块）编译链接，app 链接 VTK dylib 链（rpath 指向 031 staging） |
| 2026-09-19 | 窗口化 `./target/debug/panta-launcher`（macOS arm64，8 秒采样） | QML 加载无错误；VTK OpenGL 初始化输出 2 条 "Failed to initialize OpenGL functions" 告警——渲染错误可见、不静默；**渲染结果待维护者视觉确认** |
| 2026-09-19 | offscreen `panta-launcher -- -platform offscreen`（软件 scenegraph） | VTK 显式报不支持并 abort（API 1 = software）——无头/软件后端不受支持且错误不静默，真实视口冒烟以窗口化运行为准（vtk.md） |
| 2026-09-19 | `cargo test --locked`（macOS arm64） | 全量通过（含新增 `Qml.ViewportModuleLoads`：Panta.Visualization 注册与 CaeViewport 可创建） |

## 风险与回退

图形后端不匹配或 GUI 线程修改 VTK 状态可能造成黑屏/崩溃；先验证锁定版本的线程及重建路径，再增加场景功能。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：开始核对主线视口 API，确认候选为 VTK 9.7.0 `GUISupportQtQuick` 的 `QQuickVTKItem`；用户决定 native 第三方库预编译优先，因此暂停实现，等待 031 提供匹配 SDK。正式模块化注册留给 026。
- 2026-09-19：031 供给侧就绪（Release `sdk-vtk-9.7.0` 三平台 manifest 全部登记并经 macOS 生产消费烟测），解除"等待 031 SDK"的暂停，状态转 ready。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
