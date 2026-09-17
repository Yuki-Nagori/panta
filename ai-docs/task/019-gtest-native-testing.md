# 019 — GTest 测试配置与规则

- 状态：done
- 阶段：验证基础
- 依赖：[003](003-cmake-native-skeleton.md)（已完成）
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

003 的最小测试使用了手写断言（`if` + 返回码），无法表达失败上下文、无法随用例数扩展。本任务引入 GoogleTest 作为 C++ 测试框架：替换自有断言、固定获取方式与版本，并把 GTest 的项目使用规则制定为长期规范（[gtest](../standards/gtest.md)），供后续 geometry/mesh 等模块测试遵循。

## 必读

- [通用规范：validation-and-review](../standards/validation-and-review.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [规范：依赖获取](../standards/dependency-acquisition.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [官方：GoogleTest Primer](https://google.github.io/googletest/primer.html)（2026-09-16 查阅）

## 范围与非目标

范围：`foundation` 测试改写为 GTest；CMake 侧 GTest 获取与注册机制；GTest 项目规则成文；固定清单与基线同步。

非目标：不引入 011 的统一测试聚合；不为 CI 接入 native 测试（012）；不引入 gmock 用例（规则允许，待有依赖边界场景）；不改 Rust 侧测试。

## 前置条件与待决策

实施前待决策项（均已定，见决策记录）：GTest 版本与获取机制（Cargo 托管原则下的 CMake FetchContent + commit SHA 固定）；`BUILD_TESTING` 门控；测试命名与断言规则；CTest 注册方式（`gtest_discover_tests`）。

## 实施步骤

1. 固定 googletest 最新稳定版（tag + commit SHA），以 FetchContent 在 `BUILD_TESTING` 下按需拉取，EXCLUDE_FROM_ALL 且不进安装。
2. `foundation` 的 `version_test` 改写为 GTest 断言，经 `gtest_discover_tests` 注册。
3. 制定 [gtest 规范](../standards/gtest.md)：断言、命名、fixture、容差、death test、mock 边界与第三方隔离规则。
4. 同步依赖固定清单、baseline 与索引；验证干净构建、ctest、失败路径与 `BUILD_TESTING=OFF` 关闭路径。

## 预计改动

`native/CMakeLists.txt`、`native/foundation/tests/version_test.cpp`、`native/foundation/CMakeLists.txt`（测试定义）；新增 `ai-docs/standards/gtest.md`；更新 dependency-acquisition 固定清单、baseline、规范索引、task-index。与实际一致。

## 清理与兼容例外

删除 003 引入的手写断言测试实现（`if` + 返回码 main，含 `<cstdlib>`），由 GTest 版本替代；无兼容层。

## 验收标准

- [x] GTest 按固定版本经 FetchContent 获取，`BUILD_TESTING=OFF` 时不拉取、不构建、不影响安装树。
- [x] `foundation` 测试仅使用 GTest 断言；ctest 以独立用例注册并可诊断运行。
- [x] GTest 项目规则成文（断言/命名/fixture/容差/隔离），失败路径实测可见 gtest 报告。
- [x] 已同步固定清单、baseline、规范索引与 task-index，未将规划能力写成已完成。

## 验证计划与结果

以下命令均在 cwd=`native/`、macOS 26.3.1 arm64、Apple clang 17.0.0、cmake 4.3.3 + Ninja 1.13.2 执行：

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | GitHub API：googletest 版本核实 | 最新稳定 v1.18.0（2026-08-10），完整 commit SHA `063de7e9578f82b369302001269680b4b1553359`（即 FetchContent `GIT_TAG`） |
| 2026-09-16 | 干净 `cmake --preset debug`（FetchContent 拉取）→ `cmake --build --preset debug` → `ctest --preset debug` | 拉取与构建成功（含 gtest 主库）；ctest 以独立用例名注册：`Foundation.NativeVersionMatchesProjectConfiguration` Passed（1/1） |
| 2026-09-16 | 失败路径：临时注入 `EXPECT_EQ(actual.patch, kExpected.patch + 1)` → ctest | ctest 红，gtest 输出完整上下文（"Expected equality of these values / actual.patch / Which is: 0 / kExpected.patch + 1 / Which is: 1"）；还原后复绿 |
| 2026-09-16 | `BUILD_TESTING=OFF` 独立目录 configure+build+install（显式 prefix） | `_deps` 不存在（零拉取）、仅构建 `panta_foundation`、安装树完整；`ctest` 报 no tests |
| 2026-09-16 | debug 安装树复查 | 无任何 gtest 产物（`INSTALL_GTEST=OFF` + EXCLUDE_FROM_ALL）；与 003 安装清单一致 |
| 2026-09-16 | 无改动重建；`clang-format --dry-run -Werror` | `ninja: no work to do`；格式通过 |
| 2026-09-16 | 编辑器误报处理：`EXPECT_EQ` 展开在 IntelliSense 报"表达式必须包含 bool 类型"（error 711） | 实际编译 `-std=c++20` + ctest 通过（compile_commands.json 已核实含 `-std=c++20` 与 gtest 头路径）——确认为编辑器未使用真实编译配置的误报；`.vscode` 增加 `cmake.copyCompileCommands`/`C_Cpp.default.compileCommands`/`cppStandard`，处理说明写入 gtest.md |

未覆盖：Windows CRT 匹配（`gtest_force_shared_crt`）无本机验证，待 012 runner；gmock 未启用未验证；Rust 侧与 native 测试的统一聚合归 011。

## 风险与回退

FetchContent 使 configure 依赖网络（与托管原则一致，首次构建需联网；拉取结果缓存在构建树 `_deps`）；GTest 编译选项与本树隔离（EXCLUDE_FROM_ALL、告警仅自有 target）。回退仅撤销本任务变更，恢复 003 的手写断言测试实现并在任务记录注明。

## 决策与工作记录

- 2026-09-16：维护者要求引入 GTest 替代自写断言，并将 GTest 规则在任务内制定成文。
- 2026-09-16（决策）版本与获取：googletest v1.18.0（当前最新 release）；FetchContent `GIT_TAG` 用完整 commit SHA——仍属 Cargo 托管原则（CMake 侧按固定清单拉取到构建树），configure 阶段联网与 004 引导一致；`INSTALL_GTEST=OFF`、`EXCLUDE_FROM_ALL`，安装树与导出不含测试依赖。
- 2026-09-16（决策）门控：`include(CTest)` + `BUILD_TESTING`（默认 ON）；OFF 时零拉取零构建（实测），004 的发布链可关闭测试。
- 2026-09-16（决策）`BUILD_GMOCK=OFF`：当前无 mock 场景，需要时按对应 task 打开并说明边界；规则已写入 gtest.md。
- 2026-09-16（决策）注册用 `gtest_discover_tests`（每用例独立 CTest 项），不用整体 `add_test`；测试名即过滤名。2026-09-17 起多配置生成器使用 `DISCOVERY_MODE PRE_TEST`，把测试发现延后到 ctest，避免构建阶段缺少 Qt runtime 阻断产物生成。
- 2026-09-16（补充，维护者反馈）：编辑器对 `EXPECT_EQ` 展开报"非 bool"（cpptools error 711）属 IntelliSense 未使用真实编译配置的误报；共享 `.vscode/settings.json` 增加 `cmake.copyCompileCommands` + `C_Cpp.default.compileCommands`（指向 gitignored 的 `native/compile_commands.json`）与 `cppStandard=c++20`，处理指引写入 gtest.md。判定依据：真实编译带 `-std=c++20` 且 ctest 绿（见验证表）。
- 2026-09-16（实施）实施中修正两处自身问题并记录：初稿在注释里写了 `include(GoogleTest)` 却漏了实际命令（报 Unknown CMake command）；失败还原用 `mv` 保留了旧 mtime，ninja 判定无需重编导致"复绿"假阳性——以 `touch` 强制重建后真实验证。此坑对增量构建验证有普遍意义，收录于验证表。
- 待记录：gmock 启用时机（首个模块边界 mock 场景的 task）。

## 完成摘要

已交付：googletest v1.18.0 经 FetchContent（完整 SHA 固定）接入 native 测试，仅 `BUILD_TESTING` 下拉取；`foundation` 测试改写为 GTest 断言并以 `gtest_discover_tests` 注册独立用例；GTest 项目规则成文于新增 [gtest.md](../standards/gtest.md)（断言、命名、fixture、浮点容差、death test、mock 边界、第三方隔离），并登记规范索引。固定清单与 baseline 增加 GoogleTest 条目。验证：干净拉取构建、独立用例注册、失败注入可见 gtest 上下文、`BUILD_TESTING=OFF` 零拉取、安装树无测试产物、无改动不重编。剩余限制：MSVC/CRT 组合待 012 runner；gmock 未启用；聚合归 011。后续：004（Cargo 调度 CMake 与运行入口）保持 ready。
