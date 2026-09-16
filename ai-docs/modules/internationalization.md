# 国际化与语言字典（规划）

[模块导航](README.md) · [实施任务 022](../task/022-ui-internationalization.md)

## 目标与选择

UI 源文案统一使用 English，默认语言为英文；用户通过语言设置选择翻译。项目约定采用简洁的 `.pa` 字典源码作为唯一权威数据，Artifact 引擎按 `kind` 聚合成语言、Theme 或变量产物。Rust 编译器在构建目录自动生成临时 Qt TS XML，再由 QtTools 的 `lrelease` 生成 QM。仓库不直接维护或要求用户编写 `*.ts`，首批验证英文与简体中文；代码标识、工程数据和 DSL 关键字不随语言变化。

QML 使用稳定 message key 的 `qsTrId()`，C++ 展示文本使用 `qtTrId()`；`.pa` 只保存 key、英文基线和各 locale 文本，编译器负责生成 Qt TS 所需的 context/source/translation 节点。相同文案在不同语境使用不同 key。整句翻译并传入占位参数，不拼接已翻译片段；处理复数和翻译说明。源文案修改时重新校验 key、占位符和失效条目。机制依据：[Qt 翻译源代码指南](https://doc.qt.io/qt-6/i18n-source-translation.html)、[QTranslator](https://doc.qt.io/qt-6/qtranslator.html)（查阅 2026-09-16，页面 Qt 6.11；实施核对锁定版本）。

## 数据流与生命周期

语言设置 → C++ 语言服务加载 `.pa` 编译得到的 QM → 安装 `QTranslator` → 刷新 QML 翻译绑定与 C++ 展示属性。运行时只查 QM，不解析 `.pa` 或 TS XML。使用 [QQmlEngine::retranslate](https://doc.qt.io/qt-6/qqmlengine.html#retranslate) 更新翻译绑定；C++ 缓存的字符串还需明确 NOTIFY/重新计算，不假定自动更新。语言切换无需销毁引擎或依赖热重载。

未设置语言时使用英文；明确支持的区域语言按“完整 locale → 基础语言 → 英文”解析。缺少条目回退英文；用户主动选择的字典若损坏或加载失败，保留上一有效语言并显示错误。切换成功后才保存偏好。数字、日期显示可使用 locale；文件存储、DSL 数字字面量和协议编码保持语言无关。

服务错误保存稳定 code 与参数，展示层负责翻译，不以译文作为分支条件；现有错误字符串在 022 中梳理，未来跨语言服务按 008 的结构化错误契约接入。用户输入的名称、文件名及外部工具原始诊断不自动翻译。

## 构建与边界

规划在 `resources/i18n/` 保存 `en.pa`、`zh-CN.pa` 等纯文本字典。Rust 编译器使用 025 的共享 lexer/parser 校验后，在构建树写出临时 `panta_en.ts`、`panta_zh_CN.ts`，调用锁定的 Qt Linguist 预编译 `lrelease` 生成 QM，再嵌入只读资源；临时 TS 和 QM 不作为源码提交。构建入口由 CMake 管理，Cargo 继续负责统一调度；QtTools 缺失时按预编译依赖规范补供给，不源码构建 Qt。

## `.pa` 字典语法约定

`.pa`（Panta Artifact）是 UTF-8 文本，不是 XML 容器；`.pt` 不采用，以免与 Portuguese/locale 语义混淆。每个 locale 一个文件，英文文件是完整基线，其他文件只覆盖相同 key。语法以行式声明为主，值从首个 `=` 到行尾，不需要引号：

```text
version 1
kind language
locale zh-CN
fallback en

message app.advance_revision = 推进修订
message app.revision_count = 修订计数：%1
```

英文基线使用相同 key 和 English 文本。编译器以 key 生成 TS 的 message id，并用稳定的 `Panta` context 和英文 source 填充 Qt 节点，因此 QML/C++ 不需要把 XML 结构暴露给开发者。message key 只允许 ASCII 小写、数字、点和下划线；翻译值为 UTF-8 字符串，行尾空白会被规范化，需要保留前后空格时使用反斜杠转义。`%1`、`%n` 等占位符必须与基线集合一致。缺失 key 按 fallback 链回退，重复 key、未知 key、占位符不一致、非法 locale 或损坏文件拒绝整个字典。

`.pa` 也用于主题和变量，由 `kind theme`、`kind variables` 区分域；所有域共用 025 的 lexer、parser、版本和诊断格式。国际化域由 Rust 生成 TS XML 以使用 Qt 的成熟工具和 QM 查找，不把 XML 解析放入运行时。

022 验证界面、占位参数、缺项回退、加载失败和语言偏好；完整语言覆盖、RTL 适配与翻译协作平台不包含在首批交付。新增 UI 文案遵循英文源文本规则，注释和开发文档无需改为英文。
