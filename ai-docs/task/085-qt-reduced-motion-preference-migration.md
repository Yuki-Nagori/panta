# 085 — Qt reduced-motion 原生属性迁移

- 状态：planned
- 阶段：应用平台扩展
- 依赖：[080](080-qml-viewport-document-tabs.md)、[002](002-dependency-baseline.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-28 / 2026-09-28

## 目标与背景

当前 Qt 基线没有统一的减少动态效果属性，因此 080 在 `native/qt-adapter` 读取 macOS、Windows 与 Linux 平台偏好，并向 QML 暴露稳定的 `SystemPreferences.reducedMotion`。维护者指出 Qt 6.12 提供 `QStyleHints::motionPreference`。在仓库 Qt 基线升级到包含该 API 的版本时，切换到 Qt 原生属性，避免继续维护平台专属查询和通知。

API 名称、引入版本、取值语义、变更通知以及三平台实际支持范围均须在开始实施时重新核对 Qt 官方文档和当前工具链；本任务不要求提前升级 Qt，也不把 Qt 6.12 视作当前基线。

## 必读

- [平台、工具链与 native 依赖基线](002-dependency-baseline.md)、[视口文档页签与 STL 按需激活](080-qml-viewport-document-tabs.md)
- [分层规则](../standards/layering.md)、[代码生命周期](../standards/code-lifecycle.md)、[验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)
- 开始时补充 Qt 对应版本的官方 `QStyleHints` API 文档、查阅日期及实际构建版本。

## 范围与非目标

包含：

- 在 Qt 基线升级且该版本正式提供可用的 `motionPreference` 后，将 `native/qt-adapter` 的 reduced-motion 读取和变化通知改为 `QGuiApplication::styleHints()` 原生 API。
- 保持 QML 可见的 `SystemPreferences.reducedMotion` 行为与绑定稳定；将 Qt 的偏好枚举明确映射到现有布尔语义。
- 删除被替代的平台专属查询、监听、源文件、框架/组件依赖和只服务这些实现的构建分支。
- 更新 080、分层文档和本任务中的实现及验证记录。

不包含 Qt 基线升级本身、其他系统偏好迁移，或改变文档页签动画策略。

## 前置条件与待决策

- Qt 依赖基线已升级到官方文档确认含该 API 的版本；如果实际基线未升级，本任务保持 planned。
- 核对 `motionPreference` 对“偏好未知/系统不支持”的语义及其通知信号，决定布尔映射和 fallback。不得仅根据属性名称推测枚举含义。
- 核对三平台 Qt 实现覆盖。如果任一目标平台仍无 Qt 支持，则记录原因并保留该平台必要的最小后端；只有被 Qt 原生实现完整取代的旧代码才能删除。

## 实施步骤

1. 查阅并记录已升级 Qt 版本的官方 API、枚举值、通知信号与平台覆盖。
2. 在 `native/qt-adapter` 中替换后端实现，同时保留面向 Bridge/QML 的公开 reduced-motion 属性。
3. 删除已无调用的 AppKit、Win32、DBus 查询/监听实现及关联依赖、构建选项和文档说明。
4. 验证初始值、系统偏好运行期变化、未知值映射、QML 绑定，以及目标平台构建；记录真实结果。

## 预计改动

- `native/qt-adapter/`：reduced-motion API 的实现与相应平台代码清理。
- `native/CMakeLists.txt`、`native/qt-adapter/CMakeLists.txt`：清除不再需要的 Objective-C++ / AppKit / DBus 平台依赖。
- `ai-docs/task/080-qml-viewport-document-tabs.md`、`ai-docs/standards/layering.md`：同步当前实现与适配边界。

## 清理与兼容例外

替换后删除不再需要的 AppKit 通知观察器、Windows native event filter / `SystemParametersInfo` 查询、Linux XDG Portal Settings D-Bus 请求与订阅，以及仅由这些代码需要的构建依赖。保留无平台差异的 Qt adapter API 与 Bridge/QML 稳定属性。无兼容例外。

## 验收标准

- [ ] 官方文档确认目标 Qt 版本包含 `QStyleHints::motionPreference`；任务记录其准确枚举语义、变化通知与平台覆盖。
- [ ] Qt adapter 从该属性读取初始值并响应运行期变化，Bridge 暴露的布尔属性仍正确更新 QML 动画绑定。
- [ ] 未知/不支持取值按已记录的规则处理，不误判为必须减少动态效果。
- [ ] 被 Qt 原生 API 完全取代的平台代码及专用框架/组件依赖已删除；没有保留死代码。
- [ ] 适用平台构建与仓库统一验证入口通过，真实命令和结果已记录；任务 080 与分层文档同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| — | 待 Qt 基线升级后填写 | 见验收标准 | 未执行；前置条件未满足 |

## 风险与回退

Qt 的平台后端可能只在部分操作系统映射原生偏好。实现前先检查实际支持范围；若覆盖不完整，只保留未覆盖平台所需的专属后端并说明原因。若升级后 API 无法提供运行期变化通知，则不得假定现有 QML 属性会自动更新，需按官方契约调整方案或将该问题作为明确阻塞。

## 决策与工作记录

- 2026-09-28：根据维护者提出的 Qt 6.12 `motionPreference` 信息登记后续迁移任务；本任务等待 Qt 基线升级，不修改当前依赖版本。

## 完成摘要

未开始。Qt 基线升级并满足前置条件后实施；当前跨平台 reduced-motion 查询由任务 080 的 Qt adapter 承担。
