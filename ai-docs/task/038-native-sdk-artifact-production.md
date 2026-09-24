# 038 — Native SDK 制品生产与发布

- 状态：in-progress
- 阶段：交付基础
- 依赖：[031](031-prebuilt-native-dependencies.md)、[020](020-toolchain-provisioning.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-17 / 2026-09-20

## 目标与背景

VTK、OpenCASCADE、Netgen 在目标平台没有统一、完整且可直接消费的官方 C++ SDK。为了坚持预编译优先，由受信 CI 按固定源码 commit 和工具链生产可复用 SDK，开发者和普通 CI 只下载校验后的归档，不在本地构建第三方源码。VTK WebGPU 制品（Release `sdk-vtk-9.7.0-webgpu`）与 OCCT/Netgen 制品（Release `sdk-occt-netgen-8.0.1-6.2.2604`）已生产并登记进 031 manifest；剩余为 SBOM/provenance 发布闭环与 007/009/010 的真实链接/运行冒烟覆盖。

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

031 必须先冻结每个依赖的版本、模块和 CMake package 入口；020 必须提供可复核的 CMake/Ninja 工具链。开始生产前核实各上游许可证、目标平台编译器/运行库 ABI、Netgen × OCCT 组合和 VTK `RenderingWebGPU`/`RenderingUI` 是否能在制品中启用。VTK 制品不再链接 Qt；发布位置、签名服务和 runner 隔离策略待项目维护者确定。

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
- [ ] 缺包、哈希错误、架构/ABI 不匹配、缺 CMake target、缺 `RenderingWebGPU`/`RenderingUI` 或许可证时立即失败并给出诊断。
- [ ] 031 清除缓存后能按 manifest 重建，正确缓存支持离线 configure；不同版本缓存互不覆盖。
- [ ] 007、009、010 使用 imported targets 完成对应平台的最小链接/运行冒烟，未覆盖环境明确记录。
- [ ] SBOM、许可证和 provenance 随制品发布，升级/回退不破坏上一有效版本。
- [ ] 代码、CI、固定清单、文档和 task 自洽，废弃源码入口已清理且无未登记兼容分支。

## 验证计划与结果

按阶段记录可复核的制品生产、消费和 CI 验收；失败修复只保留影响最终构建契约的根因。当前遗留项见下方“待执行与未覆盖”。

| 日期 | 阶段 / 证据 | 结果 / 边界 |
|---|---|---|
| 2026-09-17–18 | VTK Qt/OpenGL SDK 生产与 Release [35242622228](https://github.com/Yuki-Nagori/panta/actions/runs/35242622228) | 三平台制品曾成功生成并消费，但仅服务已废止的 `QQuickVTKItem` 路线；不作为当前 SDK。 |
| 2026-09-18 | OCCT/Netgen 三平台 Release [35307708622](https://github.com/Yuki-Nagori/panta/actions/runs/35307708622) | 6 个归档、SHA256、许可证与 manifest 通过。构建固定 OCCT/Netgen ABI 配对；CI 修正 Freetype/Xlib 无用依赖，并统一大小写敏感平台上的 `Netgen` package 名。 |
| 2026-09-19–20 | VTK WebGPU 构建修正与本机生产验证 | 最终固定 VTK 9.7 要求的 Dawn `v20260421.125655`，启用 RTTI，并让 WebGPU 编译目标使用 C++20；Linux 明确采用 Wayland-only，安装时携带必要 Find 模块。macOS 完整编译、selfcheck 与 package 通过。早期下载路径、归档布局、子模块和目标依赖问题已收敛为这些生产约束。 |
| 2026-09-20 | VTK WebGPU 三平台 Release [sdk-vtk-9.7.0-webgpu](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-vtk-9.7.0-webgpu)，run [35479202321](https://github.com/Yuki-Nagori/panta/actions/runs/35479202321) | 三平台 production/selfcheck/package 成功；required targets 为 `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn`，metadata 标记 C++20 与 Cocoa/Wayland/Win32。 |
| 2026-09-18–20 | macOS 生产 consumer 与 SDK 自检 | VTK、OCCT、Netgen 均完成 Release 下载、SHA256、`find_package` 和 required-target 检查；离线缓存复用通过。该证据说明制品可被最小 consumer 消费，不等于应用运行时分发闭环。 |
| 2026-09-24 | SDK-consuming CI [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | 三平台构建及 CTest 通过（Linux 56/56、macOS ASan/UBSan 56/56、Windows 54/54）；不替代 producer 可复现性复核、真实窗口/引擎集成或 SBOM/provenance 验收。 |

### 待执行与未覆盖

- 复核 OCCT/Netgen 当前 manifest 与生产发布链，记录 Linux glibc 最低有效基线。
- 完成 SDK 消费侧 SBOM/provenance，以及 007/009/010 的真实链接、运行时分发与引擎集成验收。


## 风险与回退

构建器差异、许可证遗漏、Netgen/OCCT ABI 不匹配或 VTK QtQuick 模块缺失会产生不可用 SDK。发布前 package 自检失败即阻止上传；升级保留上一 manifest 和归档，供给脚本只切换到完整校验通过的版本。

## 决策与工作记录

- 2026-09-17：采用受信 CI 构建、发布并校验固定版本的 SDK；开发者消费 Release，不以本地源码构建作为常规回退。VTK 源码固定 9.7.0 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`。
- 2026-09-18：OCCT 8.0.1 与 Netgen 6.2.2604 必须同工具链配对，故合并为一个三平台 workflow 和一个 Release；保留 STEP/DataExchange，关闭无关 GUI、Freetype/Xlib、Python/MPI 等依赖。
- 2026-09-19–20：007 路线转为 VTK WebGPU hardware window。生产依赖固定为 VTK 9.7 所要求的 Dawn 源码版本与 RTTI 配置、C++20；Linux 只构建 Wayland 后端，并把消费所需 CMake Find 模块随 SDK 安装。
- 2026-09-21（任务 049）：Release 重产在单一 tag 内全量替换资产；Netgen 关闭 `USE_NATIVE_ARCH`，避免制品依赖构建机 ISA。
- 后续变更需同步 manifest 的 URL/SHA256、SDK metadata、许可证、自检结果与 task 031；当前待办见上方“待执行与未覆盖”。

## 完成摘要

未完成（保持 in-progress）。VTK WebGPU hardware-window 三平台 production/selfcheck/package/release 已完成，031 manifest 已切换并记录真实 SHA256、ABI、targets 与窗口系统；OCCT/Netgen 合并 workflow 已有三平台 Release。剩余：OCCT/Netgen manifest 最终复核、SBOM/provenance 自动化、007/009/010 的真实链接/运行冒烟，以及 Linux glibc 有效基线回写。
