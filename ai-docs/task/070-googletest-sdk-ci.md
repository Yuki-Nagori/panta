# 070 — GoogleTest 三平台 SDK 制品 CI

- 状态：done
- 阶段：验证基础
- 依赖：[019 GTest 测试配置与规则](019-gtest-native-testing.md)、[031 预编译 native 依赖](031-prebuilt-native-dependencies.md)、[038 native SDK 制品生产](038-native-sdk-artifact-production.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

GoogleTest v1.18.0 目前通过 CMake FetchContent 从 Git 仓库拉取固定 commit。Git 历史克隆在网络慢或受限环境中会拖住 Cargo 聚合构建。上游 GitHub Release 提供源码归档而非各平台可直接链接的二进制库；本任务增加仓库自己的受信 CI，在 macOS arm64、Ubuntu x86_64、Windows x86_64 分别构建并打包静态 GoogleTest SDK，完成包自检后上传 Actions artifacts，并支持由维护者显式发布到固定 GitHub Release。

本任务仅交付 SDK 生产管线和可验证制品，不切换普通构建中的 GoogleTest 消费路径。等三平台资产发布并取得真实 URL/SHA256 后，再由独立消费任务更新 `sdk-provision` manifest 与 `BUILD_TESTING` 集成。

## 必读

- [依赖获取与固定清单](../standards/dependency-acquisition.md)
- [GoogleTest 使用规范](../standards/gtest.md)
- [031 预编译 native 依赖](031-prebuilt-native-dependencies.md)
- [038 native SDK 制品生产](038-native-sdk-artifact-production.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [GoogleTest v1.18.0 upstream release](https://github.com/google/googletest/releases/tag/v1.18.0)（2026-09-24 查阅）

## 范围与非目标

- 新增手动触发的 SDK workflow，固定 GoogleTest tag commit `063de7e9578f82b369302001269680b4b1553359`，矩阵覆盖 macOS arm64、Ubuntu x86_64、Windows x86_64。
- 使用上游 CMake 安装规则生产静态库、headers 与 `GTestConfig.cmake`，保留 BSD-3-Clause license，写入来源、版本、目标平台和 ABI 元数据。
- 对每个平台执行真实 `find_package(GTest CONFIG)`、imported target 检查和小型编译/链接/运行自检，失败时不得打包或发布。
- 复用仓库 SDK package 规则上传每个平台归档及 SHA256；提供可选的单 job GitHub Release 发布入口，重产时统一清理旧资产后上传。
- 不在本任务切换 `native/CMakeLists.txt` 当前 FetchContent 消费路径；待首轮 Release 资产和摘要齐全后另立消费任务。
- 不启用 GoogleMock，不安装到生产应用，也不让普通 CI 源码编译 GoogleTest SDK。

## 前置条件与待决策

- GoogleTest v1.18.0 commit SHA 固定为 `063de7e9578f82b369302001269680b4b1553359`；workflow 使用浅 checkout，不获取提交历史。
- Release tag 采用 `sdk-googletest-1.18.0`；发布默认关闭，需手动启用 workflow 输入。
- SDK 使用平台 runner 默认 C++ ABI 和动态 CRT 策略；静态库由每个平台独立构建，Windows 配置 `gtest_force_shared_crt=ON`。
- 首轮三平台生产/发布成功后，再由后续消费任务记录实际 SHA256 和资产 URL。

## 实施步骤

1. 增加 GoogleTest SDK 消费自检工程，验证包内配置、targets、headers、静态链接和运行结果。
2. 增加受信 `workflow_dispatch` 三平台生产矩阵：浅 checkout 固定 commit、配置/build/install、补齐 license/元数据、自检、打包、上传 Actions artifacts。
3. 增加可选的串行 Release 收口 job，遵循任务 038 的单 tag 全量覆盖语义。
4. 更新固定依赖清单和本任务记录，区分当前 FetchContent 消费路径与未来预编译 SDK 消费路径。

## 预计改动

- `.github/workflows/sdk-googletest.yml`（新增）
- `tools/sdk/selfcheck/googletest/`（新增自检 CMake 工程与 smoke test）
- `ai-docs/standards/dependency-acquisition.md`
- `ai-docs/task-index.md`
- `ai-docs/task/070-googletest-sdk-ci.md`

不改 CMake 消费者、manifest URL/SHA256 或普通 CI 触发逻辑。

## 清理与兼容例外

无兼容例外。当前 FetchContent 实现保留为现行消费路径；预编译 SDK 消费切换及旧获取方式清理由后续任务负责，避免在发布资产不存在时引入不可构建状态。

## 验收标准

- [x] workflow 仅手动触发，三平台各产出独立 SDK 归档和 SHA256。
- [x] SDK 固定到指定上游 commit；包内含所需头文件、静态库、CMake config、BSD-3-Clause license 和来源元数据。
- [x] 每个平台的 SDK 自检均成功完成 configure、compile、link 和 smoke test 后才允许上传 artifacts / Release。
- [x] Release 默认不发布；启用发布时由单一 job 全量替换 `sdk-googletest-1.18.0` 资产。
- [x] 普通 CI 和当前本地 Cargo 测试入口行为保持不变；后续消费任务可凭真实资产 URL/SHA256 登记 manifest。
- [x] 本任务和索引记录真实 workflow/本地验证结果，CI-only 限制如实注明。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | macOS：`cargo format` | 仓库聚合格式入口通过 | 通过 |
| 2026-09-24 | macOS：`actionlint .github/workflows/sdk-googletest.yml`；Ruby YAML parser | workflow 语法与 GitHub Actions 表达式通过静态检查 | 通过 |
| 2026-09-24 | Actions run [35972333653](https://github.com/Yuki-Nagori/panta/actions/runs/35972333653) `workflow_dispatch` | Linux/macOS/Windows 每个平台完成固定源码校验、SDK build/install、自检、package、artifact upload；Release 收口成功 | 全绿；Release `sdk-googletest-1.18.0` 已发布 |
| 2026-09-24 | 下载三平台 Release 归档与 sidecar、`shasum -a 256`、`tar tzf` | 实际摘要与 GitHub Release API digest / sidecar 相同，归档内有 config、静态库、license | 通过；消费摘要记录在任务 071 与依赖固定清单 |

## 风险与回退

平台 C++ runtime 或 upstream install/export 布局变化会导致 SDK config 失效；自检须在每个平台用实际编译器验证，发布前先通过 artifacts 阶段。失败时 workflow 阻断 Release，保留上一个已发布资产；回退可恢复此 workflow 与自检工程，不触碰当前测试消费路径。

## 决策与工作记录

- 2026-09-24：维护者决定 GoogleTest 由仓库自有 CI 构建、打包；不依赖上游提供不存在的预编译二进制包。
- 2026-09-24：创建任务。
- 2026-09-24：三平台 workflow run 35972333653 全绿并发布 `sdk-googletest-1.18.0`；所有矩阵 job 的 SDK 静态链接自检与发布 job 均通过。

## 完成摘要

已交付手动三平台 SDK 生产 workflow、自检工程与单 job 全量覆盖发布流程。run 35972333653 的 macOS arm64、Linux x86_64、Windows x86_64 build/install、自检、打包和 Release 发布全部成功；三份归档均下载核对 SHA256 sidecar 与 GitHub API digest，并检查包含 `GTestConfig.cmake`、静态库、headers、metadata 和 BSD-3-Clause license。消费者接入由任务 071 负责。
