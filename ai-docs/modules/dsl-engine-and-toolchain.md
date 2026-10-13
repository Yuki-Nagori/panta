# Panta DSL 解析与 Artifact 引擎

[模块导航](README.md) · [变量 DSL](variable-dsl.md) · [国际化](internationalization.md) · [实施任务 034](../task/034-rust-panta-artifact-parser.md)

## 定位

这是应用平台的引擎基础设施，负责读取、校验、规范化和聚合 Panta DSL；它不是 CFD、OCCT、Netgen 或 VTK 的物理/几何引擎。Rust 负责确定性解析和工具链，C++ 只在应用服务边界消费已经校验的快照，QML 不接触解析器或源码文件。

022 的国际化需要 Qt 的 TS/QM 工具链，030 的主题需要 `.pa` 数据，025 的变量需要受限表达式和原子快照。三者共用一个解析内核，不能在各任务里各写一份 parser。运行时加载 QM 或已发布的主题/变量快照，不在 GUI 线程解析文本。

## 分层与技术选型

已实现 workspace 成员 `panta-dsl-core`（可复用 library）和 `panta-dslc`（单文件 CLI）。[110](../task/110-dsl-module-boundaries.md) 将核心内部整理为私有模块，根 `lib.rs` 保留既有公共 API；CLI 只编排纯函数和文件输出，不持有 Qt/QObject。

| 内部模块 | 职责 |
|---|---|
| `ast` | 数据文档及只读查询；serde 结构保持不变 |
| `diagnostic` | 共享诊断、显示文本及源码位置 |
| `syntax` | 唯一 Pest grammar、文档入口识别与语法诊断转换 |
| `parser` | Pest 节点转换、字段与声明结构检查 |
| `validation` | 文档种类、必需头部、locale、消息及占位符规则 |
| `format` | 保留声明顺序与注释的规范源码输出 |
| `ts` | Qt TS 输出及输出前的 locale / 占位符复核 |
| `fsm` | 独立 FSM schema、校验、格式化与 Rust 生成；复用 syntax / diagnostic |

`artifact` 模块已提供 `parse_artifacts`、不可变 `ArtifactSnapshot` 与 `ArtifactDocument`，保留逻辑来源 ID、版本、kind、locale 和规范化 payload。CLI 与构建脚本共用该核心；聚合采用固定排序与资源上限，任一来源失败时不产出部分快照。跨平台编码由固定快照 fixture 约束。

首选 `pest` 描述公共 `.pa` grammar：配置文件规模小，规则可审阅，`pest` 能稳定给出源码 span，适合错误定位和后续域扩展。暂不同时引入 `nom`；只有有可重复的吞吐或流式输入证据时才评估迁移，并保留同一 AST/诊断契约。`quick-xml` 负责从规范化翻译模型生成 Qt TS XML，必要时校验生成结果；不要求开发者阅读或编辑 XML。`serde` 映射领域 DTO 和机器可读诊断；依赖版本、许可证和 MSRV 进入 Cargo.lock。

## `.pa` 到 TS/QM 的构建流

`.pa` 是开发者直接编写的 UTF-8 文本字典/变量/主题源文件。首期语法采用 YAML 风格的冒号、`[Context]` 区块和两格缩进，但不实现通用 YAML；语言域用短 ID 作为条目键，以 `src` 保存 English 基线，再写 `tr`、状态和注释，变量/Theme 的类型表达式保留 `=`：

```text
version: 1
kind: language
language: zh_CN
sourcelanguage: en

[FileMenu]
open:
  src: Open
  tr: 打开
  st: finished

open-file:
  src: Open File
  oldsrc: Open
  tr: 打开
  st: unfinished
```

构建流程为：读取 `.pa` → pest 解析与领域校验 → Artifact 聚合器按 `kind` 生成语言/Theme/变量快照 → 按目标 locale 由 quick-xml 写出构建树临时 TS → 调用锁定的 Qt Linguist 预编译 `lrelease` → 生成并嵌入 QM。QM 是唯一的运行期翻译输入；应用启动和语言切换不解析 `.pa` 或 XML。`panta-dslc` 不自行下载、编译或寻找系统 Qt，`lrelease` 路径由 CMake 供给并记录版本。

构建入口已落地（034）：launcher build.rs 以 build-dependency 复用 `panta-dsl-core` 聚合 `resources/i18n/*.pa`，生成每个 locale 的临时 TS，并通过同文件系统的 staging/backup 目录发布完整文件集合；解析、生成或暂存失败时保留上一份有效目录。`PANTA_I18N_TS_DIR` 交给 `native/i18n`；该 CMake 模块从 Qt 供给（qttools 归档，`qt-provision.cmake` 锁定）解析 `lrelease` 编译 QM，以 qrc 列表嵌入 `:/i18n/<catalog>_<locale>.qm`，并由 ctest `I18n.CompiledQmLoads` 用 QTranslator 验证端到端加载。presets 纯净 configure 缺 TS 时立即失败并提示先 `cargo build`（与 FFI 输入同边界）。

临时 TS 和 QM 不作为源码提交；解析或 TS 准备失败时不覆盖上一份有效 TS 集合。限制为单文档 1 MiB、单次聚合 16 MiB、256 个来源、每文档 16,384 条声明及聚合 65,536 条声明。grammar 不支持嵌套结构、脚本或文件访问，因此没有独立的递归深度/token 预算。parser 拒绝未知必需版本、重复 context/ID、非法 locale 和占位符不一致。

catalog 的 source-text 兼容索引按 `src` 长度降序构建，`ok ok` 优先于 `ok`，重复 source 在校验阶段失败；不同且等长的 source 可以共存，QML/C++ 不走这个索引而使用 context + ID 或完整 source 精确查找。

## 诊断、快照与 FFI

解析结果由不可变 `ArtifactSnapshot` 保存来源 ID、版本、kind、locale 和规范化 payload；快照只能通过校验入口创建，再以固定 JSON 编码输出。来源 ID 在本次聚合中必须非空且唯一；文档按 kind、来源 ID 排序，不合并不同文档。语言 TS 以 catalog（缺省 `panta`）和规范化 locale 命名；任意两个输出名在大小写不敏感文件系统上冲突时拒绝整批生成。诊断带稳定 code、来源 ID、UTF-8 字节起止位置及行列；Pest 报告的零宽位置保留为零宽范围，语义诊断定位到对应声明或字段，缺失项定位到文件末尾。CLI 在读取前限制单文件字节数；launcher 在读取前限制来源数、单文件和聚合字节数，核心也会复核聚合来源数、字节数和声明数；GUI 可按 code 翻译，不按诊断文本分支。

Rust library 是唯一 parser/validator；C++/QML 不复制 grammar，也不把 AST 指针跨 FFI。未来 CXX 接口只传递已验证 DTO、诊断和 QM 路径/资源 ID；快照发布采用一次提交，失败保留旧状态。CLI 与 UI 使用相同核心库，避免“命令行能读、应用不能读”的分叉。

## 验证与演进

测试覆盖 pest 正反文法、源码位置、kind 分派、确定性快照、TS 生成、占位符/context 一致性、聚合资源上限、输出名冲突和完整目录替换。用固定夹具验证 CLI 在任意 cwd 工作、输出路径不依赖本地环境；根目录 `cargo format --check` 还会调用共享 CLI 检查 `crates/` 与 `resources/` 中的 `.pa` 源文件，并跳过刻意非规范的 fixture。用锁定的 QtTools 做 QM 编译冒烟。任务 032 当前为 deferred，因此其覆盖率目标不阻塞 DSL 工具链任务；行为和失败契约测试仍由各任务验收。

V1 只支持声明式 `.pa` 数据和由编译器生成的 Qt TS 载荷，不提供 eval、脚本、import、网络、shell、字节码或任意 XML 注入。需要新 kind 时先增加 schema、诊断和夹具，再扩展 parser；不在公共 parser 里加入业务字段猜测或隐式兼容分支。

## FSM 编译期扩展

`kind: fsm` 的方案与实施门槛见 [FSM 与 Rust 状态机](fsm.md) 和 [073](../task/073-fsm-dsl-and-import-state-machine.md)。已由 073 实施：`panta-dsl-core::fsm` 提供独立 schema / 校验 / 格式化 / 确定性生成，`panta-dslc check / format` 按头部 kind 分发；它不是运行期 Artifact 快照或 Qt TS/QM 输入，而是由领域 crate 的 build.rs 调用本公共内核，生成 `OUT_DIR` 中的 Rust 状态、事件种类、guard 与转移元数据（首个消费者为 panta-core 的 `fsm/open-saved-stl.pa`）。

FSM 沿用 `.pa` 的冒号语法、诊断和 formatter，不在 `panta-import` 另建 parser。领域 Rust 实现 guard、payload、动作及失败处理，公共 DSL 内核不依赖领域 crate。导入准备归 import，工程提交与任务生命周期归 core；运行期不解释 FSM 文本，不从 `.pa` 生成业务副作用。
