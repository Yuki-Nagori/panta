# 086 — Qt 平台服务适配层

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[002](002-dependency-baseline.md)、[005](005-qt-qml-shell.md)、[023](023-cross-platform-paths.md)、[034](034-rust-panta-artifact-parser.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-29 / 2026-09-29

## 目标与背景

跨平台系统访问目前与 QML Bridge / ViewModel 混放。建立独立的 `native/qt-adapter` C++ 层，承载 Qt 的平台 API 和操作系统差异，让 Bridge 负责稳定的 QML 属性/调用包装，Rust FFI 仍由 Bridge 负责 DTO 与服务调用。

首次实现后，GitHub Actions 在 Linux 和 Windows 编译适配层时暴露出未覆盖的分支错误：Linux D-Bus 信号注册使用了 const 连接对象；Windows 源文件在 Windows SDK 伞头之前直接包含拆分头，clang-cl 下缺少 SDK 所需的头文件上下文。本次修复保持既有平台行为，只恢复两平台分支的编译。

本次审计确认并迁移两类平台服务：

- 用户偏好：macOS NSWorkspace、Windows 系统动画选项、Linux XDG Desktop Portal reduced-motion 设置。平台实现归新层；Bridge 仅保留 QML 可见的 reduced-motion 属性代理，根窗口已将其绑定到页签组件，任务 080 继续行为验收。
- 标准目录：将 `QStandardPaths` 查询收敛到新层；Bridge 的 `PathHost` 继续负责映射 Rust 路径根类别、UTF-8 校验并调用 Rust 服务，`ProjectViewModel` 继续负责工程 UI 状态。
- 翻译运行时：Qt adapter 只负责 `QTranslator` 安装/替换/卸载；Rust core 拥有支持 locale 与当前选择的领域状态，CXX 传递 locale 字符串，不把 Qt 类型或资源 URL 引入 Rust。Bridge `Settings` 仅协调 Rust locale 校验、Qt QM 加载和 QML 通知。

已审计但不迁入：`native/visualization/src/vtk/vtk_native_surface.*` 将 Qt Quick 原生 surface 与 VTK hardware window / Wayland 子 surface 一起管理。拆出 Qt 部分会让适配层依赖 VTK 或切断当前窗口资源所有权，因此继续归 Visualization。`QQuickImageProvider` 是应用图标资源服务；`QUrl` 转换是 Bridge 与 Rust DTO 的类型适配；两者都不属于系统平台服务。普通 ViewModel、QML 控件和 Qt 事件循环集成按当前职责保留。

## 必读

- [跨平台路径服务](023-cross-platform-paths.md)、[视口文档页签与 STL 按需激活](080-qml-viewport-document-tabs.md)
- [Qt / Rust 边界](../architecture/native-domain-boundaries.md)、[分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 创建 `native/qt-adapter` 独立 CMake target；不依赖 Bridge、Rust FFI、VTK 或 QML 业务层。其翻译服务只依赖 Qt Core，不承担 locale 选择策略。
- 将 reduced-motion 平台查询与运行期变化监听从 Bridge 移入 adapter，保持 Bridge/QML 侧统一的只读属性。
- 将标准目录查找集中到 adapter，并迁移 PathHost 与 ProjectViewModel 的 `QStandardPaths` 直接调用。
- 增加 Qt 翻译器运行时服务；在 Bridge 的 `Settings` 暴露当前 locale、Rust 提供的支持 locale 和切换命令，并在 QM 安装成功后触发 `QQmlEngine::retranslate()`。产品设置界面和偏好持久化另由 [087](087-language-settings-ui.md) 与 [048](048-settings-service-and-qt-adapter.md) 承接。
- 审计结果和分层边界写入规范；适配层仅添加真实调用方所需接口，不为将来预建空服务。

不包含：VTK 原生 surface 重构、QSettings 持久化设置服务（由 048 规划）、工程路径业务规则或 080 的动画行为验收。根窗口的 reduced-motion 属性绑定作为 adapter 的实际消费端一并接入。

## 前置条件与待决策

- 002 / 005 已完成；023 的 Rust 路径服务和 034 的 `.pa` → TS/QM 资源入口已有实际消费端，086 迁移的是这些现有 Qt 调用。
- 核对三平台平台 API 的初始读取和变化通知语义；未知或无法读取时沿用 080 记录的明确 fallback。
- 新 target 只使用仓库已供给的 Qt 模块及各目标平台为实现所需的系统框架/库。

## 实施步骤

1. 盘点 native、Bridge、app 与 visualization 中直接调用系统 API 或 Qt 平台 API 的位置，记录迁移/保留判据。
2. 创建独立 `panta::qt_adapter` target，将系统偏好、标准目录查询和通用 `QTranslator` 生命周期移入。
3. 在 Rust core 定义 locale 支持集、当前 locale 和候选校验；通过 CXX DTO/字符串传递，不引入 Qt 类型。Bridge `Settings` 编排 Rust 校验与 Qt translator 安装，并只在成功后提交 locale 状态。
4. 让 Bridge `Settings` / PathHost / ProjectViewModel 只依赖 adapter 的公开值语义 API，不泄漏 AppKit、Win32、DBus 或 Qt 标准目录实现。
5. 清理 Bridge 内的 OS 宏、平台头、平台框架/组件链接和失效源文件；按文档记录审计中保留在其他层的 Qt/平台代码。
6. 根据三平台 CI 编译结果修复平台条件编译分支，并验证 adapter 的 Linux D-Bus 与 Windows SDK 头文件入口。
7. 将 Settings 的 QML 行为用例按 `Qml.*` 注册为 QtTest，遵循任务 042 已登记的 sanitizer 边界，同时保留常规 CTest 验收。
8. 完成构建与格式检查，更新 022、023、080、分层规范及本任务验证记录；同步索引状态。

## 预计改动

- 新增 `native/qt-adapter/CMakeLists.txt` 与公开头、平台实现源；`crates/panta-core/src/language.rs` 保存 locale 领域状态。
- `native/CMakeLists.txt`、`native/bridge/CMakeLists.txt`：target 接入及依赖边界。
- `native/bridge/src/path_host.*`、`project_view_model.*`、`settings.*`：改为消费 adapter API 并协调 Rust locale 服务；不保留 `system_preferences.*` 旧命名。
- `ai-docs/standards/layering.md`、任务 023 / 080 / 085：同步边界、迁移位置与先决关系。

## 清理与兼容例外

从 Bridge 清除平台专属查询、通知实现、OS 条件编译、AppKit / DBus 链接，以及已移入 adapter 的标准目录查询。VTK surface 实现不是本次废弃项。无兼容例外。

## 验收标准

- [x] `native/qt-adapter` 在完整 native 构建中成功编译，target 不依赖 Bridge、CXX / Rust、VTK 或 QML 业务 target；翻译服务只暴露 Qt 运行时机制。
- [x] reduced-motion 的三平台读取和运行期变化通知只在 adapter 实现；Bridge 只代理稳定布尔属性。
- [x] locale 支持、校验和状态由 Rust 持有；Bridge 仅协调 Qt catalog resource 与 QTranslator，切换失败保留原 translator/locale，成功后请求 QML 翻译刷新。
- [x] PathHost 与 ProjectViewModel 不再直接查询 `QStandardPaths`；所有实际消费的标准目录经 adapter API 查询，路径映射/错误语义不变。
- [x] Bridge 不残留 AppKit / Win32 / DBus 平台实现与专属链接；Windows、macOS、Linux 构建分支归 adapter target。
- [x] 审计中保留的 Qt/OS 集成均有层级理由，尤其 VTK native surface 仍由 Visualization 持有。
- [x] macOS 适用构建及格式入口成功，具体环境和平台验证边界记录在本任务；080 / 023 / 分层文档与索引一致。
- [ ] Linux D-Bus 与 Windows 系统偏好分支在 CI 中编译通过，相关 lint/build job 不再被平台源文件编译错误阻断。
- [x] Settings 语言切换用例在常规 CTest 中通过，并以 `Qml.*` 命名供任务 042 的 TSan / Windows ASan 边界过滤。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-28 | GitHub Actions run [36458145882](https://github.com/Yuki-Nagori/panta/actions/runs/36458145882)，Linux/Windows 构建、sanitizer 与 lint | 三平台适配层均可编译 | 未通过：Linux `QDBusConnection::connect` 在 const 对象上调用；Windows `system_motion_preference.cpp` 在 Windows SDK 伞头前直接包含 `windef.h`，clang-cl 触发 `No Target Architecture`。多个 lint job 因相同 native 编译错误中止；本次修复针对这两处根因 |
| 2026-09-29 | GitHub Actions run [36502014437](https://github.com/Yuki-Nagori/panta/actions/runs/36502014437)，修复提交后的 Linux/Windows 构建、lint 与 sanitizer | 平台构建、include-cleaner 和 sanitizer 策略通过 | 普通 Linux/Windows 构建已通过；includes 阶段要求 Qt 环境变量、QStringLiteral、QDBusVariant 的源文件直接 include。Linux TSan 报告 Settings QML 测试中的 Qt6Core/Qt6Qml 线程栈竞态；Windows ASan 在 GTest POST_BUILD discovery 启动该 QML 测试程序时由 Qt6Core/Qt6QmlWorkerScript 报 bad-free。任务 042 已将 `Qml.*` 列为这两种 sanitizer 的已验证第三方边界；该用例此前命名为 `SettingsTest.*` 且构建期仍执行 GTest discovery，因此本次改为 QtTest `Qml.*` 注册 |
| 2026-09-29 | macOS arm64 / Qt 6.11.2，首次 `cargo build --locked` | 编译 Rust FFI、Qt adapter、Bridge 和 QML 模块 | 未通过：`QTranslator` 头文件不完整，且 QML 注册类型不可声明为 `final`；修正头文件依赖与类声明后重跑 |
| 2026-09-29 | macOS arm64，`cargo format`；`cargo build --locked` | 格式化通过；完整 Cargo/native 构建成功 | 通过。Rust locale service、CXX、`panta_qt_adapter`、Bridge `Settings` 和 QML `Settings` 注册均编译成功；Qt 6.11.2 编译并嵌入 en/zh-CN QM。未运行测试或真实窗口切换；Windows/Linux 本次为代码审阅，未在本机编译 |
| 2026-09-29 | macOS arm64 / Qt 6.11.2，`cargo test --locked --workspace` | Rust locale 规则、Qt translator 失败保留、QML 语言切换和既有回归通过 | 通过；Rust locale 新增 3 项测试，native CTest 66/66。`Settings` 测试覆盖 QML 翻译绑定从英文切到中文再切回、非法 locale 与不可往返 UTF-16 输入保留状态；adapter 测试覆盖 en→zh-CN、加载失败保留旧字典和卸载。该证据为无头 QML/Qt 测试，不等于真实窗口验收 |
| 2026-09-29 | 三次临时消融，随后恢复源文件并重跑 `cargo test --locked --workspace` | 测试应识别 locale 校验缺失、失败替换清除旧字典、缺少 `QQmlEngine::retranslate()` | 均按预期失败：Rust 接受 `zh_CN` 时 locale 用例失败；清除旧字典后 Qt adapter 用例失败；禁用重翻译后 QML 文本仍为 `Ready`，Settings 用例失败。消融代码均已恢复，最终聚合测试 66/66 通过 |
| 2026-09-29 | `cargo format`、`cargo build --locked`、`cargo lint` | 格式、构建及八阶段 lint 通过 | 全部通过；lint 覆盖 Clippy、依赖、CMake、qmllint、clang-tidy、include-cleaner、cppcheck。默认沙箱禁止 Cargo 锁用本地 TCP 监听，完整 lint 在授权环境执行。Windows/Linux 代码分支未在本机编译 |
| 2026-09-29 | 修复后 macOS arm64：`cargo format --check`、`cargo build --locked`、`cargo test --locked --workspace`、`cargo lint`、`git diff --check` | 当前平台构建、聚合回归、lint 与补丁格式通过 | 全部通过，native CTest 66/66。macOS 主机只编译 macOS 条件分支；Linux/Windows 的修复尚未由远端 CI 复跑确认，按用户约定保留为本地提交 |
| 2026-09-29 | macOS arm64：`cargo format --check`、`cargo test --locked --workspace`、`cargo lint`、`cargo sanitize`、`git diff --check` | QML 行为测试在普通回归中通过，lint 与平台适用 sanitizer 策略通过 | 全部通过；普通 CTest 66/66，包含 `Qml.SettingsLanguageSwitch`；cargo lint 八阶段通过；ASan/UBSan 66/66；TSan 50/50，按 Task 042 规则未运行 `Qml.*`。macOS 不验证 Windows ASan 行为，修改后的 Linux/Windows 条件编译仍待后续远端 CI 确认 |

## 风险与回退

系统偏好读取与 Qt Quick 的运行线程有关，adapter 需在 GUI 线程创建并监听；跨线程调用没有需要引入的路径。Linux DBus 不可用、系统选项未知时按已登记默认值处理。回退时可恢复 adapter 前的 Bridge 查询，但不保留两套并行实现。

## 决策与工作记录

- 2026-09-29：任务开始实施。系统 reduced-motion 的操作系统查询和通知从 Bridge 移到 `native/qt-adapter`；Bridge 保留 QML 属性代理。PathHost 根类别映射和工程服务调用不变，标准目录实际查询由 adapter 统一提供。
- 2026-09-29：完成 Qt/API 盘点：QStandardPaths 使用迁入本层；`vtk_native_surface.*` 因直接创建/同步 VTK hardware window 与 Wayland 子 surface 而留在 Visualization；QQuickImageProvider、QUrl 类型转换、ViewModel 信号/模型和 QML 控件保留原层。
- 2026-09-29：用户要求先接翻译后端并将 QML 类型命名为 `Settings`。locale 策略与状态归 Rust core，CXX 只传 locale DTO/字符串；Qt adapter 只管理 QTranslator。设置界面登记为独立 task 087，持久化依赖 048。
- 2026-09-29：按用户建议将 Bridge 类型及源文件统一命名为 `Settings` / `settings.*`，删除 `SystemPreferences` 命名，不加兼容别名。
- 2026-09-29：复查 adapter 依赖方向与 QML API：Rust 只持有 locale ID/规则/当前选择；Qt adapter 只拥有 `QTranslator`；Bridge `Settings` 负责 resource 命名、失败保留、状态提交及 `QQmlEngine::retranslate()`。当前 locale 不持久化，设置 UI 与 QSettings 恢复留给 087/048。
- 2026-09-29：`cargo format` 与 macOS arm64 `cargo build --locked` 通过；完成源层级审计与旧命名清理。平台 API 的 Windows/Linux 分支本次未本机编译，运行期语言切换未执行真窗口验证。
- 2026-09-29：按维护者要求复审暂存区，补 Rust locale 状态、Qt 翻译器失败保留及 QML 实际重翻译的行为测试；使用短暂禁用关键逻辑的消融运行确认测试能识别回归，之后恢复实现并运行 Cargo 聚合入口。
- 2026-09-29：评审发现 Qt translator 是应用级状态，而多个 QML `Settings` 实例会各自持有不同 locale；将 QML 类型注册为单例，根窗口直接绑定其属性。当前应用使用一个 QML 引擎。
- 2026-09-29：新增 Rust、Qt adapter 和 Bridge/QML 回归，修正直接头文件引用、`StandardLocation` 底层类型、CMake 格式及语言服务客户端命名。三次消融均证明对应测试能捕获行为退化；恢复实现后的 Cargo 聚合、build 和 lint 通过。
- 2026-09-29：GitHub Actions 首次三平台构建发现 Linux D-Bus 和 Windows SDK 分支编译错误；重新打开任务，修复后需等待下一次 CI 运行验证远端平台结果。
- 2026-09-29：run 36502014437 确认平台普通构建通过；include-cleaner 报漏直接头文件，Settings QML 测试因 GTest discovery 未归入 Task 042 的 `Qml.*` sanitizer 过滤。本次改用 QtTest/add_test 命名为 `Qml.SettingsLanguageSwitch`，并补齐 Linux 分支直接依赖头文件。

## 完成摘要

首次实施完成：新增独立 `panta::qt_adapter` 并迁移 reduced-motion、标准目录查询和 QTranslator 生命周期；Rust `LanguageService` 拥有 locale 目录/状态，Bridge `Settings` 作为 QML 单例暴露语言后端和系统偏好。macOS arm64 的 Cargo build/test/format/lint 通过，CTest 66/66，三次消融按预期失败后恢复并复测。2026-09-28 的 CI 发现 Linux/Windows 平台分支编译问题，2026-09-29 的 CI 又发现直接 include 和 QML sanitizer 分类问题，本任务重新打开并继续修复；远端复跑证据待补。设置 UI 与语言持久化分别由 087/048 跟进。
