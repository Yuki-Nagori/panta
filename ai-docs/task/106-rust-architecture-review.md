# 106 — Rust 代码结构与职责边界评审

- 状态：done
- 阶段：架构与文档准备
- 依赖：[066](066-native-domain-boundaries.md)、[067](067-rust-mesh-domain-migration.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-20

## 目标与范围

根据用户要求，评审现有 Rust workspace、crate 依赖、应用服务 / 领域 / FFI 边界、后台任务生命周期及构建 / 质量工具的组织结构。输出带源码依据的优化点、优先级及建议实施顺序；实施重构另行确认范围与验证计划。

## 必读

- [分层规则](../standards/layering.md)
- [领域职责审计](../architecture/native-domain-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [文档规范](../standards/documentation.md)

## 验收标准

- [x] 核对实际 crate 依赖与目标分层。
- [x] 优化建议有具体代码位置、触发场景和成本，区分功能风险与可维护性问题。
- [x] 记录可保持的设计及分阶段建议，避免无消费者的 crate / trait 拆分。

## 验证与结果

本轮为源码静态评审，检查 Cargo 清单与服务 / DTO / 并发 / 工具实现。未执行行为测试或性能基准，不把代码规模当作性能证据。

## 工作记录

- 2026-10-08：登记评审。保留已有 105 验收收尾文档改动；按当前源码给出结论，不修改运行时实现。

评审源码基线为 `36bf598`。下列发现与建议是该基线的历史快照，不代表当前未解决项：修订策略由 [107](107-rust-service-boundaries-and-revisions.md) 收敛，工程写入协调由 [108](108-project-write-coordination.md) 完成，有界执行由 [109](109-bounded-background-execution.md) 完成，导入后台化由 [063](063-stl-import-and-mesh-workspace.md) 完成，DSL 与构建 / 质量工具拆分由 [110](110-dsl-module-boundaries.md) 和 [111](111-build-and-quality-module-boundaries.md) 完成；模拟任务管理链由 [128](128-dead-code-and-redundancy-cleanup.md) 删除。当前状态以这些任务及架构主题文档为准。

## 评审结论

### 已符合分层的设计

- 10 个 workspace package 的内部依赖（含 build-dependencies）无环。运行期核心方向为 `panta-ffi → panta-core → panta-import → panta-mesh`，进程级设施独立归 `panta-foundation`；core 的 DSL 依赖为构建期 FSM 生成，不是运行期解析。
- `panta-mesh`、`panta-import` 不依赖 Qt / VTK / CXX。网格领域校验由 mesh 执行，FFI 的扁平缓冲布局转换属于适配职责。
- 工程错误保持 Rust enum，CXX 输出稳定 code/category/detail；`project_response` 集中结果转换，未发现另一份工程状态。
- 路径根由宿主注入，崩溃 unsafe 集中在 foundation，构建依赖下载 / 安装与业务运行期分离。保留这些方向。

### 历史发现：工程事务及执行边界

以下各项记录 2026-10-08 评审时的问题和建议，实施状态见上方对应任务。

1. **当时确认导入仍同步，属于已登记的 063 缺口。** [import_stl](../../crates/panta-core/src/project/import.rs) 在该基线同时准备 / 读取来源、缩放网格、复制并同步资产、提交 manifest；GUI 通过同步 FFI 入口调用。建议先明确 Prepare / Commit 的候选快照、修订复核与取消截止点，再完成后台化。该建议已由 063 实施；本次历史评审没有测量延迟。
2. **当时后台执行存在多种机制，缺少共同容量 / 清理策略。** 该基线中的模拟任务管理器（历史路径 `crates/panta-core/src/task.rs`，已由 128 清理）逐任务 spawn、保存 handle，Drop 时 join；只读预检用编号 / 取消标志 / mpsc；元数据写入用分离线程及候选结果；资产激活用 FSM 协调器。共享有界执行基础由 109 实施，各消费者仍保留领域状态机和写入提交语义。原线程 / 内存规模判断是源码推断，不能视为当前测量结论。
3. **当时持久化模块的文件级协调不足。** [write_manifest](../../crates/panta-core/src/project/storage.rs) 在该基线固定使用 `.panta.tmp`，没有文件级写入互斥 / 磁盘修订复核，也未显式同步 manifest。并发写入协调已由 108 实施；当时记录的是源码风险判断，不是已复现的数据损坏。
4. **当时修订耗尽策略分散。** 工程命令、材料 / 工艺 / 导入多处使用 `saturating_add(1)`，而请求过期判断依赖 revision。该策略由 107 收敛并增加耗尽拒绝行为；历史基线的行为不能外推为当前实现。

### 历史建议：在现有 crate 内拆模块

以下模块整理建议分别由 107、110、111 完成，保留原评审依据供回溯；不作为当前待办。

- **ProjectService**：[project.rs](../../crates/panta-core/src/project.rs) 同时放模型、清单结构、错误、LRU、会话状态和命令编排；多个子模块通过 `use super::*` 扩展同一服务。建议拆为 `model / error / repository / mesh_cache / service`（规划模块名），保持 ProjectService 对外门面，降低子模块对所有私有字段的依赖。没有新增独立 crate 的必要消费者。
- **FFI**：[lib.rs](../../crates/panta-ffi/src/lib.rs) 把 CXX 契约、各服务转发、DTO 转换及测试集中在根文件。建议保留根 `#[cxx::bridge]` 的生成入口，用现有 project_response 的方式分离 `project / task / path / language / mesh` 转发实现与转换；共享桥接声明不机械拆成多个互相依赖的 bridge，避免额外的生成约束。
- **DSL**：[lib.rs](../../crates/panta-dsl-core/src/lib.rs) 同时包含 AST / 诊断、Pest 解析、语义校验、格式化和 TS 输出；FSM 已独立模块化，可作为组织方式参考。建议保留一个公共门面及同一 grammar，分离 `ast / diagnostic / parser / validate / format / emit_ts`（规划模块名），保持 CLI、构建脚本的公共 API 与输出一致。
- **构建与质量工具**：[panta-build](../../crates/panta-build/src/lib.rs) 可进一步分离工具资产登记、安装 / 校验、编译器解析与 Windows 环境；[质量入口](../../tests/src/main.rs) 可沿已有 coverage / performance 模块组织 build / lint / sanitizer 命令。优先级低于业务事务，不因单文件长度创建新 crate。

## 当时建议实施顺序

1. 在现有 crate 内整理 FFI 与 ProjectService 模块边界，保持公共 API / UI / 持久化格式；由 107 完成。
2. 明确工程写入对象、并行写入 / 修订耗尽回归及异步导入；分别由 108、107 和 063 完成。
3. 收敛真实后台消费者执行基础，验证取消、结果回收和写入收尾；由 109 完成。
4. 分批整理 DSL 和工具模块，验证 AST / 格式 / TS 稳定性和跨平台构建入口；由 110、111 完成。

## 完成摘要

静态架构评审于 2026-10-08 完成，实际依赖图与分层一致；本任务记录当时的源码依据、触发场景及未测限制，未修改运行时代码。后续建议已分别由 063、107–111、128 完成；本任务保留为历史评审，不代表这些建议仍待实施。
