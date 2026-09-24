# 052 — QML 图标规范与首页布局优化

- 状态：done
- 阶段：应用平台扩展
- 依赖：[029 组件库](029-qml-component-library.md)、[050 HTML 参考](050-qml-html-page-replica.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与范围

对照 `ai-docs/qml-html/homepage/homepage.html` 修正顶部工具分组、搜索和输出工具条，并重新设计 SVG。新增适用于 Qt Quick 的 Mono/Art 图标规范与完整清单；操作图标统一网格、描边与运行期着色，保留现有业务状态。标题栏融合仍由 051 承接，本任务不接入新业务或主题 DSL。

## 必读

[QML](../standards/qml.md)、[Qt](../standards/qt.md)、[注释](../standards/comments.md)、[文档](../standards/documentation.md)、[文件规范](../standards/repository-hygiene.md)、[测试](../standards/testing.md)、[验证](../standards/validation-and-review.md)、[生命周期](../standards/code-lifecycle.md)、[提交](../standards/commits.md)、[组件模块](../modules/qml-components-and-theme.md)。

## 实施与预计改动

1. 新增 `ai-docs/standards/icons.md`，登记现有资源，明确 HTML 的布局参考与图标新规范的边界。
2. `qml/icons/` 重绘并复用重复概念；QML 图标颜色由调用方传入，使用公开 Qt API 实现资源着色，不依赖浏览器 CSS 继承或 Qt 私有类型。
3. 调整 `qml/Components/`、`qml/Panels/` 和 Theme；按需在 Shell 增加 SVG 着色适配及构建/入口注册。
4. 更新已有 QtTest 检查图标解码、着色与按钮状态，实际渲染核对 1.0/1.5/2.0 缩放及窄窗口。

## 清理与兼容

移除同概念重复 SVG、固定色值与失效文档；不保留旧版实现，无兼容例外。Qt 版本不变。SVG 为手工几何矢量资源，不使用图像生成工具。

## 验收

- [x] 规范有 Mono/Art 分类、Qt 颜色/尺寸/可访问性规则、图标清单和审查项。
- [x] 操作 SVG 使用 24 网格、统一线宽/圆角，不含硬编码主题色；重复概念复用。
- [x] 图标颜色随调用方变化；关闭按钮悬停白图标、菜单深色背景正确显示；图标按钮提供可访问名称。
- [x] HTML 对照的顶部工具分组/搜索/输出工具条修正，窄窗口无重叠；不新增业务行为。
- [x] 构建、QML lint、格式及相关测试通过；图标和整页缩放渲染目视核验完成。
- [x] 文档、索引和实际验证同步，完成后按范围提交且不 push。

## 验证记录

环境：仓库根目录，macOS 26.3.1 arm64，Qt 6.11.2；构建工具由 Cargo 固定供给。

- 2026-09-22：现有 Shell 离屏测试通过，基线截图 `artifacts/052/before.png`；仅离屏 UI 证据，不代表真实 GPU/窗口验收。
- 2026-09-22：`cargo build --locked`、`cargo lint qmllint`、`cargo lint clang-tidy`、`cargo format --check` 通过。首次 `cargo format` 的 uv 因沙箱系统配置访问崩溃，获准在沙箱外运行同一入口后通过。
- 2026-09-22：`cargo test --locked --workspace` 通过，含 49 项 native/QML CTest。组件 QtTest 验证所有 SVG 的资源解码、16/18/24/48 像素着色和外缘透明、非法请求拒绝、动态颜色、禁用状态；Shell 测试在 1440/640 宽验证分组无溢出与聚焦搜索自动滚动。
- 2026-09-22：离屏截图 `artifacts/052/after-1440.png`、`after-150.png`、`after-200.png`、`after-640.png` 逐张核对。标题组齐边、搜索框完整、望远镜在右、重复账户已删除、地球/箭头合一；宽窗口完整展示，窄窗口标题区可横向滚动，150%/200% 图标清晰。离屏平台跳过原生 VTK 初始化，未将此记为 GPU/三平台窗口验收；本次不改 VTK 或 051 的窗口机制。
- 2026-09-22：新增语义按动作使用 `qsTranslate` 上下文，并同步中英 `.pa` 字典（规避现有同上下文 source 等长限制，不修改解析器）；两份字典经实际 DSL formatter 验证。
- 过程修正：运行期发现 Image 的 implicitHeight 不可写，改回显式逻辑尺寸；焦点可见性测试发现缩窄窗口后未滚动，改为布局完成后同时响应聚焦和宽度变化；回归通过。

- 2026-09-22：最终紧凑 Ribbon、无底槽等宽滑块及上下留白修正后，增量构建、Shell QtTest 与 1.0/1.5/2.0 倍截图再次通过；640px 窄窗回归通过。`cargo lint includes --check`、qmllint、clang-tidy 与格式检查通过。

## 风险与回退

SVG 的 currentColor 不从 QML 自动继承；必须验证实际像素颜色而非仅属性。紧凑标题区需保证常用动作可达，窄窗口应允许工具区域横向滚动。失败时撤回本任务改动，保留原有 Shell 和工程数据。

## 工作记录与完成摘要

- 2026-09-22：创建任务并登记索引后实施；051 标题栏规划已单独提交。保留同期提交 715f645 的 ToolGroup 去边距行为。
- 统一 Mono SVG、运行期主题着色与可访问名称；删除重复打开/保存/新建/清空/箭头及不再使用的放大镜资源，同步规范、图标清单、中英文案和构建注册。无兼容例外，不新增业务命令或 Art 资源。
- 顶部搜索框完整显示，右侧望远镜有统一颜色和悬停反馈；删除重复账户，地球与下箭头合为一个按钮；菜单选中态直角。工具按钮图文与悬停区域共用中心，关闭按钮正确定位；文字入口不再弹重复 tooltip。
- 最终保留无独立底槽的滑块页签，每项固定 96px，窄栏等分可用宽度；选中加粗不改变宽度，悬停与选中面共用内缩。任务栏预留关闭按钮位置，任务列表移除顶部空隙。
- Ribbon 三入口缩为 64×64px、图标 24px、文字 10px、图文间距 6px，图文整体居中；New 使用简短文案。磁贴上下各 2px，底部分隔线独占 1px；整栏高度由这些尺寸推导为 69px。
- 几何回归覆盖页签不越界、切换加粗宽度稳定、工具按钮/磁贴内容居中、磁贴上下留白相等和搜索按钮悬停；仅本地提交，不 push。
