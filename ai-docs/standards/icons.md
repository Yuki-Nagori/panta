# panta 图标设计规范

更新日期：2026-09-30。适用于 `qml/icons/`、Shell 图标使用点与 `ai-docs/qml-html/shell.js` 视觉原型。项目确定以 HTML 原型中经维护者确认的 SVG symbol 集合作为正式图标视觉基准；QML 使用的图标迁移到对应 symbol 的独立 SVG 资源，不再另画一套图形。迁移和组件适配由[081 QML 视觉语言与图标体系统一](../task/081-qml-visual-language-and-iconography.md)跟踪。

## 视觉语言

采用紧凑、几何化、轻量多色的线性图标：主要轮廓清楚，使用少量蓝、红、黄、绿等语义点缀与浅色填充；色块帮助识别，不依靠色相单独传达操作状态。保留`shell.js` symbol 中的自然构图、比例、描边与配色，不为匹配旧的 24 网格把原始图形强行拉伸或再次单色化。不同功能可以有小图标、Ribbon 图标与 Start & Learn 入口图标三档画布，最终显示尺寸由 Theme 和宿主上下文控制。

- 操作图标使用明确的主体轮廓与少量语义色；同一动作在菜单、工具栏和 Ribbon 中复用同一概念资源。
- 工程树、STL、Plan tasks 与任务图标允许按本身语义保留不同形状和颜色；任务状态仍须有形状/符号提示，不能只靠绿/灰等颜色。
- caret、关闭、翻页等辅助符号可保持单色；它们也使用 shell.js 对应 glyph 和统一的宿主交互色。
- Welcome `panta` 3D 标志不是操作图标；它的真实几何由任务 053 实现，颜色与表面方向遵循任务 081 的视觉规范。
- 不引用、描摹或嵌入第三方应用品牌、专有字形、图标字体或无明确来源的图标包。截图只作为布局与风格方向。

## HTML symbol 清单

下列 ID（在 HTML 中以 `#i-<id>` 引用）是当前正式视觉参考。迁移 QML 时保留几何和色彩，输出资源清单同时登记 QML 语义名。

| 组 | symbol ID |
|---|---|
| 通用与窗口操作 | `account`、`cart`、`globe`、`help`、`search`、`right`、`caret`、`close`、`minimize`、`maximize`、`undo`、`redo`、`new`、`open`、`save`、`print`、`preview`、`split` |
| 工程树与状态 | `project-folder`、`project-file`、`stl-file`、`plan-tasks`、`status-ok`、`layers` |
| Ribbon：工程与开始 | `ribbon-project`、`ribbon-open-project`、`ribbon-new-features`、`ribbon-start-here`、`ribbon-tutorials`、`ribbon-videos`、`ribbon-help` |
| Ribbon：CAE 工具 | `ribbon-import`、`ribbon-add`、`ribbon-dual-domain`、`ribbon-geometry`、`ribbon-mesh`、`ribbon-thermoplastics-injection-molding`、`ribbon-analysis-sequence`、`ribbon-select-material`、`ribbon-injection-locations`、`ribbon-process-settings`、`ribbon-optimization`、`ribbon-boundary-conditions`、`ribbon-analyze`、`ribbon-job-manager`、`ribbon-results`、`ribbon-reports`、`ribbon-shared-views`、`ribbon-logs` |
| Ribbon：Results | `results-new-plot`、`results-notes`、`results-xy-curve`、`results-plot-properties`、`results-save-defaults`、`results-examine`、`results-min-max`、`results-histogram`、`results-scale`、`results-reset-scale`、`results-scale-range`、`results-warpage`、`results-defect`、`results-export`、`results-mark`、`results-unmark`、`results-cutting-plane`、`results-first`、`results-last`、`results-play`、`results-pause`、`results-stop`、`results-loop`、`results-ping-pong`、`results-tile-horizontal`、`results-tile-vertical`、`results-window-grid`、`results-window-horizontal`、`results-window-vertical`、`results-window-sync`、`results-lock-window`、`results-lock-legend`、`results-lock-mesh`、`results-sync-lock-window`、`results-sync-lock-legend`、`results-sync-lock-mesh`、`results-release-lock-window`、`results-release-lock-legend`、`results-release-lock-mesh` |
| Plan tasks | `task-analysis`、`task-analysis-sequence`、`task-injection`、`task-material`、`task-mesh`、`task-optimization`、`task-settings` |
| 输出操作 | `check`、`copy`、`delete`、`export`、`image`、`log`、`wizard` |

新增图标先确认清单中没有相同概念；若需增加，先在 `shell.js` 加入原型 glyph、登记此规范中的 ID 和含义，再将同一几何迁入独立 QML SVG。HTML sprite 与 QML 资源不可长期各自演变成不同图样。

## QML 资源与调用清单

`qml/icons/` 当前包含 102 个正式 QML symbol 资源，与上表的 102 个 symbol ID 一一对应；文件名固定为 `<symbol-id>.svg`，资源内部记录对应的 `shell.js` symbol 来源。除下表说明的 caret 缩小外，其余资源保留源 viewBox、路径、比例和填色。它们均由本项目绘制，没有外部图标包、第三方许可证或外部资源依赖。消费者切换后已通过独立 cleanup commit 删除迁移前的旧 SVG。

| QML 资源组 | symbol ID / 文件名（逐项同名） | 当前调用位置 |
|---|---|---|
| 壳层与操作 | `account`、`cart`、`globe`、`help`、`search`、`right`、`caret`、`close`、`minimize`、`maximize`、`undo`、`redo`、`new`、`open`、`save`、`print`、`preview`、`split` | 顶部工具区、对话框、面板关闭、页签与下拉指示 |
| 页签与工具状态 | `layers` | Layers 工具条与页签 |
| Ribbon | `ribbon-project`、`ribbon-open-project`、`ribbon-new-features`、`ribbon-start-here`、`ribbon-tutorials`、`ribbon-videos`、`ribbon-help`、`ribbon-import`、`ribbon-add`、`ribbon-dual-domain`、`ribbon-geometry`、`ribbon-mesh`、`ribbon-thermoplastics-injection-molding`、`ribbon-analysis-sequence`、`ribbon-select-material`、`ribbon-injection-locations`、`ribbon-process-settings`、`ribbon-optimization`、`ribbon-boundary-conditions`、`ribbon-analyze`、`ribbon-job-manager`、`ribbon-results`、`ribbon-reports`、`ribbon-shared-views`、`ribbon-logs` | Start & Learn 与 Home Ribbon |
| Results | `results-*`（上表逐项同名） | Results Ribbon 的图文工具、动画控制、布局与锁定按钮 |
| 工程与任务 | `project-file`、`project-folder`、`stl-file`、`plan-tasks`、`status-ok`、`task-analysis`、`task-analysis-sequence`、`task-injection`、`task-material`、`task-mesh`、`task-optimization`、`task-settings`、`log` | Tasks 工程项、导入零件与任务列表；`project-folder` 和 `status-ok` 已打包，目前没有对应 QML 模型状态 |
| Layers 操作 | `check`、`copy`、`delete`、`export`、`image`、`wizard` | Layers 输出操作按钮 |

`ThemedIcon.preserveSourceColors` 是颜色加载模式的唯一开关，不另设彩色图标组件。默认模式将允许的单色 utility glyph 交给 `image://panta-icons/<name>/<argb>`；provider 内的白名单与颜色图标清单保持一致，未知和多色 ID 均拒绝着色。设置为 `true` 时直接从 `qrc:/qt/qml/Panta/Shell/icons/<name>.svg` 加载并保留 SVG 源色。QtTest 校验 SVG 文件名与 `shell.js` symbol 集合一一对应，QML 字面图标引用均有资源，静态加载模式与 provider 白名单一致；Ribbon、任务列表和 Layers 的动态模型模式也单独校验。单色集合检查缩放、着色与错误路径。

`caret.svg` 的源 viewBox 为 `0 0 7 5`。为响应维护者要求缩小指示三角，仅将路径由
`M0 0h7L3.5 5z` 内缩为 `M0.7 0.6h5.6L3.5 4.4z`；HTML symbol 与 QML 资源同步，viewBox
和显示槽位不变。其余源符号没有几何修改。

## QML 资源与颜色

- 以 `qml/icons/` 中独立 SVG 文件供 QML 使用，不让运行中的 QML 加载 `ai-docs/`。静态 SVG 继续经 `qt_add_resources` 随模块打包；遵守仓库对 QML/SVG 的文件规范。
- 保留 symbol 的原始 viewBox 和坐标比例；宿主用 Theme token 控制逻辑像素大小，图标不自行乘 DPR。资源中不嵌入位图、字体、脚本或外部 URL。
- 彩色 SVG 必须使用保留原始填色的资源加载路径，不可经过会把 alpha mask 染成单色的 `panta-icons` provider。caret、close 等单色 utility glyph 可继续走 provider；组件要明确区分两种呈现方式。
- 使用 QML Image / SVG 的具体渲染路径、qsvg 依赖、缓存和 DPI 行为需通过打包测试及适用的 CPU/GPU 基准验证；不得假定彩色 SVG 与单色遮罩的性能完全相同。
- SVG 作为装饰内容时设为屏幕阅读器忽略；操作按钮通过宿主英文源 `qsTr()` 文案提供 `Accessible.name` 与 tooltip，并保留键盘焦点、禁用原因和文字标签。
- 图标状态的颜色与组件状态保持协调；正常、选中、悬停、按下、禁用及焦点不能因 glyph 固定色而无法辨识。需要变色时按设计拆分资源或使用宿主叠层，不得破坏彩色图标本身含义。

## 旧资源到正式 symbol 的迁移记录

以下记录迁移前已删除的旧文件与采用的正式 symbol；调用点统一使用当前 symbol ID。
若经视觉检查发现概念不对应，应修正 symbol 或 QML 语义映射并同步清单，不得仅按文件名机械替换。

| 现有 QML 资源 | 正式 HTML symbol |
|---|---|
| `document-new.svg` | `ribbon-project` |
| `document-open.svg` | `ribbon-open-project` |
| `document-save.svg` | `save` |
| `document-print.svg` | `print` |
| `edit-undo.svg` / `edit-redo.svg` | `undo` / `redo` |
| `animation-preview.svg` / `activate-split.svg` | `preview` / `split` |
| `binoculars.svg` / `user-account.svg` / `shopping-cart.svg` | `search` / `account` / `cart` |
| `help-browser.svg` / `menubar-globe.svg` / `caret-down.svg` / `caret-right.svg` | `help` / `globe` / `caret` / `right` |
| `pane-close.svg` | `close` |
| `output-new.svg` / `output-check.svg` / `output-wizard.svg` | `new` / `check` / `wizard` |
| `output-copy.svg` / `output-image.svg` / `output-export.svg` / `output-delete.svg` | `copy` / `image` / `export` / `delete` |
| `ribbon-start.svg` / `ribbon-whatsnew.svg` / `ribbon-learn.svg` | `ribbon-start-here` / `ribbon-new-features` / `ribbon-tutorials` |
| `project-file.svg` / `project-import.svg` / `layers.svg` | `project-file` / `ribbon-import` / `layers` |
| `domain-dual.svg` / `geometry.svg` / `mesh.svg` / `molding.svg` | `ribbon-dual-domain` / `ribbon-geometry` / `ribbon-mesh` / `ribbon-thermoplastics-injection-molding` |
| `analysis-sequence.svg` / `material.svg` / `injection-location.svg` / `process-settings.svg` | `ribbon-analysis-sequence` / `ribbon-select-material` / `ribbon-injection-locations` / `ribbon-process-settings` |
| `optimization.svg` / `boundary-conditions.svg` / `analysis-run.svg` | `ribbon-optimization` / `ribbon-boundary-conditions` / `ribbon-analyze` |
| `document-report.svg` / `job-manager.svg` / `analysis-results.svg` / `shared-views.svg` | `ribbon-reports` / `ribbon-job-manager` / `ribbon-results` / `ribbon-shared-views` |
| `media-video.svg` | `ribbon-videos` |

工程文件夹、STL、Plan tasks、任务动作与状态在当前 HTML symbol 清单中有专用资源；迁移时一并创建其 QML 文件映射。`ribbon-logs` 与 `ribbon-reports` 按实际功能分别使用，不能把日志和报告合并成一个概念。

## 来源与商业使用记录

- 当前 `shell.js` symbol 是本项目原型内编写的 SVG 路径，不依赖外部图标包；Welcome 字样是 CSS 效果，不是已生成的 STL 或位图。参考截图由维护者提供，不等于其中第三方素材的授权。
- 每项正式资源登记“内部绘制 / 外部来源、作者或来源 URL、许可证、版本、修改情况、使用位置”。不确定来源的旧资源须核验，未澄清前不得视为可发布资产。
- 生成式服务的条款可能将服务输出的权利转让给用户，但该权利归属不证明内容独创，也不保证没有第三方权利冲突。OpenAI 当前条款说明用户拥有 Output，同时指出输出可能相似且不保证准确/无侵权；商业发布前仍需评估商标、外观和第三方输入权利。[OpenAI Terms of Use](https://openai.com/policies/row-terms-of-use/)

## 流程与审查

1. 先查 symbol 清单复用已有 glyph；改变符号或增加符号先同步原型、清单与 QML 映射。
2. 保留彩色资源的原始颜色；QML 调用点通过统一组件加载，避免散布 SVG 路径和魔法尺寸。
3. 从仓库根执行 `cargo build --locked`、适用的 `cargo lint qmllint`、`cargo format --check` 和 QtTest；检查资源打包后可加载，不以浏览器显示代替真实 QML 渲染证据。
4. 使用所锁定 Qt 实际检查 1.0/1.5/2.0 缩放、浅色背景、选中/悬停/禁用/键盘焦点，以及窄窗口下文字和图标对齐。
5. 影响构造、布局、资源解码、图标更新或帧呈现的 QML 变更须依 [QML 性能基准规范](qml.md#性能基准) 纳入对应手动场景；GPU 性能只在真实窗口测量。

- [ ] 图标概念复用且映射唯一；是否保留源 viewBox 与比例？
- [ ] 正常、禁用和焦点状态清楚，文本/可访问名称完整？
- [ ] 每项资源的来源、许可证、修改和调用点均可追溯？
- [ ] 多色路径未误用单色 mask；打包、缩放和性能证据齐全？
- [ ] HTML sprite、QML SVG、Theme、任务与索引同步，无旧资源残留？
