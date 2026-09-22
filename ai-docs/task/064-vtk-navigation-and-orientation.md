# 064 — VTK 视口导航、笛卡尔坐标系与六面体定位

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[063](063-stl-import-and-mesh-workspace.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

扩展当前 VTK 视口的三维导航能力：在视口右下角显示跟随相机方向的笛卡尔坐标系，在右上角显示可交互的六面体定位器，并支持通过视口拖拽旋转整体几何视图。目标是让用户能够快速理解当前空间方向、跳转到标准正交方向，并围绕模型进行连续观察。

这里的“整体几何体旋转”首期解释为相机围绕当前焦点的轨道 / trackball 旋转：不修改 STL 顶点、Mesh IR 或工程 import record，不产生工程 dirty，也不把每一帧鼠标移动写入 `.panta`。如果后续需要真正修改几何坐标或保存建模变换，另建独立任务。

当前视口使用 VTK WebGPU 原生 surface；原生渲染层位于 Qt Quick 内容之上，QML 不能直接覆盖视口矩形。因此坐标系和六面体必须优先评估 VTK 同一 render window 内的 overlay / orientation marker 能力；不得先实现一套看似可用但实际被原生 surface 遮挡的 QML 浮层。

## 必读

- [VTK WebGPU 硬件窗口与渲染数据](../standards/vtk.md)
- [可视化、RenderScene 与原生视口](../architecture/visualization.md)
- [组件与主题模块](../modules/qml-components-and-theme.md)
- [分层与依赖方向](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 在 VTK 原生视口右下角显示笛卡尔坐标系，明确 X / Y / Z 轴方向、颜色和可见性边界；坐标系随相机旋转同步更新，并适配 resize、DPR、隐藏 / 恢复。
- 在 VTK 原生视口右上角显示六面体定位器；至少支持六个标准面对应的正交视图，面标签和当前方向状态可读。
- 点击六面体面后将相机平滑或确定性地切换到对应方向；连续点击、快速重复点击和当前已在目标方向时不产生异常跳转。
- 在视口内实现鼠标拖拽的整体观察旋转，围绕当前焦点保持合理旋转中心、方向和灵敏度；默认欢迎字样与导入 STL 都可旋转。
- 统一处理原生 surface 的鼠标坐标、视口尺寸、设备像素比、窗口重建和资源销毁顺序；QML 只接收必要的语义状态，不直接操作 VTK。
- 为相机 / 导航状态定义后端中立边界；VTK、平台输入和 overlay widget 类型仅出现在 `native/visualization/src/vtk/` 或平台桥接实现中。

不包含：

- 不修改 STL、Mesh IR、顶点坐标、法线或工程 import record；不把视角状态持久化到 `.panta`。
- 不实现模型选择、测量、拾取高亮、裁剪、缩放 / 平移工具、触摸手势和动画时间轴；这些另行登记。
- 不引入 `QQuickVTKItem`、Qt OpenGL scenegraph 集成或第二套 Qt overlay；不把 VTK 类型加入公共 `CaeViewport` 头文件。
- 不在 offscreen 测试中伪造 GPU overlay 的真实像素验收；无头测试只验证状态和生命周期，真实窗口负责位置、输入和视觉效果。

## 前置条件与待决策

- 先以当前锁定 VTK SDK 的实际头文件和模块为准，核对 `vtkAxesActor`、`vtkAnnotatedCubeActor`、`vtkOrientationMarkerWidget` 或同等能力是否适用于 WebGPU render window；不能仅凭 nightly 文档假定 API 可用。
- 由于 native surface 覆盖 QML，确认 overlay 的输入归属、渲染层级和平台行为；若 SDK widget 不适用，应在同一 VTK renderer / render window 内实现最小 overlay，而不是退回 QML 浮层。
- 明确坐标系显示的是世界坐标系还是模型局部坐标系；首期默认世界坐标系，若工程未来引入局部坐标系需扩展状态契约。
- 明确六面体面、边、角的交互范围和相机过渡策略；首期至少交付六面正交面，边角交互只有在 SDK / 自绘实现和验收标准稳定后才加入。
- 记录 macOS、Windows、Wayland 原生 surface 的鼠标事件与高 DPI 差异；缺少真实平台证据时保留未验收状态。

## 实施步骤

1. 核对锁定 VTK SDK 的 orientation marker、annotation cube、相机和交互 API，记录可用能力与平台限制。
2. 扩展后端中立的导航 / 相机状态和变更入口，区分临时视角状态与工程领域数据。
3. 在 VTK 适配层实现坐标系、六面体和相机轨道旋转，先覆盖默认场景，再接入 STL 场景。
4. 接入原生 surface 的输入与坐标转换，验证 overlay 命中测试不会吞掉或误触其他窗口操作。
5. 增加 offscreen 状态 / 生命周期测试，以及 macOS 真窗口下的位置、点击、拖拽、resize、DPR、隐藏 / 恢复和销毁验证。
6. 同步可视化架构、模块说明、代码注释和本任务验证记录；若 SDK 能力不足，登记后续替代方案而不保留死实现。

## 预计改动

- `native/visualization/include/panta/visualization/`：仅增加后端中立的相机 / 导航状态或语义接口，具体文件以实现时的最小边界为准。
- `native/visualization/src/vtk/`：增加坐标系、六面体、相机交互和输入适配；隐藏所有 VTK 第三方类型。
- `native/visualization/src/<platform>/`：必要时调整原生 surface 的输入与坐标同步，不改变现有平台所有权和销毁顺序。
- `tests/cpp/visualization/` 与现有 QML / native CTest：增加可重复的状态、命中、相机和生命周期验证。
- `ai-docs/architecture/visualization.md`、相关模块说明和本任务：实现后记录实际 API、限制和验收证据。

## 清理与兼容例外

- 清理任何被新导航控制器替换的临时相机 / 输入逻辑、重复 overlay 路径和失效测试夹具；不保留双实现。
- 预计无兼容例外；不改变 `.panta` manifest schema，不新增 `COMPAT(...)`。

## 验收标准

- [ ] 默认欢迎场景和已导入 STL 在真实 VTK 窗口中均显示右下角笛卡尔坐标系，位置、大小、方向和颜色稳定。
- [ ] 真实 VTK 窗口右上角显示六面体定位器；六个面可识别，点击每个面都切换到对应的标准正交方向。
- [ ] 视口拖拽可连续旋转整体观察视图，旋转中心稳定；不修改几何数据、不污染工程 dirty 状态、不破坏 STL 重开。
- [ ] resize、DPR 变化、最小化 / 恢复、窗口重建和工程关闭不会留下旧 overlay、输入回调或 VTK 资源。
- [ ] QML 公共头和 Rust / `.panta` 数据边界不暴露 VTK 类型；offscreen 状态测试、格式检查、qmllint 和 native 测试通过。
- [ ] 至少完成 macOS 当前平台真实窗口验收；Windows / Wayland 缺少环境时明确记录未验收项，不以离屏测试替代。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 当前锁定 VTK SDK 头文件 / CMake targets | 确认 orientation marker、六面体和 WebGPU 兼容 API | 未执行 |
| — | `cargo lint qmllint --check`、C++ / QML 格式检查 | 静态规则通过 | 未执行 |
| — | native CTest：状态、相机、输入和生命周期 | 自动化边界通过 | 未执行 |
| — | macOS 真窗口：坐标系、六面体、拖拽、resize、DPR、隐藏 / 恢复、关闭 | 交互和资源生命周期通过 | 未执行 |

## 风险与回退

- VTK SDK 的 orientation widget 可能依赖未启用的模块或旧渲染后端；先核对实际制品，无法复用时在同一 WebGPU render window 内实现受限版本并记录差异。
- 原生 surface 可能使 QML overlay 不可见或无法接收鼠标；回退到 VTK 内部 overlay，输入仍由原生视口统一路由。
- 相机旋转在极端角度可能发生翻转、穿模或焦点漂移；使用明确的 up 约束、焦点和角度边界，并保留重置到标准方向的路径。
- 任一平台销毁顺序出现旧回调访问时，优先停用新导航输入并恢复上一版纯视口路径，不改变工程数据和导入资产。

## 决策与工作记录

- 2026-09-22：创建任务。用户要求推进 VTK 视口右下角笛卡尔坐标系、右上角六面体定位和整体几何视图旋转。
- 2026-09-22：首期将“整体几何体旋转”定义为相机轨道旋转，不修改几何数据或工程持久化；坐标系 / 六面体优先采用 VTK 原生 surface 内 overlay。

## 完成摘要

未完成。实现后补充锁定 SDK API、真实窗口证据、自动化测试、平台限制和剩余后续任务。
