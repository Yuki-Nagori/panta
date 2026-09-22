# 原子组件库、Theme 与主题 DSL（029 已落地组件层；DSL 为 030 规划）

[模块导航](README.md) · [QML 规范](../standards/qml.md) · [029 组件库](../task/029-qml-component-library.md) · [030 主题切换](../task/030-theme-dsl.md)

## 当前状态与目标

`qml/Themes/Theme.qml` 是颜色、间距、字号和结构尺寸的唯一 QML 门面，默认值即 050 复刻件 homepage.html 的 `:root` 设计 token（029 迁入）；沿用大写 `Theme.qml`，符合仓库 QML 类型命名约定。属性保持 readonly，运行期默认值迁 `.pa` 与切换由 030 的 C++ ThemeViewModel 发布，Theme 属性名即稳定契约。

029 已把 Shell 拼装为复刻件同构桌面框架（顶部 chrome、ribbon、任务/输出面板、VTK 视口、状态栏）；后续页面按同一套 token 与组件拼装，不新增第二套视觉常量。

## 页面设计工作流：HTML 先行复刻

新页面的视觉开发采用 HTML 先行复刻工作流（维护者 2026-09-21 确立）：先在
`../qml-html/<page>/` 用自包含 HTML 复刻目标页面——品牌与文案统一 panta，设计值
集中在 `:root` CSS 自定义属性，照片/渲染图以渐变占位——浏览器对照确认后，再由
页面任务迁移为 QML：盒模型层级映射 anchors/Layout，`:root` 设计值映射 Theme
token，占位图映射 Image 元素。这样把视觉迭代与 QML 实现解耦，效率最高。复刻件
是文档参考资产，不进 QML 构建图；约定与边界见
[`../qml-html/README.md`](../qml-html/README.md)，首个样例为
panta 桌面主窗口框架（任务 050 复刻、029 迁移）。

## 组件分层与输入输出

| 层次 | 位置 | 组件 |
|---|---|---|
| 主题契约 | `qml/Themes/Theme.qml` | 唯一 QML token 门面：颜色、间距、字号、条带高度、控件/图标尺寸、圆角、线宽、栏宽比例、窗口最小尺寸 |
| 原子组件 | `qml/Components/Atoms/` | `ThemedLabel`、`ThemedToolButton`（icon/弱化后缀/caret/包边/选中态/禁用弱化）、`ThemedIcon`（模块内 SVG）、`PanelSurface` |
| 组合组件 | `qml/Components/Composites/` | `ToolGroup`（标题条渐变分组）、`RibbonTile`、`PanelTabBar`（顶/底两态页签）、`PaneCloseButton` |
| 业务面板 | `qml/Panels/` | `TopChromePanel`、`RibbonPanel`、`TasksPanel`、`OutputPanel`、`PlaceholderPanel`（无 Bridge 变体用） |
| 页面与外壳 | `qml/App.qml`、`AppNoBridge.qml` | 布局、导航、面板装配与主题选择入口 |

组件可以封装 Qt Quick Controls，保留其焦点、键盘、禁用与可访问性行为；不为了“原子化”重新实现所有底层控件。避免为单次无独立职责的布局建立空壳组件。只提取当前界面实际使用的组件，不预建无用途库。

原子/组合组件公开语义清楚的属性与信号，例如 label、iconName、controlHeight、contentPadding、advanceRevisionTriggered。尺寸属性默认绑定 Theme token，调用方可通过属性覆盖；组合组件向内部原子组件显式传入参数，不依赖父级 id、parent 链或隐式全局业务状态。不用 imperative 赋值覆盖 token 绑定，保证换主题后未覆盖值能继续更新。

组件默认宽高通过内容及输入参数计算 implicit size，外层布局决定实际分配空间。布局拥有尺寸时，组件不要同时强制 anchors 和固定 width/height。跨区块的工作区框架（左栏/分隔线/VTK 列）用 anchors 直接锚定：嵌套 Layout 的默认最大尺寸是自身隐式尺寸，fillWidth 列展不开且剩余空间分派不可预期（029 有几何 dump 证据）。点击仅发语义信号，面板再调用 ViewModel。

Shell 自绘控件（自定义 background 等）不能运行于原生 Controls 样式：主入口以 `QQuickStyle::setStyle("Basic")` 固定 Basic，ctest 环境同源（029）。

图标为模块内 SVG 资源（`qml/icons/`，经 `qt_add_resources` 登记到 `/qt/qml/Panta/Shell/icons/`），由 Qt 供给的 qtsvg（qsvg 图像格式插件）渲染；颜色固定在资源内，主题化图标资源待 030 后评估。

布局取证（`tests/cpp/app/shell_module_load_test.cpp`，029 起常驻）：环境变量
`PANTA_SHELL_CAPTURE_PATH` 指向目标 PNG 时保存 Shell 首帧渲染，
`PANTA_SHELL_DUMP_GEOMETRY` 置非空时打印内容树前三层几何；ctest 默认不设置、
无副作用。排查布局时的示例如下（仓库根目录，先 `cargo build --locked`；
`QT_SCALE_FACTOR` 可模拟不同 DPR）：

```sh
STAGING=target/panta-deps/qt/staging
QT_QPA_PLATFORM=offscreen \
QT_PLUGIN_PATH="$PWD/$STAGING/plugins" \
QML_IMPORT_PATH="$PWD/$STAGING/qml" \
PANTA_SHELL_CAPTURE_PATH=/tmp/panta-shell.png \
PANTA_SHELL_DUMP_GEOMETRY=1 \
QT_SCALE_FACTOR=1.5 \
  target/native/debug/app/panta_qml_shell_module_test
```

## 尺寸集中化规则

Theme 集中管理 spacing、padding、radius、borderWidth、iconSize、fontSize、controlHeight、条带高度（titlebar/menubar/ribbon/statusbar）、leftPanelRatio/MinimumWidth、outputPanelRatio、windowMinimumSize 等可配置视觉值。采用语义 token，例如 `controlHeight`，而不是在各组件重复相同数值。

所有视觉尺寸默认值只在 Theme 的权威默认配置中定义；组件声明输入属性绑定 token，面板若覆盖也应传 Theme token 或根据可用空间计算的值，不另写视觉魔法数字。`0`（无间距）、比例/计数等纯算法常量可以保留，但有视觉设计含义的非零偏移也必须使用 token。width/height 的父布局绑定和内容测量不应改成固定主题尺寸。

UI 尺寸统一采用 Qt Quick 逻辑像素语义，不自行乘设备像素比；图标栅格资源分辨率与控件布局尺寸区分。高 DPI、字体变化、英文/中文长文本和小窗口下仍需验证布局。业务数值、网格单位及几何尺寸不属于 Theme。

## Theme 与 DSL 的单向数据流

029 阶段：保留 Theme.qml 中的内置默认 token，先完成参数化组件，无需等待 DSL。

030 阶段：把可配置默认值迁入内置主题 DSL；Theme.qml 保留稳定的属性名和到 C++ ThemeViewModel 的只读绑定，作为唯一 QML 访问入口。颜色、尺寸默认值不再同时维护于 QML 与 DSL。主题 schema 管键名、类型、范围和映射，不复制一套默认值。

`.pa` 按职责分层，而不是按控件拆散：

- `gui.pa`（`kind: variables`）保存不随主题改变的 GUI 设计 token，例如窗口最小尺寸、布局间距、控件高度和字号的基线。
- `light.pa`、`dark.pa` 等（`kind: theme`）只保存可覆盖的主题 token，例如颜色、圆角、阴影开关，必要时覆盖允许主题化的字号或间距。
- 合并顺序固定为 `gui` 基线 → 选中的主题覆盖 → schema 校验 → 完整快照；覆盖文件不直接被 QML 读取，也不把未声明的键静默注入 Theme。

这样“换主题”是选择另一份 `.pa` 覆盖，不是复制一份 QML。平台 surface 的运行时位置、实际 item 宽高和 DPR 仍由 C++ 原生适配器计算，不能写成 `.pa` 固定坐标；VTK 相机/几何默认值也不属于 GUI Theme，除非未来明确纳入独立的渲染配置域。

规划流向：内置默认主题 DSL + 用户选择的主题覆盖 → 共用 DSL 解析器 → 主题 schema 校验与完整快照 → C++ ThemeViewModel → Theme.qml → 组件输入属性 → 拼装界面。QML 不解析 DSL、不访问文件、不直接连接 Rust；沿用服务经 C++ ViewModel 暴露的边界。

示意主题文本（`.pa` 文法沿用 025；键映射尚未实现）：

```text
version: 1
kind: theme

values:
  spacing-small: real = 8
  control-height: real = 32
  font-body: real = 14
  color-background: string = #1e1f22
```

映射示意：`spacing-small` → `Theme.spacingSmall`，`control-height` → `Theme.controlHeight`。键名映射由单一 schema 明确定义，不按字符串猜测属性。颜色首期用 string 加主题专用校验，不要求 025 引入 color 类型；数值在 DSL 层仍无量纲，主题 schema 将特定键解释为逻辑像素，与 CAE 物理单位无关。业务 key 统一使用短横线，生成到 C++/QML 属性时才按 schema 映射为 camelCase。

应用主题作用域独立于工程变量，复用解析器和不可变快照机制，但不要求打开工程，不随工程关闭销毁，切换也不把工程标记为 dirty。主题覆盖缺省键继承内置默认值；未知键、错误类型、非法颜色、非有限或越界尺寸拒绝整个覆盖。内置默认主题本身必须键齐全且通过构建/测试校验。

## 切换、编辑与持久化

候选主题先离线解析、合并和校验，再在 GUI 线程发布完整快照，统一触发属性刷新；不得在校验过程中逐字段污染当前主题。快速连续选择以请求 generation 拒绝迟到结果，切换失败保留旧主题与偏好并呈现可翻译诊断。成功后才保存主题 ID/配置位置，保存失败单独报告，不能声称重启偏好已更新。

首次启动使用内置默认主题；重启遇到用户主题缺失或损坏时回退内置主题并提示，不覆盖损坏的用户文件。正常切换通过属性绑定更新，无需销毁 QML 引擎，也不依赖 027 热重载。

若以后提供主题编辑器，UI 修改经显式服务命令校验后写入主题 DSL，并重新发布快照；不得观察 Theme 属性再自动反写 DSL，避免绑定回环。本期 030 只交付读取、选择与应用，不含主题编辑器、动画切换或任意 QML 注入。
