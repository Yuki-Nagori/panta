# 025 — 变量 DSL 解析、求值与存储

- 状态：done
- 阶段：应用平台扩展
- 依赖：[024](024-runtime-context.md)、[034](034-rust-panta-artifact-parser.md)
- 优先级：P1
- 负责人：Codex
- 创建 / 更新：2026-09-16 / 2026-10-18

## 目标与背景

基于 034 的 Rust `.pa` 解析内核实现变量域，将人可读的源文件转换为运行时快照，并支持语义往返存储和定位诊断。语言和主题域复用同一文件头、词法、版本与诊断边界；各域的 schema 由 022、030 扩展。当前已实现独立求值器、工程运行时原子提交、`variables.pa` 持久化及 C++/ViewModel 投影，正在完成边界验证与评审。

## 必读

- [模块设计](../modules/variable-dsl.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [rust](../standards/rust.md)
- [cxx](../standards/cxx.md)

## 范围与非目标

包含变量域的基础值类型、资源引用、受限算术、依赖求值、原子提交和文件保存；公共 `.pa` 文件头、YAML 风格缩进、无引号标量和诊断由 034 提供。不包含循环、脚本函数、跨文件 import、物理单位或字节码 VM。`kind language` 的 message 语义由 022 定义，`kind theme` 的键映射由 030 定义。

变量求值与工程会话解耦：025 交付通用值/诊断输出，022 和 030 通过 034 复用解析器处理语言与主题数据。语言 fallback、占位符校验、主题 schema、颜色校验和 UI 切换不扩入 025。

## 前置条件与待决策

024 的不可变快照契约已稳定，034 的公共解析器已支持变量声明并保留表达式文本。本任务冻结 V1 求值规则：支持 bool/int/real/string/resource 五种值；变量名使用 ASCII kebab-case；数字支持十进制 int 与有限 real；表达式支持变量引用、括号、一元正负号及 `+`、`-`、`*`、`/`。二元运算符两侧必须有空白，以区分减号和 kebab-case 名称。int 的加减乘使用 checked 运算并保持 int；任一操作数为 real 时先提升 int 为 real；除法统一产生 real，除数为零时报错；目标类型为 real 时允许 int 结果提升，反向不允许。bool、string、resource 仅接受同类型字面量，不参与运算。表达式不支持隐式字符串插值、循环、函数、单位、跨文件引用或 import。沿用 034 的 1 MiB 源码与 16,384 声明上限；求值另限制依赖深度和表达式语法深度各 128、单条表达式运算符数 128、单文档求值节点数 100,000，超限返回结构化诊断。解析器实现复用 `panta-dsl-core` 现有 Pest AST，不添加第三方依赖。

## 实施步骤

- [x] 编写最小语法规格、正反例和诊断 code，明确无量纲与规范化输出语义。
- [x] 基于 034 的 AST 实现变量类型/引用检查、环检测与有限求值，输出快照及源位置诊断。
- [x] 接入运行时原子提交；资源引用按捕获作用域校验根授权，提交失败保持旧快照。
- [x] 完成变量 `.pa` 文件读写与语义往返，写入失败保持旧文件。
- [x] 按 CXX 边界暴露已验证值和结构化诊断，由 C++ ViewModel 提供加载结果；工程 generation 或变量 revision 过期时拒绝覆盖。

## 预计改动

求值器位于 `panta-dsl-core`，工程作用域提交及固定 `variables.pa` 文件持久化位于 `panta-core`；CXX DTO 与 `ProjectViewModel` 只投影已验证值和结构化诊断，不保存 AST 到外部求解器协议。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] bool/int/real/string/resource、前向引用与表达式按规则求值，重复声明和未知变量报告准确源范围。
- [x] 循环依赖、类型错误、溢出、除零、NaN/Infinity、未知版本被拒绝；输入、依赖/语法深度、单条表达式运算符数及单文档求值量上限有效。
- [x] 解析/校验/写入失败均保留旧快照或旧文件；旧 generation 和 revision 不能覆盖新工程或新数据，规范化输出也受 1 MiB 限制。
- [x] 保存后重载语义一致，类型和表达式不丢失；DSL 解析与数值语义独立于 UI 语言目录。
- [x] 解析器可脱离工程会话返回值快照与诊断，工程提交由单独调用方负责，为 030 主题作用域复用提供边界。
- [x] 变量域消费 034 的 `.pa` 文件头、UTF-8 和诊断规则，语言/主题域可在不复制 lexer 的前提下扩展声明。
- [x] DSL 不能执行 shell/网络或绕过资源根限制，诊断以 code 与参数跨 FFI。
- [x] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。

## 验证计划与结果

解析与求值行为测试、边界输入、文件往返和注入写失败测试；由 ViewModel 行为测试验证结构化诊断的位置投影。当前没有变量文本编辑器，本任务不做屏幕上的编辑器标注验收。记录实际命令与结果，不预先声称测试目标已存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-10-18 | `cargo test --locked -p panta-dsl-core`、`cargo test --locked -p panta-core` | 通过；覆盖值类型、算术、前向引用、环/类型/范围错误、上限、资源根检查及快照原子提交 |
| 2026-10-18 | `cargo verify-changed` | 六个受影响 Rust package 测试、81 项 native/QML CTest、变更范围格式/lint、完整 native clang-tidy 与依赖分层检查均通过 |
| 2026-10-18 | `cargo fmt --all --check`、`git diff --check` | 通过 |
| 2026-10-18 | `cargo test --locked -p panta-dsl-core evaluation::tests::rejects_excessive_operators_before_building_a_deep_tree` | 通过；确认 128 运算符边界含一元及二元运算符 |
| 2026-10-18 | `cargo verify-changed` 内受影响 Rust package 测试 | 通过；新增规范化输出超限拒绝与旧工程 generation 请求拒绝回归 |
| 2026-10-18 | `ProjectViewModelTest.VariableDocumentsPreserveDiagnosticsAndRejectStaleUpdates`、`ProjectViewModelTest.OldVariableRequestCannotWriteIntoAnotherProject` | 通过；Qt ViewModel 验证读写、诊断位置、旧 revision 与旧工程请求均不能覆盖文件或快照 |
| 2026-10-18 | 独立只读评审（gpt-6.1-sol medium，完整 `git diff HEAD`） | 首轮提出规范化大小与跨工程旧请求两项风险，均已修复并加入回归；复审未发现可操作问题。评审未运行测试 |

## 风险与回退

语言范围膨胀或解析阻塞 UI；限制 V1 文法与预算，拒绝失败输入并保留最后有效文本和快照。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-16：统一 DSL 源码后缀定为 `.pa`；034 提供公共解析内核，025 负责变量语义，语言与主题语义分别由 022、030 负责。

## 完成摘要

V1 变量 DSL 求值、原子快照提交、`variables.pa` 有界读取和原子写入、CXX/Qt ViewModel 结果投影均已实现。输入与规范化输出均受 1 MiB 上限约束，保存/加载同时校验工程 generation 和变量 revision；失败时保留先前文件与快照。受影响验证通过，独立复审无未解决发现；架构说明与索引已同步。
