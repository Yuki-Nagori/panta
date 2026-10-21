# 125 — 设置服务与 Qt 持久化适配

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[006](006-rust-cpp-boundary.md)、[023](023-cross-platform-paths.md)、[030](030-theme-dsl.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-20 / 2026-10-21

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
- 已冻结 schema v1：QSettings NativeFormat，`preferences/snapshot` 保存整个 JSON 快照为单一值；测试通过注入 IniFormat 文件。单值写入避免多字段在 Windows 注册表后端形成非事务更新。
- 持久化主题 ID、语言、窗口逻辑宽高与最大化状态、最近 10 个工程路径。窗口不保存屏幕坐标，工程内容与视口会话状态不进入用户偏好。
- 默认值为 `light`、`en`、1280×720、最大化、空最近列表；窗口尺寸范围为 1280..16384 × 720..16384；当前主题 ID 必须是内置 `light` 或 `dark`。
- 不支持未知 schema 版本或损坏/多余字段：回退内存默认值，显示稳定诊断，并禁用本次运行中的持久化，避免覆盖原文件。无旧格式迁移。

## 实施步骤

1. 冻结设置 schema、默认值、范围、版本和错误码，Rust 单测覆盖非法值与未知版本。
2. 经 CXX 暴露明确 DTO/命令；C++ adapter 将 DTO 映射到 QSettings，并在 GUI 线程
   完成读取、写入和变更通知。
3. 复用 030 提供的 `qml/Themes/*.pa` 资源与 `ThemeViewModel`；本任务只将选中的
   主题 ID 持久化到 QSettings，并在启动时恢复选择，不重复加载或实现主题解析。
   读取损坏或缺失的主题 ID 时回退到默认主题并报告诊断。
4. 覆盖启动恢复、写入失败保留上一份有效值、版本拒绝和 QML 绑定更新；平台存储行为由 QSettings NativeFormat 负责，整份快照只写入单个值。

## 预计改动

预计新增 `crates/panta-core` 或设置专用 Rust service、`crates/panta-ffi` 设置 DTO、
`native/bridge` Qt adapter/测试，以及用户设置测试夹具；主题资源由 030 提供于
`qml/Themes/`，本任务不重复新增主题资源；
具体路径以实施时现有模块为准，不提前创建空模块。

## 清理与兼容例外

无废弃项；默认无兼容例外。若确需迁移旧 key，必须登记 `COMPAT(125; remove-task NNN)`
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
| 2026-10-21 | macOS 本机；`cargo verify-changed`、`cargo format --changed` | 设置 schema、存储、桥接、QML 恢复及适用质量检查通过 | Rust workspace 与 CTest 91/91 通过；clang-tidy 与 include-cleaner 扫描 63 个 native 翻译单元通过；Windows/Linux NativeFormat 和真实窗口恢复仍待验收 |

## 风险与回退

QSettings 的平台路径、格式和同步时机可能不同；adapter 只暴露统一 DTO，失败时
保留内存中的最后有效快照。设置损坏不覆盖原文件，无法恢复时回退内置默认值并给出
用户可理解的诊断。

## 决策与工作记录

- 2026-09-20：确认 QSettings 属于 Qt 平台存取层，Rust 负责 schema/校验/业务设置，
  不把 Qt 依赖倒灌到 `panta-core`；主题定义与主题选择分别由 `.pa` 和设置服务管理。
- 2026-10-17：因任务索引中编号 048 已用于性能测试体系，本任务改用新编号 125。应用菜单入口由 115 提供；具体设置页面由消费任务提供，语言设置 UI（087）通过该入口访问设置服务。
- 2026-10-21：确定首版使用 QSettings NativeFormat，并仅保存用户偏好快照；设置 schema 为 v1，不实现迁移。窗口只恢复逻辑尺寸与最大化状态；屏幕位置由后续多屏任务处理。无效或未知设置不覆盖原存储。
- 2026-10-17：独立只读评审确认 125 → 115 → 087 依赖无环、任务职责和文档链接一致；保留 125 对 planned 030、115 对 deferred 051 的既有前置条件。
- 2026-10-21：单值 JSON 快照避免 Windows 注册表后端多 key 更新的非事务状态；跨进程测试覆盖空、单项和多项最近工程路径。include-cleaner 规则补充声明头/完整类型头的区分及自动修复后的编译验证，不新增全局忽略项。
- 2026-10-21：独立只读 final review 未发现阻塞问题，检查完整 staged/unstaged diff、分层、存储失败路径、测试与文档。未覆盖 Windows/Linux NativeFormat、真实窗口恢复和断电持久性；这些边界继续由平台验收补齐。

## 完成摘要

未完成。
