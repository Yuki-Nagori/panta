# 075 — CI 修复：Windows GoogleTest ABI 与 Qt benchmark lint

- 状态：in-progress
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
- clang-tidy / include-cleaner 在分析 QML benchmark 翻译单元前，确保对应 Qt moc 生成步骤已完成。
- 不改变 GoogleTest 上游版本、测试行为、QML benchmark 是否进入默认构建或 CI 性能门禁。
- 不在此任务中修改已发布 SDK；修复后的 Windows Release 制品需要生产 workflow 重新验证并发布，manifest 只记录实际产物摘要。

## 前置条件与待决策

- SDK 生产 workflow 的 Windows job 必须可访问 Visual Studio 2022 x64 生成器。
- 修正后的 Windows 归档需经 production workflow 自检、归档摘要核验，再更新消费 manifest；在此之前不能宣称 Windows consumer CI 修复完成。

## 实施步骤

1. 修正 GoogleTest SDK Windows 生产与自检配置，明确选择 MSVC x64 和 Release。
2. 为 clang-tidy 的 CMake compile database 中 `EXCLUDE_FROM_ALL` Qt benchmark 准备 moc 生成输出。
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

- [ ] GoogleTest Windows 生产 archive 的库文件由 MSVC 2022 x64 在 Release 模式构建，SDK 自检使用同一 ABI 并链接、运行通过。
- [ ] 新 Windows 归档的 checksum 与 manifest 一致，Windows consumer 构建 / sanitizer 链接不再出现 GTest 与 MinGW 运行时未解析符号。
- [x] 两个 QML benchmark 的 moc 输出在 clang-tidy 与 include-cleaner 分析开始前可用，两个 lint 检查通过。
- [x] 当前主机适用的 Cargo 聚合验证通过；Windows SDK manifest 与跨平台索引仍待新制品摘要。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | GitHub Actions run `35992062001`：查看 Windows linker 与 Ubuntu clang-tidy/includes 日志 | 确认失败符号和缺失输入 | 已确认：GTest 链接出现 `__mingw_vfprintf`、`__cxxabiv1` 与 `testing::*` 未解析符号；benchmark 报缺少源内 `.moc` include；push 包含 `da880de` 的 QML 改动，路径分类器触发代码 CI 正确 |
| 2026-09-24 | 当前工作区：检查 GoogleTest workflow、SDK manifest、CMake compile database 与 benchmark target 属性 | 确认 Windows 目标 ABI 和 lint 生成依赖 | 已确认 workflow 设 Ninja 但未声明 MSVC 编译器；消费者调用 `lld-link`；两个 benchmark 使用 `EXCLUDE_FROM_ALL` 并仍在 clang-tidy compile database 中 |
| 2026-09-24 | macOS：`actionlint .github/workflows/sdk-googletest.yml` | Windows producer workflow 语法通过 | 通过；明确使用 Visual Studio 2022 x64 生成器，并以 Release 配置 build/install；Windows 库名自检拒绝 MinGW `.a` |
| 2026-09-24 | macOS：`cargo format --check`、`cargo build`、`cargo lint --check`、`cargo test --locked --workspace` | 当前平台聚合验证通过，两个 benchmark moc 在扫描前生成 | 全部通过；clang-tidy 与 include-cleaner 两阶段通过，native CTest 56/56 |
| — | Windows SDK production workflow、更新 Release 资产 / manifest，以及修复后的三平台 consumer CI | MSVC Release SDK 发布且所有消费者链接通过 | 尚未运行；需要将 workflow 修复提交到远程后，由维护者显式启用发布并获取真实 checksum，不能在本地 macOS 证明 Windows ABI |

## 风险与回退

Windows Release 归档与 SHA manifest 必须作为一个可审计的版本化更新，不能用生产自检的 MinGW 构建结果替换 MSVC ABI 包。若生成器或 CMake package 无法在 Windows runner 复现，可撤回 workflow 变更并保持 task in-progress；不调整 CMake ABI 策略来迎合不匹配 SDK。

## 决策与工作记录

- 2026-09-24：根据 GitHub Actions run `35992062001` 登记 CI 修复；发现 Windows 归档和自检的编译器配置均未满足 manifest 声明的 MSVC ABI，lint 编译数据库收录了尚未生成 moc 的排除构建目标。
- 2026-09-24：维护者要求整体审查 `tests/src/main.rs`，相关维护整理另登记为任务 076，避免将 runner 清理伪装成 CI 根因修复。
- 2026-09-24：修复 SDK producer workflow 的 Windows 生成器与 Release 安装配置；Qt benchmark moc 在 clang-tidy 与 include-cleaner 前生成。macOS 聚合 lint 和 workspace tests 全部通过；Windows Release 资产尚未重产，不宣称 Windows consumer CI 已修复。

## 完成摘要

代码和本地主机可复现问题已修复，GitHub workflow actionlint、`cargo lint --check`（含两个 clang 相关阶段）以及 `cargo test --locked --workspace` 通过。任务保持 in-progress：Windows producer workflow 尚未在 Windows runner 运行，旧 Release 归档与 manifest SHA 仍未替换，需产出 MSVC x64 Release 归档并完成 consumer CI 后关闭。
