# Panta `.pa` DSL 规则

更新日期：2026-09-16。状态：语法与工具边界已规划，解析器和格式化器由任务 034/035 实施。

适用于所有 Panta DSL 文本：变量、Theme、国际化字典和后续 Artifact 域。`.pa` 是面向人读写的源码；国际化构建时由 Rust 生成临时 Qt TS，再由锁定的预编译 QtTools 生成 QM。运行时不解析 `.pa`。

## 设计原则

- `.pa` 统一使用 UTF-8、LF 换行和两格缩进；禁止 Tab。文件末尾保留一个换行，格式化输出稳定且可复现。
- 语法借鉴 YAML 的层级和 `key: value` 可读性，但不是通用 YAML。不得出现 `---`/`...` 多文档、anchor、alias、tag、flow collection、隐式类型推断或任意 YAML 扩展。
- 值默认是不带引号的行尾标量。首个结构分隔符只解释当前声明的 `:`；值中的 `:`、`#` 和 URL 保持字面意义。注释使用 `//`，不使用 `#`。
- 关键字、kind、locale、message key、变量名和 Theme key 使用 ASCII；用户可见文本保持 Unicode。文法关键字永远不随 UI 语言切换。
- 解析、格式化和校验共享 034 的 Rust AST；不能用正则或第二套 YAML parser 在任务模块里旁路处理。

## 推荐形状

语言字典：

```text
version: 1
kind: language
locale: zh-CN
fallback: en

messages:
  app.advance_revision: 推进修订
  app.revision_count: 修订计数：%1
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
  spacing_small: real = 8
  control_height: real = 32
```

语言的 message 值从冒号后的第一个空格开始到行尾；变量/Theme 的 `type = expression` 是受限表达式，不把 `=` 后的数字或资源路径当成 YAML 的隐式 bool/number。多行文本、复杂集合、函数和脚本不在 V1；换行和保留行尾空白使用反斜杠转义。

## 规范化与校验

`panta-dslc check file.pa` 只读执行 UTF-8、缩进、版本、kind、键名、重复键、类型/表达式、locale、占位符和域 schema 校验；失败返回稳定 code、源字节范围和行列，不写文件。`panta-dslc format file.pa` 输出规范形式，`panta-dslc format --check file.pa` 在需要修改时返回非零，不自动改变源码。

格式化只改变空白、缩进和可安全排序的映射键，不改变值、表达式、声明顺序或注释语义。写回使用临时文件、fsync/替换和失败保留策略；不能把格式化失败当成校验通过。formatter 不补引号、不把 Unicode 转义成 `\\u`、不重排变量声明来改变诊断顺序。

035 评估直接使用 `yaml-format`/同类工具与 Rust AST formatter：只有能证明不引入 YAML 隐式语义、保留无引号值、注释和自有类型表达式时才可复用；否则实现小型确定性 formatter。无论选型如何，`pa check` 是唯一权威校验入口。

## 安全与演进

解析器限制字节数、token 数、嵌套深度、声明数和表达式操作量；禁止 import、文件访问、网络、shell、eval、任意 XML 或代码执行。未知必需版本和未知 kind 直接拒绝，不保留未登记兼容分支。新增域先增加 schema、夹具和格式化规则，再扩展公共 grammar。
