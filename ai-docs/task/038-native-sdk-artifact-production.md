# 038 — Native SDK 制品生产与发布

- 状态：planned
- 阶段：交付基础
- 依赖：[031](031-prebuilt-native-dependencies.md)、[020](020-toolchain-provisioning.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-17 / 2026-09-17

## 目标与背景

VTK、OpenCASCADE、Netgen 在目标平台没有统一、完整且可直接消费的官方 C++ SDK。为了坚持预编译优先，由受信 CI 按固定源码 commit 和工具链生产可复用 SDK，开发者和普通 CI 只下载校验后的归档，不在本地构建第三方源码。当前仅完成任务编排，尚未生成制品。

## 必读

- [Native 依赖供给模块](../modules/native-dependency-supply.md)
- [依赖获取与主平台环境](../standards/dependency-acquisition.md)
- [技术基线](../standards/baseline.md)
- [CMake 规范](../standards/cmake.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

范围：为 macOS arm64、Linux x86_64/aarch64、Windows x86_64 生成 VTK/OCCT/Netgen SDK；固定源码 commit、构建器镜像、CMake 选项和 ABI；运行 package 自检；产出 manifest、SHA256、SBOM、许可证集合、provenance 和可供 031 下载的归档。

非目标：不在开发者工作区编译第三方源码，不实现 VTK/OCCT/Netgen 适配器，不替换 031 的下载/缓存逻辑，不把 Python wheel 或系统包管理器作为 C++ SDK。

## 前置条件与待决策

031 必须先冻结每个依赖的版本、模块和 CMake package 入口；020 必须提供可复核的 CMake/Ninja 工具链。开始生产前核实各上游许可证、Qt 6.11.2 兼容范围、目标平台编译器/运行库 ABI、Netgen × OCCT 组合和 VTK `GUISupportQtQuick` 是否能在制品中启用。发布位置、签名服务和 runner 隔离策略待项目维护者确定。

## 实施步骤

1. 为每个依赖建立可审计的构建描述：源码 URL/commit、工具链版本、配置开关、依赖版本、禁用项和许可证来源。
2. 在隔离的 CI runner 生产各平台 SDK，构建结果只上传到制品存储，不自动写回源码仓库或开发者缓存。
3. 运行 package 自检，确认头文件、库、运行时、CMake config/imported targets、所需模块和许可证完整；检查架构、ABI 与 Qt 兼容元数据。
4. 生成归档 manifest、SHA256、SBOM 和 provenance，按依赖/版本/triple 隔离发布；更新 031 的固定清单和下载入口。
5. 让 007、009、010 使用真实制品完成最小集成，再记录未覆盖 GPU/平台并决定是否发布。

## 预计改动

CI workflow、构建描述、制品 manifest/校验脚本、许可证汇总、031 供给清单及发布文档。归档和构建树不提交源码仓库；具体存储路径和签名配置实施时确定。

## 清理与兼容例外

删除普通开发流程中出现的 FetchContent、系统路径或隐式源码 fallback；若当前不存在则记录无废弃项。无兼容例外；历史制品升级通过新版本 manifest 管理，不在构建逻辑中保留双路径。

## 验收标准

- [ ] 三平台每个目标依赖都有可下载、可校验的 SDK 归档，包含版本、triple、ABI、Qt 兼容范围、模块、CMake package、许可证和 SHA256。
- [ ] 受信 CI 可从固定源码 commit 和工具链重建相同 manifest；开发者 `cargo build`/CMake 不编译第三方源码。
- [ ] 缺包、哈希错误、架构/ABI 不匹配、缺 CMake target、缺 `GUISupportQtQuick` 或许可证时立即失败并给出诊断。
- [ ] 031 清除缓存后能按 manifest 重建，正确缓存支持离线 configure；不同版本缓存互不覆盖。
- [ ] 007、009、010 使用 imported targets 完成对应平台的最小链接/运行冒烟，未覆盖环境明确记录。
- [ ] SBOM、许可证和 provenance 随制品发布，升级/回退不破坏上一有效版本。
- [ ] 代码、CI、固定清单、文档和 task 自洽，废弃源码入口已清理且无未登记兼容分支。

## 验证计划与结果

先在隔离 runner 运行依赖包自检和哈希复现，再在三平台清除缓存执行 configure/package smoke；以 031、007、009、010 的真实集成命令作为最终证据。当前未执行。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 依据 031 官方资产盘点建立制品任务 | 未执行；尚未冻结构建描述、发布存储和签名策略 |

## 风险与回退

构建器差异、许可证遗漏、Netgen/OCCT ABI 不匹配或 VTK QtQuick 模块缺失会产生不可用 SDK。发布前 package 自检失败即阻止上传；升级保留上一 manifest 和归档，供给脚本只切换到完整校验通过的版本。

## 决策与工作记录

- 2026-09-17：由 031 官方资产盘点拆分；上游缺少全平台 C++ SDK 时，采用受信 CI 生成一次、开发者复用的制品路线。任务不授权普通本地构建源码。

## 完成摘要

未完成。等待 031 固定清单与 020 工具链供给，然后实施隔离构建、包自检和发布。
