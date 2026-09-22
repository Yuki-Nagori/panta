# 029 — QML 原子组件库与 Theme 尺寸参数化

- 状态：done
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-22

## 目标与背景

基于当前界面建立原子/组合组件，以明确输入属性和输出信号拼装面板；所有可配置视觉尺寸统一收敛到现存 Theme.qml。本轮以 050 复刻件
`ai-docs/qml-html/homepage/homepage.html` 为设计底稿完成组件提取与 Shell 框架拼装。

## 必读

- [组件与主题设计](../modules/qml-components-and-theme.md)
- [QML 规范](../standards/qml.md)、[Qt 规范](../standards/qt.md)
- [注释规范](../standards/comments.md)、[文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

本期迁移现有 UI、提取实际需要的组件、集中 token、提供组件展示与布局验证；不接 DSL，不预建完整控件大全。

## 前置条件与实施步骤

上述依赖产物可用后实施；029 无需等待正式模块迁移 026，030 无需等待热重载 027。涉及 FFI 时沿用 025 已落地的契约。

先盘点 qml/ 内视觉常量及重复控件，定义 token 和组件 API；再提取 Atoms/Composites 并迁移调用点；最后验证参数覆盖、布局和信号。保留 Qt Controls 的键盘与可访问性行为。

## 预计改动

现存 qml/、Theme.qml 及相关 CMake 文件；按本任务范围新增组件/测试或主题服务与 DSL 资源，目录以模块说明为规划依据，不创建空占位模块。

## 清理与兼容例外

同步清理被替换的控件、样式、默认值与调用点。无兼容例外，不长期保留双数据源或两套主题入口。本期删除 `ThemedButton.qml`（由 ThemedToolButton 替代）、无消费者的 `colorAccent`/`windowControlWidth` token、PanelSurface 圆角属性，以及 `revisionCount` 标签与“推进修订”条目（维护者决定，见工作记录）；字典同步清理失效词条（revision-caption/error-title 等）。

## 验收标准

- [x] 现有界面由职责明确的组件拼装，原子组件不依赖业务 ViewModel 或任意父级 id。
- [x] 所有可配置视觉尺寸以属性传入并默认绑定 Theme，面板无散布的视觉魔法数字；布局计算和纯算法常量有明确边界。
- [x] 参数覆盖有效，未覆盖参数随 Theme 更新；不出现 binding loop、布局冲突或失效绑定。默认值经组件测试断言与 Theme 单例一致、显式覆盖不破坏其余绑定；Theme 属性保持 readonly，运行期整体重发布属 030 的 ThemeViewModel 职责。
- [x] 真实窗口验证长文本、中英文、缩放、不同 DPI、焦点和禁用状态；现有命令行为保持。离屏渲染首帧（QT_SCALE_FACTOR 1.0/1.5）对照复刻件结构一致、缩放无错乱；窄窗口下标题 elide 收敛；禁用弱化有组件测试；焦点/禁用语义由 ToolButton 承接并补 visualFocus 反馈。无 GPU 真机窗口因本会话无屏幕录制权限未截图，由维护者日常运行复核。
- [x] 移除被替代的内联控件、重复样式与尺寸常量，Theme 仅有一份权威默认值。
- [x] 代码、资源、测试、文档及索引一致，验证记录完整。

## 验证计划与结果

实施时执行现有 all_qmllint、相关构建和实际新增行为测试，并在真实窗口验证布局；030 增加 schema 正反例、快照失败保持、主题选择与重启验证。记录 cwd、工具链和实际命令，不预设测试入口存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | `cmake --preset debug` → `cmake --build build/debug` | 预编译 Qt 已缓存后配置与构建成功；生成三个原子组件并完成 QML cache 编译 |
| 2026-09-16 | `cmake --build build/debug --target all_qmllint` | 命令成功；当时仅有既存 `Panta.Bridge` 手动注册类型不可见警告，无新增组件错误；该限制已由 026 在 2026-09-17 消除 |
| 2026-09-16 | Qt 6.11.2 `qmlformat` 输出与 `qml/` 文件逐个 diff | 新增及迁移的 QML 文件格式一致 |
| 2026-09-16 | `ctest --test-dir build/debug --output-on-failure` | 6/6 native tests 通过，含 `Qml.ThemeComponentParameters` |
| 2026-09-16 | `cargo build --locked`、`cargo test --locked` | Rust 构建成功；4/4 launcher tests 通过 |
| 2026-09-16 | `native/build/debug/app/panta-native --version`、未知参数冒烟 | 版本输出正确；未知参数以 64 退出并给出用法诊断 |
| 2026-09-16 | `QT_QPA_PLATFORM=offscreen` 启动 2 秒后终止 | 事件循环保持运行且无 QML 加载错误输出；因无头进程不会自行退出，按测试时限终止 |
| 2026-09-22 | macOS / cargo 1.98.1 / CMake 4.4.3 / Qt 6.11.2（repo 根）`cargo build --locked` | 供给新增 qtsvg 后配置与构建成功；staging 出现 `plugins/imageformats/libqsvg.dylib` 与 QtSvg.framework，`qt/extracted/` 解包指纹就位 |
| 2026-09-22 | `cmake --build target/native/debug --target all_qmllint` | 0 警告；过程中修复 URL 严格比较、布局子项 `Layout.preferred*`、`pragma ComponentBehavior: Bound`、`Label.ElideRight` 作用域四处告警 |
| 2026-09-22 | `ctest --test-dir target/native/debug` | 49/49 通过：`Qml.ThemeComponentParameters` 覆盖 token 默认值断言、覆盖隔离、禁用弱化透明度、`QImageReader` 解码模块内 SVG（qsvg 插件链路）；`Qml.ShellModuleLoads` 覆盖框架装配后的 objectName 契约；`Qml.FormatCheck` 通过 |
| 2026-09-22 | `PANTA_SHELL_CAPTURE_PATH=…` 离屏抓帧，QT_SCALE_FACTOR=1.0 与 1.5 各一帧 | 两帧与 050 复刻件区块结构一致（顶部 chrome/ribbon/任务与输出面板/VTK 视口与底部页签/状态栏）；1.5 缩放图标清晰、无布局错乱；800px 窄窗口下居中标题按 elide 收敛不与搜索框重叠 |
| 2026-09-22 | 布局诊断（`PANTA_SHELL_DUMP_GEOMETRY=1`） | 发现嵌套 Layout 默认最大宽为隐式宽导致 fillWidth 列展不开、剩余空间错派给左栏；工作区改 anchors 锚定后左栏恢复 `max(26%, 320px)` |
| 2026-09-22 | 文案英文化 + 字典同步后 `cargo build`、ctest、离屏抓帧 | `.pa` 解析与 TS/QM 生成通过；`I18n.CompiledQmLoads` 更新为按组件上下文断言（App/TopChromePanel/RibbonPanel/ViewportPane）；49/49 通过；抓帧确认全部显示文本为英文，zh-CN 译文经 QM 查找断言 |
| 2026-09-22 | 维护者微调：ToolGroup 去除左右留白，激活动图与 split 箭头移入 ToolGroup 并加结构包裹（分隔线），`cargo build` + ctest 49/49 + 离屏抓帧确认 |

## 风险与回退

尺寸集中化可能暴露固定布局假设，主题值发布可能破坏绑定；先用现有面板验证最小组件，再扩大迁移。主题切换失败保持最后有效快照；文件保存失败不覆盖用户原文件。

## 决策与工作记录

- 2026-09-16：依据用户原子组件与 DSL 主题需求，由 028 编排。
- 2026-09-16：组件测试以 C++ QtTest 加载模块内 QML 资源，避免 qmltestrunner 未链接 `panta_shell` 时无法解析模块资源；测试纳入 CTest。
- 2026-09-22：维护者指示以 050 复刻件 `homepage.html` 为设计底稿完成本任务；`:root` 设计值整体迁入 Theme 作为权威默认。
- 2026-09-22：维护者决策图标走 SVG 资源而非 QtQuick.Shapes 逐个绘制（性能与资源化映射），为此扩展 Qt 供给三平台 qtsvg 归档（SHA256 经官方 .meta4 交叉核对，见 [dependency-acquisition](../standards/dependency-acquisition.md)）；ThemedIcon/ThemedToolButton 经模块内相对 qrc 路径取图。
- 2026-09-22：本任务交付时维护者决策窗口控制不自绘、交系统标题栏（复刻件中的最小化/最大化/关闭按钮不迁移）；经查供给 Qt 6.11.2 头文件无 `QWindow::setTitleBar`。后续无边框外观与标题栏融合由 [051](051-integrated-window-titlebar.md) 承接，评估已存在的扩展客户区 API，不以未来新增该接口为前提。
- 2026-09-22：Shell 自绘控件要求非原生样式，主入口 `QQuickStyle::setStyle("Basic")` 固定，与 ctest 既有 `QT_QUICK_CONTROLS_STYLE=Basic` 同源。
- 2026-09-22：工作区（左栏/分隔线/VTK 列）用 anchors 直接锚定：嵌套 Layout 默认最大尺寸为自身隐式尺寸，fillWidth 列展不开且剩余空间分派不可预期（有几何 dump 证据）。
- 2026-09-22：维护者决定删除 `revisionCount` 标签；同步取消 shell 加载测试中该断言，修订计数行为由 bridge ViewModel 测试继续覆盖。
- 2026-09-22：维护者决策显示文案全部使用英文源 + `qsTr()`，中文译文登记进
  `resources/i18n/panta-{en,cn}.pa`（上下文按 QML 组件命名；`.pa` 解析器要求
  同上下文源文本长度互异，个别措辞据此调整为 Task List/Open Project/Learning）。
- 2026-09-22：维护者决定删去“推进修订”任务项；tick 命令不再有 UI 触发，行为由
  bridge ViewModel 测试继续覆盖，shell 加载断言与字典词条同步移除。
- 2026-09-22：shell 加载测试常驻布局取证能力（`PANTA_SHELL_CAPTURE_PATH` / `PANTA_SHELL_DUMP_GEOMETRY` 环境变量门控），供后续布局优化对照。

## 完成摘要

QML 侧已按 050 复刻件完成组件库与 Shell 框架：Theme 以复刻件 `:root` 设计值为权威默认（chrome/menubar/ribbon 配色、13/13/12 字号、30/24/84/26 条带高度、间距刻度、图标/圆角/栏宽比例 token）；原子层 ThemedLabel、ThemedToolButton（icon/弱化后缀/caret/包边/选中态/禁用弱化/visualFocus）、ThemedIcon、PanelSurface，组合层 ToolGroup、RibbonTile、PanelTabBar（上下两态）、PaneCloseButton，面板层 TopChrome/Ribbon/Tasks/Output/ViewportPane，App.qml 以 anchors 工作区拼装为复刻件同构桌面框架。图标为转写自复刻件的 31 个模块内 SVG（qtsvg 供给渲染），Qt 供给 manifest 三平台登记 qtsvg。显示文案全部为英文源 + `qsTr()`，中文译文登记进 `resources/i18n/panta-{en,cn}.pa`（022 接入运行期加载）。ShellViewModel 的 caption 与错误展示绑定保持，窗口控制交系统标题栏，`revisionCount` 标签与“推进修订”任务项按维护者决定移除（tick 行为由 bridge ViewModel 测试覆盖）。参数覆盖与默认绑定、禁用弱化、SVG 解码均有组件测试；离屏首帧 1.0/1.5 缩放对照复刻件一致。030 接手主题 DSL 与运行期切换时，Theme 属性名即稳定契约。
