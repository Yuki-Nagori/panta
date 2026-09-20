# 048 — 设置服务与 Qt 持久化适配

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[006](006-rust-cpp-boundary.md)、[023](023-cross-platform-paths.md)、[030](030-theme-dsl.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-20 / 2026-09-20

## 目标与背景

建立跨平台设置边界，区分应用偏好、主题选择、窗口状态和工程数据。Qt 的
`QSettings` 负责平台原生存取适配；Rust 负责设置 schema、默认值、校验和
需要跨语言共享的逻辑，不让 `panta-core` 依赖 Qt。主题定义仍由 `.pa` 提供，
本任务只保存主题 ID/选择状态，不把完整主题文本塞进 QSettings。

## 必读

- [应用平台与工程存储](../architecture/application-and-storage.md)
- [分层与依赖方向](../standards/layering.md)
- [Rust 规范](../standards/rust.md)、[Qt 规范](../standards/qt.md)
- [CXX 边界](../standards/cxx.md)、[验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)
- [QSettings](https://doc.qt.io/qt-6/qsettings.html)、[Qt Resource System](https://doc.qt.io/qt-6/resources.html)，查阅日期：2026-09-20

## 范围与非目标

范围：设置 schema 与版本、Rust 默认值和校验、C++ Qt/QSettings adapter、
主题 ID/窗口状态/最近路径等用户偏好、原子写入失败处理、启动恢复和测试。

非目标：工程文件格式、Study/Job 数据、QML 直接访问 QSettings、把 Qt 类型带入
Rust core、每帧视口状态持久化、无依据的旧版本自动迁移。

## 前置条件与待决策

- 030 需要冻结主题 ID、`.pa` 基线/覆盖合并和 ThemeViewModel 输入快照。
- 需要决定设置 key 的命名空间、版本升级策略及 QSettings 的 NativeFormat/IniFormat
  选择；未验证前不把平台文件路径写入公共契约。
- 需要定义哪些设置属于用户偏好，哪些属于工程模型或临时会话状态。

## 实施步骤

1. 冻结设置 schema、默认值、范围、版本和错误码，Rust 单测覆盖非法值与未知版本。
2. 经 CXX 暴露明确 DTO/命令；C++ adapter 将 DTO 映射到 QSettings，并在 GUI 线程
   完成读取、写入和变更通知。
3. 将内置 `.pa` 主题作为 Qt resource 只读加载，QSettings 只保存选中的主题 ID
   和用户偏好；读取损坏或缺失时回退默认快照并报告诊断。
4. 覆盖三平台启动恢复、写入失败保留上一份有效值、版本拒绝和 QML 绑定更新。

## 预计改动

预计新增 `crates/panta-core` 或设置专用 Rust service、`crates/panta-ffi` 设置 DTO、
`native/bridge` Qt adapter/测试，以及 `resources/themes/` 和用户设置测试夹具；
具体路径以实施时现有模块为准，不提前创建空模块。

## 清理与兼容例外

无废弃项；默认无兼容例外。若确需迁移旧 key，必须登记 `COMPAT(048; remove-task NNN)`
并保留可验证的删除条件，不在 QML 或 C++ 中长期维护双 key。

## 验收标准

- [ ] `panta-core`/Rust 设置逻辑无 Qt 依赖，QSettings 只存在 C++ adapter。
- [ ] 主题 ID、窗口状态和最近路径可跨三端保存/恢复；工程数据不混入用户偏好。
- [ ] 未知版本、非法值、读写失败均有稳定诊断，并保留上一份有效状态。
- [ ] QML 只经 ViewModel 读写，主题切换不重建 QML engine，不产生工程 dirty。
- [ ] 代码、测试、文档、task 和索引一致；无未登记兼容分支。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待实施 | 三平台设置读写与失败矩阵 | 未执行 |

## 风险与回退

QSettings 的平台路径、格式和同步时机可能不同；adapter 只暴露统一 DTO，失败时
保留内存中的最后有效快照。设置损坏不覆盖原文件，无法恢复时回退内置默认值并给出
用户可理解的诊断。

## 决策与工作记录

- 2026-09-20：确认 QSettings 属于 Qt 平台存取层，Rust 负责 schema/校验/业务设置，
  不把 Qt 依赖倒灌到 `panta-core`；主题定义与主题选择分别由 `.pa` 和设置服务管理。

## 完成摘要

未完成。
