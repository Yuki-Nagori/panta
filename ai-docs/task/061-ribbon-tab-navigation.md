# 061 — Home 与 Start & Learn 工具栏切换

- 状态：done
- 阶段：应用平台扩展
- 依赖：[060](060-qml-project-workspace-reference.md)、[057](057-new-project-dialog.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

060 已提交并验收首页 / 工程布局，顶部菜单仍固定首项高亮。维护者要求继续实现 Home ↔ Start & Learn 点击切换：只替换 Ribbon 内容及选中态，不关闭工程、不重建 Tasks 或视口。

## 必读

[QML 规范](../standards/qml.md)、[组件与主题模块](../modules/qml-components-and-theme.md)、[注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[验证与评审](../standards/validation-and-review.md)、[代码生命周期](../standards/code-lifecycle.md)。

Qt 6.11.2 官方依据（2026-09-22）：[QQuickItem::ensurePolished](https://doc.qt.io/qt-6/qquickitem.html#ensurePolished) 用于在读取 Positioner 的几何之前完成其延迟布局；仅在测试命中前使用，不向生产导航加入延时。

## 范围与非目标

- App 持有独立于工程快照的活动 Ribbon 页签，并统一校验导航请求；面板显式接收选中态，菜单仅报告稳定 key，不内置页面路由规则。
- 未打开工程时保持 Start & Learn；新建 / 打开成功（包括再次打开同一工程）后选中 Home。
- 工程已打开时，Home 的 18 个工具与 Start & Learn 的 7 个工具可反复切换，扩展菜单、工程名、dirty 状态、Tasks 与视口实例保持不变。
- 改名 / 保存不改变活动页签，打开 / 创建失败保留原工程和选中态。Start & Learn 的新建 / 打开继续调用既有对话框。
- 补真实鼠标、键盘激活与窄窗口回归测试，更新当前模块说明。
- 按维护者追加要求审查软件工程边界：状态单一来源、面板信号驱动、业务快照与展示状态解耦，测试等待真实布局完成，不以延长超时掩盖时序缺陷。

非目标：不实现其他菜单、工具业务或工程关闭功能，不修改已验收视觉与 HTML 状态参考件，不改 C++ / Rust 工程服务接口，不新建 target 或切换工具链；完成后按维护者追加授权单独提交，不 push。

## 实施步骤与清理

1. 登记任务，核查 ProjectViewModel 的成功信号及现有菜单 / Ribbon 测试。
2. 移除首项固定高亮和 Ribbon 对 projectOpen 的耦合，替换为显式活动页签；用既有 projectCreated / projectOpened 信号选择 Home，避免改名 / 保存触发导航。
3. 补点击、键盘、状态保留及新建 / 打开入口测试，沿用现有 target 验证。
4. 更新模块说明、验收结果及索引。无兼容例外。

## 验收标准

- [x] 鼠标和键盘均可在 Home / Start & Learn 间切换，选中态与 18 / 7 工具内容一致。
- [x] 无工程时保持开始页；成功创建 / 打开选择 Home，失败、改名和保存不改变用户当前选择。
- [x] 切换不改变工程快照、不重建 Tasks / 视口；其他菜单不误触导航，新建 / 打开入口仍可用。
- [x] 1440 / 640 逻辑像素窗口下切换后焦点可见、滚动边界正确，布局及四档缩放无回归。
- [x] 构建、格式、QML / C++ 检查和相关测试通过，记录 Cocoa 真窗口结果；文档与索引同步。

## 验证计划与结果

全部命令在仓库根目录执行：macOS 26.3.1 arm64，沿用 target/native/debug、target 中的 Qt 6.11.2 / LLVM 22.1.7，无新构建目录或工具链。

| 场景 / 命令 | 最终结果 |
|---|---|
| cargo build --locked | 通过 |
| cargo format --check | 通过；uv 使用已验证的沙箱外环境 |
| cargo lint qmllint --check、cargo lint includes --check、cargo lint clang-tidy --check | 全部通过 |
| target 内 CTest --test-dir target/native/debug --output-on-failure | 54/54 通过；Shell 含宽 / 窄窗口、鼠标 / 键盘四组导航测试 |
| CTest -R '^Qml\.ShellModuleLoads$' --repeat until-fail:3 | 连续三次通过，未调整现有超时 |
| QT_QPA_PLATFORM=offscreen，QT_SCALE_FACTOR=1 / 1.25 / 1.5 / 2，运行 panta_qml_shell_module_test | 每档 9/9 通过；offscreen 按现有规则跳过原生 VTK surface |
| QT_QPA_PLATFORM=cocoa，运行 panta_qml_shell_module_test | 9/9 通过；鼠标、键盘、模态窗口返回及原生视口生命周期均完成；销毁时 VTK 输出正常 Destroyed 信息 |
| Qt 截图检查 | 已查看 Cocoa 和 1.25 倍 start-learn.png，学习工具栏、扩展菜单和实际工程项共存；截图位于 artifacts/qml/task061/{cocoa,scale-*}/ |
| git diff --check | 通过，无额外资源、依赖或构建配置改动 |

初版真实事件测试失败：菜单刚创建时 Row 尚未 polish，模态窗口尚未完成焦点交接时就注入下一组输入。测试改用 ensurePolished 和明确的 focusWindow 条件，并断言每次激活恰好产生一次 clicked。未放宽 CTest 超时，未向生产逻辑加入延时。Windows / Linux 真窗口及 CI 本轮未运行。

## 风险与决策记录

- 2026-09-22：创建任务。ProjectViewModel 的 projectChanged 同时通知路径 / 名称 / dirty 变化，不能用于无条件重置页签；改用现有创建 / 打开成功信号。
- 2026-09-22：活动页签仅为 Shell 展示状态，不写工程文件；失败保留当前页面。沿用两套已验收 Ribbon 数据，不引入通用路由器或额外 ViewModel。
- 2026-09-22：按维护者“符合软件工程”的要求，将菜单点击和工程成功信号收拢到 App.selectRibbonTab，统一校验可达页签；面板只接收状态、发出 key，删除 Ribbon 对 projectOpen 的旧依赖和首项固定高亮。
- 2026-09-22：测试验证未保存工程往返切换不产生 projectChanged、Tasks / 视口对象未销毁，以及失败、改名、保存、同路径重开、新建第二工程和取消新建对话框；组件测试复验切回后的新建 / 打开信号仍有效。
- 2026-09-22：构建、格式、三项 lint、54 项 CTest、四档缩放及 Cocoa 实际窗口通过，当前实现留工作区供维护者体验，未 commit / push。
- 2026-09-22：维护者要求先提交 061。单独提交已验证的导航实现、测试与文档；062 页签组件拆分另行推进，不混入本次提交。

## 完成摘要

Home ↔ Start & Learn 切换已完成。活动页签由 App 单一持有并统一校验，选中态和工具内容同步；工程快照、Tasks 及 VTK 视口保持独立。成功创建 / 打开回到 Home，其他工程变更或失败不打断当前页签。代码、注释、测试和模块说明同步，验证通过；按维护者追加授权单独提交，不包含其他菜单业务或 062 拆分，不 push。
