# 114 — View 工具栏内容

- 状态：done
- 阶段：应用平台扩展
- 依赖：060, 062, 064, 080
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-12 / 2026-10-12

## 目标与背景

根据维护者提供的 Moldflow View 工具栏截图，为已打开工程的 View 页签建立 HTML 参考并迁移到 QML。按既有工作流先交付 HTML 原型供维护者查看，确认后再做 QML。

## 必读

- [HTML 参考工作流](../qml-html/README.md)
- [Ribbon 拆分](062-ribbon-tab-components.md)、[视口导航与方向控件](064-vtk-navigation-and-orientation.md)、[视口文档页签](080-qml-viewport-document-tabs.md)
- [图标规范](../standards/icons.md)、[QML 规范](../standards/qml.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)

## 范围与非目标

- 复刻截图中的 Appearance、Window、Locking、Browse、Cutting Plane、View 六个分组，呈现实体/透视图与单位选择、窗口布局、锁定、视口浏览、剖切平面、旋转角度与视图保存等控件。
- 先在 `ai-docs/qml-html/imported-project/imported-project.html` 完成 HTML / SVG 原型，并将顶部 View 页签接入 Ribbon 切换；维护者确认原型后再迁移 QML。
- 当前不实现窗口管理、视口操作、锁定、剖切、视图命名/保存等业务行为；原型只演示外观与状态，不伪造操作结果。
- 保留 Shell、工程树、Tasks、Layers、视口文档和其他 Ribbon 内容及状态。

## 实施步骤

1. 登记任务，按截图核对现有 View 页签入口、Ribbon 尺寸与视口相关实现。
2. 完成 HTML 布局与 SVG 图标，接入 View / Home / Results / Tools 页签切换并做静态检查。
3. 提交 HTML 原型供维护者确认；按反馈修订后再迁移 QML。
4. 新增 ViewRibbon 和正式图标资源、必要翻译与内容/导航回归测试。
5. 完成独立 subagent review、聚合验证，并同步任务、索引、HTML README 与图标规范。

## 验收标准

- [x] HTML View 页签包含截图中的六个工具分组，主要控件层级、顺序、标签和分组边界清晰。
- [x] View / Home / Results / Tools 可相互切换，只替换 Ribbon 内容。
- [x] HTML 原型经维护者确认后再迁移到 QML；两者分组结构和图标语义一致。
- [x] 未接入业务服务的动作没有虚假行为或误导性状态。
- [x] 构建、聚合测试、格式与静态检查通过；GUI 证据单独记录。
- [x] 独立 subagent review 完成并处理发现；task、索引、HTML README 同步。

## 验证与工作记录

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-12 | HTML 原型与静态检查 | 按截图建立 View Ribbon，并可在工程状态切换页签 | HTML 六分组原型和 29 个 SVG symbol 已加入；View / Home / Results / Tools 共用现有 Ribbon 切换。`node --check ai-docs/qml-html/shell.js`、Python HTML parser 与 `git diff --check` 通过。内置浏览器策略拒绝本地 `file:` URL，未取得渲染截图；维护者随后要求继续并迁移 QML。 |
| 2026-10-12 | QML 迁移与回归 | View Ribbon 六组结构与 HTML 一致，切换保留工程及视口对象；未接服务操作禁用 | 新增 `ViewRibbon.qml`、Atom `RibbonComboBox.qml`、29 个匹配 QML SVG、双语文案和导航回归。按反馈修正下拉框禁用态样式并调整 HTML/QML 组宽度；旋转角度/视图名称字段统一高度，Metric Units 下拉框收窄，真实窗口中确认分组边界正常。 |
| 2026-10-12 | 最终验证与 GUI 复核 | 所有验收项通过，确认尺寸与边界符合截图调整 | `cargo build --locked`、`cargo test --locked --workspace`（Rust 与 73/73 native/QML 测试）、`cargo lint --check`、`cargo format --check`、`qmlformat`、`git diff --check` 通过。真实 macOS 窗口中确认 Rotation Angle 输入框与 View Name 下拉框等高（22px），Metric Units 下拉宽112px，Browse 组408px可容纳内容且无溢出；临时预览工程、app wrapper 已关闭并清理。独立 subagent 复核发现并促成 Browse 组扩宽，最终复核无遗留问题。 |
| 2026-10-12 | Ribbon 布局微调 | 内容未铺满视口时隐藏滚动条；Appearance 下拉居中；View Name 下拉与角度输入框右对齐 | `ViewRibbon.qml` 原有的 64px `ribbonTrailingWidth` 占位计入了 Flickable 内容宽度，移除后内容宽度回到六组宽度之和；恢复 `HorizontalToolStrip` 原有 `ScrollBar.AsNeeded` 策略。Appearance 下拉在列中水平居中；View 表单标签共用动态列宽，使两个控件右边缘对齐。`cargo build --locked`、`cargo format --check`、`qmlformat`、`git diff --check` 通过；本轮修改前 `cargo test --locked --workspace`（73/73 native/QML）和 `cargo lint --check` 通过。维护者人工确认 UI 无异常；独立 subagent 复核未发现布局宽度或焦点滚动风险。 |
| 2026-10-12 | issue #8 文档复核 | View 迁移完成后 README 描述准确，图标和索引无重复或遗漏 | README 将“View HTML 原型”改为“View 内容”；共享图标表删除重复 Results 行，任务索引完成摘要列出 113、114。`git diff --check` 通过。 |

## 清理与兼容例外

无废弃实现，无兼容例外。

## 完成摘要

HTML 原型和 QML View Ribbon 已完成。滚动条根因已修复，控件对齐符合维护者预期；构建、聚合测试、lint、格式检查、独立评审和维护者人工 UI 验收通过。
