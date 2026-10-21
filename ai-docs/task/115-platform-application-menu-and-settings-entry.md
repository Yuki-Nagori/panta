# 115 — 平台应用菜单与 Settings 入口

- 状态：implemented
- 阶段：应用平台扩展
- 依赖：[051 无边框外观与一体化窗口标题栏](051-integrated-window-titlebar.md)、[125 设置服务与 Qt 持久化适配](125-settings-service-and-qt-adapter.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-10-14 / 2026-10-21

## 目标与背景

统一应用菜单入口并遵循平台惯例。Windows 与 Linux 点击顶栏左侧红底 P 打开紧凑后台面板，左侧为文件命令，右侧为最近项目，底部提供 Settings 和退出入口；原型尺寸约为 440 × 560 CSS px，从应用窗口顶部展开，完整包含 P 品牌块并覆盖标题栏和应用菜单行，受窗口边界约束。Close 作为文件菜单项紧跟 Study Properties，不实现关闭行为。macOS 不显示 P 图标或红底品牌块，使用 Qt Quick Controls 原生 `MenuBar`；panta 与 File 菜单由应用定义，Edit 提供标准文本编辑命令，Window 提供原生窗口操作。panta 应用菜单中的 Settings 与桌面后台面板右下角 Settings 打开三端共享的 Settings 弹窗。

本任务负责平台菜单入口、Settings 入口和共享页面路由；设置 schema 与持久化由 125 定义，具体设置页面内容由消费任务定义。语言设置页面由 087 实现，并通过本任务提供的 Settings 入口访问；菜单组件不承载设置业务逻辑。

## 必读

- [Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)、[组件与主题](../modules/qml-components-and-theme.md)。
- [Qt Quick Controls MenuBar](https://doc.qt.io/qt-6/qml-qtquick-controls-menubar.html)、[QWindow](https://doc.qt.io/qt-6/qwindow.html)，查阅日期：2026-10-14。
- [验证与评审](../standards/validation-and-review.md)、[注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)。

## 范围与非目标

- Windows/Linux：可访问的 P 菜单按钮、约 440 × 560 CSS px 的紧凑受限后台面板（展开面板内显示完整 P 品牌块）、左侧文件命令/右侧最近项目布局、菜单列表内的 Close 占位项、右下角 Settings 与退出入口及与窗口标题栏布局的接线。
- macOS：Qt Quick Controls `MenuBar` 从 Qt 6.8 起在 macOS 使用原生菜单实现；显示 panta 应用菜单、File、Edit 与 Window 菜单，不显示 P 图标或红底品牌块。Edit 提供标准文本编辑命令并路由到当前焦点文本控件；Window 使用窗口系统操作。panta 菜单中的 Settings 打开与其他平台相同的 Settings 弹窗。
- Settings 入口和平台菜单路由由本任务提供；设置状态与持久化依赖 Rust 设置服务及 C++ Qt adapter（125），菜单不直接访问 QSettings。
- 非目标：完整设置页面、设置 schema、语言选择 UI、用户账户/登录实现及平台外观重构。
- QML 实现前须先完成并由维护者确认 HTML 原型；原型展示统一 Settings 弹窗和跨平台菜单差异，不实现真实设置服务。

## 前置条件与待决策

- 051 已进入 `implemented`，Windows/Linux 实机验收不阻塞本任务；按已实现的标题栏区域接入菜单，避免与系统按钮冲突。
- 125 已进入 `implemented`；可直接通过设置服务与 Qt 持久化 adapter 接入 Settings 入口，菜单动作不伪造设置状态。
- 本任务提供可供设置页面接入的共享路由；页面内容由独立消费任务（首个为 087）实现，不将具体设置项塞入菜单任务。

## 实施步骤

1. [x] 按 [HTML 原型工作流](../../prototypes/shell/README.md)登记原型先行与用户确认关口。
2. [x] 建立 Windows/Linux 与 macOS 两种平台状态；macOS 仅呈现无 P 图标的系统菜单栏，panta 与 File 为应用菜单，Settings 在三端打开同一弹窗原型。
3. [x] 完成原型主要布局与交互，并由维护者对照参考图确认；确认前不实现 QML。
4. [x] 依据确认原型实现复用菜单、跨平台入口和可注册的设置页面路由；无 Bridge Shell 提供一致入口。
5. [x] 补充键盘/菜单行为测试，执行受影响验证和独立评审，并记录平台实机验收限制。

## 预计改动

- 原型：`prototypes/shell/application-menu/application-menu.html`、独立 `settings-dialog.html`；对话框框架和 Settings 通用布局归 `prototypes/shell/shell.js` / `shell.css`。
- 实现阶段预计涉及：`qml/Panels/TopChromePanel.qml`、`qml/App.qml`、`qml/AppNoBridge.qml`、`native/app/`。
- 可能新增：应用菜单/系统菜单适配组件与相关 QML/C++ 测试；具体路径在依赖完成后确定。

## 清理与兼容例外

无废弃项。菜单职责分别由平台入口呈现，不保留重复的 macOS P 弹出菜单与系统菜单入口。无兼容例外。

## 验收标准

- [ ] Windows/Linux 点击红底 P 打开可通过键盘操作并具备可访问名称的紧凑面板，约 440 × 560 CSS px 且不溢出窗口；展开面板包含完整 P 品牌块并覆盖标题栏及 Community 等菜单行。布局参考 Autodesk 文件页：左侧为 New/Open/Save/Export/Publish/分隔线/Print/分隔线/Project/Study Properties/分隔线/Close >，右侧为搜索和最近项目，底部右侧为 Settings/退出。
- [x] macOS 不显示 P 图标或红底品牌块；panta 与 File 菜单由应用定义；Edit、Window 使用原生菜单呈现。Qt Quick 编辑命令通过 `QGuiApplication::focusObject()` 路由到当前活动窗口的文本控件，窗口操作调用 `QWindow` API。多窗口列表不在当前单窗口设计范围内。
- [ ] Settings 在 macOS panta 应用菜单和 Windows/Linux 后台页右下角均可访问，并打开同一个 Settings 弹窗；消费任务可向共享弹窗注册页面内容。
- [ ] 设置服务调用和错误反馈清晰；菜单不直接访问 QSettings 或实现具体设置项。
- [ ] 菜单交互不会触发窗口拖动；主应用与无 Bridge 变体行为一致。
- [ ] 相关自动测试、QML lint 及 macOS/Windows/Linux 实机检查结果已记录，任务状态与索引一致。

## 验证计划与结果

按仓库变更范围验证 Qt/C++ 格式、`cargo lint qmllint` 和受影响的 Shell 模块测试；Shell 测试覆盖焦点在独立 `QQuickWindow` 时的选择、复制、剪切、粘贴路由。真实菜单栏行为需在 macOS 实机检查，Windows/Linux 需验证按钮、键盘访问和旁白名称；无 Bridge 变体单独确认。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-21 | `node --check`、Python HTML parser、SVG 引用核对、`git diff --check`、独立只读评审、维护者对照参考图确认 | 原型脚本/HTML 可解析；菜单图标统一复用 SVG；Settings 弹窗使用 Cancel / OK 并可重复打开、关闭；布局符合确认后的参考图 | 静态检查和独立评审通过，维护者确认布局。菜单 SVG 复用 shell symbol，P 品牌块完整包含在展开面板内；展开时按原品牌区实测位置对齐，不使用轮询。左侧 Close 紧跟 Study Properties 且带右箭头，为不实现动作的占位项；Publish/Print、Print/Project、Study Properties/Close 之间有分隔线；面板覆盖标题栏和 Community 等菜单行，右下角为 Settings / Exit。当前环境未完成浏览器实窗渲染，维护者对照原型确认了主要位置与布局；文字换行和小窗口滚动仍需在后续验收中检查。QML 实现已开始，受影响 qmllint 和测试待本阶段执行。 |
| 2026-10-21 | `cargo lint qmllint`、`cargo format --check --changed`、`edit_commands_follow_focus_in_separate_window` | 编辑命令使用 Qt 当前焦点对象；TextField 和独立 Window 中的 TextArea 支持选择、复制、剪切、普通/HTML 粘贴，TextArea 只读时 Undo/Redo 不改内容 | 格式与 QML lint 通过；C++ 构建通过；隔离 Shell 用例 3/3 通过。`Qml.ShellModuleLoads` 首次运行在既有 `loads_shell_module` 的 `searchVisible()` 断言失败并超时；后续重跑通过（见下方 final review 验证记录）。macOS 原生菜单行为仍待实机验收。独立只读复核未发现焦点路由遗留问题。 |
| 2026-10-21 | `cargo format --changed`、`ninja -C target/native/debug all_qmllint`、`ctest --test-dir target/native/debug --output-on-failure -R Qml.ShellModuleLoads`、macOS 实窗打开 Settings 并按 Escape | 所有弹窗共用 Escape 行为和统一显示/激活入口；保存期间不能经 Escape 或标题栏关闭；Settings 关闭后焦点返回主窗口 | 格式、QML lint、Shell 聚合测试通过；构建真实 macOS 应用成功。实窗确认原生 panta 菜单可以打开 Settings，Escape 关闭弹窗并返回主窗口焦点；macOS 将标签显示为 Preferences（Qt/macOS 菜单角色行为，已在风险说明中记录）。`DialogWindow.present()` 合并重复的居中/显示/激活逻辑；修复 Fill 与 Gate 弹窗保存时标题栏关闭按钮仍可用。独立只读评审未发现新问题。Windows/Linux 实机验收仍待维护者设备确认，不阻塞实现状态。 |
| 2026-10-21 | Final review：缺省设置图标路由；`QT_QPA_PLATFORM=offscreen QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/performance/panta_qml_cpu_benchmark measures_application_menu_construct` | 替换已有页面时保留原图标呈现方式；无图标的新页面使用可着色 glyph；记录菜单和 Settings 的实际构造、打开与布局成本 | 回归验证覆盖 General 页面替换时保留彩色图标、新页面缺省使用单色 `history` glyph；Shell 定向用例 5/5、`Qml.ShellModuleLoads` 1/1、QML lint、`git diff --check` 通过。CPU 基准运行于 macOS 27.0.1 arm64、Qt 6.11.2、Debug 基准目标，31 个采样；范围为创建/显示宿主窗口、打开菜单/弹窗和处理布局事件，不包含销毁成本及 GPU 呈现。PlatformMenuBar/ApplicationMenu 构造 p50/p95 为 906/1169 μs；后台菜单最近项目 0/8/100 个 p50/p95 为 1767/2191、2208/2643、2746/3070 μs（100 项模型实际实例化 20 个 ListView 委托，可能包括缓存区域）；三页 Settings 弹窗为 1116/1447 μs。GPU harness 增加后台面板 0/8/100 项渲染场景并可编译；本机本轮无法执行：offscreen QPA 不支持创建图形上下文，默认 Cocoa 启动又报告无屏幕。Settings 是独立 `QQuickWindow`，现有离屏 renderer 不覆盖该窗口；上述性能数值仅描述 CPU 构造/布局。原生菜单栏由系统绘制，Qt Quick GPU 基准不代表其平台呈现成本。 |
| 2026-10-21 | [PR #85 全部 CI 检查](https://github.com/Yuki-Nagori/panta/actions/runs/39611577779)、`cargo verify-changed`、受影响 QML 定向 CTest、独立只读评审 | 面板在 Overlay 初始化前后均有有效尺寸；原型图标资源清单完整；动态图标颜色模式检查覆盖已声明路由 | PR #85 CI 全通过，包含 macOS、Ubuntu、Windows 构建与测试及全部 lint/audit。修复 Overlay 零尺寸时菜单宽高塌为 0；补齐与原型一致的 `settings`、`left`、`palette` SVG；一致性测试检查后台命令和注册设置页的动态图标颜色模式。受影响 `Qml.ThemeAtoms`、`Qml.ShellModuleLoads` 均通过。本地 `cargo verify-changed` 的 Rust 测试、格式和 lint 通过；91 个 native/QML CTest 中 90 个通过，唯一未通过的 `QtAdapter.UserSettingsStore` 因沙箱禁止写入 `~/Library/Preferences`，CI 三平台均通过包含该测试的完整检查。独立只读评审未发现问题。 |

## 风险与回退

Qt Quick Controls `MenuBar` 在 macOS 使用原生菜单栏，Qt 平台插件负责把原生菜单选择转成 Qt 菜单动作；不直接接管 AppKit responder chain。Qt Quick 文本控件需要应用将动作路由到焦点对象，使用 `QGuiApplication::focusObject()` 覆盖主窗口及独立对话框，不能只读取主窗口的 `activeFocusItem`。应用菜单中的 Settings 文案受 Qt 菜单合并规则和 macOS 版本影响，可能被系统显示为 “Preferences”；当前保留原生菜单行为，不为强制改名引入 Qt Labs Platform 或将 `QGuiApplication` 改成 `QApplication`。准确显示名称作为后续问题单独跟进。当前应用只有一个主窗口，Window 菜单不列举其他窗口。若原生菜单命令不可访问或外观不符合系统惯例，先通过独立平台 adapter 调整；回退时保留平台可访问的菜单入口，不在 QML 内维护两份不一致命令。

## 决策与工作记录

- 2026-10-21：维护者确认 HTML 原型后进入 QML 实现。Windows/Linux 后台面板约 440 × 560 CSS px，从应用窗口顶部展开并覆盖标题栏及 Community 菜单行；面板完整包含 P 品牌块，展开时与原位置对齐。左侧按参考顺序放置 New/Open/Save/Export/Publish/分隔线/Print/分隔线/Project/Study Properties/分隔线/Close >（Close 暂不实现功能），右侧为搜索与最近项目，右下角提供 Settings/Exit；P 按钮悬停/展开均使用深红色；macOS 不显示 P，panta 与 File 菜单由应用定义，Edit、Window 等使用系统菜单；两种入口打开同一个独立 Settings 弹窗 HTML，按钮为 Cancel / OK。QML 实现与对应验证继续推进。
- 2026-10-21：macOS 继续使用 Qt Quick Controls 原生 `MenuBar`；Qt 的系统菜单角色可能把 Settings 显示为 Preferences。暂不切换 `QGuiApplication`、不引入 Qt Labs Platform 来强制标签，准确显示名称留作后续问题跟进。原生菜单交互与标题仍需 macOS 实窗验收。
- 2026-10-21：Edit 菜单由 Qt 的原生菜单集成触发 Qt `Action`，不直接调用 AppKit responder chain。Qt Quick 焦点是窗口级对象，应用通过 `QGuiApplication::focusObject()` 路由当前活动窗口（含独立弹窗）的编辑动作；能力状态读取输入控件自身属性并订阅通知，避免自行推算剪贴板格式。
- 2026-10-14：根据维护者对 051 的补充，将 P 菜单/Settings 入口与 macOS 系统菜单栏职责拆为后续任务；Windows/Linux 使用顶栏 P 菜单，macOS 使用系统菜单栏。
- 2026-10-17：明确依赖顺序：125 提供设置服务，115 提供跨平台 Settings 入口与共享页面路由，087 在该路由中提供语言选择 UI；菜单任务不实现语言控件或设置业务逻辑。

## 完成摘要

HTML 原型与 QML 实现已完成；自动验证、macOS 实窗检查与独立评审通过。任务状态为 `implemented`；Windows/Linux 实机行为仍待维护者设备验收，不阻塞实现状态。
