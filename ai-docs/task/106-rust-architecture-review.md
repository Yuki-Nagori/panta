# 106 — Rust 代码结构与职责边界评审

- 状态：done
- 阶段：架构与文档准备
- 依赖：[066](066-native-domain-boundaries.md)、[067](067-rust-mesh-domain-migration.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

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

## 评审结论

### 已符合分层的设计

- 10 个 workspace package 的内部依赖（含 build-dependencies）无环。运行期核心方向为 `panta-ffi → panta-core → panta-import → panta-mesh`，进程级设施独立归 `panta-foundation`；core 的 DSL 依赖为构建期 FSM 生成，不是运行期解析。
- `panta-mesh`、`panta-import` 不依赖 Qt / VTK / CXX。网格领域校验由 mesh 执行，FFI 的扁平缓冲布局转换属于适配职责。
- 工程错误保持 Rust enum，CXX 输出稳定 code/category/detail；`project_response` 集中结果转换，未发现另一份工程状态。
- 路径根由宿主注入，崩溃 unsafe 集中在 foundation，构建依赖下载 / 安装与业务运行期分离。保留这些方向。

### 优先项：工程事务及执行边界

1. **确认导入仍同步，属于已登记的 063 缺口。** [import_stl](../../crates/panta-core/src/project/import.rs) 同时准备 / 读取来源、缩放网格、复制并同步资产、提交 manifest；GUI 通过同步 FFI 入口调用。建议先明确 Prepare / Commit 的候选快照、修订复核与取消截止点，再完成后台化。它比纯文件拆分更影响实际响应；本轮没有测量延迟。
2. **后台执行存在四种机制，缺少共同容量 / 清理策略。** [模拟任务管理器](../../crates/panta-core/src/task.rs) 逐任务 spawn、保存 handle，Drop 时 join；[只读预检](../../crates/panta-core/src/project/preview.rs) 用编号 / 取消标志 / mpsc；[元数据写入](../../crates/panta-core/src/project/storage.rs) 用分离线程及候选结果；[资产激活](../../crates/panta-core/src/fsm/open_saved_stl.rs) 用 FSM 协调器。预检和激活的解析阶段不可中断，快速替换大文件可能让多个旧解析并行运行，缓存预算不覆盖这些在途快照。模拟 TaskManager 的终态记录及已完成 handle 保留到销毁。建议共享有容量限制的执行基础、请求身份与回收策略；保留各消费者的领域状态机和写入提交语义，先把模拟执行体从调度生命周期分离。线程 / 内存规模风险为源码推断，需基准再定容量。
3. **持久化模块还混合事务编排，文件级协调不足。** [write_manifest](../../crates/panta-core/src/project/storage.rs) 固定使用 `.panta.tmp`，没有文件级写入互斥 / 磁盘修订复核，写入后也未显式同步 manifest。`ensure_project_writable` 仅保护一个服务实例；两个实例写同一工程可能竞争临时文件或覆盖另一实例的提交。建议集中到明确的工程包写入对象：唯一临时文件、写入排他、预期修订复核、按平台定义持久化保证；候选准备及后台编排归应用服务。当前 UI 主要是单个服务实例，这不是已复现的数据损坏；多窗口 / 写入异步化前应明确契约。
4. **修订耗尽策略应集中。** [工程命令](../../crates/panta-core/src/project.rs)、材料 / 工艺 / 导入等多处使用 `saturating_add(1)`，而 [plan_settings](../../crates/panta-core/src/project/plan_settings.rs) 以 revision 判断过期请求。manifest 可直接载入 `u64::MAX`，现有测试也接受饱和后的成功写命令；此时后续变更不再产生新 revision。建议引入统一的修订递增入口并在耗尽时拒绝变更，连同现有饱和测试一起修订。运行期请求 ID 可逐步使用不同 newtype，避免混用 preview / activation / task 编号。

### 可维护性项：在现有 crate 内拆模块

- **ProjectService**：[project.rs](../../crates/panta-core/src/project.rs) 同时放模型、清单结构、错误、LRU、会话状态和命令编排；多个子模块通过 `use super::*` 扩展同一服务。建议拆为 `model / error / repository / mesh_cache / service`（规划模块名），保持 ProjectService 对外门面，降低子模块对所有私有字段的依赖。没有新增独立 crate 的必要消费者。
- **FFI**：[lib.rs](../../crates/panta-ffi/src/lib.rs) 把 CXX 契约、各服务转发、DTO 转换及测试集中在根文件。建议保留根 `#[cxx::bridge]` 的生成入口，用现有 project_response 的方式分离 `project / task / path / language / mesh` 转发实现与转换；共享桥接声明不机械拆成多个互相依赖的 bridge，避免额外的生成约束。
- **DSL**：[lib.rs](../../crates/panta-dsl-core/src/lib.rs) 同时包含 AST / 诊断、Pest 解析、语义校验、格式化和 TS 输出；FSM 已独立模块化，可作为组织方式参考。建议保留一个公共门面及同一 grammar，分离 `ast / diagnostic / parser / validate / format / emit_ts`（规划模块名），保持 CLI、构建脚本的公共 API 与输出一致。
- **构建与质量工具**：[panta-build](../../crates/panta-build/src/lib.rs) 可进一步分离工具资产登记、安装 / 校验、编译器解析与 Windows 环境；[质量入口](../../tests/src/main.rs) 可沿已有 coverage / performance 模块组织 build / lint / sanitizer 命令。优先级低于业务事务，不因单文件长度创建新 crate。

## 建议实施顺序

1. 在现有 crate 内整理 FFI 与 ProjectService 模块边界，保持公共 API / UI / 持久化格式；做一次 Cargo 聚合回归。
2. 结合 063 明确工程写入对象和提交契约，补并行写入 / 修订耗尽回归，推进确认导入后台化。
3. 针对已有三个真实后台消费者收敛执行基础，验证快速取消、大文件替换、服务销毁及候选提交；容量依据测量确定。
4. 分批整理 DSL 和工具模块，分别验证 AST / 格式 / TS 稳定性和跨平台构建入口。

## 完成摘要

静态架构评审完成，实际依赖图与分层一致；优化重点为事务 / 执行生命周期和 crate 内模块组织。记录了源码依据、触发场景及未测限制，运行时代码未改动。本任务 done 表示评审已完成，不表示建议的重构已实施；后续按用户选定范围登记实施任务。
