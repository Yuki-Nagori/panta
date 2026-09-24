# 073 — Flow DSL 与首个异步导入状态机

- 状态：planned
- 阶段：CAE 业务编排
- 依赖：[008](008-tasks-errors-logging.md)、[034](034-rust-panta-artifact-parser.md)、[035](035-pa-formatter-and-validator.md)、[067](067-rust-mesh-domain-migration.md)、[072](072-flow-state-machine-planning.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

在首个真实多阶段异步导入闭环中引入编译期 `.pa` Flow 元数据与手写 Rust 状态机，约束导入准备、工程提交、取消和迟到结果。当前 STL 路径已经落地，通用 task 生命周期已有手写实现，但 `kind: flow`、代码生成、真实异步导入接入均未实现。本任务不因规划文档完成而自动开始。

## 必读

- [Flow 设计](../modules/flow-state-machines.md)、[DSL 工具链](../modules/dsl-engine-and-toolchain.md)、[PA 规范](../standards/pa.md)
- [分层规则](../standards/layering.md)、[Rust 领域边界](../architecture/native-domain-boundaries.md)、[应用平台与存储](../architecture/application-and-storage.md)
- [Rust](../standards/rust.md)、[Cargo](../standards/cargo.md)、[注释](../standards/comments.md)、[测试](../standards/testing.md)、[验证与评审](../standards/validation-and-review.md)
- [仓库文件](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

扩展公共 DSL grammar / AST / schema / formatter / CLI，生成强类型状态、事件种类、guard 及带 guard 的转移元数据。领域 build.rs 只写 `OUT_DIR`；`panta-import` 拥有准备流程，`panta-core` 拥有工程提交并复用现有任务服务。与首个真实业务任务共同验证主路径、故障、取消和迟到事件。

不引入通用 FSM crate、运行期解释器、动作表达式、层级 / 并行状态、用户流程编辑、持久化续跑；不为同步 STL 单独套状态机。不迁移通用 task lifecycle 到 import，也不预设将其转换成 `.pa`。完整 STEP 几何资产与 OCCT 后端实现由其业务任务负责，不扩进本任务。

## 前置条件与待决策

1. 034 / 035 的公共工具链基线与回归入口可复用，依赖状态复核完成。
2. 首个消费者优先为 STEP 异步导入；其完整业务任务尚未登记，开始本任务前须登记该 task，并在两份任务中明确接口交付与联调顺序。接口先于 Flow 集成，不能让两个任务互为完成依赖。009 的适配摘要冒烟不等于真实工程导入。
3. 先冻结业务 payload、来源快照、线程与 native 资源所有者、取消检查点、项目 generation / revision / attempt 校验和存储提交点，再将 schema 草案转为实现承诺。没有真实消费者时维持 planned，不先造空执行框架。
4. 提交存储必须定义部分写入、发布结果不确定时的核对及重开恢复规则；不得把 FSM 终态当作原子落盘保证。若平台不能判定发布结果，先扩充恢复状态 / 入口再冻结状态图，不能默认退回 `Failed`。具体 crate 子模块与错误码在开始时按现状更新本任务。
5. 明确 core 事务协调者唯一决定导入终态，Task 服务只是投影；改造现有模拟 worker 自行取消 / 成功、GUI 析构同步 join 的驱动方式。提交中的工程上下文独立于 UI 存活，关闭、保存、重命名与重新打开同一工程包都经过同一协调者。
6. Qt 模块供给与导入窗口交互状态机由 [074](074-qt-interaction-state-machine.md) 独立实施，不是本任务的 Rust 核心前置条件；074 首期可消费现有同步接口。本任务提供实际异步能力后再明确联调批次，不能由 UI 状态机模拟业务提交 / 取消。

## 实施步骤

1. 为首个消费者固定状态图、guard、终态、取消和事件相关性，记录与现有 task 状态的映射。
2. 在 `panta-dsl-core` 增加 Flow schema、输入诊断、格式化往返和确定性生成；`panta-dslc check / format` 共用该内核，保留既有 kind 回归。
3. 接入消费者 build.rs 的构建依赖、输入集合命名校验、增删改追踪与 `OUT_DIR` 私有 include，禁止生成器回写源码或用残留生成文件掩盖必需输入删除。
4. 实现领域 `enum + match`、强类型 payload 和 guard；工程事务仅在实际发布成功后转为 `Committed`，将可失败操作保留在对应阶段。
5. 用可控后端 / 存储接缝验证失败与竞态；接入真实业务并验证重开工程和 UI 展示；不长期保留两套执行路径。
6. 执行适用聚合检查，更新实际设计、清理记录与索引状态。

## 预计改动

现有 `crates/panta-dsl-core`、`crates/panta-dslc`、`crates/panta-import` 与 `crates/panta-core`；Flow 子模块、输入与消费者 build.rs 为待创建路径。FFI / ViewModel 只按业务消费需求映射快照与错误，不把生成枚举直接发布成 ABI。夹具与 native / QML 行为用例按测试规范归位。

## 验收标准

- [ ] Flow 输入在 CLI 与 build script 中按同一规则接受 / 拒绝，坏引用、重复 / 歧义边、终态出边、不可达状态、命名冲突、未知字段及资源上限都有定位诊断；旧 kind 不回归。
- [ ] 相同输入生成一致的枚举 / 元数据，每条 guard 关联保留；跨文件 / 输出路径命名冲突被拒绝；输入增删改触发正确重建，错误或已删除的必需输入不能使用残留生成文件通过，源码树保持不变。
- [ ] `ALL_STATES × ALL_EVENTS` 对实际手写决策做双向结构校验；每个 guard 有真 / 假及边界场景，生产 API 不开放任意设状态。
- [ ] 从初态到准备、提交和终态的真实事件路径可重放；失败注入观察到旧工程 / 资产 / 修订保留，成功提交及终态事件各一次。
- [ ] 覆盖取消与提交先后、不可中断阶段、关闭 / 重开、修订变化、旧 attempt、重复 / 迟到结果；拒绝或丢弃事件不会执行副作用，资源按契约释放。
- [ ] 发布边界的故障可区分未发布、已发布与待核对，待核对期间不误报失败、不盲目重试 / 清理；最终结果或明确恢复入口可验证。
- [ ] 提交期间关闭、提交自身推进修订和并发保存 / 重命名不会丢失成功回执或覆盖新工程；同工程重开受协调；Task 不能独立把已提交事务改报取消，正常退出不在 GUI 线程同步等待长 native 调用。
- [ ] Task 成功对应工程实际提交；工作线程不操作 UI，长操作不阻塞 GUI，保存 / 重开后的资产与修订一致。
- [ ] 无重复 parser、工程事务反向依赖或遗留执行路径；适用 Cargo 聚合与窗口 / 跨平台验证有真实证据，文档和状态同步。

## 验证计划与结果

下列均为计划，尚未执行。在仓库根目录、按 `rust-toolchain.toml` 与锁文件环境运行：

| 入口 / 场景 | 预期 |
|---|---|
| `cargo build --locked` | 构建期生成与实际消费者可编译；另测输入增删改、错误输入及独立构建目录 |
| `cargo test --locked --workspace` | Rust 行为回归、native 和 QML 适用测试从统一入口通过 |
| `cargo format --check`、`cargo lint` | 适用格式与静态检查通过 |
| 新建 Flow 夹具后的 `panta-dslc check / format --check` | 由上述测试覆盖实际夹具，实施后记录完整文件参数；当前不伪造不存在的输入命令 |
| 真实后端导入、取消、故障与保存 / 重开 | 记录消费者任务、平台、样例、结果和未覆盖限制 |
| 真实窗口验收及受影响平台 CI | 按仓库窗口规范验证 GUI 响应与状态展示；无头测试不能替代 |

## 清理与兼容例外

当前无代码变更、无兼容例外。接入真实任务时清理被替换的模拟执行路径及重复编排；仍服务独立测试的模拟后端须说明用途。Flow 未发布前不保留多版本 parser，持久化 schema 变化由业务任务单独评估。

## 风险与回退

主要风险是声明 / 实现漂移、生成器输入遗漏、工程提交竞态和错误的成功终态。通过结构校验、独立行为断言、构建重建测试和存储故障注入验证。若首个业务消费者尚不具备契约，延后本任务；回退仅撤销本任务改动，不保留双运行引擎，不改变已有工程数据。

## 决策与工作记录

- 2026-09-24：由 072 登记后续计划，采用公共 DSL 内核与领域内手写状态机；实现受真实消费者前置条件约束。
- 2026-09-24：提交前复审补齐单一终态决定权、提交期间关闭与修订回执、存储结果核对和跨文件生成冲突的前置条件与验收；未开始实现。

## 完成摘要

未实施。当前仅有设计、范围和验收规划。
