# 010 — Netgen 接入与最小 Mesh IR

- 状态：in-progress
- 阶段：CAE 接入基础
- 依赖：[009](009-occt-adapter-smoke.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-20

## 目标与背景

证明 Netgen/OCCT 组合可生成并转换小型分析网格，固定后续业务使用的数据边界。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

## 必读

- [规范：netgen](../standards/netgen.md)
- [规范：occt](../standards/occt.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [架构：mesh](../architecture/mesh.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不开发网格 UI、全套质量优化和大规模性能系统。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 核对固定 Netgen 的 C++ 接口、OCC 支持和导出配置，确认实际 OCCT ABI 组合。
2. 建立最小 Mesh IR：坐标、单元类型/连接、区域和边界映射；明确索引与节点顺序。
3. 对简单闭合实体生成体网格，记录参数、单位和来源修订，复用任务/错误基础。
4. 增加索引、方向/退化、分组检查，失败时丢弃候选输出。

## 预计改动

native/mesh/（include/panta/mesh/ 公共契约 + src/netgen/ 适配器，与 geometry/src/occt、visualization/src/vtk 适配层同构，均已创建）、tests/cpp/mesh/ 测试、tests/fixtures/geometry 夹具复用、native/CMakeLists.txt 挂接（已改）、standards/netgen.md 适配层路径同步（已改）。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] 小模型生成体网格并转成自有 IR，索引范围与单元方向检查通过。
- [ ] 边界/区域映射可核验，参数单位明确，失败不会发布半成品。
- [ ] 依赖基线更新为实际验证结果，没有混入不需要的 NGSolve solver。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-20 | 探针（树外 CMake 工程 `panta_require_sdk("netgen")`，托管 CMake 4.4.3，macOS arm64） | 通过：4.2s 完成下载/SHA256 校验/staging 发布与 `find_package(Netgen)` + `ngcore`/`nglib` target 自检（缓存命中后离线幂等） |
| 2026-09-20 | Netgen 6.2.2604 制品布局与依赖核对（macOS dylib + Windows 归档清单 + NetgenConfig） | macOS：libnglib/libngcore `@rpath` 安装名，nglib 运行期跨 staging 依赖 OCCT 全部工具链（NetgenConfig 以目标名引用，无 INTERFACE 传递）；Windows：DLL 在顶层 `bin/`（native_test_env 既有收集已覆盖）；包配置 `NetgenConfig.cmake` 大写 N 与 manifest 一致 |
| 2026-09-20 | nglib API 头核对（本地 SDK headers） | `Ng_OCC_Load_STEP/SetLocalMeshSize/GenerateEdgeMesh/GenerateSurfaceMesh` + `Ng_GenerateVolumeMesh` 全流程可用；nglib v1 查询接口（`Ng_GetSurfaceElement/Ng_GetVolumeElement`）不返回面/区域索引——边界与区域映射改经 libnglib 同源 C++ 头（`Ng_Mesh*` 即 `netgen::Mesh*`，FD 的 `SurfNr` 为 OCC 面序号） |
| 2026-09-20 | `Ng_DeleteMesh` 崩溃取证（纯 nglib 流程探针 + lldb + malloc 栈日志，不含自有转换代码即复现） | OCC 网格化后的 mesh 在 `Ng_DeleteMesh`→`Mesh::~Mesh` 处 `malloc: pointer being freed was not allocated`（SIGABRT）；空 mesh 与手工 tet mesh 删除正常；官方示例 `ng_occ.cpp` 不调用 `Ng_DeleteMesh`；对比源码：固定 commit `3ee489c`（v6.2.2604）的 `~Mesh` 手工 delete `region_name_cd` 名字指针数组且与 FaceDescriptor 交错持有，master 已重构为值语义不再手工 delete——属上游过渡态缺陷。处理：适配器按官方示例保留网格对象至进程结束，TODO(task 010) 指向 038 重固定制品后恢复释放并回归 |
| 2026-09-20 | 方向约定实测（box_mm.step，maxh=5） | Netgen 内部节点顺序与标准 FEM 正体积相反：236/236 有向体积为负，总体积 -6000 mm³（恰为盒体积取负）；nglib `invert_tets` 参数在该制品 OCC 路径未生效（设置后分布不变）。处理：转换层按有向体积符号归一化（负则交换末两个节点索引），IR 契约保持标准正向 |
| 2026-09-20 | 异常边界实测 | `Ng_OCC_Load_STEP` 对不可解析输入抛 C++ 异常（"Couldn't load OCC geometry"）而非返回空；适配器边界捕获转 `kGeometryLoadFailed`，其余阶段异常兜底转 `kInternalFailure` 并丢弃候选 |
| 2026-09-20 | `cargo build --locked` + `ctest -R "NetgenMesher\|MeshIr"` | 通过：NetgenMesher 5/5（box 摘要：region 1、边界分组 6、体积 6000±60 mm³、校验通过；maxh 10 vs 3 单元数递增；缺失文件 kFileNotFound；垃圾输入 kGeometryLoadFailed 不崩溃；失败后恢复生成）+ MeshIr 8/8 |
| 2026-09-20 | `ctest` 全量 + `cargo test --locked --workspace` + `cargo lint` + `cargo format --check` | 通过：ctest 49/49（既有 41 项不受影响）；workspace 18 个测试目标全 ok；lint（clippy/machete/cmake/qmllint/clang-tidy/includes/cppcheck）零告警 |
| 待 CI | push 后三平台 CI（018 矩阵） | Linux/Windows 首次 configure 下载 netgen 制品并编译本模块；Linux 验证 OCCT 工具链全集直接依赖 + 构建策略（DT_RUNPATH 非传递），Windows 验证 DLL 加载。此前 010 验收项不勾选、任务保持 in-progress |

## 风险与回退

索引/节点顺序转换错误或内部面丢失会产生看似正常但不可用的网格；对小模型验证体积与边界映射后再扩大范围。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-20：依赖 009 已完成（OCCT STEP 适配冒烟三平台 CI 全绿），依赖条件满足，状态转 ready。Netgen 制品与 OCCT 成对发布（Release `sdk-occt-netgen-8.0.1-6.2.2604`，`find_package(Netgen)` 大写 N 约束见 031 manifest 注释）；运行期依赖同平台 OCCT 资产，加载路径按 009 的消费模式处理。
- 2026-09-20（结构与消费）：模块布局与 007/009 适配层同构——`native/mesh/include/panta/mesh/` 公共契约（零第三方类型）+ `src/netgen/` 唯一 Netgen 头位置；单静态库 `panta_mesh`（IR 独立性在公共头类型层保证，不拆 core 子库；安装导出待业务任务）。OCCT imported targets 为目录作用域，mesh 目录内再次 `panta_require_sdk("occt")`（缓存 marker 命中零下载）；OCCT 工具链全集按制品 NetgenConfig 显式链接并作为可执行文件直接依赖——Linux DT_RUNPATH 不传递，传递性依赖无法经可执行文件 rpath 解析。
- 2026-09-20（API 选型）：nglib v1 的 OCC 生成流程 + libnglib 同源 C++ 头读取（区域=体单元域号、边界=FD `SurfNr`，压缩为 0 起连续 ID）；不从 Python 示例推断 C++ 签名，全部以本地 SDK 头与实测为准。制品符号位于 `namespace nglib` 而头文件声明在全局——按制品 ABI 把 nglib 头包含进 namespace 后限定调用；`mystdlib.h` 的 `using namespace std` 是 netgen 头的上游契约，污染仅限适配器 TU。
- 2026-09-20（缺陷与取舍）：`Ng_DeleteMesh` 在 OCC 网格化对象上触发上游过渡态缺陷（证据见验证表），按官方示例模式保留网格对象至进程结束并登记 TODO(task 010)（解除条件：038 将制品重固定至含修复的上游版本）；方向归一化在转换层完成（`invert_tets` 未生效实测）；两者均不改变 IR 契约。
- 待记录：三平台 CI 复验；多区域/内部面几何（fuse/glue/compound 语义）与边界映射表持久化归网格业务任务。

## 完成摘要

未完成。macOS arm64 已实现并通过全量本机验证（Netgen/OCCT 成对消费、Mesh IR 转换、方向归一化、异常边界、失败恢复）；三平台 CI 复验通过、验收项核实后，与索引一起标 done。遗留：网格对象释放恢复（制品升级后）、边界映射表持久化（业务任务）。
