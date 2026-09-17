# 008 — 后台任务、错误与日志基础

- 状态：in-progress
- 阶段：基础平台
- 依赖：[005](005-qt-qml-shell.md)（已完成）、[006](006-rust-cpp-boundary.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

建立单一后台任务生命周期和诊断路径，为导入/网格生成复用。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

已开始实施：Rust 侧任务管理器与 FFI 服务层已落地并验证；Qt/ViewModel 集成与日志落盘待续。

## 必读

- [通用规范：comments](../standards/comments.md)

- [规范：rust](../standards/rust.md)
- [规范：cpp](../standards/cpp.md)
- [规范：qt](../standards/qt.md)
- [规范：ffi](../standards/ffi.md)
- [架构：application-and-storage](../architecture/application-and-storage.md)
- [架构：ui-and-bridge](../architecture/ui-and-bridge.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不做完整工作流引擎、插件系统或真实求解器客户端。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 定义 TaskId、输入修订、状态、取消请求、错误码和结构化诊断；保持领域类型与 UI 类型分离。
2. 选择 worker 执行与事件交付机制，用模拟慢任务验证进度与可取消阶段。
3. 将日志与用户错误摘要分离，为同一任务保留关联 ID；不要要求依赖每行日志驱动 UI 状态。
4. 实现关闭工程/窗口后的回调拒绝及资源清理，定义终态不可回退规则。

## 预计改动

crates/core 或 workflow 的实际必要部分、native service/bridge、诊断配置。执行前根据真实结构修订；不得顺手实现非目标功能。

本轮实际边界：新增 `crates/panta-core`（任务状态机、事件队列、结构化日志环、销毁 join）；`panta-ffi` 桥接 `TaskService` opaque 与 `TaskEvent`/`TaskLogLine` DTO（拉取式，无跨语言回调）；`native/ffi` 新增 `task_service_test`；launcher build.rs 重建追踪补 panta-core。staticlib 仍只有 `panta_ffi.a` 一个（panta-core 被打包其中），040 的双边顺序约束不变。Qt/ViewModel 轮询集成与日志落盘未实施。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] 启动、成功、失败、取消及迟到事件路径均可验证，UI 不被模拟慢任务阻塞。
- [ ] 关闭窗口后不存在访问销毁对象，跨语言错误保持可追踪上下文。
- [ ] 日志能定位任务与原因，重复/迟到事件不使终态回到运行中。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |
| 2026-09-17 | `cargo test -p panta-core --locked`（macOS arm64） | 6/6：成功路径 Started→Succeeded 有序且终态后队列空、模拟失败带 `task.simulated_failure` 与标签上下文、取消即时且重复请求被拒、终态后迟到取消被拒、空标签/超时长结构化拒绝、析构 join 运行中任务 |
| 2026-09-17 | `cargo test -p panta-ffi --locked` | 8/8：桥接层 drain 事件/日志关联/`task.*` 错误码映射通过 |
| 2026-09-17 | native GTest `Ffi.TaskServiceLifecycle`（staticlib 最终链接，macOS arm64） | 5/5：提交→有序事件、失败结构化 code/detail、取消+迟到取消拒绝、无效提交转 `rust::Error`、双 30s 任务析构快速 join |
| 2026-09-17 | `ctest --test-dir native/build/debug`（重配指向新生成头后） | 10/10 全部通过（含原 7 项与 FFI 两目标） |
| 2026-09-17 | `cargo fmt --all -- --check`；`cargo clippy --locked --workspace --all-targets --exclude panta-launcher -- -D warnings`；`cargo test --locked --workspace --exclude panta-launcher`；`git diff --check` | 通过：Rust 25/25（6+9+2+8），Clippy 0 warning，格式与补丁检查干净 |

## 风险与回退

取消与关闭可能和完成回调竞争；用任务 ID、修订及对象存活条件拒绝迟到事件，并验证终态不回退。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-17：依赖 005、006 均已完成且范围/验收明确，状态调整为 ready；FFI 边界的 `Result`/panic 语义以 panta-ffi 实测为准。
- 2026-09-17（方案）：任务所有权按架构归 Rust，落地 `panta-core`；事件采用拉取队列而非跨语言回调（[cxx](../standards/cxx.md) 允许的简化），C++/QML 侧后续由 ViewModel 在 GUI 线程轮询并转 Qt 信号，工作线程不触碰 UI。取消为协作式（10ms 检查点，决定取消与 join 延迟上界）；终态转换与事件发布在同一锁内完成，迟到/重复事件被拒绝，终态不可回退；关闭语义 = shutdown 位 + join，析构后不存在可触达的工作线程。错误为稳定码（`task.*`）+ 诊断 detail，与用户摘要分离；结构化日志以 task_id 关联、有界环保留。
- 2026-09-17（实施）：落地上表服务层；`cargo build` 后 staticlib 刷新、native 重配链接验证通过。剩余：Qt/ViewModel 集成与"UI 不被阻塞"验证、日志落盘/Console 通路决策、三平台 CI 覆盖。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。服务层已落地：panta-core 任务状态机（事件队列、协作取消、终态不可回退、结构化日志环、销毁 join）与 panta-ffi `TaskService` 桥接，本机 Rust 25 测试 + native GTest 5 用例 + CTest 10/10 验证通过。剩余：Qt/ViewModel 集成（GUI 线程轮询转信号、"UI 不被模拟慢任务阻塞"验证）、日志落盘/Console 通路、推送后三平台 CI 覆盖；全部验收有证据后再标 done。
