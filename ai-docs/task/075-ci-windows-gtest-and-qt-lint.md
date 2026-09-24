# 075 — CI 修复：Windows GoogleTest ABI 与 Qt benchmark lint

- 状态：done
- 阶段：验证基础
- 依赖：[070 GoogleTest SDK 制品 CI](070-googletest-sdk-ci.md)、[071 GoogleTest SDK 消费接入](071-googletest-sdk-consumption.md)、[046 CI 触发拆分与缓存预算](046-ci-trigger-split-cache-budget.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-24

## 目标与背景

修复 GitHub Actions run `35992062001`（提交 `47731650fc3d42c10eff555d07e2d5d26bd98791`）暴露的 Windows GoogleTest ABI 错配和 Qt benchmark clang-tidy 生成文件缺失。该 push 从 `375e89b` 开始包含多笔提交，其中 `da880de` 修改了 QML project docks，因此触发代码 CI 正确；失败项为 Windows `cargo check and build` / sanitizer 链接 GTest，以及 Ubuntu `clang-tidy` / `includes` 缺少两个 Qt benchmark 的源内 `.moc` 文件。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [GoogleTest SDK 消费接入](071-googletest-sdk-consumption.md)
- [GoogleTest SDK 制品 CI](070-googletest-sdk-ci.md)
- [CI 触发拆分与缓存预算](046-ci-trigger-split-cache-budget.md)

## 范围与非目标

- Windows SDK 生产和自检使用明确的 MSVC 2022 x64 工具链及 Release 配置，确保生成物匹配消费者 ABI；同步记录修复后的产物校验和与三平台消费者 CI 结果。
- clang-tidy / include-cleaner / cppcheck 在分析 QML benchmark 翻译单元及 CMake autogen 单元前，确保对应 Qt moc 生成步骤已完成，包括单项 lint 命令。
- 不改变 GoogleTest 上游版本、测试行为、QML benchmark 是否进入默认构建或 CI 性能门禁。
- 不在此任务中修改已发布 SDK；修复后的 Windows Release 制品需要生产 workflow 重新验证并发布，manifest 只记录实际产物摘要。

## 前置条件与待决策

- SDK 生产 workflow 的 Windows job 必须可访问 Visual Studio 2022 x64 生成器。
- 修正后的 Windows 归档需经 production workflow 自检、归档摘要核验，再更新消费 manifest；在此之前不能宣称 Windows consumer CI 修复完成。

## 实施步骤

1. 修正 GoogleTest SDK Windows 生产与自检配置，明确选择 MSVC x64 和 Release。
2. 为静态扫描器的 CMake compile database 中 `EXCLUDE_FROM_ALL` Qt benchmark 准备 moc 生成输出；单项 lint 与聚合 lint 使用相同前置条件。
3. 运行当前主机适用的 Cargo 聚合验证，记录未能在 macOS 本机复现的 Windows CI 限制。
4. 通过 Windows SDK production workflow 产出并验证修正归档，更新 manifest 与消费证据；等待三平台 CI 全绿后完成 task。

## 预计改动

- `.github/workflows/sdk-googletest.yml`
- `tests/src/main.rs`
- [076 Cargo 测试与质量 runner 维护](076-panta-tests-runner-maintenance.md) 中的 runner 局部收敛同文件提交，但独立记录与验收。
- `native/cmake/sdk-provision.cmake`（仅当新 Windows 归档摘要确定后）
- `ai-docs/task/070-googletest-sdk-ci.md`
- `ai-docs/task/071-googletest-sdk-consumption.md`
- `ai-docs/task-index.md` 与本任务

## 清理与兼容例外

没有废弃实现或兼容例外；不为旧 ABI 归档添加运行时 fallback。

## 验收标准

- [x] GoogleTest Windows 生产 archive 的库文件由 MSVC 2022 x64 在 Release 模式构建，SDK 自检使用同一 ABI 并链接、运行通过。
- [x] 三平台新归档 checksum 与 sidecar、Release API digest 和本地 manifest 一致。
- [x] Windows consumer 构建 / sanitizer 链接不再出现 GTest 与 MinGW 运行时未解析符号，且测试通过。
- [x] 两个 QML benchmark 的 moc 输出在 clang-tidy、include-cleaner、cppcheck 分析开始前可用，相关 CI 检查通过。
- [x] 当前主机适用的 Cargo 聚合验证通过；Windows SDK manifest 与跨平台索引已同步新制品摘要。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | GitHub Actions run `35992062001`：查看 Windows linker 与 Ubuntu clang-tidy/includes 日志 | 确认失败符号和缺失输入 | 已确认：GTest 链接出现 `__mingw_vfprintf`、`__cxxabiv1` 与 `testing::*` 未解析符号；benchmark 报缺少源内 `.moc` include；push 包含 `da880de` 的 QML 改动，路径分类器触发代码 CI 正确 |
| 2026-09-24 | 当前工作区：检查 GoogleTest workflow、SDK manifest、CMake compile database 与 benchmark target 属性 | 确认 Windows 目标 ABI 和 lint 生成依赖 | 已确认 workflow 设 Ninja 但未声明 MSVC 编译器；消费者调用 `lld-link`；两个 benchmark 使用 `EXCLUDE_FROM_ALL` 并仍在 clang-tidy compile database 中 |
| 2026-09-24 | macOS：`actionlint .github/workflows/sdk-googletest.yml` | Windows producer workflow 语法通过 | 通过；明确使用 Visual Studio 2022 x64 生成器，并以 Release 配置 build/install；Windows 库名自检拒绝 MinGW `.a` |
| 2026-09-24 | macOS：`cargo format --check`、`cargo build`、`cargo lint --check`、`cargo test --locked --workspace` | 当前平台聚合验证通过，所有扫描器前两个 benchmark moc 已生成 | 全部通过；八阶段 lint 通过（含 clang-tidy、include-cleaner、cppcheck），新 manifest 对应 macOS SDK staging marker 命中，native CTest 56/56 |
| 2026-09-24 | GitHub Actions run [35999015400](https://github.com/Yuki-Nagori/panta/actions/runs/35999015400) `workflow_dispatch`，发布输入开启 | MSVC Release SDK 通过自检并替换三平台 Release 资产 | 全绿；Windows 使用 Visual Studio 2022 x64；三平台 package 与 selfcheck 均成功 |
| 2026-09-24 | 下载三个 `.sha256` sidecar 并与 Release API `digest` 对照 | 更新 manifest 的三平台归档摘要 | 一致；macOS `654e87d943c68ab964da77ac3e9514900049e6f1b3c044e311017a7725cc2230`，Linux `4b4b1281828c095cda29fdc4e7b150d816296da2b2c1375a2a2326ebc68de5ae`，Windows `d070f6fbe77d1060033e8eb6bac97b7efba736db7b40fc50ef2ce9ce914c03ff`；已更新固定清单 |
| 2026-09-24 | CI run [35998981964](https://github.com/Yuki-Nagori/panta/actions/runs/35998981964) | 新 manifest 下三平台 consumer build、tests 与 sanitizer 通过，静态扫描器准备 benchmark moc | 失败且使用旧 commit/manifest：Windows 下载新 archive 的实际 SHA `d070f6…` 与旧预期值不同；Ubuntu 独立 cppcheck 命令未生成 benchmark `mocs_compilation.cpp`。manifest 已在本地更新，cppcheck 缺失前置步骤现修正并待复验 |
| 2026-09-24 | GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4` | 三平台 consumer build/test 与 sanitizer 通过；clang-tidy、include-cleaner、cppcheck 均取得 benchmark moc 输出 | 全绿。Windows build、tests 和 sanitizer 通过；macOS/Linux consumer build/test、Linux C++/QML 与覆盖率、三个静态扫描器、格式及其他 lint 均通过；本次 CI run success |

## 风险与回退

Windows Release 归档与 SHA manifest 必须作为一个可审计的版本化更新，不能用生产自检的 MinGW 构建结果替换 MSVC ABI 包。若生成器或 CMake package 无法在 Windows runner 复现，可撤回 workflow 变更并保持 task in-progress；不调整 CMake ABI 策略来迎合不匹配 SDK。

## 决策与工作记录

- 2026-09-24：根据 GitHub Actions run `35992062001` 登记 CI 修复；发现 Windows 归档和自检的编译器配置均未满足 manifest 声明的 MSVC ABI，lint 编译数据库收录了尚未生成 moc 的排除构建目标。
- 2026-09-24：维护者要求整体审查 `tests/src/main.rs`，相关维护整理另登记为任务 076，避免将 runner 清理伪装成 CI 根因修复。
- 2026-09-24：修复 SDK producer workflow 的 Windows 生成器与 Release 安装配置；run 35999015400 已在 MSVC ABI 下重产并发布三平台 SDK，sidecar 和 API digest 已核对且 manifest 已更新。run 35998981964 因旧 manifest 哈希和独立 cppcheck 未准备 moc 而失败；补齐修复后由 run 36001859191 全绿复验。

## 完成摘要

Windows producer workflow 已在 MSVC 2022 x64 Release 下通过，三平台 Release 资产已重产，manifest SHA 与各 sidecar / API digest 一致。run 36001859191 验证三平台消费者、Windows sanitizer，以及 clang-tidy、include-cleaner、cppcheck 所需 Qt moc 生成，CI 全绿；ABI 和 benchmark lint 修复闭环完成。
