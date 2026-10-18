# 126 — OCCT Shape 仓库与 Netgen 复用

- 状态：done
- 阶段：CAE 领域模块迁移
- 依赖：009、010、066、067、119
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-10-18 / 2026-10-18

## 目标与背景

处理 GitHub issue [#59](https://github.com/Yuki-Nagori/panta/issues/59)：当前 geometry 和 mesh 分别读取同一个 STEP，统计摘要使用的形状与 Netgen 划分的形状没有对象级联系，网格边界组也不能可靠映射回来源几何面。

本任务为 native 适配层增加显式拥有和释放的运行期 shape 仓库。返回给调用方的 `ShapeId + revision` 是进程内句柄，不是工程资产 ID 或可持久化拓扑 ID；Rust 未来的几何领域模型仍负责工程身份、修订和引用规则。面 ID 只在一个导入 revision 内稳定，重新导入后不能沿用。

同时修复 issue [#58](https://github.com/Yuki-Nagori/panta/issues/58) 指出的 022 国际化文档过时内容。

## 必读

- [重库适配边界](../architecture/native-domain-boundaries.md)
- [几何架构](../architecture/geometry.md) 与 [网格架构](../architecture/mesh.md)
- [OCCT 规范](../standards/occt.md)、[Netgen 规范](../standards/netgen.md)、[分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[测试规范](../standards/testing.md)、[提交规范](../standards/commits.md)

## 范围与非目标

- 一次 STEP 导入生成摘要、按 revision 编号的稳定面 ID，并把 `TopoDS_Shape` 存入 native 仓库。
- 提供显式 shape 查询/划分与释放契约；Netgen 直接从仓库 shape 构造 OCC 几何，不再调用 `Ng_OCC_Load_STEP`。
- 将 Netgen 边界组映射到来源 shape 的稳定面 ID；保持 mesh DTO 中领域分组索引连续，并随 DTO 返回映射。
- 删除文件路径驱动的旧网格入口和相应测试/基准调用，不保留过渡兼容 API。
- 覆盖 shape 所有权、唯一 ID、revision 校验、释放后访问、Netgen 面映射及失败清理。
- 更新几何、网格、重库适配和 native layering 文档；修正 #58 指出的国际化文档状态。

不实现 Rust 工程级 `GeometryAsset`、磁盘持久化、拓扑编辑/修复、XDE 元数据、Rust→C++ 后端业务调用或跨 revision 拓扑追踪。

## 涉及分组与跨组依赖

涉及 Native L1 的 `panta_geometry` 与 Native L2 的 `panta_mesh`。允许 `panta_mesh → panta_geometry`：网格生成消费 geometry adapter 提供的运行期 shape 句柄；依赖不得反向。OCCT / Netgen 类型只出现在 native adapter 私有实现，不进入公共 API 或 CXX。

## 前置条件与待决策

- 锁定的 OCCT 8.0.1 / Netgen 6.2.2604 SDK 中，Netgen `OCCGeometry(const TopoDS_Shape&, int, bool)` 可直接从现有 shape 构造；实施时以本机头文件、库符号和真实测试确认行为及所有权。
- ShapeId 不复用；当前仓库对象不可变，因此导入后 revision 固定为初始值。所有操作都校验完整句柄。
- 面 ID 使用导入 revision 内的零起连续值。Netgen `SurfNr` 不直接作为稳定身份；适配器须以拓扑对象相等性将 Netgen face map 映射到仓库 face map。

## 实施步骤

1. 固化私有 native shape 仓库、句柄和同步/所有权契约，确保摘要与 shape 来自同一 STEP reader。
2. 增加按句柄生成网格入口，构造 Netgen OCCGeometry 时复用现有 shape，并从拓扑身份建立分组映射。
3. 迁移测试并删除文件入口；覆盖过期句柄、释放、重复生成、网格失败和几何可核对的面映射。当前仓库没有 STEP 导入/体网格性能基准，不为本任务新增不稳定的时间门槛。
4. 更新架构/分层文档及 #58 文档状态，运行受影响的 L1/L2 测试和 native 检查，独立只读评审完整 diff。

## 预计改动

`native/geometry/`、`native/mesh/`、`tests/cpp/geometry/`、`tests/cpp/mesh/`、`tests/performance/`、相关 CMake 与测试范围映射、`ai-docs/architecture/`、`ai-docs/standards/layering.md`、本任务索引，以及 `ai-docs/modules/internationalization.md`、`ai-docs/modules/README.md`、任务 022。

## 清理与兼容例外

删除 `import_step_summary` 的无仓库导入语义（若无其他消费者）及 `generate_tet_mesh_from_step(path, ...)` 文件入口和所有调用点。当前为开发阶段，不保留旧 native API 兼容层。句柄只在进程内有效，不写入工程文件。

## 验收标准

- [x] 每个有效 STEP 只由 geometry reader 解析一次；Netgen 从现有 `TopoDS_Shape` 构造 OCC 几何，无 `Ng_OCC_Load_STEP` 调用。
- [x] shape 仓库可插入、查询和释放；不同导入不复用 ID；旧 revision、未知 ID、释放后句柄返回结构化错误。
- [x] Netgen 输入面与 geometry face ID 通过拓扑身份映射，不依赖独立遍历顺序；`box_mm.step` 的 6 个面各自映射到唯一面 ID，所有边界三角面都引用有效分组。
- [x] 失败路径不发布部分 DTO；RAII 清理网格资源，释放与 shape 借用由同一把锁串行，返回时仓库已移除句柄；Netgen/OCCT 按其进程级约束串行执行。错误路径和重复生成测试通过。
- [x] `panta_geometry_step_import_test`、`panta_mesh_netgen_test`、`panta_mesh_ir_test` 与完整 native/QML CTest 通过；现有性能基准没有调用被替换的 STEP/Netgen API。
- [x] macOS、Linux、Windows PR CI 通过。
- [x] Issue #58 的国际化模块标题/现状、模块导航后续说明、任务 022 PR #57 合并记录与实际一致。
- [x] 架构、分层、任务索引与代码一致；已完成独立只读架构评审并更新本任务结果。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-18 | geometry/mesh CTest：`StepImport`、`NetgenMesher` | shape→Netgen 复用、释放/错误路径及面映射回归通过 | 15/15 通过；macOS arm64。本地链接提示 Rust 静态库最低系统版本高于 native target，不影响测试结果 |
| 2026-10-18 | `cargo verify-changed` | 本地 L1/L2 选择器、完整适用测试与质量检查通过 | 通过：公共 CMake 构建边界触发完整范围；Rust workspace、native/QML CTest 79/79、9 项 lint/format 检查均通过；macOS arm64 |
| 2026-10-18 | 官方 Netgen 源码与锁定 SDK API 核对 | 确认构造器和 `fmap` 生命周期/顺序假设 | 本地 Netgen 6.2.2604 头文件含公开 `OCCGeometry(shape, dim, copy)` 构造器和 `fmap`；上游实现说明 `copy=false` 保存传入 shape 并构建 Netgen 面表 |
| 2026-10-18 | PR [#62](https://github.com/Yuki-Nagori/panta/pull/62) CI | macOS、Linux、Windows 构建及所需质量检查全部通过 | 全部 14 项检查通过；Actions run [39110444126](https://github.com/Yuki-Nagori/panta/actions/runs/39110444126) |

## 风险与回退

Netgen face map 与 OCCT face map 的遍历顺序可能不同，必须按拓扑身份匹配并用多面夹具验证；不能假定序号一致。当前用一把锁串行保护 shape 借用、网格操作与释放；未来若调整并发模型，必须保留等价的生命周期保障。若 SDK 构造器在锁定 ABI 下不可安全消费，则任务应停止并修订接口决策，不能退回重复读取 STEP 的静默旁路。

## 决策与工作记录

- 2026-10-18：依据 issue #59 新建任务。区分 native 临时 ShapeRef 与未来 Rust 工程资产身份；face ID 只保证导入 revision 内稳定。
- 2026-10-18：将 issue #58 的小范围文档一致性修正并入本分支，避免留下已核对的过时状态。
- 2026-10-18：独立架构评审确认 geometry 所有 shape、mesh 单向依赖 geometry、FaceId 仅限 revision 内，以及统一 OCCT 锁的边界合理；据评审将面级包围盒查询拆为显式按需 API，仓库条目改为锁内值存储，并写明内部借用回调不可重入。

## 完成摘要

PR #62 的 macOS、Linux、Windows 构建及 14 项 CI 检查全部通过。issue #58 文档状态已同步；issue #59 的 shape 仓库、单次 STEP 读取、Netgen shape 复用和 revision 内面映射均已实现并通过测试。
