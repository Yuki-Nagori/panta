# 031 — 预编译 native 依赖供给与 CMake package

- 状态：in-progress
- 阶段：交付基础
- 依赖：[002](002-dependency-baseline.md)、[004](004-cargo-native-orchestration.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

建立 VTK、OpenCASCADE、Netgen 及工具链的预编译优先供给，使开发者和 CI 消费固定 SDK，而不是每次本地从源码编译第三方库。当前已完成第一轮官方发布资产盘点；供给脚本、manifest、CMake package 注入和三平台 configure 仍未实施。缺少上游 SDK 的受信制品生产由 [038](038-native-sdk-artifact-production.md) 承接。

## 必读

- [依赖获取与主平台环境](../standards/dependency-acquisition.md)
- [技术基线](../standards/baseline.md)
- [CMake 规范](../standards/cmake.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

范围：为 macOS arm64、Linux x86_64/aarch64 和 Windows x86_64 规划并实现可复核的预编译包清单、下载/缓存/校验、解包布局、CMake package 查找和 ABI/模块诊断；首批覆盖 VTK 9.7.0（含 `GUISupportQtQuick`）、OpenCASCADE 8.0.1 与 Netgen v6.2.2604，工具链 CMake/Ninja 沿用 020 的供给边界。

非目标：不把第三方源码放入普通 `cargo build` 或 CMake 构建图，不使用未锁定的系统包管理器，不在本任务实现 VTK/OCCT/Netgen 适配器，不引入 Python runtime 或用 Python wheel 冒充 C++ SDK。没有合适上游 SDK 时，可另立 CI 制品生产任务；不能静默退回本地源码构建。

## 前置条件与待决策

002、004 已完成。开始前逐平台核实：官方/可信 SDK 是否包含所需 CMake config、头文件、动态库和许可证；包与 Qt 6.11.2、Apple clang/libc++、Linux glibc/编译器及 Windows MSVC ABI 的匹配；VTK `GUISupportQtQuick` 是否存在。若上游只提供源码或 Python wheel，记录事实并评估项目制品，而不是直接改成 FetchContent。

## 实施步骤

1. 盘点各平台可用预编译 SDK、官方发布资产或可信制品源，记录 URL、版本、SHA256、架构、编译器/运行库 ABI、Qt 兼容范围、模块和许可证。
2. 设计统一 staging 布局和 manifest，下载使用固定哈希、断点/缓存策略和清晰失败诊断；禁止覆盖已校验的不同版本资产。
3. 将 staging 目录注入 CMake `find_package(... CONFIG)`，验证 VTK/OCCT/Netgen imported targets 与 transitive runtime；不把平台库名散落在适配器中。
4. 在三平台至少完成 configure/package smoke；检查缺包、哈希错误、架构/ABI 不匹配、缺模块、离线缓存和任意工作目录，记录无法覆盖的平台。
5. 只有在确实没有匹配预编译 SDK 时，另立源码制品生产任务；其结果必须作为可缓存、可校验的发布包供开发者消费。

## 预计改动

`native/cmake/` 供给脚本、构建引导/manifest、CMake package 查找配置、安装说明和固定清单；实际目录依平台包布局确定。不得把下载归档、解包目录或本机绝对路径提交进仓库。

## 清理与兼容例外

删除普通开发流程中的第三方源码 FetchContent、未锁定系统路径和重复依赖入口（如实施中出现）。无兼容例外；源码制品 fallback 另立任务，不以 `COMPAT` 长期保留。

## 验收标准

- [ ] 正常开发 configure/build 只消费校验通过的预编译 SDK，不编译 VTK、OCCT 或 Netgen 第三方源码。
- [ ] 每个支持平台的包记录 URL、SHA256、版本、架构、ABI、Qt 兼容范围、模块、许可证和 CMake package 入口。
- [ ] 缺包、哈希错误、架构/ABI 不匹配、缺少 `GUISupportQtQuick` 或 CMake target 时立即失败，并显示可操作诊断；不退回系统库或隐式源码编译。
- [ ] 清除缓存后可按 manifest 重建，已有正确缓存支持离线重复 configure；失败不会破坏另一版本缓存。
- [ ] 007、009、010 能以 imported targets 接入，不需要在各适配器重复写平台路径；安装产物能定位运行库和许可证。
- [ ] 三平台验证证据真实记录；未提供官方包的平台明确标记未覆盖，并有下一步制品任务或决策。
- [ ] 代码、构建、文档、固定清单与 task 自洽，废弃源码入口和失效引用已清理。

## 验证计划与结果

执行供给脚本/manifest 的哈希、缓存、离线、失败诊断测试；对每个平台运行 CMake configure 与最小链接/运行冒烟。第三方库本体测试不在此重复，由 007/009/010 负责集成行为。当前未执行。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | 仅完成规划 | 未执行；等待逐平台预编译 SDK 盘点 |
| 2026-09-17 | `gh api repos/Open-Cascade-SAS/OCCT/releases/latest`；下载并检查 `opencascade-release-no-pch.zip` | OCCT `V8.0.1` 提供官方 Windows 预编译 SDK；外层归档 SHA256 为 `307f694f1d4a280c7f58ee2ddb69a7f7e2b78d82749339efd40ce8b8b116d76c`，内层 `opencascade-8.0.1-vc14-64.zip` SHA256 为 `24d947bf045e8da43f559592d28eee4df389dd8034754d70f3ab341346a22ca8`；归档含 `cmake/OpenCASCADEConfig.cmake`、头文件、Windows DLL/库和许可证。尚未证明其与本项目 Qt/编译器 ABI 的集成。 |
| 2026-09-17 | [VTK 官方下载页](https://vtk.org/download/) 与最新 tag `v9.7.0` 资产盘点 | 官方页面明确提供源码归档、Python wheels，并提到 SDK packages；本轮未找到可直接消费的、包含 `GUISupportQtQuick`/`QQuickVTKItem` 的 macOS arm64、Linux 或 Windows C++ SDK 及 SHA256。因此不能将 Python wheel 或源码归档当作 C++ 预编译依赖；需另立项目制品生产/发布任务。 |
| 2026-09-17 | Netgen tag/release 资产盘点（当前清单仍固定 `v6.2.2604`） | 更新 tag `v6.2.2607` 可见但没有对应 GitHub release 预编译资产；当前 `v6.2.2604` 也未取得三平台 SDK。保持现有版本候选和 commit pin，不在本任务内擅自升级或触发源码构建；需要与 OCCT ABI 一起由后续制品任务验证。 |

## 风险与回退

上游可能只发布源码或与 Qt/编译器 ABI 不匹配；保留固定版本与回退点，优先切换已验证项目制品，不把本地长时间编译作为回退。缓存损坏时删除对应归档并重新校验，不影响其他版本；供给失败阻止 configure，避免生成半可用构建树。

## 决策与工作记录

- 2026-09-16：根据维护者要求将 native 第三方依赖改为预编译优先，新增本任务统一供给；007 暂不接入源码构建。
- 2026-09-17（官方资产盘点）：OCCT `V8.0.1` 的 Windows 官方归档是当前唯一取得可复核 CMake SDK 的候选，URL 为 `https://github.com/Open-Cascade-SAS/OCCT/releases/download/V8.0.1/opencascade-release-no-pch.zip`；其余平台仍未覆盖。VTK 官方渠道本轮只确认源码归档/Python wheels/SDK 提示，未确认可用 C++ SDK；Netgen `v6.2.2607` 只有 tag、没有 release 资产。以上只作为供给实现的输入，不代表集成通过。
- 2026-09-17（范围决策）：在 macOS arm64、Linux x86_64/aarch64、Windows x86_64 的固定平台资产完成 URL/SHA256/ABI/CMake target 记录前，031 保持 `in-progress`，007/009/010 不启动第三方源码构建；缺少上游资产时另立可缓存、可校验的项目制品任务。

## 完成摘要

未完成。已完成首轮官方资产盘点并确认 OCCT Windows 候选；仍等待全平台 SDK manifest、供给实现及 configure/package 冒烟证据。
