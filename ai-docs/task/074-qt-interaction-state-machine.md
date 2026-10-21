# 074 — Qt 交互状态机与导入交互

- 状态：done
- 阶段：应用平台扩展
- 依赖：[026](026-static-qml-modules.md)、[063](063-stl-import-and-mesh-workspace.md)、[072](072-flow-state-machine-planning.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-10-21

## 目标与背景

为复杂 Qt UI 交互统一采用 Qt 自带的 `QStateMachine`，并为仓库内每个实际使用点建立唯一、可审阅的交互状态所有者。首个消费者是现有导入窗口：集中管理打开、选文件、预览、编辑选项、等待提交结果和关闭。Qt 管理交互流程，Rust 继续拥有解析、修订、提交和业务取消的决定权。

仓库源码盘点确认，当前只有导入交互使用 C++ `QStateMachine`；页签选择、hover 等简单视觉状态不需要额外状态机。模块接入、ProjectViewModel 会话内的 C++ 控制器、回归测试及 Qt 状态机准则均已完成，最终验收见下文。

## 必读

- [Qt 交互状态机](../modules/qt-interaction-state-machines.md)、[FSM 与 Rust 状态机](../modules/fsm.md)、[界面与桥接](../architecture/ui-and-bridge.md)
- [Qt](../standards/qt.md)、[QML](../standards/qml.md)、[分层规则](../standards/layering.md)、[依赖获取](../standards/dependency-acquisition.md)
- [CMake](../standards/cmake.md)、[注释](../standards/comments.md)、[测试](../standards/testing.md)、[验证与评审](../standards/validation-and-review.md)
- [文档](../standards/documentation.md)、[仓库文件](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

官方依据（2026-09-24，页面版本 Qt 6.11.2）：[模块与 CMake / QML 入口](https://doc.qt.io/qt-6/qtstatemachine-index.html)、[QStateMachine 生命周期](https://doc.qt.io/qt-6/qstatemachine.html)。

## 范围与非目标

接入与仓库锁定 Qt 同版本的 StateMachine 模块，在 `native/bridge` 增加由 C++ ViewModel 管理的交互控制器，QML 绑定其状态和能力并发送用户意图。盘点仓库中所有实际 Qt `QStateMachine` 使用点，并按统一准则逐一纳入本任务；当前只有 STL 导入窗口，消费 063 已提供的异步预检、导入请求、取消和完成通知。Rust 继续拥有解析、修订、提交和业务取消的决定权。073 的 Rust FSM 服务于已保存 Mesh 的只读视口激活，不作为 Qt 导入交互控制器或通用任务 API。

交互契约约束命令接受到 Qt 转移完成之间的重复点击、完成信号与属性通知的唯一推进来源、提交后展示刷新失败，以及同一服务会话内的窗口重建。相应服务适配只区分 Rust 操作结果与展示刷新结果，不在 Qt 增加业务裁决。

不建立全局 UI 巨型状态机，不迁移 hover / pressed / 颜色 / 简单页签绑定，不从 `.pa` 生成 Qt 状态图，不引入 SCXML 文档或另一个 Rust FSM 库。后续新增 Qt `QStateMachine` 消费者必须遵循 [Qt 交互状态机准则](../modules/qt-interaction-state-machines.md)，并在实现同一改动中更新依赖供给、选择器、测试与文档。Qt 状态机不承担解析、写盘、业务 guard、工程恢复或后台线程调度，也不复制 Rust 操作状态。

## 前置条件与任务关系

- 063、073 已提供异步导入及只读激活能力；实施前核对当前请求身份、取消、失败与迟到结果契约，以及 026 的 QML 注册入口。
- Qt 6.11.2 的 StateMachine C++ 模块由官方 qtscxml 附加包提供；三平台归档及 SHA256 已实算并登记到 `native/cmake/qt-provision.cmake`，PR #78 的三平台 CI 已验证干净缓存构建、消费与部署。
- 074 不以 073 为 Rust 核心实现前置条件，也不把 073 的只读资产激活 FSM 接入导入对话框；两者按各自请求身份和取消语义独立验收。
- 063 的预检与导入写入均已有异步入口，不能再以“导入同步”作为当前限制。Qt `QStateMachine` 只协调交互阶段，不提供异步 I/O 或业务取消能力；界面能力必须依据现有 Rust 回执呈现。
- 027 的热重载不是前置条件；本任务验证普通销毁 / 重建，027 后续复用恢复契约。

## 实施步骤

1. 审计当前导入 UI 的状态、命令和回执，冻结最小交互图、关联 ID、状态属性的唯一写入者及关闭行为。
2. 完成 Qt 模块三平台供给、CMake 链接与安装依赖；首期用 C++ API，不为未使用的 QML API 添加 import / 插件。
3. 实现由窗口之外的同一 ProjectViewModel 会话持有的交互控制器、私有 `QStateMachine` 和必要表单草稿，对 QML 暴露只读交互属性与命令；状态对象及连接留在 GUI 线程，QML 不持有 `QState*`。
4. 接受意图时立即占用请求并冻结参数，等待状态生效后再调用服务；将 Rust 调用结果与后续展示刷新结果归一为唯一关联完成回执，属性通知只更新读模型，避免同步通知和返回值重复推进。已提交但刷新失败进入只读失败状态并显示诊断；不允许重新导入，恢复路径也不重发命令。
5. 改造 ImportDialog，统一 Browse / OK / Cancel / 标题栏关闭 / 系统关闭入口；删除替代掉的控制布尔值及命令调用点，保留必要表单数据和视觉绑定。
6. 测试启动 / 停止、重复点击、文件选择器返回、失败重试、关闭重开和组件销毁；完成真实窗口与三平台验证后同步任务状态。

## 预计改动

改动覆盖 `native/cmake/qt-provision.cmake`、`native/CMakeLists.txt`、`native/bridge`、`qml/Dialogs/ImportDialog.qml`、本地化资源、测试及相关架构文档。CMake 拥有 native 依赖图，Cargo 提供统一验证入口。

## 验收标准

- [x] 仓库内全部 Qt `QStateMachine` 使用点已盘点并符合统一准则；简单视觉状态与 Rust 领域 FSM 未被误迁入 Qt 状态机。
- [x] StateMachine 的版本、三平台来源 / 完整 SHA256 与消费 target 明确；三平台构建、测试和依赖消费由 PR CI 验证，不依赖开发机额外 Qt 安装。
- [x] 导入窗口覆盖关闭、编辑 / 选文件、预览等待、可提交、提交等待、服务失败恢复与提交后展示刷新失败；命令只走控制器入口，服务校验保持 Rust 权威。
- [x] `start()` 后尚未就绪、状态转移前连续点击、异步完成回执与属性通知均不会漏掉结果或重复提交；`stop()` / `finished()` 不被当成业务取消 / 提交完成。
- [x] 旧文件选择、旧预览、旧导入请求和旧窗口会话的信号不会覆盖当前输入；关闭后回执不访问已销毁对象，同一服务会话内窗口重建按业务快照与有效 UI 草稿恢复。请求身份和迟到结果验证基于 063/073 各自现有契约。
- [x] 提交前表单取消 / 关闭可用，在途业务取消能力如实呈现；未来请求取消的等待状态仅在真实服务支持时启用，UI 不自行设置 `Cancelled / Committed`。
- [x] Rust 命令成功后的展示刷新失败只触发读模型刷新，导入调用计数仍为一次；不能依据单一 bool 结果或全局错误属性盲目重试提交。
- [x] 控制器公开属性与 QML 绑定有唯一写入者，同一服务会话内窗口销毁 / 重建不重发导入；无双状态机共同驱动同一交互、无遗留布尔控制路径。
- [x] `cargo verify-changed`、macOS 真实窗口验收及 PR #78 的 macOS / Linux / Windows 构建和受影响测试均通过；bridge 相关诊断构建及现有导入行为回归通过。

## 验证计划与结果

仓库源码审计确认只有 STL 导入交互使用 Qt `QStateMachine`；页签和 hover 保持 QML 绑定，视口激活保持 Rust FSM。macOS 本地完成真实窗口、销毁 / 重建后的会话复用及提交成功但展示投影刷新失败的确定性故障注入；PR #78 的三平台 CI 完成干净缓存构建、测试与部署验收。

| 日期 | 场景 | 结果 / 证据 |
|---|---|---|
| 2026-10-21 | PR [#78](https://github.com/Yuki-Nagori/panta/pull/78) 最终 PR head `469eb8d`，GitHub Actions run [39536255182](https://github.com/Yuki-Nagori/panta/actions/runs/39536255182) | macOS、Linux、Windows 构建、受影响测试及适用的 Rust/C++/QML/CMake 质量检查全部通过。 |
| 2026-10-21 | macOS arm64，本地构建、受影响验证与真实窗口 | `cargo build --locked`、`cargo verify-changed`、CTest 90/90 和真实窗口导入对话框验收通过；覆盖原生选择器取消、关闭重开、ViewModel 会话复用，以及提交成功后展示刷新失败不重复提交。 |
| 2026-10-21 | Qt 6.11.2 官方 qtscxml 附加包 | 三平台 SHA256 已实算并登记：macOS `9cabcf73b1ef81cb396a205380950d947535e20b7b6e21204e59a52199a98bfe`、Linux `8579d82fba91dfe1b24e3203f740ac5647d5515c51182a4158b0e748dc6f00f7`、Windows `47539713e42a72f29ef0da0be2ad82428eefb7158bfe766b2e1dc7e63640e058`；供给、消费与部署由上述 PR CI 验证。 |
| 2026-09-24 | 初始规划检查 | 仅核对规划文档与本地链接；当时未实施状态机功能，历史记录不作为当前验收依据。 |
## 清理与兼容例外

已将导入窗口的流程控制和服务命令入口迁入 ProjectViewModel 持有的 C++ 控制器，QML 保留表单呈现和属性绑定；无兼容例外。已删除被替代的 QML 命令路径，不并行保留新旧交互实现。服务契约未来升级按实际消费者一起修改，不预建兼容适配层。

## 风险与回退

主要风险是把 Qt 异步事件循环当作后台执行、同步回执早于转移、组件恢复重复提交以及 Qt / Rust 各自决定取消。现有行为测试覆盖启动前命令、重复意图、选择取消 / 接受、预检、会话关闭、窗口对象重建和提交后的展示投影刷新失败；跨平台干净缓存构建与部署由 CI 验收。不退回系统 Qt 或保留不可构建的 import。

## 决策与工作记录

- 2026-09-24：按用户要求独立登记 Qt 状态机任务，明确采用 C++ `QStateMachine` 管复杂 UI 交互，以现有导入窗口为首个消费者；073 保持 Rust 领域边界。
- 2026-09-24：完成 Qt 交互设计、模块导航、Qt 规范、界面架构及 072 / 073 衔接说明，文档检查通过；任务保持 planned，模块供给与控制器实现尚未开始。
- 2026-09-24：按用户要求整体复审后本地提交；核对现有 `inspectStl` / `importStl` 的返回值与同步通知、`ProjectViewModel` 对 Rust 服务的所有权，修正首期验收和未来异步联调的边界。
- 2026-09-24：复审修正及适用检查完成，未发现阻碍本规划提交的剩余问题；本次以 074 为主任务，同步 072 的决策记录和 073 的交付边界，属于同一 Qt 状态机规划变更。无代码或依赖变更、无废弃项、无兼容例外；实际功能继续保持 planned。
- 2026-10-21：完成 Qt 6.11.2 StateMachine 供给和 STL 导入交互控制器，改造 ImportDialog / ProjectViewModel，补充交互回归测试、中文 / 英文文案与架构文档；macOS 本地验收及 PR #78 三平台 CI 均通过。独立只读复审未发现阻塞问题，并移除了一项与真实后台导入竞争的非确定性合成回执测试。
- 2026-10-21：按 Issue #79 巡检修正文档状态，将验收表链接更新为 PR #78 最终 PR head `469eb8d` 对应的 Actions run `39536255182`（main 合并提交为 `e66de56`）；删除会与完成状态冲突的阶段性待验收描述，保留的 2026-09-24 记录明确标为历史规划。
- 2026-10-21：Issue #76 收尾时修正文档路径，为提交后的投影刷新失败增加确定性回归，并验证 ImportDialog 对象销毁 / 重建后沿用同一 ViewModel 会话。将合并式 QML QtTest 拆为主题原子、Ribbon、对话框、分析面板、文档页签及独立 ImportDialog 会话测试；本地 CTest 90/90 与 PR #78 跨平台 CI 均通过。

## 完成摘要

已实现首个 Qt 状态机消费者并整理准则与架构说明；本地完整受影响验证、macOS 真实窗口验收及三平台 CI 通过，任务完成。
