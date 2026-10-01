# 096 — Results 工具栏内容

- 状态：done
- 阶段：应用平台扩展
- 依赖：060, 081, 095
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-10-02

## 目标与范围

Results 页签显示维护者截图中的结果工具分组，沿用现有 Ribbon 的主题、图标、滚动与组名栏。Home 的 Results 按钮也进入该页签。提供 Plots、Properties、Animation、Examine、Histogram、Scaling、Warpage、Export and Publish、Cutting Plane、Windows 与 Locking。

先交付 HTML 供 Yuki 确认，确认后迁移 QML 工具栏内容、英文文案及中文翻译，并同步 QML 图标资源。尚无结果数据与后处理服务，工具保持禁用，不伪造结果、时间范围或操作成功。动画播放、结果图、切面与锁定的领域命令不在本任务内。

## 必读

- [图标规范](../standards/icons.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 实施步骤

1. 登记任务并核对现有导航和 Ribbon 组件。
2. 先补 HTML 的 Results 内容及导航，Yuki 确认后迁移 QML；动画、缩放与窗口操作采用紧凑控件区，其余沿用图文工具。
3. HTML 阶段检查脚本语法与差异空白；QML 阶段再同步正式 SVG、资源和翻译，并运行构建与静态检查。

## 验收标准

- [x] 有工程时可从顶部 Results 页签和 Home Results 工具进入。
- [x] 截图工具分组完整，动画区域包含禁用按钮和滑块，Scaling 包含刻度尺及数值槽。
- [x] 工具未接入结果服务时禁用，未显示伪造数值或第三方产品入口。
- [x] 图标资源与 HTML symbol 对齐，主题与双语文案齐全。
- [x] 构建与静态检查结果记录，真实窗口验收单独记录。

## 验证与工作记录

- 2026-09-30：HTML 与 QML 已同步 11 个分组、Home / Results 导航、39 个 SVG 和双语翻译。首帧 / 尾帧图标对称、播放按钮居中，时间与比例值预留空间；紧凑工具复用 Theme 尺寸。
- 2026-09-30：整体评审修正比例尺的解码尺寸及禁用透明度，整理页签选择逻辑；导航只更新展示状态，无业务调用或循环信号链。
- 2026-09-30：`cargo build --locked`、`cargo lint qmllint --check`、`cargo lint cmake --check`、`cargo format --check`、`node --check ai-docs/qml-html/shell.js`、`git diff HEAD --check` 通过；未运行功能测试。
- 2026-09-30：维护者通过截图反馈并确认当前布局。Computer Use 绑定临时 PantaPreview.app 返回 `timeoutReached`，未取得自动化真实窗口证据；临时 wrapper 已清理，窗口验收待补。

## 清理与兼容例外

删除本轮误建的独立 Results 页面与跳转入口，Results 收回原工程页的 Ribbon 页签。无兼容例外。

## 完成摘要

HTML 与 QML 内容、图标和翻译已同步；构建、静态检查与 2026-10-02 真实窗口验收均通过，任务关闭。

## 2026-10-02 补充验收

同任务 102 的 macOS 实际应用窗口：创建临时工程后，顶部 Results 及 Home Results 均进入同一工具栏；11 个分组完整且组名对齐，动画首尾按钮对称、控制行居中、时间值留空，Scaling 刻度及数值槽不显示伪造数据，Windows 六格和 Locking 九格正常。AX 确认工具与动画滑块禁用。截图为 `artifacts/task-102/results.png`；构建通过，最新 CI run 36890328640 全部成功。详见 [102 GUI 与读屏验收](102-gui-and-screen-reader-acceptance.md)。
