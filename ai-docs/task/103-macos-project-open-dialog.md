# 103 — macOS 工程文件选择器打开状态复查

- 状态：done
- 阶段：验证基础
- 依赖：063 已实现的工程读写、102 已取得的文件选择器复现场景
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-02 / 2026-10-02

## 目标与范围

任务 102 的 macOS 真实 GUI 验收发现：通过 Start & Learn → Open Project，用原生 Go to Folder 定位本次创建的 `.panta` 文件，文件选中后 Open 仍 disabled，重新选中及双击也未打开。清单存在且包含已确认配置。尚未确认是 Qt 原生文件过滤、临时 app bundle 或应用实现问题；不得据此推断工程数据损坏。

优先通过自动化测试检查应用配置、接线及失败时的状态保护，再补正常启动方式与临时 bundle 的原生选择行为对照。任务 102 的窗口截图与 AX 记录是首次观测，不是根因结论。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [QML 规范](../standards/qml.md)

## 验收标准

- [x] 正常启动与临时 bundle 对照，确认 `.panta` 过滤及原生选择行为。
- [x] 工程能够从文件选择器打开，已保存 Fill / Gate Location 设置回显正确；取消不修改工程。
- [x] 必要修复、回归验证与文档同步；GUI 验收单独记录。

## 验证与工作记录

- 2026-10-02：用户确认保留原验收范围并要求继续完成；恢复真实窗口对照与 Gate Location 回显验收，自动化证据继续保留。
- 2026-10-02：登记任务 102 发现的阻碍。测试文件为 `/private/tmp/panta-gui-102/gui-acceptance-102/gui-acceptance-102.panta`，本次临时验收数据，不是用户工程。
- 2026-10-02：047 三平台 CI 验收完成后开始本任务。先核对 OpenFile / `.panta` name filter、临时工程与文件类型，再用当前 native 构建对照原生文件选择器；复现后再决定是否修改代码，不将 ViewModel 重开测试代替 GUI 证据。

### 2026-10-02 测试优先排查

用户要求优先通过自动化测试排查。源码当前为 OpenFile、`.panta` name filter，临时工程是普通可读文件。本轮使用同一 native 构建的临时 bundle：CUA 点击后 AX 未出现 selected，Open disabled；在列表按 Down 后 AX 出现 selected，Open enabled，点击后成功打开工程；Fill 设置回显 45°C。截图在忽略目录 `artifacts/task-103/open-enabled.png`、`fill-reopened.png`。这证明该路径可用；此前点击未建立选择状态，尚不能认定 Qt 过滤器或工程实现故障。

按用户要求停止继续 GUI 对照，转为既有 Shell 模块测试：检查实际 FileDialog 的 OpenFile、筛选规则和原生默认选项；覆盖 accepted / rejected 的 QML 接线、编码路径及打开失败保留当前工程。本测试验证应用边界，不模拟 Cocoa 点击或声明原生鼠标选择通过。既有 Rust / ViewModel 的持久化重开测试继续复用。普通二进制 CUA 绑定返回 Invalid app，正常启动方式的对照尚未补齐。

新增回归使用普通名称及中文、空格、`#`、`%` 路径，检查取消、选择后文件被删除、损坏清单及成功打开。失败时检查当前工程路径、名称、未保存状态、工艺设置、文档与 Ribbon 不变；成功时检查工程切换及错误状态清除。QML 仅新增 FileDialog 的稳定 `objectName`，未修改产品行为。

测试编写时校正两个前提：通过 `fileMode` 属性的元枚举读取 OpenFile，避免同名扩展枚举干扰；FileDialog 会拒绝预选不存在的文件并保留原选择，因此删除用例先选择真实文件再删除。两处均为测试构造修正，不作为产品缺陷。

最终 `cargo test --locked --workspace` 通过，包含 qmllint、69/69 CTest 及 Rust / 文档测试；日志为 `/tmp/panta-103-tests-reviewed.log`。`cargo format --check`、`cargo lint --check` 和 `git diff HEAD --check` 均通过；静态检查日志为 `/tmp/panta-103-lint.log`。整体 review 确认使用公开 Qt 属性 / 信号与既有测试入口，无新增测试框架或产品行为变更。

临时应用已退出（进程查询无匹配），wrapper 已删除；保留临时工程及截图，未改动用户工程。无产品行为修复前不增加兼容或原生对话框替代路径。

### 2026-10-02 原验收范围补齐

用户要求再次尝试正常启动验收；复查窗口清单与进程，并尝试仓库约定的启动脚本 wrapper，脚本直接 exec 原构建路径，以检查是否能绑定正常运行的二进制。

`cargo build --locked` 通过后直接启动 `target/native/debug/app/panta-native`。沙箱内启动无法创建 Qt 平台窗口；获准在沙箱外启动后正常运行，但 CUA 绑定仍返回 Invalid app，已请求用户人工检查该窗口的原生文件选择行为，等待实际结果。

使用相同构建的临时 bundle（复制二进制，SHA-256 与原文件一致）完成工具可操作的真实窗口验收：原生选择器定位临时工程目录，在列表按 Down 建立 selected 状态、Open 启用并成功打开；Gate Location 回显模温 50°C、熔温 230°C、Advanced gate locator、浇口数 4，与清单一致。临时改模温为 55 后 Cancel，再打开仍为 50，未保存修改。再次打开工程选择器、选择文件后 Cancel，当前工程名称、计划与文档保持不变；自动化同时检查未保存状态及设置保护。

新增证据为忽略目录 `artifacts/task-103/gate-reopened.png`、`gate-cancel-reopened.ax.txt`、`open-cancel.ax.txt`。CUA 的文件名称点击未建立 selected，键盘选择可以建立；工具点击结果不能据此认定用户鼠标选择失败。验收 bundle 已退出并删除，原生对照等待用户结果，不改验收范围。

### 2026-10-02 再次尝试与对照结果

原裸二进制进程已退出，按进程名绑定仍返回 Invalid app。重新使用仓库约定的脚本 wrapper 后 CUA 成功绑定；脚本仅切换仓库目录并 exec 原构建路径，进程查询确认运行 `/Users/yuki/eit/panta/target/native/debug/app/panta-native`，未复制或修改二进制。这是正常构建路径的窗口验收启动适配，仍经过 LaunchServices wrapper，不声明工具已经能够直接绑定裸进程。

该窗口的原生 Open Project → Go to Folder 能显示临时 `.panta` 工程；列表按 Down 后文件 selected、Open 启用，点击 Open 成功打开并显示 `gui-acceptance-102`、`part-a.stl`、Gate Location 与 Process Settings (Custom)。与前一轮复制二进制 bundle 的选择及打开结果一致，未复现“已有 selected 状态而 Open 仍 disabled”的故障。工具 row 点击仍未建立 selected；以实际 AX 状态区分点击动作与选择成功，不把该工具行为归因于产品。

证据为忽略目录 `artifacts/task-103/original-open-enabled.png`、`original-open-enabled.ax.txt`、`original-opened.png`、`original-opened.ax.txt`。窗口已退出，进程查询无匹配，脚本 wrapper 已删除。无需用户补充正常启动结果；本轮只更新验收文档，`git diff HEAD --check` 通过，沿用上一批已通过的 Cargo 聚合测试与质量检查。

## 清理与兼容例外

临时 app wrapper 已删除；无废弃实现或兼容例外。

## 完成摘要

应用配置与 QML 打开接线的自动化回归、原构建路径与复制二进制 bundle 的原生选择 / 打开对照、Fill / Gate Location 回显及取消验收均通过。此前工具点击未建立选择状态，未复现已选中文件无法打开的产品故障；无需修改产品行为。任务完成，自动化与真实窗口证据分别记录。
