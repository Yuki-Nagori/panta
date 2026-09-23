# 067 — Rust 统一 STL 解析与 Mesh IR 领域校验

- 状态：done
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

- 新建有生产消费者的 `crates/panta-import/` 和 `crates/panta-mesh/`，承载表面 / 四面体网格数据、STL 解析、摘要和领域校验；统一单位、索引、节点顺序、分组与错误契约。
- `panta-import` 负责格式识别、STL 来源读取、预览缓存、变更检测、选项校验和导入记录构造；`panta-core` 的工程服务只负责资产与清单的原子提交。Rust 工程服务复用解析结果或同一内容修订的缓存；预览、导入、保存后重开与显示保持一致。文件读取 / 导入事务由 Rust 服务编排，纯网格解析不依赖 Qt 或工程服务。
- 以批量 DTO / 明确所有权的快照经 CXX 向 C++ 交付网格；VTK 后端仅构造显示数据对象，不再读取并解析 STL。
- 将 `validate_tet_mesh` 等领域规则及网格总体积计算迁入 Rust；现有 Netgen 转换输出通过明确的桥接交给 Rust 校验，所有实际消费者使用同一权威规则和摘要结果。C++ 保留长度 / 溢出 / 索引转换安全检查和 Netgen 节点顺序归一化。native 对应类型明确命名为 DTO，不能形成第二套权威 Mesh IR。
- 同步清理旧 parser、重复领域校验、失效路径接口、调用点、构建注册与测试；补充行为、集成及性能验证。

保留与限制：

- 相机动画、六面体命中、输入映射、窗口、定时器和 GPU 生命周期继续由 C++ 负责，不增加逐帧 FFI 往返。
- GPU 继续用于 WebGPU 绘制；相机计算仍在 CPU。本任务不开发 GPU 计算内核，不以 CPU 基准推断 GPU 帧率。
- 五个领域 crate 按真实功能建立：本任务落地 `panta-mesh`；`panta-geom`、`panta-bc`、`panta-material`、`panta-visualization` 的职责和触发条件沿用 066，不创建空壳。若迁移确需新增其他领域功能，先拆任务再扩大范围。
- 不重写 OCCT / Netgen 核心算法，不实现完整 STEP 资产编辑、材料 / Study 功能或新渲染后端。Netgen 制品释放问题继续由 010/038 跟踪，不以本任务掩盖其未解决状态。

## 多格式扩展设计

本任务落地 `panta-import` 的 STL 导入入口与 `panta-mesh` 的表面网格解析 / Mesh IR；同时核对未来 STEP / IGES 的接口边界，不在 067 实现它们。`panta-import` 管理格式识别、来源快照、选项、解析分发与诊断；`panta-core` 管理工程修订、资产写入和原子提交。`panta-mesh` 负责 STL 与网格领域数据；后续 `panta-geom` 负责 STEP / IGES 的几何身份，经 C++ OCCT adapter 实际读取和转换。两者通过工程资产引用关联，避免把 B-rep 强行压成 STL 三角网格。具体契约见[多格式导入扩展](../architecture/native-domain-boundaries.md)。

STL 实施时保留可识别的数据类别与格式元数据，导入分发不绑定到 VTK 的文件路径。STEP / IGES 接入另建纵向任务，按实际 OCCT API、格式诊断和业务消费者验证；不预造没有消费方的 STEP / IGES parser 或泛型解析器框架。

## 前置条件与待决策

- 010 / 063 尚有未完成验收，010 / 063 仍在收尾；已核对现有 Netgen 输出、工程导入、CXX 和视口消费路径可用，本任务开始实施。依赖任务未因此标为完成。
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

- 待创建：`crates/panta-import/`、`crates/panta-mesh/` 及相应测试，按仓库测试规范确定位置；Cargo workspace / 依赖锁按实际变更同步。
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

- [x] `panta-mesh` 被生产代码使用，纯 Rust 领域代码不依赖 Qt / VTK / CXX / `panta-core` / `panta-ffi`，crate 与构建依赖无环。
- [x] ASCII / binary STL 使用唯一解析规则；覆盖空、截断、编码、额外字段、非有限坐标与数量溢出，预览 / 导入 / 显示不再各自解析。
- [x] 预览后源文件变化、包内资产重开、网格切换与加载失败有确定行为；失败不覆盖有效工程和显示状态。同步 UI 清除过期预览。
- [x] Rust Mesh IR 校验覆盖索引、非有限值与运算溢出、重复 / 退化单元、方向及区域 / 边界引用与完整性；现有 Netgen 输出实际进入该校验。总体积由 Rust 计算并随校验结果返回。
- [x] FFI 批量数据的布局与所有权有测试；转换错误可报告，不暴露第三方原始类型。坐标快照是拥有内存的值对象，无借用缓冲区跨调用。
- [x] C++ 不再持有重复 parser / 领域规则；导航与 GPU 生命周期职责不变，无逐帧 Rust 往返，无空 crate、死代码或未登记兼容路径。
- [x] 聚合 build、workspace 测试、格式与 lint 通过；真窗口启动并由用户完成手动导入验收。视口缩放、右键旋转和过渡动画属 065 行为，本任务保留其既有 C++ 实现。记录了本机窗口接管和其他平台覆盖限制。
- [x] 记录固定输入、构建配置、机器、重复次数、解析 / 校验 / FFI 转换耗时与数据复制 / 内存成本，并说明旧实现校验规则不同，基线不可直接视作等量实现。
- [x] 文档、接口、构建与 task 一致，索引同步；按约定 commit，不 push。

## 验证计划与结果

实现前先核对已有命令与注册目标；新增 crate / 目标未落地前不宣称其测试已可运行。性能使用可分发且可重放的输入，纯 CPU 解析 / 校验和真实 GPU 显示分别记录。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-23 | 仓库根目录；任务文档本地链接、编号 / 索引状态、围栏与 `git diff --check` | 规划登记一致 | 通过：2 份文档、95 个本地链接，067 索引唯一且状态一致；差异空白检查通过 |
| 2026-09-23 | `cargo build --locked` | 聚合构建成功 | 通过；debug 构建包含 CXX / CMake native 图 |
| 2026-09-23 | `cargo test --locked --workspace` | Rust、C++ / CTest、QML lint 与集成行为通过 | 通过；workspace 聚合入口退出码 0，CTest 56 项通过，project 公共 API 集成测试 15 项；性能基准按设计忽略 |
| 2026-09-23 | `cargo format --check` / `cargo lint --check` | 聚合格式与静态检查通过 | 均通过。完整 lint 显示 8 个阶段；clang-tidy 覆盖 36 个自有翻译单元（最多 8 个 worker），成功路径静默，因此阶段提示避免误判停滞。未按测试类型排除文件；单文件实测的两个复杂测试分别约 6.9 秒、3.6 秒，没有异常耗时证据 |
| 2026-09-23 | `cargo build --locked --release -p panta-launcher` | 发布配置构建可用 | 通过；release launcher 链接成功 |
| 2026-09-23 | `cargo test --locked -p panta-mesh --release --test mesh_performance -- --ignored --nocapture` | 固定输入的独立阶段成本 | 通过：100k binary STL（5,000,084 B）解析中位数 0.993 ms；展平 900k 个 f64 坐标（7.2 MB）0.392 ms；10k tetrahedra、40k nodes、10k boundary faces 校验 1.579 ms。CPU / release；Rust 1.98.1；macOS 26.3.1 arm64。性能测试各自重复采样并报告中位数 |
| 2026-09-23 | `target/native/release/mesh/panta_mesh_ir_benchmark` | 测量 native DTO → CXX → Rust 校验全路径 | 通过：10k tetrahedra 全路径中位数 1.835 ms（21 样本）；native 数组打包、CXX 转换与 Rust IR 构造都包含在内，不能解释为纯 FFI 调用开销 |
| 2026-09-23 | 消融对照：旧 Rust STL 摘要解析、旧 C++ Mesh IR 校验与新实现 | 解释迁移成本与校验规则变化 | 旧 Rust 摘要路径 0.700 ms；其只计算摘要，不构造完整 SurfaceMesh，不能与新解析作等量比较。旧 C++ 10k tetrahedra 校验 0.235 ms（clang -O3），但未包含新 Rust 的重复单元、tetrahedron-face 成员、边界面积与更多有限值检查；新校验能力更强，不能把差值归因于语言 / FFI。对应固定输入生成器和当前阶段基准保存在 Rust / C++ benchmark 文件中，旧实现取自 HEAD 临时重建，不保留死实现 |
| 2026-09-23 | 真实窗口：直接启动构建产物，经临时 app wrapper 接入 computer use，用户手动导入 | 检查 GPU 窗口和导入对象可见 | 用户手动完成导入；UI accessibility 树显示 `Part (0001-mug.stl)`。窗口交由用户操作后未继续自动控制；仅覆盖本机当前会话，其他 OS / GPU 未验 |
| 2026-09-23 | `git diff --cached --check`、`git diff --check` | 暂存与未暂存差异无空白错误 | 通过 |

## 风险与回退

解析规则统一可能改变旧宽松输入的接受结果，必须给出诊断并用夹具固定决定；不能静默产生不同网格。大数组跨 FFI 的重复复制、GUI 阻塞和缓冲区失效需实际测量与验证。Rust 校验迁移可能引入 native 测试链接回环，必须在删除旧实现前接通完整构建链。

失败时不修改既有工程资产；确需回退时回退自洽提交，不保留并行旧 parser 或第二套校验。真实平台验证缺口记录实际范围，不能用无头测试代替 GPU 窗口验收。

## 决策与工作记录

- 2026-09-23：按用户要求登记实施任务，承接 066 的职责结论；本次只创建规划和索引，不开始迁移、不创建 crate。后续按 STL 数据链、Mesh IR 校验两步落实，其他领域 crate 随实际功能另建任务。规划提交已检查链接、索引与差异空白；本次无生产代码或废弃项，无兼容例外。

- 2026-09-23：核对现有链路后开始实施。选择 Rust 工程服务缓存解析结果并将网格快照批量交给 C++ 显示层；源文件在预览后变化时拒绝导入，导入资产使用已验证的字节快照。Netgen 保留库调用与转换，Rust 接手领域校验。

- 2026-09-23：按用户补充，明确 Rust 统一导入编排与格式专属后端：STL 归 `panta-mesh`，STEP / IGES 归规划中的 `panta-geom` + OCCT adapter；不把三种格式压成通用文件解析器。

- 2026-09-23：按用户决策新增独立 `panta-import`，将 STL 格式识别、来源快照、预览复用、选项与记录构造移入该 crate；`panta-core` 的 `project.rs` 拆出导入事务、清单持久化和测试文件。已将 VTK 文件解析替换为工程网格快照，当前仍在完成 native 校验与验收。
- 2026-09-23：按用户给出的 crate 判断标准补充 Rust 规范：重库边界、变化频率、消费者范围与逻辑规模共同决定拆分；独立 `panta-import` 不直接链接重库。
- 2026-09-23：按用户指定将工程服务公共 API 的黑盒测试归档到根 `tests/rust/project.rs`，由 `panta-core` 显式注册；私有名称校验保留源码内单元测试。同步更新测试目录规范，避免根测试路径没有 Cargo 入口。STL 解析新增非有限边界和单位换算溢出校验，导入失败保持原网格与修订。
- 2026-09-23：按用户要求在根 `AGENTS.md` 固定 Cargo 聚合入口与 computer use 真窗口验收方式；完成聚合检查、真窗口用户验收和阶段性能 / 消融对照。review 暂存区时同步修订过期的 native Mesh IR 头文件与架构说明，使之明确 C++ 只转换 DTO，Rust 拥有校验和总体积规则。验证发现 CMake 重配会暂时清除未重建 GTest target 的 POST_BUILD 清单，Cargo native 集成入口已在 all_qmllint 重配后构建 `all`，避免 CTest 把这些用例误报为 `NOT_BUILT`。完整 lint 增加可见阶段进度；clang-tidy 保持全部自有测试覆盖，未因测试文件复杂而排除。

## 完成摘要

067 已完成。ASCII / binary STL 解析、导入会话与工程事务归入 `panta-import` / `panta-mesh`；工程服务提交经 CXX 发布拥有内存的 mesh snapshot，VTK 只转换 snapshot。Netgen 仍由 C++ 调用和归一化输出，Mesh IR 领域校验统一由 Rust 实施。移除了 VTK 文件 parser 与 C++ 重复校验规则，补充单元、根 `tests/rust` 集成、桥接、VTK 和手动性能基准。

聚合构建、workspace 测试、格式与 lint 均通过；56 个 CTest 和 15 个工程 API 集成测试通过。Release 基准记录了解析、快照展平、纯 Rust 校验和 native→CXX→Rust 全路径；与旧校验对照时注明了规则差异。真实窗口通过 computer use 启动，用户在窗口中完成手动导入，本机 UI 树显示导入 part；自动化未接管用户操作，其他平台 / GPU 未覆盖。无兼容例外。提交按用户要求仅 commit，不 push。
