# 051 — 无边框外观与一体化窗口标题栏

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005 Qt/QML 主窗口](005-qt-qml-shell.md)、[029 QML 组件库](029-qml-component-library.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-22 / 2026-10-21

## 目标与背景

主窗口需要把应用顶栏与系统窗口控制整合，同时保留各平台原有操作习惯。实施前，`App.qml` 使用普通 `ApplicationWindow`，`TopChromePanel.qml` 位于系统标题栏下方；029 尚未实现标题栏融合。

平台布局按维护者确认分开处理：

- macOS：最上方一行使用系统红、黄、绿窗口控制按钮；不显示左侧 P 品牌标记和系统按钮旁重复的 `panta` 窗口标题，保留顶栏中间的 `panta 2027` 标题，快捷操作与搜索区从系统按钮右侧开始；顶栏使用浅灰背景，第二行导航栏顶格排列。
- Windows 与 Linux：保留左侧应用 P 标记，隐藏原生应用图标和标题，并保留系统最小化、最大化及关闭按钮。Windows 按 Qt 当前标题栏几何从 Windows DPI 系统指标运行时计算右侧避让；Linux 保留窗口管理器装饰和原生按钮，使用零宽标题并不请求系统菜单图标，仍需 Linux/Wayland 实机验收。
- 点击带红底的 P 打开应用菜单、菜单中的 Settings 入口，以及 macOS 以系统菜单栏承载这些菜单职责，属于独立后续任务 115；本任务只预留不冲突的窗口标题区域，不实现菜单或设置页面。后续依赖顺序为：125 提供设置服务，115 提供跨平台菜单入口与共享设置路由，087 将语言选择页面接入该路由。

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
- [x] macOS 标题栏布局通过真实窗口检查；缩放窗口后布局保持正确，绿色按钮可进入全屏。主窗口保留系统原生控制，不自绘平台按钮。
- [x] 搜索焦点滚动、标题行几何、键盘 Tab 顺序、QML 资源加载、无 Bridge 启动变体及相关静态检查由既有验证覆盖；详见下方验证记录。
- [x] macOS 实机确认全屏进入/退出、最小化后恢复、原生窗口按钮及边框缩放；确认窗口恢复后布局可用。
- [x] macOS 实机复验居中标题空白区拖动和当前系统双击动作；确认拖动和缩放工作正常，未再观察到约 1 秒的缩放等待；业务控件与搜索输入不会触发窗口移动。
- [ ] macOS 其它双击偏好（Fill、Minimize、No Action）、跨屏 DPR 与四角缩放仍待实机核对；Fill 与 Qt 最大化请求是否完全等效原生 Fill 尚未确认。
- [ ] macOS 实机确认四角缩放及恢复后 VTK 视口输入；边框缩放已由维护者确认。
- [ ] Windows 实机确认原生最小化、最大化、关闭、恢复、边缘缩放、标题区拖动/双击、Win+方向键和 Snap Layouts；确认系统按钮命中区与按 DPI 计算的内容避让一致，并检查 100%/150%/200% 缩放与跨屏时无错位。
- [ ] Linux/Wayland 在明确记录的 compositor 下实机确认原生标题栏、按钮、系统移动/缩放及最大化恢复、内容避让和跨屏 DPI 行为；记录 compositor 的双击标题栏偏好是否可由客户端沿用，确认 VTK 视口对齐、输入和显示生命周期正常。
- [ ] macOS、Windows、Linux 实机确认点击业务控件、编辑/选择搜索文本、使用菜单及键盘焦点不会误触窗口移动；原生窗口控制具有正确的可访问名称和交互状态。
- [ ] 无 Bridge 变体真实窗口仍可启动、移动、缩放与关闭；记录 Qt 平台插件、系统/窗口管理器、DPR、窗口状态和结果。macOS 可用平台验收项全部完成后可标为 `implemented`；Windows/Linux 实机验收完成后转为 `done`。

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
| 2026-10-21 | macOS arm64，Qt 6.11.2；`cargo build --locked`，启动 `target/native/debug/app/panta.app` | 当前分支构建通过；真实窗口保留系统交通灯、居中标题及一致的 Ribbon/VTK 内容布局 | 构建通过。真实窗口检查确认交通灯和内容未重叠；触发原生 Zoom 后窗口尺寸变化且布局仍完整；点击原生绿色按钮后系统标题栏控件隐藏、窗口进入全屏。按验收规则关闭了本次启动的应用。Windows/Linux 的原生窗口行为仍待实机验证。 |
| 2026-10-21 | `tests/cpp/app/shell_module_load_test.cpp` 当前实现核对 | 自动化覆盖窗口标志、平台标题/品牌可见性、标题栏内缩几何、窄窗口搜索可见性、键盘焦点顺序及面板交互 | 源码核对确认上述回归项已在 Shell 测试中覆盖；本轮 `cargo build --locked` 通过。该 Qt 测试不模拟 Windows/Wayland 窗口管理器，不能替代相应平台的实机验收。 |
| 2026-10-21 | macOS arm64；真实窗口操作 | 验证系统缩放、全屏、最小化/恢复、标题栏双击与窗口生命周期 | 缩放与绿色按钮进入全屏可观察；退出全屏快捷键没有让辅助功能树恢复原生窗口控件，故退出全屏仍未确认。最小化操作后计算机使用接口无法采集窗口截图（ScreenCaptureKit `-3812`）；重新绑定应用后窗口可见，但不足以确认最小化/恢复过程。后续通过 macOS Panta 菜单退出应用并确认进程关闭。全屏后退出应用再启动时恢复为普通窗口。 |
| 2026-10-21 | macOS 系统设置与 Panta 真实窗口 | 确认标题栏双击偏好并检查业务控件/搜索输入不会拖动窗口 | 系统设置显示“窗口标题栏连按操作”为“缩放”。点击 Ribbon 项并输入、清除临时搜索文本时窗口仍保持原布局。双击标题栏过程中 CUA 曾返回 `noWindowsAvailable`，重绑定并执行辅助功能 Raise 后窗口恢复；无法稳定采集双击前后尺寸，因此双击偏好行为仍未验收。 |
| 2026-10-21 | macOS arm64；维护者真实窗口手动操作 | 检查标题区拖动、双击系统动作、原生按钮、边框缩放、全屏退出和最小化恢复 | 原生按钮、边框缩放、全屏退出、最小化恢复通过；标题区拖动和双击操作未生效。已在居中标题空白区加入 Qt 系统移动请求；双击由 macOS 偏好分派原生动作。Windows/Linux 使用 Qt 向窗口系统请求移动与最大化/还原。当前 macOS Fill 是否等效原生 Fill、Linux compositor 双击偏好是否可沿用尚未确认，均需实机核验。 |
| 2026-10-21 | `qml/App.qml` 信号处理；`cargo build --locked` | 确认构建日志不再提示 `visibility` 隐式注入弃用 | 将窗口状态、宽度和高度处理器改为显式参数，并在尺寸处理器中读取窗口的 `visibility` 属性。构建通过，日志中未出现该弃用警告。 |
| 2026-10-21 | macOS arm64；维护者重新运行当前分支窗口并双击标题栏 | 当前系统 Zoom 偏好立即触发缩放；输入法诊断日志不影响窗口行为 | 维护者确认操作“好像没什么问题”，未再报告约 1 秒等待。运行期间仍出现 `IMKCFRunLoopWakeUpReliable` 日志；窗口控制调用链不涉及输入法，且用户未报告输入异常。 |
| 2026-10-21 | `cargo build --locked`、`cargo format --check`、`cargo lint qmllint`、`git diff HEAD --check`；`gpt-6.1-sol` medium 独立只读评审 | 构建、格式、QML lint 与变更空白检查通过；评审核对完整实现和状态证据 | 全部检查通过。评审未发现代码阻塞问题；指出 macOS 剩余验收尚未完成，因此状态保留 `in-progress`；Windows/Linux 与待验证 macOS 场景继续列在验收清单。 |

## 风险与回退

平台可能忽略部分 flags，安全边距未必足以表达期望的横向按钮布局；应以原型决定适配方式。macOS 原生双击偏好包含 Fill，当前 Qt 的最大化请求是否与其完全一致尚未实测；不得将其误记为已验证原生等效。Linux/Wayland 的客户端最大化请求由 compositor 执行，但双击语义目前由应用选择，未证明会沿用桌面环境的可配置偏好。完全去掉原生装饰可能丢失阴影、缩放或系统贴靠，VTK 原生子视图也可能出现偏移。候选路径未满足验收前不替换默认窗口行为；失败时撤回本任务窗口改动，保留现有可操作 Shell，不修改用户工程数据。

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
- 2026-10-17：维护者确认 macOS、Windows、Linux 三平台真实窗口的标题栏视觉效果均正常；完成相应外观验收。窗口管理器行为继续单独验证。
- 2026-10-17：按维护者决定暂缓窗口行为验收，状态曾改为 `deferred`。
- 2026-10-21：维护者确认 macOS 原生按钮、边框缩放、全屏退出及最小化恢复正常；首次检查时标题区拖动和双击系统动作未生效。三平台标题空白区统一使用 Qt 的系统移动请求；Windows/Linux 双击经 Qt 向窗口系统请求最大化/还原，macOS 按标题栏双击偏好分派原生动作。后续维护者复测当前 Zoom 配置下的标题区操作，反馈无明显问题。macOS Fill 与 Cocoa Zoom 的等效性、Linux compositor 的可配置双击语义仍待核实。同步修复 `App.qml` 信号参数弃用写法；Windows/Linux 实机行为仍待验证。
- 2026-10-21：为标题栏 `TapHandler` 显式设置 `exclusiveSignals: TapHandler.DoubleTap`，让该区域只识别双击动作。维护者重新运行后反馈窗口操作“好像没什么问题”；仍观察到 macOS `IMKCFRunLoopWakeUpReliable` 输入法诊断日志，但未报告输入异常，且未发现它与缩放调用链相关。macOS 当前系统 Zoom 偏好下的标题栏行为已复验；其它 macOS 偏好、跨屏 DPI、四角缩放、VTK 输入和无 Bridge 实窗验收仍未完成，因此状态按索引约定保持 `in-progress`。上述 macOS 验收完成后可转为 `implemented`，Windows/Linux 实机验收后转为 `done`。

## 完成摘要

标题栏融合实现已完成；维护者确认三平台视觉布局正常，macOS 本机确认原生按钮、边框缩放、全屏进入/退出、最小化恢复、标题区拖动及当前 Zoom 双击行为可用。因 macOS 其它双击偏好、跨屏 DPI、四角缩放、VTK 输入和无 Bridge 实窗验收仍未完成，任务保持 `in-progress`；这些 macOS 验收完成后可转为 `implemented`。Windows 原生 hit-test、Snap Layouts / DPI 行为，以及 Linux/Wayland 窗口管理器行为待实机验收，完成后转为 `done`。`IMKCFRunLoopWakeUpReliable` 日志仍出现，但当前未影响用户输入或窗口操作，且未发现其与标题栏调用链的联系。
