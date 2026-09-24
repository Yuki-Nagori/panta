# 068 — QML 工程 / 任务 Dock 与 Layers Dock

- 状态：done
- 阶段：应用平台扩展
- 依赖：063（STL 工程数据与导入状态）、060（工程工作区参考）、062（Ribbon 页签组件）
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

按已确认的 HTML 参考调整 QML 左栏：上方工程 / 任务 Dock 是一个整体，内部上半部展示工程及 STL 零件树、下半部展示当前零件任务；下方是独立 Layers Dock，包含工具按钮、按导入状态显示的单一 Layers 图标页签和空内容区。未导入 STL 时保留 Dock 外框与工具栏，但隐藏 Layers 页签行；导入 STL 后显示该行。

Rust 工程服务已经保存完整导入记录，但 QML ViewModel 当前只发布最新一项。本任务同时让工程树展示所有已导入 STL；C++ 只把 Rust 返回的来源名称转成 Qt `QStringList` 供 QML 绑定，不派生新的 Study / 工程语义。用户逐个导入多个 STL 时，每项都追加显示，最新项仍是当前任务和视口状态。此任务不增加图层业务，也不改变 Rust 导入和持久化格式。

## 必读

- [QML 组件与声明式界面](../standards/qml.md)
- [QML 图标设计规范](../standards/icons.md)
- [分层规则](../standards/layering.md)
- [Qt / 原生领域边界](../architecture/native-domain-boundaries.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 将工程树区与零件任务区组成上部同一 Dock 的上下分区。
- 将原左下输出区域替换为 Layers Dock：8 个工具按钮、图标加 `Layers` 单页签、空内容区。
- 绑定导入记录列表以显示工程内所有 STL；未导入时隐藏 Layers 页签行，导入后显示。
- 新增 Mono `layers.svg` 图标并登记图标清单；给 Layers 文案与工具栏可访问名称补齐英文 / 简体中文字典。
- 按 QML / C++ ViewModel / Rust 分层，以 ViewModel 将 Rust 已保存的来源名称适配成 Qt 模型数据；不让 QML 读取文件或直接调用领域服务，也不在 Qt 侧复制领域状态。
- 将旧输出面板显示的持久错误信息转到状态栏，操作对话框继续显示其当前操作错误。

不包含：

- Layers 节点、可见性、编辑、选择或其他图层行为；面板内容保持空白。
- STL 几何、视口渲染、导入持久化格式或 Rust 服务行为变更。
- 单次文件对话框多选；多个 STL 通过重复导入追加到工程，批量选择作为后续交互决策处理。
- 改动右侧视口导航及 Ribbon 功能。

## 前置条件与待决策

- 现有 `ProjectViewModel` 已能查询 Rust 工程服务的完整 import records；实现需以其为只读 UI 数据源。
- 多个零件的选择与各自切换视口不是本次范围；初始默认项为最近成功导入的 STL。
- Layers 工具按钮只复刻图标入口，不接入动作。

## 实施步骤

1. 检查现有左栏布局、导入记录 ViewModel 和图标资源装配。
2. 调整 TasksPanel 内部上下分区并显示独立的 Layers Dock。
3. 让工程树列出全部导入 STL，更新图标、Theme、QML 资源及相关组件。
4. 执行适用的 QML 格式 / lint、构建和界面验收；同步验证记录及索引状态。

## 预计改动

- `qml/App.qml`、`qml/Panels/TasksPanel.qml`；新增 `qml/Panels/LayersPanel.qml`，清理被其取代的 `OutputPanel.qml` 使用点并保留错误可见性。
- `qml/icons/layers.svg`、`qml/CMakeLists.txt`（QML 文件清单按需登记）、`resources/i18n/panta-{en,cn}.pa`、`ai-docs/standards/icons.md`、`ai-docs/modules/qml-components-and-theme.md`。
- `native/bridge/src/project_view_model.hpp/.cpp`：把 Rust 工程服务返回的 STL 来源名称适配为只读 `QStringList`；不改变 Rust 公共接口或承担领域逻辑。
- 对应 ViewModel/QML 验收及本任务、索引记录。

## 清理与兼容例外

无兼容例外。Layers Dock 替代左栏原 `OutputPanel`；若输出错误展示仍由其他位置使用，需保留该职责并验证调用方，不留未引用的旧面板。

## 验收标准

- [x] Tasks 页签下工程树与当前零件任务呈上下两区，视觉上属于同一上部 Dock。
- [x] Layers Dock 外框与工具栏保持可见；无 STL 导入时隐藏 Layers 图标页签行，有导入记录时显示，导入后 TasksPanel 仍可见。
- [x] 通过重复导入加入多个 STL 后，工程树完整显示所有项目；最新导入仍对应现有视口快照（Rust 返回完整记录，ViewModel 映射名称列表，QML Repeater 展示）。
- [x] Layers Dock 包含工具按钮行、Layers 图标单页签及空白内容区；按钮无未声明的图层副作用。
- [x] QML 组件经 ViewModel 接收数据，无文件 I/O 或领域库调用；图标清单、资源和可访问名称完整。
- [x] 持久错误仍能在状态栏读取，创建 / 导入错误继续在对应对话框中展示（静态绑定检查）。
- [x] `native/CMakeLists.txt` 与目标链接实际使用的 Qt 运行时模块和架构文档白名单一致（Core、Gui、Qml、Quick、QuickControls2、QuickDialogs2、Svg）。
- [x] 适用的格式、qmllint、构建、相关 Qt 验收及真实窗口 / QML 状态证据通过；task 与索引证据一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | `cargo format`、`cargo format --check` | 聚合格式通过 | `cargo format` 完成；格式检查通过 |
| 2026-09-24 | `cargo lint qmllint --check` | QML lint 通过 | 通过。第一次真窗口启动发现缺少 `QtQuick.Controls` import，已修复后重跑通过 |
| 2026-09-24 | `cargo test --workspace` | Rust 测试及 native 聚合测试通过 | Rust 测试通过；CTest 56/56 通过。新增 `ProjectViewModelTest.PreviewsImportsAndPersistsLatestAsset` 多 STL 数据覆盖、`Qml.ThemeComponentParameters` 页签显隐覆盖，以及 `Qml.ShellModuleLoads` 中的 App 集成状态覆盖 |
| 2026-09-24 | `cargo build --locked` | Cargo 构建产物可启动 | 通过；`target/native/debug/app/panta-native` 启动并经临时 `PantaPreview.app` 绑定验收 |
| 2026-09-24 | 真实窗口：打开空 `.panta`，通过导入对话框重复导入两个 STL | 空工程隐藏 Layers 页签行；导入后显示，TasksPanel 保留并展示最新零件 | 通过。空工程时 Layers 工具栏与 Tasks 可见、页签行隐藏；导入后 AX 树及截图显示 Layers、Tasks、`first.stl` 和 `second.stl`，任务标题切换为 `second.stl`。临时工程和 wrapper 已清理 |

## 风险与回退

若 ViewModel 的展示角色不适合 QML 列表，优先新增稳定的只读模型 / QVariant 视图，不把 CXX Rust 类型暴露给 QML。实现遇到空间约束时优先使用既有 Theme 尺寸 token；不得通过隐藏溢出裁剪树项。回退时恢复旧面板组件引用并保留已验证的工程数据及 HTML 参考。

## 决策与工作记录

- 2026-09-24：按用户确认的 HTML 结构登记任务；Layers Dock 外框与工具栏常驻，未导入 STL 时隐藏 Layers 页签行，导入后显示。允许重复导入多个 STL，并把 ViewModel 当前只展示最新记录的问题纳入此任务。
- 2026-09-24：按用户要求对照 `native-domain-boundaries.md` 审查 Qt 与 Rust 职责；QML 仅呈现来源名称，C++ 仅做 Qt `QStringList` 类型适配，工程导入记录仍以 Rust 为权威。
- 2026-09-24：按用户反馈修复上下 Dock 抢占高度的问题；外层按 Tasks / Layers 的 58% / 42% 分配，Tasks 内工程树 / 任务区按 36% / 64% 分配。用户进一步明确条件作用于 Layers 图标页签行，不是整个 Dock；本轮同步修正 QML 与 HTML。
- 2026-09-24：补充 Layers 页签显隐组件测试、工程导入列表测试及 App 集成测试；覆盖空工程、连续导入多个 STL、切换到空工程和重开已导入工程。真实窗口复核同样确认导入后 TasksPanel 保留、Layers 行出现。

## 完成摘要

QML Dock 结构、STL 名称列表适配、按导入状态显示的 Layers 页签行、图标与文案已实现；格式、QML lint、Cargo 构建、完整工作区测试及真实窗口验收均通过。
