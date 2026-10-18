# 022 — UI 英文源文案与语言字典

- 状态：done
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)、[034](034-rust-panta-artifact-parser.md)、[086](086-qt-platform-adapter.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-10-17

## 目标与背景

统一现有 UI 的英文源文案，以 Panta DSL（`.pa`）字典支持英文与简体中文切换。`.pa` 是唯一权威源码；Qt `.qm` 仅是构建期生成的运行时产物。Rust 拥有 locale 选择规则；C++/Qt 只负责把 locale 对应到 QM resource 并管理 QTranslator。设置页和偏好持久化分别由 [087](087-language-settings-ui.md) 与 [125](125-settings-service-and-qt-adapter.md) 承接；设置页通过 [115](115-platform-application-menu-and-settings-entry.md) 提供的平台 Settings 入口访问。

## 必读

- [模块设计](../modules/internationalization.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [qt](../standards/qt.md)
- [qml](../standards/qml.md)
- [cmake](../standards/cmake.md)

## 范围与非目标

包含现有 QML/C++ 展示文案、`.pa` 字典 schema、TS/QM 构建转换、语言运行时切换/刷新及缺项回退；不包含设置页面与偏好持久化（087/125）、完整多语言翻译、RTL 适配或 DSL 关键字本地化。VTK 欢迎画面字标下方的 `Welcome!` 按用户决策保留英文，作为品牌装饰文案，不计为漏译。

## 前置条件与待决策

034 的 `.pa` 解析、Artifact 聚合、临时 TS 生成、QtTools `lrelease` 与 QM 资源已接入；086 的 Rust locale 服务、Qt translator adapter 和动态重翻译也已接入。当前工作以现有实现为基线，不重建这些管线：逐项核对真实 UI 调用与中英文目录、修复缺失或失配文案，验证无效字典/locale 的失败保持行为，并检查剩余硬编码用户文案。语言文件继续使用 `kind language` 受限声明，不允许脚本、import、网络或 shell。QSettings 持久化由 125 提供，界面由 087 提供并经 115 的 Settings 入口访问。

## 实施步骤

1. 审计 QML/C++ 的静态翻译调用、产品文案及错误路径，与中英文 `.pa` 目录逐项比对，确定真实缺项和失配。
2. 修正文案 key、英文源句和简体中文译文；覆盖参数、复数和无译文回退，不修改用户名称、工程数据或协议值。
3. 用既有 086 运行时验证切换无需重建引擎，校验未知 locale 与损坏 QM 保留上一有效语言，并在真实窗口检查英文默认主页面与弹窗；设置 UI 中的端到端切换和持久化由 087/125 验收，并经 115 的 Settings 入口访问。

## 预计改动

现存 qml/、native/bridge/、native/i18n/、`resources/i18n/` 与 Rust `.pa` 编译入口。翻译运行时由 086 接入；设置页由 087 承接。路径以实施时实际模块归属为准。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] 英文默认启动，英文/简体中文切换后 QML 与 C++ 展示属性同步更新且不重建引擎。
- [x] 缺项回退英文；损坏字典保留上一语言且不提交 locale。设置偏好写入和重启恢复由 125/087 验收。
- [x] 现存产品文案无缺项，参数占位正确，项目名、文件名、序号和数值作为数据参数保留。当前 UI 没有复数文案；DSL 复数表单与 TS 生成由 034 的测试覆盖。
- [x] 干净构建含翻译资源，任意 cwd 启动可读字典；构建工具版本与三平台供给可追溯。
- [x] 三平台 CI 对锁定 Qt host tools 的构建与适用测试通过。
- [x] 真实窗口英文默认主页面与新建工程弹窗文案可读；设置界面的语言切换验收由 087 承接。
- [x] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。

## 验证计划与结果

现有 Cargo 构建、native CTest、all_qmllint 与真实窗口默认文案抽查；自动化用例验证语言切换/重翻译和失败字典场景。设置界面操作、语言偏好写入及重启恢复由 125/087 验收。实施时填写 cwd、工具链、依赖版本和退出结果；不预先声称测试目标已存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | 本次完成 `.pa` 后缀、语言域 schema 与构建产物边界设计 | 实现与功能验证未执行 |
| 2026-10-17 | 开始实施前检查 `resources/i18n/`、`Settings`、`TranslationCatalog`、QML 调用及既有 CTest | 034/086 的构建和运行时管线、基础切换测试已存在；完整文案覆盖与真实窗口验收仍需审计。 |
| 2026-10-17 | 静态引用、参数、运行时失败路径、构建及窗口抽查 | 324 个静态 QML 引用均能匹配中英文目录，各有 407 个 key，互相无缺项。带用户名称、文件名、序号及物理数值的参数替换在 en/zh-CN QM 测试通过；当前 UI 无复数文案，034 已验证 DSL 复数表单及 TS 输出。`cargo verify-changed` 通过，Cargo 聚合测试与 CTest 77/77 通过；缺失/损坏 QM 保留旧 translator、缺项回退 source、同一 QML 引擎切换及 C++ 错误/修订文案重译均有回归覆盖。`cargo build --locked` 通过；从 `/tmp` 以 Qt offscreen 平台启动当前 debug app 成功，无 cwd/QML/翻译资源错误。现存 GUI 的英文主窗口和新建工程弹窗 AX 文案正常；语言选择 UI 尚未接入，真实窗口切换归 087 验收。本机 shell 直接启动 AppKit 窗口被 pasteboard 服务拒绝。构建工具锁定及三平台 Qt 供给可追溯于 005/034；当前变更仍待独立三平台 CI。`gpt-6.1-sol` medium 独立只读评审发现并协助定位修复修订说明的缓存翻译问题，最终 diff 无其他可操作问题。 |
| 2026-10-17 | 删除 QM 参数测试中的重复断言后复验与最终复核 | `cargo format --changed` 通过；重建 `panta_i18n_qm_test` 并运行 `I18n.CompiledQmLoads`，1/1 通过；`git diff HEAD --check` 通过。`gpt-6.1-sol` medium 只读复核最终 diff，确认中英文参数断言保留且无新的可操作问题。 |
| 2026-10-17 | 全新 Cargo/native 构建树与任意 cwd 翻译资源验收 | 首次隔离构建发现 CMake 从 Homebrew 选取 Qt 6.12 `qmlcachegen`，与锁定的 Qt 6.11.2 头文件不匹配；在 `native/CMakeLists.txt` 将 `QT_HOST_PATH` 固定到托管 staging 后，全新 `CARGO_TARGET_DIR` 的 `cargo build --locked` 成功，生成的 `Qt6QmlTools_DIR` 与 qmlcachegen 均来自 staging。lrelease 为英文、中文各生成 407 条；干净树 `I18n.CompiledQmLoads` 1/1 通过；从 `/tmp` 启动干净构建的 offscreen 应用到达事件循环。三平台 Qt 版本与固定资产见 005/034；本机实际构建平台为 macOS arm64。 |
| 2026-10-17 | CMake host-tools 修复后的变更范围门禁 `cargo verify-changed` | 选择器因公共 CMake 构建边界将范围升级为全量；格式、9 阶段 lint、全部 Rust package 测试及 native/QML CTest 77/77 通过。 |
| 2026-10-17 | 独立 final review 后修复 Welcome 页签重译 | `DocumentTabBar` 按 welcome 文档类型动态翻译显示名和无障碍名称；新增同一 QML 引擎切换中英文时验证页签标题与关闭按钮名称的回归测试。VTK 欢迎画面的 `Welcome!` 根据用户决策保留英文。 |
| 2026-10-17 | PR #57 三平台 CI 验收并合并 | 所有 CI 检查通过，包括 macOS / Ubuntu / Windows 构建、Rust 格式与 lint、CMake / C++ / QML lint、依赖审计；run [39021842925](https://github.com/Yuki-Nagori/panta/actions/runs/39021842925)。PR 已合并至 `main`，合并提交为 `c94ed1e`。 |

## 风险与回退

翻译刷新遗漏 C++ 缓存字符串；失败保留旧 translator 与设置，可恢复到内置英文。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-16：语言、主题和变量统一使用 `.pa` 作为 Panta DSL 源文件后缀；`.pt` 不采用，以免与 Portuguese/locale 语义混淆。
- 2026-09-16：首期语言标识固定为 `en` 与 `zh-CN`；`.qm` 只作为构建产物，语言服务切换失败保持既有状态。
- 2026-09-17：语言源码按 locale 拆分为 `resources/i18n/panta-en.pa`、`panta-cn.pa` 等文件；每个文件只有一个 `language`，`[Context]` 表示 Qt 调用上下文，条目内 `comment` 才表示翻译注释。
- 2026-10-17：VTK 欢迎画面字标下方的 `Welcome!` 是品牌装饰文字，保留英文；文档页签的 Welcome 属于导航名称，随当前语言显示。

## 完成摘要

已完成。现有 QML/C++ 展示文案、缺项与失败回退、占位参数、运行时重译及默认窗口文案均完成验收；锁定 Qt host tools 的 macOS、Ubuntu、Windows CI 全部通过。设置界面端到端语言切换与偏好持久化仍由 087/125 承接，并通过 115 的 Settings 入口访问。
