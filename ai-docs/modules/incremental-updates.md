# 软件内增量更新

这是打包交付后的更新模块设计，目标是让已安装的 Panta 在应用内发现、下载并安全切换到新版本，同时保留可回退的上一版本。当前只有方案和任务安排；现有 `cargo build`、CMake install 树和 Qt 预编译供给不代表已经具备更新能力。

## 边界与原则

- 更新对象是版本化的程序和只读资源。用户工程、缓存、日志、许可证和个人设置属于数据面，由路径/运行时模块管理，更新器不得覆盖或删除。
- 运行中的主程序不自我替换。Qt UI 只负责状态展示和用户决策，独立的 updater helper 在主程序退出后完成替换，再启动新版本。
- 更新元数据、清单和包都必须可验证。TLS 只保护传输，不能代替签名；公钥以内置根信任和明确的轮换记录为准。
- 先并行安装到 staging/version 目录，再做一次原子切换；切换前保留 current，健康检查失败自动恢复 previous。下载中断、校验失败、磁盘不足和重启不会破坏当前版本。
- 增量优先采用文件/内容块级复用：未变化文件从当前版本复用，变化文件下载压缩块；没有可用 delta 时回退到同一签名清单描述的完整包。二进制差分（bsdiff 等）不作为第一版协议，须有体积和恢复性证据后另立任务。

## 运行时组件

```text
Qt/QML UpdateViewModel
        │ 状态、进度、重启请求
        ▼
UpdateCoordinator（C++ service；生命周期、并发与错误）
        │ 版本化本地协议 / CXX 边界
        ▼
panta-updater helper（Rust；清单、签名、下载、staging、切换、回滚）
        │ HTTPS/代理/断点续传
        ▼
签名 manifest + 内容寻址包存储
```

Rust core 只处理确定性的 manifest、哈希、路径安全、状态机和文件事务；网络实现、平台进程启动和 Qt 展示分别通过窄接口注入。QML 不直接访问文件、网络或安装目录。若 helper 与主程序版本不兼容，helper 先拒绝更新并保留 current。

## 发布清单与包格式

每个平台、架构和渠道发布一个签名 manifest，采用固定字段顺序的 canonical JSON 后进行 Ed25519 detached signature。第一版字段至少包括：`schema`、`product`、`channel`、`version`、单调递增的 `sequence`、`platform`、`arch`、`min_runtime`、`published_at`、`expires_at`、完整包摘要、可选 delta 基线版本、文件/块路径、大小、SHA-256 和 `key_id`。清单签名覆盖所有下载地址和哈希，防止只替换 CDN URL。

包内路径必须是相对路径，拒绝绝对路径、盘符、`..`、未声明的符号链接和重复条目；解包大小、文件数量、压缩比、单文件大小和总下载量有上限。临时下载使用内容寻址目录与校验后改名，重复块可复用，失败不会覆盖已验证块。发布端生成完整包、delta 包、SBOM/许可证清单和签名，客户端只消费已登记平台与架构。

信任根随应用发布，`key_id` 轮换需要旧根签名的新公钥和生效序号；客户端拒绝过期、降级、跨渠道或低于 `min_runtime` 的 manifest。密钥、token 和私有发布凭据不进入仓库或客户端日志。

## 状态机与事务

`idle → checking → available → downloading → staged → awaiting-restart → applying → health-check → active`。任何阶段的可恢复错误进入 `failed` 并保留 current；应用退出或 helper 超时后可从 staging 继续或清理。状态文件写入临时文件并 fsync 后原子改名，记录 manifest 摘要、current、previous、helper 版本和失败原因，防止断电后产生“已切换但无记录”的状态。

安装布局由 023/013 的平台适配器提供逻辑根，不把 `/Applications`、`Program Files` 等路径写进协议。布局语义为：版本目录只读、current 指针或平台等价的启动选择、previous 保留至少一个健康版本、用户数据目录独立。macOS 需在切换后维持 app bundle 签名/公证；Windows 需由未锁定的 helper 替换并保持 MSVC/Qt 运行时；Linux 的 AppImage 或发行包布局在 013 决策后接入同一协议。

健康检查至少验证新 helper 可启动、版本/架构匹配、Qt/QML 模块可加载和最小 native 自检通过。启动失败、崩溃循环或自检超时触发一次自动回滚；用户工程迁移若失败，只回滚程序槽位，不回滚用户数据，迁移必须有独立版本与备份策略。

## 数据流与用户体验

用户主动检查或按策略定时检查 manifest；默认不在后台静默重启。UI 展示当前版本、目标版本、包大小、已下载量、校验/签名状态、重启需求和失败原因；下载可取消、可暂停并在下次继续。强制安全更新由 manifest 策略标记，但仍需先完成签名、空间和回滚前置检查。离线、代理、限速和 CDN 暂时不可用时保留当前版本并提供可复核诊断。

## 任务安排

任务 [037](../task/037-incremental-update-foundation.md) 先冻结协议、威胁模型和状态机，再按依赖推进：打包任务 013 确定三平台安装布局后，落地发布制品与签名；随后实现 Rust helper 的 staging/切换/回滚，再接入 C++ ViewModel 和 QML 更新界面。质量任务需覆盖篡改、断点、断电、磁盘不足、路径穿越、降级、并发启动和回滚；差分压缩、灰度发布和遥测只有在第一版稳定后另行拆任务。

## 未决决策

- 首发渠道和清单托管位置（项目 release、对象存储或企业镜像）及缓存/代理策略。
- macOS notarization、Windows 签名证书与 Linux 首发包格式，需与 013 的实际打包产物一起冻结。
- 文件块大小、压缩算法、并行度和保留版本数，需以真实安装包大小、磁盘预算和恢复测试确定。
- 签名库、canonical JSON 实现、HTTP 客户端和 Rust/C++ bridge 的固定版本与许可证，实施前登记到依赖规范。
