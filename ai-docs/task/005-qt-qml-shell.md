# 005 — Qt/QML 主窗口与 C++ ViewModel

- 状态：done
- 阶段：M0
- 依赖：[004](004-cargo-native-orchestration.md)（已完成）
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

通过统一入口启动最小 QML 桌面窗口，验证属性、命令与模块资源。

## 必读

- [通用规范：comments](../standards/comments.md)

- [规范：qt](../standards/qt.md)
- [规范：qml](../standards/qml.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [架构：ui-and-bridge](../architecture/ui-and-bridge.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不做几何/网格业务，不加入 VTK；占位状态不得伪装功能完成。

## 前置条件与待决策

依赖 004 已完成。实施中维护者关键决策：**Qt 不源码构建，直接下载官方预编译产物，且三平台都要就绪**（覆盖依赖获取文档原"源码构建首选"方案）。其余待决策（模块 URI、注册方式、退出码、lint 接入）见决策记录。

## 实施步骤

1. 创建 Qt native 入口与 QML 模块，明确模块 URI 和目录，将 UI 布局由 native CMake 纳入构建。
2. 注册一个最小 ViewModel，验证命令触发和属性通知，使用占位区域表达未来面板。
3. 建立主题/资源入口与错误展示；应用失败启动返回非零结果。
4. 接入所锁定 Qt 的 lint/格式工具，验证 QML/资源修改会触发必要重建。

## 预计改动

native/app/、native/bridge/、qml/ 及模块 CMake、resources/。实际改动：`native/app/{main.cpp,CMakeLists.txt}`（Qt 实现）、`native/bridge/`（ShellViewModel + GTest 测试）、`qml/{CMakeLists.txt,App.qml,Themes/Theme.qml,Panels/PlaceholderPanel.qml}`、`native/cmake/qt-provision.cmake`（Qt 预编译供给）、`native/CMakeLists.txt`（供给 + find_package + qt_standard_project_setup + 新子目录）、build.rs rerun 清单扩展（qml/、bridge/）。`resources/` 未创建：主题入口由 Theme 单例承担，资源扩展点以 qt_add_resources 注释标注（决策记录）。文档：README、build-and-development、repository-layout、qml.md、baseline、dependency-acquisition、task-index。

## 清理与兼容例外

native/app 的非 Qt 骨架实现（打印版本退出）被 Qt 实现替换；`--version` 行为保留。无兼容层。

## 验收标准

- [x] cargo run 启动主窗口，QML 无缺失 import 或绑定错误。
- [x] 按钮能通过 C++ bridge 改变可观察状态，重复值不会产生错误通知。
- [x] 非源码 cwd 可加载资源，窗口 resize/关闭正常，记录实际启动命令。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

验收第 3 项口径：资源加载以"运行 `target/debug/panta-launcher`（cwd=仓库根之外语义）+ QML 全部来自资源系统 qrc"验证；窗口关闭经维护者人工点击（首次会话）与 SIGTERM 路径双重验证；resize 拖拽未自动化实测（System Events 被权限拒绝，见验证表），仅以 `minimumWidth/Height` 约束与可交互会话为旁证——此项视为部分覆盖并如实标注。

## 验证计划与结果

环境：macOS 26.3.1 arm64、Apple clang 17.0.0、cmake 4.3.3 + Ninja 1.13.2、Qt 6.11.2 预编译（供给脚本）、rustup 1.98.1。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | qtsdkrepository 三端资产核实与下载 | macOS/Linux/Windows 的 qtbase+qtdeclarative 归档全部下载并实测 SHA256（记录于 qt-provision.cmake，强校验）；7z 解包用 `cmake -E tar` 实测可用（三平台零额外工具） |
| 2026-09-16 | 预编译 Qt 冒烟：find_package(Qt6 Qml Quick QuickControls2) + 链接运行 | configure/build/run 通过；staging 含 qmake/qmllint/qmlformat/146 个 Qt6 CMake 包 |
| 2026-09-16 | `cargo build --locked`（多轮） | 首轮起暴露并逐项修复：缺 AUTOMOC（qt_standard_project_setup）、qt_import_qml_plugins_to_target 命令名错误（实为 qt_import_qml_plugins）、add_subdirectory(app) 重复、QML 模块 include 路径缺失、QTP0001/QTP0004 策略、Theme 单例属性声明顺序（必须先于 qt_add_qml_module） |
| 2026-09-16 | `cargo run` 首启（维护者人工交互） | 主窗口启动；维护者点击"推进修订"9 次并正常关闭窗口（进程干净退出）——命令→bridge→属性通知→QML 绑定全链人工验证；暴露 Theme 单例未生效（qmldir 缺 singleton 标记，属性 undefined 告警），修复后复验 |
| 2026-09-16 | 修复后 `cargo run` + 截图 | 深色主题正确渲染（背景/面板/文字色、Theme 单例生效）；运行日志无任何绑定/导入错误 |
| 2026-09-16 | 无障碍 AX 点击"推进修订" | 计数 8→9、caption 同步"修订 9"——命令链路程序化复证 |
| 2026-09-16 | `cargo test --locked`；`ctest --test-dir <OUT_DIR>/native-build` | launcher 4 passed；native ctest 5 passed（foundation 1 + ShellViewModel 4，含重复写入零通知断言） |
| 2026-09-16 | SIGTERM 转发：`kill -TERM <launcher 子进程>` | launcher 退出码 143（128+15），与 004 定义一致；首次测试因 pgrep 误选旧 detached 进程得出不可信结果，改 `pgrep -P` 后重测（过程记录于工作记录） |
| 2026-09-16 | 非源码 cwd 资源加载：`/tmp` 下启动安装树/构建树产物 | QML 全部经 qrc 资源系统（qrc:/qt/qml/Panta/Shell/...）加载，无 cwd 依赖 |
| 2026-09-16 | qmllint：`cmake --build <树> --target all_qmllint` | exit 0；唯一告警类别为手动注册类型（ShellViewModel）对 qmllint 不可见——NO_PLUGIN+手动注册方案的已知限制（决策记录） |
| 2026-09-16 | 增量重建：修改 qml/App.qml（加窗口背景色）后 `cargo build` | build.rs 重跑 → qmlcachegen 重跑 → 重链，1.6s 完成；QML/资源重建追踪实测生效 |
| 2026-09-16 | `cargo fmt --all -- --check`、`cargo clippy --locked --all-targets` | 均通过 |

未覆盖：Linux/Windows 运行时（预编译资产与供给逻辑三端就绪，运行验证待 012 runner；Linux 需 glibc ≥2.34，Windows 需 MSVC2022）；窗口 resize 拖拽未自动化实测；qmllint 对手动注册类型的可见性（随类型增多需重评模块化注册）。

## 风险与回退

开发目录 import/资源路径风险已按验收消除（纯 qrc 加载）；Qt 预编译包与未来 VTK 源码构建的版本一致性由 staging 前缀保证（007 验证）。回退仅撤销本任务自身变更（Qt 供给脚本与 Qt 化代码），恢复 004 的非 Qt 骨架；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-16（决策，维护者）：Qt 采用官方预编译产物（qtsdkrepository，与在线安装器同源、免账号），**三平台资产全部固定并实测 SHA256**；解包用 `cmake -E tar`（内建 libarchive 支持 7z）。原"源码构建首选"方案作废，dependency-acquisition 已改写。
- 2026-09-16（决策）QML 模块形态：Panta.Shell 用 `NO_PLUGIN` 纯资源模块（QML/qmldir 进 qrc）；ShellViewModel 为普通静态库 + main.cpp `qmlRegisterType` 显式注册。偏离 qt.md"模块化注册优先"：静态 QML 模块插件的链接依赖 qmlimportscanner 扫描 exe 自身 QML 源，与"App.qml 独立于 exe 目录（根 qml/ 布局）"冲突；恢复模块化注册的条件（App.qml 移入 exe 模块或扫描路径方案验证）记为后续任务输入。
- 2026-09-16（决策）ViewModel 契约落地：`caption`（写去重：值相同不发 NOTIFY）、`count`（tick() 命令推进）、`error`（用户可读摘要，QML 错误展示入口）；GTest + QSignalSpy 断言通知次数而非仅终值。
- 2026-09-16（决策）失败路径：QML objectCreationFailed 或根对象为空 → 打印引擎错误并以 69（EX_UNAVAILABLE）退出；`--version` 保留（不进事件循环）；未知参数 64，`--` 后参数透传给 QGuiApplication。
- 2026-09-16（决策）lint 接入：`all_qmllint` CMake 目标（qml.md 的 qmllint 要求落地）；qmlformat 未纳入门禁（011 决定格式门禁范围）。
- 2026-09-16（实施）实施坑位记录：Theme 单例的 `set_source_files_properties(QT_QML_SINGLETON_TYPE)` 必须先于 qt_add_qml_module，否则 qmldir 缺 singleton 标记且运行时属性全 undefined；`qt_import_qml_plugins_to_target` 实名 `qt_import_qml_plugins`；Windows 归档下载首测因 curl 未加 `-f` 把 404 页面存成归档（哈希相同暴露），重下修正。
- 2026-09-16（范围外修正）.vscode/README 徽章等此前的维护者反馈项不属本任务，保持原样。
- 待记录：007 需确认预编译 Qt 的图形后端（Metal）与 VTK 的兼容；模块化注册恢复条件。

## 完成摘要

已交付：Qt 6.11.2 预编译供给链（三平台固定清单 + SHA256 强校验 + `cmake -E tar` 解包，缓存于构建树）；Qt Quick 主窗口（深色主题、命令按钮、修订计数、占位面板、错误展示入口）经 `cargo run` 一键启动；ShellViewModel（caption 去重通知 / tick 命令 / error 属性）以 GTest+QSignalSpy 全量断言；qmllint 经 `all_qmllint` 接入；QML 修改触发增量重建实测。验证包括真实 GUI 交互（维护者人工 9 次点击 + AX 程序化点击 + SIGTERM 143 转发）。剩余限制：Linux/Windows 运行验证待 012；resize 拖拽未自动化实测；手动注册类型对 qmllint 不可见。后续：007（VTK 视口）已 ready，主线推进；006（FFI）可并行。
