# 073 — FSM DSL 与首个异步 STL 视口资源激活

- 状态：done
- 阶段：CAE 业务编排
- 依赖：[008](008-tasks-errors-logging.md)、[034](034-rust-panta-artifact-parser.md)、[035](035-pa-formatter-and-validator.md)、[067](067-rust-mesh-domain-migration.md)、[072](072-flow-state-machine-planning.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-10-17

## 目标与背景

在首个真实多阶段异步视口资源激活流程中引入编译期 `.pa` FSM 元数据与手写 Rust 状态机。首个消费者为任务 [080](080-qml-viewport-document-tabs.md)：用户首次打开已保存在 `.panta` 内的 STL 时，Rust 按稳定 ImportRecord ID 读取并解析该资产，再将拥有明确所有权的网格快照交给视口层。加载不改写工程或提交记录；FSM 负责加载阶段、取消、工程代次校验与迟到结果丢弃。登记时 `ProjectService` 只在内存持有最新导入网格，`kind: fsm`、代码生成和真实异步业务尚未实现。此后 DSL、只读激活及 080 联调均已落地，验收结果见完成摘要。

存储来源已由后续任务 [088](088-mesh-asset-sidecar-storage.md) 从工程内 STL 副本迁移为 schema 4 Mesh sidecar。073 的稳定导入 ID、只读激活、取消及过期结果契约保持不变；当前激活读取并验证 Mesh sidecar，不读取来源 STL。

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

- [x] FSM 输入在 CLI 与 build script 中按同一规则接受 / 拒绝，坏引用、重复 / 歧义边、终态出边、不可达状态、命名冲突、未知字段及资源上限都有定位诊断；旧 kind 不回归。
- [x] 相同输入生成一致的枚举 / 元数据，每条 guard 关联保留；输出路径注册拒绝重复目标；真实 Cargo fixture 验证输入增删改触发重建，删除 `include!` 所需输入即使仍有其他 FSM 也会因缺少生成文件而构建失败；错误后无残留输出，源码树保持不变。当前 canonical kebab-case 名称和 stem 一致约束下无法构造不同合法名称映射到同一输出的输入，防御性冲突检查另有直接测试。
- [x] `ALL_STATES × ALL_EVENTS` 对实际手写决策做双向结构校验；每个 guard 有真 / 假及边界场景，生产 API 不开放任意设状态。
- [x] 从初态到解析就绪、失败、取消和过期丢弃的资源激活事件路径可重放；结果与已提交 ImportRecord 一致，ProjectManifest、revision 与 dirty 原值不变。
- [x] 覆盖关闭待加载 tab、连续切换目标、工程关闭 / 重开、project generation 变化、旧 attempt 与重复 / 迟到结果；过期事件不发布快照且解析资源得到释放。
- [x] Task 成功只表示网格快照已加载并可供 ViewModel 使用；失败和取消各终止一次；长解析不阻塞 GUI，GUI 线程不等待 worker join。
- [x] 可控解析器接缝覆盖读取失败、格式错误、取消和 worker 启动失败；当前活动视口与工程持久状态按契约保留。
- [x] 无重复 parser、工程事务反向依赖或遗留执行路径；现有真实窗口和三平台验证均有记录。本轮 Cargo fixture 已随 PR #19 通过 macOS、Linux、Windows 构建与测试。

## 验证计划与结果

以下为已执行的验证记录。命令在仓库根目录、按 `rust-toolchain.toml` 与锁文件环境运行：

| 日期 | 入口 / 场景 | 预期 | 结果 |
|---|---|---|---|
| 2026-09-26 | `cargo test --locked`（dsl-core / dslc / core / ffi / import 定向） | FSM schema 双向结构校验、激活竞态与 FFI 面回归 | 通过；15 个测试套件全绿（FSM schema 15、CLI 14、core 单元 38、激活集成 7、工程 15、FFI 15 等） |
| 2026-09-26 | `cargo coverage` | 新增 FSM / 激活代码纳入函数 / 行门禁，边界完整 | 通过；函数 89.59%（门槛 89）、行 92.97%（门槛 92），退出码 0。边界补齐：状态 / 边数恰好达限（128 / 512）接受、超限拒绝，from / on / to 逐字段缺失，states 节内转移声明，1MiB 源上限，无 guard 生成物，checked 读取未登记扩展名，清单资产 `..` / 绝对路径穿越在激活提交边界拒绝，drain 旧代次过滤，空推进无操作；`fsm/mod.rs` 行覆盖 83.61%→95.51%、`generate.rs` 100% |
| 2026-09-26 | `cargo format --check`、`cargo lint`（clippy + clang-tidy + cppcheck） | 质量门禁通过且不再被 windows.h 自动修复破坏 | 通过；`.clang-tidy` / comments.md 例外条款落地后 vtk_native_surface.cpp include-cleaner 全清 |
| 2026-09-26 | `cargo test --locked --workspace`（聚合入口，含 native CTest 56 项） | 聚合门禁全绿；C++ ViewModel 侧与新 open 契约一致 | 通过；100% tests passed。顺带修正 `ProjectViewModelTest` 对已移除的同步网格重载断言（重开工程后快照为空，激活恢复由 FFI / Rust 集成测试覆盖） |
| 2026-10-14 | `cargo test --locked -p panta-core --test fsm_build` | 验证生成器和真实 Cargo 输入生命周期 | 通过；4 项测试覆盖生成器增删改、无效输入清理、stem 与输出名冲突，以及临时 Cargo 消费者的目录监听重建和必需输入删除失败。Fixture 使用仓库 `build.rs`、根 `Cargo.lock` 与转义后的绝对依赖路径；变更后的构建使用 `--locked --offline`。 |
| 2026-10-14 | `cargo test --locked --workspace` | Rust、native、QML 聚合回归通过 | 通过；Rust workspace 测试通过，CTest 75/75，包含 FSM、资产激活、ViewModel、QML 和 FFI 回归。 |
| 2026-10-14 | `cargo format --check`、`cargo lint` | 格式与静态质量门禁通过 | 通过；Clippy、依赖检查、CMake/QML lint、clang-tidy、include-cleaner、cppcheck 均通过。 |
| 2026-10-14 | [PR #19 CI](https://github.com/Yuki-Nagori/panta/actions/runs/38491168172) | 新增 Cargo fixture 的三平台构建 / 测试与聚合质量门禁 | 通过；15/15 作业通过，包含 macOS、Linux、Windows 构建及 Rust / native 覆盖率、QML 测试和静态检查。Windows 作业通过；编译日志中的 `lock_crash_state` 未使用警告来自既有 Unix 测试辅助函数，不影响构建，也不属于本任务改动。 |
| 2026-10-17 | `cargo +nightly-2026-09-15 miri test --locked -p panta-core --test fsm_build` | Miri 验证真实 Cargo fixture 的输入重建与失败路径 | 通过；3 项测试通过，Cargo 子进程 fixture 按设计在 Miri 下跳过。该 fixture 依赖独立 Cargo 进程与宿主文件系统复制 syscall，仍由普通 Cargo 测试覆盖。 |
| 2026-10-17 | `cargo verify-changed` | 验证受影响 Rust 与 native / QML 范围 | 通过；Rust 受影响包与 CTest 67/67 通过。随后按 [032](032-cross-language-quality-gates.md) 将 Miri 范围细化为跳过 zstd FFI 用例、保留纯 Rust 工程 / 激活用例；因本机 rustup 无法在沙箱外创建工具链临时文件，聚合 `cargo ub-check` 留待 CI 验证。 |
| 2026-10-17 | Miri 范围后续验证 | 保留纯 Rust 工程与激活路径的 Miri 检查 | 15 个白名单测试通过；`cargo verify-changed` 的 Rust workspace 测试与 CTest 77/77 通过。聚合 `cargo ub-check` 在本机受 rustup 临时目录权限限制，等待 CI 验证。 |
| 2026-10-09 | [CI 37910040544](https://github.com/Yuki-Nagori/panta/actions/runs/37910040544) 及 080 真窗口验收 | 三平台 CI 与异步页签真实视口行为 | 三平台 CI 全部 19 个作业通过；真实窗口切换、关闭、迟到结果与 VTK 资源释放证据见 [080](080-qml-viewport-document-tabs.md#验证计划与结果)。 |


## 清理与兼容例外

登记时尚无代码变更或兼容例外。后续接入应清理被替换的模拟执行路径及重复编排；仍服务独立测试的模拟后端须说明用途。FSM 未发布前不保留多版本 parser，持久化 schema 变化由业务任务单独评估。

## 风险与回退

主要风险是声明 / 实现漂移、生成器输入遗漏、工程切换后的旧结果发布和网格快照泄漏。通过结构校验、独立行为断言、构建重建测试及可控异步解析器验证。首期只读加载不涉及写入 / 提交竞态；未来写入型导入的存储发布核对必须由后续业务任务单独冻结。若 080 的只读 DTO 或 session generation 契约未冻结，延后本任务；回退只撤销本任务实现，不改变已有工程数据或保留重复运行引擎。

## 决策与工作记录

- 2026-09-24：由 072 登记后续计划，采用公共 DSL 内核与领域内手写状态机；实现受真实消费者前置条件约束。
- 2026-09-24：提交前复审补齐单一终态决定权、提交期间关闭与修订回执、存储结果核对和跨文件生成冲突的前置条件与验收；未开始实现。
- 2026-09-24：将 080 登记为首个消费者，073 首期范围改为只读激活已提交 STL 资产；不推进工程 revision / dirty、不处理写入提交。原提交阶段、发布核对等设计仅约束未来写入型导入消费者。
- 2026-09-26：完成整体 review 与设计评估。循环 / 重入论证：①协调器单锁（`Mutex<CoordinatorState>`）+ 独立 `AtomicU64` 代次，无嵌套锁，死锁构造上不可能；②dispatch/decide/evaluate/outcome_for 均为纯自由函数，公共方法先取锁后派发且互不调用，无重入环；③跨语言单向（Rust 不回调 Qt，拉取式 drain），迟到结果在 Rust 边界过滤，UI 轮询不构成事件环；④状态图严格前进无环，终态禁出边，attempt 终态至多发布一次（出表），无事件风暴。已知边界：`parse_stl` 不可中断（旧代次大解析持续到完成边界，073 明文允许）；drain 间隔依赖 UI 轮询（10ms）；worker 数 ≤ 在飞 attempt 数。review 优化：事件 / guard 转换冲突分表（跨枚举误报）、`validate_document` 参数聚合为 `FsmHeader`、`bfs` 更名 `reachable_from`、begin 失败兜底具名化、补 `pa.fsm_missing_initial` 诊断（原必填项静默通过）、公共结构体补字段契约；工具链上 windows.h 伞头以「保留伞头 + granular provider 头 + 包含行 NOLINT」落地（IgnoreHeaders 会剥夺 provider 语义，实证不可用），`.clang-tidy` 与 comments.md 同步修订。`panta-dsl-core::fsm` 落地 V1 schema、校验、格式化与确定性生成，CLI check / format 按头部 kind 分发；`fsm/open-saved-stl.pa` + build.rs + 手写 `Idle → LoadingAsset → Parsing → Ready`（Failed/Cancelled/Expired 终态）与生成转移表做双向结构校验；取消检查点为分块读取与完成边界（`parse_stl` 不可中断，边界 guard 释放过期快照）；panta-core `open()` 移除同步最新导入解析，新增 begin / cancel / drain 入口与 FFI 面。guard 拒绝语义：`record-valid` 在 begin 提交边界同步拒绝（不建 attempt），`session-current` 在完成边界拒绝后由协调器的代次失效路径终态化。

## 完成摘要

实现与验收完成。DSL 内核、只读激活 FSM、FFI 与 080 联调已实现；本轮补齐真实 Cargo 输入生命周期与必需输入删除的失败验收。现有失败 / 取消 / 过期、状态矩阵、视口生命周期、三平台 CI 和 109 执行容量回归均通过。输出命名冲突检查作为防御路径测试；根据 canonical 名称和 stem 一致规则，不同合法输入之间无法形成同名输出。PR #19 的 Linux、Windows、macOS 构建与完整质量门禁均通过。2026-10-14 同步清理了目标、验证说明和本轮范围中残留的进行中措辞。

- 2026-10-14 独立只读评审：首轮指出直接调用生成器不足以证明 Cargo 重建，且唯一输入删除未覆盖仍有其他 FSM 的情况；已增加使用仓库原版 build.rs 的临时 Cargo 消费者 fixture，补测真实新增 / 修改 / 删除和必需 `include!` 输入删除。复核指出 Windows canonical path 直接替换分隔符会破坏 verbatim 路径，已改用 JSON 字符串编码写入 TOML；复制根 Cargo.lock 作为解析起点，首次离线锁定 fixture 后后续检查传 `--locked`。评审未实际运行 Windows，跨平台结果留给 CI。
- 2026-10-17：Nightly Miri 在 Linux 上运行真实 Cargo 子进程 fixture 时触发 Miri 不支持的 `copy_file_range` syscall。该用例验收 Cargo 子进程的构建重建行为，不是 Rust 内存安全边界；增加仅对 Miri 生效的 ignore，普通测试仍执行完整夹具。定向 Miri 通过；本机完整 `cargo ub-check` 因另一个 zstd FFI 外部函数限制未能完成。
- 2026-10-17：Nightly Miri 随后触及 Mesh sidecar 的 zstd 原生 FFI；具体失败和修订范围记录在 [032](032-cross-language-quality-gates.md)。保留不访问 Mesh sidecar 的工程 / 激活集成用例进行定向 Miri 检查；本机聚合入口受 rustup 临时目录权限限制，等待 CI 验证。

## 本轮已完成范围（2026-10-14）

- 已为 `panta-core/build.rs` 的真实生成逻辑建立可控临时目录测试，覆盖输入新增、修改、删除后的输出集合与内容更新，以及唯一必需输入删除、空目录、无效声明和输出命名冲突的失败行为。
- 已验证生成器在失败时不会留下可被后续构建误用的旧输出；测试直接调用 build script 共用的生成逻辑，不复制生成规则。所属 crate 测试、workspace 聚合验证、格式与 lint 均通过，并完成独立只读评审。
- 本轮没有改写 FSM 业务状态机或 DSL schema，只补齐了原验收中缺失的消费者构建生命周期证据。
