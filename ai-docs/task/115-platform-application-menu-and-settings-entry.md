# 115 — 平台应用菜单与 Settings 入口

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[051 无边框外观与一体化窗口标题栏](051-integrated-window-titlebar.md)、[048 设置服务与 Qt 持久化适配](048-settings-service-and-qt-adapter.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-10-14 / 2026-10-14

## 目标与背景

统一应用菜单的入口，同时遵循各平台惯例。Windows 与 Linux 点击顶栏左侧带红底的 P 标识时打开应用菜单，Settings 入口置于菜单内；macOS 使用系统菜单栏承载应用菜单与 Settings 入口，不重复放置 P 弹出菜单。

本任务只规划菜单入口、平台呈现和到 Settings 页面/命令的接线。设置 schema、持久化和页面内容由各自任务定义，不在菜单组件中实现。

## 必读

- [Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)、[组件与主题](../modules/qml-components-and-theme.md)。
- [Qt Quick Controls MenuBar](https://doc.qt.io/qt-6/qml-qtquick-controls-menubar.html)、[QWindow](https://doc.qt.io/qt-6/qwindow.html)，查阅日期：2026-10-14。
- [验证与评审](../standards/validation-and-review.md)、[注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)。

## 范围与非目标

- Windows/Linux：可访问的 P 菜单按钮、菜单结构、Settings 命令及与窗口标题栏布局的接线。
- macOS：系统菜单栏原生菜单及 Settings 命令，不显示重复的 P 弹出菜单。Qt Quick Controls `MenuBar` 从 Qt 6.8 起在 macOS 使用原生菜单实现。
- 设置状态与持久化依赖 Rust 设置服务及 C++ Qt adapter；菜单不直接访问 QSettings。
- 非目标：完整设置页面、设置 schema、语言选择 UI、用户账户/登录实现及平台外观重构。

## 前置条件与待决策

- 051 完成并验证平台窗口区域后开始接入菜单，避免标题栏按钮区域与菜单入口冲突。
- 048 完成设置服务与 Qt 持久化 adapter 后接入设置命令；在此之前菜单动作不伪造设置状态。
- 实施前明确 Settings 页面实际承接任务/路由；如果页面任务尚未建立，先更新任务索引与依赖，不在本任务内顺手扩展页面范围。

## 实施步骤

1. 核实现有 QML 导航命令和 Qt macOS 菜单行为，选择可复用的菜单命令接口。
2. 为 Windows/Linux 实现 P 菜单按钮和 Settings 入口；macOS 将等价命令放进系统菜单栏。
3. 保证键盘、旁白和窗口焦点行为符合平台预期，菜单不会触发标题栏拖动。
4. 接入已定义的设置页面/命令，补充测试和平台验收记录。

## 预计改动

- 现存：`qml/Panels/TopChromePanel.qml`、`qml/App.qml`、`qml/AppNoBridge.qml`、`native/app/`。
- 可能新增：应用菜单/系统菜单适配组件与相关 QML/C++ 测试；具体路径在依赖完成后确定。

## 清理与兼容例外

无废弃项。菜单职责分别由平台入口呈现，不保留重复的 macOS P 弹出菜单与系统菜单入口。无兼容例外。

## 验收标准

- [ ] Windows/Linux 点击红底 P 打开可通过键盘操作并具备可访问名称的应用菜单。
- [ ] macOS 通过系统菜单栏访问同一组应用命令，不显示重复的 P 弹出菜单。
- [ ] Settings 命令只在设置页面/服务可用后接线，行为和错误反馈清晰。
- [ ] 菜单交互不会触发窗口拖动；主应用与无 Bridge 变体行为一致。
- [ ] 相关自动测试、QML lint 及 macOS/Windows/Linux 实机检查结果已记录，任务状态与索引一致。

## 验证计划与结果

按仓库聚合入口验证：`cargo lint qmllint`、`cargo test --locked --workspace`。真实菜单栏行为需在 macOS 实机检查，Windows/Linux 需验证按钮、键盘访问和旁白名称；无 Bridge 变体单独确认。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 依赖完成后的实施验证 | 满足上述验收条件 | 未执行；任务尚未开始 |

## 风险与回退

Qt 菜单栏在 macOS 的原生呈现受平台集成配置影响。若原生菜单命令不可访问或外观不符合系统惯例，先通过独立平台 adapter 调整；回退时保留平台可访问的菜单入口，不在 QML 内维护两份不一致命令。

## 决策与工作记录

- 2026-10-14：根据维护者对 051 的补充，将 P 菜单/Settings 入口与 macOS 系统菜单栏职责拆为后续任务；Windows/Linux 使用顶栏 P 菜单，macOS 使用系统菜单栏。

## 完成摘要

未开始，等待 051 和 048 完成。
