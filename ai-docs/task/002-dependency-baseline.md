# 002 — 平台、工具链与 native 依赖基线

- 状态：done
- 阶段：M0
- 依赖：[001](001-cargo-config.md)（已完成）
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

选出一个可复现的主验证平台和 native 依赖组合，分清候选版本与运行验证结果。

## 必读

- [规范：baseline](../standards/baseline.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [规范：ninja](../standards/ninja.md)
- [规范：qt](../standards/qt.md)
- [规范：vtk](../standards/vtk.md)
- [规范：occt](../standards/occt.md)
- [规范：netgen](../standards/netgen.md)
- [架构：总览](../architecture/README.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不承诺所有操作系统，不完成产品功能；Python 环境留给 014。

## 前置条件与待决策

依赖 001 已完成。当前决策是依赖获取采用 Cargo 统一托管，并优先消费官方或项目发布的预编译包；本机 Homebrew 安装的 Qt/VTK 等只作个人便利，不进入项目基线。没有匹配预编译包时，源码构建只能由独立制品任务评估，不能作为普通开发机构建步骤。

## 实施步骤

1. 记录 OS、架构、编译器/标准库，决定依赖获取方式和版本固定机制。
2. 为 CMake、Ninja、Qt、VTK、OCCT、Netgen 记录精确版本、来源、链接方式、构建选项和许可证入口。
3. 核对 Qt/VTK 的 Qt Quick 支持，核对 Netgen 与应用的 OCCT 依赖一致性；列出缺失开发包或导出配置。
4. 记录环境参数、干净重建步骤、版本升级与回退方式；在基线表区分待验证项，不提前宣称全部兼容。

## 预计改动

ai-docs/standards/baseline.md、依赖获取配置/说明（路径在执行前确定）。实际改动：新增 [依赖获取与主平台环境](../standards/dependency-acquisition.md)（含固定清单）；更新 baseline.md、规范索引、build-and-development.md、architecture/README.md（修正 001 前的过时现状描述）。未创建任何构建配置文件——托管引导是 003/004 的实施内容，本文只记录契约与清单。

## 清理与兼容例外

无废弃项（此前无依赖配置）。architecture/README.md 中"当前仓库没有应用源码、构建脚本、依赖锁文件和测试"的过时表述已修正。

## 验收标准

- [x] 每个 native 依赖都有版本/来源/模块记录，没有以 latest 或个人绝对路径代替配置。
- [x] 主平台具备编译器、CMake、Ninja；Qt/VTK 和 OCCT/Netgen 的候选组合及后续验证责任清晰。
- [x] 依赖获取步骤可复核，未知的 ABI/运行问题显式转交 007、009、010，不被勾为已验证。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

验收第 2 项的口径：编译器（Apple clang 17.0.0）为主平台必备前置；CMake/Ninja 的基线版本（4.3.3 / 1.13.2）已固定且本机存在同版本安装（Homebrew，仅旁证与 003 直接诊断便利），项目要求的供给方式是 Cargo 引导（004 实装）。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | macOS 26.3.1 arm64；`sw_vers`、`clang++ --version`、`xcrun --show-sdk-version/path`、`perl -v` | clang 17.0.0（CLT）、SDK 26.2、perl 5.34.1（Qt syncqt 可用） |
| 2026-09-16 | 本机 Qt 现状盘点：`qmake6 --version`、`brew list --versions qtbase …` | 本机有 Qt 6.11.1（Homebrew）——仅作版本旁证，不进基线 |
| 2026-09-16 | `brew search netgen`、`brew info --json=v2 vtk/opencascade/ninja` | netgen 无 formula（必须源码构建）；brew VTK 9.7.0 formula 依赖 qtbase+qtdeclarative（上游 Qt 集成存在的旁证）；OCCT 7.9.3、Ninja 1.13.2 |
| 2026-09-16 | GitHub API：上游 release/tag 核实 | OCCT 最新 V8.0.1（2026-07-30），tag `V7_9_3`→commit `a016080bf673`；netgen 最新 `v6.2.2604`→`3ee489c7d58f`；VTK `v9.7.0`→`23f0a095621e`（最新稳定 tag）；Qt 五模块 `v6.11.1` 各 commit SHA 均存在；CMake `v4.3.3` 有 `cmake-4.3.3-macos-universal.tar.gz` 资产 |
| 2026-09-16 | 升级核实（维护者要求尽量最新）：GitHub API 复查 | CMake 4.4.3（2026-08-25，含 macos-universal 资产）、Qt 6.11.2（五模块 SHA 均核实）为上游最新；Ninja 1.13.2、Netgen v6.2.2604、VTK 9.7.0、Rust 1.98.1 已是最新 |
| 2026-09-16 | OCCT 8.0.1 兼容性核实：tag `V8.0.1`→commit `b8f597c67781`；`cmake_minimum_required 3.10`；netgen `occ_utils.hpp` 版本守卫 | OCCT 8.0.1 与 CMake 4.4 兼容；netgen 守卫为 `AT_LEAST` 风格（≥7.8 走 TKDE），无排除 8.x 的证据 → 升级固定 8.0.1，回退点 7.9.3，实测归 010 |
| 2026-09-16 | netgen v6.2.2604 CMakeLists 源文件核对 | `USE_OCC` 默认 ON，`find_package(OpenCascade …)`，含 OCCT ≥7.8 的 TKDE 目标名适配 → 与 OCCT 7.9.3 组合自洽（构建级一致性仍归 010） |
| 2026-09-16 | netgen tag 源码 LICENSE 核对 | LGPL-2.1 |
| 2026-09-16 | `brew install ninja`（本机环境） | 本机获得 ninja 1.13.2；仅个人便利与 003 诊断用，不写入任何项目配置 |

未覆盖：Qt/VTK/OCCT/Netgen 均未下载构建（引导实装归 004，集成验证归 005/007/009/010）；二进制 SHA256 未回填；Qt Quick 运行时、Metal 后端与 VTK 交互未验证。

## 风险与回退

不同来源的 Qt/VTK、OCCT/Netgen 可能使用不兼容的编译选项或 ABI；固定清单保留候选组合与验证状态，不以 pin 一致代替集成验证。回退仅撤销本任务自身变更（新增 runbook 与文档同步），保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-16（实施）依赖获取方式（维护者决策）：**Cargo 统一托管 + 预编译优先**——开发者只需 git、rustup（工具链由 rust-toolchain.toml 固定）与 Apple 命令行工具；CMake/Ninja/Qt/VTK/OCCT/Netgen 由构建引导优先获取固定的预编译包到构建树。否决无锁定的系统包管理器路线：Homebrew 版本浮动、跨机器不可复现，且本机包不能作为团队基线。没有合适预编译包时，另由 031 评估项目制品或明确的源码兜底，不让每个开发者重复本地编译。托管引导的实现（build.rs/超级构建细节）由 004/031 决定，本文只记录基线契约。
- 2026-09-16（实施）主平台：macOS 26.3.1 / arm64 / Apple clang 17.0.0 / libc++ / SDK 26.2。不承诺其他平台。
- 2026-09-16（实施）版本固定（候选）：CMake 4.3.3（官方 macos-universal 二进制 + SHA256 待回填）；Ninja 1.13.2（源码构建）；Qt 6.11.1 五模块源码 tag（SHA 前缀已记录，qt5compat 由 005 裁剪）；VTK 9.7.0；OCCT 7.9.3（V7_9_3）；Netgen v6.2.2604（USE_OCC 默认 ON，与 OCCT ≥7.8 目标名适配核实）。git tag 以 commit SHA 校验。
- 2026-09-16（实施）范围调整：未安装 VTK/OCCT/Netgen；先确认可用的预编译 SDK 与 CMake package，不因本机缺包就默认源码编译。OCCT 保持 7.9.x 而非上游最新 8.0.1：Netgen 未声明支持 8.0，一致性优先。
- 2026-09-16（实施）显式转交：QQuickVTKItem 可用性与图形后端 → 007；OCCT STEP/元数据与模块裁剪 → 009；Netgen/OCCT 组合 ABI 与 C++ 接口 → 010；托管引导实装与首次 configure → 004；CMake 骨架直接诊断 → 003。
- 2026-09-16（补充，维护者要求）：选版原则改为"尽量上游最新 + 核对依赖间版本关系"。清单升级：CMake 4.3.3 → 4.4.3、Qt 6.11.1 → 6.11.2（五模块 SHA 重核）、OCCT 7.9.3 → 8.0.1（依据 netgen 守卫级兼容证据；回退点 V7_9_3 记录于依赖获取文档）；Ninja/Netgen/VTK/Rust 已是最新，不动。OCCT 与 Netgen 的关系从"不升级"改为"升级 + 实测前置 + 回退点"，仍由 010 把关。
- 2026-09-16（补充，维护者要求）：native 第三方库改为预编译优先。VTK/OCCT/Netgen 的具体平台 SDK、SHA256、CMake package 布局和 ABI 匹配由新增任务 031 负责；在 031 完成前，007/009/010 不得把源码构建接入普通开发流程。

## 完成摘要

已交付：主平台记录（macOS arm64 + Apple clang 17）、Cargo 统一托管的依赖获取决策、六项 native 依赖的固定清单（tag/commit SHA/来源/候选选项/依赖间版本关系/许可证入口/验证责任），落地于新增的 [依赖获取与主平台环境](../standards/dependency-acquisition.md)，baseline.md 同步为候选/实测两态。同日按维护者要求升级至各上游最新（CMake 4.4.3、Qt 6.11.2、OCCT 8.0.1；Ninja/Netgen/VTK 已是最新），OCCT 8.0.1 × Netgen v6.2.2604 组合保留 7.9.3 回退点，实测归 010。全部集成都标记为待实测，未宣称兼容。剩余限制：引导未实装（004）、二进制 SHA256 未回填、Qt/VTK/OCCT/Netgen 零构建证据。后续：003（CMake/Ninja 骨架）已就绪；CI 见 018。
