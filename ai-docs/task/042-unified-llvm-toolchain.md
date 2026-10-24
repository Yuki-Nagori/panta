# 042 — 三平台自有 C++ 统一 LLVM/Clang 工具链

- 状态：done
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[020](020-toolchain-provisioning.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-18 / 2026-10-25

## 目标

让 macOS arm64、Linux x86_64、Windows x64 上的自有 C++ 构建和 LLVM 质量工具使用同一固定 LLVM 版本，避免各平台默认编译器及工具版本漂移。统一编译器来源和版本，不统一平台 ABI、系统 SDK 或运行库。

## 范围

- CMake native 与 Cargo CXX（`cxx-build`）使用托管 LLVM 22.1.7；macOS/Linux 使用 `clang`/`clang++`，Windows 使用 `clang-cl`。
- `clang-format`、`clang-tidy` 和 native coverage 工具使用同一 LLVM 发行包。Rust 编译器及 Rust coverage 工具继续由 Rust 工具链提供。
- 编译器、构建工具和依赖资产统一安装到仓库根 `.toolchain/`；构建树及临时产物仍位于 `target/`。显式系统工具旁路只由 `PANTA_USE_SYSTEM_TOOLS=1` 启用。
- 通过现有三平台构建、测试及工具链核验验证当前 Qt、GoogleTest 和 native SDK 消费路径。

## 不在本任务范围

- 重产或发布 VTK、OCCT、Netgen 等第三方 SDK；生产由 [sdk-packaging](https://github.com/Jack-Lab-at-EIT/sdk-packaging) 负责，Panta 的管线迁移和消费清单切换见 [038](038-native-sdk-artifact-production.md)。应用消费已发布 SDK 的构建测试仍属于本任务验证范围。
- Rust 覆盖率门槛及跨语言质量门禁；由 [032](032-cross-language-quality-gates.md) 决定。本任务只保证 native coverage 使用对应 LLVM 的工具。
- 穷举所有 Debug/Release、系统 SDK、ABI 和第三方库版本组合。验证以仓库当前支持配置的真实构建和测试为准。
- 宣称 sanitizer 在各平台能力等价。支持组合与平台差异以 `tests/src/sanitize.rs`、`native/cmake/build-policy.cmake` 及其 CI 结果为准。

## 当前实现

- `crates/panta-build/src/assets.rs` 固定 LLVM 版本及平台资产；供给代码校验摘要并安装到 `.toolchain/tools/`。
- `crates/panta-build/src/compiler.rs` 为 Cargo CXX、CMake 和质量工具解析共享 LLVM。`native/cmake/build-policy.cmake` 检查托管工具链版本；macOS 的标准库头和运行库仍取自 Apple SDK，Windows 保留 MSVC ABI、CRT/STL 与 Windows SDK。
- `tests/src/toolchain.rs` 的 `panta-tests toolchain` 核对工具链路径、CMake 配置及实际 native 编译器信息。PR CI 执行受影响范围检查；nightly 执行三平台完整测试、平台工具链核验及 sanitizer 矩阵。
- Windows ASan 链接边界及 Netgen ISA 基线曾在 [049](049-ci-mesh-sigill-windows-asan.md) 单独修复并验收；其历史诊断不在本任务重复维护。

### Sanitizer 支持范围

| 平台 | Nightly 组合 |
|---|---|
| Linux x86_64、macOS arm64 | ASan+UBSan；独立 TSan |
| Windows x64 | ASan |

TSan 不支持 Windows，clang-cl 的 UBSan 能力不满足本仓库完整矩阵；TSan 不能与其他 sanitizer 组合。插桩范围是自有 native CMake C++，不包含 Rust、Cargo CXX 桥接和预编译 Qt/SDK。Windows ASan 不运行 QML 测试；TSan 按 runner 中的精确过滤排除无法观察未插桩 Rust/Qt 同步的测试，其余常规测试仍运行。以 [Clang ASan](https://clang.llvm.org/docs/AddressSanitizer.html)、[UBSan](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html)、[TSan 支持平台](https://clang.llvm.org/docs/ThreadSanitizer.html#supported-platforms)文档及 runner 实际过滤为准。

## 验收结果

- [x] 任务 130 的 `.toolchain/` 布局合并后，三平台 nightly 完整测试及 `panta-tests toolchain` 通过；三平台均解析到固定 LLVM，实际构建链可用。
- [x] 同一 main commit 的 nightly sanitizer job 通过仓库已登记的平台/组合。
- [x] 按验收结果更新本任务与索引；没有触发需要扩展范围的平台构建或 ABI 失败。

不再要求单独穷举 clean/incremental × Debug/Release 矩阵，也不以 SDK 生产链、覆盖率阈值或未接入的第三方库组合阻塞本任务。

## 验证记录

| 日期 / 证据 | 结果 |
|---|---|
| 2026-09-24：Actions [run 36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4` | macOS/Linux/Windows 构建与测试成功（Linux CTest 56/56、macOS ASan/UBSan 56/56、Windows CTest 54/54）。该运行早于 `.toolchain/` 迁移，不能作为迁移后的三平台验收。 |
| 2026-10-25：PR [#88](https://github.com/Yuki-Nagori/panta/pull/88) Actions run `40123160060` | Ubuntu PR 影响范围 CI 全部通过；冷缓存构建与测试成功。该运行没有覆盖 Windows/macOS。 |
| 2026-10-25：本地 `cargo clean`、`cargo build --locked`、`cargo run --locked --package panta-tests -- toolchain` | `target/` 清理后重建成功；CMake、Ninja、LLVM、Qt 与 GoogleTest 均从 `.toolchain/` 解析。 |
| 2026-10-25：main commit `ae4f7c5` nightly [run 40125142431](https://github.com/Yuki-Nagori/panta/actions/runs/40125142431) | macOS/Linux/Windows workspace tests 与 `panta-tests toolchain` 全部通过；三平台 sanitizer 全部通过（Windows ASan、macOS/Linux ASan+UBSan 与 TSan），Ubuntu 全量质量检查及 macOS Objective-C++ 检查通过。这是 `.toolchain/` 迁移后的验收证据；其他 nightly job 不属于本任务门槛。 |
| 2026-10-25：独立只读 final review（`gpt-6.1-sol`, medium） | 检查任务范围、依赖、实现/CI 对齐及 sanitizer 矩阵；补回精简矩阵和插桩/过滤边界后复查通过。review 本身未重跑测试；nightly 证据见上行。 |

## 决策

- 2026-09-18：维护者要求三平台自有 C++ 统一 LLVM；锁定 LLVM 22.1.7，Windows 继续使用 clang-cl/MSVC ABI。
- 2026-09-21：sanitizer 只按各平台实际支持矩阵验证；Rust 与 C++ coverage 工具分别配套各自编译器工具链。
- 2026-10-25：任务 130 将已安装工具和依赖迁至 `.toolchain/`，并将 Windows/macOS 完整测试移至 nightly。042 只保留迁移后尚缺的三平台证据；SDK 生产与质量门槛不再作为依赖或阻塞项。
- 2026-10-25：main commit `ae4f7c5` 的 nightly run `40125142431` 完成三平台 workspace/toolchain 验收与 sanitizer 矩阵，所有 042 验收项通过，状态改为 done。

## 完成摘要

三平台统一 LLVM 22.1.7 的供给、CMake/Cargo CXX 编译器选择和 LLVM 质量工具闭环已完成。迁移至 `.toolchain/` 后，main nightly 的三平台完整测试、实际工具链核验和登记的 sanitizer 矩阵全部通过。
