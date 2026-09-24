# QML 页面 HTML 先行复刻

QML 页面设计的视觉参考件目录（任务 050 确立工作流）。每个状态一个子目录，
入口为 `<page>/<page>.html`；公共资源放在本目录，以 homepage 为视觉基准。

## 文件组织

- [shell.css](shell.css)：两页共用的设计 token、Ribbon、面板和工具栏样式。
- [shell.js](shell.js)：公共壳层、SVG 图标和页签选中态演示。
- [homepage/homepage.html](homepage/homepage.html)：启动 / 学习首页。
- [open-project/open-project.html](open-project/open-project.html)：工程任务项与项目工具 Ribbon。
- [imported-project/imported-project.html](imported-project/imported-project.html)：STL 导入完成后的工程树与视口状态。

HTML 只保留标题、菜单、Ribbon、任务、视口和对话框状态模板，脚本将模板嵌入公共壳层。
这些入口通过相对路径引用公共文件，无网络资源和构建步骤；浏览器需启用
JavaScript。可直接打开 HTML；复制参考件时应保留整个目录结构。

`homepage` 通过 New Project 任务入口演示新建项目弹窗；`open-project` 展示打开后的
工程树和多选 STL 导入确认弹窗，尚未导入 STL 时不显示 Layers Dock；`imported-project`
展示导入后的两个独立 Dock：上部工程 / 任务 Dock 内含工程树和零件任务两个分区，底部
保留工具按钮与 Layers 图标页签，内容暂空。同一工程中可展示多个 STL 和当前选中零件
的任务。这些页面只展示状态和交互边界，不写工程文件，也不声称已实现 STL 解析或视口渲染。

## 工作流

1. 取目标页面截图，确定布局与视觉语言；品牌、导航与文案全部使用 panta
   （英文源文案，与 022 口径一致），不出现第三方品牌信息。
2. 在公共壳层上编写状态模板，不复制公共 CSS 或 HTML。设计值（间距、字号、
   配色、圆角）集中在 `shell.css` 的 `:root`；图标集中在 `shell.js` 的 SVG
   symbol 中，以 use 引用。无网络字体、图片或脚本，照片 / 渲染图以 CSS 渐变
   占位并注明语义。
3. 浏览器渲染截图，与参照截图对照区块结构、布局与配色。
4. 后续 QML 页面任务以复刻件为底稿迁移：盒模型层级 → anchors/Layout，
   `:root` 设计值 → Theme token，占位图 → Image 元素；迁移完成后复刻件保留
   为设计档案。

## 边界

- 复刻件是文档参考资产，不进 QML 构建图，不参与任何质量门禁。
- 复刻件是布局/配色参考，不追求像素级逐帧校对；仅演示页签滑块与按钮反馈，
  不执行工程命令或切换真实业务视图，不实现移动端布局。
- 复刻对象里的内嵌网页视图等内容区可按需求留空（如桌面主窗口的 WebEngineView
  区域），迁移时映射为对应 QML 元素。

首页早期版本已于任务 029 按设计值迁移为 QML（`qml/App.qml` 与面板组件，见
[组件库与主题 DSL](../modules/qml-components-and-theme.md)）。本轮两状态整理见
[任务 059](../task/059-open-project-html-reference.md)，HTML 已经维护者手工验收，
作为 [QML 同步任务 060](../task/060-qml-project-workspace-reference.md) 的底稿。
布局与状态由 QML 实现，正式图标沿用仓库 Mono 规范，不直接复制彩色占位路径。
