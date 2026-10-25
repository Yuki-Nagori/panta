# 038 — Native SDK 制品生产与发布

- 状态：in-progress
- 阶段：交付基础
- 依赖：[031](031-prebuilt-native-dependencies.md)、[020](020-toolchain-provisioning.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-17 / 2026-10-25

## 目标与边界

SDK 的生产、打包、自检和 Release 发布由独立仓库 [sdk-packaging](https://github.com/Jack-Lab-at-EIT/sdk-packaging) 负责。Panta 只保留消费端 manifest、下载校验、缓存与 imported target 接入；不重复维护 SDK producer workflow、CMake 构建描述、自检工程或制品说明副本。

本任务负责清理 Panta 内重复的生产实现，并在外部仓库提供完整可用资产后，将消费端 URL 与 SHA256 切换到新 Release。切换前不得移除仍在工作的旧资产引用。

## 验收标准

- [x] Panta 删除 SDK producer workflows 和 `tools/sdk/` 生产/自检实现；消费端 `native/cmake/sdk-provision.cmake` 保持独立。
- [x] 外部仓库的 VTK、OCCT/Netgen、GoogleTest Release 均具备消费端所需平台资产、sidecar、license 与 package metadata；12 个归档的 sidecar 与 Release API digest 一致。
- [x] Panta manifest 在外部资产完整发布并核对摘要后更新 URL / SHA256；消费验证通过后删除 Panta 中重复的 SDK Releases 与 Git tags。
- [x] 受影响的 SDK provision、native 构建与测试通过；依赖说明、任务 031/070/071 和索引指向外部生产源，职责边界一致。

## 验证计划与结果

| 日期 | 环境 / 场景 | 结果 |
|---|---|---|
| 2026-10-25 | `sdk-packaging` Releases | VTK run [40123093281](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40123093281) 三平台生产、自检、打包和发布成功；OCCT/Netgen 与 GoogleTest Releases 也已发布。12 个归档 SHA256 均与下载 sidecar 及 Release API digest 一致。 |
| 2026-10-25 | Panta 分支迁移与本地验证 | 已移除三条 SDK producer workflow 与 `tools/sdk/`，并把 12 个消费资产的 URL/SHA256 切换到外部 Releases。删除空 `tools/` 后同步修复格式/lint CMake 扫描根。`cargo test-scope l1` 通过（Rust tests 与 CTest 91/91）；`cargo format --check`、`cargo lint cmake --check`、`cargo lint --changed clippy` 均通过。PR/CI 待完成。 |
| 2026-10-25 | Panta 旧 SDK Releases 清理 | 三个外部资产切换后的旧 Panta Releases 与同名 Git tags 已删除；远端 tag 查询确认均不存在。当前 manifest 只引用 `Jack-Lab-at-EIT/sdk-packaging`。 |
| 2026-10-25 | Ubuntu 24.04 基线兼容性重产 | sdk-packaging PR [#1](https://github.com/Jack-Lab-at-EIT/sdk-packaging/pull/1) 合并后，VTK run [40164985126](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164985126)、OCCT/Netgen run [40164987110](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164987110)、GoogleTest run [40164986409](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164986409) 全部通过三平台构建、自检与 Release 发布。新归档 `.sha256` sidecar 的内容与 Release API archive digest 一致；Panta 12 项 SHA 已同步，消费端验证待 CI。 |
| 2026-10-25 | Panta macOS consumer smoke | `cargo verify-changed` 下载新 Release 资产并通过 SHA、Rust workspace 测试、native/QML CTest（91/91）、格式及适用 lint。Linux consumer CI 尚待 Panta 分支验证。 |

## 决策记录

- 2026-10-25：SDK producer 的唯一维护位置改为 `Jack-Lab-at-EIT/sdk-packaging`；Panta 不再复制生产脚本或 Release 说明。VTK Windows 资产发布并核验完成后，再完成消费端切换及 PR。
