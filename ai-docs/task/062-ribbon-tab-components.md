# 062 — Ribbon 页签内容与公共渲染拆分

- 状态：done
- 阶段：应用平台扩展
- 依赖：[061](061-ribbon-tab-navigation.md)、[060](060-qml-project-workspace-reference.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

Home / Start & Learn 导航已完成，但 RibbonPanel 仍包含两套完整工具定义、通用渲染和具体动作分发。维护者确认按页签拆分，使新增 / 修改页签内容不再扩大公共面板的数据和条件分支。

## 必读

[QML 规范](../standards/qml.md)、[组件与主题模块](../modules/qml-components-and-theme.md)、[注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)、[验证与评审](../standards/validation-and-review.md)。

Qt 6.11.2 官方依据（2026-09-22）：[Loader](https://doc.qt.io/qt-6/qml-qtquick-loader.html) 的组件切换、尺寸跟随和焦点作用域规则。使用静态 Component 引用，不通过字符串拼接资源路径或忽略未知信号。

## 范围与分层

- RibbonPanel 保留公共背景、横向滚动、当前页签选择及对宿主的信号接口；不保存各页签的工具数据。
- 新建 qml/Panels/Ribbon/HomeRibbon.qml 与 StartLearnRibbon.qml，分别维护分组、工具、启用条件及具体命令映射。
- 新建 qml/Components/Composites/RibbonContent.qml，共用分组 / 工具渲染，复用 RibbonGroup / RibbonTile；只发出工具 key，不认识工程命令。
- 用 Loader 仅实例化当前页签；不保留非活动页签的隐藏控件与焦点。App 继续单独持有导航状态和工程 / 对话框命令。
- 注册模块资源，补各页签独立加载和切换释放测试；保持图标、翻译上下文、尺寸、排序及已有命令行为不变。

非目标：不实现额外工具业务，不修改 HTML 视觉底稿、C++ / Rust 服务或导航规则，不引入插件注册框架、预留组件或新工具链；继续使用现有 target。061 已单独提交；062 验收后按维护者追加授权单独提交，不 push。

## 实施步骤与清理

1. 登记任务，确认现有工作区与两套工具内容基准。
2. 提取公共渲染和两套页签，将外壳切换到静态组件加载。
3. 删除原面板的重复模型 / 命令分支，补模块登记和独立资源 / 生命周期测试。
4. 验证命令路由、切换状态保留、长文本 / 窄窗口 / 缩放及 Cocoa 实际窗口，更新模块与任务记录。

无兼容例外；不保留旧模型、副本或备用渲染路径。

## 验收标准

- [x] 外壳、公共渲染、页签内容职责独立；现有视觉和 7 / 18 工具定义不变。
- [x] 页签可独立加载，公共渲染不认识 new / open 等业务命令；当前页签信号只触发一次。
- [x] 切换释放旧页签，不遗留隐藏工具或焦点；App、工程快照、Tasks 与视口保持既有行为。
- [x] 格式、构建、QML / C++ 检查、全量 native 测试、四档缩放和 Cocoa 实际窗口通过。
- [x] 资源、注释、模块说明、任务与索引同步，无旧实现残留。

## 验证计划与结果

全部命令在仓库根目录执行：macOS 26.3.1 arm64，沿用 target/native/debug、Qt 6.11.2 和 target 内 LLVM 22.1.7，未新建 target 或更换工具链。

| 场景 / 命令 | 最终结果 |
|---|---|
| cargo build --locked、cargo format --check | 通过；格式检查使用已验证的沙箱外 uv 环境 |
| cargo lint qmllint --check、cargo lint includes --check、cargo lint clang-tidy --check | 全部通过；新增焦点测试已补直接依赖的 QtCore/qnamespace.h |
| target 内 CTest --test-dir target/native/debug --output-on-failure | 54/54 通过 |
| CTest -R '^Qml\.(ShellModuleLoads\|ThemeComponentParameters)$' --repeat until-fail:3 | Shell 和组件测试各连续三次通过 |
| 页签独立加载 / 命令路由 | 两个资源独立创建成功，6 / 3 分组及 18 / 7 工具、固有尺寸和通用 key 信号正确；切回后新建 / 打开请求各恰好发出一次 |
| Loader 生命周期 | 同页签重选复用当前实例；带工具焦点切换后旧页签 QPointer 清空，旧工具从可视树消失，宽度跟随当前内容 |
| QT_QPA_PLATFORM=offscreen，QT_SCALE_FACTOR=1 / 1.25 / 1.5 / 2，运行 panta_qml_shell_module_test | 每档 9/9 通过；按现有规则跳过原生 VTK surface |
| QT_QPA_PLATFORM=cocoa，运行 panta_qml_shell_module_test | 最终独立顺序运行连续三轮 9/9 通过，包含宽 / 窄窗口、鼠标 / 键盘、模态返回及原生视口生命周期 |
| 内容及截图对照 | 忽略格式空白后，两套工具数组与拆分前完全相同；四档 offscreen 的 start-learn.png / opened.png 共 8 张与 task061 文件字节一致；已查看 Cocoa 两种工具栏截图 |
| git diff HEAD --check | 通过；新 QML 文件已登记模块资源，无额外依赖、HTML 或服务改动 |

截图位于 artifacts/qml/task062/{scale-*,cocoa}/。Cocoa 初次整套运行出现一次窄窗口键盘 clicked 未触发，随后一轮出现全局 focusWindow 为空；单独窄窗口用例通过，停止并行工具操作后的三轮完整测试均通过，期间未改导航实现、等待阈值或断言。记录观察到的系统窗口失焦，不推断具体外部抢焦来源。VTK 销毁时的 Destroyed 信息符合现有生命周期。Windows / Linux 真窗口和 CI 本轮未运行。

## 风险与工作记录

- 2026-09-22：创建任务，保留 061 的现有未提交修改。主要风险为 Loader 的内容宽度、焦点作用域、旧页签释放及新目录的 QML 资源注册；沿用现有交互 / 几何回归并增加独立加载验证。
- 2026-09-22：维护者要求先单独提交 061，提交为 85a00a9；062 的记录与实现不混入该提交。
- 2026-09-22：将两套工具数组原样移入 HomeRibbon / StartLearnRibbon；提取 RibbonContent，共用 RibbonGroup / RibbonTile。公共外壳通过静态 Component + Loader 切换，不引入动态资源路径或通用命令注册器。
- 2026-09-22：页签内负责具体 key 到语义信号的映射，App 保持导航和对话框所有权。Loader 不固定宽度，并启用焦点作用域；测试覆盖独立加载、重复选择、旧实例销毁和切回后信号路由。
- 2026-09-22：构建、格式、三项 lint、54 项 CTest、重复运行、四档缩放及最终三轮 Cocoa 通过；模块说明、任务及索引同步。062 留工作区验收，不 commit / push。
- 2026-09-22：维护者确认提交 062。核对代码、测试、模块资源、文档与任务边界，单独提交 Ribbon 组件拆分，不 push。

## 完成摘要

Ribbon 已按公共外壳、通用渲染和具体页签拆分。各页签独立维护工具内容与命令映射，切换仅加载当前页签；旧数组和面板内的工程命令判断已删除。视觉基准、翻译上下文及 Home ↔ Start & Learn 行为保持不变，验证与文档同步完成。061 已单独提交，062 按维护者追加授权另行提交，不 push。
