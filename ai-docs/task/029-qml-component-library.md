# 029 — QML 原子组件库与 Theme 尺寸参数化

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

基于当前界面建立原子/组合组件，以明确输入属性和输出信号拼装面板；所有可配置视觉尺寸统一收敛到现存 Theme.qml。 当前仅规划，未实施。

## 必读

- [组件与主题设计](../modules/qml-components-and-theme.md)
- [QML 规范](../standards/qml.md)、[Qt 规范](../standards/qt.md)
- [注释规范](../standards/comments.md)、[文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

本期迁移现有 UI、提取实际需要的组件、集中 token、提供组件展示与布局验证；不接 DSL，不预建完整控件大全。

## 前置条件与实施步骤

上述依赖产物可用后实施；029 无需等待正式模块迁移 026，030 无需等待热重载 027。涉及 FFI 时沿用 025 已落地的契约。

先盘点 qml/ 内视觉常量及重复控件，定义 token 和组件 API；再提取 Atoms/Composites 并迁移调用点；最后验证参数覆盖、布局和信号。保留 Qt Controls 的键盘与可访问性行为。

## 预计改动

现存 qml/、Theme.qml 及相关 CMake 文件；按本任务范围新增组件/测试或主题服务与 DSL 资源，目录以模块说明为规划依据，不创建空占位模块。

## 当前进展

- 已在 `qml/Components/Atoms/` 落地 `ThemedLabel`、`ThemedButton` 和 `PanelSurface` 三个无业务依赖的原子组件。
- `App.qml` 与 `PlaceholderPanel.qml` 已改为通过原子组件拼装；窗口最小尺寸、控件高度、面板圆角等视觉 token 集中到 `Themes/Theme.qml`。
- `ThemedButton` 保留 Qt Quick Controls 的键盘、焦点和禁用行为，组件通过 `contentPadding`、`controlHeight` 等属性接受覆盖。
- 尚未完成真实窗口的长文本、中英文、缩放和不同 DPI 验证，也未建立专门的 QML 行为测试；因此任务仍为 `in-progress`。

## 清理与兼容例外

同步清理被替换的控件、样式、默认值与调用点。无兼容例外，不长期保留双数据源或两套主题入口。

## 验收标准

- [x] 现有界面由职责明确的组件拼装，原子组件不依赖业务 ViewModel 或任意父级 id。
- [x] 所有可配置视觉尺寸以属性传入并默认绑定 Theme，面板无散布的视觉魔法数字；布局计算和纯算法常量有明确边界。
- [ ] 参数覆盖有效，未覆盖参数随 Theme 更新；不出现 binding loop、布局冲突或失效绑定。
- [ ] 真实窗口验证长文本、中英文、缩放、不同 DPI、焦点和禁用状态；现有命令行为保持。
- [x] 移除被替代的内联控件、重复样式与尺寸常量，Theme 仅有一份权威默认值。
- [ ] 代码、资源、测试、文档及索引一致，验证记录完整。

## 验证计划与结果

实施时执行现有 all_qmllint、相关构建和实际新增行为测试，并在真实窗口验证布局；030 增加 schema 正反例、快照失败保持、主题选择与重启验证。记录 cwd、工具链和实际命令，不预设测试入口存在。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | `cmake --preset debug` → `cmake --build build/debug` | 预编译 Qt 已缓存后配置与构建成功；生成三个原子组件并完成 QML cache 编译 |
| 2026-09-16 | `cmake --build build/debug --target all_qmllint` | 命令成功；仅有既存 `Panta.Bridge` 手动注册类型不可见警告，无新增组件错误 |
| 2026-09-16 | Qt 6.11.2 `qmlformat` 输出与 `qml/` 文件逐个 diff | 新增及迁移的 QML 文件格式一致 |
| 2026-09-16 | `ctest --test-dir build/debug --output-on-failure` | 5/5 native tests 通过 |
| 2026-09-16 | `cargo build --locked`、`cargo test --locked` | Rust 构建成功；4/4 launcher tests 通过 |
| 2026-09-16 | `native/build/debug/app/panta-native --version`、未知参数冒烟 | 版本输出正确；未知参数以 64 退出并给出用法诊断 |
| 2026-09-16 | `QT_QPA_PLATFORM=offscreen` 启动 2 秒后终止 | 事件循环保持运行且无 QML 加载错误输出；因无头进程不会自行退出，按测试时限终止 |

## 风险与回退

尺寸集中化可能暴露固定布局假设，主题值发布可能破坏绑定；先用现有面板验证最小组件，再扩大迁移。主题切换失败保持最后有效快照；文件保存失败不覆盖用户原文件。

## 决策与工作记录

- 2026-09-16：依据用户原子组件与 DSL 主题需求，由 028 编排。

## 完成摘要

未完成。
