# 071 — GoogleTest SDK 消费接入

- 状态：in-progress
- 阶段：验证基础
- 依赖：[070 GoogleTest 三平台 SDK 制品 CI](070-googletest-sdk-ci.md)、[031 预编译 native 依赖](031-prebuilt-native-dependencies.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

任务 070 已把 GoogleTest v1.18.0 静态 SDK 发布到 `sdk-googletest-1.18.0`。各平台归档均有 `.sha256` sidecar，且经下载实测与 GitHub Release API digest 相符。把 `BUILD_TESTING=ON` 时的 GTest 获取切换到仓库 `sdk-provision` 固定 manifest，使 `cargo test --locked --workspace` 不再触发完整 Git 历史克隆，并能在 `target/panta-deps/sdk` 缓存已校验的包。

## 必读

- [依赖获取与固定清单](../standards/dependency-acquisition.md)
- [GoogleTest 使用规范](../standards/gtest.md)
- [031 预编译 native 依赖](031-prebuilt-native-dependencies.md)
- [070 GoogleTest SDK CI](070-googletest-sdk-ci.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [GoogleTest SDK Release](https://github.com/Yuki-Nagori/panta/releases/tag/sdk-googletest-1.18.0)（2026-09-24 实测）

## 范围与非目标

- 在 `sdk-provision.cmake` 登记 GoogleTest v1.18.0 与三平台 Release URL/SHA256，统一通过 `panta_require_sdk(googletest ...)` 下载、校验、解包与缓存。
- 仅 `BUILD_TESTING=ON` 时请求该 SDK，并消费导出的 `GTest::gtest` / `GTest::gtest_main` targets。
- 移除 `native/CMakeLists.txt` 中旧 FetchContent 获取方式及对应选项；不保留源码克隆回退。
- 更新依赖固定清单、任务 070 和 task index，保留上游 source SHA、SDK build ABI 与 Release 证据。
- 不改 GoogleTest 版本、不启用 GoogleMock、不新增 system package 旁路。

## 前置条件与待决策

- Release `sdk-googletest-1.18.0` 已由修正版 workflow run [35999015400](https://github.com/Yuki-Nagori/panta/actions/runs/35999015400) 重产三平台资产与 sidecar。新归档 SHA256 与 sidecar 内容及 Release API digest 一致：
  - macOS arm64：`654e87d943c68ab964da77ac3e9514900049e6f1b3c044e311017a7725cc2230`
  - Linux x86_64：`4b4b1281828c095cda29fdc4e7b150d816296da2b2c1375a2a2326ebc68de5ae`
  - Windows x86_64：`d070f6fbe77d1060033e8eb6bac97b7efba736db7b40fc50ef2ce9ce914c03ff`
- 归档内包含 `GTestConfig.cmake`、静态库、headers、`panta-sdk.json` 和 BSD-3-Clause license。

## 实施步骤

1. 增加 manifest 固定版本与三平台资产条目，SHA256 对照 sidecar 与实际下载。
2. 在 native testing 配置中以 `panta_require_sdk` 供给 `GTest` CONFIG package，删除 FetchContent 源码下载。
3. 同步更新依赖清单和 task 019 的现行获取路径说明。
4. 使用 Cargo 聚合入口运行三平台适用测试；确认关测试的 configure 不请求 GoogleTest，缓存命中和损坏修复走现有 sdk-provision 行为。

## 预计改动

- `native/cmake/sdk-provision.cmake`
- `native/CMakeLists.txt`
- `crates/launcher/build.rs`
- `tests/src/main.rs`
- `ai-docs/standards/dependency-acquisition.md`
- `ai-docs/standards/gtest.md`
- `ai-docs/standards/baseline.md`
- `ai-docs/architecture/build-and-development.md`
- `ai-docs/task/019-gtest-native-testing.md`
- `ai-docs/task/070-googletest-sdk-ci.md`
- `tools/sdk/releases/googletest-1.18.0.md`
- `ai-docs/task-index.md`
- `ai-docs/task/071-googletest-sdk-consumption.md`

## 清理与兼容例外

删除 `native/CMakeLists.txt` 的 GoogleTest FetchContent 声明和仅服务该获取方式的配置；没有源码或系统库 fallback，也无兼容例外。

## 验收标准

- [x] 三平台 manifest 固定正确 Release URL 与已核实 SHA256。
- [x] `BUILD_TESTING=ON` 仅通过 `panta_require_sdk` 获得 `GTest::gtest` 和 `GTest::gtest_main`，无 FetchContent/Git clone。
- [x] `BUILD_TESTING=OFF` 不请求、不下载、不查找 GoogleTest SDK。
- [x] `cargo test --locked --workspace` 在当前 macOS 平台通过；native CTest 中各 GoogleTest suite 全绿。
- [x] SDK 归档二次复用不下载；损坏 marker/哈希时按现有 provision 契约重建或拒绝。
- [ ] 三平台消费 CI 使用固定资产完成构建与测试；task / index / 固定清单已同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | 下载三平台 Release 归档及 sidecar，`shasum -a 256` 与 `tar tzf` | 摘要一致，SDK 含 CMake config、静态库、headers 与 license | 通过；见前置条件中的 SHA256 |
| 2026-09-24 | macOS：`cargo test --locked --workspace` | Rust 与 native/GoogleTest 聚合测试通过 | 两次均通过；native CTest 56/56，包含 `Build.SdkProvision` |
| 2026-09-24 | macOS：缓存命中后再次运行 `cargo test --locked --workspace` | GoogleTest staging marker 命中且不需要归档下载 | 通过；`.panta-sdk-provisioned` SHA256 与 manifest 一致，SDK `archives` 目录无归档 |
| 2026-09-24 | macOS：managed CMake configure `-DBUILD_TESTING=OFF`、`-DPANTA_ENABLE_FFI_TEST=OFF` | 不请求 GoogleTest，其他产品依赖正常配置 | 通过；cache 确认为 `BUILD_TESTING:BOOL=OFF`，configure 输出无 GoogleTest 供给步骤 |
| 2026-09-24 | macOS：`cargo run --locked -p panta-tests -- toolchain` | 当前平台解析到固定 GoogleTest SDK config | 通过；解析为 `macos-arm64/lib/cmake/GTest/GTestConfig.cmake` |
| 2026-09-24 | GitHub Actions run [35992062001](https://github.com/Yuki-Nagori/panta/actions/runs/35992062001)，Windows `cargo check and build` 与 sanitizer | 三平台固定 SDK consumer 链接与测试通过 | Windows 失败；`lld-link` 报 `testing::*`、`__mingw_vfprintf`、`__cxxabiv1` 未解析符号，输入为 MinGW `libgtest.a`，不兼容 MSVC ABI。修复和重产证据见任务 075 |
| 2026-09-24 | GitHub Actions run [35999015400](https://github.com/Yuki-Nagori/panta/actions/runs/35999015400)：下载三平台 `.sha256` sidecar，并与 Release API archive digest 对照 | 新制品摘要准确且 Release 资产完整 | 通过；三平台 sidecar 值逐一等于 Release API 对应 `.tar.gz` digest，已同步至 manifest 与本 task |
| 2026-09-24 | macOS：更新三平台摘要后运行 `cargo test --locked --workspace` | staging 用新 manifest 摘要下载/接受当前平台 SDK，native 与 Rust 测试通过 | 通过；staging marker SHA256 为 `654e87d943c68ab964da77ac3e9514900049e6f1b3c044e311017a7725cc2230`，native CTest 56/56 |
| 2026-09-24 | GitHub Actions run [35998981964](https://github.com/Yuki-Nagori/panta/actions/runs/35998981964) | 三平台 consumer CI 使用新 Release 摘要通过 | 该 run 的 commit `262ad8b` 尚未包含新 SHA；Windows 比较旧期望值与新资产 digest 后按设计拒绝，不能视为新 manifest 的消费者失败。下一次 push CI 待复验 |

## 风险与回退

SDK 的静态库 ABI 必须匹配消费者的系统标准库和 Windows CRT。任务 070 早期 Windows 生产自检未覆盖该边界，run 35992062001 已暴露 MinGW/MSVC 不匹配；任务 075 固定生产工具链并重产归档。在 manifest 摘要更新且 consumer CI 通过前，本任务保持 in-progress。Release 资产被删除或哈希变化时 manifest 强校验会阻断 configure，不回退系统包或源码克隆；恢复对应资产或回退整笔消费接入即可。

## 决策与工作记录

- 2026-09-24：维护者确认三平台 SDK 已打包，开始登记 manifest 并接入 Cargo 测试构建。
- 2026-09-24：创建任务。
- 2026-09-24：平台支持限定为 macOS arm64、Linux x86_64、Windows x86_64；其他 OS/架构组合明确报错。
- 2026-09-24：完成 manifest、CMake 消费、toolchain 检查及文档迁移；macOS 聚合测试两次通过、`BUILD_TESTING=OFF` configure 和 toolchain 检查通过。等待三平台消费者 CI 结果后关闭任务。
- 2026-09-24：run 35992062001 的 Linux/macOS consumer checks 通过，但 Windows normal build 和 sanitizer 均在链接 GTest 时失败；确认 Release Windows `libgtest.a` 是 MinGW ABI。跟踪任务 075 将修复 Windows producer、自检与资产摘要。

## 完成摘要

接入实现与本地 macOS 验证已完成：GoogleTest 静态 SDK 通过固定 URL/SHA256 manifest 供给；移除 FetchContent；仅支持 macOS arm64、Linux x86_64、Windows x86_64。macOS Cargo 聚合测试（native CTest 56/56）、缓存复用、`BUILD_TESTING=OFF` configure、toolchain 检查及格式检查通过。run 35992062001 的 Linux/macOS consumer checks 通过，Windows 因 SDK ABI 错配失败；等待任务 075 重产归档并完成三平台 consumer CI。
