# 038 — Native SDK 制品生产与发布

- 状态：done
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
| 2026-10-25 | Panta 分支迁移与本地验证 | 已移除三条 SDK producer workflow 与 `tools/sdk/`，并把 12 个消费资产的 URL/SHA256 切换到外部 Releases。删除空 `tools/` 后同步修复格式/lint CMake 扫描根。`cargo test-scope l1` 通过（Rust tests 与 CTest 91/91）；`cargo format --check`、`cargo lint cmake --check`、`cargo lint --changed clippy` 均通过。 |
| 2026-10-25 | Panta 旧 SDK Releases 清理 | 三个外部资产切换后的旧 Panta Releases 与同名 Git tags 已删除；远端 tag 查询确认均不存在。当前 manifest 只引用 `Jack-Lab-at-EIT/sdk-packaging`。 |
| 2026-10-25 | Ubuntu 24.04 基线兼容性重产 | sdk-packaging PR [#1](https://github.com/Jack-Lab-at-EIT/sdk-packaging/pull/1) 合并后，VTK run [40164985126](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164985126)、OCCT/Netgen run [40164987110](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164987110)、GoogleTest run [40164986409](https://github.com/Jack-Lab-at-EIT/sdk-packaging/actions/runs/40164986409) 全部通过三平台构建、自检与 Release 发布。新归档 `.sha256` sidecar 的内容与 Release API archive digest 一致；Panta 12 项 SHA 已同步。 |
| 2026-10-25 | Panta consumer smoke 与 PR CI | macOS arm64 `cargo verify-changed` 下载新 Release 资产并通过 SHA、Rust workspace 测试、native/QML CTest（91/91）、格式及适用 lint。Panta PR/nightly/package 的 Linux runner 已固定 Ubuntu 24.04；cargo 工具成品缓存 key 隔离 Linux 发行版版本。Panta PR [#91](https://github.com/Yuki-Nagori/panta/pull/91) 的 CI run [40169893498](https://github.com/Yuki-Nagori/panta/actions/runs/40169893498) 在 Ubuntu 24.04 上全部通过，PR 已合并（合并提交 `557deec`）。 |

## 决策记录

- 2026-10-25：SDK producer 的唯一维护位置改为 `Jack-Lab-at-EIT/sdk-packaging`；Panta 不再复制生产脚本或 Release 说明。三个 SDK 的 Ubuntu 24.04 重产、全部 12 个资产校验、消费 manifest 更新和 Panta PR #91 的 Ubuntu 24.04 CI 均已通过并合并。

## 完成摘要

三平台 SDK 生产与发布由外部 `sdk-packaging` 维护；Panta 消费 manifest 已切换到最新 Release，SHA 校验、本地消费验证和 Ubuntu 24.04 PR CI 均通过。任务验收完成。
