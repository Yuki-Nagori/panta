# Panta `.pa` DSL 规则

更新日期：2026-09-17。状态：语法与工具边界已规划，解析器和格式化器由任务 034/035 实施。

适用于所有 Panta DSL 文本：变量、Theme、国际化字典和后续 Artifact 域。`.pa` 是面向人读写的源码；国际化构建时由 Rust 生成临时 Qt TS，再由锁定的预编译 QtTools 生成 QM。运行时不解析 `.pa`。

## 设计原则

- `.pa` 统一使用 UTF-8、LF 换行和两格缩进；禁止 Tab。文件末尾保留一个换行，格式化输出稳定且可复现。
- 语法借鉴 YAML 的层级和 `key: value` 可读性，但不是通用 YAML。不得出现 `---`/`...` 多文档、anchor、alias、tag、flow collection、隐式类型推断或任意 YAML 扩展。
- 值默认是不带引号的行尾标量。首个结构分隔符只解释当前声明的 `:`；值中的 `:`、`#` 和 URL 保持字面意义。注释使用 `//`，不使用 `#`。
- 关键字、kind、locale、变量名和 Theme key 使用 ASCII；变量/Theme 业务标识统一采用 kebab-case（短横线 `-`）。语言条目使用短 ID 作为键，`src` 保存 English 基线，允许点号、短横线和下划线以承接 Qt 现有条目。locale 按标准允许 `zh-CN` 或 `en_US`，并支持 `.pa` 内的 `cn` 简写；文法关键字永远不随 UI 语言切换。
- 语言 catalog 使用 `[Context]` 分组；Context 是 Qt 调用上下文，不是注释，注释使用条目内的 `comment`/`extra` 字段。每个 `.pa` 文件只对应一个目标 locale，同一 context 内 ID 唯一，`src` 是 Qt 的 source 节点。源文本兼容索引按 `src` 构建，不把人工 ID 当作用户可见文案；相同 context 的同长度 source 冲突必须报错。
- 解析、格式化和校验共享 034 的 Rust AST；不能用正则或第二套 YAML parser 在任务模块里旁路处理。

## 推荐形状

语言字典采用 TS 兼容的 catalog：每个 `.pa` 文件保存一个 `language`、context、条目 ID、English source 和该 locale 的 translation；Artifact 引擎聚合多个 locale 文件后生成独立 TS/QM。头部写 `language: zh_CN` 时使用 `tr:`，`cn` 是同义简写：

```text
version: 1
kind: language
language: zh_CN
sourcelanguage: en

[FileMenu]
open:
  src: Open
  tr: 打开
  comment: 菜单项：打开

files-selected:
  src: "%n file(s) selected"
  numerus: true
  tr: ["已选择 %n 个文件"]

"menu.file.exit":
  src: Exit
  tr: 退出

[Settings]
theme:
  src: Theme
  tr: 主题

// source-text 兼容索引仍按 src 长度匹配
```

变量域：

```text
version: 1
kind: variables

values:
  divisions: int = 24
  scale: real = 0.5
  mesh: resource = project:/assets/mesh.vtu
```

Theme 域沿用 `values:`，由 Theme schema 解释键名、类型、范围和默认值：

```text
version: 1
kind: theme

values:
  spacing-small: real = 8
  control-height: real = 32
```

`[Context]` 后的短 ID 是条目键，`src` 是 English 基线；双引号只在 ID 或值包含结构字符时使用。每个语言 `.pa` 文件只声明一个 `language`，其中 `tr:` 就是该文件 locale 的译文；Artifact 引擎负责聚合多个语言文件。`cn` 是 `.pa` 的简写，编译器归一化为 `zh-CN` 并在 TS 中输出 `zh_CN`。`st: unfinished`、`st: vanished`、`oldsrc`、`comment`、`extra`、`numerus` 和数组译文分别承接 Qt TS 的状态、旧 source、注释、复数元数据和 plural forms。每个 context 内 ID 必须唯一，每个 locale 最多一个 translation；变量/Theme 的 `type = expression` 是受限表达式，不把 `=` 后的数字或资源路径当成 YAML 的隐式 bool/number。多行文本、复杂集合、函数和脚本不在 V1；换行和保留行尾空白使用反斜杠转义。

Qt 运行时按 context + ID 或 source 精确查 QM，不做隐式全局字符串替换。需要兼容 source-text 查找时，在同一 context 中采用最长 `src` 优先：`ok ok` 覆盖 `ok`，同长度候选必须报错；单词边界和是否允许子串匹配由调用方明确选择，不能由 formatter 猜测。

## 规范化与校验

`panta-dslc check file.pa` 只读执行 UTF-8、缩进、版本、kind、键名、重复键、类型/表达式、locale、占位符和域 schema 校验；失败返回稳定 code、源字节范围和行列，不写文件。`panta-dslc format file.pa` 输出规范形式，`panta-dslc format --check file.pa` 在需要修改时返回非零，不自动改变源码。

格式化只改变空白、缩进和可安全排序的映射键，不改变 context、ID、src、translation、状态、表达式、声明顺序或注释语义。写回使用临时文件、fsync/替换和失败保留策略；不能把格式化失败当成校验通过。formatter 不补引号、不把 Unicode 转义成 `\\u`、不重排变量声明来改变诊断顺序。

035 评估直接使用 `yaml-format`/同类工具与 Rust AST formatter：只有能证明不引入 YAML 隐式语义、保留无引号值、source 条目、指令和注释时才可复用；否则实现小型确定性 formatter。无论选型如何，`pa check` 是唯一权威校验入口。

## 安全与演进

解析器限制字节数、token 数、嵌套深度、声明数和表达式操作量；禁止 import、文件访问、网络、shell、eval、任意 XML 或代码执行。未知必需版本和未知 kind 直接拒绝，不保留未登记兼容分支。新增域先增加 schema、夹具和格式化规则，再扩展公共 grammar。
