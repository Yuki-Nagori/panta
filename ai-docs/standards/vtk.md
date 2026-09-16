# VTK、Qt Quick 与渲染数据

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 VTK adapter 和 `CaeViewport`；VTK 不是公共数据模型。`QQuickVTKItem` 为优先验证候选，正式采用须在任务 007 验证锁定版本。

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
