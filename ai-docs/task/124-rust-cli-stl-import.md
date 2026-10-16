# 124 — Rust CLI 导入 STL

- 状态：in-progress
- 阶段：CAE 领域模块迁移
- 依赖：[121 GUI 与 CLI 共享 Rust 命令入口](121-rust-gui-cli-command-layer.md)、[063 STL 导入与工程工作区](063-stl-import-and-mesh-workspace.md)、[067 Rust 统一 STL 解析与 Mesh IR 领域校验](067-rust-mesh-domain-migration.md)、[073 FSM DSL 与导入状态机](073-fsm-dsl-and-import-state-machine.md)、[088 Mesh 资产旁置存储与按需加载](088-mesh-asset-sidecar-storage.md)、[123 Rust CLI 检查 STL](123-rust-cli-stl-inspection.md)
- 优先级：P2
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-17

## 目标与背景

提供脚本化的 STL 工程导入能力，复用 GUI 的导入选项、单位处理、来源记录、资产修订及事务失败回滚。CLI 不直接拼写工程文件或绕过工程服务。

## 必读

- [应用命令层](../architecture/command-layer.md)
- [分层标准](../standards/layering.md)
- [测试规范](../standards/testing.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 命令采用 `stl import <工程文件.panta> <文件.stl> [--mesh-type <id>] [--units <unit>]`，在独立 `ProjectService` 会话中打开目标工程并同步完成导入；先复用现有预览读取、解析和面数摘要，在需要时提前发出大 Mesh 提醒，再由确认导入流程复核来源字节并复用缓存网格；不重复解析、不启动 GUI、不新增任务 ID 或轮询器。
- 默认网格类型沿用 Rust `DEFAULT_MESH_TYPE`，默认单位为毫米；显式选项只接受 GUI 当前支持的网格类型 ID 和单位值，并复用 `ImportOptions` 校验及相同的 Rust 导入提交实现。GUI 保留后台 begin/finish 生命周期，CLI 调用同步入口。
- 将 STL 解析为工程内部 `SurfaceMesh`，复用 088 已完成的 sidecar v2 压缩格式、SHA-256 校验、不可变 revision 路径和“先发布 sidecar、后提交清单”的事务。工程内不保留原始 STL 副本，只保存导入时选定的原始路径、文件名、格式、SHA-256、mtime 和 size 等来源快照；源文件随后移动或删除不影响工程重开和 Mesh 加载。
- 成功时输出导入记录 ID、Mesh asset ID、显示名和来源摘要。重复来源名允许存在，由记录 ID 区分；显示名继续遵循导入记录的去扩展名规则，GUI 序号由现有文档工作区负责。
- 扩展现有 `project inspect` 输出资产 ID、显示名和来源摘要；不新增只返回资产清单的独立命令，也不依据来源路径重新读取文件。
- 估算展开 Mesh 数据超过 2 GiB 时输出非阻断提醒并继续导入；不设文件或展开数据硬上限，也不建议用户改建工程。
- 失败沿用稳定 `ProjectError` 诊断并保证不留下部分资产、错误修订或损坏的既有工程。
- 覆盖单文件成功、来源文件删除后的重开与加载、失败回滚、同名来源、路径与单位/网格类型校验及工程写入冲突。

非目标：批量并行导入、几何修复、B-rep 重建、网格优化、体网格生成或求解器执行。

## 前置条件与待决策

- 063、067、073、088、121 与 123 的前置能力均已完成；实施时复用当前 schema 4 ImportRecord、sidecar v2 和 `ProjectService` 现有同步导入入口，不引入旧 schema 兼容或原始 STL 副本。
- CLI 每次调用创建隔离的 `ProjectService`，通过 `ProjectCommand::Open` 打开指定工程，再调用同步导入入口；写入租约负责处理并发修改与事务冲突。
- 单位和网格类型的默认值及允许值以 Rust 导入领域定义为准，不在 CLI 复制校验表；CLI 仅将参数转换为领域输入并呈现结果。

## 验收标准

- [x] CLI 通过 `ProjectService` 同步入口复用 GUI 的 Rust 导入提交实现、选项校验和持久化事务语义；CLI 不启动 GUI 或轮询任务状态。
- [x] `stl import` 成功输出导入记录 ID、Mesh asset ID、显示名和来源摘要；`project inspect` 能列出这些已保存记录。
- [x] 导入后删除原始 STL，重新打开工程仍可按资产 ID/revision 加载 Mesh；工程不包含原始 STL 副本。
- [x] 同名来源、无效 STL、工程写入冲突、参数错误和目标路径错误均有 CLI 或领域行为测试；CLI 错误使用稳定诊断及非零退出码，事务失败不改变已提交工程状态。
- [x] 展开数据估算超过 2 GiB 时仅输出非阻断提醒并允许导入继续。
- [x] 任务记录适用验证、独立评审及所有配套文档状态。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-17 | `cargo test-plan`、`cargo verify-changed`（macOS arm64） | 选择直接影响的 Rust L1、消费者 L2 与相关 CTest；格式、Clippy、Rust/native/QML 测试通过 | 通过。L1：`panta-cli`、`panta-core`；L2：`panta-ffi`、`panta-launcher`；CTest labels：`app`、`bridge`、`ffi`、`i18n`、`mesh`、`qml`、`qt-adapter`；没有触发完整回退。CLI 单元测试 7/7、CLI 进程测试 14/14、core 115/115、ffi 32/32、launcher 4/4、测试选择器 23 项通过（3 项按设计忽略）、`native_and_qml_suite_passes` 3/3，native/QML CTest 67/67。 |
| 2026-10-17 | CLI 导入行为回归 | 验证显式选项、来源快照、同名来源、失败诊断、无部分提交及删除来源后仍可读取工程 | 通过：`tests/rust/cli.rs` 覆盖成功导入、源文件不被修改、删除来源后 `project inspect` 仍可读、单位/网格类型错误、缺失/损坏/不支持的 STL、无效工程文件及同名资产；core 导入回归覆盖来源删除后重新激活 Mesh、写入冲突与失败回滚。 |
| 2026-10-17 | 展开 Mesh 大小提醒边界 | 72 字节/三角形的估算跨越 2 GiB 阈值时正确分类，CLI 提醒不阻断提交 | 通过：`panta-cli` 单元测试覆盖阈值两侧；CLI 在同步提交前输出 stderr 提醒，继续调用共用导入事务。 |

## 风险与回退

工程导入涉及持久化和修订，必须经过已有应用服务事务边界。CLI 使用服务现有同步导入入口，不直接调用 sidecar、清单或导入解析实现；GUI 继续使用已有后台 begin/finish 生命周期。CLI 导入成功后不依赖来源文件存续。

## 决策与工作记录

- 2026-10-16：根据任务 121 的命令层盘点登记；排序在工程创建和 STL 只读检查之后，以降低写事务与取消语义风险。
- 2026-10-17：开始实施前复核依赖状态：088 已完成并合并，明确当前 schema 4、sidecar v2、Zstandard 与 SHA-256、导入事务及不复制来源 STL；123 已完成并合并，提供 CLI 原生路径和错误呈现模式。本任务沿用这些现状，不增加存储层或 CLI 专用解析规则。
- 2026-10-17：第一版决定采用同步 `stl import`；命令内部隔离打开工程，先生成预览摘要用于提前提醒，再调用现有同步 `ProjectService::import_stl`。该流程复用预览缓存的 Mesh 并校验源文件未变化，不新增轮询器。单位默认为毫米，网格类型沿用 `DEFAULT_MESH_TYPE`。
- 2026-10-17：超过 2 GiB 的展开 Mesh 估算仅显示提醒并继续，不设上限，也不建议新建工程。
- 2026-10-17：将 `panta-cli` 拆分为轻量 `main.rs`、命令分发与输出模块，以及独立的 project/STL 命令适配模块；用分段 `concat!` 定义可读的 usage 文本。同步 `ProjectService::import_stl` 改为调用与 GUI 后台 begin/finish 相同的 `commit_import` 事务实现，避免两套提交、清理及冲突处理逻辑。
- 2026-10-17：按 L1/L2 影响范围运行 `cargo verify-changed`，所有受影响 Rust、native 与 QML 验证通过。
- 2026-10-17：`gpt-6.1-sol` medium 独立只读评审完整 11 文件 diff，通过，未发现需要修复的问题。评审追踪了 `prepare_checked`、`WriteLease`、sidecar 写入与激活加载；限制为未现场执行 Windows/Linux 文件锁与替换、实际超 2 GiB 网格导入，后者由边界单测覆盖。

## 完成摘要

已实现同步 `stl import`、`project inspect` 资产来源摘要、共享导入事务、失败回滚及来源文件删除后的可用性回归，并同步 README 与命令层架构文档。受影响本地验证与独立只读评审通过；任务保持 in-progress，等待 PR/CI 验收。
