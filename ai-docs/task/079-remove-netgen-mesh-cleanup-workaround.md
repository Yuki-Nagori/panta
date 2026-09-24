# 079 — 清理 Netgen mesh 销毁 workaround

- 状态：planned
- 阶段：CAE 接入基础维护
- 依赖：[010](010-netgen-adapter-smoke.md)、[038](038-native-sdk-artifact-production.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

移除 task 010 为 Netgen v6.2.2604 保留的调用侧析构 workaround。只有项目实际消费的 Netgen SDK 已包含安全的 OCC mesh 销毁实现后才执行；SDK 版本变化本身不代表缺陷已修复。

## 必读

- [Netgen 规范](../standards/netgen.md)
- [代码生命周期规范](../standards/code-lifecycle.md)
- [验证与评审规范](../standards/validation-and-review.md)

## 范围与非目标

范围仅包括删除 `native/mesh/src/netgen/netgen_mesher.cpp` 中 `COMPAT(task 010; remove-task 079)` 特例、恢复对 Netgen OCC mesh 的正常 `Ng_DeleteMesh` 调用、同步 task 010 与 Netgen 规范，并运行 sanitizer 和重复生成/释放验证。

非目标：不修改业务网格算法、Mesh IR、SDK 构建选项或 Netgen/OCCT 的 ABI 配对策略。

## 前置条件与待决策

- task 038 已发布、manifest 已登记且本仓库已消费包含安全 OCC mesh 析构实现的固定 Netgen SDK。
- 修复证据可追溯到上游 commit 或本项目正式 SDK patch；不能仅依据 release 版本号推断。
- 在 macOS ASan/UBSan 与三平台 CTest 上验证创建、转换、销毁及重复生成。

## 实施步骤

1. 核实 SDK 源码修复与制品 provenance，确认项目 staging 实际版本。
2. 删除兼容指针修正逻辑，恢复普通 `Ng_DeleteMesh` 生命周期。
3. 运行 task 010 的重复生命周期、sanitizer 和适用平台测试，记录结果。
4. 移除本任务的 COMPAT 标记与引用，保持 task 010、规范和索引一致。

## 预计改动

`native/mesh/src/netgen/netgen_mesher.cpp`、`tests/cpp/mesh/netgen_mesher_test.cpp`、`ai-docs/task/010-netgen-adapter-smoke.md`、`ai-docs/standards/netgen.md` 和本任务状态。

## 清理与兼容例外

本任务的唯一目的就是删除 `COMPAT(task 010; remove-task 079)`。不引入其他兼容分支。

## 验收标准

- [ ] 使用的 SDK 已有可追溯的安全 OCC mesh 销毁实现。
- [ ] 适配器不再含针对 v6.2.2604 的名称指针 workaround，成功与失败路径都释放资源。
- [ ] ASan/UBSan、重复生成/释放用例与三平台 native mesh 测试通过。
- [ ] task 010、Netgen 规范和索引准确反映移除结果及验证证据。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 等待固定 SDK 更新 | 未执行；前置条件未满足 |

## 风险与回退

升级 SDK 后仍可能存在其他 Netgen 内部所有权缺陷。验证失败时保留旧 adapter workaround 与本任务，不可先标记完成；更新任务证据并重新确认修复边界。

## 决策与工作记录

- 2026-09-24：登记 task 010 调用侧 workaround 的明确删除条件；仅在项目实际切换至含析构修复的固定 SDK 后执行。

## 完成摘要

未完成。等待 SDK 修复进入消费基线后删除 workaround 并回归。
