# panta 图标设计规范

项目约定，更新日期：2026-09-22。适用 `qml/icons/` 与 Shell 图标使用点；新增图标先登记清单。布局参考 [homepage.html](../qml-html/homepage/homepage.html)，HTML 共用壳层中的彩色 SVG 只作设计占位，本规范是正式图标的几何与颜色依据。

## 设计语言

采用几何化、克制的线性图标。以直线、折线、矩形、圆和圆角构成，每个图标表达一个概念；允许为了问号、人物、灯泡和转向语义使用规则圆弧，不使用手绘轮廓、阴影、渐变或无意义细节。

- 统一 `viewBox="0 0 24 24"`；方形画布，不强制文件、箭头等自然轮廓拉伸为正方形。
- 主轮廓中心线通常位于 `3.5–20.5`，2 单位描边向外扩展后仍留至少 2.5 单位画布空白。下拉/关闭符号允许更小的居中主体，保持视觉重量。
- 统一 `stroke-width="2"`、`stroke-linecap="round"`、`stroke-linejoin="round"`；独立圆角矩形 `rx="2.5"`。圆与语义轮廓沿自身曲率，不强加矩形半径。
- 非连接元素优先保留 2 单位可见间隙；刻意连接的结构边线、叠层转折和语义交点不按独立元素计。最终以 16/18/24 逻辑像素目视检查，不能仅放大审图。

## 双轨分类

| 分类 | 使用场景 | 颜色 / 填充 | 命名 |
|---|---|---|---|
| Mono | 工具按钮、菜单、状态、可点击的 Ribbon 入口 | `stroke="currentColor"`、`fill="none"`，无固定色值；颜色由 QML 宿主提供 | `kebab-case.svg` |
| Art | 说明卡、空态、引导插图；不能作为操作按钮图标 | 同一 Mono 几何骨架，最多三色，10%–20% 同色透明填充 | `kebab-case-art.svg` |

Art 候选配色采用 panta 品牌红 `#c8322b`、强调金 `#d49b32` 与中性灰 `#71717a`；新 Art 落地时必须在浅/深背景核对对比度。当前没有 Art 资源，不为规范预建未使用文件。HTML 中曾带颜色的文件夹、保存及 Ribbon 操作入口也统一为 Mono。

## Qt Quick 实现规则

- 一个概念一个 SVG。不同面板的打开/保存/新建复用同一资源，通过宿主语义区分，不能复制并换文件名。SVG 文件头一行记录概念与网格；不写固定 width/height，不内嵌图片、脚本、字体或外部 URL。
- `ThemedIcon` 接收 `name`、`iconSize`、`color`；尺寸默认绑定 Theme token，保持宽高比，布局使用逻辑像素，不在 QML 乘 DPR。Qt 6.11.2 的 Image 在设置 sourceSize 后会按窗口 DPR 向 provider 请求像素尺寸，provider 直接使用 requestedSize，不重复缩放。
- SVG 的 `currentColor` 不是 QML 颜色继承机制。本项目的 `panta-icons` image provider 将资源透明度遮罩按显式 ARGB 颜色着色，输出由 Qt Image 缓存；引擎在加载 Shell 前安装 provider。它仅消费内置 Mono SVG，Art 将直接使用原始资源，不能走单色遮罩。
- 颜色默认取 `Theme.colorIcon`；按钮传入 `contentColor`，深色菜单取 `Theme.colorMenubarText`，关闭悬停取白色，禁用态由宿主统一弱化，不能每个 SVG 烘焙一套颜色。
- `ThemedIcon` 设置 `Accessible.ignored: true`（对应装饰图像的 aria-hidden）；按钮以 `Accessible.name` 提供英文源 `qsTr()` 文案。图标专用按钮提供 tooltip，不能把文件名当可访问名称。Ribbon 保留文字和键盘焦点反馈。
- 不使用 Vue SFC、CSS class、`aria-*` 或 `bun` 命令；不依赖 Qt 私有 QML 类型。SVG 由 qtsvg 渲染，构建资源经现有 `qt_add_resources` 收集。

Qt 官方依据（2026-09-22，6.11.2）：[Image 与 sourceSize](https://doc.qt.io/qt-6/qml-qtquick-image.html)、[QQuickImageProvider](https://doc.qt.io/qt-6/qquickimageprovider.html)、[Accessible](https://doc.qt.io/qt-6/qml-qtquick-accessible.html)。这些是渲染/可访问性机制；上述网格和配色是项目约定。

## 图标清单

| 资源 | 分类 | 概念 / 用途 |
|---|---|---|
| `document-new.svg` | Mono | 新建文档 / 工程 |
| `document-open.svg` | Mono | 打开文档 / 工程 |
| `document-save.svg` | Mono | 保存 |
| `document-print.svg` | Mono | 打印 |
| `edit-undo.svg` | Mono | 撤销 |
| `edit-redo.svg` | Mono | 重做 |
| `animation-preview.svg` | Mono | 动画预览 |
| `activate-split.svg` | Mono | 动画选项 |
| `binoculars.svg` | Mono | 搜索框右侧按钮（双筒望远镜） |
| `user-account.svg` | Mono | 账户 / 登录 |
| `shopping-cart.svg` | Mono | 购物车 |
| `help-browser.svg` | Mono | 帮助 |
| `menubar-globe.svg` | Mono | 语言 / 在线内容 |
| `caret-down.svg` | Mono | 下拉指示 |
| `caret-right.svg` | Mono | 搜索框左侧引导 |
| `pane-close.svg` | Mono | 关闭 / 清空 |
| `output-new.svg` | Mono | 新建输出 |
| `output-check.svg` | Mono | 检查输出 |
| `output-wizard.svg` | Mono | 输出向导 |
| `output-copy.svg` | Mono | 复制输出 |
| `output-image.svg` | Mono | 输出图像 |
| `output-export.svg` | Mono | 导出输出 |
| `output-delete.svg` | Mono | 删除输出 |
| `ribbon-start.svg` | Mono | 开始入口 |
| `ribbon-whatsnew.svg` | Mono | 新功能入口 |
| `ribbon-learn.svg` | Mono | 教程 / 学习入口 |
| `project-file.svg` | Mono | 工程文件 |
| `media-video.svg` | Mono | 视频 |
| `project-import.svg` | Mono | 导入 |
| `domain-dual.svg` | Mono | 双层面 |
| `geometry.svg` | Mono | 几何 |
| `mesh.svg` | Mono | 网格 |
| `molding.svg` | Mono | 注塑成型 |
| `analysis-sequence.svg` | Mono | 分析序列 |
| `material.svg` | Mono | 材料 |
| `injection-location.svg` | Mono | 注射位置 |
| `process-settings.svg` | Mono | 工艺设置 |
| `optimization.svg` | Mono | 优化 |
| `boundary-conditions.svg` | Mono | 边界条件 |
| `analysis-run.svg` | Mono | 运行分析 |
| `document-report.svg` | Mono | 日志 / 报告文档 |
| `job-manager.svg` | Mono | 作业管理 |
| `analysis-results.svg` | Mono | 分析结果 |
| `shared-views.svg` | Mono | 共享视图 |

## 流程与审查

1. 先查清单复用已有概念；新增项先登记，再写 SVG 与使用点。
2. 按 Theme 设置尺寸和颜色，给宿主补可翻译的语义与字典条目。
3. 从仓库根执行 `cargo build --locked`、`cargo lint qmllint`、`cargo format --check` 与相关 QtTest；使用实际 Qt 渲染检查，不以浏览器 SVG 预览替代运行期证据。
4. 检查 1.0/1.5/2.0 缩放、浅深背景、悬停/禁用/键盘焦点，以及整页窄窗口布局。

- [ ] 24 网格、统一描边与圆端，主体比例自然、无裁切或拥挤？
- [ ] Mono 无固定色、填色或细节堆叠，Art 分类和三色限制正确？
- [ ] 颜色实际传到像素，尺寸由 Theme/调用方提供？
- [ ] 装饰图标忽略读屏，操作按钮有可访问名称和焦点反馈？
- [ ] 清单、使用点、资源注册、字典与验证证据一致，重复图标已删除？
