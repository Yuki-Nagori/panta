# Panta DSL 解析与 Artifact 引擎（规划）

[模块导航](README.md) · [变量 DSL](variable-dsl.md) · [国际化](internationalization.md) · [实施任务 034](../task/034-rust-panta-artifact-parser.md)

## 定位

这是应用平台的引擎基础设施，负责读取、校验、规范化和聚合 Panta DSL；它不是 CFD、OCCT、Netgen 或 VTK 的物理/几何引擎。Rust 负责确定性解析和工具链，C++ 只在应用服务边界消费已经校验的快照，QML 不接触解析器或源码文件。

022 的国际化需要 Qt 的 TS/QM 工具链，030 的主题需要 `.pa` 数据，025 的变量需要受限表达式和原子快照。三者共用一个解析内核，不能在各任务里各写一份 parser。运行时加载 QM 或已发布的主题/变量快照，不在 GUI 线程解析文本。

## 分层与技术选型

规划 Rust workspace 成员 `panta-dsl-core`（可复用 library）和 `panta-dslc`（单文件 CLI）。核心层分为 `.pa` lexer/parser、领域 schema、Artifact 聚合器、TS XML 生成器、诊断和规范化快照；CLI 只编排这些纯函数并输出文件，不持有 Qt/QObject。

首选 `pest` 描述公共 `.pa` grammar：配置文件规模小，规则可审阅，`pest` 能稳定给出源码 span，适合错误定位和后续域扩展。暂不同时引入 `nom`；只有有可重复的吞吐或流式输入证据时才评估迁移，并保留同一 AST/诊断契约。`quick-xml` 负责从规范化翻译模型生成 Qt TS XML，必要时校验生成结果；不要求开发者阅读或编辑 XML。`serde` 映射领域 DTO 和机器可读诊断；依赖版本、许可证和 MSRV 进入 Cargo.lock。

## `.pa` 到 TS/QM 的构建流

`.pa` 是开发者直接编写的 UTF-8 文本字典/变量/主题源文件。首期语法采用 YAML 风格的冒号和两格缩进，但不实现通用 YAML；字典值不加引号，语言域将 `msgid`、source、translation 和指令作为一等元素，变量/Theme 的类型表达式保留 `=`：

```text
version: 1
kind: language
catalog: panta-ui

msgid app.advance-revision:
  source: Advance revision
  translation zh-CN: 推进修订
  @unfinished ja
```

构建流程为：读取 `.pa` → pest 解析与领域校验 → Artifact 聚合器按 `kind` 生成语言/Theme/变量快照 → 语言 catalog 按 locale 由 quick-xml 写出构建树临时 TS → 调用锁定的 Qt Linguist 预编译 `lrelease` → 生成并嵌入多个 QM。QM 是唯一的运行期翻译输入；应用启动和语言切换不解析 `.pa` 或 XML。`panta-dslc` 不自行下载、编译或寻找系统 Qt，`lrelease` 路径由 CMake 供给并记录版本。

临时 TS 和 QM 不作为源码提交；失败时不覆盖上一份有效 TS/QM。编译器限制输入字节、token、嵌套深度和 message/变量数，拒绝未知必需版本、重复 key、非法 locale、占位符不一致和任意脚本/文件访问。

catalog 的 source-text 兼容索引按 source 长度降序构建，`ok ok` 优先于 `ok`，同长度冲突在校验阶段失败；QML/C++ 不走这个索引而使用 `msgid` 精确查找。

## 诊断、快照与 FFI

解析结果是不可变 ArtifactSnapshot：源文档 ID、版本、kind、locale、规范化 payload、源位置和诊断。诊断包含稳定 code、参数、源文件、字节范围及行列转换；UI 只翻译 code，不按错误文本分支。输入字节、token、嵌套深度、声明数和生成节点数有可测试上限。

Rust library 是唯一 parser/validator；C++/QML 不复制 grammar，也不把 AST 指针跨 FFI。未来 CXX 接口只传递已验证 DTO、诊断和 QM 路径/资源 ID；快照发布采用一次提交，失败保留旧状态。CLI 与 UI 使用相同核心库，避免“命令行能读、应用不能读”的分叉。

## 验证与演进

测试覆盖 pest 正反文法、源码 span、领域 kind 分派、TS XML 生成/校验、占位符/context 一致性、输入资源上限和失败保留。用固定夹具验证 CLI 在任意 cwd 工作、输出路径不依赖本地环境；用锁定的 QtTools 只做 QM 编译冒烟。任务 032 对自有 Rust 可执行代码施加 line/branch 100% 门禁，生成 XML 和第三方解析代码按排除清单记录。

V1 只支持声明式 `.pa` 数据和由编译器生成的 Qt TS 载荷，不提供 eval、脚本、import、网络、shell、字节码或任意 XML 注入。需要新 kind 时先增加 schema、诊断和夹具，再扩展 parser；不在公共 parser 里加入业务字段猜测或隐式兼容分支。
