# 051 — 无边框外观与一体化窗口标题栏

- 状态：deferred
- 阶段：应用平台扩展
- 依赖：[005 Qt/QML 主窗口](005-qt-qml-shell.md)、[029 QML 组件库](029-qml-component-library.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-22 / 2026-10-17

## 目标与背景

主窗口需要把应用顶栏与系统窗口控制整合，同时保留各平台原有操作习惯。实施前，`App.qml` 使用普通 `ApplicationWindow`，`TopChromePanel.qml` 位于系统标题栏下方；029 尚未实现标题栏融合。

平台布局按维护者确认分开处理：

- macOS：最上方一行使用系统红、黄、绿窗口控制按钮；不显示左侧 P 品牌标记和系统按钮旁重复的 `panta` 窗口标题，保留顶栏中间的 `panta 2027` 标题，快捷操作与搜索区从系统按钮右侧开始；顶栏使用浅灰背景，第二行导航栏顶格排列。
- Windows 与 Linux：保留左侧应用 P 标记，隐藏原生应用图标和标题，并保留系统最小化、最大化及关闭按钮。Windows 按 Qt 当前标题栏几何从 Windows DPI 系统指标运行时计算右侧避让；Linux 保留窗口管理器装饰和原生按钮，使用零宽标题并不请求系统菜单图标，仍需 Linux/Wayland 实机验收。
- 点击带红底的 P 打开应用菜单、菜单中的 Settings 入口，以及 macOS 以系统菜单栏承载这些菜单职责，属于独立后续任务 115；本任务只预留不冲突的窗口标题区域，不实现菜单或设置页面。

可行性结论：目标可实施，但“Qt 6.5 引入 `QWindow::setTitleBar(QWidget*)`”不成立，不能采用该示例。项目使用 Qt Quick，亦不应为此引入 QWidget 标题栏。当前实现基于 Qt 6.11.2 扩展客户区方案；macOS 已通过真实窗口验收，Windows/Linux 及 Wayland 的窗口行为仍待实机验证，不能据此宣称各平台仅设置 flags 即可完整保留所有系统功能。

## 必读与官方依据

- [Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)、[组件与主题](../modules/qml-components-and-theme.md)。
- [显示缩放与多屏](../modules/display-scaling-and-multi-monitor.md)、[VTK 原生视口任务](007-vtk-quick-viewport.md)。
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)、[测试规范](../standards/testing.md)、[代码生命周期](../standards/code-lifecycle.md)。
- 提交时阅读[提交规范](../standards/commits.md)。

官方资料查阅日期：2026-09-22；目标版本为仓库已供给的 Qt 6.11.2，非依赖升级。

1. [Qt 6.5 QWindow](https://doc.qt.io/qt-6.5/qwindow.html) 与 [QWindow 文档（查阅时 6.11.2）](https://doc.qt.io/qt-6/qwindow.html) 均无 `setTitleBar(QWidget*)`。本机 Qt 6.11.2 `qwindow.h` 同样无该成员。
2. [Qt 6.9 新增功能](https://doc.qt.io/qt-6/whatsnew69.html)及 [WindowType flags](https://doc.qt.io/qt-6/qt.html#WindowType-enum)：6.9 新增 `ExpandedClientAreaHint`、`NoTitleBarBackgroundHint` 与 `QWindow::safeAreaMargins()`，可请求客户区向标题栏延伸并融合背景。本机头文件已核实存在；flags 是平台请求，效果需实测。
3. [ApplicationWindow 安全区域](https://doc.qt.io/qt-6/qml-qtquick-controls-applicationwindow.html#safe-areas) 与 [SafeArea QML 类型](https://doc.qt.io/qt-6/qml-qtquick-safearea.html)：6.9 起内容区自动应用安全边距，header/footer/menuBar 不自动应用同样的 padding；布局必须明确处理，避免顶部空带或遮挡控件。
4. [QWindow::startSystemMove](https://doc.qt.io/qt-6/qwindow.html#startSystemMove) / [startSystemResize](https://doc.qt.io/qt-6/qwindow.html#startSystemResize)：优先请求系统交互式移动/缩放，检查返回值与平台支持；不以逐帧修改 x/y 替代系统拖动。
5. [Microsoft：Windows 11 Snap Layouts](https://learn.microsoft.com/zh-cn/windows/apps/desktop/modernize/ui/apply-snap-layout-menu)：自定义标题栏可能影响贴靠布局；若自绘最大化按钮，需评估 `WM_NCHITTEST` / `HTMAXBUTTON`。拖到屏幕边缘吸附与悬停最大化按钮弹出布局必须分别验证。
6. [Apple：NSWindow.standardWindowButton](https://developer.apple.com/documentation/appkit/nswindow/standardwindowbutton%28_%3A%29) 与 [Apple：macOS Toolbars HIG](https://developer.apple.com/design/human-interface-guidelines/toolbars)：AppKit 按窗口样式提供标准标题栏按钮并返回实际控件；官方文档未规定交通灯按钮组的固定像素尺寸或左右间距。因此避让使用原生按钮矩形，外侧留白按左右对称计算，不维护固定按钮尺寸常量。

## 范围与非目标

- 主窗口标题栏融合、窗口控制、空白区拖动与双击、边缘缩放、系统控制区域避让。
- macOS 顶部使用系统红黄绿窗口按钮，不显示左侧品牌 P 与系统按钮旁重复的窗口标题，保留顶栏中间标题；快捷操作和搜索区避开系统按钮，顶栏为浅灰纯色，导航行贴齐窗口顶部。Windows 与 Linux 保留现有顶栏布局和左侧品牌 P，并按安全区域避让原生标题栏控件。
- 顶栏工具组使用 `Theme.colorChrome` 浅灰纯色背景，不显示渐变。
- 优先保留各平台原生窗口控制，由 QML 绘制业务控件；无边框外观不等于必须设置 `FramelessWindowHint`。不在扩展客户区方案上无条件叠加该 flag。
- 平台能力不足时，在 C++ 窗口适配层评估真正无框窗口及原生事件处理；这是待验证候选，不能将所有平台都切换到未经验证的自绘控制。
- 覆盖主应用与无 Bridge 启动变体的可操作性；无 Bridge 变体不依赖 Bridge 模块。
- 非目标：应用菜单及其 Settings 入口（后续任务 115）、业务按钮对应的新建/保存/登录功能、主题 DSL、设置持久化、外部引擎、Qt 升级和第三方无框库引入。现有按钮保持其业务接入状态，但点击、焦点和文本编辑不得被拖动逻辑吞掉。

## 前置条件与待决策

005、029 已完成，当前进入实现与平台验收阶段。007 的视口实现已存在，本任务需要集成回归；033 的整体多屏基础不作为启动依赖，但窗口 DPI 与跨屏命中测试是本任务验收项。

先在 macOS、Windows、Linux 目标环境记录 Qt 平台插件、系统/窗口管理器和实际能力，再确定各平台采用的路径。Linux 主集成按现有 VTK SDK 的 Wayland-only 边界验证，不由 Shell 在 X11 能启动推断 VTK X11 已受支持。缺少目标平台实测时保留未验收状态。

## 实施步骤

1. 基于现有 Shell 做扩展客户区最小原型，记录各平台原生按钮、标题文本、背景、安全区域和缩放边缘的效果；确定是否确有平台适配缺口。
2. 调整 `TopChromePanel` 的标题行与 `ApplicationWindow` 安全区域布局，消除重复标题条；macOS 隐藏左侧品牌 P 和原生窗口标题，保留顶栏中间标题，并让快捷操作从系统交通灯右侧开始；Windows/Linux 保留左侧品牌 P 和系统窗口按钮。Windows 按 Qt 6.11.2 标题栏按钮几何，使用 `GetDpiForWindow` 与 `GetSystemMetricsForDpi` 动态查询所需边框和标题栏高度；Linux 显式保留标题栏和按钮 hints，使用零宽窗口标题且清空应用显示名，避免 QPA 回退或追加 `panta`。Linux/Wayland 实机验收不得由 QML 单测替代。
3. 空白区进入系统拖动，双击按平台约定处理最大化/还原等操作；按钮、输入框、弹出菜单及文本选择区域排除拖动命中。若系统未接管双击，适配层明确实现并验证。
4. 必要的平台代码封装在 C++ 窗口控制适配层，QML 通过 ViewModel/控制器发出语义请求。处理窗口对象生命周期、激活状态、最小化/最大化/全屏及恢复，保留正常关闭流程。
5. 按平台验证原生视口对齐、遮挡与输入、跨屏 DPI 变化和窗口恢复；同步组件文档、相关代码注释与任务状态。

## 预计改动

- 现存：`qml/App.qml`、`qml/AppNoBridge.qml`、`qml/Panels/TopChromePanel.qml`、`qml/Components/Atoms/ChromeToolButton.qml`、`qml/Themes/Theme.qml`、`qml/CMakeLists.txt`。
- 现存：`native/app/main.cpp`、`native/app/CMakeLists.txt`、`native/app/window_chrome_metrics.*`。
- 现存：`tests/cpp/app/shell_module_load_test.cpp`、`tests/qml/`；补充窗口行为测试和必要的构建注册。
- 现存：[组件与主题文档](../modules/qml-components-and-theme.md)、本任务与[任务索引](../task-index.md)。

## 清理与兼容例外

实施时清理重复标题区和“窗口控制仅由独立系统标题栏承接”的失效代码注释；历史任务保留当时实现事实并链接本任务。不并存两套已废弃标题栏实现，不加入 Qt 6.5 兼容分支。当前无兼容例外。

## 验收标准

- [x] macOS 左侧不显示品牌 P 和系统按钮旁的重复 `panta` 窗口标题，系统红黄绿按钮位于最上方一行；顶栏中间标题保留，快捷操作从按钮右侧开始且不重叠；第二行导航栏顶格且无多余顶部空隙。
- [x] Windows/Linux 保持现有顶栏行列布局并保留左侧应用 P 标记；视觉上隐藏原生图标/标题，应用内容不覆盖系统窗口控制区域。
- [x] 三个平台的主窗口均无视觉重复标题条。
- [x] 顶栏工具组背景为统一浅灰纯色，没有渐变。
- [ ] Windows/Linux 系统最小化、最大化及关闭按钮可用；Windows 使用系统 DPI 指标匹配 Qt 按钮范围；Linux/Wayland 在实际窗口管理器下确认标题栏、图标和按钮表现。
- [ ] 窗口按钮与应用按钮/搜索的实际命中区域明确，不互相截获输入。
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
| 2026-10-14 | macOS；`cargo format --check`、`cargo test --locked --workspace` | 构建、格式、QML lint、Rust/native 行为测试通过 | 通过；CTest 75/75，Rust 与文档测试通过；包含 `ChromeToolButton` 资源、顶栏组件加载和 QML lint。保留既有 OCCT 弃用及平台环境警告，不影响本次结果 |
| 2026-10-14 | macOS；`cargo test --locked --workspace --no-default-features` | Bridge 关闭时 Shell、Objective-C++ 几何适配和应用入口仍可构建并通过测试 | 通过；CTest 30/30，包含 `Qml.ShellModuleLoads` |
| 2026-10-14 | macOS Qt 6.11.2；启动 `target/native/debug/app/panta.app` 并检查真实窗口与辅助功能树 | 顶栏保留居中标题；品牌标识不重复；快捷操作避开交通灯；右侧不留异常空白 | 通过；真实窗口可见居中 `panta 2027`，左侧无品牌 P/重复原生标题，快捷操作从原生控件右侧开始，搜索区延伸至窗口右侧。Windows/Linux/Wayland 尚未实机验收 |
| 2026-10-14 | `gpt-6.1-sol` medium 独立评审与修复后复核 | 检查需求覆盖、平台生命周期、构建接线及完整变更 | 已按评审修正原生控件边距刷新和安全区域重复留白；复核未发现新的阻塞问题。真实窗口和跨平台验收仍按上项跟踪 |
| 2026-10-14 | PR #32 CI 的 `cargo lint (includes, ubuntu)`；本机对变更翻译单元运行 `misc-include-cleaner` | 新增窗口指标适配代码通过 include-cleaner | CI 指出实现文件通过间接头文件使用 Qt 类型；补充 `QEvent`、`QObject`、`QQuickWindow`、`QWindow` 和 `QtCore/qtmetamacros.h` 直接包含。对应检查通过；同时将仓库中唯一的 `Q_EMIT` 用法统一为 `emit` |
| 2026-10-14 | macOS；`cargo format --check`、`cargo lint`、`cargo test --locked --workspace` | 格式、质量门禁与完整 Rust/native/QML 聚合测试通过 | 通过；lint 八阶段完成，CTest 75/75。测试只验证当前平台的系统 inset，Windows/Linux/Wayland 仍需实机验收 |
| 2026-10-17 | 维护者在 macOS、Windows、Linux 三个平台进行真实窗口视觉检查 | 顶栏平台布局、品牌/原生标题处理、控件位置与浅灰背景符合预期 | 维护者确认三个平台的视觉效果均正常。该确认覆盖外观，不代表窗口拖动/缩放、快捷键、Snap Layouts、全屏与跨 DPI 场景均已完成验收。 |

## 风险与回退

平台可能忽略部分 flags，安全边距未必足以表达期望的横向按钮布局；应以原型决定适配方式。完全去掉原生装饰可能丢失阴影、缩放或系统贴靠，VTK 原生子视图也可能出现偏移。候选路径未满足验收前不替换默认窗口行为；失败时撤回本任务窗口改动，保留现有可操作 Shell，不修改用户工程数据。

## 决策与工作记录

- 2026-09-22：按维护者新要求登记无边框外观与标题栏按钮融合，承接 029 的后续窗口工作；纠正 `QWindow::setTitleBar` 前提，优先评估当前 Qt 已有扩展客户区 API，无需等待不存在的接口或升级 Qt。
- 2026-10-14：开始实现。按维护者确认拆分 macOS 与 Windows/Linux 顶栏：macOS 保留无红底 P 标识、系统交通灯与顶格导航行；Windows/Linux 保持现有应用布局并在右侧加入系统窗口控制。P 菜单、Settings 入口及 macOS 系统菜单栏职责另登记为 115。
- 2026-10-14：维护者要求顶栏背景改为浅灰纯色。移除 `ToolGroup` 的上下渐变，统一使用 Theme 中既有的 `colorChrome`；macOS 标题栏避让由原生标准按钮矩形和对称外间距计算。
- 2026-10-14：按维护者截图修正 macOS 顶栏：移除左侧 P 标记和系统按钮旁重复的 `panta` 原生标题，保留中间 `panta 2027` 标题；避让按 AppKit 实际按钮组矩形及其镜像外间距计算，右侧不使用 SafeArea 边距，避免留下大块空白。
- 2026-10-14：核对 Apple 官方资料后，确认没有公开固定交通灯按钮尺寸/间距；移除固定 inset 配置，布局直接使用原生几何测量结果。
- 2026-10-14：窗口指标使用可见性、窗口状态、屏幕切换和 expose 事件刷新；不使用定时轮询。后续若确需引入轮询机制，先征求维护者意见。
- 2026-10-14：维护者确认三个平台都隐藏原生应用图标和标题，Windows/Linux 保留应用内 P 品牌块。独立评审发现 Windows 的 Qt `SafeArea.margins.right` 为零、Linux XCB 的无标题自定义标志会移除全部装饰，且 Qt 会把空窗口标题回退为应用名；改为 Windows 按 Qt 6.11.2 标题栏几何查询系统 DPI 指标，Linux 显式保留标题栏/按钮、用零宽标题并清空应用显示名。当前仅 macOS 有实机验收，Windows 与 Linux/Wayland 待实机确认。
- 2026-10-14：独立评审发现 macOS 无 Bridge 构建未启用 Objective-C++；将语言启用条件改为适用于所有 Apple 构建。并修正步骤中残留的固定最小避让宽度描述。
- 2026-10-14：独立评审后完成 macOS 无 Bridge 构建验证；同步移除该启动变体遗留的原生 `panta` 标题。顶栏 hover 使用中性灰、按压态使用稍深灰，并增加短时颜色过渡，不另设圆角。
- 2026-10-14：修复 PR #32 的 include-cleaner 门禁：`window_chrome_metrics.cpp` 显式包含使用的 Qt 类型头文件与 `emit` 宏定义头；同步规定自有 Qt 代码发射信号统一使用 `emit`。
- 2026-10-14：复核发现顶栏按钮仍继承通用组件的圆角；新增 `ChromeToolButton` 原子组件统一直角与顶栏悬停/按压颜色，避免重复配置并限定样式作用范围。
- 2026-10-14：维护者反馈悬停状态闪烁；移除通用工具按钮的颜色过渡和无其他用途的 Theme 时长 token，让悬停/按压状态直接切换，避免动画被频繁打断。
- 2026-10-14：按维护者要求将 Ribbon 背景渐变替换为统一的 `#b0b0b0` 深灰单色，并移除上下渐变专用 Theme token。
- 2026-10-14：移除顶栏中只用于平台条件的 `desktopPlatform`、`macOS` 派生属性和右边距条件分支，直接用系统测得的 `nativeControlsRightInset`。该值在 macOS/Linux 为零，在 Windows 由系统 DPI 指标计算；独立评审未发现新增问题，格式、lint 与聚合测试通过。
- 2026-10-17：维护者确认 macOS、Windows、Linux 三平台真实窗口的标题栏视觉效果均正常；完成相应外观验收。拖动/缩放、Snap Layouts、全屏、键盘交互与 DPI 跨屏等行为仍按下方清单单独验收。
- 2026-10-17：按维护者决定暂缓剩余窗口行为验收，任务状态改为 `deferred`；恢复时从未完成的验收项继续。

## 完成摘要

macOS、Windows、Linux 三平台的顶栏视觉布局已由维护者通过真实窗口确认，macOS 另有本机辅助功能树和自动化验证。原生控制交互、窗口拖动/缩放/全屏、Snap Layouts、键盘焦点、DPI 跨屏及 Linux Wayland compositor 行为尚未完成验收；维护者决定暂缓，任务状态为 `deferred`。
