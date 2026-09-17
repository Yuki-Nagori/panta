# 026 — C++ 静态库边界与 QML 自动注册

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

接续 005 的注册与 lint 缺口，验证静态库、正式 QML 模块和按需编译的组合。Panta.Bridge 的静态模块迁移已落地，本轮补齐可重复的运行时加载测试、Release 验证和真实的模块开关；跨平台运行仍需后续证据。

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

包含 Panta.Bridge 自动注册、生成 typeinfo、插件链接、Cargo `bridge-module` feature/CMake `PANTA_ENABLE_BRIDGE_MODULE` 开关和运行时资源加载测试；不包含动态 C++ 插件加载、状态热重载或没有实际产品用途的示例模块。关闭开关用于最小 Shell/资源链路和依赖裁剪验证，默认生产构建保持开启。

## 前置条件与待决策

005 已完成。实施核对 Qt 锁定版本与现存 NO_PLUGIN 布局；分别验证 backing target、静态 plugin/import scan 方案，失败时记录证据，不以关闭 lint 掩盖问题。

本轮采用 `panta_bridge` 静态库作为 `Panta.Bridge` QML 模块 backing target，由 `qt_add_qml_module` 生成 typeinfo 和静态 plugin；`Panta.Shell` 在开关开启时通过模块依赖声明消费它，应用目标由 `Q_IMPORT_QML_PLUGIN` 显式保留注册代码。关闭时不加入 bridge 子目录，Shell 改用不导入 `Panta.Bridge` 的 `AppNoBridge` 最小入口资源，应用不链接 plugin。两种入口均编译进对应构建的 Shell 模块资源，app 目标没有可供 qmlimportscanner 扫描的源文件，因此不调用空结果的 app 级 import scan。

## 实施步骤

1. 梳理现存模块依赖与 URI，验证最小自动注册/静态链接组合。
2. 迁移 ViewModel 注册，保留 `Panta.Shell → Panta.Bridge` 单向依赖，并加入运行时加载 CTest。
3. 增加 `PANTA_ENABLE_BRIDGE_MODULE` 开/关构建图和无 Bridge fallback，验证两种模式的资源、依赖和 lint。
4. 删除替代的 qmlRegisterType、过期注释及冗余配置，补资源、Debug/Release 和 lint 验证。

## 预计改动

现存 native/、native/app/tests/、qml/、qml/AppNoBridge.qml、launcher build.rs 与构建配置；不新增无长期用途的示例模块。上述新增路径/类型均以实际模块归属为准。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] ShellViewModel 自动注册可用，生成类型信息使 qmllint 可识别，现有信号行为保持。
- [ ] 静态链接及优化构建保留注册与资源，macOS/Linux/Windows 构建和运行证据明确（当前已有 macOS Debug/Release 证据，跨平台运行待 CI）。
- [x] 开关关闭 `Panta.Bridge` 时主界面无悬空 import；启用时 `ShellViewModel` 类型可用，依赖图无环。
- [x] 脱离源码 cwd 可启动；旧手动注册与重复路径已删除，不同时保留两套注册实现。
- [x] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。

## 验证计划与结果

CMake/Cargo 构建、CTest、all_qmllint、Release 构建和真实启动验证模块资源；Cargo feature 开/关和直接 CMake 开关均需验证，013 再负责完整安装包冒烟。在实施时填写 cwd、工具链、依赖版本和退出结果；不预先声称测试目标已存在。

| 日期 | 场景 | 预期 | 实际结果 |
|---|---|---|---|
| 2026-09-16 | 本次仅完成规划 | 无 | 实现与功能验证未执行 |
| 2026-09-17 | `cmake --preset debug`；`cmake --build build/debug`；`cmake --build build/debug --target all_qmllint`；`ctest --test-dir build/debug --output-on-failure`；Qt offscreen 启动 | 静态 QML 模块迁移后配置、构建、lint、测试和运行时加载通过 | 生成 `Panta.Bridge` typeinfo/qmldir/static plugin；qmllint 无告警；6/6 native tests 通过；offscreen 运行 2 秒后由测试终止且无 QML 加载错误 |
| 2026-09-17 | macOS arm64 Debug；`cmake --build native/build/debug --target all_qmllint`；`ctest --test-dir native/build/debug --output-on-failure` | 自动注册与资源加载测试进入 CTest | qmllint 通过；7/7 native tests 通过，新增 `Qml.ShellModuleLoads` 验证 `Panta.Shell` 资源和 `Panta.Bridge` 静态 plugin 可加载 |
| 2026-09-17 | macOS arm64 Release；隔离构建树，复用已校验 Qt staging 与本地 GTest source；Release build、`all_qmllint`、CTest | 优化构建保留 typeinfo、资源和静态注册 | 65/65 构建步骤完成；qmllint 通过；7/7 native tests 通过 |
| 2026-09-17 | `/private/tmp` cwd；`QT_QPA_PLATFORM=offscreen` 启动 Release `panta-native`，2 秒后 SIGTERM | 脱离源码 cwd 加载模块并正常响应终止 | 退出码 143；无 QML import/binding 错误，仅有 Qt 字体别名性能提示 |
| 2026-09-17 | `PANTA_ENABLE_BRIDGE_MODULE=ON`；macOS Debug；`all_qmllint` + CTest | 默认模式保留 Bridge typeinfo/plugin 和原有行为 | qmllint 通过；7/7 native tests 通过，`Qml.ShellModuleLoads` 加载 `Panta.Shell/App` |
| 2026-09-17 | `PANTA_ENABLE_BRIDGE_MODULE=OFF`；隔离构建树 `/private/tmp/panta-native-bridge-off.z0r2OC`；build、`all_qmllint`、CTest | 裁剪 Bridge 子目录和依赖，Shell 不留下悬空 import | 58/58 构建步骤完成；qmllint 通过；3/3 tests 通过；生成资源无 `Panta.Bridge` import 或 `ShellViewModel`，无 Bridge artifacts |
| 2026-09-17 | OFF 模式 `/private/tmp` cwd；`QT_QPA_PLATFORM=offscreen` 启动 `panta-native`，1 秒后 SIGTERM | 最小 Shell fallback 可独立启动 | 退出码 143；无 QML import/binding 错误，仅有 Qt 字体别名性能提示 |
| 2026-09-17 | macOS arm64 Debug；`CMAKE` 指向已缓存 Qt/GTest staging 的 Cargo wrapper；`cargo build --locked` | 默认 `bridge-module` feature 映射为 CMake ON | 通过；launcher/native 构建成功，CMakeCache 确认 `PANTA_ENABLE_BRIDGE_MODULE=ON` |
| 2026-09-17 | macOS arm64 Debug；同上；`cargo build --locked --no-default-features` | 无默认 feature 映射为 CMake OFF，构建最小 Shell | 通过；launcher/native 构建成功，CMakeCache 确认 OFF，`Panta.Shell/qmldir` 仅列出 `AppNoBridge` 入口 |

## 风险与回退

静态链接裁剪注册入口；先用最小样例定位，验证完成后整套切换，失败不留下混合注册状态。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-17：开始实施；优先验证 `panta_bridge` backing target + `Panta.Bridge` 静态 plugin + `Panta.Shell` TARGET 依赖，移除 main.cpp 手动注册。
- 2026-09-17：新增 `Qml.ShellModuleLoads` CTest，直接从资源模块加载 `Panta.Shell/App`，把静态 plugin 注册和运行时资源验证纳入 Debug/Release 构建。
- 2026-09-17：将 `PANTA_ENABLE_BRIDGE_MODULE` 定为真实开关；开启走静态 Bridge plugin，关闭裁剪 bridge 子目录并使用无 Bridge 的最小 Shell 入口，服务于最小资源链路/依赖诊断构建。
- 2026-09-17（CI 兼容）：保留显式 `Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)`，删除 app 级 `qt_import_qml_plugins` 空扫描，避免预编译 Qt Linux 工具的 ICU 运行时依赖；后续 app 有可扫描 QML 源时再按 Qt 版本重新评估。

## 完成摘要

核心实现已完成并补齐 macOS Debug/Release、开关 ON/OFF 自动验证；仍待 Linux/Windows 运行证据。
