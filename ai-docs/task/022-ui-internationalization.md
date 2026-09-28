# 022 — UI 英文源文案与语言字典

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)、[034](034-rust-panta-artifact-parser.md)、[086](086-qt-platform-adapter.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-29

## 目标与背景

统一现有 UI 的英文源文案，以 Panta DSL（`.pa`）字典支持英文与简体中文切换。`.pa` 是唯一权威源码；Qt `.qm` 仅是构建期生成的运行时产物。Rust 拥有 locale 选择规则；C++/Qt 只负责把 locale 对应到 QM resource 并管理 QTranslator。设置页和偏好持久化分别由 [087](087-language-settings-ui.md) 与 [048](048-settings-service-and-qt-adapter.md) 承接。

## 必读

- [模块设计](../modules/internationalization.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [qt](../standards/qt.md)
- [qml](../standards/qml.md)
- [cmake](../standards/cmake.md)

## 范围与非目标

包含现有 QML/C++ 展示文案、`.pa` 字典 schema、TS/QM 构建转换、语言运行时切换/刷新及缺项回退；不包含设置页面与偏好持久化（087/048）、完整多语言翻译、RTL 适配或 DSL 关键字本地化。

## 前置条件与待决策

复用 034 的 `.pa` 解析、Artifact 聚合和诊断边界；语言文件使用 `kind language` 的受限声明，不允许脚本、import、网络或 shell。构建阶段将校验后的 `.pa` 快照转换为临时 TS，再用预编译 QtTools 生成 Qt `.qm`，但不提交或维护 TS 源文件。运行时 locale 支持集和选择状态归 Rust；C++ 按既定 resource 命名契约加载 QM。QSettings 持久化由 048 提供，界面由 087 提供。

## 实施步骤

1. 盘点现有文案和 UI 错误，建立稳定的英文 message key、完整句子、占位参数和复数示例。
2. 在 `.pa` 语言域中定义英文基线与 `zh-CN` 覆盖，接入校验快照到 QM 的构建转换和资源打包。
3. 使用 086 的 Rust locale 服务与 Qt translator adapter 验证动态刷新、缺项回退和损坏字典失败路径，清理失效词条和原硬编码中文 UI 文案；持久化与 UI 由 048/087 验收。

## 预计改动

现存 qml/、native/bridge/、native/i18n/、`resources/i18n/` 与 Rust `.pa` 编译入口。翻译运行时由 086 接入；设置页由 087 承接。路径以实施时实际模块归属为准。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [ ] 英文默认启动，英文/简体中文切换后 QML 与 C++ 展示属性同步更新且不重建引擎。
- [ ] 缺项回退英文；损坏字典保留上一语言且不提交 locale。设置偏好写入和重启恢复由 048/087 验收。
- [ ] 参数与复数用例正确，提取后无遗漏的现有产品文案；用户名称和存储数字不随语言改写。
- [ ] 干净构建含翻译资源，任意 cwd 启动可读字典；构建工具版本与三平台供给可追溯。
- [ ] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。

## 验证计划与结果

现有 Cargo 构建、native CTest、all_qmllint，加真实窗口切换/重翻译和失败字典场景；语言偏好写入及重启恢复由 048/087 验收。实施时填写 cwd、工具链、依赖版本和退出结果；不预先声称测试目标已存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | 本次完成 `.pa` 后缀、语言域 schema 与构建产物边界设计 | 实现与功能验证未执行 |

## 风险与回退

翻译刷新遗漏 C++ 缓存字符串；失败保留旧 translator 与设置，可恢复到内置英文。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-16：语言、主题和变量统一使用 `.pa` 作为 Panta DSL 源文件后缀；`.pt` 不采用，以免与 Portuguese/locale 语义混淆。
- 2026-09-16：首期语言标识固定为 `en` 与 `zh-CN`；`.qm` 只作为构建产物，语言服务切换失败保持既有状态。
- 2026-09-17：语言源码按 locale 拆分为 `resources/i18n/panta-en.pa`、`panta-cn.pa` 等文件；每个文件只有一个 `language`，`[Context]` 表示 Qt 调用上下文，条目内 `comment` 才表示翻译注释。

## 完成摘要

未完成。`.pa` 到 QM 的构建链和 Rust/Qt 运行时切换已由 034/086 接入；现有展示文案、缺项回退、占位参数与真实窗口翻译验收仍归本任务。
