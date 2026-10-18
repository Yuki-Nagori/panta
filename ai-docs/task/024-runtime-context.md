# 024 — 工程运行时上下文与变量快照

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[008](008-tasks-errors-logging.md), [023](023-cross-platform-paths.md)
- 优先级：P1
- 负责人：Codex
- 创建 / 更新：2026-09-16 / 2026-10-18

## 目标与背景

建立由 `ProjectService` 持有的轻量运行时上下文，独立于 UI 引擎管理工程会话身份、只读资源解析、不可变变量快照与任务作用域。

## 必读

- [模块设计](../modules/paths-and-runtime.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [rust](../standards/rust.md)
- [cxx](../standards/cxx.md)
- [qt](../standards/qt.md)

## 范围与非目标

包含内存会话、不可变快照、revision/generation、受控能力和取消；不包含字节码、安全沙箱、完整工程存储或 DSL 解析器。

## 前置条件与待决策

008 的任务/取消/错误契约和 023 路径服务已落地。上下文由 `ProjectService` 持有并复用其工程切换 generation；变量快照修订独立递增，不冒充工程清单 revision。首版值类型由 024 定义为 025 可复用的基础值，变量解析、表达式、校验和持久化仍由 025 负责。运行时上下文只开放变量读取和受根约束的现存资源解析，不提供写入、进程或网络能力。当前没有需要跨 CXX 的运行时消费者，因此本任务不新增 FFI 句柄；C++/QML 仍经 Rust 应用服务访问状态。

## 实施步骤

1. 定义上下文拥有者、基础值与不可变快照；复用 `ProjectService` 的会话 generation。
2. 实现按预期快照 revision 原子提交、旧修订拒绝和任务作用域捕获。
3. 工程替换/服务销毁时撤销上下文并取消作用域；验证旧结果拒绝、只读资源边界及 UI 无关性，形成 025/027 可依赖契约。

## 预计改动

`crates/panta-core/src/runtime.rs`、`ProjectService` 生命周期接入、所属 Rust 单元测试及模块/任务文档。若实现证据表明模块归属应调整，以实际职责为准；不新增无消费者的 CXX/QML API。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] 同一任务看到固定快照；变量提交创建新 revision，不改变已捕获输入。
- [x] 工程替换或服务销毁后，旧作用域被协作取消，旧任务结果被拒绝。
- [x] 运行时状态由 Rust 服务持有，不保存 JS 值或引擎对象；QML 订阅不拥有运行时上下文。
- [x] 资源能力仅解析注入根内的现存资源；旧 revision、空变量名、非有限数值和已关闭上下文的提交被拒绝且不替换当前快照。
- [x] 代码、测试和模块说明一致；不新增无消费者的 FFI/QML API。
- [ ] CI 验收通过并同步最终状态。

## 验证计划与结果

验证快照提交、失效作用域、项目替换/销毁取消和资源根约束；不把内存夹具视作完整工程持久化。本任务没有新增 FFI 所有权测试，因为没有运行时 FFI 消费者。

| 日期 | 环境 / 命令 | 实际结果 |
|---|---|---|
| 2026-10-18 | 仓库根目录；macOS 26.3.1 arm64、Rust/Cargo 1.98.1；`cargo verify-changed` | 通过。格式检查通过；受影响 Rust 包 `panta-core`、`panta-cli`、`panta-ffi`、`panta-launcher` 测试通过；相关 CTest 标签 `app`、`bridge`、`ffi`、`i18n`、`mesh`、`qml`、`qt-adapter` 共 67 项通过；未运行 workspace 全量测试。 |
| 2026-10-18 | `cargo test -p panta-core runtime::tests`；独立只读评审（`gpt-6.1-sol`, medium） | 运行时单元测试通过；评审复核完整变更与模块文档，未发现待处理问题。 |

## 风险与回退

上下文变成全局可变容器；以工程作用域和不可变快照限制，失败保留最后有效修订。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-10-18：先核对 `paths-and-runtime.md` 与当前实现，再修正文档边界；确认 023 的路径服务已落地，但标准目录尚未注入 `ProjectService` 运行时，`session` 临时目录也没有通用任务级管理器。024 只实现工程根下现存资源解析，不扩展写入、进程、网络或无消费者的 FFI。
- 2026-10-18：`RuntimeContext` 接入 `ProjectService`；工程替换/服务销毁关闭旧上下文并协作取消捕获的作用域。取消是协作式的，不宣称中断已进入的文件系统调用，也未接入 008 演示型 `TaskManager`。

## 完成摘要

实现与本地适用验证已完成；等待 CI 验收后同步最终状态。
