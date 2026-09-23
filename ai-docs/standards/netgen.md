# Netgen 网格生成适配

查阅 / 更新日期：2026-09-20。状态：Netgen 6.2.2604 制品消费与 OCC 网格适配已由任务 010 完成实现并通过 macOS 全量验证，三平台 CI 复验中；制品升级项见文末。

适用于 `native/mesh/src/netgen/`（适配层与 `geometry/src/occt`、`visualization/src/vtk` 同构：公共契约在模块 `include/panta/<module>/`，第三方头收敛在适配层目录）。只接入网格生成，不因此把 NGSolve 求解能力带入本仓库。

## 官方依据

Netgen 上游说明其网格生成及几何内核连接能力。[Netgen upstream](https://github.com/NGSolve/netgen)

官方 OCC 教程展示几何到网格的流程，并说明 fuse、glue、compound 对内部界面/分区的不同影响。教程包含 Python/NGSolve 展示代码，不等同于本项目 C++ API 选型。[OCC Geometry Tutorial](https://docu.ngsolve.org/latest/i-tutorials/unit-4.4-occ/occ.html)

## 职责边界（2026-09-23 项目约定）

Rust 拥有网格领域数据、业务校验与任务提交；C++ adapter 实际调用 Netgen 生成算法，索引 / 分组 / 节点顺序归一化属于接口转换。 当前实现与迁移项见 [重库适配审计](../architecture/native-domain-boundaries.md)，执行 [分层规则](layering.md)；规划不代表已落地。

## 项目规则

- 所锁定版本的 C++ 接口、导出 targets 和 OCC 支持已由 010 以制品头文件与运行实测核实；不从 Python 示例推断 C++ 签名，也不照搬文档中的 NGSolve solver 依赖。
- Netgen 和 OCCT 使用经过验证的一致依赖组合。记录网格库构建选项，特别是 OCC、线程、Python 与 GUI 相关选项；不编入无用途的额外 UI。
- 输出立刻转换到自有 Mesh IR；明确索引起点、局部节点顺序、单元类型、材料区和边界标签的映射。
- 生成参数包含单位与范围；输入采用几何修订快照。生成失败或取消时不覆盖已有网格。
- 首期串行提交网格任务，只有确认库实例与全局状态的线程约束后才启用并发。无法安全中断的阶段应明确报告，不强杀线程。
- 为保证材料区边界，几何操作选择须保留所需内部面；不能把融合后的单一实体误当成多个独立区域。
- 先完成可追踪转换，再根据测量优化复制。显示用表面网格和求解用体网格各有用途，不能相互替代。

## 集成验证与消费约定（010 落地）

- 消费统一走 `panta_require_sdk("netgen")`：`find_package(Netgen)` 必须大写 N（Linux ext4 大小写敏感，038 实证）；与 OCCT 同 Release 成对发布（ABI 锁定，升级必须成对重建，不单换一侧）。
- 链接与运行库：`NetgenConfig` 以目标名引用 OCCT targets（无 INTERFACE 传递），OCCT 必须先于 Netgen 完成 find_package（imported targets 目录作用域，跨目录消费需在消费目录重新 `panta_require_sdk("occt")`）；OCCT 工具链全集须按制品 `NETGEN_OCC_LIBRARIES` 显式链接为可执行文件直接依赖——Linux DT_RUNPATH 不传递，仅靠 rpath 无法解析 libnglib 的传递性依赖。Windows DLL 在顶层 `bin/`（`native_test_env` 既有收集覆盖）。
- API 事实（以制品头文件与实测为准）：nglib v1 查询接口（`Ng_GetSurfaceElement`/`Ng_GetVolumeElement`）不返回面/区域索引——区域取体单元域号、边界取 FaceDescriptor `SurfNr`（OCC 面序号），经 libnglib 同源 C++ 头读取（`Ng_Mesh*` 即 `netgen::Mesh*`）；制品实现符号位于 `namespace nglib` 而头文件声明在全局命名空间，消费 TU 须把 nglib 头包含进 namespace 后限定调用；`mystdlib.h` 必须最先包含（netgen 头依赖其 `using namespace std`，污染仅限适配器 TU）。
- 行为与坑（v6.2.2604 实测）：`Ng_DeleteMesh` 在 OCC 网格化对象上释放悬空边界名指针（`Mesh::~Mesh` 手工 delete `region_name_cd` 名字数组；master 已重构为值语义修复），按官方示例 `ng_occ.cpp` 模式保留网格对象至进程结束，制品升级后恢复释放；Netgen 内部四面体方向与标准 FEM 正体积相反（box 实测 236/236 为负）且 `invert_tets` 在 OCC 路径未生效，转换层按有向体积符号归一化；`Ng_OCC_Load_STEP` 对不可解析输入抛 C++ 异常而非返回空，须在适配器边界捕获转结构化状态。
- 生成流程契约：`Load_STEP → SetLocalMeshSize → GenerateEdgeMesh → GenerateSurfaceMesh → GenerateVolumeMesh`，生成参数含单位与范围（mm），输出立刻转换为自有 Mesh IR 并过有效性检查，失败丢弃候选。

## 验证

用简单闭合实体生成小型体网格，检查索引、正体积/方向、边界分组和参数单位；构造失败场景验证旧资产不受影响。质量阈值属于项目/分析需求，不能把一次生成成功当成所有求解场景适用。

## 制品升级项（2026-09-20 登记）

固定 commit `3ee489c`（v6.2.2604）处于上游名字/描述符所有权重构中期：`Mesh::~Mesh` 手工 delete 名字指针数组与 FaceDescriptor 交错持有，OCC 网格化对象删除时触发 `pointer being freed was not allocated`（master 已重构为值语义修复，官方示例 `ng_occ.cpp` 不调用 `Ng_DeleteMesh`）。由 038 将制品重固定至含修复的上游版本（文档现版 6.2.2607）后，恢复适配器的网格对象释放并回归全部网格用例（TODO(task 010)）。
