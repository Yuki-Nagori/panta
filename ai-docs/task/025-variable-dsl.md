# 025 — 变量 DSL 解析、求值与存储

- 状态：done
- 阶段：应用平台扩展
- 依赖：[024](024-runtime-context.md)、[034](034-rust-panta-artifact-parser.md)
- 优先级：P1
- 负责人：Codex
- 创建 / 更新：2026-09-16 / 2026-10-20

## 目标与背景

基于 034 的 Rust `.pa` 解析内核实现变量域，将人可读的源文件转换为运行时快照，并支持语义往返存储和定位诊断。语言和主题域复用同一文件头、词法、版本与诊断边界；各域的 schema 由 022、030 扩展。已实现独立求值器、工程运行时原子提交、`variables.pa` 持久化及 C++/ViewModel 投影，并完成边界验证与评审。

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

2026-10-20 评审修复：将 ViewModel 变量加载/保存替换为 Rust 有界后台读写请求。GUI 仅发起请求并通过 Qt 完成通知投影；Rust 保留同步服务供 CLI/测试，后台沿用 RuntimeTaskScope 的 generation/revision 校验及写入冲突准入。加载迟到不覆盖新工程/新快照，保存失败保留旧文件与快照；销毁 ViewModel 不在 GUI 线程等待工作完成。当前没有变量编辑器或 QML 调用方，不新增编辑器 UI。

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
| 2026-10-20 | [PR #69 CI](https://github.com/Yuki-Nagori/panta/actions/runs/39386991774)，实现提交 `971c33a` | 15 个 job 全部成功，三平台构建、Rust 测试及 native/QML CTest 通过（macOS 86/86、Linux 85/85、Windows 83/83）。后台变量请求与 ViewModel 行为回归通过；Unix FIFO 专属测试不在 Windows 执行。 |
| 2026-10-20 | 整合后的 `cargo test-scope all`；定向 Miri；Cocoa 生命周期回归 | core 139 项单元测试、FFI 33 项、包括 7 项变量 GUI 回归的 CTest 86/86 通过；固定 nightly 的变量模块 Miri 11 项通过、1 项 1 MiB 压力用例按既有策略忽略。沙箱内无法访问屏幕，沙箱外以 Cocoa 运行原生 surface 生命周期回归 3/3 通过；不将此称为真实 GPU 帧呈现验收。 |
| 2026-10-18 | `cargo test --locked -p panta-dsl-core`、`cargo test --locked -p panta-core` | 通过；覆盖值类型、算术、前向引用、环/类型/范围错误、上限、资源根检查及快照原子提交 |
| 2026-10-18 | `cargo verify-changed` | 六个受影响 Rust package 测试、81 项 native/QML CTest、变更范围格式/lint、完整 native clang-tidy 与依赖分层检查均通过 |
| 2026-10-18 | [PR #66 GitHub Actions](https://github.com/Yuki-Nagori/panta/actions/runs/39130225241) | 最终 PR 提交 `14a5434` 的 Ubuntu、macOS、Windows 构建及全部格式、lint、依赖审计检查通过 |
| 2026-10-18 | `cargo fmt --all --check`、`git diff --check` | 通过 |
| 2026-10-18 | `cargo test --locked -p panta-dsl-core evaluation::tests::rejects_excessive_operators_before_building_a_deep_tree` | 通过；确认 128 运算符边界含一元及二元运算符 |
| 2026-10-18 | `cargo verify-changed` 内受影响 Rust package 测试 | 通过；新增规范化输出超限拒绝与旧工程 generation 请求拒绝回归 |
| 2026-10-18 | `ProjectViewModelTest.VariableDocumentsPreserveDiagnosticsAndRejectStaleUpdates`、`ProjectViewModelTest.OldVariableRequestCannotWriteIntoAnotherProject` | 通过；Qt ViewModel 验证读写、诊断位置、旧 revision 与旧工程请求均不能覆盖文件或快照 |
| 2026-10-18 | 独立只读评审（gpt-6.1-sol medium，完整 `git diff HEAD`） | 首轮提出规范化大小与跨工程旧请求两项风险，均已修复并加入回归；复审未发现可操作问题。评审未运行测试 |
| 2026-10-20 | `cargo test --locked -p panta-core -p panta-ffi` | 通过；core 138 单测、8 activation、4 fsm-build、40 project；FFI 33 单测及 doc-tests。新增后台成功/失败、拥有式跨线程结果与旧修订诊断回归 |
| 2026-10-20 | `cargo test --locked -p panta-core --lib variable_document` | 通过；12 项定向测试，后台阻塞时请求立即返回、文件替换失败/解析失败保留旧文件与快照、旧 revision/generation 不覆盖新快照或新 pending、销毁取消排队写入、容量拒绝不遗留 pending |
| 2026-10-20 | `cargo build --locked -p panta-launcher`；CMake 构建 `panta_bridge_project_view_model_test`；执行该目标 | 通过；23 项 ViewModel 行为测试，含 6 项变量测试。Unix FIFO 阻塞文件读取期间 GUI 事件继续执行并收到完成通知；覆盖保存冲突切换、跨工程迟到结果及 QObject 销毁 |
| 2026-10-20 | `cargo clippy --locked -p panta-core -p panta-ffi --all-targets -- -D warnings`、两个变更 C++ TU 的托管 clang-tidy、局部 rustfmt/clang-format、`git diff --check` | 通过；CXX opaque 请求与完成结果的 Box 消费签名有局部 `boxed_local` expect，说明必须移交唯一所有权。未运行 workspace 全量验收或真实 GPU 窗口验收 |
| 2026-10-20 | 独立评审 P2 修复；native 变量行为回归 | 改为等待线程启动成功后才准入 Rust 请求，完成回调捕获独立 QFuture，避免 QThread 启动失败无回执及 watcher 删除顺序风险。7 项变量回归通过，新增确定性启动失败注入覆盖 save/load 无副作用、可切换工程和正常重试；独立复核未发现其他可操作问题，评审者未运行测试 |

## 风险与回退

语言范围膨胀或解析阻塞 UI；限制 V1 文法与预算，拒绝失败输入并保留最后有效文本和快照。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-16：统一 DSL 源码后缀定为 `.pa`；034 提供公共解析内核，025 负责变量语义，语言与主题语义分别由 022、030 负责。
- 2026-10-20：变量 GUI 接口改为 `beginLoadVariableDocument` / `beginSaveVariableDocument`。Rust 复用有界 reads/writes executor，独占 request/receiver 可移交等待线程，finish 校验 scope 身份、generation/revision 后原子发布；Qt 先启动可报告失败的等待线程，再准入 Rust 请求；线程通过 gate 获取独占请求，完成经 `QPromise` / `QFutureWatcher::finished` 队列通知，无新增轮询器。等待线程启动失败时尚未提交 Rust 请求，返回结构化 I/O 错误，不遗留 pending 或写盘副作用。保存期间按现有写入 guard 拒绝冲突工程切换及变量提交，读取允许工程切换；销毁服务撤销 scope，销毁 ViewModel 自动断开完成连接，均不在 GUI 线程 join。取消为协作式，已进入文件系统调用无法强制中断。演示型 TaskManager 未新增真实作业接入；当前没有变量编辑器或 QML 消费者。

## 完成摘要

V1 变量 DSL 求值、原子快照提交、`variables.pa` 有界读取和原子写入、CXX/Qt ViewModel 结果投影均已实现。输入与规范化输出均受 1 MiB 上限约束，保存/加载同时校验工程 generation 和变量 revision；失败时保留先前文件与快照。受影响验证通过，独立复审无未解决发现；架构说明与索引已同步。
