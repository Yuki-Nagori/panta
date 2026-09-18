# 041 — 构建产物归一与第三方缓存共享

- 状态：done
- 阶段：验证基础
- 依赖：[004](004-cargo-native-orchestration.md)（已完成）、[020](020-toolchain-provisioning.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-17 / 2026-09-18

## 目标与背景

004 之后存在两类 native 构建树：Cargo 统一入口写在 `target/…/OUT_DIR/native-build`，CMake presets 写在 `native/build/<preset>`；两处各存一份产物。更重的问题：launcher 构建脚本目录名带内容哈希，build.rs 每次变更都生成新 OUT_DIR，Qt staging（约 1.4GB）在新目录里完整重新下载——2026-09-17 实测 `target` 下残留 13 个 launcher 目录、约 13.6GB 重复 Qt 副本，`native/build` 另占 1.6GB。

本任务把产物与第三方缓存归一到根 `target/` 下：

1. native 构建树固定为 `target/native/<profile>`，Cargo 与 presets 共用同一目录（presets 的 `binaryDir` 从 `native/build/<preset>` 重定向）；
2. Qt staging 与 googletest FetchContent 提升为跨目录共享缓存 `target/panta-deps/`，build.rs 每次变更不再触发 Qt 重下；
3. 清理 `native/build` 与残留 launcher 哈希目录，`native/` 源码树不再含产物。

## 必读

- [规范：cmake](../standards/cmake.md)
- [规范：cargo](../standards/cargo.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [规范：dependency-acquisition](../standards/dependency-acquisition.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 范围与非目标

范围：build.rs 的 CMake 二进制目录与缓存变量注入、CMakePresets `binaryDir` 重定向、compile_commands 经 `.clangd` 指向构建树、磁盘清理、受影响文档同步。

非目标：不改变 004/040 的构建图（Cargo → build.rs → CMake 单向）、不引入多 profile 并发构建支持、不改 FFI/CMake target 结构；CI 缓存策略由 012 负责，当前只缓存可验证依赖资产。

## 前置条件与待决策

- `QT_PROVISION_DIR` 已是 qt-provision.cmake 的显式缓存变量（未定义时才回落构建树）；`FETCHCONTENT_BASE_DIR` 是 CMake 内建变量，native/CMakeLists 的 googletest 拉取无需改动即可被重定向。
- 共享树的风险与对策：presets 单独 configure 时缺少 Cargo 注入的 `PANTA_FFI_*` 缓存值——FFI 早校验会给出明确错误（维持现状语义，presets 供 native-only 调试与 cargo 之后的复用）；切换 CMake generator 需删除对应 `target/native/<profile>` 重配（CMake 自身给出清晰诊断）。

## 实施步骤

1. build.rs：二进制目录改为 `target_root/native/<profile>`；向 configure 注入 `-DQT_PROVISION_DIR=<target>/panta-deps/qt` 与 `-DFETCHCONTENT_BASE_DIR=<target>/panta-deps/fetchcontent`。
2. CMakePresets：`binaryDir` 改为 `${sourceDir}/../target/native/${presetName}`。
3. 根 `.clangd` 指向 `target/native/debug/compile_commands.json`：CMAKE_EXPORT_COMPILE_COMMANDS 的产物只存在于共享构建树，编辑器索引经配置文件定位，源码树不落任何副本。
4. 清理磁盘：删除 `native/build` 与残留 launcher 哈希目录（保留当前活跃目录）。
5. 验证：cargo 双跑幂等、build.rs 变更（哈希变化）后 Qt 不重下、presets 与 Cargo 共用同一树、ctest 全绿；同步文档。

## 预计改动

`crates/launcher/build.rs`、`native/CMakePresets.json`、`native/cmake/qt-provision.cmake`（注释）、`ai-docs/architecture/build-and-development.md`、`ai-docs/standards/dependency-acquisition.md`、task-index 与本文件。产物目录不入库（.gitignore 已覆盖 `/target/`、`/native/build/`）。

## 清理与兼容例外

删除 `native/build/` 下的旧 preset 产物与残留 launcher 哈希目录（均为未跟踪缓存）；presets 的旧 binaryDir 路径是本次变更对象，无兼容层。

## 验收标准

- [x] `cargo build --locked` 与 `cmake --preset debug`（/release）使用同一 `target/native/<profile>` 构建树；`native/` 下不再产生构建产物。
- [x] build.rs 变更（launcher 哈希目录更替）后，Qt staging 与 googletest 不重新下载，共享缓存 `target/panta-deps/` 只保留一份。
- [x] 共享树上 `ctest` 全绿；干净 PATH 场景（020）不受影响。
- [x] 磁盘清理完成：`native/build` 删除、残留哈希目录清空；文档中的产物路径描述同步。
- [x] task、索引与实际行为一致；无未登记兼容代码。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 现状测量：`du -sh native/build target`；launcher 哈希目录清点 | `native/build` 1.6G；`target` 15G，其中 13 个 launcher 目录含重复 Qt staging 约 13.6G（最大 10 个各 1.4G） |
| 2026-09-17 | `cargo build --locked`（macOS arm64，正常 PATH） | 共享树 `target/native/debug` 首次 configure 成功；Qt 单份下载至 `target/panta-deps/qt`（1.4G），googletest 至 `panta-deps/fetchcontent`（25M）；CMakeCache 中 `QT_PROVISION_DIR`/`FETCHCONTENT_BASE_DIR` 指向共享缓存 |
| 2026-09-17 | `touch crates/launcher/build.rs`（强制 launcher 哈希更替）后重建 | 新 OUT_DIR 目录生成，Qt 字节数前后一致（2843272KB，未重下），共享树复用（构建 1.44s），构建树内无 qt/ 副本 |
| 2026-09-17 | `cmake --preset debug` + `ctest --preset debug`（native/ 下，系统 cmake） | preset 二进制目录即 `target/native/debug`（configure 0.3s 增量）；ctest 15/15 全绿 |
| 2026-09-17 | 干净 PATH 冒烟（020 场景回归）：`PATH=<cargo 符号链接>:/usr/bin:/bin:/usr/sbin:/sbin cargo build` | 通过：托管工具（panta-tools）+ 共享树 + 共享缓存全链 9.27s；随后常规 PATH 幂等 0.08s |
| 2026-09-17 | 磁盘清理：`rm -rf native/build`；删除当前活跃之外的全部 launcher 哈希目录 | `target` 15G → 4.4G（剩余：Rust 缓存 + 单份 Qt 1.4G + 工具 363M + 共享树 54M）；`native/` 源码树无产物 |
| 2026-09-17 | `cargo clippy --locked --workspace --all-targets -- -D warnings`；`cargo fmt --all -- --check`；`git diff --check` | 通过 |
| 2026-09-17 | 三平台 CI 复跑（push 97b7d32，run [35226508625](https://github.com/Yuki-Nagori/panta/actions/runs/35226508625)） | windows-2022 / macos-latest / ubuntu-latest 全绿（7m41s / 5m58s / 5m14s），共享树与缓存路径变更在 CI 干净环境同样成立 |
| 2026-09-18 | Cargo 驱动完整工具链复核：`cargo build --locked --workspace`、`cargo run --locked --package panta-tests -- toolchain` | 历史记录修正：当时验证的是托管 CMake/Ninja、Qt、GoogleTest 与 native 数据库；LLVM 22.1.7 于 2026-09-19 接入，本行不能证明新版 LLVM 已通过 |

## 风险与回退

共享树引入跨入口耦合：presets 单独构建会沿用 Cargo 上次 configure 的 FFI 缓存值（非目标场景下可删除对应 profile 目录重配）；并发同 profile 构建本就不受支持。回退仅撤销本任务自身的路径变更，恢复 presets 旧 binaryDir 与 OUT_DIR 构建树，并保留文档旧描述的 git 历史。

## 决策与工作记录

- 2026-09-17：根据维护者反馈（两份产物、仓库整洁）创建任务。选择"presets 重定向 + Cargo/presets 共树"而非"删除 presets"：presets 保留 native-only 调试入口，且 039/041 的本机验证依赖它；共享依赖缓存是磁盘问题的根因修复，清目录只是止血。
- 2026-09-17（实施）：build.rs 将 binary_dir 改为 `target_root/native/<profile>` 并注入 QT_PROVISION_DIR/FETCHCONTENT_BASE_DIR；presets binaryDir 重定向；新增根 `.clangd`（compile_commands 随共享树，编辑器经配置定位）。哈希更替不重下 Qt 的验证通过。
- 2026-09-17（收尾）：三平台 CI 复跑全绿（run 35226508625），验收关闭；presets fresh configure 需先 `cargo build` 注入 FFI 缓存值、切换 generator 需删 profile 目录、同 profile 并发构建不支持这三个边界保持记录在风险节。

## 完成摘要

产物与缓存归一完成并经三平台 CI 复跑验证（run 35226508625）：native 构建树固定 `target/native/<profile>`（Cargo 与 presets 共用），Qt/googletest 缓存共享于 `target/panta-deps/`，compile_commands.json 只存在于构建树内并经根 `.clangd` 提供给编辑器。磁盘效果：`target` 15G → 4.4G，`native/build`（1.6G）删除，launcher 哈希更替不再重下 Qt。当前受支持平台默认强制使用 Cargo 托管的 LLVM 22.1.7、CMake/Ninja，CI 在 build 后运行 `panta-tests toolchain` 校验完整工具链路径；`PANTA_USE_SYSTEM_TOOLS=1` 仅作为不支持固定资产平台的显式旁路。已知边界：presets 单独 fresh configure 缺 FFI 缓存值会被早校验明确拒绝（需先 `cargo build` 一次）；切换 generator 需删除对应 profile 目录；同 profile 并发构建不受支持（与之前一致）。

2026-09-18 复查发现 CMake 默认缓存会把 `CMAKE_EXPORT_COMPILE_COMMANDS` 留为空，导致 VS Code 即使指向正确构建树也找不到数据库；`native/cmake/build-policy.cmake` 现以 `CACHE BOOL ... FORCE` 确保导出开启。`cargo build` 已验证 `target/native/debug/compile_commands.json` 生成，`.vscode/settings.json` 与 `.clangd` 统一指向该路径，不保留源码树副本。

## 2026-09-19 质量入口联动

042/043 将工具缓存细分为 `target/panta-tools/<tool>/<version-sha256>/`，共享安装加锁并在成功后发布。普通 native 构建仍在 `target/native/<profile>`；覆盖率使用独立的 `target/native/debug-coverage`，Rust coverage 通过 `PANTA_TOOL_CACHE_ROOT` 复用根 target 工具资产，不重复安装 LLVM。Cargo 合并的 CMake/CXX 自有代码数据库位于 `target/native/<profile>/quality/compile_commands.json`，`.clangd` 与 VS Code 同步使用；原始 CMake 数据库仍由构建图管理。当前托管 LLVM 的 macOS 构建、Debug/Release 测试和实际工具链核验已通过；Linux/Windows 新链路由 042/043 待验收，不覆盖本任务早期路径归一的历史 CI 结论。
