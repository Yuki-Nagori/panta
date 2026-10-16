# 121 — GUI 与 CLI 共享 Rust 命令入口

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)、[107 Rust 工程服务边界](107-rust-service-boundaries-and-revisions.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 GitHub issue #36 的统一命令/查询入口部分。GUI 与未来 CLI 不应各自复制领域校验或流程编排；Rust 应用层拥有命令输入、领域校验、执行结果和可诊断错误，Qt/C++ 与 CLI 仅将各自输入适配为同一接口并呈现结果。

## 必读

- [分层标准](../standards/layering.md)
- [仓库服务与修订边界](../architecture/application-and-storage.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [任务 107](107-rust-service-boundaries-and-revisions.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 首个共享操作采用“打开工程”：GUI 继续经现有 FFI 打开工程；新增最小 `panta-cli project inspect <path>` 消费同一 Rust 命令并输出工程摘要。
- 在 `panta-core` 的应用入口定义 Qt 无关的打开工程命令，由 GUI FFI 与 CLI adapter 共同调用；两侧不得复制路径、扩展名或清单校验。
- 命令输入/输出使用 Qt 无关的 Rust 类型；错误保持稳定、可翻译或可供 CLI 诊断。
- 测试命令行为和 GUI/CLI 两个调用入口；文档说明未来新增命令的归属及开发调用方法。

非目标：把 QML 控件状态、Qt 类型、文件选择器或显示逻辑迁入 Rust；一次性建立大型命令总线/插件系统；修改工程格式。

## 涉及分组与跨组依赖

- 涉及分组：Rust 领域/应用服务、CLI、FFI、Qt bridge/UI。
- 是否新增跨组依赖：由应用服务提供命令；GUI 和 CLI 依赖应用服务，不允许 Rust 反向依赖 Qt/bridge。

## 验收标准

- [x] 一条用户可观察的真实命令/查询由 Rust 应用入口定义并实现。
- [x] GUI 经 FFI/bridge 与 CLI 都调用同一应用入口，行为测试证明两侧共用执行逻辑。
- [x] 输入校验、错误语义和结果结构不依赖 Qt；调用侧只做适配与呈现。
- [x] 配套文档、crate 依赖方向、测试选择、本地适用验证与独立只读评审完成。
- [x] 分支 CI 通过；通过后将任务和索引转为 `done`。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | macOS；初次 `cargo verify-changed` | L2 Rust 包、质量检查和相关 native/QML 测试通过 | 11 个 Rust 包和 77/77 CTest 通过。此轮早于最后的相对路径与共享黑盒测试归档修正。 |
| 2026-10-16 | `cargo test --locked -p panta-cli`；定向 `panta-core`、`panta-ffi`、`panta-tests` 测试 | CLI 真进程行为、共享 Rust 命令、GUI FFI 转发和测试范围选择通过 | CLI 6/6；core 打开命令 1/1；FFI bridge 1/1；测试选择器 1/1 均通过。 |
| 2026-10-16 | `cargo format --check`；`cargo clippy --locked --all-targets -p panta-cli -p panta-core -p panta-ffi -p panta-tests -- -D warnings` | 格式与受影响 Rust 包 Clippy 检查通过 | 均通过；该记录是最终聚合验证之前的定向检查。跨平台 CI 结果见下一行。 |
| 2026-10-16 | 最终 `cargo verify-changed` | L2 Rust、完整适用质量检查和 native/QML 测试通过 | 11 个 Rust 包、77/77 CTest 均通过；包含 clang-tidy、include-cleaner、cppcheck、QML 与 Rust/CMake 分层检查。 |
| 2026-10-16 | 独立只读评审（`gpt-6.1-sol`, medium） | 无未解决问题 | 复核完整 diff；相对路径、真 CLI 进程测试、共享黑盒测试位置及 L1/L2 选择器均已检查，无剩余发现。 |
| 2026-10-16 | [PR #46 首轮 CI](https://github.com/Yuki-Nagori/panta/actions/runs/38904766090) | macOS、Linux、Windows 构建及所有适用 lint/quality checks 通过 | 所有 checks 通过；三平台构建、Clippy、clang-tidy、cppcheck、include-cleaner、QML lint、CMake lint、分层和依赖检查均成功。 |

## 风险与回退

`panta-cli` 当前是开发用窄 CLI，不替代桌面 launcher，也不代表产品命令行界面已完整。后续 CLI 命令必须继续调用共享应用入口；不要在参数适配层复制业务校验。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；先消费任务 118 的边界设计，再由本任务根据现有应用服务选择共享命令。
- 2026-10-16：确认 `ProjectService::open` 已是 GUI 工程打开路径；本任务将其收敛到共享的 Rust 打开工程命令，并以只读 `panta-cli project inspect` 作为窄 CLI 消费者，不增加空的命令框架。
- 2026-10-16：新增 `ProjectCommand::Open`，GUI FFI 与 CLI 均通过 `ProjectService::execute` 调用；CLI adapter 将相对路径按工作目录解析为绝对路径。新增真实二进制测试、显式注册的 `tests/rust/cli.rs` 黑盒测试，以及 CLI/core 影响范围回归。更新 Rust 分层 lint 和测试选择器以识别新 crate 与共享测试文件。
- 2026-10-16：本地适用验证和独立评审通过；分支 CI 尚未运行，任务先标记 `implemented`，待 CI 通过后转 `done`。
- 2026-10-16：根据独立评审将 usage 提示中的程序名修正为 `panta-cli`，同步 CLI 进程测试断言；重新运行 `cargo test --locked -p panta-cli` 与 `cargo verify-changed` 均通过。
- 2026-10-16：PR #46 首轮 CI run 38904766090 全部通过，任务验收完成并转为 `done`。

## 完成摘要

GUI 与 CLI 已共用工程打开应用命令；真实 CLI 进程测试、L1/L2 选择器回归、本地聚合验证、独立评审及 PR #46 三平台 CI 均通过。
