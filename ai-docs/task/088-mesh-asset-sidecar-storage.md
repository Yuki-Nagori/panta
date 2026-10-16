# 088 — Mesh 资产旁置存储与按需加载

- 状态：in-progress
- 阶段：CAE 领域模块迁移
- 依赖：[063](063-stl-import-and-mesh-workspace.md)、[067](067-rust-mesh-domain-migration.md)、[073](073-fsm-dsl-and-import-state-machine.md)、[080](080-qml-viewport-document-tabs.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-29 / 2026-10-16

## 目标与背景

将 `.panta` 保持为小型工程元数据和资产索引；Surface / Volume Mesh 使用工程目录内可独立校验、带版本与修订身份的旁置资产文件。打开工程只读取清单与索引；首次打开某个文档页签时，Rust `panta-mesh` 按资产 ID 和 revision 加载权威 Mesh，缓存拥有内存的快照并批量交给 C++ 显示适配层。关闭页签释放该文档的运行期快照。

当前实现把导入后的 `SurfaceMesh` sidecar 作为工程内唯一权威资产，不保留原始 STL 副本。工程保留导入时选中的原始路径、文件名、格式、SHA-256、mtime 和 size 作为追溯诊断快照；来源文件不存在或路径失效不影响重开工程或加载 Mesh。打开页签只按工程内资产 ID/revision 加载 sidecar。当前开发阶段不迁移 schema 1–3；打开旧 schema 时明确拒绝且不修改工程文件，后续若出现实际迁移需求再单独设计。

当前 Rust `SurfaceMesh` 每个面存三个 `[f64; 3]` 顶点，展开数据为 72 字节 / 面；binary STL 约 50 字节 / 面。Mesh IR 不改变顶点语义，sidecar 在文件层用 Zstandard 压缩 payload，解码按 Mesh 计数流式构造领域对象，不额外分配完整未压缩副本。压缩率依赖实际几何数据，需同时测量文件字节和编解码耗时。

Mesh sidecar 的确定收益是工程保存独立于外部来源文件，并以工程内版本化 Mesh IR 作为唯一权威数据。它可能避免来源格式解析，但 sidecar 仍需读取、校验和构造运行期快照；对 binary STL，尚不能预设该路径更快。它不消除转换 VTK 数据或上传图形资源的成本，也不承诺降低显示内存 / GPU 驻留。性能基准应分开比较 STL 解析、sidecar 编解码、工程激活、缓存命中与 VTK；080 已通过跳过 STL 三角面上重复的 VTK 法线过滤器降低渲染切换高水位，不能把两种优化混为一项。

## 必读

- [应用平台、Rust 与工程存储](../architecture/application-and-storage.md)、[网格架构](../architecture/mesh.md)、[几何架构](../architecture/geometry.md)、[重库适配边界](../architecture/native-domain-boundaries.md)
- [063 STL 导入](063-stl-import-and-mesh-workspace.md)、[067 Rust Mesh IR](067-rust-mesh-domain-migration.md)、[073 异步激活](073-fsm-dsl-and-import-state-machine.md)、[080 文档页签](080-qml-viewport-document-tabs.md)
- [分层规则](../standards/layering.md)、[CXX](../standards/cxx.md)、[Rust](../standards/rust.md)、[测试](../standards/testing.md)、[性能测试](../modules/performance.md)
- [仓库文件规范](../standards/repository-hygiene.md)、[文档规范](../standards/documentation.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 设计可移植的 Mesh sidecar 布局和格式版本；`.panta` 清单只保存稳定 Mesh asset ID、revision、相对路径及验证所需的少量元数据。
- 由 `panta-mesh` 拥有 Surface Mesh 与体 Mesh 的持久化编解码、内容校验和领域校验入口；不把 C++ VTK 快照 DTO 当作持久领域模型。
- `.panta` Mesh 索引保存稳定 asset ID、revision 和用户可见 `display_name`；名称与原始来源路径、文件名、格式、SHA-256、mtime 和 size 分开，不能用显示名或源文件名充当资产身份。索引记录来源快照元数据，但不引用或内嵌原始 STL 文件。
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
- 当前 `SurfaceMesh` 是三角面顶点数据；sidecar 保持表示方式、精度、单位和连接关系。不要把单个 STL 的读时或压缩率直接外推为其他模型的性能或缓存预算。
- 原始 STL 不保留在工程内。导入时持久化所选原始路径、文件名、格式、源字节 SHA-256、mtime 和 size 用于追溯诊断；原始路径不作为重开工程或加载 Mesh 的依赖。Mesh sidecar 自身另有 payload 内容摘要。修改来源字段和显示名规则时，与 063 的 ImportRecord 契约保持一致。
- 明确文件格式版本、压缩 codec、压缩内容摘要算法、相对路径规则、写入持久性与损坏诊断；若格式或写入事务无法保证失败后保留上一份有效索引，暂不提交 schema 变更。
- 明确 sidecar 发布顺序：写入临时文件、校验并发布不可变资产，再提交引用它的新清单；崩溃留下未引用文件不得影响已提交工程。垃圾回收不删除当前或仍被任务引用的 revision。

## 本轮实施边界

- 当前工程仅读写 schema 4 和 ImportRecord v4；schema 1–3 明确报不支持，不做旧格式迁移。
- 新导入从 `PreparedStl` 已解析并换算为毫米的 Mesh 生成 sidecar，不将来源 STL 复制到工程。来源绝对路径仅是可失效的诊断快照。
- sidecar v2 使用固定小端字段、显式格式版本、Mesh 类型、单位、元素数量、Zstandard codec 标记及压缩 payload SHA-256；解码先验证头、长度与摘要，再流式解压并校验领域值。未知版本、codec 和损坏数据明确失败，不尝试 STL fallback。
- 清单索引字段独立于来源信息：稳定 asset ID、revision、工程相对 sidecar 路径、payload 摘要和格式版本。当前导入记录继续通过稳定 ID 向 UI 展示。

## 当前实现

- `panta-mesh` 定义 sidecar v2 的 Surface / Tet Mesh 编解码，使用显式小端字段、f64 坐标、毫米单位、元素计数、Zstandard 压缩与压缩 payload SHA-256；格式与 VTK、OCCT、Netgen 对象布局无关。
- `panta-import` 只负责 STL 来源快照及领域导入记录；`panta-core` 生成工程资产索引、管理写租约、发布 sidecar 并按需激活。`panta-ffi` 只投影工程相对 Mesh 路径，不重做解析或路径拼接策略。
- 新导入对预览时冻结的内容做确认复核，复用已解析网格换算毫米后写 sidecar；源字节在提取追溯元数据后释放，不复制到工程。
- 同步和异步导入都在工程写租约内发布 sidecar，再提交 schema 4；重复的同内容孤儿文件可安全复用，不同内容的崩溃残留使用有界恢复 ID，提交失败只清理本次新建的文件。
- 工程资产路径沿用 `PathService` 的相对片段和符号链接包含检查；导入投影使用真实工程根目录。sidecar 激活先检查固定头与压缩长度和摘要，再流式解压并验证 Mesh；解码不另建完整未压缩 payload 副本，文件格式不设 2 GiB 上限。预计展开 Mesh 数据超过 2 GiB 时，确认导入前提醒用户耗时、内存和磁盘空间风险，但允许继续。
- sidecar v2 的 Surface / Tet payload 使用 Zstandard 压缩，SHA-256 覆盖压缩字节；格式头标明 codec。实测数据见下方，压缩体积和耗时随模型变化，不能把单个模型的结果当作普遍压缩率。

## 实施步骤

1. 核对当前 ProjectManifest、ImportRecord、SurfaceMesh / TetMesh 与异步激活所有权；补测 STL 解析、Mesh 解码、磁盘读写和批量 CXX 快照成本。
2. 定义 MeshAssetRef、资产 revision、来源 revision、单位、格式版本、摘要及损坏 / 缺失错误；审查是否需要索引化 SurfaceMesh 表示，避免未经测量改变 067 的领域模型。
3. 实现 Rust 持久化读取与写入。STL 导入复用已解析的 `SurfaceMesh` 写 sidecar，不复制源 STL；新资产使用不可变 revision 路径，完成校验后先发布数据文件，再原子更新 `.panta` 清单索引。
4. 把 073 只读激活改为按 ID/revision 读取 Mesh sidecar；缓存命中复用已拥有快照，关闭页签 / 工程、修订变化与任务过期按 080 释放或拒绝。
5. 为 Surface / Volume Mesh 增加序列化往返、边界值、缺失/损坏/不支持版本、revision 失配、写入中断、并发读取与工程移动测试。
6. 对真实小 / 中 / 大输入比较原 STL 解析和 Mesh sidecar 首次加载、缓存命中、保存时间、磁盘字节和内存峰值；更新 063、067、073、080 与存储架构文档。

## 预计改动

- `crates/panta-mesh/`：领域 Mesh 编解码与校验，按实现需要拆分模块。
- `crates/panta-core/src/project/`：Mesh asset index、revision 引用和事务提交；当前 manifest 内联 STL `imports` 的关系按最终 schema 明确。
- `crates/panta-ffi/`、`native/bridge/`：异步加载结果和批量快照边界，保持 CXX 只声明适配契约。
- `ai-docs/architecture/{application-and-storage,mesh,native-domain-boundaries}.md`、相关 task 与 `ai-docs/task-index.md`。

## 清理与兼容例外

当前开发阶段不保留旧工程读取或迁移路径。清点并删除仅服务于 schema 1–3 的字段、分支和测试；这些版本明确拒绝且原文件保持不变。出现真实存量项目需求时另行登记迁移任务，不在此处保留双读兜底。

## 验收标准

- [x] `.panta` 只含工程元数据和 Mesh asset 索引；有效 Mesh payload 存在工程内相对路径指向的旁置文件。
- [x] 打开工程仅读取清单 / 索引；未打开页签的 Mesh payload 不进入进程内存。
- [x] 按稳定 asset ID 与 revision 加载一个 Mesh，CXX 批量快照正确；已缓存页签激活不重读、不重复解析。
- [x] STL 预览、导入提交与 Mesh sidecar 使用同一内容快照；源文件变化检测不会让清单记录与 Mesh payload 指向不同输入。
- [x] 导入不在工程中留下原始 STL 文件；保留所选路径、来源名、格式、SHA-256、mtime 和 size 后，删除或移动输入 STL 不影响工程重开与 sidecar Mesh 加载，失效路径也不产生阻断错误。
- [x] 关闭页签释放其独占快照；关闭工程及 revision 失效后旧数据和迟到任务不能覆盖新资产。
- [x] Sidecar 保存和清单提交失败时，上一份有效工程保持可打开；孤立新文件不会被当作已提交资产，后续导入可复用同内容 sidecar 或选择恢复 ID。
- [x] 缺失、损坏、摘要不符、格式不支持和来源 revision 失配均返回稳定结构化错误；schema 1–3 返回不支持错误且不改写原工程。
- [x] Surface / Volume Mesh 持久化往返后领域值、单位、连接关系、分组和 revision 一致，校验结果可复现。
- [x] Sidecar 头部先于 payload 校验，声明压缩长度与文件长度一致；单文件不设 2 GiB 上限。预计解压后的网格数据超过 2 GiB 时，确认导入前显示耗时、内存和磁盘空间非阻断提醒。
- [x] Zstandard 编码 / 解码保持 Surface 与 Tet Mesh 领域值，压缩摘要校验、未知 codec、截断流及多余解压内容均有回归；解码不构造完整解压 payload 副本。
- [ ] 代表性测量覆盖保存、打开工程、首次激活、缓存命中及关闭释放；给出解析 / 解码时延、磁盘字节、CPU/RSS峰值与测量边界。
- [ ] Cargo 构建、workspace 测试、格式、lint、CMake/native 测试和适用的真窗口加载验收通过；所有 task 与架构说明一致。

## 验证计划与结果

实施已覆盖功能和架构验收。真实 `cover.STL` 与 608,218 面的 `Large_Earth_Elemental.stl` 编解码基准已通过；压缩后体积显著低于 STL，但大型模型的 sidecar 解码慢于 STL 解析。还需工程级首次激活、缓存命中和 RSS 测量。CPU / 磁盘性能基准不作为普通 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-29 | 架构评审：080 高水位测量、063 manifest / STL 资产、067 Mesh IR 与现有分层文档 | 区分解析成本、运行期 Mesh / VTK / GPU 成本；登记独立实施边界 | 已核对：080 的 608,218 面真实模型经 31 轮 VTK 切换峰值 RSS 已由约 1.65 GiB 降到 0.46–0.47 GiB；sidecar 可免重复 STL 解析，但仍需加载 Mesh 和创建 VTK / GPU 数据。本任务只规划 Mesh 旁置存储及按需加载。 |
| 2026-10-16 | `cargo test -p panta-mesh --lib --offline`; `cargo test -p panta-import --lib --offline`; `cargo test -p panta-core --lib --offline`; `cargo test -p panta-ffi --lib --offline --no-run` | codec、来源快照、工程事务与 FFI 投影回归 | 通过：该轮 panta-mesh、panta-import、panta-core 测试及 FFI 测试目标编译成功；之后旧 schema 兼容测试已按维护者决定删除。 |
| 2026-10-16 | 压缩前基线：macOS arm64 Release，100k 合成 binary STL | 比较引入压缩前后 sidecar 字节数与编解码耗时 | 未压缩 v1：源 5,000,084 B，展开数据估算 7,200,000 B，sidecar 7,200,064 B；解析 p50 0.581 ms、编码 12.817 ms、解码 12.844 ms、热盘读+解码 13.187 ms、首次写入+sync 11.391 ms。此行是 2026-10-16 历史基线；当前实现为 v2，不再生成此格式。 |
| 2026-10-16 | `cargo test --locked --offline -p panta-mesh --release --test mesh_performance supplied_stl_and_sidecar_costs -- --ignored --nocapture` | 验证 zstd 对合成网格压缩、往返和编解码代价 | 100k 合成 binary STL：源 5,000,084 B，展开 sidecar 估算 7,200,064 B，zstd sidecar 340,685 B（展开尺寸的 4.73%）；解析 p50 0.582 ms、编码 7.133 ms、解码 4.425 ms、热盘读+解码 4.477 ms、首次写入+sync 6.050 ms。此规则网格高度可压缩，不代表一般 STL。 |
| 2026-10-16 | macOS arm64 Release；`PANTA_BENCH_STL=/Users/yuki/Downloads/cover.STL cargo test --locked --offline -p panta-mesh --release --test mesh_performance supplied_stl_and_sidecar_costs -- --ignored --nocapture` | 使用真实 STL 验证 v2 编解码值相同，并测量压缩体积和耗时 | `cover.STL` 164 面，源 8,284 B，展开 sidecar 估算 11,872 B，压缩 sidecar 1,268 B（展开尺寸的 10.68%，源文件的 15.31%）；解析 p50 0.003 ms、编码 0.059 ms、解码 0.029 ms、热盘读+解码 0.060 ms、首次写入+sync 9.279 ms。小文件首次写入受同步和计时粒度影响；该样本不足以代表大型几何性能或 RSS。 |
| 2026-10-16 | macOS arm64 Release；`PANTA_BENCH_STL=/Users/yuki/Downloads/Large_Earth_Elemental.stl cargo test --locked --offline -p panta-mesh --release --test mesh_performance supplied_stl_and_sidecar_costs -- --ignored --nocapture` | 真实大模型完整 parse / encode / decode 往返正确；记录空间与 CPU 成本 | 608,218 面，源 30,410,984 B（29.0 MiB），展开 sidecar 估算 43,791,760 B，压缩 sidecar 17,712,259 B（展开数据的 40.45%，源文件的 58.25%）；最新复测解析 p50 3.592 ms、编码 163.888 ms、解码 84.208 ms、热盘读+解码 85.526 ms、首次写入+sync 12.002 ms。该模型显示存储体积下降约 41.8%，但 sidecar 解码慢于本次测得的 STL 解析；不代表工程首次激活、缓存或 RSS。 |
| 2026-10-16 | `gh auth status`、`gh issue list --repo Yuki-Nagori/panta --state open`、`gh issue view 47` | 核对任务相关 issue 并按用户要求更新状态 | gh 已认证；issue #47 文档问题已在本分支修正并关闭。#44（task 119 文档同步）与 #36（分层与影响范围测试）仍开放，均不属于本任务，不作状态变更。 |
| 2026-10-16 | `cargo verify-changed` | Rust、C++、QML 聚合回归和代码质量检查通过 | 通过：workspace Rust 测试、native / QML CTest 77/77、格式和 Cargo lint 9/9 阶段；sidecar 往返、损坏流、目录边界、事务失败及来源文件独立性回归通过。 |
| 2026-10-16 | `cargo clippy --locked --workspace --all-targets -- -D warnings`; `cargo lint --changed --check`; `cargo format --check`; `git diff HEAD --check` | 代码质量门禁和格式检查通过 | 通过：Clippy、Cargo lint 9/9 阶段、格式与 diff 检查均成功。 |
| 2026-10-16 | `cargo verify-changed`（检测到公共 Cargo / native 边界变更，自动扩大为完整质量与测试范围） | 完整 Rust、native、QML 测试和适用质量检查通过 | 通过：L2 workspace Rust 包、native / QML CTest 77/77、格式检查及 Cargo lint 9/9 阶段均成功；`git diff --check` 通过。 |
| 2026-10-16 | 移除旧 schema 迁移后的 `cargo verify-changed` | 确认 schema 版本优先拒绝、当前 schema 校验和各受影响层仍通过 | 通过：选择器因公共 Cargo / native 边界变更扩大为完整范围；Rust workspace 测试、native / QML CTest 77/77、格式、Cargo lint 9/9 阶段全部成功。首轮发现并修复一个 FFI revision 溢出测试仍使用 schema 2 夹具的问题，之后重跑全项通过。 |
| 2026-10-17 | PR #48 CI（提交 `5c0467e`） | 三平台构建、受影响测试及代码质量检查全部通过后合并 | 通过并已 squash 合并至 `main`：macOS、Linux、Windows 构建与 L1/L2 Rust、native/QML 测试，格式、Clippy、依赖审计、分层、clang-tidy、include-cleaner、cppcheck、qmllint、CMake lint 均通过。 [Actions run 38927686098](https://github.com/Yuki-Nagori/panta/actions/runs/38927686098)。 |
| 2026-10-16 | 超 2 GiB sidecar 读取边界回归；`cargo test --locked -p panta-mesh -p panta-core --lib` | 不设文件大小硬上限；回归不实际分配超大 payload | 通过：mesh 13/13、core 120/120。core 用大于 2 GiB 的稀疏文件写入有效头部，并在 payload 分配前取消，验证读取路径接受该长度。2 GiB 导入提醒由面数估算，在确认前显示且不阻止继续。 |
| 2026-10-16 | 088 独立只读终审及修复复核 | 无路径越界、崩溃残留阻塞或来源摘要不一致 | 首轮评审中的路径投影、symlink 边界、孤儿恢复、超大 sidecar 与来源摘要问题已修复并增加回归。压缩实现终审补充截断 Zstandard 流和计数不匹配回归后复核通过。最终评审发现旧 schema 迁移未先验唯一 ID；按维护者决定删除迁移路径，并新增旧 schema 拒绝且原文件不变的回归。后续复审发现版本检查发生在当前 ImportRecord 解析之后；现改为先读版本封套，并以缺少当前必填字段的旧记录验证拒绝结果。删除无生产调用方的 `parse_stl_asset` 包装入口，单位换算测试改走实际导入准备路径。最新独立复审通过，未发现新问题；限制为评审者未重复全量测试，完整验证由主代理完成。2 GiB 提醒阈值、提示时机和翻译正确，超大 sidecar 测试不实际分配 payload。 |

## 风险与回退

sidecar 与清单是多文件资产事务；先发布资产、后提交索引，并使用不可变 revision 路径，避免中断写入破坏旧工程。损坏资产不能被当作成功加载，也不能在未确认的情况下覆盖来源数据。Mesh 格式需服务 Rust 领域值，不绑定 VTK；格式调整应明确版本与迁移任务。回退时恢复旧 schema / 读取契约需由单独 task 决定，不能提交无法读取已有资产的半成品。

## 决策与工作记录

- 2026-10-16：维护者决定不在工程中保留原始 STL 副本。导入后以 `SurfaceMesh` sidecar 为唯一权威资产；将原始选择路径、文件名、格式、SHA-256、mtime 和 size 保存为来源快照元数据，仅用于追溯与诊断，不能用于重开工程或读取 Mesh。
- 2026-10-16：维护者决定在本 PR 中加入 Zstandard 压缩，并用真实 STL 验证；采用 `zstd 0.14.2`，sidecar 格式升级至 v2，摘要覆盖压缩 payload。解码逐 Mesh 元素流式读取，避免同时持有完整压缩与未压缩 payload。
- 2026-10-13：与 063 对齐，Mesh 索引中的 `display_name` 不承担资产身份；原始来源元数据与 sidecar payload 自身摘要分开记录。063 的 ImportRecord 提供显示名与来源字段设计参考；088 保留显示名及来源快照，移除源 STL 文件副本和工程内副本路径。
- 2026-09-29：按用户提出的 STL → `panta-mesh::SurfaceMesh`，显示 / B-rep 修复 / 体网格三条路线，以及 `.panta` 元数据 + Mesh 索引、旁置 payload 的方案登记。评估结论：该存储能避免打开页签时重新解析 STL，并支持只加载所选 Mesh；不会消除读 Mesh、构造 VTK 数据或 GPU 资源成本。当前 080 高水位由 VTK 法线过滤的大额临时分配造成，已在 5ac1ca4 修复。B-rep 重建不保证恢复原始 CAD 参数曲面，具体修复和 Netgen 任务不在本任务范围。

## 完成摘要

实现及独立架构评审已完成；sidecar v2 保持 f64 三角面值语义并压缩 payload。当前仅支持 schema 4，schema 1–3 在解析当前记录前拒绝且不改写原文件。PR #48 的三平台 CI 已通过并合并；真实 `cover.STL`、608,218 面的 `Large_Earth_Elemental.stl` 与合成网格基准已记录。工程级首次激活、缓存与 RSS 测量仍待完成，因此任务保持 in-progress。不保留原始 STL 副本的策略由维护者于 2026-10-16 确认。
