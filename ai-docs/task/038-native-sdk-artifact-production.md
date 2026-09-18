# 038 — Native SDK 制品生产与发布

- 状态：in-progress
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

先在隔离 runner 运行依赖包自检和哈希复现，再在三平台清除缓存执行 configure/package smoke；以 031、007、009、010 的真实集成命令作为最终证据。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | `git ls-remote --tags https://gitlab.kitware.com/vtk/vtk.git "*9.7.0*"` | `v9.7.0` 为 annotated tag（对象 `a78e2d95…`），解引用 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`（"Update version number to 9.7.0"）——规范原 pin `23f0a095621e` 为其前缀，无误 |
| 2026-09-17 | `cmake -S tools/sdk -B target/sdk-production/superbuild -G Ninja -DQT_PROVISION_DIR=<共享 Qt 缓存>` → `cmake --build`（macOS arm64，Apple Silicon 全并行） | 全程约 13 分钟：clone ≈2min、configure ≈1min、编译+安装 ≈10min。安装树 `out/vtk/9.7.0/macos-arm64/`（295MB）：bin/include/lib/share/licenses + `panta-sdk.json`；Qt 模块 dylib 齐备（GUISupportQt/GUISupportQtQuick/GUISupportQtSQL/RenderingQt/ViewsQt）。**构建级证实 VTK 9.7.0 × Qt 6.11.2 可产出 GUISupportQtQuick**（窗口运行时行为仍归 007） |
| 2026-09-17 | 自检 `cmake -S tools/sdk/selfcheck -B … -DPSDK_ROOT=<安装树> -DPSDK_PACKAGE=VTK -DPSDK_REQUIRED_TARGETS="VTK::GUISupportQtQuick VTK::RenderingQt" -DCMAKE_PREFIX_PATH=<Qt staging>` | 通过。两轮教训记档：`cmake -P` 脚本模式无法执行 `add_library(IMPORTED)`，VTK config 加载半途而断且脚本仍退出 0（假阳性）——自检必须是真实 configure 工程；VTK config 会调用 FindThreads 等编译探测，工程需启用 CXX。必需 target 以 `VTK::` 命名空间断言 |
| 2026-09-17 | 打包 `cmake -DPKG_ROOT=<安装树> … -P tools/sdk/package.cmake` | `vtk-9.7.0-macos-arm64.tar.gz`（平铺布局，56MB），SHA256 `0cc143dc6545d96f25d537b4ee31f76f4d6e3cc7dfb147bc205c7fdd1e1b6a0f`，与 `.sha256` 文件一致 |
| 2026-09-17 | 增量二本地回归：vtk.cmake 去除子工程硬编码 Ninja（继承外层生成器）后 `cmake -S tools/sdk …` 重配 + `cmake --build`；`python3 -c "yaml.safe_load(...)"` 校验 `sdk-vtk.yml` 与 `ci.yml` | 重配/幂等构建通过（已产出的 stamp 不重编）；两个 workflow YAML 解析通过。dispatch 级验证（三平台真实生产/发布）待推送后执行 |
| 2026-09-18 | 用户 dispatch 实测：workflow run [35242622228](https://github.com/Yuki-Nagori/panta/actions/runs/35242622228)（三平台，1h58m） | 三平台 success；Release [sdk-vtk-9.7.0](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-vtk-9.7.0) 落地：三平台 tar.gz（56-76MB）+ `.sha256` 共 6 资产，发布说明来自合规模板 |
| 2026-09-18 | 闭环验证（031 侧）：manifest 按发布资产登记后，macOS 生产 consumer 从 Release 真实下载消费，`find_package(VTK CONFIG)` + required-target 自检通过，离线二跑零下载；ctest 27/27 | 通过。VTK 从生产到消费全链路闭环；剩余：OCCT/Netgen 构建描述与制品、SBOM/provenance 自动化、007/009/010 的真实链接/运行冒烟 |

## 风险与回退

构建器差异、许可证遗漏、Netgen/OCCT ABI 不匹配或 VTK QtQuick 模块缺失会产生不可用 SDK。发布前 package 自检失败即阻止上传；升级保留上一 manifest 和归档，供给脚本只切换到完整校验通过的版本。

## 决策与工作记录

- 2026-09-17：由 031 官方资产盘点拆分；上游缺少全平台 C++ SDK 时，采用受信 CI 生成一次、开发者复用的制品路线。任务不授权普通本地构建源码。
- 2026-09-17（增量一，已实施）：建立可审计构建描述 `tools/sdk/`——CMake superbuild（ExternalProject）+ 自检/打包脚本，首个目标 VTK 9.7.0。源码固定 tag `v9.7.0` 解引用 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`（git ls-remote 复核）；Qt 依赖复用 qt-provision 锁定预编译 staging；构建开关冻结：共享库、Release、`VTK_QT_VERSION=6`、`VTK_GROUP_ENABLE_Qt=YES`、显式 `VTK_MODULE_ENABLE_VTK_GUISupportQtQuick=YES`；安装布局含 `panta-sdk.json` 与 `share/licenses/`。本机（macOS arm64）真实生产一次并自检/打包通过（见验证表），开发机生产仅为构建描述验证与 007 前置证据，不改变"受信 CI 生产、开发者只下载"的目标形态。
- 2026-09-17（增量二，CI 生产管线）：新增 `.github/workflows/sdk-vtk.yml`（workflow_dispatch，三平台矩阵：macos-latest/Ninja、ubuntu-latest/Ninja + GL/X 开发包、windows-2022/VS17-x64——生成器经 CMAKE_GENERATOR 继承进 ExternalProject 子构建，vtk.cmake 不再硬编码 Ninja 并显式 `--config Release` 应对多配置）。流程：qt-provision（actions/cache 加速层，键=qt-provision.cmake 哈希）→ 生产 → selfcheck 真实 configure 消费 → package → upload-artifact；`publish` 输入开启时以 `gh release` 幂等发布到 tag `sdk-vtk-9.7.0`，发布说明模板 `tools/sdk/releases/vtk-9.7.0.md` 承载合规元数据：源码 pin + unmodified 声明、BSD-3 随包许可证、Qt LGPL-3 只链接不复分发、`.sha256` 校验与 panta-sdk.json provenance。工具链取 runner 预装 cmake/ninja（受信生产环境，非开发者基线）。workflow 需推送后 dispatch 验证，当前未执行；031 manifest 的 vtk 三平台登记待首个 Release 落地后按实际 URL/SHA256 回填。剩余：workflow dispatch 实测、OCCT（macOS/Linux）与 Netgen 构建描述、SBOM/provenance 自动化、031 manifest 正式登记。

## 完成摘要

未完成。等待 031 固定清单与 020 工具链供给，然后实施隔离构建、包自检和发布。
