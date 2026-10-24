# 130 — 工具链目录与 PR CI 收敛

- 状态：done
- 阶段：验证基础
- 依赖：[012](012-ci-reproducibility.md)、[020](020-toolchain-provisioning.md)、[041](041-build-artifact-consolidation.md)、[120](120-pr-affected-test-selection.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-10-25 / 2026-10-25

## 目标与背景

目前固定工具、Qt 和 native SDK 安装在 Cargo `target/` 中，清理构建输出会一并删除大型下载；PR CI 还在每次代码变更时运行 Windows、macOS 构建测试和 macOS Objective-C++ 检查。本任务将下载并安装完成的工具链资产放在仓库根 `.toolchain/`，PR 保留 Ubuntu 影响范围构建与检查，Windows/macOS 构建测试及完整 Objective-C++ 检查由 nightly 执行。

Cargo/native 构建树、coverage、性能报告和按需编译工具的临时输出仍写入 `target/`。

## 必读

- [规范：依赖获取](../standards/dependency-acquisition.md)
- [规范：验证与评审](../standards/validation-and-review.md)
- [架构：构建与开发](../architecture/build-and-development.md)
- [规范：Cargo](../standards/cargo.md)

## 范围与非目标

- 将 CMake、Ninja、LLVM、uv、hyperfine、Qt、native SDK 与 Cargo 质量工具等已安装/解包资产统一放在 `.toolchain/tools/` 和 `.toolchain/deps/`。
- 更新应用构建、测试 runner、格式/lint 工具定位、缓存 actions、CMake 配置入口与依赖文档。
- PR `check` 矩阵只保留 Ubuntu；macOS/Windows 全量测试继续由 nightly 三平台矩阵运行，Objective-C++ 完整检查保留在 nightly macOS job。
- 保留 PR 上 Ubuntu 的变更范围测试与适用质量检查。
- 不移动 Cargo registry、Cargo 编译产物、CMake 构建树、coverage 与性能报告。
- 不删除 nightly 的任何平台覆盖。

## 涉及分组与跨组依赖

涉及 Cargo 构建供给、native SDK/CMake、测试 runner、CI 缓存及文档维护；不改变领域依赖方向。

## 前置条件与待决策

- 已有 `.gitignore` 中的 `.toolchain/` 忽略规则需核实并补齐，避免提交下载资产。
- 缓存键继续包含平台、架构、工具版本和供给配置；迁移后首次 CI 缓存冷启动时间需观察。

## 实施步骤

1. 梳理所有工具与依赖安装、消费、验证和缓存路径，确定唯一 `.toolchain` 根解析方式。
2. 迁移代码与 CI cache actions；更新 CMake 供给参数和路径断言。
3. 缩小 PR 平台矩阵并确认 nightly 已承接所有移出检查。
4. 更新依赖获取说明与本任务，执行受影响 L1/L2、格式/lint、CMake/native 检查，并独立评审。

## 预计改动

`crates/panta-build/`、`crates/launcher/build.rs`、`tests/`、`native/cmake/`、`.github/actions/`、`.github/workflows/ci.yml`、`ai-docs/standards/dependency-acquisition.md`、本任务与索引。

## 清理与兼容例外

清理活动代码、CI 配置和现行规范中的旧安装路径引用。任务 020 保留迁移前的历史实现记录；缓存键按新布局版本化，不读取旧目录作为兼容回退。

## 验收标准

- [x] 干净构建将全部已安装工具与依赖放入仓库根 `.toolchain/`，而 CMake/Cargo 构建树及临时输出仍位于 `target/`。
- [x] 工具链核验、native 测试环境、格式/lint 入口与 VS Code Qt/QML 配置均从新路径解析资产；活动代码、CI 配置与现行规范中无过期旧缓存路径。
- [x] PR CI 不再运行 Windows/macOS 平台构建测试或 macOS Objective-C++ 检查；Ubuntu 影响范围验证保持有效。
- [x] nightly 仍覆盖三平台完整测试，以及 macOS Objective-C++ 完整检查；full-tests 继续执行平台工具链路径核验。
- [x] 相关路径单测、cache/action 配置、适用格式/lint 和 L1/L2 测试通过；task 与索引证据同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-25 | `cargo clean`；`cargo build --locked`；`cargo run --locked --package panta-tests -- toolchain` | 清理 Cargo/native 输出后重建，工具链核验指向仓库 `.toolchain/` | 通过；`cargo clean` 清理 target 49.5 GiB 后构建成功，LLVM/CMake/Ninja/Qt/GoogleTest 均从 `.toolchain/` 定位 |
| 2026-10-25 | `cargo verify-changed` | 变更范围格式、lint 与测试通过 | 格式、9 项 lint、Rust workspace 测试以及 90/91 native/QML 测试通过；唯一失败为 `QtAdapter.UserSettingsStore` 的 NativeFormat 用例，沙箱拒绝写入 `~/Library/Preferences` |
| 2026-10-25 | `PANTA_TOOLCHAIN_ROOT=/tmp/panta-toolchain-override cargo test --locked -p panta-build`；`cargo fmt --all --check`；`git diff --check` | 覆盖根设置时安装测试仍隔离，相关代码格式及 diff 干净 | 通过；23 项 panta-build 测试通过；安装器运行时保留覆盖根，测试夹具固定使用独立临时目录 |
| 2026-10-25 | 独立只读评审（`gpt-6.1-sol`, medium）与修复复核 | 检查缓存边界、平台验收承接、活动路径及覆盖根行为 | 4 项发现已修复并复核通过：VS Code Qt 路径、Cargo 质量工具缓存排除、nightly 平台工具链核验、测试夹具隔离 |
| 2026-10-25 | PR [#88](https://github.com/Yuki-Nagori/panta/pull/88) CI（run `40123160060`） | Ubuntu 影响范围质量检查、构建与受影响测试通过 | 全部 CI jobs 通过；冷缓存下构建与测试约 12 分钟，clang-tidy 约 14 分钟；仅有 GitHub runner 的 Node.js 20 与 Ubuntu 26 迁移提示 |

## 风险与回退

路径迁移遗漏会导致干净构建成功但测试、部署或质量工具找不到资产。以全仓检索和 `cargo clean` 后的重建核对每个消费者；回退时恢复原路径和对应 cache key，并重新构建 native 树。本机沙箱不能写入 macOS `~/Library/Preferences`，因此真实 NativeFormat 系统存储用例未能本地通过；其余 native/QML 用例通过，需由 CI 验证该用例。

## 决策与工作记录

- 2026-10-25：创建任务。保留 Ubuntu PR 快速反馈；Windows/macOS 完整平台验收由 nightly 覆盖。
- 2026-10-25：将迁移后的缓存键升版，并让 nightly 在三平台保存对应工具链资产；PR 仍只恢复缓存，避免不同 PR 写入共享缓存。
- 2026-10-25：独立评审发现 VS Code Qt/QML 设置仍指向旧 Qt staging 路径，且通用工具链缓存会包含质量工具的独立缓存；已修正路径并保留 `cargo-*` 排除。nightly 三平台增加工具链路径核验，承接 PR 移出的平台检查。安装测试改用独立临时根，避免 `PANTA_TOOLCHAIN_ROOT` 影响测试隔离，同时保留运行时环境覆盖行为；复核通过。
- 2026-10-25：创建 PR 前删除旧布局 `panta-cache-*` 与 `cargo-quality-tool-v1-*` 缓存，保留 Cargo home 缓存。PR #88 首轮 CI 全绿。

## 完成摘要

工具链资产已迁至仓库根 `.toolchain/`，PR CI 收敛为 Ubuntu 影响范围检查，Windows/macOS 完整测试及 Objective-C++ 检查由 nightly 承接。独立评审与 PR [#88](https://github.com/Yuki-Nagori/panta/pull/88) CI 全部通过；本机 `cargo clean` 后重建及工具链核验通过。

本机沙箱不能写入 `~/Library/Preferences`，因此 `QtAdapter.UserSettingsStore` 的 NativeFormat 系统存储用例未在本机验证。该平台行为由 nightly 三平台测试承接，不阻塞本任务完成；nightly 后续结果按常规 CI 维护处理。
