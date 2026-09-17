# 026 — C++ 静态库边界与 QML 自动注册

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

接续 005 的注册与 lint 缺口，验证静态库、正式 QML 模块和按需编译的组合。 当前仅规划，未实施。

## 必读

- [模块设计](../modules/qml-modules-and-reload.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [cmake](../standards/cmake.md)
- [qt](../standards/qt.md)
- [qml](../standards/qml.md)
- [cpp](../standards/cpp.md)

## 范围与非目标

包含 Panta.Bridge 自动注册、生成 typeinfo、插件链接、可选模块裁剪；不包含动态 C++ 插件加载或状态热重载。

## 前置条件与待决策

005 已完成。实施核对 Qt 锁定版本与现存 NO_PLUGIN 布局；分别验证 backing target、静态 plugin/import scan 方案，失败时记录证据，不以关闭 lint 掩盖问题。

本轮采用 `panta_bridge` 静态库作为 `Panta.Bridge` QML 模块 backing target，由 `qt_add_qml_module` 生成 typeinfo 和静态 plugin；`Panta.Shell` 通过模块依赖声明消费它，应用目标由 Qt 的 plugin import 入口保留注册代码。

## 实施步骤

1. 梳理现存模块依赖与 URI，验证最小自动注册/静态链接组合。
2. 迁移 ViewModel 注册，保留单向依赖，并验证一个有实际用途的可选模块开关。
3. 删除替代的 qmlRegisterType、过期注释及冗余配置，补部署资源和 lint 验证。

## 预计改动

现存 native/bridge/、native/app/、qml/CMakeLists.txt 与 native 构建配置；示例模块仅在有长期用途时保留。 上述新增路径/类型均为规划，以实施时实际模块归属为准。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] ShellViewModel 自动注册可用，生成类型信息使 qmllint 可识别，现有信号行为保持。
- [ ] 静态链接及优化构建保留注册与资源，macOS/Linux/Windows 构建和运行证据明确。
- [ ] 开关关闭可选模块时主界面无悬空 import；启用时类型可用，依赖图无环。
- [ ] 脱离源码 cwd 可启动；旧手动注册与重复路径已删除，不同时保留两套注册实现。
- [ ] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。

## 验证计划与结果

Cargo 构建、CTest、all_qmllint，可选开关开/关与 Release 构建；真实启动验证模块资源，013 再负责完整安装包冒烟。 在实施时填写 cwd、工具链、依赖版本和退出结果；不预先声称测试目标已存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | 本次仅完成规划 | 实现与功能验证未执行 |
| 2026-09-17 | `cmake --preset debug`；`cmake --build build/debug`；`cmake --build build/debug --target all_qmllint`；`ctest --test-dir build/debug --output-on-failure`；Qt offscreen 启动 | 静态 QML 模块迁移后配置、构建、lint、测试和运行时加载通过 | 生成 `Panta.Bridge` typeinfo/qmldir/static plugin；qmllint 无告警；6/6 native tests 通过；offscreen 运行 2 秒后由测试终止且无 QML 加载错误 |

## 风险与回退

静态链接裁剪注册入口；先用最小样例定位，验证完成后整套切换，失败不留下混合注册状态。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-17：开始实施；优先验证 `panta_bridge` backing target + `Panta.Bridge` 静态 plugin + `Panta.Shell` TARGET 依赖，移除 main.cpp 手动注册。

## 完成摘要

未完成，等待实施与验收。
