# 099 — QML 组件与弹窗整理

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：029, 094, 095, 097
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-01 / 2026-10-09

## 目标与背景

落实现有原子组件 → 组合组件 → 业务界面 → 页面拼装四层规则，集中视觉参数并减少控件和弹窗重复。按已确认方案统一下拉、按钮与勾选控件尺寸，窗口尺寸保持不变；下拉选项使用 Import 的展开样式，右侧指示统一为材料选择的向下实心三角。整体沿用 Import 的 Basic 外观，仅替换右侧箭头；不保留白底细边框变体。网格主按钮改为普通字重。

## 必读

- [QML 规范](../standards/qml.md)
- [组件与 Theme 模块](../modules/qml-components-and-theme.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 先更正文档中的职责边界和当前状态，再整理 Theme、实际复用的原子控件、表单组合与弹窗公共结构。
- 统一下拉框的展开选项样式与指示图标，下拉高度统一为 Process Settings 的 30 个逻辑像素并归入原子层；按钮和 CheckBox 按已确认尺寸统一，其他布局和窗口尺寸保持现状。
- 弹窗统一公共实现；业务草稿、上下文快照、校验反馈与保存状态仍由具体界面持有。
- 不改变 Rust / C++ 业务接口、工程持久化格式、业务默认值和约束，不实施主题 DSL。
- 按用户补充要求优化 GPU 基准：默认通过 QQuickRenderControl 渲染到硬件纹理，统计每帧完成耗时和 GPU 时间戳；提取 Qt Quick 离屏辅助类供其他测试复用，分别报告离屏完成、窗口呈现和 GPU 时间戳；同机同场景比较两种路径，只有结果支持等价时才移除真实窗口性能模式，不混用历史基线。

## 已确认的组件方案

- 普通文字按钮使用 ThemedButton 原子组件，主操作由 primaryAction 表达；按钮区布局归组合层。
- 普通按钮、输入框与勾选框使用固定 Theme 圆角；胶囊页签和圆形指示保留半高 / 半径计算。
- 图标尺寸使用固定 Theme token。
- 普通勾选统一为蓝底白勾，Import 的日志勾选迁移，指示框统一为 Logs 的 16 × 16，由原子组件管理；结果选择继续互斥，材料模式继续使用圆形指示。
- 下拉高度统一为 30，移除调用方高度覆盖；按钮和 CheckBox 按已确认尺寸统一，已有按钮排列保持现状。

Theme 分类按通用视觉、控件、Shell / Ribbon / 侧栏 / 文档 / 表单 / 日志布局及各类窗口约束归位，保持现有属性契约。

## 侧栏范围补充

TasksPanel 更名为 SidebarPanel，Tasks / Tools / Shared Views 内容拆入 Panels/Sidebar；宿主装配页签并转发信号，不依赖任意父级对象。工程任务页隐藏滚动条、保留滚动；Layers 当前无滚动容器，保持不显示滚动条。材料模式的圆形指示迁入 ThemedRadioButton。

菜单页签通过 MenuTabButton 预留普通 / 加粗字重的最大文字宽度，选中时不改变相邻项位置，翻译和字体变化时重新测量。

普通文字按钮高度统一 24，弹窗操作按钮宽度统一 92；CheckBox 行高统一 24、padding 为 0。数字输入属于单控件，迁入 ThemedNumberField，父布局分配由调用方负责。

运行日志补充：等宽字体通过 Qt 系统字体查询取得真实字体族，归 C++ 平台适配层；核对 macOS 输入法日志与 VTK 设备销毁日志，不修改 Rust 业务或屏蔽真实设备错误。

## 实施步骤

1. 核对 QML 规范及模块说明，明确四层职责与样式基准。
2. 盘点视觉数值与公共控件，按语义提取 Theme 参数，默认值沿用现有值。
3. 提取实际复用控件与弹窗结构，迁移调用点并清除旧实现。
4. 验证布局、键盘焦点、关闭及异步保存行为；同步组件清单和任务证据。

## 清理与兼容例外

替换时删除重复控件实现与失效引用；无兼容例外。

## 验收标准

- [x] 文档明确四层规则、Dialog 归属和原子控件的布局边界。
- [x] 下拉选项样式统一为 Import 基准，指示图标与材料选择一致。
- [x] 下拉高度统一为 30、勾选指示框统一为 16 × 16，普通按钮统一 92 × 24、CheckBox 行高 24 / padding 0，窗口尺寸保持不变，视觉参数按语义集中管理。
- [x] 面板与弹窗关闭按钮上、右间距一致，沿用当前右间距。
- [x] 弹窗公共实现复用，业务状态不进入组件库；旧实现已清理。
- [x] 焦点、Tab / Enter、取消、关闭与保存防重入行为保持有效。
- [x] 新组件映射到现有 CPU / GPU 手动基准场景，记录真实验证及限制。

## 验证计划与结果

环境：仓库根目录，macOS 26.3.1 / arm64，Apple M4，Qt 6.11.2，native Debug。手动基准不进入 CI 时间门禁；通过表示场景断言及采样完成，无固定耗时阈值，不宣称相对旧版性能提升。

| 命令 / 场景 | 实际结果 |
|---|---|
| `cargo test --locked --workspace` | Rust 测试及文档测试通过，native / QML 67 / 67 通过；新增固定控件尺寸和菜单字重不改变宽度的断言。初次复查发现旧 96px 断言和按钮分数像素居中偏移，已修复后重跑。 |
| `cargo build --locked`、`cargo lint --check`、`cargo format --check`、`git diff HEAD --check` | 全部通过，已完成提交前复验。 |
| `cmake --build target/native/debug --target panta_qml_cpu_benchmark panta_qml_gpu_benchmark --parallel 4` | 两个手动目标编译通过。 |
| `QT_QPA_PLATFORM=offscreen QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/qml/panta_qml_cpu_benchmark` | 7 通过、0 失败 / 跳过；1 次预热、31 次构造采样。最终夹具重跑：1 / 8 / 24 组控件构造 p95 为 0.837 / 3.312 / 8.777 ms。Qt offscreen 平台字体匹配与 Cocoa 不同，可能输出系统 FixedFont 的 Monospace 别名诊断，不代表原生 Menlo 查询失败。 |
| `QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/qml/panta_qml_gpu_benchmark` | Metal 硬件离屏：5 通过、0 失败 / 跳过。1000 × 700 逻辑尺寸，DPR 2，2000 × 1400 纹理，每场景 30 帧预热 + 3 × 60 帧。 |
| `PANTA_BENCHMARK_PRESENTATION=1 QT_QUICK_CONTROLS_STYLE=Basic` + 同一 GPU 二进制 | Metal 真实窗口：5 通过、0 失败 / 跳过，96783 ms；通过临时 `.app` 中的同一二进制启动。采集全部请求帧，并报告有效 GPU 完成时间戳观测数。 |

离屏与窗口 GPU 时间戳不一致，保留独立模式和基线。以同机 24 组控件为例：离屏 p50 / p95 约 0.126 / 0.129 ms，窗口 p50 / p95 约 1.120 / 1.238 ms（有效观测 180 / 180）；呈现间隔约 16.67 ms，包含显示器刷新等待。离屏完成耗时与 GPU 时间戳分别报告，不能互相替代。

初次真实窗口采样曾因遮挡超时，结果作废；初始化帧无时间戳、窗口流水线时间戳尚未就绪和夹具内额外 Window 引起的动画驱动冲突已分别处理。正式离屏样本无有效时间戳、窗口完全没有有效时间戳观测、设备丢失及帧呈现超时仍失败，不降低采样帧数或改为软件渲染。

真实整应用逐弹窗截图未取得：computer use 不能绑定临时应用，随后绑定基准窗口也超时；硬件性能基准通过不代表整应用像素验收。跨平台真实窗口视觉复核仍待补充，任务保持 in-progress。

## 风险与回退

控件提取可能影响默认 implicit size、焦点和弹窗生命周期；迁移时逐项保留现有配置，公共组件不接收业务 ViewModel。回退仅涉及本任务改动，不修改工程数据。

## 基准场景映射

| 组件 / 页面 | CPU 构造场景 | GPU 呈现场景 |
|---|---|---|
| 通用按钮、菜单页签、数字输入、选择控件、DialogButtonRow / FormSection | ControlGallery，1 / 8 / 24 组 | ControlGallery，1 / 8 / 24 组 |
| DialogWindow / DialogFrame 与标题栏 | 既有 AnalysisSequenceDialog 构造场景 | 不放入离屏控件夹具；窗口视觉另行验收 |
| SidebarPanel 与三个页面、Layers、关闭入口 | 既有工程 Dock 0 / 1 / 100 / 1000 条目场景 | 既有工程 Dock 0 / 1 / 100 / 1000 条目场景 |

PlatformFonts 在各引擎首次读取 Theme 时创建；不进入 Rust 领域服务。

## 决策与工作记录

- 2026-10-01：按 Yuki 要求先检查文档；沿用四层架构。Dialog 公共结构允许统一，下拉展开样式使用 Import、指示使用材料选择的实心三角，尺寸保持现状。已完成文档整理，开始公共关闭入口调整。

- 2026-10-01：用户确认所有下拉完整沿用 Import Basic 外观，仅换实心三角，高度统一 30；弹窗按钮统一 92 × 24，普通文字按钮高度 24，普通勾选行高 24、无 padding、指示框 16 × 16。ThemedButton 与 DialogButton 合并，仅保留 ThemedButton。菜单选中不改变宽度。
- 2026-10-01：字体修复使用 Qt 系统 FixedFont 的实际字体族。当前锁定 VTK 源码在 DeviceLost 回调中对 Destroyed 原因输出 INFO，视口释放存在 Finalize / Destroy；不屏蔽真实设备错误。输入法 mach port 日志按用户要求暂缓，待其复现场景。

- 2026-10-01：修复 PlatformFonts 未定义：Theme 显式导入模块，Shell 从纯资源模块调整为含静态插件的模块；应用、现有检查及基准入口显式导入插件，避免静态链接丢弃 C++ 类型注册。输入法日志已由用户补充为启动时出现且可正常使用，按用户要求暂不处理。

- 2026-10-01：当前 Qt Cocoa 平台字体查询为 FixedFont → Menlo → 实际 Menlo。复查移除弹窗 OK / Cancel / Help 的重复宽度计算和主按钮边框配置，保留长文本动作的显式宽度覆盖；同步组件和字体文档。

- 2026-10-01：用户要求最终复查、实际执行 CPU / GPU 性能基准和聚合验证，通过后提交。补充固定尺寸及菜单选中字重不影响宽度的回归断言，整理手动基准夹具格式。

- 2026-10-01：最终复查提取 Qt Quick 硬件离屏辅助类，固定逻辑尺寸和 DPR；窗口 GPU 时间戳按已完成观测统计并报告数量，保留完整帧呈现断言。纯 Item 控件夹具避免额外 Window 接入默认动画驱动；删除重复 Browse 宽度 token，数值输入高度使用固定 30。扩展到全部 GPU 基准的要求登记为任务 100，分批提交。

## 完成摘要

组件与调用点迁移、固定尺寸回归及 CPU / Metal 离屏和窗口性能验证已完成。本次提交清理 TasksPanel 和工艺内联控件；无兼容分支。整应用视觉及其他平台验收尚未齐备，保持进行中；其他 GPU 基准的共享整理已由 100 完成；102 等后续补验覆盖部分界面，仍不等于整应用与跨平台视觉验收闭环。2026-10-09 复查见 [101](101-active-task-ci-acceptance.md#2026-10-09-当前复查)。
