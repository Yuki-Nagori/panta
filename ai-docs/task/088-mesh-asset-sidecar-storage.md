# 088 — Mesh 资产旁置存储与按需加载

- 状态：planned
- 阶段：CAE 领域模块迁移
- 依赖：[063](063-stl-import-and-mesh-workspace.md)、[067](067-rust-mesh-domain-migration.md)、[073](073-fsm-dsl-and-import-state-machine.md)、[080](080-qml-viewport-document-tabs.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-29 / 2026-09-29

## 目标与背景

将 `.panta` 保持为小型工程元数据和资产索引；Surface / Volume Mesh 使用工程目录内可独立校验、带版本与修订身份的旁置资产文件。打开工程只读取清单与索引；首次打开某个文档页签时，Rust `panta-mesh` 按资产 ID 和 revision 加载权威 Mesh，缓存拥有内存的快照并批量交给 C++ 显示适配层。关闭页签释放该文档的运行期快照。

当前 schema 2 的 `.panta` 保存 STL import record 与相对路径；原始 STL 复制到 `assets/imports/`。073 激活未打开记录时读取该 STL 并解析为 `SurfaceMesh`，080 再把快照交给 VTK。此任务评估并落地持久 Mesh sidecar，避免每次工程重开和页签首次打开都重复解析原始 STL，同时支持只加载用户实际打开的资产。

当前 Rust `SurfaceMesh` 每个面存三个 `[f64; 3]` 顶点，内存 payload 约 72 字节 / 面；binary STL 约 50 字节 / 面。若保留两份，工程磁盘占用会增加；不能默认 sidecar 会压缩文件或节省内存。格式评估应比较紧凑坐标、索引 / 连接关系与压缩方案，同时保持单位、硬边语义和精度可验证。

Mesh sidecar 的主要收益是避免原始格式解析及其临时分配。它仍需读取 Mesh 数据、创建本次运行期快照、转换成 VTK 数据并上传图形资源；不承诺消除显示内存或 GPU 驻留。080 已通过跳过 STL 三角面上重复的 VTK 法线过滤器降低渲染切换高水位，不能把两种优化混为一项。

## 必读

- [应用平台、Rust 与工程存储](../architecture/application-and-storage.md)、[网格架构](../architecture/mesh.md)、[几何架构](../architecture/geometry.md)、[重库适配边界](../architecture/native-domain-boundaries.md)
- [063 STL 导入](063-stl-import-and-mesh-workspace.md)、[067 Rust Mesh IR](067-rust-mesh-domain-migration.md)、[073 异步激活](073-fsm-dsl-and-import-state-machine.md)、[080 文档页签](080-qml-viewport-document-tabs.md)
- [分层规则](../standards/layering.md)、[CXX](../standards/cxx.md)、[Rust](../standards/rust.md)、[测试](../standards/testing.md)、[性能测试](../modules/performance.md)
- [仓库文件规范](../standards/repository-hygiene.md)、[文档规范](../standards/documentation.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 设计可移植的 Mesh sidecar 布局和格式版本；`.panta` 清单只保存稳定 Mesh asset ID、revision、相对路径及验证所需的少量元数据。
- 由 `panta-mesh` 拥有 Surface Mesh 与体 Mesh 的持久化编解码、内容校验和领域校验入口；不把 C++ VTK 快照 DTO 当作持久领域模型。
- STL 导入时使用预览 / 导入事务已经解析的同一 `SurfaceMesh` 写旁置资产；不为持久化再读取和解析一次源文件。
- Rust 按需读取指定资产；打开工程不加载所有 Mesh，打开页签时异步加载，关闭页签后按 080 生命周期释放快照。
- 每个派生 Mesh 记录来源几何 / Mesh revision 与生成器或解析器版本；过期 revision 不得被静默当作当前数据。
- 测量保存、工程打开、首次激活、缓存命中/释放的 CPU 时间和内存，并与当前 STL 读盘解析路径比较。

不包含：

- B-rep 修复、OCCT shape 持久化、Netgen 生成任务与参数 UI；这些消费旁置 Mesh / Geometry asset 的工作另行登记。固定 Netgen SDK 暴露从 STL 三角面输入到体网格的 C adapter 函数，但闭合性、方向、内存规模及真实模型输出仍需独立网格任务验证。
- 改变 Rust 领域权威或把 Mesh IR 挪入 Qt、C++ ViewModel、VTK；C++ 只做 VTK / 重库适配。
- 把大型数组内联进 `.panta`、依赖开发机绝对路径，或用 VTK / OCCT 对象内存布局定义磁盘格式。
- 自动兼容多个未知二进制格式版本或静默修复损坏资产；不支持版本、revision 不匹配及校验失败必须可诊断。

## 前置条件与待决策

- 063、067、073、080 的领域身份、异步相关性与关闭释放契约完成并稳定。
- 当前 `SurfaceMesh` 是三角面顶点数据；在确定 sidecar 前验证表示方式、精度、单位、连接关系和未来体 Mesh 数据量。不要把示例工程一类 STL 的读时直接外推为压缩比或缓存预算。
- 明确原始 STL 的留存责任：Mesh 是工程重开和视口激活的规范资产；原 STL 是否继续作为可重新导入的来源副本，须按来源追溯和磁盘开销测量决定。
- 明确文件格式版本、内容摘要算法、相对路径规则、写入持久性与损坏诊断；若格式或写入事务无法保证失败后保留上一份有效索引，暂不提交 schema 变更。
- 明确 sidecar 发布顺序：写入临时文件、校验并发布不可变资产，再提交引用它的新清单；崩溃留下未引用文件不得影响已提交工程。垃圾回收不删除当前或仍被任务引用的 revision。

## 实施步骤

1. 核对当前 ProjectManifest、ImportRecord、SurfaceMesh / TetMesh 与异步激活所有权；补测 STL 解析、Mesh 解码、磁盘读写和批量 CXX 快照成本。
2. 定义 MeshAssetRef、资产 revision、来源 revision、单位、格式版本、摘要及损坏 / 缺失错误；审查是否需要索引化 SurfaceMesh 表示，避免未经测量改变 067 的领域模型。
3. 实现 Rust 持久化读取与写入。STL 导入复用已解析的 `SurfaceMesh` 写 sidecar；新资产使用不可变 revision 路径，完成校验后先发布数据文件，再原子更新 `.panta` 清单索引。
4. 把 073 只读激活改为按 ID/revision 读取 Mesh sidecar；缓存命中复用已拥有快照，关闭页签 / 工程、修订变化与任务过期按 080 释放或拒绝。
5. 为 Surface / Volume Mesh 增加序列化往返、边界值、缺失/损坏/不支持版本、revision 失配、写入中断、并发读取与工程移动测试。
6. 对真实小 / 中 / 大输入比较原 STL 解析和 Mesh sidecar 首次加载、缓存命中、保存时间、磁盘字节和内存峰值；更新 063、067、073、080 与存储架构文档。

## 预计改动

- `crates/panta-mesh/`：领域 Mesh 编解码与校验，按实现需要拆分模块。
- `crates/panta-core/src/project/`：Mesh asset index、revision 引用和事务提交；当前 manifest 内联 STL `imports` 的关系按最终 schema 明确。
- `crates/panta-ffi/`、`native/bridge/`：异步加载结果和批量快照边界，保持 CXX 只声明适配契约。
- `ai-docs/architecture/{application-and-storage,mesh,native-domain-boundaries}.md`、相关 task 与 `ai-docs/task-index.md`。

## 清理与兼容例外

资产读取路径替换完成后删除只供运行期 STL 重解析的旧激活路径，除非仍被显式恢复/重新导入流程使用。清点不再引用的 DTO、版本常量和旧测试。默认无兼容例外；存量工程迁移或读取兼容需另行记录明确策略，不能留双读路径兜底。

## 验收标准

- [ ] `.panta` 只含工程元数据和 Mesh asset 索引；有效 Mesh payload 存在工程内相对路径指向的旁置文件。
- [ ] 打开工程仅读取清单 / 索引；未打开页签的 Mesh payload 不进入进程内存。
- [ ] 按稳定 asset ID 与 revision 加载一个 Mesh，CXX 批量快照正确；已缓存页签激活不重读、不重复解析。
- [ ] STL 预览、导入提交与 Mesh sidecar 使用同一内容快照；源文件变化检测不会让清单记录与 Mesh payload 指向不同输入。
- [ ] 关闭页签释放其独占快照；关闭工程及 revision 失效后旧数据和迟到任务不能覆盖新资产。
- [ ] Sidecar 保存和清单提交失败时，上一份有效工程保持可打开；孤立新文件不会被当作已提交资产。
- [ ] 缺失、损坏、摘要不符、格式不支持和来源 revision 失配均返回稳定结构化错误，保留原始资产与工程可恢复性。
- [ ] Surface / Volume Mesh 持久化往返后领域值、单位、连接关系、分组和 revision 一致，校验结果可复现。
- [ ] 代表性测量覆盖保存、打开工程、首次激活、缓存命中及关闭释放；给出解析 / 解码时延、磁盘字节、CPU/RSS峰值与测量边界。
- [ ] Cargo 构建、workspace 测试、格式、lint、CMake/native 测试和适用的真窗口加载验收通过；所有 task 与架构说明一致。

## 验证计划与结果

尚未开始；依赖任务完成后按仓库聚合入口和性能场景验证。CPU / 磁盘性能基准不作为普通 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-29 | 架构评审：080 高水位测量、063 manifest / STL 资产、067 Mesh IR 与现有分层文档 | 区分解析成本、运行期 Mesh / VTK / GPU 成本；登记独立实施边界 | 已核对：080 的 608,218 面真实模型经 31 轮 VTK 切换峰值 RSS 已由约 1.65 GiB 降到 0.46–0.47 GiB；sidecar 可免重复 STL 解析，但仍需加载 Mesh 和创建 VTK / GPU 数据。本任务只规划 Mesh 旁置存储及按需加载。 |

## 风险与回退

sidecar 与清单是多文件资产事务；先发布资产、后提交索引，并使用不可变 revision 路径，避免中断写入破坏旧工程。损坏资产不能被当作成功加载，也不能在未确认的情况下覆盖来源数据。Mesh 格式需服务 Rust 领域值，不绑定 VTK；格式调整应明确版本与迁移任务。回退时恢复旧 schema / 读取契约需由单独 task 决定，不能提交无法读取已有资产的半成品。

## 决策与工作记录

- 2026-09-29：按用户提出的 STL → `panta-mesh::SurfaceMesh`，显示 / B-rep 修复 / 体网格三条路线，以及 `.panta` 元数据 + Mesh 索引、旁置 payload 的方案登记。评估结论：该存储能避免打开页签时重新解析 STL，并支持只加载所选 Mesh；不会消除读 Mesh、构造 VTK 数据或 GPU 资源成本。当前 080 高水位由 VTK 法线过滤的大额临时分配造成，已在 5ac1ca4 修复。B-rep 重建不保证恢复原始 CAD 参数曲面，具体修复和 Netgen 任务不在本任务范围。

## 完成摘要

未实施。等待 063、067、073、080 的相关契约收敛后，选择持久格式、事务语义、来源留存策略和测量门槛。
