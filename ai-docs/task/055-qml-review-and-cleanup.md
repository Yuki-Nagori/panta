# 055 — QML 组件评审与整理

- 状态：done
- 阶段：应用平台扩展
- 依赖：[052 首页优化](052-qml-icon-and-layout-polish.md)、[054 顶部细节](054-titlebar-search-details.md)
- 优先级：P1
- 负责人：Codex
- 创建 / 更新：2026-09-22 / 2026-09-22

## 范围与目标

整体评审当前 Shell QML、原子/组合组件、图标接入和相关测试，保持已确认的默认视觉及撤回的交互状态。整理重复配置、无用属性与失效注释，修正有证据的布局/绑定问题。053 保持 planned，不修改 VTK 字样或接入 STL。

## 规则

遵循 [QML](../standards/qml.md)、[注释](../standards/comments.md)、[生命周期](../standards/code-lifecycle.md)、[验证](../standards/validation-and-review.md)、[提交](../standards/commits.md) 和 [组件模块](../modules/qml-components-and-theme.md)。

## 评审与实施

- 核对组件边界、尺寸分配、焦点可达性、图标颜色和可访问性；保留已确认的滑块、磁贴尺寸和搜索箭头。
- 收敛输出工具条重复的尺寸配置，不增加未使用的公共组件或模型框架。
- 标题内容宽度应跟随工具分组实际需求，文字/字号变化后仍可滚动到焦点；以回归用例验证。
- 修复工具按钮字号未传递到文本的问题，并补回归验证。
- 根据用户反馈核对 VS Code 的 `color was not found [import]`：统一编辑器与 Cargo 的 Qt 工具、构建目录和导入路径，不屏蔽诊断。
- 清理无消费者的 id/token，注释仅描述当前职责、约束和必要原因，去除复刻历史与不准确的布局解释。

## 验收

- [x] 评审发现已记录并处理，默认视觉和当前交互保持一致。
- [x] 长文案、字号变化与窄窗口下，标题工具分组不重叠且焦点可达。
- [x] 重复配置、无用声明及失效注释已清理，无兼容例外。
- [x] 相关测试、QML lint、格式/静态检查和截图核对通过。
- [x] 文档、任务与索引一致，仅本地提交，不 push。

## 验证记录

- 2026-09-22：评审全部 Shell QML 与图标/测试接入。新增用例在修复前分别复现字号覆盖仍为 13px（预期 19px）、标题文案增长后搜索焦点右侧超出可滚动视区；修复后通过。
- 收敛输出工具条十个按钮的公共尺寸配置为面板内 `OutputAction`；删除无用 id、`logoHeight` / `radiusLarge`，修正 Layout、图标资源、强调/禁用状态等注释；默认外观、搜索箭头与当前 VTK 字样保持。
- VS Code 原日志显示使用扩展下载的 qmlls，未传构建/导入路径；新增可移植工作区配置后，日志确认自动切换到 Cargo Qt 6.11.2 与 `target/native/debug`。同参数 LSP 检查当前启用的 16 个 QML 文件零诊断，`color` 导入错误不再出现。未启用的 `AppNoBridge.qml` 不属于当前构建元数据，在此配置下仍有组件解析提示；切换变体需要对应构建树，不屏蔽诊断。
- macOS 26.3.1 arm64、Qt 6.11.2、Debug：`cargo build --locked`、`cargo test --locked --workspace` 通过（含 50/50 native/QML CTest）；`cargo format`、`cargo lint qmllint --check`、`cargo lint includes --check`、`cargo lint clang-tidy --check`、`git diff --check` 通过。
- Shell 回归在 `QT_QPA_PLATFORM=offscreen` 与 `QT_SCALE_FACTOR=1/1.25/1.5/2` 下通过，覆盖 1440/640px、内容增长、搜索焦点、页签和磁贴几何。1440px/2×、640px/1× 截图已目视核对；offscreen 跳过原生 WebGPU surface，不作为 GPU 验证。
- 另启动本机 Panta Preview 实际窗口，确认顶部、滑块、输出工具条与现有 VTK 字样显示正常，搜索框可输入并清空。未验证 Windows/Linux 实际窗口。
- 验证日志、截图与 LSP 诊断位于忽略目录 `artifacts/055/`；053 在索引和任务中仍为 planned。
