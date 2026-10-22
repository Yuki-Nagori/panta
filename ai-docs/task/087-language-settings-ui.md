# 087 — 语言设置界面

- 状态：done
- 阶段：应用平台扩展
- 依赖：[022](022-ui-internationalization.md)、[029](029-qml-component-library.md)、[086](086-qt-platform-adapter.md)、[115](115-platform-application-menu-and-settings-entry.md)、[125](125-settings-service-and-qt-adapter.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-29 / 2026-10-22

## 目标与背景

在应用设置界面提供语言选择。用户从跨平台 Settings 入口进入设置区域；入口和共享路由由 115 提供，语言设置页面由本任务实现。语言目录、切换命令和当前 locale 由 Rust/`Settings` 后端提供；QML 只展示支持项并发出用户意图，不自行访问 QSettings、解析 `.pa` 或加载 QM。切换与翻译刷新机制由 086 接通，用户偏好持久化由 125 提供。

## 必读

- [国际化模块](../modules/internationalization.md)、[应用与存储架构](../architecture/application-and-storage.md)
- [分层规则](../standards/layering.md)、[Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

包含设置页的语言选择控件、对 `Settings` locale 属性/切换 API 的调用、失败反馈、设置服务持久化和启动恢复，以及应用语言切换时 macOS Cocoa 原生菜单角色所需的本地化词条。

不包含 locale 目录定义、QM 资源编译、QTranslator 生命周期、Rust locale 校验、其他设置类别、RTL 支持或翻译覆盖扩充。

## 前置条件与待决策

- 022 的语言目录与 QM 构建验收完成。
- 125 提供 Rust 设置 schema、CXX DTO 和 Qt QSettings adapter；偏好写入只在 translator 加载成功后提交。
- 086 提供 QML `Settings` 类型、supported/current locale 和运行时切换 API。
- 115 提供 Windows/Linux 的 P 菜单与 macOS 系统菜单栏 Settings 入口，以及共享设置页面路由；本任务将语言选择内容接入该路由。
- 029 提供的表单/选择器组件可满足布局与键盘操作；否则记录最小必要组件扩展。

## 实施步骤

1. [x] 更新并由维护者确认 HTML 原型，覆盖 Settings 的 Language 页面、可用语言、提交/取消及失败状态。
2. [x] 以 Rust 返回的支持列表构造语言选择控件，不在 QML 维护第二份 locale 列表。
3. [x] 对话框内暂存候选语言；OK 提交，失败时保留对话框并显示诊断；Cancel 丢弃候选。
4. [x] 启动时从设置服务恢复 locale；无效或缺失偏好使用英文默认，并保持显示文案一致。
5. [x] 记录真实窗口、自动回归、构建验证和独立评审结果。

## 预计改动

预计涉及 `qml/` 语言设置页面与公共组件、`native/bridge/` settings view model，以及 Rust 设置服务/CXX DTO；路径以 125 和 115 的实际接口为准。

## 清理与兼容例外

删除 QML 或 C++ 中重复维护的 locale 列表和临时切换入口。无兼容例外。

## 验收标准

- [x] 设置界面只展示 Rust 返回的可用语言，并调用 `Settings` 后端完成切换。
- [x] 成功切换后 QML 与 C++ 展示文本更新；非默认语言持久化并在重新创建设置服务时恢复。
- [x] QM 加载或设置写入失败时保留上一有效语言和偏好，并向用户提供可理解反馈。
- [x] macOS 切换应用语言后 About / Settings / Quit 仍合并进系统应用菜单，不出现第二个自定义应用名菜单；系统 Settings 文案随应用语言显示。
- [x] QML 不访问 QSettings、不解析 `.pa`、不操作 `QTranslator`；Rust 不依赖 Qt。
- [x] 代码、配置、task 与索引一致，真实验证记录在本任务。

## 验证计划与结果

真实窗口操作切换语言、失败反馈和重启恢复；使用项目适用的 Cargo 构建/格式入口。依赖未完成前不声称这些入口已通过。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-22 | 原型与 QML 迁移 | 可切换设置分类并演示语言候选、OK/Cancel 与错误状态；QML 结构复用共享 DialogWindow / Settings 路由 | `node --check prototypes/shell/shell.js`、`git diff --check` 通过；维护者逐步确认布局要求后完成 QML 迁移。独立评审后修复错误状态被隐藏及内部术语文案。当前浏览器策略拒绝 `file://`，原型未完成浏览器实窗渲染验收 |
| 2026-10-22 | macOS 中文菜单与重启实窗验收；中英文目录清理；`cargo build --locked`；`I18n.CompiledQmLoads`、`Qml.SettingsLanguageSwitch`、`QtAdapter.TranslationCatalog`；`cargo format --check --changed` | Cocoa 能识别简体中文 About / Settings / Quit 等角色，系统菜单显示“设置…”且不留下第二个 `panta` 顶级项；目录只保留有效条目；非默认语言重启恢复，加载或写入失败不破坏旧翻译和偏好 | 通过：实窗从 Settings 选择简体中文后，菜单栏只有系统 `Panta`、File、Edit、Window；`Panta` 菜单包含“关于 Panta / 设置… / 服务 / 隐藏 Panta / 隐藏其他 / 全部显示 / 退出 Panta”，点击“设置…”打开共享设置弹窗。实窗退出并重新启动后仍显示简体中文。`Qml.SettingsLanguageSwitch` 验证非默认 `zh-CN` 持久化并在新 `Settings` 服务实例初始化时恢复，且失败写入回滚 translator 并保留磁盘偏好；`QtAdapter.TranslationCatalog` 验证 QM 加载失败时保留已安装翻译。上述三个 CTest 定向用例通过，语言目录解析与格式检查通过，构建生成 QM 成功。全量 `cargo verify-changed` 中 Rust、格式、lint 与 90/91 native/QML 测试通过；唯一失败的 `QtAdapter.UserSettingsStore` 因沙箱拒绝写入 `~/Library/Preferences`。静态引用审计删除两份目录中无消费者的 10 个 `IconAction*Output` context 和已移除顶级菜单的 `ApplicationMenu.panta`；保留跨 context 同文案（Qt 按 context + source 查找），中英文共有 447 项一致，中文另有 20 条 Cocoa 专用菜单译文 |

## 风险与回退

启动恢复时若 QM 已缺失，不能让持久化 locale 与当前实际语言分离；加载失败应保留英文默认并给出诊断，不能覆盖已知有效的设置值。

## 决策与工作记录

- 2026-09-29：登记为 086 翻译后端的独立 UI 消费任务。语言目录和切换规则来自 Rust/`Settings`，QML 不复制 locale 列表。
- 2026-10-22：选择在 Settings 对话框内暂存候选语言；仅 OK 时提交，以保持 Cancel 语义。失败时不关闭对话框，并展示后端诊断。实现以原型确认结果为准。
- 2026-10-22：Qt macOS 原生菜单会依赖 `QCocoaMenuItem` 翻译识别菜单角色；应用中文目录补充角色关键词与 `MAC_APPLICATION_MENU` 标准标题，并由 115 移除重复的自定义应用名顶级菜单，将角色候选合入系统菜单。真实窗口通过中文切换及 Settings 路由验收。
- 2026-10-22：核对中英文目录：清理后共有 447 项 context / ID / source 对齐；移除两边均无调用方的 10 个旧 `IconAction*Output` context。相同 source 若属于不同 Qt context 必须分别保留；英文基线显式保留 `tr == src`，供当前 `QTranslator::translate()` 直接查询契约使用。评审发现 Cocoa 精确识别 `Copy` 角色要求两处译文一致，已统一为“复制”并添加交叉 context 回归断言。

## 完成摘要

语言设置页、共享 Settings 路由、偏好持久化与启动恢复、失败回滚和 macOS 菜单角色本地化已实现并通过自动回归；macOS 中文菜单、Settings 路由及重启恢复通过真实窗口验收。HTML 原型的主要布局与交互由维护者确认；当前环境的浏览器策略拒绝 `file://`，未能在内置浏览器完成渲染检查。
