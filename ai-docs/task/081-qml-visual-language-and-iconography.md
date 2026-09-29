# 081 — QML 视觉语言与图标体系统一

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[029](029-qml-component-library.md)、[050](050-qml-html-page-replica.md)、[069](069-project-docks-review-and-ablation.md)、[078](078-qml-performance-benchmark-policy.md)、[080](080-qml-viewport-document-tabs.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-30

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

- 不改变 STL/网格业务、导入/导出、求解工作流或 FSM 状态机；不因换图标而改变命令语义。
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
4. 迁移现存 QML 组件和页面，逐项替换旧样式及图标，删除失效 token 和重复 CSS；旧 SVG 在所有消费者切换完成后单独提交清理，并同步 QML Theme、图标清单、模块文档与页面任务。
5. 将 VTK Welcome 标志要求交接给 053，将页签视觉契约交接给 080；完成窗口截图、键盘/无障碍、翻译、高 DPI 和打包资源检查。
6. 图标资源请求与 SVG 解码进入 069 CPU 手动基准，对比空 source、单色 provider 和彩色 qrc 加载；仅当改动了连续帧更新或 GPU 呈现工作时再运行 GPU 基准。记录环境、负载、采样、p50/p95 与消融结果，并执行适用 Cargo 聚合检查、更新 task / 索引。

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
- [ ] 自动一致性检查覆盖 SVG 资源 ID、QML 图标引用及彩色/单色加载模式，确保单色 provider 白名单与 QML 调用约定一致；新增或误配图标时检查失败并指出位置。
- [ ] 多色图标的正常、禁用、焦点和高对比场景可辨；颜色不作为唯一的信息通道；键盘焦点和可访问名称完整。
- [ ] HTML 参考与真实 QML 在相关组件层级、颜色、图标和交互状态一致；QML 打包后所有模块资源可载入，`qmllint` 与组件行为测试通过。
- [ ] 窄宽度、长英文/中文、至少 1.0/1.5/2.0 DPR、焦点/禁用态与真实图形窗口无截断、重叠或看不清的状态。
- [ ] 每个新增 QML 组件及受性能影响的组件映射到 069 CPU/GPU harness 场景；图标加载映射到 CPU，并按适用基准记录输入规模、平台/Qt/图形后端、采样、p50/p95 和测量边界。
- [ ] 旧资源、旧 token 和失效视觉路径清理完毕；task、索引、Theme、图标清单、HTML 与 QML 状态一致。

## 验证计划与结果

实施中。在仓库根目录按锁定工具链运行 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`；补充真实窗口截图、SVG 资源打包、键盘/焦点和多 DPI 验收。按实际受影响的 QML 场景手动运行 069 的 CPU / GPU 基准并记录测量，GPU 端只在真实图形窗口执行，不接入 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | 参考图来源核验与仓库资产盘点 | 区分用户提供参考图、CSS 字样、HTML 内联 SVG 与 QML 图标文件 | 已初步核实 HTML Welcome 是 CSS 立体文字，Ribbon 图标是 `shell.js` 内联手写 SVG；完整 QML SVG 来源审计待实施 |
| — | 真实 QML 视觉迁移、真窗口、多 DPI 与 QML CPU / GPU 手动基准 | 按以上验收执行 | 未实施 |
| 2026-09-29 | 盘点 029/050/069/078/080/053 的已交付组件、基准与交接边界 | 在已有组件库、主题 token、页签和 benchmark 上完成剩余统一工作 | 核心实现可开始；080 的 VoiceOver 验收与 053 的 3D Welcome 完整验收仍按各自 task 记录，不阻止本任务完成图标、HTML 与 QML 视觉体系 |
| 2026-09-30 | `qml/icons/` SVG 资源首批与 `shell.js` caret symbol | 独立资源可与正式 symbol 对照，应用已有资源仍保持可用 | 新增 61 个正式 symbol SVG，并同步 caret 的内缩 path；保留现有资源，避免在消费者迁移前改变运行引用。`layers.svg` 与 `project-file.svg` 两个同名资源在调用迁移批次替换；旧资源清理、Qt 加载测试与性能测量随后完成 |
| 2026-09-30 | `Qml.ThemeComponentParameters` 与 `panta_qml_cpu_benchmark measures_icon_loading` | 验证 Qt 可以解码全部打包 SVG，并比较空 source、单色 provider 和原色 qrc 的首轮及缓存路径 | QtTest 通过；CPU 基准通过。环境：macOS 26.3.1 arm64、Qt 6.11.2、`target/native/debug`。首轮样本包含进程/路径初始化；缓存后采样 31 次，p50/p95 见下表。场景在 offscreen 下测 Image 创建至 Ready，不测 GPU 呈现 |
| 2026-09-30 | 旧图标资源 cleanup commit 与 QML 调用审计 | 旧路径无消费者，打包目录仅保留 63 个正式 symbol | 删除 43 个旧 SVG；源码引用扫描未发现旧图标 ID，`Qml.ThemeComponentParameters` 断言 qrc 中恰有 63 个 SVG |
| 2026-09-30 | `Qml.ThemeComponentParameters`；`cargo build --locked`、`cargo format --check`、`cargo lint --check`、`cargo test --locked --workspace` | 对齐 `shell.js` symbol、Qt 打包 SVG、QML 图标引用、单色 provider 白名单及静态/动态颜色模式；适用聚合检查通过 | 一致性 QtTest 通过；SVG 集合与正式 symbol 集合逐项相等，图标文件名与 SVG 内来源标记相符；QML 引用均有资源，provider 着色模式、Ribbon / 任务 / 图层数据和动态转发链路匹配。`cargo build --locked`、`cargo format --check`、`cargo lint --check` 的 8 个阶段及 `cargo test --locked --workspace` 通过，CTest 66/66。默认会自动修复的 `cargo lint` 因沙箱拒绝绑定本地锁 listener 未能启动；只读 lint 全检查通过 |

### 图标加载 CPU 消融

单位为微秒；首轮样本 / 缓存后 p50 / p95。计时覆盖创建指定数量的 `ThemedIcon` 并等待图像状态就绪；空 source 基线创建相同数量的图像项。source-color 的 1 项首轮样本包含本进程首次初始化 SVG 资源加载路径；缓存后各模式的 p50/p95 与空 source 基线在同一量级。

| 加载模式 | 图标数 | 首轮样本 | 缓存 p50 / p95 |
|---|---:|---:|---:|
| 空 source 基线 | 1 | 632.292 | 20.667 / 32.334 |
| 空 source 基线 | 8 | 92.417 | 76.667 / 128.916 |
| 空 source 基线 | 24 | 352.875 | 195.708 / 372.500 |
| 单色 provider | 1 | 1352.210 | 21.125 / 31.375 |
| 单色 provider | 8 | 332.042 | 84.375 / 110.167 |
| 单色 provider | 24 | 278.125 | 222.542 / 400.000 |
| 原色 qrc SVG | 1 | 5518.000 | 21.292 / 25.083 |
| 原色 qrc SVG | 8 | 436.834 | 70.667 / 77.417 |
| 原色 qrc SVG | 24 | 993.208 | 183.875 / 354.208 |

## 风险与回退

全局视觉改动容易造成已完成页面回归或图标语义变化；按页面和组件小批迁移，保留可对比截图与旧值清单，发现回归时仅回退对应页面并修复公共 token，不复制临时第二套 Theme。商标和第三方素材风险通过内部绘制、明确许可与逐项来源记录控制；本任务不把 OpenAI 对输出的权利归属视为非侵权保证。

## 决策与工作记录

- 2026-09-24：根据维护者提供的 Welcome 3D `panta` 标志和多色线性 Ribbon 图标参考，新建应用级视觉统一任务。图像是风格参考，不要求照搬第三方产品标识或专有图标。
- 2026-09-24：检查当前 HTML 原型：Welcome 文字由 CSS 样式绘制，Ribbon glyph 为 `shell.js` 手写内联 SVG；二者不是 ImageGen 位图。现存 QML SVG 仍需对照图标规范和 029 记录核验来源。
- 2026-09-24：053 负责 VTK 立体几何落地并依赖本任务的外观方向；080 负责文档页签行为，本任务为其提供统一视觉契约。
- 2026-09-30：正式 SVG 资源先提交；按维护者要求，在 QML 调用迁移完成后将旧 SVG 删除作为独立 cleanup commit。图标性能采用 069 CPU harness，GPU 帧计时不用于没有连续更新的静态资源加载问题。

## 完成摘要

未实施。已登记视觉方向、资源出处审计、Theme/QML/HTML 迁移、与 053/080 的边界，以及必须记录的 QML 性能基准。
