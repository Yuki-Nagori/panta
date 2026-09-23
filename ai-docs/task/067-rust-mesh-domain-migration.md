# 067 — Rust 统一 STL 解析与 Mesh IR 领域校验

- 状态：planned
- 阶段：CAE 领域模块迁移
- 依赖：[010](010-netgen-adapter-smoke.md)、[063](063-stl-import-and-mesh-workspace.md)、[066](066-native-domain-boundaries.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-23 / 2026-09-23

## 目标与背景

按 066 的职责审计，将重复的 STL 解析和 Mesh IR 领域校验收敛到 Rust，让导入预览、工程资产与 VTK 显示使用一致的网格数据和规则。以真实消费者建立 `panta-mesh`，保持 C++ 对 OCCT / Netgen / VTK 的适配职责。

当前 STL 在 `crates/panta-core/src/project.rs` 与 `native/visualization/src/vtk/stl_mesh.cpp` 各解析一次；Mesh IR 与校验在 `native/mesh/`。本任务包含两段完整迁移：先打通 STL 数据链，再将 native Mesh IR 的领域校验迁入 Rust 并接通现有 Netgen 输出消费者。

## 必读

- [重库适配与 Rust 领域模块](../architecture/native-domain-boundaries.md)、[网格架构](../architecture/mesh.md)、[可视化架构](../architecture/visualization.md)
- [分层](../standards/layering.md)、[CXX](../standards/cxx.md)、[跨语言边界](../standards/ffi.md)、[Netgen](../standards/netgen.md)、[VTK](../standards/vtk.md)
- [Rust](../standards/rust.md)、[注释](../standards/comments.md)、[测试](../standards/testing.md)、[验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)、[仓库文件](../standards/repository-hygiene.md)、[文档](../standards/documentation.md)、[提交](../standards/commits.md)

## 范围与非目标

包含：

- 新建有生产消费者的 `crates/panta-mesh/`，承载表面 / 四面体网格数据、STL 解析、摘要和领域校验；统一单位、索引、节点顺序、分组与错误契约。
- Rust 工程服务复用解析结果或同一内容修订的缓存；预览、导入、保存后重开与显示保持一致。文件读取 / 导入事务由 Rust 服务编排，纯网格解析不依赖 Qt 或工程服务。
- 以批量 DTO / 明确所有权的快照经 CXX 向 C++ 交付网格；VTK 后端仅构造显示数据对象，不再读取并解析 STL。
- 将 `validate_tet_mesh` 等领域规则迁入 Rust；现有 Netgen 转换输出通过明确的桥接交给 Rust 校验，所有实际消费者使用同一权威规则。C++ 保留长度 / 溢出 / 索引转换安全检查和 Netgen 节点顺序归一化。
- 同步清理旧 parser、重复领域校验、失效路径接口、调用点、构建注册与测试；补充行为、集成及性能验证。

保留与限制：

- 相机动画、六面体命中、输入映射、窗口、定时器和 GPU 生命周期继续由 C++ 负责，不增加逐帧 FFI 往返。
- GPU 继续用于 WebGPU 绘制；相机计算仍在 CPU。本任务不开发 GPU 计算内核，不以 CPU 基准推断 GPU 帧率。
- 五个领域 crate 按真实功能建立：本任务落地 `panta-mesh`；`panta-geom`、`panta-bc`、`panta-material`、`panta-visualization` 的职责和触发条件沿用 066，不创建空壳。若迁移确需新增其他领域功能，先拆任务再扩大范围。
- 不重写 OCCT / Netgen 核心算法，不实现完整 STEP 资产编辑、材料 / Study 功能或新渲染后端。Netgen 制品释放问题继续由 010/038 跟踪，不以本任务掩盖其未解决状态。

## 前置条件与待决策

- 010 / 063 尚有未完成验收，任务先保持 planned；开始前核对可用代码、现有测试和剩余阻塞，明确本任务需要的基线已满足后同步状态，不将依赖任务默认视为完成。
- 实施前定义 ASCII / binary STL 的唯一接受规则，包括编码、额外字段、截断、空模型、非有限值及数量溢出；不能直接把现有宽松 parser 当成完整规范。
- 明确预览后源文件变化的处理：内容快照或修订校验必须保证提交资产与显示一致；重新打开工程时使用包内资产而非原始外部文件。
- 明确 CXX 数组布局、坐标精度、索引宽度、所有权、释放与异步有效期；领域层不得依赖 `panta-ffi` 或反向依赖 `panta-core`。Netgen 校验桥接需同时保持 Cargo / CMake 构建依赖无环。
- 实施前审查工程格式与 parser 版本影响；需要改变持久化契约时同步版本、所有消费者和明确错误，不暗中保留双 parser 兼容路径。

## 实施步骤

1. 固定现有合法 / 非法 STL、Mesh IR 与工程往返行为，记录两套解析差异和迁移前性能输入；核对实际测试入口。
2. 定义 Rust 网格类型、单位 / 分组契约与结构化错误，建立 `panta-mesh` 并接入工程服务，统一解析与摘要。
3. 接通修订化网格快照到 CXX / C++ / VTK 的数据链，覆盖导入、资产切换、重开和资源重建；完整替换后删除 C++ STL 文件解析。
4. 迁移 Mesh IR 领域校验，接通现有 Netgen 输出与测试消费者；删除 C++ 重复规则。仅供转换的 native DTO 不得继续承担第二套领域模型。
5. 运行行为、集成、真实窗口与性能验证，复核 C++ 导航边界；更新架构、使用说明和清理记录。
6. 完成验收后同步任务与索引，按用户要求 commit，不 push。若分阶段提交，每个提交均包含可用的数据链和相应清理、测试、文档。

## 预计改动

- 待创建：`crates/panta-mesh/` 及相应测试，按仓库测试规范确定位置；Cargo workspace / 依赖锁按实际变更同步。
- 现存：`crates/panta-core/src/project.rs`、`crates/panta-ffi/`、`native/bridge/` 的导入与显示数据适配。
- 现存：`native/visualization/include/panta/visualization/`、`src/cae_viewport.cpp`、`src/vtk/stl_mesh.*` 和 `src/vtk/vtk_viewport.cpp` 的数据提交 / 转换；对应 QML 消费者同步。
- 现存：`native/mesh/include/panta/mesh/mesh_ir.hpp`、`src/mesh_ir.cpp`、`src/netgen/netgen_mesher.cpp` 与 CMake 注册。
- 现存：`tests/cpp/mesh/`、`tests/cpp/visualization/`、桥接 / 工程测试及小型夹具；相关架构文档与任务索引。

## 清理与兼容例外

- 删除 VTK 内 STL 读盘、binary / ASCII parser 和仅服务它们的辅助代码；Rust 工程模块不保留独立 parser 副本。
- 删除迁移后的 C++ 领域校验与失效调用；替换路径驱动显示接口时同步清理 QML、ViewModel、测试、配置和文档。
- 保留 C++ VTK 数据转换、Netgen 库适配和必要边界安全检查；不以“去重复”为由删除防越界检查或回归断言。
- 默认无兼容例外。必要例外按生命周期规范登记 `COMPAT(...)`、验证和实际清理任务，不留下备用旧实现。

## 验收标准

- [ ] `panta-mesh` 被生产代码使用，纯 Rust 领域代码不依赖 Qt / VTK / CXX / `panta-core` / `panta-ffi`，crate 与构建依赖无环。
- [ ] ASCII / binary STL 使用唯一解析规则；覆盖空、截断、编码、额外字段、非有限坐标与数量溢出，预览 / 导入 / 显示不再各自解析。
- [ ] 预览后源文件变化、包内资产重开、网格切换与加载失败有确定行为；失败 / 取消 / 迟到修订不覆盖有效工程和显示状态。
- [ ] Rust Mesh IR 校验覆盖索引、非有限值与运算溢出、重复 / 退化单元、方向及区域 / 边界引用与完整性；现有 Netgen 输出实际进入该校验。
- [ ] FFI 批量数据的布局、所有权、释放与缓冲区有效期有测试；转换错误可报告，不暴露第三方原始类型。
- [ ] C++ 不再持有重复 parser / 领域规则；导航与 GPU 生命周期职责不变，无逐帧 Rust 往返，无空 crate、死代码或未登记兼容路径。
- [ ] 实际构建、相关 Rust / C++ / QML 测试、格式 / 静态检查与真窗口导入、缩放、右键旋转、六面体过渡回归通过并记录平台限制。
- [ ] 记录固定输入、构建配置、机器、重复次数、解析 / 校验 / FFI 转换耗时与数据复制 / 内存成本；对照迁移前结果解释变化，不设置未经依据的绝对时间门槛。
- [ ] 文档、接口、构建与 task 一致，索引同步；按约定 commit，不 push。

## 验证计划与结果

实现前先核对已有命令与注册目标；新增 crate / 目标未落地前不宣称其测试已可运行。性能使用可分发且可重放的输入，纯 CPU 解析 / 校验和真实 GPU 显示分别记录。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-23 | 仓库根目录；任务文档本地链接、编号 / 索引状态、围栏与 `git diff --check` | 规划登记一致 | 通过：2 份文档、95 个本地链接，067 索引唯一且状态一致；差异空白检查通过 |
| — | Rust STL / Mesh IR 单元与非法输入测试 | 单一规则及边界可重复验证 | 未实施 |
| — | Netgen 输出 → Rust 校验；工程导入 → CXX → VTK；工程往返与生命周期回归 | 生产消费者完整接通 | 未实施 |
| — | 迁移前后 CPU 性能、复制 / 内存成本与真窗口验收 | 功能无回归，成本可解释 | 未实施 |

## 风险与回退

解析规则统一可能改变旧宽松输入的接受结果，必须给出诊断并用夹具固定决定；不能静默产生不同网格。大数组跨 FFI 的重复复制、GUI 阻塞和缓冲区失效需实际测量与验证。Rust 校验迁移可能引入 native 测试链接回环，必须在删除旧实现前接通完整构建链。

失败时不修改既有工程资产；确需回退时回退自洽提交，不保留并行旧 parser 或第二套校验。真实平台验证缺口记录实际范围，不能用无头测试代替 GPU 窗口验收。

## 决策与工作记录

- 2026-09-23：按用户要求登记实施任务，承接 066 的职责结论；本次只创建规划和索引，不开始迁移、不创建 crate。后续按 STL 数据链、Mesh IR 校验两步落实，其他领域 crate 随实际功能另建任务。规划提交已检查链接、索引与差异空白；本次无生产代码或废弃项，无兼容例外。

## 完成摘要

未实施。任务规划已建立；代码迁移、测试、性能与真窗口验收均待完成。
