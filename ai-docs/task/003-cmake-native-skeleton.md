# 003 — CMake/Ninja 原生构建骨架

- 状态：done
- 阶段：M0
- 依赖：[002](002-dependency-baseline.md)（已完成）
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

建立有安装和测试入口的自有 C++20 target，明确 native 构建目录与配置。

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

依赖 002 已完成。实施前待决策项：单配置 vs Multi-Config、cmake_minimum_required、最小 target 的命名与归属、presets 路径——均已写入决策记录。直接诊断构建使用本机已有 cmake 4.3.3 + Ninja 1.13.2（[依赖获取](../standards/dependency-acquisition.md)：本机同名包仅作诊断便利，项目基线由 004 的托管引导供给 4.4.3）。

## 实施步骤

1. 创建 native 顶层 CMake、最小可编译 target 与测试，选择单配置 Ninja 或 Multi-Config 并记录原因。
2. 为自有 targets 设置 C++20、标准必需、禁用扩展及局部告警；建立格式配置。
3. 建立共享 presets 和本机配置入口，显式定义 binary/install 路径，避免硬编码机器依赖前缀。
4. 增加 CTest 注册和安装规则，用最小 native 行为验证构建链；此时无需 Qt。

## 预计改动

native/CMakeLists.txt、最小 native target/test、CMakePresets.json、.clang-format。实际改动：`native/{CMakeLists.txt,CMakePresets.json,cmake/panta-native-config.cmake.in}`、`native/foundation/{CMakeLists.txt,include/panta/foundation/version.hpp,src/version.cpp,tests/version_test.cpp}`、根 `.clang-format`、`.vscode/settings.json`（编辑器自动生成的绝对路径改为 `${workspaceFolder}` 相对引用，见决策记录）。

## 清理与兼容例外

无废弃项（此前无 native 构建配置）。

## 验收标准

- [x] 干净 configure/build/ctest/install 均成功，记录实际 preset 名与命令。
- [x] 无改动构建不重复编译，修改头/源可正确重建。
- [x] Debug/Release 或相应配置目录隔离；缺失工具/依赖诊断可定位。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

以下命令均在 cwd=`native/`、macOS 26.3.1 arm64、Apple clang 17.0.0、cmake 4.3.3 + Ninja 1.13.2（本机诊断用）执行：

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | `cmake --preset debug`（干净目录） | configure 成功，构建目录 `native/build/debug`（gitignored） |
| 2026-09-16 | `cmake --build --preset debug` → 再跑一次 | 首次 4 步全量编译链接；第二次 `ninja: no work to do`，无重复编译 |
| 2026-09-16 | `ctest --preset debug` | `panta.foundation.version` Passed（1/1） |
| 2026-09-16 | `cmake --install build/debug`；`find build/debug/install -type f` | 安装树完整：include/panta/foundation/version.hpp、lib/libpanta_foundation.a、lib/cmake/panta-native/*（config + version + targets） |
| 2026-09-16 | release preset 全流程；与 debug 目录对照 | `build/release` 独立 configure/build/test/install 成功，两配置互不污染 |
| 2026-09-16 | `touch version.hpp` 后重建；`touch version.cpp` 后重建 | 改头触发 2 个 TU 重编（lib+test）；改源仅 1 个 TU + 链接；随后回到 no-op |
| 2026-09-16 | 缺失工具诊断：`env PATH=/usr/bin:/bin cmake -S . -B … -G Ninja` | 报错直达缺失项："unable to find a build program corresponding to Ninja / CMAKE_MAKE_PROGRAM is not set" |
| 2026-09-16 | `clang-format --dry-run -Werror`（CLT 17.0.0） | 首轮发现缩进不符（已用 `clang-format -i` 统一为项目格式），复检通过 |
| 2026-09-16 | 安装树消费冒烟：/tmp 独立工程 `find_package(panta-native CONFIG REQUIRED)` 链接 `panta::foundation` | configure/build/run 全通过（首轮暴露导出名为 `panta::panta_foundation`，经 `EXPORT_NAME foundation` 修正后重验通过） |
| 2026-09-16 | presets 负样本（实施中发现） | `installPresets` 非 presets schema 合法字段（初稿误用，报 Invalid extra field）；改为显式 `cmake --install build/<preset>`，schema 降为实际需要的 v3 |

未覆盖：MSVC 分支（/W4 /permissive-）无 Windows 本地验证，待 012 的 runner；ctest 失败路径未单独演示（无业务失败注入点）；clang-format 未纳入自动门禁（011 决定）。

## 风险与回退

全局编译配置容易污染第三方 target，preset 与 CMake 版本也可能不匹配；已将配置限制到自有 target（告警 PRIVATE、C++20 经 PUBLIC compile features 传播），`cmake_minimum_required 3.22` 覆盖 presets schema 3 所需。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-16（实施）单配置 Ninja（非 Multi-Config）：配置由 preset 的 `CMAKE_BUILD_TYPE` 固定，Debug/Release 以 `build/<preset>` 独立目录隔离；路径确定、增量最快，004 的 launcher 定位无需 `$<CONFIG>` 展开。已同步 ninja.md。
- 2026-09-16（实施）`cmake_minimum_required(VERSION 3.22)`：本树实际用到的 presets schema 3（build/test presets）所需最低版本；初稿写 3.25/schema 6 属无依据抬高，已纠正。CMakePresets 以 `${sourceDir}/build/${presetName}` 与 `${sourceDir}/build/${presetName}/install` 定义 binary/install 路径，无个人绝对路径；本机覆盖走 CMakeUserPresets.json（gitignored）。
- 2026-09-16（实施）最小 target 为 `native/foundation`（`panta_foundation`，导出名 `panta::foundation`）：编译链验证 + native 版本契约（`native_version()` 与根 Cargo.toml 版本一致性，004 校验依赖此约定）。不预建 geometry/mesh 等占位目录。版本单一来源是顶层 `project(VERSION 0.1.0)`，经编译定义注入实现与测试。
- 2026-09-16（实施）根 `.clang-format`（LLVM 基、4 空格、100 列、C++20）；检查命令记入 cpp.md。告警基线 -Wall -Wextra -Wpedantic（MSVC /W4 /permissive-），仅自有 target。
- 2026-09-16（范围外修正）`.vscode/settings.json` 被编辑器写成个人绝对路径，按维护者要求改为 `${workspaceFolder}/native` 共享引用（repository-hygiene 允许提交可移植 .vscode 设置）。
- 2026-09-16（实施）实施中修正两处自身错误：installPresets 非法字段（见验证表）；PUBLIC include 的源码树绝对路径不可导出，改 `$<BUILD_INTERFACE>`/`$<INSTALL_INTERFACE>` 双接口。

## 完成摘要

已交付 native 构建骨架：顶层 `native/CMakeLists.txt`（3.22）+ 单配置 Ninja presets（debug/release，独立目录隔离与安装前缀）+ `panta_foundation` C++20 静态库（导出为 `panta::foundation`，含版本契约）+ CTest 测试 + 根 `.clang-format`。干净构建、无改动不重编、头/源重建触发、双配置隔离、缺失工具诊断、安装树消费冒烟全部通过（本机 cmake 4.3.3 + Ninja 1.13.2，诊断用途）。剩余限制：MSVC 分支未实测（待 012 runner）；托管引导未接（004）；尚无第三方依赖。后续：004（Cargo 调度 CMake 与运行入口）已就绪。
