# 051 — 无边框外观与一体化窗口标题栏

- 状态：ready
- 阶段：应用平台扩展
- 依赖：[005 Qt/QML 主窗口](005-qt-qml-shell.md)、[029 QML 组件库](029-qml-component-library.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

主窗口需要无独立系统标题条的外观：现有快捷按钮、标题、搜索和账户区域进入窗口顶部，与窗口控制共用一行，并保留系统窗口管理行为。当前 `App.qml` 使用普通 `ApplicationWindow`，`TopChromePanel.qml` 仍位于系统标题栏下方；029 未实现标题栏融合。

可行性结论：目标可实施，但“Qt 6.5 引入 `QWindow::setTitleBar(QWidget*)`”不成立，不能采用该示例。项目使用 Qt Quick，亦不应为此引入 QWidget 标题栏。优先在当前 Qt 6.11.2 上验证扩展客户区方案；各平台的实际窗口行为尚未验证，不能宣称仅设置 flags 即可完整保留所有系统功能。

## 必读与官方依据

- [Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)、[组件与主题](../modules/qml-components-and-theme.md)。
- [显示缩放与多屏](../modules/display-scaling-and-multi-monitor.md)、[VTK 原生视口任务](007-vtk-quick-viewport.md)。
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)、[测试规范](../standards/testing.md)、[代码生命周期](../standards/code-lifecycle.md)。
- 提交时阅读[提交规范](../standards/commits.md)。

官方资料查阅日期：2026-09-22；目标版本为仓库已供给的 Qt 6.11.2，非依赖升级。

1. [Qt 6.5 QWindow](https://doc.qt.io/qt-6.5/qwindow.html) 与 [QWindow 文档（查阅时 6.11.2）](https://doc.qt.io/qt-6/qwindow.html) 均无 `setTitleBar(QWidget*)`。本机 Qt 6.11.2 `qwindow.h` 同样无该成员。
2. [Qt 6.9 新增功能](https://doc.qt.io/qt-6/whatsnew69.html)及 [WindowType flags](https://doc.qt.io/qt-6/qt.html#WindowType-enum)：6.9 新增 `ExpandedClientAreaHint`、`NoTitleBarBackgroundHint` 与 `QWindow::safeAreaMargins()`，可请求客户区向标题栏延伸并融合背景。本机头文件已核实存在；flags 是平台请求，效果需实测。
3. [ApplicationWindow 安全区域](https://doc.qt.io/qt-6/qml-qtquick-controls-applicationwindow.html#safe-areas)：6.9 起内容区自动应用安全边距，header/footer/menuBar 不自动应用同样的 padding；布局必须明确处理，避免顶部空带或遮挡控件。
4. [QWindow::startSystemMove](https://doc.qt.io/qt-6/qwindow.html#startSystemMove) / [startSystemResize](https://doc.qt.io/qt-6/qwindow.html#startSystemResize)：优先请求系统交互式移动/缩放，检查返回值与平台支持；不以逐帧修改 x/y 替代系统拖动。
5. [Microsoft：Windows 11 Snap Layouts](https://learn.microsoft.com/zh-cn/windows/apps/desktop/modernize/ui/apply-snap-layout-menu)：自定义标题栏可能影响贴靠布局；若自绘最大化按钮，需评估 `WM_NCHITTEST` / `HTMAXBUTTON`。拖到屏幕边缘吸附与悬停最大化按钮弹出布局必须分别验证。

## 范围与非目标

- 主窗口标题栏融合、窗口控制、空白区拖动与双击、边缘缩放、系统控制区域避让。
- 优先保留原生窗口控制（Windows 按钮、macOS 红黄绿按钮），由 QML 绘制业务控件；无边框外观不等于必须设置 `FramelessWindowHint`。不在扩展客户区方案上无条件叠加该 flag。
- 平台能力不足时，在 C++ 窗口适配层评估真正无框窗口及原生事件处理；这是待验证候选，不能将所有平台都切换到未经验证的自绘控制。
- 覆盖主应用与无 Bridge 启动变体的可操作性；无 Bridge 变体不依赖 Bridge 模块。
- 非目标：业务按钮对应的新建/保存/登录功能、主题 DSL、设置持久化、外部引擎、Qt 升级和第三方无框库引入。现有按钮保持其业务接入状态，但点击、焦点和文本编辑不得被拖动逻辑吞掉。

## 前置条件与待决策

005、029 已完成，可开始最小原型，故状态为 ready。007 的视口实现已存在，本任务需要集成回归；033 的整体多屏基础不作为启动依赖，但窗口 DPI 与跨屏命中测试是本任务验收项。

先在 macOS、Windows、Linux 目标环境记录 Qt 平台插件、系统/窗口管理器和实际能力，再确定各平台采用的路径。Linux 主集成按现有 VTK SDK 的 Wayland-only 边界验证，不由 Shell 在 X11 能启动推断 VTK X11 已受支持。缺少目标平台实测时保留未验收状态。

## 实施步骤

1. 基于现有 Shell 做扩展客户区最小原型，记录各平台原生按钮、标题文本、背景、安全区域和缩放边缘的效果；确定是否确有平台适配缺口。
2. 调整 `TopChromePanel` 的标题行与 `ApplicationWindow` 安全区域布局，消除重复标题条；保持菜单/ribbon 布局。系统按钮占用区域来自运行期窗口信息，不能把平台固定像素宽度当作跨平台契约。
3. 空白区进入系统拖动，双击按平台约定处理最大化/还原等操作；按钮、输入框、弹出菜单及文本选择区域排除拖动命中。若系统未接管双击，适配层明确实现并验证。
4. 必要的平台代码封装在 C++ 窗口控制适配层，QML 通过 ViewModel/控制器发出语义请求。处理窗口对象生命周期、激活状态、最小化/最大化/全屏及恢复，保留正常关闭流程。
5. 按平台验证原生视口对齐、遮挡与输入、跨屏 DPI 变化和窗口恢复；同步组件文档、相关代码注释与任务状态。

## 预计改动

- 现存：`qml/App.qml`、`qml/AppNoBridge.qml`、`qml/Panels/TopChromePanel.qml`、`qml/Themes/Theme.qml`、`qml/CMakeLists.txt`。
- 现存：`native/app/main.cpp`、`native/app/CMakeLists.txt`；必要时新增 app 层窗口控制适配文件（具体命名在原型后确定，尚未创建）。
- 现存：`tests/cpp/app/shell_module_load_test.cpp`、`tests/qml/`；补充窗口行为测试和必要的构建注册。
- 现存：[组件与主题文档](../modules/qml-components-and-theme.md)、本任务与[任务索引](../task-index.md)。

## 清理与兼容例外

实施时清理重复标题区和“窗口控制仅由独立系统标题栏承接”的失效代码注释；历史任务保留当时实现事实并链接本任务。不并存两套已废弃标题栏实现，不加入 Qt 6.5 兼容分支。当前无兼容例外。

## 验收标准

- [ ] 主窗口只有一条融合后的标题工具栏，无额外系统标题条；按钮/搜索与原生控制不重叠，窄窗口仍有可用拖动区。
- [ ] 窗口控制、边缘/四角缩放、空白区拖动、双击行为、最大化/还原、最小化恢复与关闭流程通过真实窗口验证。
- [ ] 点击业务按钮、编辑/选择搜索文本、使用菜单和键盘焦点不会触发窗口移动；窗口控制具有可访问名称与正确交互状态。
- [ ] Windows 验证拖动吸附、Win+方向键、最大化按钮悬停 Snap Layouts 与任务栏恢复；受系统配置限制的场景记录配置及结果。
- [ ] macOS 验证红黄绿按钮、全屏进出、系统双击偏好及窗口恢复；Linux Wayland 在明确的 compositor 下验证移动/缩放与窗口状态。
- [ ] 常规/最大化/全屏切换和 100%/150%/200% 缩放、不同 DPR 跨屏过程中，无顶部空带、点击偏移或控件遮挡；VTK 视口对齐、输入和显示生命周期无回归。
- [ ] 无 Bridge 变体仍可启动、移动、缩放与关闭；QML lint、资源加载及相关自动测试通过。
- [ ] 官方 API/平台限制、真实验证证据与清理情况已回填，必需平台未验证时不标 done。

## 验证计划与结果

实施验证均在仓库根目录，通过现有 `cargo build --locked`、`cargo lint qmllint`、`cargo format`、`cargo test --locked --workspace` 检查构建、静态规则及行为回归；新增测试接入现有入口。真实窗口验证单独记录操作系统、Qt/平台插件、DPR、窗口状态、VTK 后端及结果，离屏截图不能替代系统窗口行为证据。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-22 | 仓库根；读取 Qt 供给清单、App/TopChromePanel 与本机 Qt 头文件 | 核实版本、现状及 API | 供给固定 6.11.2；QML 尚未配置标题栏融合；`qwindow.h` 无 `setTitleBar`，存在 `safeAreaMargins`、系统移动/缩放；`qnamespace.h` 存在两个扩展 flags |
| 2026-09-22 | 官方 Qt / Microsoft 文档核验 | 区分真实 API 与平台待验项 | 可按扩展客户区方向立项；原示例不可用，未执行窗口原型或跨平台运行测试 |
| 2026-09-22 | 仓库根；Python 3 本地链接/代码围栏/编号与依赖检查、`git diff --check` | 新任务登记一致且无坏链/空白错误 | 通过；四份变更文档链接有效，051 编号唯一，005/029 已完成，索引与任务均为 ready |
| — | 实施后的构建、lint、行为测试及三平台真实窗口 | 满足上述验收条件 | 未执行，本次仅评估与立项 |

## 风险与回退

平台可能忽略部分 flags，安全边距未必足以表达期望的横向按钮布局；应以原型决定适配方式。完全去掉原生装饰可能丢失阴影、缩放或系统贴靠，VTK 原生子视图也可能出现偏移。候选路径未满足验收前不替换默认窗口行为；失败时撤回本任务窗口改动，保留现有可操作 Shell，不修改用户工程数据。

## 决策与工作记录

- 2026-09-22：按维护者新要求登记无边框外观与标题栏按钮融合，承接 029 的后续窗口工作；纠正 `QWindow::setTitleBar` 前提，优先评估当前 Qt 已有扩展客户区 API，无需等待不存在的接口或升级 Qt。

## 完成摘要

已完成可行性评估与立项。窗口实现及运行验证尚未开始，任务保持 ready。
