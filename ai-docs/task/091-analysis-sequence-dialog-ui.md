# 091 — Analysis Sequence 选择弹窗

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：050, 068, 081, 090
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-09-30

## 目标与背景

点击 Plan tasks 中当前显示为 Fill 的分析序列行，打开 Select Analysis Sequence 弹窗。Fill 是默认序列值；任务 ID、组件、SVG 和 HTML symbol 使用 Analysis Sequence 命名。布局参考用户截图，样式沿用现有弹窗和 Theme。

## 范围与非目标

- HTML 参考和 QML 弹窗包含十个序列选项、当前候选值、OK / Cancel / More。
- 初始为 Fill；弹窗维护候选选择，OK 发出确认请求，Cancel、关闭和 Esc 放弃候选改动。More 暂禁用。
- 新增双语文案并同步图标资源、清单和引用。
- UI 接收目录和已确认快照；领域目录、工程持久化及 Mesh 类型展示由 [092](092-plan-analysis-sequence-and-mesh-settings.md) 接入，不校验网格适用性或启动求解器。

## 验收标准

- [x] 点击分析序列任务行打开弹窗，默认选中 Fill。
- [x] 支持鼠标与键盘选择；确认更新展示，取消保留已确认值。
- [x] 组件和图标使用 Analysis Sequence 语义，旧 task-fill 引用清理。
- [x] 文案、构建资源和任务索引一致，适用检查通过。

## 验证计划与结果

| 日期 | 检查 / 场景 | 实际结果 |
|---|---|---|
| 2026-09-30 | `cargo format --check`、`cargo lint qmllint --check`、`cargo lint`、`cargo build --locked` | 通过；lint 的本机锁服务需允许 TCP 监听 |
| 2026-09-30 | `cargo test --locked --workspace` | 通过，CTest 66/66；QML 回归覆盖任务行入口、方向键选择、Enter 确认、取消及再次打开恢复已确认值 |
| 2026-09-30 | `node --check ai-docs/qml-html/shell.js`、源码检查、用户 HTML 截图反馈 | 脚本检查通过；分析序列按钮改为整行宽度，悬停背景避开左侧状态列。最终 HTML 未做自动截图验收 |
| 2026-09-30 | Qt 6.11.2 / macOS arm64 / Basic / offscreen，CPU `measures_analysis_sequence_construct` | 十个选项，31 次采样；空白 / 面板 / 完整弹窗 p50/p95 分别为 20.1/38.7、606.7/893.7、2117.4/2926.3 µs；包含构造与布局，不代表 GPU 性能 |
| 2026-09-30 | GPU `measures_analysis_sequence_frame_presentation` | 场景登记并构建，启动报 `Cannot create window: no screens available`，未取得帧呈现数据 |
| 2026-09-30 | 构建后使用临时 PantaPreview.app 连接真实应用 | 两次连接均报 `timeoutReached`；临时 wrapper 已删除，真实窗口验收待补 |

CPU 场景实例化 `AnalysisSequencePanel` 与 `AnalysisSequenceDialog`，GPU 场景呈现选择面板。因真实窗口验收尚未完成，任务保持 in-progress。

## 决策与工作记录

- 2026-09-30：HTML 和 QML 已实现选择、确认与取消；修复 HTML 悬停行宽，完成命名与资源清理。More 保持禁用；工程持久化由 092 接入。
- 2026-09-30：保留 Fill 作为默认显示值，统一功能命名为 Analysis Sequence；初版提供 QML 会话内选择，现由 092 替换为 Rust 权威及确认信号。

## 清理与兼容例外

删除 task-fill SVG 和正式 symbol，替换为 task-analysis-sequence；无兼容例外。
