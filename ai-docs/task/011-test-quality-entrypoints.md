# 011 — 统一测试与质量入口

- 状态：in-progress
- 阶段：验证基础
- 依赖：[007](007-vtk-quick-viewport.md)、[008](008-tasks-errors-logging.md)、[010](010-netgen-adapter-smoke.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

让统一入口可证明 Rust、native 与 QML 的检查实际执行，形成 CI 可复用命令。

本任务整体仍未完成；本次先落地当前 Rust workspace 的 Clippy 质量门禁。依赖任务尚未全部完成，因此不把 CTest/QML 聚合和覆盖率能力提前标成完成。

## 必读

- [通用规范：validation-and-review](../standards/validation-and-review.md)
- [通用规范：documentation](../standards/documentation.md)

- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [规范：cpp](../standards/cpp.md)
- [规范：rust](../standards/rust.md)
- [规范：qml](../standards/qml.md)
- [规范：qt](../standards/qt.md)
- [规范：vtk](../standards/vtk.md)
- [架构：milestones-and-validation](../architecture/milestones-and-validation.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。本轮范围收窄为：清理现有 Clippy warning，并让 workspace 的 Clippy warning 在本地和 CI 中直接失败。

非目标：不要求文档/无代码目录执行不存在的检查，不建设大型测试框架。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 清点已存在测试并分 Rust、CTest、QML lint 和图形冒烟，标注平台/显示服务要求。
2. 实现 cargo test 的原生聚合调度，防止从聚合测试递归调用自身；失败返回非零。
3. 接入 rustfmt、Clippy、C++ 格式/选定静态分析及文档链接检查，并记录真实命令。
4. 区分默认自动检查、目标平台图形检查和可选 sanitizer，规定跳过必须显式报告。
5. 当前 Rust workspace 以 `-D warnings` 运行 Clippy；测试与 build script 不通过 `expect`/`unwrap` 警告豁免掩盖问题。

## 预计改动

测试聚合入口、CTest 注册、质量工具配置、文档检查与使用说明。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] cargo test --locked 实际执行约定 Rust/native 套件；受控失败能导致顶层命令失败。
- [x] 当前 Rust workspace 的 Clippy warning 直接失败；既有 warning 已清理，CI 与本地命令保持一致。
- [ ] 报告含测试数量和跳过原因；零个意外缺失的 native 测试不能视作通过。
- [ ] QML 检查与真实图形冒烟有明确执行方式，格式/静态检查针对实际源码。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | `cargo clippy --locked --workspace --all-targets -- -D warnings`（完整 workspace，复用已缓存 Qt staging）；`cargo fmt --all -- --check`；`cargo test --locked --workspace --exclude panta-launcher` | Rust warning 直接失败且测试/格式通过 | 完整 workspace Clippy 通过且无 warning；Rust 测试 15/15；fmt 通过 |

## 风险与回退

聚合命令可能漏跑 native 测试、递归调用自身或将跳过当成功；用受控失败与执行数量核验实际覆盖。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-17：根据质量要求先收敛 Rust workspace 门禁；workspace lints 将 Clippy warning 提升为 deny，CI 额外传入 `-D warnings`，并清理 FFI build script/测试中的 `expect` warning。
- 待记录：CTest/QML 聚合、跨语言静态分析和覆盖率方案，依赖任务完成后继续实施。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
