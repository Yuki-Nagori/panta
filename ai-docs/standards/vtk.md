# VTK、Qt Quick 与渲染数据

查阅日期：2026-09-16；007 实测回填：2026-09-19。状态：锁定版本（VTK 9.7.0 + Qt 6.11.2）已按本规范完成视口集成验证（渲染结果视觉确认待维护者记录）。

适用于 VTK adapter 和 `CaeViewport`；VTK 不是公共数据模型。`QQuickVTKItem` 已在锁定版本验证采用（007）。

## 9.7.0 实测细节（007，以 SDK 头文件为准）

- `QQuickVTKItem::vtkUserData` 是 `vtkSmartPointer<vtkObject>` 别名（非独立基类、非裸指针）；回调内经 `SafeDownCast` 恢复自有场景数据（`vtkObject` 派生 + `vtkStandardNewMacro`）。
- 图形 API 入口实际名为 `setGraphicsApi()`，必须在 `QGuiApplication` 构造前调用；运行期错误文案仍写 `setupGraphicsApi`，属上游不一致，以头文件为准。
- 031 最小组件集不含全部 FiltersSources 头：`vtkTriangleSource.h` 缺失、`vtkSphereSource.h` 可用；新增图元前先核对 SDK include 与 manifest 登记，不整包拉取 VTK。
- 后端边界落地（architecture/visualization.md）：`CaeViewport`（QQuickItem 子类，公共头自包含、零第三方类型）+ `ViewportBackend` 接口 + `src/vtk/` 适配器；QML 类型注册要求被注册头自包含（注册器不解析第三方传递包含），且被注册类不可 `final`。
- 无头限制：offscreen/软件 scenegraph 不受 VTK 支持，运行即显式报错或中止（不静默）；无头测试仅覆盖 QML 注册与类型可创建，视口冒烟必须窗口化并人工记录。
- QML 静态模块消费规则：凡加载引入了某静态模块 QML 的二进制/测试，必须链接并 `Q_IMPORT_QML_PLUGIN` 该模块 plugin，否则运行时报 "module not installed"。
- macOS 实测（26.3/arm64）：OpenGLRhi 下可创建 GL 4.1 Core 上下文，模块注册与场景构建信号正常；但**窗口化渲染在两种场景图循环下均崩溃**——basic 循环同步阶段无当前上下文，glad 函数表为空，首次 Render 段错误（pc=0x0，经 dispatch_async 命令触发）；threaded 循环在 RHI 创建阶段即被 AppKit 断言（`NSOpenGLContext setView` SIGTRAP）。VTK 上游对两种循环均无适配分支（v9.7.0 源码核对），macOS 26 渲染路径待专项解决（007 记录解除条件）。

## 官方依据

当前 `QQuickVTKItem` 文档规定相关 VTK 状态在渲染线程访问，通过初始化、销毁及排队命令处理；场景节点销毁后可能重建。其图形 API 设置须发生在应用对象创建前。[QQuickVTKItem API（nightly）](https://vtk.org/doc/nightly/html/classQQuickVTKItem.html)

VTK 的 C++ 接入文档使用 CMake 与模块依赖。[Using C++ and CMake](https://docs.vtk.org/en/latest/getting_started/using_cpp.html)

Qt Quick 场景图存在不同渲染循环与图形资源生命周期。[Qt Quick Scene Graph](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html)

## 项目规则

- nightly 资料仅用于设计线索，任务 007 必须对照选定 release 的实际头文件/API；若接口不同，记录替代方案，不临时切到未锁定版本。
- 初始化时先确定图形后端和所需格式，验证与 Qt 构建一致；不默认各平台 Qt 的首选 backend 一定适合 VTK。
- 通过 `initializeVTK`、`destroyingVTK`、`dispatch_async` 等经确认的机制操作视口 VTK 状态，GUI/worker 不直接修改 actor、mapper 或 pipeline。
- RenderScene 保存可恢复的应用状态，节点重建时重建渲染对象；任务完成回调携带场景修订以拒绝迟到更新。
- CMake 显式声明所需模块，核实静态/动态构建所需模块初始化；不为简化链接而无条件拉入整个 VTK。
- Mesh/Field 转换检查整数范围、点/单元关联、分量数和数据长度。零拷贝须证明底层数组有效期，否则优先有界复制。
- VTK 引用计数资源按 VTK 规则管理；显示裁剪和抽稀不修改分析输入。相机、色标、单位和缺失值策略由应用模型定义。

## 验证

空视口和测试三角形分别验证；检查 resize、高 DPI、隐藏/恢复、场景重建、关闭时排队命令，以及退出时无访问已释放资源。软件/无头图形测试不能替代真实目标平台的视口冒烟。
