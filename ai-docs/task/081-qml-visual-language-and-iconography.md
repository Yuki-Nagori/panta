# 081 — QML 视觉语言与图标体系统一

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[029](029-qml-component-library.md)、[050](050-qml-html-page-replica.md)、[069](069-project-docks-review-and-ablation.md)、[078](078-qml-performance-benchmark-policy.md)、[080](080-qml-viewport-document-tabs.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-25

## 目标与背景

以维护者提供的参考图为方向，统一 panta 桌面应用的视觉语言：浅色中性工作区、简洁的工具分组、辨识清楚且有节制的多色线性图标，以及具浅立体厚度的 Welcome `panta` 标志。Welcome 保留既有冷暖过渡渐变作为品牌装饰，不用结果云图代替品牌材质；字形避免过重。当前 QML Theme 和图标体系沿用早期 HTML 截图底稿；新参考强调的颜色、图形和图标呈现需要先成为一套可维护的规则，再统一映射到 HTML 参考与实际 QML。

本任务负责视觉规范、资源审计和应用级迁移；默认视口内真正的 VTK 立体字形/网格由 [053](053-default-panta-wordmark.md) 按本任务定下的视觉方向实现。浏览器式视口页签的交互由 [080](080-qml-viewport-document-tabs.md) 实现，本任务统一其最终颜色、图标、边框和交互态。

## 必读

- [组件与主题](../modules/qml-components-and-theme.md)、[SVG 图标规范](../standards/icons.md)、[QML 规范](../standards/qml.md)
- [HTML 先行复刻](../task/050-qml-html-page-replica.md)、[默认 Welcome 立体字样](053-default-panta-wordmark.md)、[视口文档页签](080-qml-viewport-document-tabs.md)
- [QML 性能基准登记规范](../task/078-qml-performance-benchmark-policy.md)、[性能测试模块](../modules/performance.md)、[Docks 评审与消融](069-project-docks-review-and-ablation.md)
- [文件规范](../standards/repository-hygiene.md)、[注释规范](../standards/comments.md)、[文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

包含：

- 审计 `qml/Themes/Theme.qml`、现有 QML 组件、Ribbon、工程/任务/Layers Dock、对话框、视口与 080 页签；建立可追溯的色彩、字号、图标比例、圆角、间距、层级、边框与交互状态规范。
- 以 `shell.js` 的全部内联 SVG symbol 作为唯一图标造型基准，将现有 QML 图标全集替换为对应 symbol 的独立多色 SVG 资源；为 HTML 树与任务区中尚未进入 QML 清单的工程文件夹、STL、Study、任务操作和状态符号补全 QML 映射。形成应用内资源清单，记录内部绘制/第三方来源、许可证、修改情况和对应功能；无法确认来源的资源不得直接作为可发布资产沿用。
- 适配 QML 图标加载组件，避免彩色 SVG 经过当前单色 `panta-icons` 渲染路径而丢失颜色；颜色表达语义，不依赖色相作为唯一状态信息。
- 更新 `ai-docs/qml-html/` 中适用参考页面，先稳定视觉底稿，再将规则映射至 Theme token 和 QML 组件；保留 HTML 与 QML 一致的 token 与图标语义。
- 统一常态、悬停、按下、选中、禁用、焦点、关闭和空白视口等可见状态；覆盖英文/中文长度、键盘焦点与高 DPI 布局。
- 为 053 的 VTK Welcome 标志规定渐变语义、表面质感、浅挤出、背景/投影关系和留白约束；具体几何实现与真实窗口验收归 053。
- 依据 [QML 性能规范](../standards/qml.md#性能基准)，把本任务新增或影响创建、布局、绑定、更新与绘制的 QML 组件纳入 069 CPU / GPU 手动基准场景。

不包含：

- 不改变 STL/网格业务、导入/导出、求解工作流或 Flow 状态机；不因换图标而改变命令语义。
- 不照搬第三方应用品牌、专有图标或字体文件；截图只提供风格方向，不作为允许复制外部受保护素材的依据。
- 不在本任务实现 053 的 VTK 字形网格，不实现 080 的多文档状态与资源生命周期，不重做 030 主题 DSL。
- 不把视觉校对或图像生成的使用权结论等同于法律上的商标/著作权清查。

## 前置条件与待决策

- 依赖任务完成后盘点实际运行页面和可复用组件；确认 080 最终标签结构和 053 Welcome 渲染边界，避免在 HTML、QML、VTK 三处维护重复样式规则。
- 开始批量替换资源前，逐项确认现有 SVG 的历史来源；保留来源可证的内部资产，重绘或删除来历不明的资产。若选用第三方资源，必须在清单记明许可证、版本、来源 URL、修改和归属要求。
- 决定应用首期为浅色单主题还是为浅色/深色都提供映射；若扩展主题覆盖，不绕过 030 的权威主题数据流，也不复制第二份默认值。
- 确定多色 SVG 资源是独立的正常 `<Image>` 资源，还是现有单色图标 provider 的显式模式；由真实组件需求与 SVG 渲染、打包和性能测量作决定。
- Welcome 的 3D `panta` 图形按 053 与真实 VTK 能力验收；HTML 只能作为外观底稿，不用 CSS 字体效果冒充实际 VTK 几何完成。

## 实施步骤

1. 盘点所有现有 Theme token、SVG 和 HTML 内联图标，核对功能映射与可确认来源；记录待移除和待重绘清单。
2. 依据参考图建立一页视觉规格与关键状态样例，确定色板、排版、组件表面和 Welcome 标志材质方向；图标来源已由本次决策固定为 `shell.js` symbol，不再另选图标集。检查文本/焦点对比度与色觉可区分性。
3. 更新相关 HTML 参考页并核对选中/关闭标签、工具组、Dock、空 Welcome 和空视口；将确定的视觉规则落为唯一 Theme token 和 SVG 资源映射。
4. 迁移现存 QML 组件和页面，逐项替换旧样式及图标，删除失效资源、旧 token 和重复 CSS；同步 QML Theme、图标清单、模块文档与页面任务。
5. 将 VTK Welcome 标志要求交接给 053，将页签视觉契约交接给 080；完成窗口截图、键盘/无障碍、翻译、高 DPI 和打包资源检查。
6. 按受影响组件运行 QML CPU / GPU 手动基准，记录基线、运行环境、负载、采样、p50/p95 与消融结果；执行适用 Cargo 聚合检查并更新 task / 索引。

## 预计改动

- `qml/Themes/Theme.qml`、`qml/Components/`、`qml/Panels/`、`qml/Panels/Ribbon/` 与 `qml/icons/*.svg`。
- `ai-docs/qml-html/shell.css`、`shell.js` 和实际受影响的页面；`ai-docs/qml-html/README.md`。
- `ai-docs/standards/icons.md`、`ai-docs/modules/qml-components-and-theme.md` 与本任务依赖页面；图标来源/功能清单维护在 `ai-docs/standards/icons.md`。
- 现有组件测试、截图/可访问性检查，以及 `tests/qml/project_docks_cpu_benchmark.cpp`、`tests/qml/project_docks_gpu_benchmark.cpp` 中适用的场景。
- 053/080 的设计依赖与实现决策；不改 Rust / native 业务接口。

## 清理与兼容例外

替换时删除无消费者的旧 SVG、重复颜色值和已被新视觉系统取代的 QML/HTML 样式，不能长期并存两套视觉 token 或含义重复的图标。必要时旧视觉语言分阶段迁移到剩余消费者，但需逐个登记范围和删除条件；默认无兼容例外。

## 验收标准

- [ ] 所有产品页面使用同一套 Theme 语义 token；色彩、字号、间距、图标尺寸和组件状态无重复权威定义。
- [ ] 关键界面与所给风格方向一致：浅色工作区、轻量 chrome、分组工具条、清晰图标层级、统一圆角/边框/悬停反馈；Welcome 3D 标志由 053 使用约定外观实现。
- [ ] 所有实际使用图标在清单中有唯一语义、功能映射、来源/许可证及修改记录；第三方素材满足许可证和署名要求，未确认来源的资源已移除或原创重绘。
- [ ] 多色图标的正常、禁用、焦点和高对比场景可辨；颜色不作为唯一的信息通道；键盘焦点和可访问名称完整。
- [ ] HTML 参考与真实 QML 在相关组件层级、颜色、图标和交互状态一致；QML 打包后所有模块资源可载入，`qmllint` 与组件行为测试通过。
- [ ] 窄宽度、长英文/中文、至少 1.0/1.5/2.0 DPR、焦点/禁用态与真实图形窗口无截断、重叠或看不清的状态。
- [ ] 每个新增 QML 组件及受性能影响的组件映射到 069 CPU/GPU harness 场景；按适用基准记录输入规模、平台/Qt/图形后端、采样、p50/p95 和测量边界。
- [ ] 旧资源、旧 token 和失效视觉路径清理完毕；task、索引、Theme、图标清单、HTML 与 QML 状态一致。

## 验证计划与结果

尚未实施。依赖完成后，在仓库根目录按锁定工具链运行 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`；补充真实窗口截图、SVG 资源打包、键盘/焦点和多 DPI 验收。按实际受影响的 QML 场景手动运行 069 的 CPU / GPU 基准并记录测量，GPU 端只在真实图形窗口执行，不接入 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | 参考图来源核验与仓库资产盘点 | 区分用户提供参考图、CSS 字样、HTML 内联 SVG 与 QML 图标文件 | 已初步核实 HTML Welcome 是 CSS 立体文字，Ribbon 图标是 `shell.js` 内联手写 SVG；完整 QML SVG 来源审计待实施 |
| — | 真实 QML 视觉迁移、真窗口、多 DPI 与 QML CPU / GPU 手动基准 | 按以上验收执行 | 未实施 |

## 风险与回退

全局视觉改动容易造成已完成页面回归或图标语义变化；按页面和组件小批迁移，保留可对比截图与旧值清单，发现回归时仅回退对应页面并修复公共 token，不复制临时第二套 Theme。商标和第三方素材风险通过内部绘制、明确许可与逐项来源记录控制；本任务不把 OpenAI 对输出的权利归属视为非侵权保证。

## 决策与工作记录

- 2026-09-24：根据维护者提供的 Welcome 3D `panta` 标志和多色线性 Ribbon 图标参考，新建应用级视觉统一任务。图像是风格参考，不要求照搬第三方产品标识或专有图标。
- 2026-09-24：检查当前 HTML 原型：Welcome 文字由 CSS 样式绘制，Ribbon glyph 为 `shell.js` 手写内联 SVG；二者不是 ImageGen 位图。现存 QML SVG 仍需对照图标规范和 029 记录核验来源。
- 2026-09-24：053 负责 VTK 立体几何落地并依赖本任务的外观方向；080 负责文档页签行为，本任务为其提供统一视觉契约。

## 完成摘要

未实施。已登记视觉方向、资源出处审计、Theme/QML/HTML 迁移、与 053/080 的边界，以及必须记录的 QML 性能基准。
