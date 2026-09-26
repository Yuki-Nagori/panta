# 073 — FSM DSL 与首个异步 STL 视口资源激活

- 状态：in-progress
- 阶段：CAE 业务编排
- 依赖：[008](008-tasks-errors-logging.md)、[034](034-rust-panta-artifact-parser.md)、[035](035-pa-formatter-and-validator.md)、[067](067-rust-mesh-domain-migration.md)、[072](072-flow-state-machine-planning.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-26

## 目标与背景

在首个真实多阶段异步视口资源激活流程中引入编译期 `.pa` FSM 元数据与手写 Rust 状态机。首个消费者为任务 [080](080-qml-viewport-document-tabs.md)：用户首次打开已保存在 `.panta` 内的 STL 时，Rust 按稳定 ImportRecord ID 读取并解析该资产，再将拥有明确所有权的网格快照交给视口层。加载不改写工程或提交记录；FSM 负责加载阶段、取消、工程代次校验与迟到结果丢弃。当前 `ProjectService` 只在内存持有最新导入网格，通用 task 生命周期也仍是模拟执行体；`kind: fsm`、代码生成和真实异步业务尚未实现。本任务不因规划文档完成而自动开始。

## 必读

- [FSM 设计](../modules/fsm.md)、[DSL 工具链](../modules/dsl-engine-and-toolchain.md)、[PA 规范](../standards/pa.md)
- [分层规则](../standards/layering.md)、[Rust 领域边界](../architecture/native-domain-boundaries.md)、[应用平台与存储](../architecture/application-and-storage.md)
- [Rust](../standards/rust.md)、[Cargo](../standards/cargo.md)、[注释](../standards/comments.md)、[测试](../standards/testing.md)、[验证与评审](../standards/validation-and-review.md)
- [仓库文件](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

扩展公共 DSL grammar / AST / schema / formatter / CLI，生成强类型状态、事件种类、guard 及带 guard 的转移元数据。领域 build.rs 只写 `OUT_DIR`；`panta-import` / `panta-mesh` 按现有职责解析已保存 STL，`panta-core` 按工程记录身份协调只读激活流程并复用现有任务服务。通过 080 已登记的接口验证主路径、故障、取消和迟到结果。

不引入通用 FSM crate、运行期解释器、动作表达式、层级 / 并行状态、用户流程编辑、持久化续跑；不为 tab 切换、关闭或已驻留网格复用套状态机，也不把现有同步 STL 导入重写进本任务。不迁移通用 task lifecycle 到 import，也不预设将其转换成 `.pa`。本次只读取已提交的 STL 资产，不写清单、不推进 revision / dirty；写入型 STEP 导入事务、完整几何资产与 OCCT 后端实现由后续业务任务承接。

## 前置条件与待决策

1. 034 / 035 的公共工具链基线与回归入口可复用，依赖状态复核完成。
2. 080 已登记并依赖 073；开始实现前，080 需冻结请求/结果 DTO、tab 重复打开语义与 session generation。两任务按“先提供 Rust 异步只读激活接口，再由 080 接 UI”的顺序联调，不要求 080 完成后 073 才能开始，也不形成互相完成依赖。
3. 冻结读取记录 ID、工程 generation、attempt、取消检查点、工作线程与 Mesh 快照所有者；结果只有在记录和运行期请求仍有效时才可交给 UI。加载不修改工程 revision / dirty，也不写存储。
4. 允许取消在解析器安全检查点生效；无法中断的解析需在结果边界验证过期身份并释放快照。不能因 UI tab 关闭或工程会话析构而在 GUI 线程同步等待长任务。
5. 通用 Task 终态只是 Rust 只读激活结果的投影；Task `Succeeded` 意味着资产读取/解析快照就绪，不代表持久化提交，也不能由 UI 事件自行制造。
6. Qt 模块供给与导入窗口交互状态机由 [074](074-qt-interaction-state-machine.md) 独立实施，不是本任务的 Rust 核心前置条件；tab 交互也不需要借用导入窗口状态机。未来写入型导入的事务协调、发布核对与重开恢复需另登记消费者任务。

## 实施步骤

1. 为 080 固定只读资源激活状态图、guard、终态、取消与事件相关性；分清 Rust 资源加载终态和 QML tab 展示状态。
2. 在 `panta-dsl-core` 增加 FSM schema、输入诊断、格式化往返和确定性生成；`panta-dslc check / format` 共用该内核，保留既有 kind 回归。
3. 接入消费者 build.rs 的构建依赖、输入集合命名校验、增删改追踪与 `OUT_DIR` 私有 include，禁止生成器回写源码或用残留生成文件掩盖必需输入删除。
4. 在 `panta-core::project` 实现按导入记录 ID 读取已保存 STL 的只读异步激活；领域 `enum + match`、强类型 payload 与 guard 拒绝无效 project generation、记录 ID 或 attempt。
5. 用可控文件 / parser 接缝验证失败、取消、重复与过期竞态；通过 FFI 暴露稳定任务 ID、进度/终态和可释放快照，由 080 接入 UI；不长期保留同步与异步的重复加载路径。
6. 执行适用聚合检查，更新实际设计、清理记录与索引状态。

## 预计改动

现有 `crates/panta-dsl-core`、`crates/panta-dslc`、`crates/panta-import`、`crates/panta-core` 和 `crates/panta-ffi`；FSM 子模块、输入与消费者 build.rs 为待创建路径。FFI 只映射稳定任务状态、导入记录身份和快照，不把生成枚举直接发布成 ABI；tab 及 ViewModel 展示集成由 080 承接。夹具与 native 行为用例按测试规范归位。

## 验收标准

- [ ] FSM 输入在 CLI 与 build script 中按同一规则接受 / 拒绝，坏引用、重复 / 歧义边、终态出边、不可达状态、命名冲突、未知字段及资源上限都有定位诊断；旧 kind 不回归。
- [ ] 相同输入生成一致的枚举 / 元数据，每条 guard 关联保留；跨文件 / 输出路径命名冲突被拒绝；输入增删改触发正确重建，错误或已删除的必需输入不能使用残留生成文件通过，源码树保持不变。
- [ ] `ALL_STATES × ALL_EVENTS` 对实际手写决策做双向结构校验；每个 guard 有真 / 假及边界场景，生产 API 不开放任意设状态。
- [ ] 从初态到解析就绪、失败、取消和过期丢弃的资源激活事件路径可重放；结果与已提交 ImportRecord 一致，ProjectManifest、revision 与 dirty 原值不变。
- [ ] 覆盖关闭待加载 tab、连续切换目标、工程关闭 / 重开、project generation 变化、旧 attempt 与重复 / 迟到结果；过期事件不发布快照且解析资源得到释放。
- [ ] Task 成功只表示网格快照已加载并可供 ViewModel 使用；失败和取消各终止一次；长解析不阻塞 GUI，GUI 线程不等待 worker join。
- [ ] 可控解析器接缝覆盖读取失败、格式错误、取消和 worker 启动失败；当前活动视口与工程持久状态按契约保留。
- [ ] 无重复 parser、工程事务反向依赖或遗留执行路径；适用 Cargo 聚合与窗口 / 跨平台验证有真实证据，文档和状态同步。

## 验证计划与结果

下列均为计划，尚未执行。在仓库根目录、按 `rust-toolchain.toml` 与锁文件环境运行：

| 入口 / 场景 | 预期 |
|---|---|
| 2026-09-26 | `cargo test --locked`（dsl-core / dslc / core / ffi / import 定向） | FSM schema 双向结构校验、激活竞态与 FFI 面回归 | 通过；15 个测试套件全绿（FSM schema 15、CLI 14、core 单元 38、激活集成 7、工程 15、FFI 15 等） |
| 2026-09-26 | `cargo coverage` | 新增 FSM / 激活代码纳入函数 / 行门禁，边界完整 | 通过；函数 89.59%（门槛 89）、行 92.97%（门槛 92），退出码 0。边界补齐：状态 / 边数恰好达限（128 / 512）接受、超限拒绝，from / on / to 逐字段缺失，states 节内转移声明，1MiB 源上限，无 guard 生成物，checked 读取未登记扩展名，清单资产 `..` / 绝对路径穿越在激活提交边界拒绝，drain 旧代次过滤，空推进无操作；`fsm/mod.rs` 行覆盖 83.61%→95.51%、`generate.rs` 100% |
| 2026-09-26 | `cargo format --check`、`cargo lint`（clippy + clang-tidy + cppcheck） | 质量门禁通过且不再被 windows.h 自动修复破坏 | 通过；`.clang-tidy` / comments.md 例外条款落地后 vtk_native_surface.cpp include-cleaner 全清 |
| 2026-09-26 | `cargo test --locked --workspace`（聚合入口，含 native CTest 56 项） | 聚合门禁全绿；C++ ViewModel 侧与新 open 契约一致 | 通过；100% tests passed。顺带修正 `ProjectViewModelTest` 对已移除的同步网格重载断言（重开工程后快照为空，激活恢复由 FFI / Rust 集成测试覆盖） |
| `cargo build --locked` | 构建期生成与实际消费者可编译；另测输入增删改、错误输入及独立构建目录 |
| `cargo test --locked --workspace` | Rust 行为回归、native 和 QML 适用测试从统一入口通过 |
| `cargo format --check`、`cargo lint` | 适用格式与静态检查通过 |
| 新建 FSM 夹具后的 `panta-dslc check / format --check` | 由上述测试覆盖实际夹具，实施后记录完整文件参数；当前不伪造不存在的输入命令 |
| 按 ImportRecord ID 只读读取 STL、取消与过期结果 | 验证资产内容正确、网格快照可释放，且 Manifest / revision / dirty 均不变 |
| 关闭待加载标签、切换工程与同记录重复请求 | 用可控解析后端验证 session generation / attempt 过滤和资源释放；由 080 联调 |
| 真实窗口验收及受影响平台 CI | 按仓库窗口规范验证异步完成后更新单一 VTK 视口；无头测试不能替代 |

## 清理与兼容例外

当前无代码变更、无兼容例外。接入真实任务时清理被替换的模拟执行路径及重复编排；仍服务独立测试的模拟后端须说明用途。FSM 未发布前不保留多版本 parser，持久化 schema 变化由业务任务单独评估。

## 风险与回退

主要风险是声明 / 实现漂移、生成器输入遗漏、工程切换后的旧结果发布和网格快照泄漏。通过结构校验、独立行为断言、构建重建测试及可控异步解析器验证。首期只读加载不涉及写入 / 提交竞态；未来写入型导入的存储发布核对必须由后续业务任务单独冻结。若 080 的只读 DTO 或 session generation 契约未冻结，延后本任务；回退只撤销本任务实现，不改变已有工程数据或保留重复运行引擎。

## 决策与工作记录

- 2026-09-24：由 072 登记后续计划，采用公共 DSL 内核与领域内手写状态机；实现受真实消费者前置条件约束。
- 2026-09-24：提交前复审补齐单一终态决定权、提交期间关闭与修订回执、存储结果核对和跨文件生成冲突的前置条件与验收；未开始实现。
- 2026-09-24：将 080 登记为首个消费者，073 首期范围改为只读激活已提交 STL 资产；不推进工程 revision / dirty、不处理写入提交。原提交阶段、发布核对等设计仅约束未来写入型导入消费者。
- 2026-09-26：完成整体 review 与设计评估。循环 / 重入论证：①协调器单锁（`Mutex<CoordinatorState>`）+ 独立 `AtomicU64` 代次，无嵌套锁，死锁构造上不可能；②dispatch/decide/evaluate/outcome_for 均为纯自由函数，公共方法先取锁后派发且互不调用，无重入环；③跨语言单向（Rust 不回调 Qt，拉取式 drain），迟到结果在 Rust 边界过滤，UI 轮询不构成事件环；④状态图严格前进无环，终态禁出边，attempt 终态至多发布一次（出表），无事件风暴。已知边界：`parse_stl` 不可中断（旧代次大解析持续到完成边界，073 明文允许）；drain 间隔依赖 UI 轮询（10ms）；worker 数 ≤ 在飞 attempt 数。review 优化：事件 / guard 转换冲突分表（跨枚举误报）、`validate_document` 参数聚合为 `FsmHeader`、`bfs` 更名 `reachable_from`、begin 失败兜底具名化、补 `pa.fsm_missing_initial` 诊断（原必填项静默通过）、公共结构体补字段契约；工具链上 windows.h 伞头以「保留伞头 + granular provider 头 + 包含行 NOLINT」落地（IgnoreHeaders 会剥夺 provider 语义，实证不可用），`.clang-tidy` 与 comments.md 同步修订。`panta-dsl-core::fsm` 落地 V1 schema、校验、格式化与确定性生成，CLI check / format 按头部 kind 分发；`fsm/open-saved-stl.pa` + build.rs + 手写 `Idle → LoadingAsset → Parsing → Ready`（Failed/Cancelled/Expired 终态）与生成转移表做双向结构校验；取消检查点为分块读取与完成边界（`parse_stl` 不可中断，边界 guard 释放过期快照）；panta-core `open()` 移除同步最新导入解析，新增 begin / cancel / drain 入口与 FFI 面。guard 拒绝语义：`record-valid` 在 begin 提交边界同步拒绝（不建 attempt），`session-current` 在完成边界拒绝后由协调器的代次失效路径终态化。

## 完成摘要

未完成。DSL 内核、只读激活 FSM 与 FFI 面已实现并有测试覆盖；剩余真实窗口 / 跨平台验证证据与 080 UI 联调收口后同步。
