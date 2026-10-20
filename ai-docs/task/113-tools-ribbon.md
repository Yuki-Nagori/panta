# 113 — Tools 工具栏内容

- 状态：done
- 阶段：应用平台扩展
- 依赖：050, 062, 096
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-12 / 2026-10-12

## 目标与背景

按维护者提供的 Tools 页签截图，在已打开工程时提供 Tools Ribbon。依照既有页面工作流，先更新 `prototypes/shell/imported-project/imported-project.html` 与公共 SVG symbol，再按该原型迁移到 QML。原有 Shell、工程树、视口和其他 Ribbon 状态保持不变。

## 必读

- [HTML 参考工作流](../../prototypes/shell/README.md)
- [Ribbon 拆分](062-ribbon-tab-components.md)、[Results 工具栏](096-results-ribbon.md)
- [图标规范](../standards/icons.md)、[QML 规范](../standards/qml.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)

## 范围与非目标

- 按截图建立 Database、Automation、Specify Macro、Options、Task Key Manager、Performance 六个工具组及其图标、标签、紧凑宏编号区和分组栏。
- Tools 顶部页签在工程工作区可选中并显示该 Ribbon；切换 Home / Results / Tools 只替换 Ribbon 内容。
- 先完成人工可查看的 HTML 原型，再在同一任务中迁移 QML、SVG 资源、双语文案与对应测试。
- 本轮不实现数据库、宏录制/播放、命令行、任务密钥或旋转性能算法；未接入服务的操作保持禁用，不显示伪造状态。

## 实施步骤

1. 登记任务并核对 imported-project 页签导航、Ribbon 布局及现有图标资源。
2. 先完成 HTML / SVG 原型并运行脚本与差异检查；尝试浏览器渲染，受限时记录原因。
3. 按 HTML 结构迁移 QML，新增 ToolsRibbon 和正式 SVG 资源，接入页签切换与翻译。
4. 补充 Ribbon 内容、资源、禁用状态和页签切换测试，进行独立 subagent review。
5. 运行仓库适用的聚合验证，更新本文和 HTML 参考说明。

## 验收标准

- [x] Tools 页签选中时呈现六个截图分组，工具顺序、组边界和宏数字区布局对应参考。
- [x] 在工程状态下 Home / Results / Tools 可往返切换，不重建工程树、Tasks、Layers 或视口。
- [x] 无业务实现的工具呈禁用态且有可访问名称，不触发虚假的命令或状态。
- [x] HTML symbol、QML SVG 资源和 QML 翻译文案同步，HTML 与 QML 的分组结构一致。
- [x] 构建、聚合测试、格式与静态检查通过；GUI 证据单独记录。
- [x] task、索引和 HTML README 同步；完成独立 subagent review 并处理发现。

## 验证与工作记录

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-12 | HTML / QML 实施与验证 | HTML 先行、Tools 六组可切换、业务未接入项禁用 | 已完成 HTML 模板、SVG symbol、QML Ribbon、双语翻译和导航回归；`cargo build --locked`、`cargo test --locked --workspace`（73 项 CTest 全通过）、`cargo format --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`node --check prototypes/shell/shell.js`、`git diff --cached --check` 通过。`cargo lint` 启动器因沙箱禁止绑定 TCP listener 失败，直接运行其 Rust Clippy 检查通过。内置浏览器策略拒绝 `file:` 本地页面，未取得 HTML 截图。独立 review 发现宏槽位排列和往返导航覆盖不足，已修正；修正后聚合测试再次全通过。 |
| 2026-10-12 | Tools 宏槽布局微调 | 编号 Grid 在工具行可用高度内垂直居中 | Grid 增加垂直中心锚点；`cargo format --check`、`git diff --check`、`cargo test --locked --workspace` 通过（73 项 CTest 全通过）。独立 review 确认纵向锚定到 RibbonGroup 工具行不会影响 Row 的横向排布。 |
| 2026-10-12 | issue #8 文档复核 | 图标清单和任务完成摘要无重复或遗漏 | 删除图标规范中重复的 Results 行，并将任务索引摘要补齐 113、114；文档检查与 `git diff --check` 通过。 |

## 清理与兼容例外

无废弃实现，无兼容例外。

## 完成摘要

Tools HTML 原型与 QML Ribbon 已完成；独立 review 发现并推动修复宏槽排列及 Results / Tools 往返测试问题。所有验收项与聚合测试通过。浏览器安全策略拒绝打开本地 `file:` HTML，未生成浏览器截图；此限制已记录。
