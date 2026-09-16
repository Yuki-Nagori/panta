# Panta `.pa` DSL 规则

更新日期：2026-09-17。状态：语法与工具边界已规划，解析器和格式化器由任务 034/035 实施。

适用于所有 Panta DSL 文本：变量、Theme、国际化字典和后续 Artifact 域。`.pa` 是面向人读写的源码；国际化构建时由 Rust 生成临时 Qt TS，再由锁定的预编译 QtTools 生成 QM。运行时不解析 `.pa`。

## 设计原则

- `.pa` 统一使用 UTF-8、LF 换行和两格缩进；禁止 Tab。文件末尾保留一个换行，格式化输出稳定且可复现。
- 语法借鉴 YAML 的层级和 `key: value` 可读性，但不是通用 YAML。不得出现 `---`/`...` 多文档、anchor、alias、tag、flow collection、隐式类型推断或任意 YAML 扩展。
- 值默认是不带引号的行尾标量。首个结构分隔符只解释当前声明的 `:`；值中的 `:`、`#` 和 URL 保持字面意义。注释使用 `//`，不使用 `#`。
- 关键字、kind、locale、变量名和 Theme key 使用 ASCII；业务标识统一采用 kebab-case（短横线 `-`），禁止用下划线拼接，例如 `spacing-small`。语言条目的 source 是用户可见 English 文案，可以是带空格和标点的 Unicode 文本。locale 仍按标准允许 `zh-CN` 或 `en_US`，并支持 `.pa` 内的 `cn` 简写；文法关键字永远不随 UI 语言切换。
- 语言 catalog 直接以 English source 文案作为条目键（它就是 Qt 的 msgid），下面挂各 locale 的 translation；不要求人工维护第二套业务 key。相同 source 只能出现一次，同一查找上下文不得产生相同长度冲突。
- 解析、格式化和校验共享 034 的 Rust AST；不能用正则或第二套 YAML parser 在任务模块里旁路处理。

## 推荐形状

语言字典是全局 catalog：一个文件保存所有 English source 和多个 locale 的 translation。构建器再按 locale 生成独立 TS/QM：

```text
version: 1
kind: language
catalog: panta-ui

Advance revision:
  translation cn: 推进修订

ok:
  translation cn: 好的

ok ok:
  translation cn: 好好的
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

源文本行本身就是语言条目键，下面的 `translation <locale>` 是该 English 文案的译文；每个 source 必须唯一，每个 locale 最多一个 translation。`cn` 是 `.pa` 的简写，编译器归一化为 `zh-CN` 并在 TS 中输出 `zh_CN`。`@unfinished <locale>` 标记未完成翻译，`@source-note` 和 `@translation-note <locale>` 分别记录源文本与译文注释。变量/Theme 的 `type = expression` 是受限表达式，不把 `=` 后的数字或资源路径当成 YAML 的隐式 bool/number。多行文本、复杂集合、函数和脚本不在 V1；换行和保留行尾空白使用反斜杠转义。

Qt 运行时按 English source 精确查 QM，不做隐式全局字符串替换。需要兼容 source-text 查找时，在同一 context 中采用最长 source 优先：`ok ok` 覆盖 `ok`，同长度候选必须报错；单词边界和是否允许子串匹配由调用方明确选择，不能由 formatter 猜测。

## 规范化与校验

`panta-dslc check file.pa` 只读执行 UTF-8、缩进、版本、kind、键名、重复键、类型/表达式、locale、占位符和域 schema 校验；失败返回稳定 code、源字节范围和行列，不写文件。`panta-dslc format file.pa` 输出规范形式，`panta-dslc format --check file.pa` 在需要修改时返回非零，不自动改变源码。

格式化只改变空白、缩进和可安全排序的映射键，不改变 source 文案、translation、表达式、声明顺序或注释语义。写回使用临时文件、fsync/替换和失败保留策略；不能把格式化失败当成校验通过。formatter 不补引号、不把 Unicode 转义成 `\\u`、不重排变量声明来改变诊断顺序。

035 评估直接使用 `yaml-format`/同类工具与 Rust AST formatter：只有能证明不引入 YAML 隐式语义、保留无引号值、source 条目、指令和注释时才可复用；否则实现小型确定性 formatter。无论选型如何，`pa check` 是唯一权威校验入口。

## 安全与演进

解析器限制字节数、token 数、嵌套深度、声明数和表达式操作量；禁止 import、文件访问、网络、shell、eval、任意 XML 或代码执行。未知必需版本和未知 kind 直接拒绝，不保留未登记兼容分支。新增域先增加 schema、夹具和格式化规则，再扩展公共 grammar。
