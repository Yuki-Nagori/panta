# 064 — VTK 视口导航、笛卡尔坐标系与六面体定位

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[063](063-stl-import-and-mesh-workspace.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-23

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
- 在 VTK 原生视口右上角显示六面体定位器；至少支持六个标准面对应的正交视图，面标签和当前方向状态可读；六个方向显示轴向和英文方向。
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
| 2026-09-22 | 当前锁定 VTK 9.7.0 WebGPU SDK 头文件与 CMake targets | 确认可用 overlay 几何、文本和 interactor 能力 | 通过；缺少现成 orientation widget / trackball style，采用 `vtkRenderer` 第二层 + `vtkLineSource` / `vtkCubeSource`，坐标轴使用 billboard 文本，六面体使用 `vtkVectorText` + `vtkPolyDataMapper` + `vtkActor`，补充 `VTK::FiltersSources` / `VTK::FiltersGeneral` |
| 2026-09-22 | `cmake --build target/native/debug` | 使用现有 target 完成 native 构建 | 通过；仅有既存重复静态库链接 warning |
| 2026-09-22 | C++ / CMake 格式检查、`git diff --check` | 静态格式规则通过 | 直接 clang-format、cmake-format 与 diff 检查通过；统一 `cargo format` 的 uv/system-configuration 检查因本机 NULL system object 失败，未影响直接格式结果 |
| 2026-09-23 | `ctest --test-dir target/native/debug --output-on-failure -R 'I18n\\.CompiledQmLoads|Visualization\\.DefaultWordmark|Qml\\.FormatCheck|Qml\\.ShellModuleLoads|Qml\\.ViewportModuleLoads'` | i18n、视口、QML 模块和方向命中边界通过 | 通过 5/5；`Visualization.DefaultWordmark` 覆盖六面投影命中、面中心、正向矩阵、生命周期消融路径 |
| 2026-09-23 | `target/native/debug/visualization/panta_default_wordmark_test` | 获取 overlay 更新与命中路径的 CPU 基线 | 通过；无 overlay 更新约 0.0000058 ms/次，完整 overlay 更新约 0.000094 ms/次，射线命中约 0.00088 ms/次；仅衡量 CPU，不包含 WebGPU 提交 |
| 2026-09-23 | `cargo test -p panta-dsl-core --locked`、`cargo clippy --locked -p panta-dsl-core --all-targets -- -D warnings`、`cargo build -p panta-launcher --locked`、`cargo lint qmllint` | Rust/i18n 解析、launcher 和 QML lint 回归通过 | 全部通过；`cargo lint cmake` 另受本机 uv/system-configuration 的 NULL system object 崩溃影响，直接 `clang-format`、`git diff --check` 和已有 CMake 配置构建通过 |
| — | macOS 真窗口：坐标系、六面体、拖拽、resize、DPR、隐藏 / 恢复、关闭 | 交互和资源生命周期通过 | 未执行，待手工验收 |

## 风险与回退

- VTK SDK 的 orientation widget 可能依赖未启用的模块或旧渲染后端；先核对实际制品，无法复用时在同一 WebGPU render window 内实现受限版本并记录差异。
- 原生 surface 可能使 QML overlay 不可见或无法接收鼠标；回退到 VTK 内部 overlay，输入仍由原生视口统一路由。
- 相机旋转在极端角度可能发生翻转、穿模或焦点漂移；使用明确的 up 约束、焦点和角度边界，并保留重置到标准方向的路径。
- 任一平台销毁顺序出现旧回调访问时，优先停用新导航输入并恢复上一版纯视口路径，不改变工程数据和导入资产。

## 决策与工作记录

- 2026-09-22：创建任务。用户要求推进 VTK 视口右下角笛卡尔坐标系、右上角六面体定位和整体几何视图旋转。
- 2026-09-22：首期将“整体几何体旋转”定义为相机轨道旋转，不修改几何数据或工程持久化；坐标系 / 六面体优先采用 VTK 原生 surface 内 overlay。
- 2026-09-22：核对锁定 VTK 9.7.0 WebGPU SDK，未提供 `vtkOrientationMarkerWidget`、`vtkAnnotatedCubeActor` 或 trackball style；改用第二层 renderer、自有轴线 / 六面体 / `vtkBillboardTextActor3D` 文本 actor，并显式加入 `VTK::FiltersSources`。
- 2026-09-22：完成第一版 VTK 后端实现：方向标记随主相机方向更新，六面体提供六个确定性方向命中区，左键拖拽通过 interactor 驱动相机轨道旋转；新增方向命中单测。真实窗口下的 overlay 位置、标签可读性和 Cocoa 输入仍待验收。
- 2026-09-22：按验收反馈细化方向标记：保留右下角独立的 X / Y / Z 轴标注，六面体方向改用短标签 `RIGHT / LEFT / TOP / BOTTOM / FRONT / BACK`；方向文字由 VTK 适配层内置，不经 QML 或翻译字典传入。
- 2026-09-23：按真窗口反馈调整标签几何：六面体标签锚点落在对应面的中心并留最小面外偏移，文本水平 / 垂直居中；坐标轴标签以轴端点为锚点，避免标签漂移或偏离方向标记。
- 2026-09-23：按验收反馈放大六面体文字并保持粗体；使用 billboard 文本使字形始终朝向屏幕，六面体位置仍随方向状态更新，但文字本身不随六面体姿态旋转。
- 2026-09-23：根据真窗口截图修正六面体标签堆叠：标签仍固定在面中心，但只显示朝向当前 overlay 相机的可见面，隐藏背向面；字号调整为适合定位器尺寸的中等值，避免多个背面文字投影到同一区域。
- 2026-09-23：按验收反馈放大右上角六面体定位器，边长调整为 1.6，并同步面中心锚点和 overlay 相机比例；交互命中区域保持原视口语义范围不变。
- 2026-09-23：按验收反馈将六面体与右下角坐标系整体向右移动，统一调整 overlay viewport 与命中区域；标签字号继续放大，并采用勃艮第红 X、勃艮第绿 Y、勃艮第蓝 Z 的对应配色，同时增大轴线端点与 X/Y/Z 标签的间距。
- 2026-09-23：按验收反馈为右上角六面体和右下角坐标系增加统一的右侧内边距，并同步收窄 overlay 命中区域。
- 2026-09-23：按验收截图将六面体与坐标系继续向右调整到目标区域，同时保留右边界内缩；六面体方向标签改为面内三维矢量文本方案，随定位器相机姿态同步旋转。
- 2026-09-23：根据定位截图修正 `vtkVectorText` 的局部几何原点：先在 `vtkTransformFilter` 中按文字 bounds 缩放并居中，再将 actor 原点放到六面体面中心；定位器改为不透明实心面，避免文字与半透明面产生深度混淆。
- 2026-09-23：按标准方向切换反馈，为六个面显式定义“屏幕右 / 屏幕上 / 面法向”正交基，使用完整面内旋转矩阵替代近似欧拉角，保证切到 TOP 等方向时文字保持正向可读。
- 2026-09-23：修正六面体点击命中：不再按 overlay 矩形的屏幕象限猜方向，改为使用定位器 renderer 的显示坐标反投影射线与实体六面体求交，点击实际可见面后返回对应 `CubeDirection`。
- 2026-09-23：对暂存区 VTK 代码进行整体 review：收敛方向面姿态和颜色常量，补充 overlay 生命周期 / 无 renderer 消融路径与 CPU 命中、方向更新基准测试；同时校正 VTK interactor 和 hardware window 的释放顺序。
- 2026-09-23：按 VTK 9.7 的实际契约收敛方向文本：`vtkVectorText` 只接受可打印 ASCII，移除中文字体、翻译条目和第二套文本渲染路径；方向文案固定由 VTK 适配层内置，避免外部本地化输入产生空标签。
- 2026-09-23：提交评审通过的导航实现、测试与记录；`detach()` 显式移除窗口中的 overlay renderer。保留本轮翻译梳理发现的 `.pa` 校验修复：同一 context 允许不同的等长 source，仅拒绝重复 source，相关测试和规范同步更新。用户已确认当前视觉效果；跨平台输入与完整窗口生命周期仍按未验收项跟进。

## 完成摘要

第一版导航后端已落地：坐标轴和六面体在 VTK 原生 render window 的第二层 renderer 绘制，主相机支持左键轨道旋转，六面体六个方向可按实际面命中并确定性切换；方向文案由 VTK 适配层内置，六面体标签使用贴面矢量几何并保持标准视角正向。当前通过现有 target 构建、投影命中 / 几何姿态 / 生命周期消融 / CPU 基准测试及 5 个 QML/native 回归测试；仍需 macOS 真窗口完成 resize、DPR、隐藏 / 恢复和关闭的人工验收，以及后续 Windows / Wayland 输入路径校准。
