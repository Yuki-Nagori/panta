# 039 — CI FFI 构建链修复

- 状态：done
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

2026-09-17 本轮扩展：根据 run `35202550564` 中 Qt 生成的 `panta_bridgeplugin_init` 仍使用 Debug CRT/iterator ABI 的证据，统一目标创建前的构建策略，覆盖 Qt 自动生成目标和第三方源码目标；整理默认配置、依赖顺序、FFI 输入检查，并更新 `ai-docs/standards/cmake.md`。删除逐目标 ABI 修补；本机验证与 Windows runner 验证分开记录。

范围：修复 `panta-ffi` staticlib 在 CI 干净构建中的供给顺序；修正 CXX bridge 的 MSVC C++ 标准选项；对 Windows native 测试统一 MSVC runtime/迭代器 ABI 并补齐 Rust std 系统库；补充能复现这些环境差异的本地/CI 验证和任务记录。

非目标：不改变 FFI DTO、错误语义、Qt 版本、CMake 生成器矩阵或 native 业务实现；不引入依赖缓存和统一质量入口。

## 前置条件与待决策

GitHub Actions 的 `ubuntu-latest`、`macos-latest` 和 `windows-2022` 均应继续使用当前固定 Rust toolchain 与已有 CMake/Qt 供给。staticlib 的生成必须通过现有 Cargo 工作流完成，不在 launcher build script 中递归调用 Cargo。

## 实施步骤

1. 在隔离 target 目录复现普通 `cargo build --locked` 的 staticlib 缺失行为，并确认 `cargo build -p panta-ffi --locked` 的产物位置。
2. 先用 workflow 的独立 `panta-ffi` 预构建验证竞态根因；随后由任务 040 将相同约束收回 launcher manifest，最终 workflow 只调用 `cargo build --locked`，不在 launcher build script 中递归调用 Cargo。
3. 按目标编译器选择 C++20 选项：MSVC 使用 `/std:c++20`，GCC/Clang 使用 `-std=c++20`。
4. Windows native 测试及 GoogleTest 使用与 Rust staticlib 一致的 `/MD`、`_ITERATOR_DEBUG_LEVEL=0`，并显式链接 FFI 边界测试所需的 `ws2_32`、`userenv`、`ntdll`。
5. 运行 Rust 格式、测试、Clippy 和可用的 native/CMake 验证；更新索引与本文真实结果。

## 预计改动

`.github/workflows/ci.yml`、`crates/panta-ffi/build.rs`、`crates/launcher/build.rs`、native 根/test CMake 配置及本文件；任务 040 另调整 launcher manifest 的依赖边，最终 CI 保留单一 `cargo build --locked` 入口，launcher 保留 profile 根目录和 deps/的定位逻辑。

## 清理与兼容例外

删除或替换导致干净构建依赖旧 staticlib 残留的隐式路径假设；无兼容例外，不保留未启用 C++ 标准的 MSVC 回退分支。

## 验收标准

- [x] 三平台干净 `cargo build --locked` 均能完成 FFI staticlib、native CMake 和 launcher 构建（顺序约束由任务 040 的 Cargo manifest 提供）。
- [x] MSVC 日志显示 `/std:c++20`，GCC/Clang 继续使用 `-std=c++20`，CXX bridge 不再因嵌套命名空间失败。
- [x] Windows FFI 边界测试的 MSVC runtime 与 iterator ABI 和 Rust staticlib 一致，且 Rust std 所需 Windows 系统库均可解析。
- [x] staticlib 由当前 Cargo profile 生成并被 native 链接；不依赖工作区历史构建残留，也不在 build script 中递归执行 Cargo。
- [x] 可用范围内的 Rust 测试、格式、Clippy 与 native FFI 测试通过；三平台 CI 完整复跑已由 run `35203709898`（冷缓存）与 `35204747014`（缓存命中）覆盖。
- [x] task、workflow、构建脚本和索引一致，无死代码、失效路径或未登记兼容代码。

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
| 2026-09-17 | GitHub Actions run `35201746062`，Windows Build | 修复后 native 测试与 GTest 的 MSVC CRT/iterator ABI 一致 | 仍失败：Bridge/Foundation 测试使用 Debug ABI，而 FFI 修复把共享 GTest target 固定为 `/MD`、iterator 0；根因扩大为所有 native target 需统一 ABI，已在本轮移至 native 根配置 |
| 2026-09-17 | GitHub Actions run `35203709898`（`5a6f452`，三平台，Windows 日志确认 `Cache not found` 冷缓存）；run `35204747014`（`ab0a130`，恢复缓存后复跑） | Build、Test、Format、Clippy 全部成功 | 两轮三平台全绿：冷缓存 run 以单一 `cargo build --locked` 完成 Qt staging、native CMake、launcher 全量构建（Windows 5m10s），此前 staticlib 缺失、LNK2038、`ws2_32`/`userenv`/`ntdll` 未解析等失败模式全部消失。成功 run 中 cargo 隐藏 build script 输出，`/std:c++20` 无直接日志行；以行为证据收口——run `35199280596` 因缺少该开关使 CXX bridge 编译失败，补上后 Windows 冷构建成功链接 `panta_ffi_boundary_test`，GCC/Clang 路径不变 |
| 2026-09-17 | run `35202550564`（`8e35810`）Windows Build | 定位剩余生成目标差异 | `panta_bridgeplugin_init.obj` 仍为 `/MDd`、iterator 2，App/Bridge 为 `/MD`、iterator 0；存在 LNK2038 与 LNK4098。确认逐目标 helper 不覆盖 Qt 生成目标 |
| 2026-09-17 | macOS arm64、AppleClang 17；`native/build/debug` 完整 build 后 CTest 与 `all_qmllint` | 保持默认 Shell | 7/7 既有测试及 lint 通过 |
| 2026-09-17 | 新目录 `/private/tmp/panta-cmake-039.k1DDcn/multi`，Ninja Multi-Config；复用已缓存 Qt/GTest，FFI 链接已有 Cargo Debug staticlib；`cmake --build … --config Debug/Release --parallel 4` 后 `ctest --test-dir … -C Debug/Release --output-on-failure` | 双配置构建、运行并验证 ABI 策略回归 | Debug/Release 各 9/9 通过；Debug 再构建无额外工作。Release 的 Rust 库仍为 Debug 产物，本结果不代表完整 Cargo Release 验证；链接器提示已有 Rust 对象最低 macOS 26.2 高于 CMake 26.0 |
| 2026-09-17 | `minimal` 新目录，Ninja、Bridge OFF、BUILD_TESTING OFF，再启用测试并 build/CTest/lint | 默认 Debug、无测试依赖构建及开关切换 | OFF/OFF 完整构建成功；安装前缀默认位于构建树 install；启用测试后 4/4 与 lint 通过 |
| 2026-09-17 | FFI ON + 测试 OFF；FFI ON + 缺少生成头/staticlib | 下载依赖前失败 | 两种 configure 均准确报错，未创建 Qt 下载目录 |
| 2026-09-17 | `Build.MsvcAbiPolicy` | 生成 object target 继承 ABI，拒绝不一致覆盖 | 正例通过；注入 `/MDd` 或 iterator 2 均被审计拒绝。非 Windows 仅检查 CMake 元数据，不替代 MSVC 编译链接 |

## 风险与回退

Cargo crate-type 调度、CMake 链接和 CXX bridge 的编译器选项属于不同阶段；若调整后仍有平台差异，保留原始构建诊断并分别定位，不通过复用旧 target 产物掩盖问题。回退时仅撤销本 task 的构建调度和标准选项改动，保留工作区已有的 FFI 功能。

## 决策与工作记录

- 2026-09-17（整体整理）：新增 `cmake/build-policy.cmake`，在创建依赖/Qt 目标前设置整个 native 图的 CRT/iterator ABI；Windows Debug 映射 Release ABI 的预编译 Qt；删除逐目标 helper 与 GTest 二次覆盖。顶层提前验证 FFI 输入，按基础库→Bridge→Shell→App 创建目标，关闭测试时不发现 Qt Test。增加递归 ABI 审计和正反例配置测试，更新 `standards/cmake.md`；039 在索引中继续保持 in-progress，Windows 新 run 待验证。

- 2026-09-17：根据 GitHub Actions run `35195983722` 创建任务；确认失败分为 staticlib 供给顺序和 MSVC C++ 标准选项两类。
- 2026-09-17（实施）：launcher 优先定位 Cargo profile 根目录 staticlib，并以 deps/作为布局兜底；panta-ffi 按 target 环境向 MSVC 传递 `/std:c++20`，非 MSVC 保持 `-std=c++20`。
- 2026-09-17（收敛）：CI 预构建曾验证为有效但不适合作为长期入口；任务 040 通过 launcher 的普通依赖 + build-dependency 双边表达静态库先后顺序，workflow 恢复为单一 `cargo build --locked`。

## 完成摘要

已完成。FFI staticlib 供给顺序（最终由 040 的 manifest 双边表达）、MSVC `/std:c++20`、native 全图 CRT/iterator ABI 统一（`build-policy.cmake` + 递归审计）与 Windows 系统库链接，均由三平台 CI 冷缓存 run `35203709898` 与缓存命中 run `35204747014` 验证通过；本地另有多配置构建、失败诊断与 ABI 审计正反例证据。遗留限制：CI 只链接 Windows 边界测试而未执行（执行验证目前在本机 macOS ctest），GTest 运行聚合由 011 统一入口承接。
