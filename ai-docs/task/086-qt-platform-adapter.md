# 086 — Qt 平台服务适配层

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[002](002-dependency-baseline.md)、[005](005-qt-qml-shell.md)、[023](023-cross-platform-paths.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-29 / 2026-09-29

## 目标与背景

跨平台系统访问目前与 QML Bridge / ViewModel 混放。建立独立的 `native/qt-adapter` C++ 层，承载 Qt 的平台 API 和操作系统差异，让 Bridge 负责稳定的 QML 属性/调用包装，Rust FFI 仍由 Bridge 负责 DTO 与服务调用。

本次审计确认并迁移两类平台服务：

- 用户偏好：macOS NSWorkspace、Windows 系统动画选项、Linux XDG Desktop Portal reduced-motion 设置。平台实现归新层；Bridge 仅保留 QML 可见的 reduced-motion 属性代理，供任务 080 后续接入页签动画。
- 标准目录：将 `QStandardPaths` 查询收敛到新层；Bridge 的 `PathHost` 继续负责映射 Rust 路径根类别、UTF-8 校验并调用 Rust 服务，`ProjectViewModel` 继续负责工程 UI 状态。

已审计但不迁入：`native/visualization/src/vtk/vtk_native_surface.*` 将 Qt Quick 原生 surface 与 VTK hardware window / Wayland 子 surface 一起管理。拆出 Qt 部分会让适配层依赖 VTK 或切断当前窗口资源所有权，因此继续归 Visualization。`QQuickImageProvider` 是应用图标资源服务；`QUrl` 转换是 Bridge 与 Rust DTO 的类型适配；两者都不属于系统平台服务。普通 ViewModel、QML 控件和 Qt 事件循环集成按当前职责保留。

## 必读

- [跨平台路径服务](023-cross-platform-paths.md)、[视口文档页签与 STL 按需激活](080-qml-viewport-document-tabs.md)
- [Qt / Rust 边界](../architecture/native-domain-boundaries.md)、[分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)、[仓库文件规范](../standards/repository-hygiene.md)、[验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 创建 `native/qt-adapter` 独立 CMake target；不依赖 Bridge、Rust FFI、VTK 或 QML 业务层。
- 将 reduced-motion 平台查询与运行期变化监听从 Bridge 移入 adapter，保持 Bridge/QML 侧统一的只读属性。
- 将标准目录查找集中到 adapter，并迁移 PathHost 与 ProjectViewModel 的 `QStandardPaths` 直接调用。
- 审计结果和分层边界写入规范；适配层仅添加真实调用方所需接口，不为将来预建空服务。

不包含：VTK 原生 surface 重构、QSettings 持久化设置服务（由 048 规划）、工程路径业务规则或 080 的 QML 动画接线/行为验收。

## 前置条件与待决策

- 002 / 005 / 023 已完成，Qt 与 Rust 路径服务均有实际消费端。
- 核对三平台平台 API 的初始读取和变化通知语义；未知或无法读取时沿用 080 记录的明确 fallback。
- 新 target 只使用仓库已供给的 Qt 模块及各目标平台为实现所需的系统框架/库。

## 实施步骤

1. 盘点 native、Bridge、app 与 visualization 中直接调用系统 API 或 Qt 平台 API 的位置，记录迁移/保留判据。
2. 创建独立 `panta::qt_adapter` target，将系统偏好后端和 `QStandardPaths` 查询移入。
3. 让 Bridge 的 SystemPreferences / PathHost 与 ProjectViewModel 只依赖 adapter 的公开值语义 API，不泄漏 AppKit、Win32、DBus 或 Qt 标准目录实现。
4. 清理 Bridge 内的 OS 宏、平台头、平台框架/组件链接和失效源文件；按文档记录审计中保留在其他层的 Qt/平台代码。
5. 完成构建与格式检查，更新 023、080、分层规范及本任务验证记录；同步索引状态。

## 预计改动

- 新增 `native/qt-adapter/CMakeLists.txt` 与公开头、平台实现源。
- `native/CMakeLists.txt`、`native/bridge/CMakeLists.txt`：target 接入及依赖边界。
- `native/bridge/src/path_host.*`、`project_view_model.*`、`system_preferences.*`：改为消费 adapter API。
- `ai-docs/standards/layering.md`、任务 023 / 080 / 085：同步边界、迁移位置与先决关系。

## 清理与兼容例外

从 Bridge 清除平台专属查询、通知实现、OS 条件编译、AppKit / DBus 链接，以及已移入 adapter 的标准目录查询。VTK surface 实现不是本次废弃项。无兼容例外。

## 验收标准

- [ ] `native/qt-adapter` 可独立构建，target 不依赖 Bridge、CXX / Rust、VTK 或 QML 业务 target。
- [ ] reduced-motion 的三平台读取和运行期变化通知只在 adapter 实现；Bridge 只代理稳定布尔属性。
- [ ] PathHost 与 ProjectViewModel 不再直接查询 `QStandardPaths`；所有实际消费的标准目录经 adapter API 查询，路径映射/错误语义不变。
- [ ] Bridge 不残留 AppKit / Win32 / DBus 平台实现与专属链接；Windows、macOS、Linux 构建分支归 adapter target。
- [ ] 审计中保留的 Qt/OS 集成均有层级理由，尤其 VTK native surface 仍由 Visualization 持有。
- [ ] 适用构建及格式入口成功，具体环境和结果记录在本任务；080 / 023 / 分层文档与索引一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待实施 | CMake/Cargo 构建与格式入口按仓库要求通过 | 未执行 |

## 风险与回退

系统偏好读取与 Qt Quick 的运行线程有关，adapter 需在 GUI 线程创建并监听；跨线程调用没有需要引入的路径。Linux DBus 不可用、系统选项未知时按已登记默认值处理。回退时可恢复 adapter 前的 Bridge 查询，但不保留两套并行实现。

## 决策与工作记录

- 2026-09-29：审计发现 QStandardPaths 查询散落在 PathHost 与 ProjectViewModel；系统 reduced-motion 实现已在工作树中为 080 开发，后续移入本层。VTK native surface 因 VTK 窗口所有权耦合而保留在 Visualization。

## 完成摘要

未开始。建立适配层并完成平台服务迁移后填写构建证据及实际保留边界。
