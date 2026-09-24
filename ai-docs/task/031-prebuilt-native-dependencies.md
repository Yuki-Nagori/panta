# 031 — 预编译 native 依赖供给与 CMake package

- 状态：in-progress
- 阶段：交付基础
- 依赖：[002](002-dependency-baseline.md)、[004](004-cargo-native-orchestration.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

建立 VTK、OpenCASCADE、Netgen 及工具链的预编译优先供给，使开发者和 CI 消费固定 SDK，而不是每次本地从源码编译第三方库。供给脚本、manifest、CMake package 注入和三平台制品自检已落地；VTK WebGPU 制品由 [038](038-native-sdk-artifact-production.md) 生产并完成三平台 Release。当前剩余为 007/009/010 的真实消费与运行集成证据。

## 必读

- [依赖获取与主平台环境](../standards/dependency-acquisition.md)
- [技术基线](../standards/baseline.md)
- [CMake 规范](../standards/cmake.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

范围：为 macOS arm64、Linux x86_64/aarch64 和 Windows x86_64 规划并实现可复核的预编译包清单、下载/缓存/校验、解包布局、CMake package 查找和 ABI/模块诊断；首批覆盖 VTK 9.7.0（WebGPU + `RenderingUI`/Cocoa hardware window）、OpenCASCADE 8.0.1 与 Netgen v6.2.2604，工具链 CMake/Ninja 沿用 020 的供给边界。

非目标：不把第三方源码放入普通 `cargo build` 或 CMake 构建图，不使用未锁定的系统包管理器，不在本任务实现 VTK/OCCT/Netgen 适配器，不引入 Python runtime 或用 Python wheel 冒充 C++ SDK。没有合适上游 SDK 时，可另立 CI 制品生产任务；不能静默退回本地源码构建。

## 前置条件与待决策

002、004 已完成。开始前逐平台核实：官方/可信 SDK 是否包含所需 CMake config、头文件、动态库和许可证；包与 Apple clang/libc++、Linux glibc/编译器及 Windows MSVC ABI 的匹配；VTK `RenderingWebGPU`、`RenderingUI` 及目标平台 hardware window 是否存在。若上游只提供源码或 Python wheel，记录事实并评估项目制品，而不是直接改成 FetchContent。

## 实施步骤

1. 盘点各平台可用预编译 SDK、官方发布资产或可信制品源，记录 URL、版本、SHA256、架构、编译器/运行库 ABI、Qt 兼容范围、模块和许可证。（首轮已完成，见验证表）
2. 设计统一 staging 布局和 manifest，下载使用固定哈希、断点/缓存策略和清晰失败诊断；禁止覆盖已校验的不同版本资产。
3. 将 staging 目录注入 CMake `find_package(... CONFIG)`，验证 VTK/OCCT/Netgen imported targets 与 transitive runtime；不把平台库名散落在适配器中。
4. 在三平台至少完成 configure/package smoke；检查缺包、哈希错误、架构/ABI 不匹配、缺模块、离线缓存和任意工作目录，记录无法覆盖的平台。
5. 只有在确实没有匹配预编译 SDK 时，另立源码制品生产任务；其结果必须作为可缓存、可校验的发布包供开发者消费。

## 预计改动

`native/cmake/` 供给脚本、构建引导/manifest、CMake package 查找配置、安装说明和固定清单；实际目录依平台包布局确定。不得把下载归档、解包目录或本机绝对路径提交进仓库。

## 清理与兼容例外

删除普通开发流程中的第三方源码 FetchContent、未锁定系统路径和重复依赖入口（如实施中出现）。无兼容例外；源码制品 fallback 另立任务，不以 `COMPAT` 长期保留。

## 验收标准

- [x] 正常开发 configure/build 只消费校验通过的预编译 SDK，不编译 VTK、OCCT 或 Netgen 第三方源码。
- [x] 每个支持平台的包记录 URL、SHA256、版本、架构、ABI、Qt 兼容范围、模块、许可证和 CMake package 入口。
- [x] 缺包、哈希错误、架构/ABI 不匹配、缺少 `RenderingWebGPU`/`RenderingUI` 或 CMake target 时立即失败，并显示可操作诊断；不退回系统库或隐式源码编译。
- [x] 清除缓存后可按 manifest 重建，已有正确缓存支持离线重复 configure；失败不会破坏另一版本缓存。
- [ ] 007、009、010 能以 imported targets 接入，不需要在各适配器重复写平台路径；安装产物能定位运行库和许可证。
- [x] 三平台验证证据真实记录；未提供官方包的平台明确标记未覆盖，并有下一步制品任务或决策。
- [ ] 代码、构建、文档、固定清单与 task 自洽，废弃源码入口和失效引用已清理。

## 验证计划与结果

执行供给脚本/manifest 的哈希、缓存、离线和失败诊断测试；对每个平台运行 CMake configure 与最小链接/运行冒烟。fixture 级供给路径、三平台 SDK 制品生产/自检、macOS 生产 consumer 以及 run 36001859191 的三平台 SDK 消费和 CTest 均有记录。剩余应用运行时分发与真实窗口/引擎集成由 007/009/010 验收，不因 SDK configure 成功而视为闭环。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 官方 VTK、OCCT、Netgen 资产盘点 | 仅 OCCT 8.0.1 有可复核的官方 Windows SDK，无法覆盖本项目所需的三平台 ABI；VTK 缺可用 C++ SDK，Netgen 无固定版本预编译资产。因此转为项目维护的固定 SDK 制品，不以源码/Python 包代替 C++ SDK。 |
| 2026-09-17 | `ctest --preset debug -R Build.SdkProvision`、`ctest --preset debug`、`cargo build --locked`（macOS arm64） | SDK 供给 fixture 10/10、CTest 16/16、Cargo build 通过；覆盖 SHA256、离线缓存、坏缓存、归档布局、`find_package` 与 imported targets。早期三平台 configure CI [35230584355](https://github.com/Yuki-Nagori/panta/actions/runs/35230584355) 通过，但当时 CI 尚未聚合 CTest。 |
| 2026-09-18 | OCCT/Netgen Release [35307708622](https://github.com/Yuki-Nagori/panta/actions/runs/35307708622)；macOS 生产 consumer | 三平台六个 SDK 资产、许可证与 manifest 通过；macOS 两个 consumer 完成 Release 下载、哈希校验、`find_package` 和 target 自检，离线复用通过。Netgen 包名大小写差异已按 Linux 实测统一为 `Netgen`。 |
| 2026-09-20 | VTK WebGPU Release [sdk-vtk-9.7.0-webgpu](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-vtk-9.7.0-webgpu)，run [35479202321](https://github.com/Yuki-Nagori/panta/actions/runs/35479202321) | 三平台 production/selfcheck/package 成功；manifest 记录 SHA256、C++20、WebGPU targets 与 Cocoa/Wayland/Win32 窗口系统。旧 Qt/OpenGL 制品不再作为当前输入。 |
| 2026-09-24 | SDK-consuming CI [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | macOS/Linux/Windows 构建与 CTest 通过（56/56、56/56、54/54）；确认当前 SDK 可被测试工程消费，不代表应用运行时分发或真实窗口集成已完成。 |


## 风险与回退

上游可能只发布源码或与 Qt/编译器 ABI 不匹配；保留固定版本与回退点，优先切换已验证项目制品，不把本地长时间编译作为回退。缓存损坏时删除对应归档并重新校验，不影响其他版本；供给失败阻止 configure，避免生成半可用构建树。

## 决策与工作记录

- 2026-09-16–17：确定预编译 SDK 优先。因上游资产不能覆盖三平台 ABI，落地 `sdk-provision.cmake`，以 manifest、SHA256、隔离缓存、原子 staging 和 imported-target 自检供给依赖；fixture 与早期构建证据见验证表。
- 2026-09-18–20：VTK WebGPU、OCCT 与 Netgen 三平台制品发布并登记；OCCT/Netgen 作为 ABI 配对制品，VTK 则独立发布。旧 VTK Qt/OpenGL manifest 已替换。
- 2026-09-24：run 360018 提供三平台 SDK 消费和 native CTest 证据。应用运行时分发、真实窗口/引擎集成、Linux glibc 基线及 SBOM/provenance 仍由本任务与 007/009/010 收尾。

## 完成摘要

未完成（保持 in-progress）。VTK WebGPU、OCCT 和 Netgen 的固定资产、SHA256、ABI、模块、许可证与 CMake package 入口均已登记；旧 VTK Qt/OpenGL 条目已被新 WebGPU manifest 替换。当前剩余项是 007/009/010 的真实链接/运行冒烟、Linux glibc 有效基线回写，以及消费侧 SBOM/provenance 自动化。
