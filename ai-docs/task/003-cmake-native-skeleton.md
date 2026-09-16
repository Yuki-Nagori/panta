# 003 — CMake/Ninja 原生构建骨架

- 状态：ready
- 阶段：M0
- 依赖：[002](002-dependency-baseline.md)（已完成：主平台与依赖固定清单就绪）
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

建立有安装和测试入口的自有 C++20 target，明确 native 构建目录与配置。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

## 必读

- [通用规范：comments](../standards/comments.md)
- [通用规范：repository-hygiene](../standards/repository-hygiene.md)

- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [规范：ninja](../standards/ninja.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [架构：repository-layout](../architecture/repository-layout.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不接 Rust FFI、Qt 或 CAE 算法；不手写 build.ninja。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 创建 native 顶层 CMake、最小可编译 target 与测试，选择单配置 Ninja 或 Multi-Config 并记录原因。
2. 为自有 targets 设置 C++20、标准必需、禁用扩展及局部告警；建立格式配置。
3. 建立共享 presets 和本机配置入口，显式定义 binary/install 路径，避免硬编码机器依赖前缀。
4. 增加 CTest 注册和安装规则，用最小 native 行为验证构建链；此时无需 Qt。

## 预计改动

native/CMakeLists.txt、最小 native target/test、CMakePresets.json、.clang-format。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] 干净 configure/build/ctest/install 均成功，记录实际 preset 名与命令。
- [ ] 无改动构建不重复编译，修改头/源可正确重建。
- [ ] Debug/Release 或相应配置目录隔离；缺失工具/依赖诊断可定位。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |

## 风险与回退

全局编译配置容易污染第三方 target，preset 与 CMake 版本也可能不匹配；将配置限制到自有 target 并验证干净构建。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
