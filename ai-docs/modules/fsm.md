# `.pa` 有限状态机（FSM）与 Rust 状态机

[模块导航](README.md) · [DSL 工具链](dsl-engine-and-toolchain.md) · [PA 规范](../standards/pa.md) · [职责边界](../architecture/native-domain-boundaries.md) · [评估任务 072](../task/072-flow-state-machine-planning.md) · [实施任务 073](../task/073-fsm-dsl-and-import-state-machine.md) · [视口首个消费者 080](../task/080-qml-viewport-document-tabs.md)

更新 / 官方资料查阅日期：2026-09-26。本文是 FSM 的设计依据。V1 语法、校验、格式化与确定性生成已由 [073](../task/073-fsm-dsl-and-import-state-machine.md) 随 `panta-dsl-core::fsm` 落地，首个只读消费者为 panta-core 的 `fsm/open-saved-stl.pa` 资产激活（[080](../task/080-qml-viewport-document-tabs.md) 视口文档页签）；写入型导入事务与后续消费者仍为规划。

## 结论与当前基线

采用「编译期声明有限流程图，Rust 手写 guard、行为与失败处理」的方向。运行期使用领域内的 `enum + match`，先不引入状态机库或建立 `panta-fsm`。FSM 的价值是让多阶段、可取消、有迟到结果的业务流程可审阅、可校验；同步短路径不为统一形式而套状态机。

当前已有三项基础：

- [`panta-dsl-core`](../../crates/panta-dsl-core/src/grammar.pest) 与 `panta-dslc` 共用冒号语法、AST、诊断与格式化；kind 为 `language`、`variables`、`theme`，`fsm` 由 `panta-dsl-core::fsm` 独立 schema 入口承接，CLI `check` / `format` 按头部 kind 行分发。
- [`panta-import`](../../crates/panta-import/src/lib.rs) 已负责来源快照、STL 预览和导入准备；[`panta-core::project`](../../crates/panta-core/src/project/import.rs) 写资产与清单、维护修订，成功后替换当前网格。
- [`panta-core::task`](../../crates/panta-core/src/task.rs) 已有 `Running / Succeeded / Failed / Cancelled` 生命周期、取消请求和拉取式事件队列。当前执行体仍为模拟任务，不能把它当作已经接入 STEP 的异步事务。

因此，067 已完成的 STL 导入路径不追加 FSM 改造；首个 FSM 消费者改为任务 080 中按需读取已提交 STL 资产并重建视口网格的异步激活流程。它只读工程，不承担提交；未来 STEP 等写入型导入事务另行登记消费者并明确后端、提交和取消边界。

## 对原方案的调整

| 原方案 | 评估与决定 |
|---|---|
| `.pa` 只描述形状 | 保留。允许状态、事件、边、终态标记和 guard 名称引用；不允许表达式、动作、脚本或 I/O |
| Rust 是权威，`.pa` 只是摘要 | 改为职责明确的双层契约：`.pa` 是流程结构的唯一声明，Rust 是规则、数据和副作用的唯一实现。两者不一致应失败，不能选择一方静默覆盖 |
| `kind = "fsm"`、内联对象数组 | 与既有 `.pa` 不一致；沿用 `key: value` 和两格缩进，在公共 parser 内新增 schema，不引入 TOML / YAML 方言 |
| 首个 parser 放 `panta-import/src/parser` | 已有公共 DSL 内核，不再另写 parser。FSM schema、格式化和确定性生成放 `panta-dsl-core` 的独立模块，由领域 crate 的 build-dependency 使用 |
| 生成到 `src/fsm/generated` | 改为 `OUT_DIR`，源码只保留输入与手写实现，避免多配置构建改写同一源码树 |
| 导入事务与 task lifecycle 都放 import | 导入准备放 `panta-import`；工程提交和通用任务生命周期留在 `panta-core`，不形成反向依赖 |
| 只生成三元组和 guard 字符串常量 | 转移表必须保留每条边的 guard 关联；生成强类型 `Guard` 和 `Option<Guard>`，避免只声明名字却无法校验使用位置 |
| 先写 `state = next` 再调用 `on_entry` | 不采用。`commit()` 失败时不能已经处于 `Committed`，任意可失败 entry/exit hook 也不能充当通用事务保证 |
| 枚举全部转移即双向证明 | 只能验证有限结构一致性；guard、payload、写盘失败、事件顺序与竞态需要独立行为测试 |
| `enum + match` 已是性能上限 | 无测量不作此断言。当前选择依据是依赖少、逻辑可审阅、状态规模有限；状态机库也可能零成本生成匹配代码 |

## 权威与模块归属

| 内容 | 唯一拥有者 | 边界 |
|---|---|---|
| 状态 / 事件种类、初态、终态、允许的边、guard 名称 | 领域 crate 随代码提交的 `fsm/*.pa` | 不可在运行期编辑或热加载；不承载事件数据 |
| FSM grammar、schema、AST、诊断、formatter、Rust 元数据生成 | `panta-dsl-core`（扩展规划） | 不依赖 `panta-import` / `panta-core`、Qt 或重库，不生成业务动作 |
| 来源固定、格式分发、解析完成与取消 | `panta-import` / 数据领域 crate 的准备或读取服务 | 产出拥有明确所有权的准备结果；首个 073 消费者只读取已提交 STL 资产 |
| 工程身份、资产定位与只读激活请求相关性 | `panta-core::project` | 校验 session / generation 与 ImportRecord ID；只读激活不改 revision 或 dirty |
| 修订校验、资产发布与提交终态 | `panta-core::project` 的未来写入型导入事务 | 调用准备服务并协调工程存储；重试建立新事务 |
| TaskId、执行状态、进度、取消请求、终态事件 | 现有 `panta-core::task` | 与领域阶段关联，不能把准备完成直接当作工程提交成功 |
| 展示属性、加载动画、按钮启用、弹窗阶段 | C++ ViewModel / QML | 消费 Rust 快照与事件；UI 禁用按钮不能代替 Rust 校验 |

编译依赖维持 `panta-core → panta-import → panta-mesh / 未来 panta-geom`。各 FSM 消费者只在构建期依赖公共 DSL 内核；运行期不加载 FSM 源码。首个只读激活 flow 由 `panta-core` 持有请求身份和工程 session 上下文，解析复用 `panta-mesh`；未来的导入准备 flow 可留在 `panta-import`，跨工程提交的写入事务由 core 编排。不以一个公共 FSM 对象跨 crate 持有所有上下文。

通用任务状态机已存在，不在 import 内再造 `task_lifecycle`。073 首期复用任务身份、队列与展示契约，使 Task 成功表示只读网格快照就绪；旧模拟执行体不能独立覆盖取消、过期请求或资源释放结果。首个流程没有工程提交，不能套用写事务终态。对未来一次写入型导入，core 事务协调者唯一裁决提交 / 取消及最终结果，Task 状态只是结果投影；通用 worker 的取消标志、超时或 shutdown 不得独立覆盖事务结论。是否将该生命周期也描述成 `.pa`，以实际复杂度和重复维护成本决定，另记消费者任务后实施。

## FSM V1 语法草案

沿用冒号、两格缩进和 `//` 注释。以下为 **core 工程提交阶段的独立示意流程**，从已有准备结果开始；不代表完整导入，也不能交给当前 `panta-dslc` 验收：

```text
version: 1
kind: fsm
name: import-commit
initial: ReadyToCommit

states:
  ReadyToCommit: active
  Committing: active
  Committed: terminal
  Failed: terminal
  Cancelled: terminal

transitions:
  begin-commit:
    from: ReadyToCommit
    on: commit-requested
    to: Committing
    guard: revision-valid
  reject-preparation:
    from: ReadyToCommit
    on: fail
    to: Failed
  cancel-before-commit:
    from: ReadyToCommit
    on: cancel-acknowledged
    to: Cancelled
  finish-commit:
    from: Committing
    on: commit-succeeded
    to: Committed
  fail-commit:
    from: Committing
    on: commit-failed
    to: Failed
```

`active / terminal` 是 schema 枚举值，终态显式声明，避免把漏写出边误判为正常结束。转移 ID 只用于诊断 / 测试关联，每条边仍只允许 `from / on / to / guard`。`version` 是 FSM schema 版本，不是工程修订或可持久化状态枚举版本。

V1 的静态约束：

- 一个文件一个流程；顶层仅允许 `version / kind / name / initial / states / transitions`。状态名采用受限 ASCII UpperCamelCase，流程、事件、guard、转移 ID 使用 kebab-case；生成前拒绝 Rust 关键字、保留名称及转换后的命名冲突。事件种类从 `on` 的并集生成，guard 从引用的并集生成，源码 span 保留到诊断。
- `initial`、`from`、`to` 必须引用已声明状态；状态 / 转移 ID 唯一，未知字段、重复字段、空名称、未知版本和非法缩进均拒绝。
- 每个 `(from, on)` 至多一条边，V1 不支持用多个 guard 选择不同目标或隐含优先级。guard 是对当前一致快照的纯谓词，不做 I/O、不改上下文，也不投递事件；guard 为假表示拒绝本事件，原状态不变且不执行动作。需要业务失败终态时，Rust 编排显式投递已声明的失败事件。
- 初态合法，所有状态在忽略 guard 条件的图上可达；终态无出边，每个 active 状态有出边并存在到某个终态的结构路径。结构可达不证明 guard 可满足或流程一定终止，仍需 Rust 测试。
- 不支持层级 / 并行状态、历史状态、通配边、自动转移、定时表达式、action / entry / exit 字段。超时是 Rust 调度器投递的显式事件；事件 payload 与错误详情也是 Rust 类型。
- parser 保留现有输入资源上限，并为状态数、边数和生成体积设置可测试上限。formatter 保持语义、声明顺序和注释；CLI check、format 与 build script 共用同一 schema。

V1 扩展已由 073 实现：schema、定位诊断、正反夹具与 grammar 扩展随 `panta-dsl-core::fsm` 落地（`tests/flow.rs`），CLI 分发与既有 kind 回归由 `panta-dslc` 测试覆盖。

## 构建与生成契约

消费者布局如下；073 已随 panta-core 落地首个实例，后续消费者按同样布局归入各自领域 crate，不从 import 导出完整工程状态机：

```text
crates/panta-dsl-core/src/fsm/        # schema、校验与生成；不持有领域行为
crates/panta-core/
  fsm/open-saved-stl.pa             # 首个只读视口资源激活输入
  build.rs                            # 调用公共 DSL 内核（build-dependencies）
  src/fsm/mod.rs                     # 私有模块 include! OUT_DIR 生成文件
  src/fsm/open_saved_stl.rs          # 手写 Rust 行为
OUT_DIR/fsm/open_saved_stl.rs        # 构建输出；不提交
```

元数据生成器输出 `State / EventKind / Guard`、`INITIAL_STATE`、`ALL_STATES / ALL_EVENTS / TERMINAL_STATES` 和带边 ID 的 `TRANSITIONS`。转移元素语义为 `Transition { from, on, to, guard: Option<Guard> }`；guard 不能在生成时丢弃或降为无关联字符串表。

手写 `Event` 负责源快照、准备资产、错误等 payload，通过穷尽匹配映射到生成的 `EventKind`；手写 guard 求值对 `Guard` 穷尽匹配，不用默认真分支。生成的 tag、名字或枚举序号不能直接作为 CXX ABI / 工程持久化格式。测试夹具可构造状态上下文，生产接口不能公开任意 `set_state`。

Cargo 的 [build script 规则](https://doc.rust-lang.org/cargo/reference/build-scripts.html) 要求输出留在 `OUT_DIR`，构建依赖显式声明为 `build-dependencies`。本项目约定：生成器按稳定顺序输出，不嵌入时间戳或本机绝对路径；监听 `fsm/` 及输入文件，覆盖新增、删除和重命名的重建场景；先完整校验再写出当前生成集合。失败中止构建，不回退编译上一次成功结果。`OUT_DIR` 可能保留旧文件，因此只 include 本轮输入明确对应的输出，清理本生成器拥有的失效文件，不扫描并加载残留 `.rs`。

同一个消费者的输入集合还要检查跨文件唯一性：文件 stem 与 `name` 采用同一个受限 kebab-case 名称，生成模块名按唯一规则转换；重复流程名、转换后冲突和目标平台不区分大小写时的路径冲突均拒绝，不能靠生成顺序覆盖前一文件。显式列出的必需输入被删掉时必须构建失败；从目录发现的输入被删除时应同步移除其生成注册，不允许残留文件继续满足旧 include。

生成器只编译结构元数据，不生成执行引擎。手写匹配仍与声明存在结构重复，因此双向测试是必需成本；若长期出现明显漂移，再评估由声明生成纯转移决策，不能默默把它变成可执行业务 DSL。

## 写入事务、取消与异步结果（后续消费者）

本节的工程发布、提交回执与修订规则适用于未来写入型导入，不是 073 首个只读 STL 激活流程的验收范围。首期取消可以在解析器安全检查点生效；不可中断解析在完成边界校验 project generation、ImportRecord ID 和 attempt，过期快照应释放，不得发布到 UI。

073 / 080 首期只读资产激活的候选阶段为 `Idle → LoadingAsset → Parsing → Ready`，另有失败、取消与过期结果终态；它不发布工程变更。未来写入型导入仍可分为 `Idle → Snapshotting → Parsing → Prepared` 的准备流程和 core 提交流程，两者通过有所有权的结果组合，不引入层级 FSM 框架。来源快照、单位选择、工程身份与输入修订在写入任务提交时固定。

提交阶段遵守以下顺序：

1. Rust 串行处理同一事务的事件，先校验事务身份、当前状态、payload 与 guard。拒绝事件不改变状态、工程资产或修订，也不发布成功事件。
2. 在 `Committing` 阶段完成 staging、资产检查与工程清单发布。仅在存储层确认发布成功后接受内部 `commit-succeeded`、进入 `Committed`，然后对外报告成功。不要在 `Committed` 的 entry hook 内才执行持久化。
3. 提交前再次校验工程 generation / revision，并由 core 序列化最后校验和清单发布，避免「检查通过后工程已变化」的竞态。所有修改该工程的入口（包括保存、重命名和其他导入）都服从同一提交协调，不只是锁住一个状态机。准备阶段和长 native 调用在锁外进行，不能用长时间占用 GUI 线程或大锁解决问题。
4. 提交前失败 / 取消只丢弃本事务拥有的暂存产物，保留旧资产、清单和可见快照；失败与取消分别进入 `Failed` / `Cancelled` 并记录原因。一次文件原子替换不等于多文件事务，存储层还需明确提交点、孤立 staging 的清理和崩溃恢复方案。
5. 提交点之后不再把任务改报失败或取消。通知 / 日志失败不能回滚已提交工程；UI 可重新读取权威快照。持久化完成到内存发布之间若发生崩溃，以重开工程的恢复规则为准，FSM 本身不提供持久化执行日志。

存储回执必须区分「确认未发布」与「确认已发布」，后者带本事务和提交修订的凭据。示意图的 `commit-failed` 只代表确认未发布；不能把底层任意 I/O 错误都映射到该事件。若错误发生在发布边界、结果尚不确定，存储层先核对权威清单 / 提交凭据；核对期间保持 `Committing`、报告诊断并禁止同工程继续写入、自动重试或删除可能已被引用的资产。073 与业务任务冻结契约时必须证明目标平台能完成该核对；如果无法判定，先补充明确的恢复状态和恢复入口，再验收实现，不能用无限等待或虚假的 `Failed` 代替。这是存储恢复要求，不是让 FSM 自动重放业务动作。

取消请求与取消完成分开：`cancel_requested` 表示意图，只有工作单元停止发布结果、资源已安全移交或清理后才确认 `Cancelled`。不可中断的 OCCT / Netgen 阶段只记录请求，在安全检查点处理，不能承诺即时停止。开始提交前由 core 在同一串行决策中裁决取消与提交；本草案的 `Committing` 不接受取消，已越过该边界的请求返回明确的 too-late 结果。

事件信封由 Rust 持有 `(project_id, generation, task_id, attempt_id, input_revision)` 等相关性信息，具体类型由实施任务冻结。关闭 / 重开工程必须使准备结果与 UI 投递的 generation 失效；同一任务的重试不能接受上一 attempt 的结果。输入修订用于提交前校验，提交成功回执携带实际提交修订，不能因提交本身推进了 revision 就被通用“修订不匹配”过滤器丢弃。进程内每个事务的终态至多发布一次，迟到 / 重复完成和进度事件不会触发副作用或重启流程；过时 payload 的资源仍须释放。重试创建新事务或 attempt，不给终态添加隐含返回初态的边；当前方案不承诺崩溃后的 exactly-once 执行。

关闭与提交也由 core 串行裁决：准备阶段可停止接收结果并等待资源安全释放；进入 `Committing` 后，事务拥有者必须保留原工程上下文和资源直至确认结果，窗口关闭只能停止 UI 投递，不能销毁接收提交回执的拥有者。原事务收尾前禁止重新打开同一工程包并写入；应用正常退出先异步排空提交，不能把现有 GUI 侧析构中同步 join 模拟短任务的方式直接用于不可中断的 native 长调用。已完成存储发布的内部回执始终由原事务处理，即使原 UI generation 已失效；强制结束进程按存储崩溃恢复契约处理。

通用 Task 的 `Succeeded` 对完整导入意味着工程已提交，不只是 `Prepared`；进度是阶段信息，不因每个百分比变化增加一个业务状态。手写 FSM 核心收到图外事件返回 `IllegalTransition`；外层收件入口可以识别迟到 / 重复信封并丢弃或记录，二者测试分开。

## 校验与验收边界

双向结构校验遍历完整的 `ALL_STATES × ALL_EVENTS`，将生成表与**实际手写决策结果**比较：声明边在合法 payload 且 guard 为真时得到规定目标；未声明边必须拒绝。不能写一个查询 `TRANSITIONS` 的“实现”再拿同一张表当期望值，这样发现不了手写行为漂移。

| 验证层 | 必须观察的结果 |
|---|---|
| grammar / schema / formatter | 合法输入往返稳定；坏引用、重名、转换冲突、歧义边、未知字段、终态出边、不可达状态、资源超限有定位诊断；既有 kind 不回归 |
| 代码生成 / 构建 | 相同输入产物一致；guard 关联保留；缺少 guard 实现暴露为编译失败；跨文件命名冲突被拒绝；增删改输入会重建，删除必需输入不能借残留 include 通过；源码树不被改写 |
| 结构双向校验 | 所有状态 / 事件对有结果，实际 guard 标识与声明边一致，所有终态拒绝后续推进 |
| guard / payload | 每个 guard 的真、假与关键边界有独立夹具；拒绝时状态和资产不变且 action 未执行；错误阶段 / 不匹配资产的 payload 被拒绝 |
| 副作用与故障注入 | 读源、解析、分配 / staging 与确认未发布的失败保留旧工程；发布结果不确定时禁止盲目重试 / 清理并核对凭据；成功只提交一次；通知失败不篡改已提交结果 |
| 异步序列 | 取消先于 / 晚于提交、保存 / 重命名与提交竞争、重复完成、提交期间关闭和同工程重开、提交自身推进修订、旧 attempt 结果、worker 启动失败均有可重放测试；Task 不能独立改报取消 |
| 真实业务与桥接 | 准备 → 提交 → 显示 → 保存 / 重开可验证；Task 终态与领域事务一致；UI 不因长操作阻塞，native 资源按拥有者释放 |

白盒转移测试使用每个状态合法的上下文夹具，不能凭 `new_for_state` 配一个通用空上下文就证明真实流程。另需从初态经过实际事件顺序的业务测试和失败注入，独立断言修订、资产、事件次数及后端调用次数。结构一致不等于数学证明，也不证明所有 guard 可满足或外部 I/O 原子性。

最终验证从仓库 Cargo 聚合入口执行，具体命令与证据由 [073](../task/073-fsm-dsl-and-import-state-machine.md) 记录；只有接入真实 UI / FFI 时才包含相应窗口与跨平台验收，不以无头状态图测试替代。

## Qt 交互状态与演进门槛

复杂 UI 交互明确采用 Qt 自带的 `QStateMachine`，由 [074](../task/074-qt-interaction-state-machine.md) 独立接入，设计见 [Qt 交互状态机](qt-interaction-state-machines.md)。首期由 C++ ViewModel / 控制器管理导入窗口的选文件、预览、等待回执与关闭；Qt 拥有这些局部交互状态，业务结果仍来自 Rust。Qt StateMachine 当前尚未在构建供给与链接中登记，不把本决策写成已实现。

简单展示继续使用属性绑定和 QML `states / transitions`；[Qt Quick States](https://doc.qt.io/qt-6/qtquick-statesanimations-states.html) 描述对象 / 属性配置，不等同于领域事务执行器。Qt 状态机依赖事件循环，不能替代业务工作线程，也不自动使当前 STL 同步调用异步化。

UI 关闭、重建或重载后的业务展示以 Rust 快照为准，表单草稿和 UI 服务会话的存续范围由 074 / 027 明确，不能根据动画结束、按钮点击或本地状态推断工程已提交。任何 Qt 状态方案都不读取 FSM `.pa`，不重复 guard 或工程提交规则。

| 触发条件 | 后续动作 |
|---|---|
| 视口按需读取已提交 STL 资产 | 由 073 / 080 首先验证只读资源激活与迟到结果处理 |
| STEP 等写入型导入任务准备就绪 | 新增业务消费者，扩展 073 或另立实施范围以涵盖提交裁决 |
| Qt 复杂交互需要集中编排 | 由 074 独立接入 QStateMachine 与导入窗口；不等待 073 完成，异步业务能力随后按实际契约联调 |
| 第二个领域需要 FSM | 复用已有 DSL 内核；只在实际共同语义明确时提取执行辅助，不自动建立 `panta-fsm / panta-common` |
| 多处出现相同调度、异步 action、层级 / 并行状态需求 | 先记录具体难点，再核验 `rust-fsm`、`smlang` 等上游能力及依赖成本，不以库名提前承诺解决方案 |
| 希望在类型层限制合法调用序列 | 单独评估 typestate 与运行期事件的适配；不将某个库直接等同于完整形式化证明 |
| 转移决策被测量为瓶颈 | 固定输入、硬件与构建配置后比较实现，当前无性能结论 |
| 需要用户编辑 / 热加载流程或崩溃后续跑 | 属于信任模型、版本与持久化语义变化，另立设计，不扩展本 V1 |
