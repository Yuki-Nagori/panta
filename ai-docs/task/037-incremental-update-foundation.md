# 037 — 软件内增量更新基础

- 状态：planned
- 阶段：交付基础
- 依赖：[005](005-qt-qml-shell.md)、[008](008-tasks-errors-logging.md)、[013](013-desktop-deployment-smoke.md)、[023](023-cross-platform-paths.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-17 / 2026-09-17

## 目标与背景

为打包后的 Panta 建立应用内增量更新能力：客户端读取签名 manifest，按平台和架构下载完整包或 delta，校验后在独立 staging 中安装，主程序退出后由 helper 原子切换并在健康检查失败时自动回滚。更新器不能覆盖用户工程和设置，也不能把未验证的包交给 Qt/CMake 运行时。

## 必读

- [软件内增量更新模块](../modules/incremental-updates.md)
- [桌面安装布局与部署冒烟](013-desktop-deployment-smoke.md)
- [跨平台路径与资源引用服务](023-cross-platform-paths.md)
- [后台任务、错误与日志基础](008-tasks-errors-logging.md)
- [依赖获取与预编译规则](../standards/dependency-acquisition.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

范围：冻结更新 manifest/schema、签名与密钥轮换规则、文件/内容块级增量包、Rust helper 的下载/校验/staging/原子切换/回滚状态机、C++ UpdateCoordinator 与 QML 状态展示，以及三平台打包制品和失败路径测试。

非目标：第一版不实现 bsdiff 等二进制差分、不静默重启、不更新用户工程数据、不提供任意安装脚本或远程执行能力、不替代系统级 MDM/软件仓库；灰度发布、遥测和复杂渠道策略在稳定基础上另立任务。

## 前置条件与待决策

013 必须先确定各平台安装布局和签名/公证产物；023 提供逻辑路径根；008 提供任务取消、错误和日志生命周期。实施前需冻结 manifest canonicalization、Ed25519 crate/版本、HTTP 客户端、压缩格式、块大小、版本保留数、清单托管和证书/公钥注入方式，并记录在依赖规范。

## 实施步骤

1. 以模块文档为基线编写协议 schema、威胁模型、状态转换和兼容/降级规则；为 manifest、包和状态文件定义版本字段与大小上限。
2. 在 Rust 中实现纯校验与事务核心：签名/哈希验证、路径穿越拒绝、断点块下载、空间检查、staging、fsync、原子 current/previous 切换和恢复；helper 通过版本化本地协议被主程序调用。
3. 接入 C++ UpdateCoordinator 和 ViewModel，提供检查、下载、取消、重启、失败与回滚状态；QML 只消费属性和信号，不直接触碰网络或文件系统。
4. 将 013 的 macOS/Windows/Linux 预编译打包产物转换为完整包与 delta，发布签名 manifest、SBOM/许可证，并在 CI 验证平台/架构矩阵。
5. 测试正常升级、无变化复用、断点续传、篡改/过期/降级、错误签名、路径穿越、磁盘不足、断电/进程终止、并发 helper、启动失败回滚和用户数据隔离；记录覆盖率与未覆盖平台。

## 预计改动

预计新增 `crates/panta-update-core/` 与 `crates/panta-updater/`、native UpdateCoordinator/ViewModel、QML 更新页面、打包/发布 workflow、manifest schema/测试 fixtures 和 `ai-docs/standards/` 的更新协议规范。实际目录以 013/023 的落地产物为准；不得提前提交生成包、签名私钥或平台个人路径。

## 清理与兼容例外

不保留旧更新协议的静默兼容分支。manifest `schema`、helper 协议和状态文件版本不匹配时明确拒绝并保留 current；若未来必须兼容，使用 `COMPAT(...)` 登记清理任务和删除版本。

## 验收标准

- [ ] manifest 签名覆盖版本、平台、架构、下载地址、文件/块哈希和策略；错误签名、过期、降级、跨渠道及未知 schema 均拒绝。
- [ ] 下载和解包只写 staging/缓存，拒绝绝对路径、`..`、重复/未声明条目和资源上限突破；用户数据目录不受更新影响。
- [ ] 主程序运行时不替换自身；退出后 helper 完成原子切换，健康检查失败自动回滚至上一健康版本。
- [ ] 完整包与文件/块级 delta 结果一致；中断可恢复或安全清理，磁盘不足/断电不破坏 current。
- [ ] C++/QML UI 只通过 ViewModel 访问更新服务，取消、重启、失败与回滚状态可观察且可记录。
- [ ] macOS/Linux/Windows 的打包、签名/公证、运行时加载和回滚证据齐全；所有平台差异有明确记录。
- [ ] 测试、覆盖率、格式、lint、依赖审计和文档索引同步；未实现能力不标记为已完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 方案设计与任务拆分 | 已完成模块文档与任务登记；未实现代码 |
| — | Rust helper 单元/集成测试与故障注入 | 待实施 |
| — | 三平台打包、签名、升级/回滚冒烟 | 待 013 与本任务实施 |

## 风险与回退

签名密钥泄露、错误的安装路径或不完整的断电恢复会使更新不可启动；客户端必须先验证 manifest 和 staging，再切换并保留 previous。发布端无法提供匹配平台/架构的预编译包时，回退为完整包或暂不发布，不在用户机器源码构建。回退仅切回 previous/current，不删除用户数据。

## 决策与工作记录

- 2026-09-17：新增增量更新模块规划。决定采用签名 canonical manifest、文件/内容块级增量、独立 helper、staging + 原子切换 + 健康检查回滚；二进制差分和灰度发布后置。

## 完成摘要

未完成。待 013/023 产物可用、协议与 helper/UI/打包测试全部有证据后再标记 done。
