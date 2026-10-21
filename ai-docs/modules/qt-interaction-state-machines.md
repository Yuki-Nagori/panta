# Qt 交互状态机

[模块导航](README.md) · [界面与桥接](../architecture/ui-and-bridge.md) · [Rust FSM](fsm.md) · [实施任务 074](../task/074-qt-interaction-state-machine.md)

更新 / 官方资料查阅日期：2026-10-21，Qt 文档版本 6.11.2。仓库当前唯一的 Qt `QStateMachine` 消费者是 STL 导入窗口；新增交互按本文步骤设计和验收。

## 什么时候使用

当一个 Qt 操作包含多个阶段，并需要处理异步成功 / 失败、取消、重试或窗口重建时，由 C++ ViewModel / 交互控制器持有 `QStateMachine`。只有 hover、按下态、展开 / 折叠、页签选择等局部视觉反馈时，继续用 QML 属性绑定或 Qt Quick `State`，不要为了统一而造状态机。

| 层次 | 负责内容 | 示例 |
|---|---|---|
| Rust 领域状态机 | 业务规则、输入快照、修订、资源所有权、提交和取消裁决 | 能否提交、工程是否已提交、工作是否已取消 |
| C++ `QStateMachine` | 本地交互流程、用户意图排序、等待服务回执、弹窗和关闭协调 | 选文件后进入预览等待，失败后保留表单以供重试 |
| QML 属性绑定 / `states / transitions` | 属性配置、视觉反馈、动画和简单局部状态 | hover、展开 / 折叠、错误颜色、等待提示 |

Qt State Machine 的 C++ 入口是 `Qt6::StateMachine`，也提供 `QtQml.StateMachine` QML API；它与 Qt Quick 的视觉 `State` 不是同一个 API。[Qt 模块文档](https://doc.qt.io/qt-6/qtstatemachine-index.html) 本项目使用 C++ API：控制器独占交互状态，QML 只调用命令并绑定状态属性，不在两层各保存一份流程。

## 照着导入窗口的顺序实现

先用这个判断：如果操作只是改变颜色、可见性或选中项，用 QML 绑定；如果要等待服务结果、允许取消 / 重试，还要处理关闭和迟到回执，才建立 C++ 交互控制器。新流程不要从“有哪些状态类”开始，先写出用户动作和服务结果的顺序。

导入窗口的实际调用顺序如下，新增类似交互时可按此检查每一步：

| 用户动作 / 结果 | 当前实现入口 | 控制器要保证什么 |
|---|---|---|
| 打开弹窗 | `open()` | 初始化新草稿；若只是重建窗口，则继续原会话，不重发业务命令 |
| 点击 Browse | `beginFileSelection()` | 先占用选择请求并进入 `ChoosingFile`，避免重复打开选择器 |
| 文件选择器取消 | `cancelFileSelection(selectionId)` | 以选择请求 ID 丢弃过期取消；回到选择前的编辑 / 可提交状态 |
| 选择文件 | `previewFileForSelection(path, selectionId)` | 只接受当前选择 ID，旧预览不再启用提交 |
| Rust 预检完成 | `importPreviewFinished(requestId, succeeded)` | 仅当前请求可从 `Previewing` 转入 `Ready` 或回到 `Editing` |
| 点击 OK | `submit()` | 立即锁住提交意图；只在 `Submitting` 状态进入后调用一次 Rust 导入 |
| Rust 导入完成 | `projectImportFinished(requestId, operationSucceeded, projectionUpdated)` | 失败回到可编辑状态；提交且刷新成功后关闭；提交成功但刷新失败时禁止再次导入 |
| 用户点 Cancel、按 Esc 或系统关闭弹窗 | `close()` | 结束当前 UI 会话并取消预检；下次打开时按在途导入状态初始化草稿，业务取消只按 Rust 服务能力请求 |
| QML 窗口对象销毁，稍后重建 | `detachWindow()` | 保留 ProjectViewModel 中的会话和草稿，不取消导入；若文件选择器仍打开，只废弃它的选择令牌并回到选择前状态 |

代码位置也按职责分开：状态图和命令门禁放在 `native/bridge/src/import_interaction_controller.cpp`；QML 只调用控制器并绑定它的属性，不能直接调用 `inspectStl()` / `beginImportStl()`；解析、校验、提交和取消仍由 Rust 服务决定。控制器对 QML 公开 `stateName`、`canEdit`、`canSubmit` 等稳定属性，不公开 `QState*`，也不允许 QML 任意设置状态。

实现新流程时，把上表改成自己的动作 / 回执表，再补测试：状态机尚未进入初态时发命令、快速重复点击、服务失败后重试、旧请求迟到、关闭重开、销毁重建。每个测试都要检查服务命令次数和最终状态，不能只检查按钮颜色或可见性。

不把所有 UI 布尔值改成状态机，也不建立覆盖整个桌面的巨型状态图。独立 Dock 显隐、Ribbon 页签和 hover 保持简单绑定。视口逐帧输入与渲染留在既有 C++ 后端。

## 首个消费者：导入窗口

[`ImportDialog.qml`](../../qml/Dialogs/ImportDialog.qml) 将窗口打开、选文件、预览、提交与关闭委托给 ProjectViewModel 持有的交互控制器；实际解析、校验与工程写入仍通过 ProjectViewModel 调用 Rust。

以下是导入窗口的实际交互阶段，不复制 Rust 导入事务状态：

| UI 状态 | 进入原因 | 可接受的后续意图 / 回执 |
|---|---|---|
| Closed | 初始状态或用户关闭已完成 | 打开窗口并初始化新会话 |
| Editing | 打开窗口、预览失败或导入被服务拒绝 | 浏览文件、修改选项、关闭 |
| ChoosingFile | 接受一次 Browse | 接受路径后预览；取消时恢复先前 Editing / Ready 状态 |
| Previewing | 已接受当前文件并提交异步预览 | 匹配的预览结果、失败或关闭；旧选择不能启用 Submit |
| Ready | 当前文件预览已确认 | 修改选项、重新选文件、提交、关闭 |
| Submitting | 接受提交意图并等待唯一导入回执 | 成功且展示刷新成功后关闭；业务失败回到 Editing；关闭按服务能力请求取消 |
| ProjectionRefreshFailed | Rust 已提交，但 ViewModel 无法刷新展示 | 明确提示已提交、展示刷新失败；只允许关闭，禁止重复导入 |

重新选文件时记录选择前的 UI 状态；取消文件选择器可恢复有效预览，接受新文件则使旧预览失效。选项是否需要重新预览由服务契约确定，Qt 不复制格式、单位或几何校验规则。提交按钮可用性由本地交互阶段和 Rust 提供的有效性 / 能力共同决定，Rust 仍需校验每个请求。

STL 预检与确认导入由 Rust 后台执行。预检通过 `inspectStl` 接受请求并以编号关联 `importPreviewFinished`；导入通过 begin/finish API 返回结果。控制器分别消费预览和导入回执，避免属性通知重复推进。关闭表单会取消预检，但不伪造写入事务取消；导入是否可取消由 Rust 在提交点前裁决。若提交成功而展示刷新仍失败，状态停在 `ProjectionRefreshFailed`，显示 ViewModel 诊断且不开放再次导入。

## 事件、命令与生命周期

`QStateMachine` 依赖运行中的 Qt 事件循环，启动、停止和完成有不同的生命周期信号；不能在调用 `start()` 后立即假定初态已进入，`stop()` 也不代表完成业务。[QStateMachine 官方文档](https://doc.qt.io/qt-6/qstatemachine.html) 本项目规定控制器与状态对象留在 GUI 线程，长业务经服务调度，转移判断不执行文件 I/O 或长 native 调用。

- 对 UI 暴露稳定的交互 phase、能力和命令，不暴露 `QState*` 或任意设置状态的接口。启动未就绪时明确拒绝或排队意图，并验证其次数 / 顺序。
- 接受用户意图时就在 GUI 线程内占用本次请求并冻结参数，再请求状态转移；不能等目标状态的 entered 才阻止重复 OK / Browse。等待状态生效后才派发一次命令，异常退出或拒绝必须释放待发请求。同步回执也按本次关联在转移完成后处理，避免重入或结果早于目标状态。
- 同一次服务调用只有一个完成来源。预检的 bool 只表示接收请求，完成由关联请求编号的信号通知；后台导入的 begin/finish 回执表示提交结果，工程 / 资产属性信号只更新读模型。这些属性信号不能与完成回执各推进一次交互。适配层须区分 Rust 拒绝请求、提交成功与后续展示刷新失败；已提交但刷新失败只能重读展示，不能再次导入。
- 状态进入只更新 UI 配置；业务命令必须来自明确接受的用户意图。初始化、组件重建和恢复快照不能自动再次导入、保存或取消。若实现用状态进入触发派发，必须消费一次性待发意图，恢复路径不创建该意图。
- 文件选择、预览及导入回执分别携带可验证的请求 / 窗口会话关联和工程 generation。关闭重开、重新选文件和过时结果不得污染当前表单。若底层信号不带 ID，控制器需限制同时在途请求并建立可靠的关联，不能猜测它属于当前会话。
- OK、Cancel、Esc、标题栏及系统关闭请求走同一控制器入口。等待提交期间只能按 Rust 能力请求取消或分离展示；Qt 的关闭 / stopped / finished 信号不得生成领域 `Cancelled / Committed`。导入提交与收尾遵循 063 的 Rust 回执契约；073 是已保存 Mesh 的只读激活 FSM，不拥有或驱动导入事务。
- QObject parent 和有上下文的连接管理所有权，销毁前断开投递；计时器、延迟事件和回调不能引用已销毁的窗口。控制器及表单草稿由窗口之外的同一 ProjectViewModel 会话拥有，不能把 Rust 工程快照当作尚未提交的文件路径 / 选项草稿。窗口对象重建时恢复当前状态和有效草稿，不重发命令；文件选择器随窗口销毁，正在选择时退回先前 Editing / Ready 状态。主动关闭后下次打开会初始化新草稿。整个 ProjectViewModel / QQmlEngine 销毁后的服务延续不属于 074 范围；不能跨引擎搬运 QObject / JS 值，也不能依靠 `QHistoryState` 充当业务恢复。
- 交互属性只有控制器一个写入者，QML 以绑定消费；视觉状态不覆盖控制器的业务等待标志。表单字段有自己的明确所有者，避免 QState 属性赋值与 QML 绑定相互覆盖。

## 模块供给与实施边界

[`native/CMakeLists.txt`](../../native/CMakeLists.txt) 在启用 bridge 时查找 `StateMachine`；[`qt-provision.cmake`](../../native/cmake/qt-provision.cmake) 固定官方 qtscxml 三平台归档与 SHA256。074 验证依赖供给、链接和首个消费者；新增使用点应复用该供给路径，不另行引入 Qt 次版本或系统 Qt 回退。

首期不引入 `QtQml.StateMachine` import，因此不额外承诺部署它的 QML 插件；若以后使用该 API，再同步 import 扫描、qmllint、插件与安装产物。采用 Qt 状态机不要求引入 SCXML 文件，也不改变 `.pa` 只描述 Rust 流程结构的决定。

任务 074 交付模块接入、导入 UI 控制器、仓库现有 Qt 状态机使用审计和服务回归；控制器消费 063 已实现的异步预检与导入契约。073 只负责已保存 Mesh 的只读激活，不与导入事务共享命令或回执。普通销毁重建属于 074 验收，开发模式 QQmlEngine 重载仍由 027 负责。Qt 模块只由使用它的 native target 消费。

## 验证重点

用 QtTest / GTest 与 QML 行为用例观察状态、命令次数和信号顺序，覆盖未启动就绪、状态尚未转移时连续点击、同步多次属性通知与唯一完成回执、提交后刷新失败、旧选择 / 旧请求回执、表单关闭和同一服务会话内窗口销毁重建。恢复 UI 后再次观察命令计数，确保没有业务重放。带延迟的测试后端只验证交互，不作为真实异步业务或 GPU 窗口验收证据；请求关联、业务取消和完整引擎重载验收留在实际异步 / 重载功能联调阶段。

最终使用仓库 Cargo 聚合入口，并验证真实窗口、模块部署和三平台构建；完整命令与验收条件见 [074](../task/074-qt-interaction-state-machine.md)。
