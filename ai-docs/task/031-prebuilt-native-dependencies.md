# 031 — 预编译 native 依赖供给与 CMake package

- 状态：deferred
- 阶段：交付基础
- 依赖：[002](002-dependency-baseline.md)、[004](004-cargo-native-orchestration.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-10-25

## 目标与背景

建立 VTK、OpenCASCADE、Netgen 及工具链的预编译优先供给，使开发者和 CI 消费固定 SDK，而不是每次本地从源码编译第三方库。消费端 manifest、下载校验、缓存和 CMake package 注入由本仓库维护；SDK 生产、制品自检与发布现由 [038](038-native-sdk-artifact-production.md) 跟踪，并在 [sdk-packaging](https://github.com/Jack-Lab-at-EIT/sdk-packaging) 实施。009/010 的 native adapter 冒烟已完成；本任务剩余为桌面产品运行时分发、Linux glibc 基线及外部制品 provenance/SBOM 的消费侧核验。

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
- [x] VTK、OCCT、Netgen adapter 通过 imported targets 接入，不在 adapter 重复写平台 SDK 路径。
- [ ] 安装产物能定位运行库和许可证；需在临时 install prefix 与搬迁目录验收。
- [x] 三平台验证证据真实记录；未提供官方包的平台明确标记未覆盖，并有下一步制品任务或决策。
- [x] Producer 迁移后，当前构建、消费 manifest、依赖说明和 task 边界自洽；已移除 Panta 的 producer workflow 和生产脚本。

## 验证计划与结果

执行供给脚本/manifest 的哈希、缓存、离线和失败诊断测试；对每个平台运行 CMake configure 与最小链接/运行冒烟。fixture 级供给路径、三平台 SDK 制品生产/自检、macOS 生产 consumer 以及 run 36001859191 的三平台 SDK 消费和 CTest 均有记录。它们证明 native targets 可消费制品，不等同于桌面安装包的运行时部署或真实窗口验收。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 官方 VTK、OCCT、Netgen 资产盘点 | 仅 OCCT 8.0.1 有可复核的官方 Windows SDK，无法覆盖本项目所需的三平台 ABI；VTK 缺可用 C++ SDK，Netgen 无固定版本预编译资产。因此转为项目维护的固定 SDK 制品，不以源码/Python 包代替 C++ SDK。 |
| 2026-09-17 | `ctest --preset debug -R Build.SdkProvision`、`ctest --preset debug`、`cargo build --locked`（macOS arm64） | SDK 供给 fixture 10/10、CTest 16/16、Cargo build 通过；覆盖 SHA256、离线缓存、坏缓存、归档布局、`find_package` 与 imported targets。早期三平台 configure CI [35230584355](https://github.com/Yuki-Nagori/panta/actions/runs/35230584355) 通过，但当时 CI 尚未聚合 CTest。 |
| 2026-09-18 | OCCT/Netgen Release [35307708622](https://github.com/Yuki-Nagori/panta/actions/runs/35307708622)；macOS 生产 consumer | 三平台六个 SDK 资产、许可证与 manifest 通过；macOS 两个 consumer 完成 Release 下载、哈希校验、`find_package` 和 target 自检，离线复用通过。Netgen 包名大小写差异已按 Linux 实测统一为 `Netgen`。 |
| 2026-09-20 | VTK WebGPU 历史生产 run [35479202321](https://github.com/Yuki-Nagori/panta/actions/runs/35479202321) | 当时三平台 production/selfcheck/package 成功；manifest 记录 SHA256、C++20、WebGPU targets 与 Cocoa/Wayland/Win32 窗口系统。Panta 中重复的 SDK Releases 与 tags 已在 2026-10-25 清理，当前资产由外部 `sdk-packaging` 提供。 |
| 2026-09-24 | SDK-consuming CI [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | macOS/Linux/Windows 构建与 CTest 通过（56/56、56/56、54/54）；确认当前 SDK 可被测试工程消费，不代表应用运行时分发或真实窗口集成已完成。 |
| 2026-10-25 | 外部 SDK 生产迁移与 Panta 消费清单 | Producer 迁至 sdk-packaging；Panta 删除三条 producer workflow 与 `tools/sdk/`。VTK run [40123093281](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40123093281) 三平台构建、自检、打包及 Release 发布成功；OCCT/Netgen、GoogleTest Release 资产也齐备。12 个资产的 SHA256 与 sidecar 和 Release API digest 一致，消费 manifest 已在本分支切换至外部 Release。消费端构建/测试见任务 038 的本轮记录；旧 Panta Releases 与 tags 已删除。 |


## 风险与回退

外部固定 Release 资产可能不可用或与 Qt/编译器 ABI 不匹配；供给失败时 configure 明确失败，不退回源码构建或系统库。恢复需由外部制品仓库重新发布已验证资产，或修正 Panta manifest 并复验；缓存损坏时只删除对应归档并重新校验，不影响其他版本。

## 决策与工作记录

- 2026-09-16–17：确定预编译 SDK 优先。因上游资产不能覆盖三平台 ABI，落地 `sdk-provision.cmake`，以 manifest、SHA256、隔离缓存、原子 staging 和 imported-target 自检供给依赖；fixture 与早期构建证据见验证表。
- 2026-09-18–20：VTK WebGPU、OCCT 与 Netgen 三平台制品发布并登记；OCCT/Netgen 作为 ABI 配对制品，VTK 则独立发布。旧 VTK Qt/OpenGL manifest 已替换。
- 2026-09-24：run 360018 提供三平台 SDK 消费和 native CTest 证据。应用运行时分发、真实窗口验收、Linux glibc 基线及 SBOM/provenance 仍未完成；009/010 的 adapter 冒烟已通过，不列为当前待办。
- 2026-10-14：VTK/Dawn、OCCT/Netgen 与 GoogleTest 发布资产按 macOS 13.0 最低部署目标重产并替换；三组共 12 个归档的本地 SHA256 与 sidecar、manifest 一致，归档目录表检查通过。新摘要的消费端 CI 待复验；任务维持 deferred，安装分发与真实引擎集成范围仍未完成。
- 2026-10-25：SDK producer 唯一维护位置迁至 [sdk-packaging](https://github.com/Jack-Lab-at-EIT/sdk-packaging)。VTK run [40123093281](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40123093281) 三平台生产、自检、打包和 Release 发布成功；OCCT/Netgen、GoogleTest 资产同样发布。下载 sidecar 与 Release API digest 对照通过，Panta manifest 已切换并完成本机消费验证。
- 2026-09-25：Windows 本地供给两处修复：LLVM NSIS 安装器经 `__COMPAT_LAYER=RunAsInvoker` 免提权运行（登记于 042）；Qt 供给（`qt-provision.cmake`）官方源失败时自动回退清华 TUNA 镜像，`PANTA_QT_MIRROR` 可置其他同构镜像根优先，来源切换不改 SHA256 强校验。按 IP/地域自动选镜像被否决：`download.qt.io` 自身已按 IP 做 geo-CDN（本机被解析到 JAIST 仍中断），IP 猜测不可靠且掩盖实际失败信号；回退触发在真实下载失败上，行为可在 configure 输出观察。TUNA 路径同构性已 HEAD 实测（qtdeclarative 字节数与官方一致）。

## 完成摘要

仍为 deferred，不能整体验收完成。SDK producer 外部迁移与消费 manifest 更新已在本分支完成；剩余项是桌面产品运行时分发与真实窗口验收、Linux glibc 有效基线，以及消费侧 SBOM/provenance 核验。009/010 adapter 冒烟已完成。

## 2026-10-02 优先级跟进

接续 P0 任务 007 的本机窗口证据，复查 SDK 消费与安装分发边界。当前 `native/app/CMakeLists.txt` 的 install 仅安装应用 executable；Windows 裸启运行库部署服务于构建目录，不等同于可搬迁安装包。下一步先在临时 install prefix 验证运行库解析与许可证布局，并对照 imported targets，定位后补齐安装与回归；开发构建成功和 SDK 内含许可证不作为安装分发闭环证据。031 保持 deferred；SBOM / provenance 在外部生产链生成，038 负责确认交接和供给，031 跟踪消费侧核验。

### 本轮安装复现

使用 `target/native/debug/CMakeCache.txt` 中固定 CMake 4.4.3 运行 `cmake --install target/native/debug --prefix /private/tmp/panta-install-031`，退出 0。安装树仅有 `bin/panta-native`、foundation 静态库/头和 CMake config；没有 Qt/VTK 动态库或许可证目录。`otool -L` 显示 executable 依赖 `@rpath/QtQuickControls2.framework/...` 等；`otool -l` 未发现 LC_RPATH。直接运行安装产物在进入应用前由 dyld 终止（退出 134）：`Library not loaded: @rpath/QtQuickControls2.framework/Versions/A/QtQuickControls2`，`Reason: no LC_RPATH's found`。日志 `/tmp/panta-031-install.log` 与 `/tmp/panta-031-installed-launch.log`。这是安装分发缺陷的复现，区别于构建目录原生窗口已通过；不以设置指向开发缓存的绝对 RPATH 作为可搬迁分发修复。

后续实现需同时处理平台运行库/Qt 插件部署、相对加载路径与 SDK 许可证安装，并验证安装树搬迁后启动；先完成 macOS 当前环境，Windows 与 Wayland 证据分别记录，不改动开发裸启路径。

### 延后决定

2026-10-02：Yuki 确认安装分发属于后续打包工作，要求延后，031 改为 deferred。停止本轮安装部署测试并撤回未提交的 bundle / CMake 部署代码；保留已提交的安装失败复现，恢复时继续定位与验证。当前开发构建与裸启路径保持原实现，未产生可发布安装包。
