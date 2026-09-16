# 004 — Cargo 调度 CMake 与运行入口

- 状态：ready
- 阶段：M0
- 依赖：[001](001-cargo-config.md)、[003](003-cmake-native-skeleton.md)（均已完成：workspace 与 native CMake 骨架就绪）
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

让根目录 Cargo 命令构建并运行 CMake 产物，验证无环依赖与增量追踪。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

## 必读

- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [规范：ninja](../standards/ninja.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不将当前 native 骨架称为桌面；CTest 聚合由 011 完成。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 选定 build.rs/cmake crate/辅助调度方案，记录 Cargo→CMake 图和未来 Rust native 库接入位置。
2. 映射 profile、target、依赖前缀和产物目录；build script 生成物只写 OUT_DIR。
3. 实现 launcher 对 native 产物定位、参数和退出码转发；不依赖启动 cwd，不拼 shell 字符串。
4. 追踪 CMake、C++、QML/资源目录及环境参数；此时没有 QML 时先定义扩展点，005 补齐实际验证。

## 预计改动

launcher、build.rs/调度模块、Cargo manifest、native CMake 与构建说明。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] cargo build --locked 可构建 native 骨架；cargo run 启动 native 骨架并正确返回退出码。
- [ ] 首次/无改动/修改 C++ 与配置后构建行为符合预期，失败构建不被 launcher 掩盖。
- [ ] 含空格的工作路径及 Debug/Release 定位正确，没有 Cargo↔CMake 递归。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |

## 风险与回退

Cargo/CMake 可能递归构建，或遗漏 QML/环境参数变化导致旧产物被复用；先验证无环依赖和重建追踪，再扩展桌面目标。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
