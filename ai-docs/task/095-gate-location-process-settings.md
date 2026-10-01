# 095 — Gate Location 工艺设置弹窗

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：081, 092, 094
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-10-02

## 目标与背景

Analysis Sequence 为 Gate Location 时，点击 Plan tasks 的 Process Settings 打开浇口定位设置。沿用现有工艺弹窗视觉风格，截图仅作为字段与布局参考，不显示左侧图片。

## 必读

- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [原生领域边界](../architecture/native-domain-boundaries.md)

## 范围与实施步骤

1. 先完成 HTML：注塑机、模具表面温度、熔体温度、定位算法和浇口数量，提供 OK / Cancel，未实现的 Edit / Select / Advanced Options / Help 禁用。
2. Yuki 确认设计后迁移 QML，Process Settings 按当前分析序列打开对应弹窗。
3. 实现 Rust 默认值、校验、按导入记录保存与后台确认，复用 094 元数据事务；Qt 只适配候选与展示快照。具体机器目录与算法执行尚未实现，本任务不接入求解器。

HTML 使用 Default machine 和 Advanced gate locator 展示选择字段；温度示例为 40 °C / 230 °C，浇口数量默认 1、允许 1–10。HTML 仅保存当前页面的演示状态，不写工程文件；这些示例值不表示设备数据库或求解算法已经接入。

## 验收标准

- [x] HTML 的 Gate Location 工艺弹窗由 Yuki 确认。
- [x] Gate Location 与 Fill 打开各自设置，切换序列不混用候选或已确认配置。
- [x] 温度输入等宽，底部按钮与材料 / Fill 弹窗一致，取消不提交。
- [x] QML 文案与翻译齐全，未实现入口禁用。
- [ ] Rust 校验与后台持久化完成，并验证重开回显和失败保留旧设置。
- [x] 真实窗口验收及适用 Cargo 聚合检查结果已记录。

## 验证计划与结果

工作目录为仓库根目录。HTML 阶段检查脚本语法、差异空白并通过浏览器查看设计；QML 与持久化阶段使用适用的 Cargo 聚合入口，真实窗口验收单独记录。

- 2026-09-30：登记任务，开始 HTML 设计；尚未实现 QML 和领域持久化。
- 2026-09-30：HTML 字段及按序列路由已实现，`node --check ai-docs/qml-html/shell.js` 与 `git diff --check` 通过。浏览器安全策略拒绝访问本地 `file:` 页面，未取得渲染验收证据，后续已由 Yuki 确认 HTML 设计；未运行功能测试。

- 2026-09-30：`cargo build --locked`、`cargo format --check`、`cargo lint`、脚本语法与差异空白检查通过。评审核对按序列路由、配置隔离、整数 DTO 转换、后台互斥和失败保留旧状态；未新增或运行行为测试，未完成真实窗口验收。

## 清理与兼容例外

扩展既有 Process Settings 路由，不复制公共 Shell；Fill 和保压曲线原有重复控件样式迁入 `ProcessSettingsControls` 并清理旧内联定义。无兼容例外。

## 完成摘要

HTML 已由 Yuki 确认，QML 与 Rust 后台持久化已实现。构建与静态检查结果记录于本任务；已有自动化回归及本轮真实窗口证据；原生文件选择器阻碍了工程重开 GUI 验收，保留 in-progress，问题见任务 103。

## 工作记录

- 2026-09-30：Yuki 确认 HTML，开始 QML、跨语言投影与 Rust 后台持久化；机器目录与高级设置保持禁用。

## 实施记录

- `.panta` 的 `gate_location_settings` 按导入记录 ID 保存设备引用、温度、算法和浇口数量，缺少条目时由 Rust 提供默认配置。算法只支持 `advanced-gate-locator`，设备只支持 `default-machine`，尚未接入算法执行和设备目录。
- 温度为有限值且不低于绝对零度，浇口数量为 1–10 的整数。后台确认复用材料 / Fill 元数据写入锁与 Qt 定时器；成功后发布快照，失败保留旧状态。
- Fill 与 Gate Location 的任务完成状态分别读取当前分析序列的配置，切换序列保留已保存配置。
- `ProcessSettingsControls` 共用按钮、分组、输入与下拉样式，Fill / Gate Location / 保压曲线消费者同步迁移；共享组件不持有领域状态。采用 Qt 官方支持的限定名内联组件引用，依据：[QML 类型定义](https://doc.qt.io/qtforpython-6.8/overviews/qtqml-documents-definetypes.html)，查阅日期 2026-09-30。

- 2026-09-30：Yuki 报告启动失败，根因为共用内联组件错误使用 `ComponentBehavior: Bound`，跨文件实例化被运行时拒绝。移除该声明，组件只引用自身 ID 与 Theme；工艺表单内容宽度改为明确绑定 ScrollView，消除创建阶段空 parent 依赖。此前构建与 lint 通过未证明运行时加载通过。

- 2026-09-30：再次复核工程配置隔离、DTO 整数转换、元数据写入锁与 Qt 通知生命周期。修正后的 `cargo build --locked`、`cargo format --check` 与 `cargo lint` 通过；尝试用临时 app wrapper 检查实际启动窗口，CUA 仍绑定超时，清单显示应用未运行，未取得真实窗口证据，已清理 wrapper。

## 2026-10-02 补充验收

任务 102 在 macOS 26.3.1 / Apple M4 / Qt 6.11.2 实际应用中验证：独立机器 / 算法默认项、温度等宽、底部按钮及禁用入口正确；Fill 保存 45°C 后 Gate Location 默认仍 40°C。温度 / 数量修改后取消，重新打开为原值；浇口数 11 时 OK disabled，合法值确认后显示 Custom。磁盘清单保存温度 50°C、浇口数 3。

在本轮临时工程创建 `.panta.tmp` 同名目录阻断写盘：确认 4 个浇口时弹窗显示错误，磁盘及取消后重开的弹窗仍为 3；删除障碍后重试保存成功，磁盘为 4。切回 Fill 仍为 45°C，配置隔离通过。截图为 `artifacts/task-102/gate-location.png`。任务 098 已补后台确认、重开、非法清单与写盘失败回归；最新 CI run 36890328640 对应 `a0f0872`，19 个 job 全部成功。

本轮从原生文件选择器重开工程时，选中 `.panta` 文件后 Open 仍禁用，未取得该 GUI 路径的重开回显证据；由 [103](103-macos-project-open-dialog.md) 对照正常启动与临时 bundle 定位。保留未完成的持久化 / 重开验收项，不把读取清单或自动化重开替代本轮失败的 GUI 路径。
