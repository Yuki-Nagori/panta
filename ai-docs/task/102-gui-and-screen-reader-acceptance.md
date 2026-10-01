# 102 — 弹窗 GUI 与文档页签读屏验收

- 状态：done
- 阶段：验证基础
- 依赖：091, 092, 094, 095, 096, 080
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-02 / 2026-10-02

## 目标与范围

按用户要求优先补较容易的真实应用 GUI 验收，再尝试文档页签 VoiceOver 读屏。优先检查 Analysis Sequence、Fill、Gate Location 与 Results；仅在各原任务全部条件都有证据时同步关闭。测试使用临时工程，避免修改用户工程。补充复核文档关闭按钮的旁白导航；最终用户确认进入页签组后可访问关闭按钮，无需修改产品实现。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [文档规范](../standards/documentation.md)

## 验收标准

- [x] 构建后通过实际应用窗口检查，记录具体操作及结果。
- [x] 检查弹窗选择、取消、确认与 Results 导航和布局；发现问题明确记录。
- [x] 尝试 VoiceOver；区分 AX 可访问性文本与真实读屏证据，完成关闭按钮导航复核。
- [x] 原任务与索引按实际证据同步，临时应用退出并清理 wrapper。

## 验证与工作记录

- 2026-10-02：开始验收；首次连接旧 wrapper 时文件不存在，按仓库要求构建后重建临时 wrapper。

## 清理与兼容例外

不涉及兼容代码；临时 wrapper 放入忽略目录，验收后删除。

## 完成摘要

本轮 GUI 补验已完成，关闭 091、094、096；080 经用户旁白补验通过，一并关闭。095 已补布局、配置隔离、取消、校验与写盘失败恢复，原生工程重开问题由 103 跟踪，保持进行中。092 的多零件与 093 的完整异步材料窗口验收未纳入本轮，不作通过声明。

## 本轮环境与操作证据

- 仓库根目录 `cargo build --locked` 通过，日志 `/tmp/panta-102-build.log`；实际产物为 `target/native/debug/app/panta-native`。仓库路由所列 `target/native/debug/panta-native` 在此构建不存在。
- 脚本 exec wrapper 能启动但 CUA 无法绑定进程；改为临时 bundle 内复制同一构建的二进制，不修改产品源码、QML 或配置，即可连接 AX 并截图。macOS 26.3.1 / Apple M4 / Qt 6.11.2，截图为 DPR 2。首帧白屏截图不计通过，Raise 后确认完整渲染才保存证据。
- 临时工程创建在 `/private/tmp/panta-gui-102/`，导入本轮生成的单三角 STL；未修改用户工程。截图保存在忽略目录 `artifacts/task-102/`，不提交图形产物。

| 场景 | 实际结果 |
|---|---|
| Results | 顶部与 Home 两个入口均可进入；11 组、动画及刻度、Windows / Locking 正常，工具禁用，无伪造数据。截图 `results.png`。 |
| Analysis Sequence | 十个选项与禁用 More 显示完整；Down 选择 Fill + Pack 后 Cancel 保留 Fill，鼠标确认 Gate Location 后显示更新，可切回 Fill。截图 `analysis-sequence.png`。 |
| Fill / Holding Profile | 600 高度完整显示、输入等宽；曲线取消保留 10 s；温度取消保持 40，确认后重开弹窗为 45，磁盘及切回序列保持 45。截图 `fill.png`、`holding-profile.png`。 |
| Gate Location | 温度默认独立；取消回滚、浇口数越界禁用 OK、确认保存与 Custom 状态通过；写盘障碍时错误反馈、保留旧配置，移除障碍后重试通过。截图 `gate-location.png`。 |
| 工程重开 | 原生文件选择器选中本轮 `.panta` 后 Open 仍禁用，重选及双击无效。根因未知，任务 103 跟踪；此路径未通过。 |
| VoiceOver | 自动连接系统旁白应用超时，不作自动化朗读证据。Yuki 最初报告关闭按钮无法朗读，随后确认普通 Tab 可到达关闭按钮，使用 Control + Option + Shift + 下方向键进入页签组后旁白也可访问并朗读；人工验收通过。 |
| 清理 | 关闭验收窗口后 CUA 应用清单确认 PantaGuiAcceptance `isRunning: false`；两个临时 wrapper 已删除。保留临时工程与截图便于 103 复现。 |

## 文档复查

状态与本地链接、`git diff HEAD --check` 检查通过。本轮只构建并操作真实 GUI，不重复执行已由当前提交 CI 通过的功能套件；未增加产品代码或测试框架。

- 2026-10-02：按用户更正恢复 080 为 in-progress，本任务继续检查文档关闭按钮读屏；不将初次笼统确认当作最终通过。

- 2026-10-02：源码复核确认文档关闭控件有 Accessible.Button、Close 名称、pressAction、activeFocusOnTab 与键盘激活处理；既有回归检查角色、名称及强制焦点，但不证明 VoiceOver 导航可达。Qt 6.11.2 [Accessible 文档](https://doc.qt.io/QT-6/qml-qtquick-accessible.html)说明 Button 默认可聚焦、focused 跟随实际焦点。本轮先确认旁白导航的实际结果，不凭悬停无声增加冗余焦点绑定或改动产品样式。

- 2026-10-02：用户曾怀疑 DocumentTabBar 被作为整体，源码核对期间进一步确认普通 Tab 可切到关闭按钮，Control + Option + Shift + 下方向键进入页签组后旁白可访问并朗读关闭按钮。按最终人工证据关闭 080 和本任务；未改动产品代码，也未新增测试。
