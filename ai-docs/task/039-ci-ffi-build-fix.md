# 039 — CI FFI 构建链修复

- 状态：in-progress
- 阶段：验证基础
- 依赖：[006](006-rust-cpp-boundary.md)、[018](018-cross-platform-ci.md)、[036](036-ci-native-build-fix.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-17 / 2026-09-17

## 目标与背景

GitHub Actions 最新 run `35199280596`（提交 `e9e20ac`）的 Windows Build 在 `panta_ffi_boundary_test` 链接阶段失败。Linux/macOS 的早期干净构建中，Cargo 将 `panta-ffi` staticlib 生成在 profile 根目录，但 `panta-launcher` 构建脚本只从 `target/debug/deps` 读取，因此报告未找到静态库。Windows 还暴露出两类 FFI 链接差异：Rust staticlib 使用 `/MD`、`_ITERATOR_DEBUG_LEVEL=0`，而 CMake Debug/GTest 默认使用 `/MDd`、`_ITERATOR_DEBUG_LEVEL=2`；Rust std 需要的 `ws2_32`、`userenv`、`ntdll` 也未由 CMake 显式链接。

完成后，CI 的三平台干净构建应在 launcher 的 native CMake 阶段前拥有可供链接的 `panta-ffi` staticlib，CXX bridge 在 MSVC 上应使用 `/std:c++20`，并保持 GCC/Clang 的 `-std=c++20` 路径。staticlib 的先后顺序最终由任务 040 收回 Cargo 构建图，不要求 CI 额外记忆预构建命令。

## 必读

- [Rust 编码与工具链](../standards/rust.md)
- [CXX：Rust ↔ C++ 首选桥接方案](../standards/cxx.md)
- [Cargo 与 build script 规范](../standards/cargo.md)
- [三平台 CI 基础](018-cross-platform-ci.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

范围：修复 `panta-ffi` staticlib 在 CI 干净构建中的供给顺序；修正 CXX bridge 的 MSVC C++ 标准选项；对 Windows FFI 边界测试统一 MSVC runtime/迭代器 ABI 并补齐 Rust std 系统库；补充能复现这些环境差异的本地/CI 验证和任务记录。

非目标：不改变 FFI DTO、错误语义、Qt 版本、CMake 生成器矩阵或 native 业务实现；不引入依赖缓存和统一质量入口。

## 前置条件与待决策

GitHub Actions 的 `ubuntu-latest`、`macos-latest` 和 `windows-2022` 均应继续使用当前固定 Rust toolchain 与已有 CMake/Qt 供给。staticlib 的生成必须通过现有 Cargo 工作流完成，不在 launcher build script 中递归调用 Cargo。

## 实施步骤

1. 在隔离 target 目录复现普通 `cargo build --locked` 的 staticlib 缺失行为，并确认 `cargo build -p panta-ffi --locked` 的产物位置。
2. 先用 workflow 的独立 `panta-ffi` 预构建验证竞态根因；随后由任务 040 将相同约束收回 launcher manifest，最终 workflow 只调用 `cargo build --locked`，不在 launcher build script 中递归调用 Cargo。
3. 按目标编译器选择 C++20 选项：MSVC 使用 `/std:c++20`，GCC/Clang 使用 `-std=c++20`。
4. Windows FFI 边界测试使用与 Rust staticlib 一致的 `/MD`、`_ITERATOR_DEBUG_LEVEL=0`，并显式链接 `ws2_32`、`userenv`、`ntdll`。
5. 运行 Rust 格式、测试、Clippy 和可用的 native/CMake 验证；更新索引与本文真实结果。

## 预计改动

`.github/workflows/ci.yml`、`crates/panta-ffi/build.rs`、`crates/launcher/build.rs`、`native/ffi/CMakeLists.txt` 及本文件；任务 040 另调整 launcher manifest 的依赖边，最终 CI 保留单一 `cargo build --locked` 入口，launcher 保留 profile 根目录和 deps/的定位逻辑。

## 清理与兼容例外

删除或替换导致干净构建依赖旧 staticlib 残留的隐式路径假设；无兼容例外，不保留未启用 C++ 标准的 MSVC 回退分支。

## 验收标准

- [ ] 三平台干净 `cargo build --locked` 均能完成 FFI staticlib、native CMake 和 launcher 构建（顺序约束由任务 040 的 Cargo manifest 提供）。
- [ ] MSVC 日志显示 `/std:c++20`，GCC/Clang 继续使用 `-std=c++20`，CXX bridge 不再因嵌套命名空间失败。
- [ ] Windows FFI 边界测试的 MSVC runtime 与 iterator ABI 和 Rust staticlib 一致，且 Rust std 所需 Windows 系统库均可解析。
- [x] staticlib 由当前 Cargo profile 生成并被 native 链接；不依赖工作区历史构建残留，也不在 build script 中递归执行 Cargo。
- [x] 可用范围内的 Rust 测试、格式、Clippy 与 native FFI 测试通过；三平台 CI 完整复跑仍待推送后验证。
- [ ] task、workflow、构建脚本和索引一致，无死代码、失效路径或未登记兼容代码。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-17 | GitHub Actions run `35195983722`，`gh run view --log-failed` | 固定 CI 失败根因 | Linux/macOS 缺少 `target/debug/deps` 中的 `panta-ffi` staticlib；Windows MSVC 缺少 C++20 标准开关 |
| 2026-09-17 | GitHub Actions run `35197347393`，`gh run view --log-failed` | 验证 039 首次修复 | 三平台仍在 launcher build script 中找不到 staticlib；确认 Cargo 并发编译未保证依赖包的 staticlib 在 build script 执行前产出，需在 workflow 显式预构建 |
| 2026-09-17 | GitHub Actions run `35199280596`，Windows Build 失败日志 | 固定 Windows FFI 链接根因 | `panta_ffi.lib` 与 Debug C++ 测试出现 `RuntimeLibrary`/`_ITERATOR_DEBUG_LEVEL` LNK2038；另有 33 个 Rust std Windows 符号未解析，归因于 `/MD`↔`/MDd` ABI 不一致及缺少 `ws2_32`、`userenv`、`ntdll` |
| 2026-09-17 | 隔离 target `CARGO_TARGET_DIR=/private/tmp/panta-ci-target.2KJV8J`，`cargo build --locked` | 确认 Cargo staticlib 实际布局 | `target/debug/libpanta_ffi.a` 生成在 profile 根目录；原 launcher 扫描 `deps/` 无法找到它；Qt 下载因沙箱 DNS 失败，未完成 launcher configure |
| 2026-09-17 | macOS arm64；`cargo fmt --all -- --check`、`cargo test --locked --workspace --exclude panta-launcher`、`cargo clippy --locked --workspace --all-targets --exclude panta-launcher`、`git diff --check` | Rust 侧检查通过 | 全部退出 0；DSL 11 个测试、FFI 4 个测试通过；Clippy 仅报告既有测试/构建脚本 `expect` 警告 |
| 2026-09-17 | macOS arm64；隔离 Ninja CMake configure/build + `ctest --test-dir /private/tmp/panta-native-ci.4sY0dA --output-on-failure`，使用当前 profile staticlib 和已有 Qt/GTest staging | native 构建及 FFI 边界测试通过 | 84 个构建步骤完成，native 7/7 测试通过，`Ffi.RustCppBoundary` death test 通过 |
| 2026-09-17 | macOS arm64；独立 FFI CMake 构建，开启 `PANTA_ENABLE_FFI_TEST`，链接当前 `panta-ffi` staticlib；`ctest -R Ffi.RustCppBoundary` | Windows 链接规则改动不破坏 Unix FFI 边界 | CMake 构建与 `Ffi.RustCppBoundary` 通过；macOS 仍使用 `Threads::Threads`/`CMAKE_DL_LIBS` 路径 |
| — | 修复后 GitHub Actions 三平台 run | Build、Test、Format、Clippy 全部成功 | 待推送后验证 |

## 风险与回退

Cargo crate-type 调度、CMake 链接和 CXX bridge 的编译器选项属于不同阶段；若调整后仍有平台差异，保留原始构建诊断并分别定位，不通过复用旧 target 产物掩盖问题。回退时仅撤销本 task 的构建调度和标准选项改动，保留工作区已有的 FFI 功能。

## 决策与工作记录

- 2026-09-17：根据 GitHub Actions run `35195983722` 创建任务；确认失败分为 staticlib 供给顺序和 MSVC C++ 标准选项两类。
- 2026-09-17（实施）：launcher 优先定位 Cargo profile 根目录 staticlib，并以 deps/作为布局兜底；panta-ffi 按 target 环境向 MSVC 传递 `/std:c++20`，非 MSVC 保持 `-std=c++20`。
- 2026-09-17（收敛）：CI 预构建曾验证为有效但不适合作为长期入口；任务 040 通过 launcher 的普通依赖 + build-dependency 双边表达静态库先后顺序，workflow 恢复为单一 `cargo build --locked`。

## 完成摘要

未完成。代码与本地验证已完成；待推送后取得三平台 CI 新 run，并据此同步最终状态。
