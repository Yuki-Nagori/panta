# 变量 DSL 与数据快照（规划）

[模块导航](README.md) · [实施任务 025](../task/025-variable-dsl.md) · [路径与运行时](paths-and-runtime.md)

## 用途与最小范围

DSL 用于可审阅地声明软件内的变量、参数及逻辑资源引用。先做声明式文本与受限表达式，不引入通用脚本执行；运行时消费校验后的变量快照，路径解析调用 023 的服务。它不替代 QML、不直接调用 OCCT/Netgen/VTK，也不改变外部求解器协议。

Panta DSL 的源码统一使用 `.pa` 后缀（Panta Artifact）；`.pt` 不采用，以免与 Portuguese/locale 语义混淆。`.pa` 面向人读写，Artifact 引擎按 `kind` 聚合为字典、Theme 或变量快照。国际化 `.pa` 由 Rust 编译器转换为构建树中的临时 TS XML，再交给 QtTools 生成 QM，源码目录不需要出现 `.ts`。语法借鉴 YAML 的层级和冒号，但不承诺通用 YAML 兼容；以下为待实现的变量域语法示意，完整关键字和 API 仍由 025 冻结：

```text
version: 1
kind: variables

values:
  divisions: int = 24
  scale: real = 0.5
  visible: bool = true
  label: string = Inlet
  mesh: resource = project:/assets/mesh.vtu
  half: real = divisions * scale
```

国际化字典复用同一文件头和诊断格式，使用全局 catalog 的简洁 `msgid` 声明；TS XML 仅是编译中间文件：

```text
version: 1
kind: language
catalog: panta-ui

msgid app.advance-revision:
  source: Advance revision
  translation zh-CN: 推进修订
```

语言字典统一放在 `resources/i18n/panta-ui.pa`，构建器再按 locale 生成 TS/QM；主题和变量也使用 `.pa` 文本，域由 `kind` 明确区分。顶层和区块字段使用 `key: value`，值默认取到行尾，不写引号；变量/Theme 的表达式使用 `type = expression`。业务 key 采用 kebab-case，locale 可用标准短横线或下划线形式。注释使用 `//`，反斜杠用于转义 `:`、`=`、换行和行尾空白。`.pa` 必须是 UTF-8，不能执行脚本、import、网络或 shell。语言域的 key、fallback、占位符和缺项规则见[国际化模块](internationalization.md)。

V1 包含 bool、int、有限 real、string、resource；类型显式声明，int 算术检查溢出，int 到 real 的提升规则固定并测试。字符串定义 UTF-8 与转义规则；关键字、标识与小数点均不随 UI 语言变化。标识符初期限定 ASCII，用户可读标签允许 Unicode。表达式仅允许引用、括号及类型允许的算术，不提供 eval、循环、函数定义、import、网络或 shell。

本期数值为无量纲，不隐含 mm/m 等物理单位；若正式接入材料/Study 参数，须先另立单位与量纲任务。不得把尚无量纲检查的 DSL 宣称为完整 CAE 参数系统。

## 解析、求值与提交

Rust 端作为唯一解析和求值实现：UTF-8 源文本 → 带源位置的 AST → schema/类型/引用检查 → 依赖图 → 确定性求值 → 不可变变量快照。允许前向引用，用拓扑排序求值，环依赖和未知变量报告位置与引用链。禁止重复声明、非有限数值、除零和不支持的版本。

输入字节数、token 数、嵌套深度、变量数和求值操作量设置可测试上限；达到上限返回错误，避免卡住 UI。解析在服务层完成；应用快照必须一次提交，失败保留旧版本，不能边解析边改变工程状态。

诊断使用 code、参数、源范围（字节偏移及面向用户的行列转换规则），由 UI 翻译摘要。FFI 仅传递诊断 DTO 与已验证值，不暴露 AST 内存或 Qt 对象。resource 值在求值时校验语法与根策略，实际资源存在性按消费操作判断，不使文本解析隐式读取任意文件。

## 存储与状态归属

DSL 文本保存用户声明；求值快照是可重建派生产物。V1 提供读写往返，序列化采用明确规范形式并允许丢失排版/注释，但不得丢失声明类型、表达式与语义；不宣称提供保留原格式的编辑器。内存解析成功不代表保存完成，写入须使用临时文件与提交策略，失败保留上一份有效文件。

变量按消费场景区分作用域：工程参数属于工程数据，主题变量属于应用级主题配置，局部输入框、选中项与窗口位置属于会话/UI 状态，不能塞入同一全局字典。主题和语言复用 DSL 解析器，但由独立服务校验/发布，不要求打开工程，不污染工程修订。主题键映射与默认值归属见[组件库与主题 DSL](qml-components-and-theme.md)，由任务 030 接入；语言字典由任务 022 接入。CLI 与 UI 如同时编辑，应基于预期 revision 拒绝覆盖过时快照。未支持版本直接拒绝，未来真实迁移需求另建任务，不预先保留多版本兼容分支。

## 延后能力

单位系统、跨文件模块、完整撤销/重做、脚本函数和字节码编译不在 025 中。运行时底座先成立，DSL 是其输入适配器；只有需求和性能证据成立后再扩大语言能力。
