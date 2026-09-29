# Moldfill 充填全流程演示 MVP

- 更新：2026-09-29
- 状态：固定默认值的充填演示 MVP 已实现；GUI 已获用户人工验收。
- 工作分支：`preview`；按用户要求不登记 ai-docs task，实施与验证记录维护在本文，使用说明另写根目录 `MVP使用说明书.md`。
- 分析对象：[`target/Moldfill_HITL_v1`](target/Moldfill_HITL_v1/README.md)，提交 `022cfa39a6c15a8da1f7f33063a38e8dc422d7ad`，版本 `0.3.1.dev1`。

## 1. 目标与结论

**这个 MVP 只做一件事：复用仓内固定样例，用户在 GUI 中全程使用默认值，完成“载入模型 → 生成网格 → 确认默认设置 → 运行充填 → 播放充填过程”。**

现有求解器已具备支撑这条流程的计算能力。分析时的主要缺口是 Panta 与求解器之间的进程调用、默认流程页面、结果读取和充填动画；本次已补齐这条固定样例路径，实际实现与验收见第 6～8 节。

首版不开放任意模型、材料库编辑、边界条件编辑、自动浇口优化、参数扫描或方案对比；不以工业精度认证、完整依赖治理、GPU 优化、保压/冷却/翘曲作为完成条件。工程化扩展留到演示闭环之后。

## 2. 这个文件夹里的代码已经能做什么

| 环节 | 已有能力 | 本次怎么复用 |
|---|---|---|
| 几何输入 | STL、专用中面网格 YAML、程序生成的平板等 | 固定使用 cover STL |
| 网格处理 | STL 摄入、单位处理、表面重划、质量报告 | 使用现有 MMGS/mmgpy 路径，固定 12 mm |
| 壁厚与配对 | 对壁匹配、厚度估算、节点捆绑 | 使用现有 surface-pair 路径和显式厚度带 |
| 材料与工艺 | Cross-WLF 等流变模型、热物性、流量及压力控制 | 原样复用 quick 算例引用的材料和工况 |
| 充填 | 一维、中面 2.5D、实体表面配对；压力、前沿、可选热计算 | 固定运行非等温 surface-pair |
| 浇口 | 自动推荐、指定节点、指定坐标 | 固定样例给出的坐标，不增加选点交互 |
| 结果 | 填充时间场、压力/温度等终态场、曲线、摘要 | 首版只接填充时间场和最少摘要 |
| 运行管理 | 独立 run 目录、JSONL 事件、日志、manifest、失败分类 | 直接读取已有产物与状态 |

这是一套薄壁近似的 2.5D 充填求解器。三维 STL 作为输入，不表示已经实现完整三维实体流动。保压、模具冷却、翘曲等目前主要是后续接口规划，不影响本次充填演示。

主要代码入口：[CLI](target/Moldfill_HITL_v1/src/moldfill/cli.py)、[充填内核](target/Moldfill_HITL_v1/src/moldfill/solver2d/__init__.py)、[网格重划](target/Moldfill_HITL_v1/src/moldfill/mesh/remesh.py)、[表面配对](target/Moldfill_HITL_v1/src/moldfill/mesh/pairing.py)、[运行存储](target/Moldfill_HITL_v1/src/moldfill/workflow/store.py)。

## 3. 直接复用哪些数据

### 主演示算例：cover 的快速非等温版本

| 用途 | 现存文件 | 本次核查 |
|---|---|---|
| 原始模型 | [examples/cover/cover.STL](target/Moldfill_HITL_v1/examples/cover/cover.STL) | 文件存在；与测试夹具逐字节一致 |
| 测试夹具 | [tests/fixtures/cover.STL](target/Moldfill_HITL_v1/tests/fixtures/cover.STL) | 与示例 STL 的 SHA-256 相同 |
| 默认配置 | [benchmarks/cover_noniso_quick.case.yaml](target/Moldfill_HITL_v1/benchmarks/cover_noniso_quick.case.yaml) | 引用的几何、材料路径均有效 |
| 默认材料 | [materials/polyflam-ripp-3625-cs1.yaml](target/Moldfill_HITL_v1/materials/polyflam-ripp-3625-cs1.yaml) | quick 与标准 cover 算例引用同一文件 |
| 标准细网格算例 | [examples/cover/cover.case.yaml](target/Moldfill_HITL_v1/examples/cover/cover.case.yaml) | 3 mm 网格、自动浇口；留作后续，不放进首版流程 |
| 轻量排障算例 | [cases/midplane-plate150x60x2.yaml](target/Moldfill_HITL_v1/cases/midplane-plate150x60x2.yaml) | 可用于结果读取/渲染排障，不替代 cover 全流程验收 |

本次对原始 cover STL 做了不依赖数值库的检查：164 个三角面，包围盒为 `202 × 6 × 152 mm`；按相同坐标合并后每条无向边均被两个面使用；有向体积约 `180.0032 cm³`，与仓内记录一致。这是基础完整性检查，不等于完整网格质量或求解验收。

示例与测试夹具的共同 SHA-256：

```text
4102bfe5acccad37ca3a0eed80ddd43997a7dcf42a518cfd58398fb53a5c227f
```

**选择 quick 算例的原因**：它保持标准 cover 的材料、熔体/模具温度、流量和压力限制，使用 12 mm 粗网格及固定浇口，适合缩短演示迭代。3 mm 细网格增加计算量，对展示流程没有必要。

仓内 [2026-09-25 cover 审计](target/Moldfill_HITL_v1/benchmarks/cover_fast_mode_audit_20260925.md) 记录了 12 mm、同材料工况下完成充填的结果，快速路径充填物理时间约 1.914 s。**这是模拟中的物理时间，不是程序运行耗时，也不是本次复现结果。** 历史完整结果目录没有随快照提供，GUI 应实际计算生成结果。

不要直接把 [W0 重型测试](target/Moldfill_HITL_v1/tests/test_w0_e2e_full.py) 的配置当作演示默认：该测试服务于运行存储验证，使用不同材料/等温设置，还显式覆盖配对冲突门禁。这里复用正式 quick 算例，不复用测试中的放行策略。

## 4. 默认参数与试算方式

下表记录 GUI 的默认演示参数，输入权威来源是 `cover_noniso_quick.case.yaml`。GUI 不提供逐项编辑控件；网格与充填步骤每次启动时都会读取该文件当前内容，因此可以直接修改 YAML 做参数试算。应保持 `geometry.file` 和 `material` 指向默认资产，避免首步预览与后续求解输入不一致。

| 项目 | 默认值 |
|---|---|
| 模型 / 单位 | cover.STL / mm |
| 求解路线 | surface-pair |
| 网格 | MMG，目标边长 12 mm |
| 厚度带 | 2.4～15 mm |
| 材料 | POLYFLAM RIPP 3625 CS1 |
| 浇口 | nearest，坐标 `[102, 3, 82] mm` |
| 熔体温度 / 模具温度 | 230 °C / 40 °C |
| 注射流量 | 94.7 cm³/s |
| V/P 切换 | 体积充填比例 99% |
| 压力上限 / 锁模力限制 | 140 MPa / 350 ton |
| 热计算 | 非等温，厚度方向 12 层 |
| 输出 | `store: final`、`vtk: true`，保持 quick 配置不变 |

改变网格参数后，需要从头重跑网格步骤；充填步骤会用本次生成的网格并读取当前工艺参数。实际生效配置可从本次 run 的 `manifest.json`、`case.snapshot.yaml` 和 `remesh_report.yaml` 核对。模型仍从固定默认示例载入，首版不提供文件选择器。

计算后端同样固定。已用以下 CPU 快速预设在本机跑通：

```text
MOLDFILL_DEVICE=cpu
MOLDFILL_PRESSURE_DEVICE=cpu
MOLDFILL_PRESSURE_SOLVER=direct
MOLDFILL_FAST_SOLVER=1
MOLDFILL_PICARD_MAX=4
MOLDFILL_CFL=0.9
MOLDFILL_DT_GROWTH=0.5
```

这些值由启动器统一设置，不显示成 GUI 调参项，也不继承用户终端中其他 `MOLDFILL_*` 数值覆盖。快速模式不是原始内核默认，应将这组设置明确记录为演示预设。首次跑通后固定下来，不同时尝试更换材料、网格、GPU 和算法。

## 5. GUI 的最小流程

| 步骤 | 用户操作与显示 | 背后行为 |
|---|---|---|
| 1. 载入模型 | 点击“载入默认示例”，视口出现 cover | 使用现有 STL 导入与视口能力，单位固定 mm |
| 2. 生成网格 | 点击“下一步/生成网格”，完成后看到重划网格 | 调用现有 `--remesh-only`，读取本次产物；不开放网格参数 |
| 3. 确认设置 | 显示材料、温度、流量和固定浇口，点击“开始充填” | 使用同一份 quick 配置，以 `--mesh` 传入上一步网格，避免重复重划 |
| 4. 运行 | 展示阶段、进度和日志；运行中禁用重复提交 | 启动独立求解器进程，读取事件；失败时显示原因并允许重试 |
| 5. 查看结果 | 自动显示充填时间云图，提供播放/暂停、时间滑块、重播 | 读取当前 run 的 `fill_pattern.vtk`，按充填时间场驱动回放 |

不增加向导中的决策分支。厚度带已经显式指定，材料与浇口固定，正常样例应一路推进。若默认样例被质量门禁拒绝，就记录并修复具体阻塞，不在 GUI 中增加“忽略错误继续”的默认放行。

“全流程”是上述从模型到结果的完整流程。计算期间先显示真实进度，计算结束后播放真实求解结果；首版不要求计算与三维前沿动画同时进行。

### 充填动画如何实现

现有 VTK 已导出每个节点的 `fill_time_s`。把它作为标量场，在播放时刻 `t`：

- `fill_time_s ≤ t` 的区域显示为已充填；
- 未到达的区域显示为未充填底色；
- 用标量阈值及插值表现单元内部的前沿；
- NaN 表示未充填，不能当作 0 秒；
- 滑块范围使用实际结果时间，播放速度可以与物理时间分开。

这是基于到达时间场的过程回放，足以展示“熔体从浇口逐步铺开”。不需要修改求解器逐帧导出，也不需要启用 `keyframes.npz`。该预设的 `store: final` 可以原样保留。压力/温度动画和更细的控制体积充填比例留到后续。

结果页首版只展示：填满状态、充填物理时间、充填时间云图及动画。峰值压力可作为已有摘要附带展示；不接剪切黏度、剪切应力等可选字段，避免扩大本次验证范围。

## 6. MVP 已补齐的实现

### 进程与结果链路

新增 [panta-solver](crates/panta-solver/src/lib.rs)，负责固定输入校验、步骤前置条件、后台作业、CLI 调用、事件读取及结果提交；[foundation 进程设施](crates/panta-foundation/src/process.rs) 负责子进程取消和回收。[CXX](crates/panta-ffi/src/lib.rs) 只转发服务及批量 DTO。

- 使用 `target/moldfill-venv` 的独立 Python 环境，不污染 Panta 自身的 Python 工具环境。
- 原始 STL 与默认材料文件使用已验证 SHA-256 校验；quick 配置允许编辑，run manifest 会记录实际读取的配置快照与哈希。
- 重划调用 `--remesh-only`；充填通过 `--mesh` 复用本次生成的网格，不重复重划。
- 每个操作使用 `target/moldfill-preview/gui-<pid>-<时间戳>` 新目录，保存固定设置和控制台输出。
- 工作线程增量读取完整 JSONL 行，支持 UTF-8 被分块截断、staging 目录完成后重命名；GUI 每 100 ms 拉取事件，不同步等待求解。
- 忙时禁止重复提交；取消与析构都回收直接启动的求解器进程。失败保留上一步有效输入，允许重试。
- [结果读取](crates/panta-solver/src/result.rs) 只接受当前目录中的一个已提交成功 run，检查模式、schema、产物 SHA-256、坐标单位和字段边界。`filled=false`、失败退出或坏文件均不能显示成功。

### GUI 与视口

[默认流程面板](qml/Panels/FillingPreviewPanel.qml) 经 [FillingPreviewModel](native/bridge/src/filling_preview_model.cpp) 使用 Rust 服务。入口为主窗口功能区下方的 **Filling MVP · default example**，无需建立工程；返回普通工作区不会卸载当前工程或演示。

[视口](qml/Panels/ViewportPane.qml) 显示原始模型、带边线的重划网格以及最终充填时间场。结果节点与三角面索引同步展开，不混用原始 STL 的节点顺序。灰色表示尚未到达；播放/暂停、滑块和到结尾后重播均由本地 C++ 状态驱动，不逐帧跨 Rust 或调用求解器。

[VTK 显示转换](native/visualization/src/vtk/surface_mesh.cpp) 按到达时间在三角面内线性切分已填/未填区域，保留完整轮廓和相机边界。完整回放约 10 秒，图例与时间轴标注模拟物理秒。含 NaN 的三角面保持灰色，不补造充填时间。

本次没有修改外部 Python 求解器的物理内核，也没有实现多核改造。完整使用步骤见根目录 [MVP使用说明书](MVP使用说明书.md)。

## 7. 完成标准与范围控制

- [x] 用户无需修改材料、浇口、网格或工艺参数即可进入并推进默认流程。
- [x] 默认 cover 已通过本机 CLI 基线完整求解；GUI 完整运行验收见下方记录。
- [x] 载入模型与重划网格已在真实窗口中看到，164 → 2,638 面，尺寸一致。
- [x] GUI 最终结果、滑块、暂停、重播通过真实窗口验收。
- [x] 运行中禁止重复提交；真实窗口停止后恢复同一步，支持重试。
- [x] 取消后重试使用新运行目录，保留并复用原重划网格。
- [x] 分别记录物理充填时间与运行耗时。

高级物理模型、任意 STL 稳健性、材料库、全面续算校验、工业精度评估、正式安装包及多核优化都不扩进这次演示 MVP。此实现依赖仓库旁的固定本地求解器快照；不声称已完成通用 MoldSolver 协议客户端或正式依赖供给。

## 8. 本次实际验证记录

验收环境：2026-09-29，macOS 26.3.1 arm64，Qt 6.11.2，native Debug；Python 3.14.7、NumPy 2.5.3、SciPy 1.18.1、PyYAML 6.0.3、meshio 5.3.5、mmgpy 0.17.0。所有命令从 Panta 根目录执行，求解器子进程的 cwd 为 `target/Moldfill_HITL_v1`。

### 默认算例 CLI 基线

固定环境见第 4 节。实际命令使用 `target/moldfill-venv/bin/python` 的绝对路径，执行 `-m moldfill run benchmarks/cover_noniso_quick.case.yaml --out ../moldfill-preview/baseline`。

产物：[基线 manifest](target/moldfill-preview/baseline/runs/20260929T140408Z-fab2ce/manifest.json)、[结果摘要](target/moldfill-preview/baseline/runs/20260929T140408Z-fab2ce/artifacts/result.yaml)。

| 指标 | 本机结果 |
|---|---|
| 终态 | `mode=solve`，`status=SUCCEEDED`，`filled=true` |
| 节点 / 三角面 | 1,319 / 2,638 |
| 物理充填时间 | 1.9150767935 s |
| 峰值压力 | 2.1465074803 MPa |
| 配对面积比例 / tie conflicts | 92.5821% / 0 |
| 体积误差 | 4.10755 × 10⁻⁷ |
| 完整 run 墙钟时间 | 214.97 s（manifest 起止时间）；求解阶段约 212 s |

### 自动化与窗口检查

- `cargo build --locked`：通过。
- `cargo test --locked --workspace`：通过，包含 66 项 CTest、qmllint 及 Rust 测试。新增校验覆盖步骤前置条件、重复提交、取消回收、分块 UTF-8、staging 重命名、失败/损坏 manifest、字段索引与单位、短射/非有限值，以及回放切分面积守恒。
- `cargo format --check`、`cargo lint`：通过，最终命令均以退出码 0 完成；日志保存在 `target/moldfill-preview/`。
- 原生窗口：已确认模型、网格、固定参数和真实进度；已执行停止并确认回到同一步重试。GUI 完整求解成功，终态显示 Filled、1.915 s、2.147 MPa。已检查最终云图、约 0.957 s 的部分充填区域及灰色未填区、播放、暂停与结尾重播。
- GUI 运行：[manifest](target/moldfill-preview/gui-49393-1790692164072286000/runs/20260929T142924Z-bb0850/manifest.json)。该次显示总耗时 344.7 s，期间同时运行了构建和质量检查；不能与独立 CLI 基线作公平性能比较。`result.yaml` 和 `fill_pattern.vtk` 与基线逐字节一致。
- 已检查取消后新运行目录与独立启动的新窗口；尚未额外做关闭再打开后的第二次完整 GUI 求解。用户明确反馈“我看没问题”，并要求不再测试 GUI，记录为用户人工验收通过；已停止窗口操作。用户正在试用的窗口与忽略目录下的临时 app wrapper 保留，未将 wrapper 纳入交付或包装成正式安装包。

### 手动性能场景

在现有 `tests/qml/project_docks_cpu_benchmark.cpp` 和 `tests/qml/project_docks_gpu_benchmark.cpp` 中增加 `measures_filling_preview`，实际装配默认流程面板、ViewModel 和独立的 [FillingPlaybackBar](qml/Panels/FillingPlaybackBar.qml)。固定预设只有 4 个步骤，不存在可变数量的参数编辑项。CPU 用 offscreen 测构造及布局；GPU 场景使用真实窗口，记录 Qt Quick 帧呈现间隔，不称为 GPU 内核耗时，也不代表 Python 求解性能。

构建目标 `panta_qml_cpu_benchmark` / `panta_qml_gpu_benchmark` 后执行各二进制的 `measures_filling_preview`。CPU 预热 1 次、采样 31 次，默认 4 步面板与回放条构造/布局的 p50 / p95 为 **1.545 / 1.899 ms**（offscreen）。GPU 场景计划预热 30 帧、采样 3 × 60 帧；最终包含面板和回放条的场景出现 Qt Quick 帧计数超时，**不记为性能验收通过**。较早只包含面板的场景曾测得 16.694 / 21.108 ms，但不能代替最终组合场景的结果。

包含 VTK 子窗口的早期场景和分离后的控件场景均遇到采样超时，根因尚未完成归因，不能断言只由 VTK 引起。回放条已提取为独立组件，控件基准只测 Qt Quick。用户随后明确要求停止 GUI 测试，本轮不再重跑性能窗口；这项未完成的帧率测量与充填流程功能验收分开记录，不作性能保证。

求解器样例及生成产物位于忽略的 `target/`。上述产物链接用于本工作区追溯，不会自动随 Panta 提交分发。

## 2026-10-01 main GUI 合并验证

- 状态：进行中；沿用本分支根目录记录约定。
- 范围：完成 preview 对 main 的变基，保留 main 工艺设置、日志、结果面板和原有文档页签，同时接入充填演示与回放条。
- 验证：待执行 Cargo 聚合检查；沿用用户要求，不启动真实 GUI 验收。
