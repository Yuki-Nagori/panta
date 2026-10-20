# 128 — 冗余代码与状态收敛

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[008](008-tasks-errors-logging.md)、[029](029-qml-component-library.md)、[056](056-qml-native-review.md)、[109](109-bounded-background-execution.md)、[117](117-rust-project-document-workspace.md)、[127](127-rust-qt-boundary-followup.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-10-20 / 2026-10-20

## 目标与背景

基于仓库实际引用和产品架构，清理无消费者的属性链、重复运行时状态及失效演示逻辑。可复用的领域职责保留在正确分层；仅因当前界面尚未消费，不删除有明确后续扩展价值的稳定接口。

## 必读

- [代码生命周期](../standards/code-lifecycle.md)
- [分层规则](../standards/layering.md)
- [Qt 与 QML 规范](../standards/qt.md)
- [C++/Rust 边界](../architecture/native-domain-boundaries.md)
- [UI 与桥接](../architecture/ui-and-bridge.md)

## 范围与非目标

- 清理 QML 只写不读的 `importedPartName` 透传及性能夹具赋值。
- 清理 ShellViewModel 中无生产调用的演示计数、空标题和空错误状态，保留真实页面所需行为。
- 合并 `ProjectService.latest_mesh_id` 与 `SurfaceMeshCache.active_id` 的重复状态，保持对外可观察的激活与缓存语义。
- 删除无生产消费者的最后导入网格展示缓存/桥接属性；保留导入记录和网格 DTO 中未来业务实际需要的数据。
- 避免对工作区激活结果进行无用 FFI DTO 转换。
- 删除仅提供 duration/fail 模拟入口、没有业务消费者的 TaskManager、TaskService 与 TaskHost；真实工程操作已经使用领域状态机和共享 `ProjectExecution`。未来任务 / 作业作为产品概念保留，不把模拟 API 当作生产契约。
- 不改动既定 `.panta` schema 兼容政策、有效 upstream `COMPAT`、生成代码元数据或有稳定产品消费者的接口。

## 实施步骤

1. 以当前源码、调用图和任务/架构决策核对每项候选，记录保留或删除理由。
2. 收敛确认冗余的 QML、ViewModel、Rust 缓存状态和 FFI 转换；让现有行为测试覆盖真实缓存/工作区语义。
3. 同步修改测试夹具、CMake/QML 注册、架构说明和相关历史任务摘要，删除过时的保留决定。
4. 执行受影响的 L1/L2 验证、完整适用 lint/格式检查，并完成独立只读评审。

## 审查结论

| 候选 | 证据与处理 |
|---|---|
| `TaskManager` / `TaskService` / `TaskHost` | 生产 QML、ViewModel 与服务没有业务调用；实现和专属测试只覆盖模拟 duration/fail。删除模拟链，保留 `ProjectExecution` 调度及各领域 FSM。 |
| `ProjectService.latest_mesh_id` | 与 `SurfaceMeshCache.active_id` 表达相同活动资产；删除重复字段，以缓存查询和工作区快照验证状态。 |
| 最近导入元数据属性 | QML 无读取者，只有测试和基准夹具设置；删除 ViewModel 属性与无效夹具值，保留导入记录名 / ID 和预览元数据。 |
| workspace 激活 FFI DTO | C++ 丢弃全部坐标结果，只依赖已更新的工作区快照；Rust 直接消费并释放结果，不分配 DTO / 坐标向量。 |
| `ShellViewModel` 演示属性 | 首页没有生产调用，其空标题、演示计数及错误属性只支撑自身测试；移除对象和专属测试，状态栏使用真实工程诊断。 |

## 验收标准

- [x] 每项清理均有源码引用证据；未来扩展接口只有在职责真实、边界清晰时保留。
- [x] 不再存在 QML/C++/Rust 的无消费者属性链、重复状态源或无效 FFI 结果分配。
- [x] 模拟 TaskManager/TaskService/TaskHost API、专属测试、执行池与生产注册点均已删除；共享执行器的真实消费者和领域激活状态机未回归。
- [x] 所有受影响测试、格式、lint、分层检查通过；选择器覆盖移动/删除的路径。
- [x] 独立评审无未处理问题，任务与架构文档反映最终状态。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-10-20 | macOS arm64 `cargo format --changed`、`cargo verify-changed` | 格式、Rust/native/QML 质量检查与 L1/L2 全量回退通过；Cargo 聚合测试与 CTest 81/81 通过。缓存激活回归确认所选 Mesh 在预算淘汰时保留，其他非活动 Mesh 被回收。 |
| 2026-10-20 | 独立只读架构评审（完整 diff） | 初审发现的四项问题均已修正；复核确认实现、分层、测试与当前文档一致，无遗留问题。 |

## 决策与工作记录

- 2026-10-20：创建任务。将清理建立在真实消费者和分层职责上；不得因“当前没用”而删除已确认的领域数据或未来稳定契约，也不得为了消除未使用项人为制造调用。
- 2026-10-20：确认模拟 TaskManager/TaskService/TaskHost 没有业务消费者；`ProjectExecution` 和每个工程操作的领域状态机已经承担实际后台执行。073 的资产激活使用独立 `ActivationCoordinator`，文档中“复用 Task 队列 / 展示契约”与代码不符；本任务删除模拟实现并同步当前架构说明，不将其接入真实工程操作。
- 2026-10-20：独立只读评审确认职责划分与生产引用清理成立，并指出 FFI 坐标转换开关、缓存激活行为测试及文档准确性问题；修复后复核通过。

## 完成摘要

清理与适用本地验证完成。当前后台工程操作由 `ProjectExecution` 调度，领域服务和 FSM 管理各自的请求生命周期、取消及结果；没有未验证的通用任务管理器。删除无消费者模拟链、重复缓存状态和只读未用属性，保留真实 UI 所需导入记录、网格缓存及异步事务。架构、运行时和任务文档均已对齐当前实现。
