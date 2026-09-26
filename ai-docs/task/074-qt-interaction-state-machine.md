# 074 — Qt StateMachine 与导入窗口交互编排

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[026](026-static-qml-modules.md)、[063](063-stl-import-and-mesh-workspace.md)、[072](072-flow-state-machine-planning.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

为复杂 UI 交互采用 Qt 自带的 `QStateMachine`，首个消费者是现有导入窗口：集中管理打开、选文件、预览、编辑选项、等待提交结果和关闭，避免 QML 布尔值、回调和弹窗状态分散决定可执行操作。Qt 管理交互流程，Rust 继续拥有解析、修订、提交和业务取消的决定权。

当前 `qml/Dialogs/ImportDialog.qml` 直接组织 `open / chooseFile / submit` 与 `FileDialog.onAccepted`；`native/CMakeLists.txt` 尚未查找 `StateMachine` 组件，bridge 未链接 `Qt6::StateMachine`。本任务只有设计与实施准备，未接入模块或改造窗口。

## 必读

- [Qt 交互状态机设计](../modules/qt-interaction-state-machines.md)、[FSM 与 Rust 状态机](../modules/fsm.md)、[界面与桥接](../architecture/ui-and-bridge.md)
- [Qt](../standards/qt.md)、[QML](../standards/qml.md)、[分层规则](../standards/layering.md)、[依赖获取](../standards/dependency-acquisition.md)
- [CMake](../standards/cmake.md)、[注释](../standards/comments.md)、[测试](../standards/testing.md)、[验证与评审](../standards/validation-and-review.md)
- [文档](../standards/documentation.md)、[仓库文件](../standards/repository-hygiene.md)、[代码生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

官方依据（2026-09-24，页面版本 Qt 6.11.2）：[模块与 CMake / QML 入口](https://doc.qt.io/qt-6/qtstatemachine-index.html)、[QStateMachine 生命周期](https://doc.qt.io/qt-6/qstatemachine.html)。

## 范围与非目标

接入与仓库锁定 Qt 同版本的 StateMachine 模块，在 `native/bridge` 增加由 C++ ViewModel 管理的交互控制器，QML 绑定其状态和能力并发送用户意图。用现有 STL 导入窗口验证真实消费者，再在 073 提供异步契约后衔接进度、取消请求及其回执。

交互契约同时约束命令接受到 Qt 转移完成之间的重复点击、同步返回与属性信号的唯一完成来源、提交后展示刷新失败，以及同一服务会话内的窗口重建。相应服务适配只区分已有 Rust 调用结果与展示刷新结果，不在 Qt 增加业务裁决。

不建立全局 UI 巨型状态机，不迁移 hover / pressed / 颜色 / 简单页签绑定，不从 `.pa` 生成 Qt 状态图，不引入 SCXML 文档或另一个 Rust FSM 库。Qt 状态机不承担解析、写盘、业务 guard、工程恢复或后台线程调度；本任务也不把同步 STL 路径改成 Rust FSM。

## 前置条件与任务关系

- 063 的导入与预览接口、失败行为及现有测试需先收口，026 的 QML 注册入口保持一致。
- 核实三平台官方预编译包中提供 StateMachine 的具体归档、依赖、文件清单与 SHA256，再扩展现有 Qt provision；当前没有对应制品验证证据，不假定已有 qtbase / qtdeclarative 归档足够。
- **074 的首期交付不依赖 073。** 可以基于现有同步服务结果改造 UI 编排，但必须保留“同步路径尚不能保证长任务响应”的现状说明，不能声称 QStateMachine 自动使 I/O 异步。
- 073 也不以 074 为 Rust 核心实现前置条件。待 073 的 TaskId、能力快照和取消回执可用后，登记两者的联调范围与证据；不能要求两个任务互相完成后才开始。没有真实异步后端时不交付虚假的业务取消按钮，提交前的表单关闭仍可用。
- 027 的热重载不是前置条件；本任务验证普通销毁 / 重建，027 后续复用恢复契约。

## 实施步骤

1. 审计当前导入 UI 的状态、命令和回执，冻结最小交互图、关联 ID、状态属性的唯一写入者及关闭行为。
2. 完成 Qt 模块三平台供给、CMake 链接与安装依赖；首期用 C++ API，不为未使用的 QML API 添加 import / 插件。
3. 实现由窗口之外的同一 ProjectViewModel 会话持有的交互控制器、私有 `QStateMachine` 和必要表单草稿，对 QML 暴露只读交互属性与命令；状态对象及连接留在 GUI 线程，QML 不持有 `QState*`。
4. 接受意图时立即占用请求并冻结参数，等待状态生效后再调用服务；将 Rust 调用结果与后续展示刷新结果归一为唯一关联完成回执，属性通知只更新读模型，避免同步通知和返回值重复推进。已提交但刷新失败不允许重新导入，状态恢复也不重发命令。
5. 改造 ImportDialog，统一 Browse / OK / Cancel / 标题栏关闭 / 系统关闭入口；删除替代掉的控制布尔值及命令调用点，保留必要表单数据和视觉绑定。
6. 测试启动 / 停止、重复点击、文件选择器返回、失败重试、关闭重开和组件销毁；完成真实窗口与三平台验证后同步任务状态。

## 预计改动

现有 `native/cmake/qt-provision.cmake`、`native/CMakeLists.txt`、`native/bridge`、`qml/Dialogs/ImportDialog.qml`、必要的 Shell 连接和测试。交互控制器文件待创建，命名在实施时冻结；CMake 拥有依赖图，Cargo 保持统一入口。

## 验收标准

- [ ] StateMachine 的版本、三平台来源 / SHA256 与消费 target 明确，干净 / 增量构建及安装后启动可复现；不依赖开发机额外 Qt 安装。
- [ ] 导入窗口至少覆盖关闭、编辑 / 选文件、预览等待、可提交、提交等待和失败恢复的真实路径；命令只走控制器入口，服务校验保持 Rust 权威。
- [ ] `start()` 后尚未就绪、状态转移前连续点击、同步多次属性通知与唯一完成回执、异步测试回执均不会漏掉结果或重复提交；`stop()` / `finished()` 不被当成业务取消 / 提交完成。
- [ ] 旧文件选择、旧预览和旧窗口会话的信号不会覆盖当前输入；关闭后回执不访问已销毁对象，同一服务会话内窗口重建按业务快照与有效 UI 草稿恢复。真实 TaskId 的过期回执验证在 073 联调时登记。
- [ ] 提交前表单取消 / 关闭可用，在途业务取消能力如实呈现；未来请求取消的等待状态仅在真实服务支持时启用，UI 不自行设置 `Cancelled / Committed`。
- [ ] Rust 命令成功后的展示刷新失败只触发读模型刷新，导入调用计数仍为一次；不能依据单一 bool 结果或全局错误属性盲目重试提交。
- [ ] 控制器公开属性与 QML 绑定有唯一写入者，同一服务会话内窗口销毁 / 重建不重发导入；无双状态机共同驱动同一交互、无遗留布尔控制路径。
- [ ] Cargo 聚合检查、真实窗口和受影响平台部署证据齐全；现有导入行为及 bridge 关闭的诊断构建不回归。

## 验证计划与结果

实现验证尚未执行。仓库根目录按锁定工具链运行 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`；补充 `cargo build --locked --no-default-features` 验证 bridge 裁剪。控制器行为用 GTest / QtTest，QML 用真实绑定测试，最终由 Cargo 聚合。真实窗口按仓库规范启动 native 应用，验证正常、错误、重复点击和关闭重开；实际异步取消与长任务响应证据由 073 联调补充，不能以本任务同步路径代替。

| 日期 | 场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-24 | 代码与官方资料核对 | 已确认当前导入窗口、Qt 模块消费缺口与官方 API 入口；仅规划，未验证模块制品或运行行为 |
| 2026-09-24 | 仓库根目录临时 Python 标准库检查与 `git diff HEAD --check` | 通过：本次 9 份 Markdown、207 个本地链接、围栏 / 空白、072 / 073 / 074 索引状态一致；073 与 074 无互相完成依赖。仅文档验证，未运行应用构建或行为测试 |
| 2026-09-24 | 整体复审与修正后的同类文档检查；macOS arm64，仓库根目录 `cargo format --check` | 文档检查通过；格式命令退出 0，输出 `format 检查通过`。已复核首期真实同步接口与后续异步 / 重载验收边界，未执行 UI 状态机行为测试 |

## 清理与兼容例外

当前无代码变更、无兼容例外。实施时删除被控制器替代的 QML 命令与流程控制路径；保留数据 / 视觉绑定，不并行保留新旧交互实现。服务契约未来升级按实际消费者一起修改，不先建猜测性的兼容适配层。

## 风险与回退

主要风险是把 Qt 异步事件循环当作后台执行、同步回执早于转移、组件恢复重复提交以及 Qt / Rust 各自决定取消。按设计区分交互状态和业务结果，通过信号顺序、命令次数与销毁后事件测试验证；若模块供给未完成，维持 planned，不退回系统 Qt 或先提交不可构建的 import。

## 决策与工作记录

- 2026-09-24：按用户要求独立登记 Qt 状态机任务，明确采用 C++ `QStateMachine` 管复杂 UI 交互，以现有导入窗口为首个消费者；073 保持 Rust 领域边界。
- 2026-09-24：完成 Qt 交互设计、模块导航、Qt 规范、界面架构及 072 / 073 衔接说明，文档检查通过；任务保持 planned，模块供给与控制器实现尚未开始。
- 2026-09-24：按用户要求整体复审后本地提交；核对现有 `inspectStl` / `importStl` 的返回值与同步通知、`ProjectViewModel` 对 Rust 服务的所有权，修正首期验收和未来异步联调的边界。
- 2026-09-24：复审修正及适用检查完成，未发现阻碍本规划提交的剩余问题；本次以 074 为主任务，同步 072 的决策记录和 073 的交付边界，属于同一 Qt 状态机规划变更。无代码或依赖变更、无废弃项、无兼容例外；实际功能继续保持 planned。

## 完成摘要

未实施。当前仅登记设计、依赖、验证与后续联调范围。
