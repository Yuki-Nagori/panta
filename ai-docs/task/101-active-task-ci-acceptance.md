# 101 — 进行中任务 CI 验收复查（2026-10-02）

- 状态：done
- 阶段：文档维护
- 依赖：无
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-10-02 / 2026-10-02

## 目标与范围

按用户要求复查索引全部进行中任务，通过 `gh` 核对最新 CI；只关闭剩余条件仅为 CI 且对应提交已通过的任务。产品实现、真实窗口、读屏、硬件及性能验收仍按各任务要求保留。

## 必读

- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 验收标准

- [x] 对照全部进行中任务的未完成条件及完成摘要。
- [x] 核对最新 CI 提交与各 job，关闭符合条件的任务并同步索引。
- [x] 记录剩余条件；文档链接、状态和差异检查通过。

## 验证与工作记录

- 2026-10-02：开始复查 25 个进行中任务；最新 CI 对应当前 HEAD `a0f0872061627a969d60e422a2c945e8fc22f7b7`。

## 复查结果

以下为本次 CI 复查时的快照；后续 GUI 与读屏补验及状态变更见 [102](102-gui-and-screen-reader-acceptance.md)。

最新 [CI run 36890328640](https://github.com/Yuki-Nagori/panta/actions/runs/36890328640) 对应当前 HEAD `a0f0872061627a969d60e422a2c945e8fc22f7b7`，19 个 job 全部 success，无 job 跳过。macOS / Linux / Windows build/test 与 sanitizer、Rust coverage / Miri、native coverage / QML tests、format、audit 和各 lint 均成功。Linux prerequisite 步骤在 macOS / Windows 按平台条件跳过，不是缺失平台测试。

| 任务 | 本次结论 / 剩余条件 |
|---|---|
| 089 | done；Linux includes 修复版本验收通过。 |
| 098 | done；Windows / Linux 构建、测试及 sanitizer 与 Rust coverage 验收通过。 |
| 007 | 真实 Welcome / 空视口及 resize、DPR、隐藏恢复、重建生命周期仍待验收。 |
| 031、038 | SDK 运行部署、manifest / Linux 基线及 SBOM / provenance 闭环仍缺证据。 |
| 023 | Windows junction、UNC 行为仍需专门验证；普通三平台 CI 不代替这些场景。 |
| 053 | 宽窄窗口字样、实际几何 / 渲染与清理验收未齐备。 |
| 063 | 结构化错误 DTO 与真实视口生命周期验收仍未完成。 |
| 064 | 原生导航、定位器及 DPR / 隐藏恢复 / 重建和平台输入验收仍缺。 |
| 073 | DSL / FSM 完整失败与事件矩阵、激活生命周期及窗口联调验收仍有未勾项。 |
| 080 | 唯一剩余条件是 VoiceOver 真读屏朗读检查。 |
| 081 | 全局视觉、DPR / 无障碍、HTML / QML 一致性与完整组件性能映射尚未闭环。 |
| 091 | 验收框已勾选，但正文明确真实应用窗口验收未完成，不能仅据勾选关闭。 |
| 092、093、094 | 多零件选择、材料异步确认、Fill 布局与保存的最终真实窗口验收仍缺。 |
| 095、096 | Gate Location 行为与真实窗口、Results 工具栏真实窗口验收尚未齐备。 |
| 099 | 勾选标准之外，正文保留整应用逐弹窗视觉与其他平台窗口验收；硬件基准不能替代。 |
| 032 | Rust 100% 函数覆盖、C++ / 分模块防回退门禁及受控失败目标仍缺。 |
| 034、035 | 目标覆盖率与共享消费者收敛仍未完成，当前 coverage 绿灯只代表阶段下限。 |
| 042 | cold / incremental、Debug / Release 与 SDK / ABI 完整组合矩阵仍缺。 |
| 047 | POSIX stderr 断言、Windows SEH 子进程与默认 WER 验证仍缺。 |
| 048 | Criterion 可比较基线及火焰图 / hyperfine / Profiler / Massif 流程仍未完成。 |

保留历史验证记录；本次不将 CI 成功解释为真实图形、VoiceOver、性能或上游 SDK 发布验收，也不替未执行场景勾选验收项。

- 2026-10-02：完成索引与任务状态、改动文件的本地链接及 `git diff HEAD --check` 检查；均通过。本次仅更新文档，不运行产品测试或性能基准。

## 清理与兼容例外

无代码改动或兼容例外；只更新任务状态与证据。

## 完成摘要

复查 25 个进行中任务：089、098 的剩余 CI 条件已通过，任务与索引同步为 done；其余 23 项仍有非 CI 条件，保留 in-progress。
