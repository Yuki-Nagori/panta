# 077 — 进行中任务状态盘点

- 状态：done
- 阶段：文档维护
- 依赖：无
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

核对任务索引中所有 `in-progress` 项，结合各自验收标准、验证记录、实际实现和可查 CI 证据，确保任务状态真实、剩余工作明确，已完成项不滞留在进行中。此次盘点仅修改任务文档与索引，不实施功能代码。

## 必读

- [任务索引](../task-index.md)
- [文档与示例维护规范](../standards/documentation.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

覆盖盘点开始时索引中的全部 `in-progress` 任务；对已具备验收证据的任务更新 task 完成摘要、验证记录、验收勾选和索引状态。仍未完成的任务保留 `in-progress`，并核实其未完成项和解除条件。顺手收敛过长或重复的验证时间线，保留决定状态所需的代表性证据和关键根因。

不修改产品实现、测试逻辑、CI 配置或 SDK 制品；不以计划、推断或未经验证的外部状态关闭任务。

## 前置条件与待决策

任务索引和各 task 文件均为本仓库当前工作依据。涉及 GitHub CI / Release 的状态仅按可查询到的 run 和资产证据记录；无法核验的内容保留为待确认。

## 实施步骤

1. 从任务索引枚举所有 `in-progress` 项及对应 task 文件。
2. 对照验收、验证证据、完成摘要和代码 / CI 状态，判定关闭条件是否满足；只进行文档状态收敛。
3. 更新对应 task 与索引，验证引用、状态一致性和差异范围。

## 预计改动

- `ai-docs/task-index.md`
- 盘点出的 `ai-docs/task/NNN-*.md`
- 本任务记录

## 清理与兼容例外

无废弃实现或兼容例外。

## 验收标准

- [x] 盘点范围内的每个 `in-progress` task 都经过验收与证据检查，并记录处理结果。
- [x] 仅在全部验收具备证据时标记 `done`；未完成任务保留准确状态、剩余工作和阻塞条件。
- [x] 所有状态变化与任务索引同步，task 文件元数据、完成摘要和验证记录自洽；过长的历史验证时间线已按阶段收敛。
- [x] 本次变更仅包含文档，文档链接、代码围栏与差异空白检查通过。

## 盘点结果

盘点范围是本次开始时索引内的 14 个既有 `in-progress` task。每项对照其验收清单、完成摘要、最近验证记录及可查实现/CI 证据；没有把 CI 通过等同于真实窗口验收，也没有把规划中的消费者接入记为完成。

| 编号 | 处理 | 检查结论与剩余条件 |
|---|---|---|
| [007](007-vtk-quick-viewport.md) | 保留 in-progress | 三平台 CI 修复和 CTest 通过不证明硬件窗口渲染。仍需实际 WebGPU 窗口验收空视口/测试球体，以及 resize、高 DPI、隐藏恢复、关闭重开和资源释放。 |
| [010](010-netgen-adapter-smoke.md) | 改为 blocked | 网格生成、IR、边界映射与三平台 NetgenMesher/MeshIr 测试均通过；固定 Netgen v6.2.2604 对 OCC mesh 调用 `Ng_DeleteMesh` 可复现 invalid free。解除条件是 038 提供含上游修复的新 SDK，并完成安全销毁及生命周期回归。 |
| [031](031-prebuilt-native-dependencies.md) | 保留 in-progress | manifest 与校验/缓存路径已落地，run 360018 提供三平台 SDK 消费和测试证据。仍需 007/009/010 安装运行时分发闭环、Linux glibc 有效基线，以及消费侧 SBOM/provenance 自动化。 |
| [038](038-native-sdk-artifact-production.md) | 保留 in-progress | VTK WebGPU 与 OCCT/Netgen 三平台 Release 管线和归档已验证；仍需 OCCT/Netgen manifest/生产链最终复核、消费集成冒烟、Linux glibc 基线及 SBOM/provenance。 |
| [023](023-cross-platform-paths.md) | 保留 in-progress | Rust 路径规则与 macOS C++ host 测试已有证据；Windows junction、UNC 行为及 C++ PathHost 三平台验证记录仍缺。 |
| [063](063-stl-import-and-mesh-workspace.md) | 保留 in-progress | 首期导入与工程恢复链路已实现；结构化错误 DTO、QML lint/真实窗口与视口生命周期、异步任务化和实际 Create Mesh 流程尚未闭环。 |
| [064](064-vtk-navigation-and-orientation.md) | 保留 in-progress | 导航、坐标轴/六面体后端与 offscreen 回归已实现；仍需 macOS 真窗口 resize/DPR/隐藏恢复/关闭验收，并在可用环境校准 Windows/Wayland 输入路径。 |
| [032](032-cross-language-quality-gates.md) | 保留 in-progress | run 360018 确认当前 89%/92% Rust gate 与聚合格式检查通过；任务目标还包括 Rust 100% 函数覆盖、C++ line/branch 与模块回归门禁、覆盖报告和受控失败等。 |
| [034](034-rust-panta-artifact-parser.md) | 保留 in-progress | parser、TS/QM 生成和 CLI 主体已实现；三平台相同规范化快照、关键失败夹具、032 覆盖要求及 022/025/030 消费方收敛尚未完成。 |
| [035](035-pa-formatter-and-validator.md) | 保留 in-progress | formatter/validator 的核心验收已完成；032 覆盖率门禁和 022/025/030 共享消费方接入仍未完成。 |
| [042](042-unified-llvm-toolchain.md) | 保留 in-progress | 统一 LLVM 供给与编译器选择已落地，run 360018 是正向 CI 证据；完整三平台 clean/incremental、Debug/Release、SDK/toolchain 组合矩阵仍缺。 |
| [047](047-crash-signal-logging.md) | 保留 in-progress | POSIX 子进程信号记录/重发及三平台 Cargo 测试通过，但 POSIX stderr 尚无捕获断言；Windows 目前只测 filter 安装和路径，尚缺子进程触发 SEH 后验证异常码、stderr/日志及 WER 行为。 |
| [048](048-performance-testing.md) | 保留 in-progress | 入口测量和热路径 review 已记录；Criterion 可比较基线、火焰图/hyperfine/QML Profiler/Massif 可复现说明、性能文档引用一致性及 CI 前后对照仍待完成。 |
| [058](058-ci-regression-after-project-package.md) | 改为 done | run 360018 全绿；format、89%/92% coverage gate、macOS sanitizer `Qml.ShellModuleLoads`（未跳过）和三平台项目包/native 测试均通过，所有验收有证据。 |

同一 CI 收尾文档也关闭了 [070](070-googletest-sdk-ci.md)、[071](071-googletest-sdk-consumption.md) 与 [075](075-ci-windows-gtest-and-qt-lint.md)；对应 task 和索引已同步为 done。时间线整理仅针对重复的逐轮排障与重跑记录；外部资产校验、导致路线变化的根因和最新验收证据保留。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | 索引 14 个既有 `in-progress` 项逐项检查验收、摘要和最新验证记录；核对 GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | 每项有证据支持的状态结论；已完成项关闭，剩余项明确条件 | 12 项保留 in-progress；010 因固定 Netgen SDK 安全销毁缺陷改 blocked；058 因 run 36001859191 全部验收通过改 done。详情见上表；070/071/075 同步关闭。 |
| 2026-09-24 | 本地文档检查：17 个变更 Markdown 文件、259 个本地链接、24 张表格、围栏及 task-index 状态；`git diff --check`、`git diff --cached --check` | 引用有效，格式、状态和差异范围一致 | 全部通过；未发现失效本地链接、围栏不配对或表格列数不一致；差异仅含 `ai-docs/`。 |

## 风险与回退

任务记录可能滞后于实际代码或 CI；关闭任务前需检查对应实现与验证证据。若无法证明全部验收条件成立，保留原状态并记清剩余工作。

## 决策与工作记录

- 2026-09-24：用户要求检查所有 `in-progress` task，并要求本次提交只包含文档更新。
- 2026-09-24：按逐项验收和证据审查更新 010/058 状态，校正 task-index 中 007/010 的过期总览；未用三平台 CI 通过替代真窗口、全覆盖率或完整矩阵验收。
- 2026-09-24：CI run 36001859191 为本轮已关闭 CI 任务提供三平台证据；提交范围限定为 task、索引与规范性文档。
- 2026-09-24：用户要求简化过长时间线；将合并重复逐轮日志纳入本任务范围，保留关键根因、代表性 run 和最新状态证据。

## 完成摘要

完成 14 个既有进行中任务的状态审计：058 已完成，010 因 Netgen OCC mesh 销毁的上游缺陷阻塞，其余 12 项保留进行中并明确剩余验收条件。相关 SDK CI 任务 070/071/075 也根据 run 36001859191 关闭；QML 新增性能场景规则由任务 078 补入规范。审计中冗长的排障时间线已按阶段整合并保留关键证据。所有状态已同步任务索引；本次提交范围仅为文档。
