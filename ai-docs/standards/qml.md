# QML 组件与声明式界面

查阅日期：2026-09-17。状态：Qt 6.11.2 预编译链路已由任务 005 落地验证；任务 026 已将 Bridge 迁移为带 typeinfo 的静态 QML 模块，qmllint 可识别其类型。

适用于 `qml/`；Qt 对象模型参见 [Qt](qt.md)，视口参见 [VTK](vtk.md)，高 DPI 与多显示屏参见[显示缩放模块](../modules/display-scaling-and-multi-monitor.md)。

## 官方依据

Qt 建议分离界面与业务逻辑，并使用资源系统组织应用资源；QML 文件与模块目录的关系影响 implicit imports。[QML Best Practices](https://doc.qt.io/qt-6/qtquick-bestpractices.html)

`qt_add_qml_module` 管理 QML 文件、资源和相关构建工具集成。[qt_add_qml_module](https://doc.qt.io/qt-6/qt-add-qml-module.html)

## 项目规则

- 组件文件 `PascalCase.qml`，id/属性/信号使用 `camelCase`；命名反映界面语义，不把库名当作用户功能名。
- 每个可复用组件声明输入属性与输出信号，通过显式属性注入 ViewModel；避免依赖任意上层 id 或散布全局 context properties。
- 状态优先用属性绑定表达；命令式赋值前检查是否会破坏原绑定。组件不直接读写工程文件，也不调用 OCCT、Netgen、VTK 算法。
- 长列表使用有明确角色的模型和 delegate；不将大型网格数组转换成 JavaScript 对象供 UI 管理。
- 将 `qt_add_qml_module` 放在与模块 QML 布局一致的位置，由 native 顶层纳入构建；若必须跨目录，任务 005 要验证 import 和资源别名，不能仅靠开发机 import path 可用。
- 图标和内置 QML 通过模块资源定位；用户输入文件通过应用服务处理 URL/path 转换。避免当前工作目录相关的相对资源路径。
- 主题、间距和色彩集中管理；文字通过翻译接口包裹；焦点、键盘操作与禁用原因可理解。布局统一使用 Qt 逻辑像素，不在 QML 里乘 `devicePixelRatio` 或缓存屏幕比例；跨屏和 framebuffer 转换集中由 C++/渲染边界处理。第一版只搭壳，不提前堆砌业务面板。
- 自绘控件样式（自定义 background 等）不能运行于原生 Controls 样式；应用入口以 `QQuickStyle::setStyle("Basic")` 固定非原生样式，测试环境保持同源（029）。
- 图标遵循 [图标设计规范](icons.md)，使用模块内 SVG 资源并经 `qt_add_resources` 登记；Mono 图标由 `panta-icons` provider 使用 QtSvg 渲染并着色，普通 SVG Image 解码使用 qsvg 插件。不在 QML 里用 Shapes/Canvas 逐个绘制静态图标。

## 原子组件与主题参数（项目约定）

- UI 按原子组件 → 组合组件 → 业务面板 → 页面拼装；组件库只沉淀实际复用或具有独立职责的控件，原子层不接业务服务。
- 每个组件以明确属性接收内容、状态和尺寸，以信号输出意图；组合组件向子组件传参，不读取任意父级对象。
- 可配置视觉尺寸（间距、内边距、圆角、边框、字号、图标、控件高度及面板/窗口最小尺寸等）统一由 `qml/Themes/Theme.qml` 暴露，组件输入属性默认绑定 token。调用方覆盖也使用 token 或布局计算，不散布视觉数值。
- 内容测量、父布局分配和纯算法常量无需伪装成主题尺寸；UI 尺寸使用逻辑像素语义，不能把工程几何单位混入 Theme。
- Theme 是 QML 的唯一主题入口；后续默认值与主题覆盖由 DSL 提供，经服务校验及 C++ ViewModel 发布。QML 不直接解析 DSL，不维护另一套默认主题值。
- 主题切换靠属性绑定更新，与语言切换、QML 引擎重载分别处理。应用主题不属于工程变量，不影响工程 dirty 状态。

详见[原子组件库与主题 DSL](../modules/qml-components-and-theme.md)。组件层已落地；运行期主题 DSL 与切换仍由 030 承接。

## 验证

组件库需验证默认与覆盖参数、主题切换后的绑定更新、长文本及 1.0/1.25/1.5/2.0 DPR 下布局，禁止以隐藏溢出代替正确布局。

使用所锁定 Qt 提供的 qmllint/格式工具检查实际模块；验证必需属性、导入、绑定循环、窗口缩放和键盘焦点。打包后的资源加载必须独立验证，不能只用源码目录启动成功作为证据。

### 性能基准

每次新增 QML 文件或可复用组件，都要把它实际放进一个可复现的手动性能基准场景；修改会改变对象创建、delegate 数量、属性绑定、布局、JavaScript、渲染或高频交互更新的 QML 时，也要更新受影响的场景。基准入口使用任务 [069](../task/069-project-docks-review-and-ablation.md) 建立的可扩展 CPU / GPU harness；在对应实现 task 中记录组件到场景的映射、运行命令和结果，不能只登记文件名或用静态截图代替测量。

| 要测的成本 | 基准入口与边界 |
|---|---|
| 组件构造、模型/delegate 创建与回收、绑定/布局/JavaScript 更新 | `tests/qml/project_docks_cpu_benchmark.cpp`。允许使用 offscreen 场景测 CPU 构造和更新成本，不据此声称 GPU 渲染性能。 |
| 可见内容的帧呈现、动画或场景更新 | `tests/qml/project_docks_gpu_benchmark.cpp`。必须在真实图形窗口测量，并把结果表述为 Qt Quick 端到端帧间隔；不得称作 GPU 内核耗时。 |
| 同时影响 CPU 更新和可见渲染 | 两种入口都覆盖。 |

每个新场景选择能代表实际使用的低、典型和较大负载；列表、重复委托等按实际数量测量，并比较适用的组件消融。记录 Qt 版本、平台/图形后端、构建配置、输入规模、预热与采样次数、p50/p95 及测量限制；性能改动还要在同环境和同输入下比较前后结果。性能基准是开发侧诊断工具，不注册为 CTest，也不增加 CI 性能门槛，遵循[性能测试模块](../modules/performance.md)。

VS Code 的 Qt Qml 扩展通过 [工作区配置](../../.vscode/settings.json) 使用 Cargo 供给的 qmlls，并传入 `target/native/debug` 构建目录与 Qt staging 的 QML 导入路径。先执行 `cargo build --locked` 生成模块类型信息；编辑器不自行触发 CMake。若出现 `color was not found [import]` 等导入诊断，先对照 `cargo lint qmllint --check`，再检查 QML Language Server 日志中的工具路径和参数，必要时重启语言服务；不关闭 import 诊断。
