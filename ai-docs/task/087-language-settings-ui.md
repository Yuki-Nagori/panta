# 087 — 语言设置界面

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[022](022-ui-internationalization.md)、[029](029-qml-component-library.md)、[048](048-settings-service-and-qt-adapter.md)、[086](086-qt-platform-adapter.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-29 / 2026-09-29

## 目标与背景

在应用设置界面提供语言选择。语言目录、切换命令和当前 locale 由 Rust/`Settings` 后端提供；QML 只展示支持项并发出用户意图，不自行访问 QSettings、解析 `.pa` 或加载 QM。切换与翻译刷新机制由 086 接通，用户偏好持久化由 048 提供。

## 必读

- [国际化模块](../modules/internationalization.md)、[应用与存储架构](../architecture/application-and-storage.md)
- [分层规则](../standards/layering.md)、[Qt 规范](../standards/qt.md)、[QML 规范](../standards/qml.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

包含设置页的语言选择控件、对 `Settings` locale 属性/切换 API 的调用、失败反馈、设置服务持久化和启动恢复。

不包含 locale 目录定义、QM 资源编译、QTranslator 生命周期、Rust locale 校验、其他设置类别、RTL 支持或翻译覆盖扩充。

## 前置条件与待决策

- 022 的语言目录与 QM 构建验收完成。
- 048 提供 Rust 设置 schema、CXX DTO 和 Qt QSettings adapter；偏好写入只在 translator 加载成功后提交。
- 086 提供 QML `Settings` 类型、supported/current locale 和运行时切换 API。
- 029 提供的表单/选择器组件可满足布局与键盘操作；否则记录最小必要组件扩展。

## 实施步骤

1. 在设置页加入语言选择控件，以 Rust 返回的支持列表填充，不在 QML 维护第二份 locale 列表。
2. 用户切换时调用 `Settings` 后端；失败保留旧选项并展示稳定诊断，成功后保存偏好。
3. 启动时从设置服务恢复 locale；无效/缺失偏好回退英文并确保 QML/C++ 展示字符串一致。
4. 记录实际窗口验收、格式/构建验证和失败路径结果。

## 预计改动

预计涉及 `qml/` 设置页面与公共组件、`native/bridge/` settings view model，以及 Rust 设置服务/CXX DTO；路径以 048 的实际接口为准。

## 清理与兼容例外

删除 QML 或 C++ 中重复维护的 locale 列表和临时切换入口。无兼容例外。

## 验收标准

- [ ] 设置界面只展示 Rust 返回的可用语言，并调用 `Settings` 后端完成切换。
- [ ] 成功切换后 QML 与 C++ 展示文本更新，偏好持久化且重启恢复。
- [ ] QM 加载或设置写入失败时保留上一有效语言和偏好，并向用户提供可理解反馈。
- [ ] QML 不访问 QSettings、不解析 `.pa`、不操作 `QTranslator`；Rust 不依赖 Qt。
- [ ] 代码、配置、task 与索引一致，真实验证记录在本任务。

## 验证计划与结果

真实窗口操作切换语言、失败反馈和重启恢复；使用项目适用的 Cargo 构建/格式入口。依赖未完成前不声称这些入口已通过。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待实施 | UI 切换、失败保留与启动恢复 | 未执行 |

## 风险与回退

启动恢复时若 QM 已缺失，不能让持久化 locale 与当前实际语言分离；加载失败应保留英文默认并给出诊断，不能覆盖已知有效的设置值。

## 决策与工作记录

- 2026-09-29：登记为 086 翻译后端的独立 UI 消费任务。语言目录和切换规则来自 Rust/`Settings`，QML 不复制 locale 列表。

## 完成摘要

未完成，等待 022、048、086 依赖完成。
