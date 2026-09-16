# 001 — Cargo workspace 与 Rust 工具链

- 状态：ready
- 阶段：M0
- 依赖：无
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

建立可检查的最小 Rust workspace，为后续统一构建入口提供可靠起点。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

## 必读

- [通用规范：repository-hygiene](../standards/repository-hygiene.md)

- [规范：baseline](../standards/baseline.md)
- [规范：rust](../standards/rust.md)
- [规范：cargo](../standards/cargo.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [架构：repository-layout](../architecture/repository-layout.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不接 CMake/Qt，不创建所有空业务 crate，不安装全套 native 依赖。

## 前置条件与待决策

本任务无前置任务；动手前核实当前平台、Rust 工具链来源及选定 CXX 版本的要求。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 确认主验证环境并选择支持 edition 2024 的固定 stable 工具链；记录 rustc/cargo 版本及 rust-version 策略，并核对首选 CXX release 的 MSRV，不能只按 edition 的最低版本选工具链。
2. 建立最少必要成员，选定唯一默认 launcher package；集中声明 edition、resolver 3、公共依赖和 lint 继承。
3. 提交应用 Cargo.lock、工具链配置与格式约定；检查忽略规则覆盖产物且不忽略锁文件。
4. launcher 暂只提供明确的未接入桌面诊断；更新 README，区分 Rust 骨架可用和桌面尚未可用。

## 预计改动

Cargo.toml、Cargo.lock、rust-toolchain.toml、最小 crates/、按需 .cargo/ 与 README.md。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] cargo metadata 能正确解析实际成员，根目录运行目标唯一。
- [ ] cargo build --locked、cargo test --locked 和 cargo fmt --all -- --check 对现有 Rust 骨架成功。
- [ ] 新 checkout 可按记录安装工具链并重建；执行 cargo run 不会伪称启动了 GUI。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |

## 风险与回退

工具链可能支持 edition 2024 却不满足选定 CXX 的 MSRV；固定版本前核对两者。骨架检查通过只证明 Rust 入口可用。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
