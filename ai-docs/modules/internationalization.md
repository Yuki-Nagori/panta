# 国际化与语言字典（规划）

[模块导航](README.md) · [实施任务 022](../task/022-ui-internationalization.md)

## 目标与选择

UI 源文案统一使用 English，默认语言为英文；用户通过语言设置选择翻译。项目约定采用简洁的 `.pa` 字典源码作为唯一权威数据，Artifact 引擎按 `kind` 聚合成语言、Theme 或变量产物。Rust 编译器在构建目录自动生成临时 Qt TS XML，再由 QtTools 的 `lrelease` 生成 QM。仓库不直接维护或要求用户编写 `*.ts`，首批验证英文与简体中文；代码标识、工程数据和 DSL 关键字不随语言变化。

QML/C++ 使用 context + ID 或 source 查找；`.pa` 保存条目 ID、English `src` 与各 locale 文本，编译器负责生成 Qt TS 所需的 context/source/translation 节点。`[Context]` 是 Qt 的调用上下文分组，用来区分同一文案在不同界面语境下的含义，不是注释；条目内的 `comment` 才是给翻译人员看的注释。每个语言文件只维护一个目标 locale，构建器再聚合多个文件。整句翻译并传入占位参数，不拼接已翻译片段；处理复数和翻译说明。源文案修改时重新校验 ID、src、占位符和失效条目。机制依据：[Qt 翻译源代码指南](https://doc.qt.io/qt-6/i18n-source-translation.html)、[QTranslator](https://doc.qt.io/qt-6/qtranslator.html)（查阅 2026-09-16，页面 Qt 6.11；实施核对锁定版本）。

## 数据流与生命周期

语言设置 → C++ 语言服务加载 `.pa` 编译得到的 QM → 安装 `QTranslator` → 刷新 QML 翻译绑定与 C++ 展示属性。运行时只查 QM，不解析 `.pa` 或 TS XML。使用 [QQmlEngine::retranslate](https://doc.qt.io/qt-6/qqmlengine.html#retranslate) 更新翻译绑定；C++ 缓存的字符串还需明确 NOTIFY/重新计算，不假定自动更新。语言切换无需销毁引擎或依赖热重载。

未设置语言时使用英文；明确支持的区域语言按“完整 locale → 基础语言 → 英文”解析。缺少条目回退英文；用户主动选择的字典若损坏或加载失败，保留上一有效语言并显示错误。切换成功后才保存偏好。数字、日期显示可使用 locale；文件存储、DSL 数字字面量和协议编码保持语言无关。

服务错误保存稳定 code 与参数，展示层负责翻译，不以译文作为分支条件；现有错误字符串在 022 中梳理，未来跨语言服务按 008 的结构化错误契约接入。用户输入的名称、文件名及外部工具原始诊断不自动翻译。

## 构建与边界

规划在 `resources/i18n/panta-en.pa`、`resources/i18n/panta-cn.pa` 等文件中按语言保存字典。Rust 编译器使用 034 的共享 parser 校验并聚合 context/ID 后，在构建树写出对应临时 `panta_en.ts`、`panta_zh_CN.ts`，调用锁定的 Qt Linguist 预编译 `lrelease` 生成 QM，再嵌入只读资源；临时 TS 和 QM 不作为源码提交。构建入口由 CMake 管理，Cargo 继续负责统一调度；QtTools 缺失时按预编译依赖规范补供给，不源码构建 Qt。`.pa` 中可用 `cn` 作为 `zh-CN` 的简写。

## `.pa` 字典语法约定

`.pa`（Panta Artifact）是 UTF-8 文本，不是 XML 容器；`.pt` 不采用，以免与 Portuguese/locale 语义混淆。语言域按目标语言拆分源文件，每个文件只声明一个 `language` 和对应的 `tr`；Artifact 引擎在构建时聚合这些文件并生成各自的 TS/QM。语法采用 YAML 风格的区块和冒号，不需要引号：

```text
version: 1
kind: language
language: zh_CN
sourcelanguage: en

[FileMenu]
open:
  src: Open
  tr: 打开

open-file:
  src: Open File
  oldsrc: Open
  st: unfinished
```

编译器直接以 `src` 生成 TS 的 source 节点，并按 locale 选择 `tr`；QML/C++ 使用 context + ID 或 source 查找，不需要把 XML 结构暴露给开发者。locale 采用标准写法，`.pa` 允许用 `cn` 简写并归一化为 `zh-CN`/`zh_CN`。`src`/`tr` 为 UTF-8 标量，行尾空白会被规范化，需要保留前后空格时使用反斜杠转义。`%1`、`%n` 等占位符必须在 `src` 与每个已完成 `tr` 间一致。缺失或 `st: unfinished` 的译文由 QM 回退 source，重复 ID、重复 locale、占位符不一致、非法 locale 或损坏文件拒绝整个字典。

`.pa` 也用于主题和变量，由 `kind theme`、`kind variables` 区分域；所有域共用 025 的 lexer、parser、版本和诊断格式。国际化域由 Rust 生成 TS XML 以使用 Qt 的成熟工具和 QM 查找，不把 XML 解析放入运行时。

同一 context 的 source-text 兼容查找采用最长 `src` 匹配：`ok ok` 先于 `ok`，同长度候选直接报错；正常 QML/C++ 路径始终用 context + ID 或完整 source 精确查找，避免在任意用户文本中替换短词。

022 验证界面、占位参数、缺项回退、加载失败和语言偏好；完整语言覆盖、RTL 适配与翻译协作平台不包含在首批交付。新增 UI 文案遵循英文源文本规则，注释和开发文档无需改为英文。
