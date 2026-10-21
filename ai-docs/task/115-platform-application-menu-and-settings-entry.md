# 115 — 平台应用菜单与 Settings 入口

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[051 无边框外观与一体化窗口标题栏](051-integrated-window-titlebar.md)、[125 设置服务与 Qt 持久化适配](125-settings-service-and-qt-adapter.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-10-14 / 2026-10-21

## 目标与背景

统一应用菜单入口并遵循平台惯例。Windows 与 Linux 点击顶栏左侧红底 P 打开紧凑后台面板，左侧为文件命令，右侧为最近项目，底部提供 Settings 和退出入口；原型尺寸约为 440 × 560 CSS px，从应用窗口顶部展开，完整包含 P 品牌块并覆盖标题栏和应用菜单行，受窗口边界约束。Close 作为文件菜单项紧跟 Study Properties，不实现关闭行为。macOS 不显示 P 图标或红底品牌块。macOS 的 panta 应用菜单和 File 菜单由应用定义，Edit、Window 等标准菜单交由 macOS/Qt 提供。panta 应用菜单中的 Settings 与桌面后台面板右下角 Settings 打开三端共享的 Settings 弹窗。

本任务负责平台菜单入口、Settings 入口和共享页面路由；设置 schema 与持久化由 125 定义，具体设置页面内容由消费任务定义。语言设置页面由 087 实现，并通过本任务提供的 Settings 入口访问；菜单组件不承载设置业务逻辑。

## 必读

- [Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)、[组件与主题](../modules/qml-components-and-theme.md)。
- [Qt Quick Controls MenuBar](https://doc.qt.io/qt-6/qml-qtquick-controls-menubar.html)、[QWindow](https://doc.qt.io/qt-6/qwindow.html)，查阅日期：2026-10-14。
- [验证与评审](../standards/validation-and-review.md)、[注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)。

## 范围与非目标

- Windows/Linux：可访问的 P 菜单按钮、约 440 × 560 CSS px 的紧凑受限后台面板（展开面板内显示完整 P 品牌块）、左侧文件命令/右侧最近项目布局、菜单列表内的 Close 占位项、右下角 Settings 与退出入口及与窗口标题栏布局的接线。
- macOS：系统菜单栏中显示 panta 应用菜单和自定义 File 菜单；Edit、Window 等标准菜单使用系统提供项。不显示 P 图标或红底品牌块。panta 菜单中的 Settings 打开与其他平台相同的 Settings 弹窗。Qt Quick Controls `MenuBar` 从 Qt 6.8 起在 macOS 使用原生菜单实现。
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
4. [ ] 依据确认原型实现复用菜单、跨平台入口和可注册的设置页面路由；无 Bridge Shell 提供一致入口。
5. [ ] 补充键盘/菜单行为测试，执行受影响验证和独立评审，并记录平台实机验收限制。

## 预计改动

- 原型：`prototypes/shell/application-menu/application-menu.html`、独立 `settings-dialog.html`；对话框框架和 Settings 通用布局归 `prototypes/shell/shell.js` / `shell.css`。
- 实现阶段预计涉及：`qml/Panels/TopChromePanel.qml`、`qml/App.qml`、`qml/AppNoBridge.qml`、`native/app/`。
- 可能新增：应用菜单/系统菜单适配组件与相关 QML/C++ 测试；具体路径在依赖完成后确定。

## 清理与兼容例外

无废弃项。菜单职责分别由平台入口呈现，不保留重复的 macOS P 弹出菜单与系统菜单入口。无兼容例外。

## 验收标准

- [ ] Windows/Linux 点击红底 P 打开可通过键盘操作并具备可访问名称的紧凑面板，约 440 × 560 CSS px 且不溢出窗口；展开面板包含完整 P 品牌块并覆盖标题栏及 Community 等菜单行。布局参考 Autodesk 文件页：左侧为 New/Open/Save/Export/Publish/分隔线/Print/分隔线/Project/Study Properties/分隔线/Close >，右侧为搜索和最近项目，底部右侧为 Settings/退出。
- [ ] macOS 不显示 P 图标或红底品牌块；panta 与 File 菜单由应用定义，Edit、Window 等标准菜单使用系统提供项。
- [ ] Settings 在 macOS panta 应用菜单和 Windows/Linux 后台页右下角均可访问，并打开同一个 Settings 弹窗；消费任务可向共享弹窗注册页面内容。
- [ ] 设置服务调用和错误反馈清晰；菜单不直接访问 QSettings 或实现具体设置项。
- [ ] 菜单交互不会触发窗口拖动；主应用与无 Bridge 变体行为一致。
- [ ] 相关自动测试、QML lint 及 macOS/Windows/Linux 实机检查结果已记录，任务状态与索引一致。

## 验证计划与结果

按仓库聚合入口验证：`cargo lint qmllint`、`cargo test --locked --workspace`。真实菜单栏行为需在 macOS 实机检查，Windows/Linux 需验证按钮、键盘访问和旁白名称；无 Bridge 变体单独确认。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-21 | `node --check`、Python HTML parser、SVG 引用核对、`git diff --check`、独立只读评审、维护者对照参考图确认 | 原型脚本/HTML 可解析；菜单图标统一复用 SVG；Settings 弹窗使用 Cancel / OK 并可重复打开、关闭；布局符合确认后的参考图 | 静态检查和独立评审通过，维护者确认布局。菜单 SVG 复用 shell symbol，P 品牌块完整包含在展开面板内；展开时按原品牌区实测位置对齐，不使用轮询。左侧 Close 紧跟 Study Properties 且带右箭头，为不实现动作的占位项；Publish/Print、Print/Project、Study Properties/Close 之间有分隔线；面板覆盖标题栏和 Community 等菜单行，右下角为 Settings / Exit。当前环境未完成浏览器实窗渲染，维护者对照原型确认了主要位置与布局；文字换行和小窗口滚动仍需在后续验收中检查。QML 实现已开始，受影响 qmllint 和测试待本阶段执行。 |

## 风险与回退

Qt 菜单栏在 macOS 的原生呈现受平台集成配置影响。若原生菜单命令不可访问或外观不符合系统惯例，先通过独立平台 adapter 调整；回退时保留平台可访问的菜单入口，不在 QML 内维护两份不一致命令。

## 决策与工作记录

- 2026-10-21：维护者确认 HTML 原型后进入 QML 实现。Windows/Linux 后台面板约 440 × 560 CSS px，从应用窗口顶部展开并覆盖标题栏及 Community 菜单行；面板完整包含 P 品牌块，展开时与原位置对齐。左侧按参考顺序放置 New/Open/Save/Export/Publish/分隔线/Print/分隔线/Project/Study Properties/分隔线/Close >（Close 暂不实现功能），右侧为搜索与最近项目，右下角提供 Settings/Exit；P 按钮悬停/展开均使用深红色；macOS 不显示 P，panta 与 File 菜单由应用定义，Edit、Window 等使用系统菜单；两种入口打开同一个独立 Settings 弹窗 HTML，按钮为 Cancel / OK。QML 实现与对应验证继续推进。
- 2026-10-14：根据维护者对 051 的补充，将 P 菜单/Settings 入口与 macOS 系统菜单栏职责拆为后续任务；Windows/Linux 使用顶栏 P 菜单，macOS 使用系统菜单栏。
- 2026-10-17：明确依赖顺序：125 提供设置服务，115 提供跨平台 Settings 入口与共享页面路由，087 在该路由中提供语言选择 UI；菜单任务不实现语言控件或设置业务逻辑。

## 完成摘要

HTML 原型已按最新平台设计完成并由维护者对照确认；QML 实现已开始，尚待受影响验证与独立评审。
