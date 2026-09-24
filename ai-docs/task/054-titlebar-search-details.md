# 054 — 顶部折叠图标与搜索框引导

- 状态：done
- 阶段：应用平台扩展
- 依赖：[052 首页优化](052-qml-icon-and-layout-polish.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 范围

按截图优化激活动画右侧的折叠 SVG：轮廓更紧凑，颜色统一为工具条图标色；搜索框左侧添加独立右箭头引导区，修正边框接缝和裁切。沿用当前搜索行为，不新增菜单或搜索服务。

## 规则与验收

遵循 [图标](../standards/icons.md)、[QML](../standards/qml.md)、[验证](../standards/validation-and-review.md) 与 [提交](../standards/commits.md) 规范。

- [x] SVG 遵循 Mono 规范，统一颜色与大小。
- [x] 搜索框左侧箭头、边界与文字对齐；宽窄窗口无重叠。
- [x] 构建、QML lint、已有图标/布局测试通过，截图核对。
- [x] 文档与任务同步，本地提交，不 push；无废弃实现或兼容例外。

## 验证记录

- 2026-09-22：登记任务后实施。
- 2026-09-22：按后续截图去掉搜索左侧箭头的外框白边，保留透明容器，直接透出工具栏渐变背景。

- 2026-09-22：按用户撤回保留装饰性右箭头，无悬停/点击聚焦行为；移除尝试的按钮与 i18n 文案。
- 提交前 `cargo test --locked --workspace` 通过（含图标资源与 1440/640px 布局回归）；`cargo format --check`、`cargo lint includes --check`、`cargo lint qmllint`、`cargo lint clang-tidy` 与 diff 检查通过。本机实际窗口确认箭头无白边、搜索文字未裁切、折叠图标颜色统一。
- 本次提交仅包含顶部细节、图标清单与任务记录；无兼容例外，不 push。
