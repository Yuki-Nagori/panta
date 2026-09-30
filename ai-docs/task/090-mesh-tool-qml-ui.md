# 090 — Mesh Tool QML 界面

- 状态：done
- 阶段：应用平台扩展
- 依赖：050（HTML 参考）、068（工程 / 任务 Dock）、081（视觉语言）
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-09-30

## 目标与背景

从 Plan tasks 的 `Create Mesh...` 进入 Tools 页签，以网格工具替换该页签原有的 Information 内容。以 [HTML 参考](../qml-html/mesh-tool/mesh-tool.html) 为控件和分区底稿，颜色、字号、间距及控件样式沿用现有 QML Theme。工程树和 Plan tasks 都属于 Tasks 页签，切到 Tools 后一起隐藏。

## 必读

- [QML 规范](../standards/qml.md)
- [国际化模块](../modules/internationalization.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

- HTML 参考从 `Create Mesh...` 进入时选中 Tools；整个页签内容区展示工具。
- 直接选择 Tools 时仍显示 Information；只有 `Create Mesh...` 打开网格工具，再点 Tools 回到 Information。
- QML 中点击 `Create Mesh...` 切换到 Tools 并显示 Mesh Tool；布局包含操作选择、Mesh / Help / Preview / Cancel、两个上层复选框、General 区、全局边长及两个默认勾选项。
- 新增 QML 可见文案登记到英 / 中文 `.pa` 目录。
- 本任务只处理 QML 内部显示与页签切换。Mesh 生成、预览、帮助、取消的业务信号和参数持久化另行接入。

## 前置条件与待决策

沿用 068 的上方工程 / 任务 Dock 布局及已有 Plan tasks 数据；网格工具参数目前仅为静态初始值，不能作为领域数据源。

## 实施步骤

1. 完成并检查 HTML 参考的分区和页签状态。
2. 新增 QML Mesh Tool 视图，并将 Plan tasks 行的 UI 导航接入 Tools 页签。
3. 补全翻译目录，检查 QML 格式、lint 和构建结果，记录实际验证。

## 预计改动

`ai-docs/qml-html/` 参考件、`qml/Panels/TasksPanel.qml`、新增 QML 视图及其 CMake 登记、`resources/i18n/panta-{en,cn}.pa`、QML 行为测试和手动 CPU / GPU 基准场景、本任务和索引。无 Rust / C++ 接口、工程格式或网格数据变更。

## 清理与兼容例外

Tools 默认 Information 保留；进入 Mesh Tool 后在同一页签内替换其内容。保留 Shared Views 信息视图。无废弃项和兼容例外。

## 验收标准

- [x] 直接点 Tools 显示 Information；点击 `Create Mesh...` 后选中 Tools，Mesh Tool 占用整个页签内容区，Tasks 页的工程树和 Plan tasks 不再显示；再点 Tools 回到 Information。
- [x] 操作、按钮、选项、General 和全局边长均在 HTML / QML 中呈现；窄 Dock 的 QML 内容高度超过视口并可滚动。
- [x] 新增 UI 文案使用 `qsTranslate` 并在英 / 中文目录中有对应条目。
- [x] 没有调用 Mesh 后端或写入参数；QML、资源清单和任务状态一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-30 | HTML 源码、`node --check ai-docs/qml-html/shell.js`、用户截图反馈 | Tasks / Tools 切换和样式结构正确 | 脚本检查通过；按用户截图反馈调整了操作选择框、F2 与 General 左对齐。浏览器自动化禁止打开本地 `file:` 页面，最终 HTML 未做自动截图验收 |
| 2026-09-30 | `cargo format --check`、`cargo lint qmllint --check`、`cargo lint` | 格式与质量入口通过 | 均通过；`cargo lint` 需在允许本机锁服务 TCP 监听的受控环境执行 |
| 2026-09-30 | `cargo build --locked`、`cargo test --locked --workspace` | 构建和聚合回归通过 | 均通过；CTest 66/66。新增 QML 测试覆盖 Tools 默认 Information、Create Mesh 进入工具、再次点 Tools 返回 Information，以及窄 Dock 滚动 |
| 2026-09-30 | Qt 6.11.2 / macOS arm64，Basic 样式；`panta_qml_cpu_benchmark measures_mesh_tool_construct` | 记录空白与 Mesh Tool 构造开销 | offscreen、31 次采样：空白 p50/p95 14.5/19.1 µs，Mesh Tool 1002.3/1273.0 µs；不代表 GPU 帧耗时 |
| 2026-09-30 | `panta_qml_gpu_benchmark measures_mesh_tool_frame_presentation` | 真窗口帧呈现消融 | 当前命令会话无可用屏幕，进程报 `Cannot create window: no screens available`，未取得 GPU 数据 |

## 风险与回退

Dock 高度可能不足以容纳全部选项，工具内容使用滚动容器。回退时恢复 Tools 提示内容和 Create Mesh 行的原静态状态。

## 决策与工作记录

- 2026-09-30：用户确认旧截图只作布局参考，采用现有主题；点击 Create Mesh 切到 Tools 并以工具内容替换 Information。工程树和 Plan tasks 只在 Tasks 页显示；业务信号延后。
- 2026-09-30：Review 时保留 Tools 的默认 Information，并让页签显式点击重置网格工具状态；HTML 参考也按相同规则展示。修正图标清单测试对既有 `taskRow.modelData.icon` 写法的识别，新增行为与窄屏滚动回归。CPU 构造场景已登记；GPU 场景已登记但本机窗口环境不可用。

## 完成摘要

HTML 参考、QML 静态 Mesh Tool、页签入口和双语文案已完成。格式、lint、构建和聚合测试通过；CPU 基准通过。Mesh 动作与参数持久化仍按本任务范围暂不接线；GPU 真窗口数据因当前环境没有屏幕而缺失。
