# Task 索引

任务文件是范围、验收、决策与证据的唯一详细记录；本页只提供编号、依赖、阶段和状态。当前产品能力见 [README](../README.md)，规范入口见[技术规范](standards/README.md)。新增任务从[模板](task/_template.md)创建；任务状态或依赖变化时同步本页。

## 状态约定

| 状态 | 含义 |
|---|---|
| draft | 缺少范围/验收，不能开始 |
| planned | 内容已编排，依赖尚未满足 |
| ready | 依赖完成且可开始，尚无实现 |
| in-progress | 正在实施 |
| implemented | 实现和当前可用平台验收已完成；仅待任务列出的 Windows / Wayland 等跨平台实机验收，不阻塞依赖任务 |
| blocked | 有具体阻塞，记录解除条件 |
| done | 验收完成且有证据 |
| deferred | 暂不排入当前交付 |
| cancelled | 已取消，保留编号和原因 |

## 基础设施队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 001 | [Cargo workspace 与 Rust 工具链](task/001-cargo-config.md) | M0 | — | done |
| 002 | [平台、工具链与 native 依赖基线](task/002-dependency-baseline.md) | M0 | 001 | done |
| 003 | [CMake/Ninja 原生构建骨架](task/003-cmake-native-skeleton.md) | M0 | 002 | done |
| 004 | [Cargo 调度 CMake 与运行入口](task/004-cargo-native-orchestration.md) | M0 | 001, 003 | done |
| 005 | [Qt/QML 主窗口与 C++ ViewModel](task/005-qt-qml-shell.md) | M0 | 004 | done |
| 006 | [Rust/C++ FFI 最小契约](task/006-rust-cpp-boundary.md) | 基础平台 | 004 | done |
| 007 | [VTK WebGPU 硬件窗口原生视口](task/007-vtk-quick-viewport.md) | M0 | 005, 031, 038 | deferred |
| 008 | [后台任务、错误与日志基础](task/008-tasks-errors-logging.md) | 基础平台 | 005, 006 | done |
| 009 | [OCCT 依赖与 STEP 适配冒烟](task/009-occt-adapter-smoke.md) | CAE 接入基础 | 003, 008 | done |
| 010 | [Netgen 接入与最小 Mesh IR](task/010-netgen-adapter-smoke.md) | CAE 接入基础 | 009 | done |
| 079 | [清理 Netgen mesh 销毁 workaround](task/079-remove-netgen-mesh-cleanup-workaround.md) | CAE 接入基础维护 | 010, 038 | planned |
| 011 | [统一测试与质量入口](task/011-test-quality-entrypoints.md) | 验证基础 | 007, 008, 010 | done |
| 012 | [CI 与依赖缓存](task/012-ci-reproducibility.md) | 验证基础 | 011 | done |
| 013 | [桌面安装布局与部署冒烟](task/013-desktop-deployment-smoke.md) | 交付基础 | 011 | deferred |
| 014 | [Python/uv 质量工具环境](task/014-python-tooling-foundation.md) | 验证基础 | 001 | done |
| 018 | [三平台 CI 基础](task/018-cross-platform-ci.md) | 验证基础 | 001, 002 | done |
| 019 | [GTest 测试配置与规则](task/019-gtest-native-testing.md) | 验证基础 | 003 | done |
| 020 | [托管引导：CMake/Ninja 二进制供给](task/020-toolchain-provisioning.md) | M0 | 004 | done |
| 031 | [预编译 native 依赖供给与 CMake package](task/031-prebuilt-native-dependencies.md) | 交付基础 | 002, 004 | deferred |
| 036 | [三平台 CI native 构建修复](task/036-ci-native-build-fix.md) | 验证基础 | 004, 005, 018 | done |
| 038 | [Native SDK 制品生产与发布](task/038-native-sdk-artifact-production.md) | 交付基础 | 031, 020 | deferred |
| 039 | [CI FFI 构建链修复](task/039-ci-ffi-build-fix.md) | 验证基础 | 006, 018, 036 | done |
| 040 | [统一 Cargo 构建编排入口](task/040-cargo-build-orchestration.md) | 验证基础 | 004, 039 | done |
| 041 | [构建产物归一与第三方缓存共享](task/041-build-artifact-consolidation.md) | 验证基础 | 004, 020 | done |

## 应用平台扩展队列

设计入口：[重要模块说明](modules/README.md)。下列任务部分已实施或完成，状态以表内为准。

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 022 | [UI 英文源文案与语言字典](task/022-ui-internationalization.md) | 应用平台扩展 | 005, 034, 086 | done |
| 023 | [跨平台路径与资源引用服务](task/023-cross-platform-paths.md) | 应用平台扩展 | 005, 006 | done |
| 024 | [工程运行时上下文与变量快照](task/024-runtime-context.md) | 应用平台扩展 | 008, 023 | done |
| 025 | [变量 DSL 解析、求值与存储](task/025-variable-dsl.md) | 应用平台扩展 | 024, 034 | done |
| 026 | [C++ 静态库边界与 QML 自动注册](task/026-static-qml-modules.md) | 应用平台扩展 | 005 | done |
| 027 | [开发模式 QML 重载与状态恢复](task/027-qml-state-reload.md) | 应用平台扩展 | 007, 024, 026 | planned |
| 029 | [QML 原子组件库与 Theme 尺寸参数化](task/029-qml-component-library.md) | 应用平台扩展 | 005 | done |
| 030 | [DSL 主题配置与运行期主题切换](task/030-theme-dsl.md) | 应用平台扩展 | 025, 029 | done |
| 050 | [QML 页面设计 HTML 先行复刻](task/050-qml-html-page-replica.md) | 应用平台扩展 | 028, 029 | done |
| 051 | [无边框外观与一体化窗口标题栏](task/051-integrated-window-titlebar.md) | 应用平台扩展 | 005, 029 | implemented |
| 052 | [QML 图标规范与首页布局优化](task/052-qml-icon-and-layout-polish.md) | 应用平台扩展 | 029, 050 | done |
| 053 | [默认视口立体 panta 字样](task/053-default-panta-wordmark.md) | 应用平台扩展 | 007, 081 | done |
| 054 | [顶部折叠图标与搜索框引导](task/054-titlebar-search-details.md) | 应用平台扩展 | 052 | done |
| 055 | [QML 组件评审与整理](task/055-qml-review-and-cleanup.md) | 应用平台扩展 | 052, 054 | done |
| 056 | [QML 周边 C++ 简化与性能评审](task/056-qml-native-review.md) | 应用平台扩展 | 055, 007 | done |
| 057 | [新建项目对话框与工程命令边界](task/057-new-project-dialog.md) | 应用平台扩展 | 023, 029, 055 | done |
| 059 | [打开工程后的 QML HTML 参考同步](task/059-open-project-html-reference.md) | 应用平台扩展 | 050, 052, 055, 057 | done |
| 060 | [已验收 HTML 的 QML 工程工作区同步](task/060-qml-project-workspace-reference.md) | 应用平台扩展 | 059, 057, 055 | done |
| 061 | [Home 与 Start & Learn 工具栏切换](task/061-ribbon-tab-navigation.md) | 应用平台扩展 | 060, 057 | done |
| 062 | [Ribbon 页签内容与公共渲染拆分](task/062-ribbon-tab-components.md) | 应用平台扩展 | 061, 060 | done |
| 063 | [STL 导入、导入选项持久化与工程工作区](task/063-stl-import-and-mesh-workspace.md) | 应用平台扩展 | 057, 060, 062 | done |
| 064 | [VTK 视口导航、笛卡尔坐标系与六面体定位](task/064-vtk-navigation-and-orientation.md) | 应用平台扩展 | 007, 063 | implemented |
| 065 | [VTK 视口缩放、右键旋转与方向过渡](task/065-vtk-zoom-and-cube-transition.md) | 应用平台扩展 | 007, 064 | done |
| 067 | [Rust 统一 STL 解析与 Mesh IR 领域校验](task/067-rust-mesh-domain-migration.md) | CAE 领域模块迁移 | 010, 063, 066 | done |
| 068 | [QML 工程 / 任务 Dock 与 Layers Dock](task/068-qml-project-and-layers-docks.md) | 应用平台扩展 | 063, 060, 062 | done |
| 069 | [工程 / Tasks / Layers 面板整体 Review 与性能消融](task/069-project-docks-review-and-ablation.md) | 应用平台扩展 | 068 | done |
| 073 | [FSM DSL 与首个异步 STL 视口资源激活](task/073-fsm-dsl-and-import-state-machine.md) | CAE 业务编排 | 008, 034, 035, 067, 072；首个消费者 080 | done |
| 074 | [Qt 交互状态机与导入交互](task/074-qt-interaction-state-machine.md) | 应用平台扩展 | 026, 063, 072 | done |
| 080 | [视口文档页签与 STL 按需激活](task/080-qml-viewport-document-tabs.md) | 应用平台扩展 | 007, 063, 068, 073, 086 | done |
| 081 | [QML 视觉语言与图标体系统一](task/081-qml-visual-language-and-iconography.md) | 应用平台扩展 | 029, 050, 069, 078, 080 | deferred |
| 085 | [Qt reduced-motion 原生属性迁移](task/085-qt-reduced-motion-preference-migration.md) | 应用平台扩展 | 080, 086, 002 | planned |
| 086 | [Qt 平台服务适配层](task/086-qt-platform-adapter.md) | 应用平台扩展 | 002, 005, 023, 034 | done |
| 087 | [语言设置界面](task/087-language-settings-ui.md) | 应用平台扩展 | 022, 029, 086, 115, 125 | done |
| 088 | [Mesh 资产旁置存储与按需加载](task/088-mesh-asset-sidecar-storage.md) | CAE 领域模块迁移 | 063, 067, 073, 080 | done |
| 090 | [Mesh Tool QML 界面](task/090-mesh-tool-qml-ui.md) | 应用平台扩展 | 050, 068, 081 | done |
| 091 | [Analysis Sequence 选择弹窗](task/091-analysis-sequence-dialog-ui.md) | 应用平台扩展 | 050, 068, 081, 090 | done |
| 092 | [方案分析序列与网格类型接线](task/092-plan-analysis-sequence-and-mesh-settings.md) | 应用平台扩展 | 057, 067, 080, 091 | done |
| 093 | [材料选择弹窗与默认材料](task/093-material-selection-dialog.md) | 应用平台扩展 | 068, 081, 092 | done |
| 094 | [Fill 工艺设置与保压曲线弹窗](task/094-fill-process-settings.md) | 应用平台扩展 | 081, 092, 093 | done |
| 095 | [Gate Location 工艺设置弹窗](task/095-gate-location-process-settings.md) | 应用平台扩展 | 081, 092, 094 | done |
| 096 | [Results 工具栏内容](task/096-results-ribbon.md) | 应用平台扩展 | 060, 081, 095 | done |
| 097 | [分析日志面板与结果目录](task/097-analysis-logs-dock.md) | 应用平台扩展 | 080, 091, 096 | done |
| 098 | [工艺确认跨平台 CI 修复](task/098-ci-metadata-confirmation.md) | 质量与构建 | 094, 095, 097 | done |
| 099 | [QML 组件与弹窗整理](task/099-qml-components-and-dialog-consolidation.md) | 应用平台扩展 | 029, 094, 095, 097 | implemented |
| 100 | [tests 整体整理与性能入口统一](task/100-tests-consolidation.md) | 验证基础 | 048, 099 | done |
| 113 | [Tools 工具栏内容](task/113-tools-ribbon.md) | 应用平台扩展 | 050, 062, 096 | done |
| 114 | [View 工具栏内容](task/114-view-ribbon.md) | 应用平台扩展 | 060, 062, 064, 080 | done |
| 115 | [平台应用菜单与 Settings 入口](task/115-platform-application-menu-and-settings-entry.md) | 应用平台扩展 | 051, 125 | implemented |
| 116 | [Rust 工艺参数约束统一暴露](task/116-rust-process-parameter-constraints.md) | 应用平台扩展 | 094, 095, 099 | done |
| 117 | [Rust 工程文档工作区状态](task/117-rust-project-document-workspace.md) | Rust 架构整理 | 067, 080, 107 | done |
| 118 | [实际依赖图与应用命令层设计](task/118-layering-lint-and-command-boundaries.md) | Rust 架构整理 | 106, 111 | done |
| 119 | [Rust 与 CMake 分层依赖 lint](task/119-rust-cmake-layering-lint.md) | Rust 架构整理 | 118 | done |
| 120 | [PR 按影响范围选择测试](task/120-pr-affected-test-selection.md) | 验证基础 | 100, 118 | done |
| 121 | [GUI 与 CLI 共享 Rust 命令入口](task/121-rust-gui-cli-command-layer.md) | Rust 架构整理 | 107, 118 | done |
| 122 | [Rust CLI 创建工程](task/122-rust-cli-project-create.md) | Rust 架构整理 | 057, 107, 121 | done |
| 123 | [Rust CLI 检查 STL](task/123-rust-cli-stl-inspection.md) | CAE 领域模块迁移 | 063, 067, 121 | done |
| 124 | [Rust CLI 导入 STL](task/124-rust-cli-stl-import.md) | CAE 领域模块迁移 | 063, 067, 073, 088, 121, 123 | done |
| 125 | [设置服务与 Qt 持久化适配](task/125-settings-service-and-qt-adapter.md) | 应用平台扩展 | 006, 023, 030 | implemented |
| 126 | [OCCT Shape 仓库与 Netgen 复用](task/126-native-shape-repository-and-netgen-reuse.md) | CAE 领域模块迁移 | 009, 010, 066, 067, 119 | done |
| 127 | [Rust 与 Qt 职责边界收尾](task/127-rust-qt-boundary-followup.md) | Rust 架构整理 | 057, 080, 088, 092, 107, 117, 124 | done |
| 128 | [冗余代码与状态收敛](task/128-dead-code-and-redundancy-cleanup.md) | Rust 架构整理 | 008, 029, 056, 109, 117, 127 | done |

## 验证与质量扩展队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 032 | [跨语言测试质量门禁与行为契约](task/032-cross-language-quality-gates.md) | 验证基础 | 011, 018, 019 | deferred |
| 033 | [高 DPI 缩放与多显示屏基础](task/033-display-scaling-and-multi-monitor.md) | 应用平台扩展 | 005, 007 | planned |
| 034 | [Rust Panta Artifact 解析与 TS/QM 编译入口](task/034-rust-panta-artifact-parser.md) | 应用平台扩展 | 001 | done |
| 035 | [`.pa` 格式化器与格式校验器选型](task/035-pa-formatter-and-validator.md) | 应用平台扩展 | 034 | done |
| 037 | [软件内增量更新基础](task/037-incremental-update-foundation.md) | 交付基础 | 005, 008, 013, 023 | planned |
| 042 | [三平台自有 C++ 统一 LLVM/Clang 工具链](task/042-unified-llvm-toolchain.md) | 验证基础 | 018, 032, 038 | in-progress |
| 043 | [根目录质量入口与测试聚合](task/043-root-quality-runner.md) | 验证基础 | 011, 032 | done |
| 044 | [Windows CI 停滞诊断与修复](task/044-windows-ci.md) | 验证基础 | 018, 042 | done |
| 045 | [Windows CI 分支代码审查与收敛](task/045-branch-code-review.md) | 验证基础 | 044 | done |
| 046 | [CI 触发拆分与缓存预算](task/046-ci-trigger-split-cache-budget.md) | 验证基础 | 018, 012 | done |
| 047 | [崩溃信号处理与日志落地](task/047-crash-signal-logging.md) | 验证基础 | 008 | done |
| 048 | [性能基线与性能测试体系](task/048-performance-testing.md) | 验证基础 | 011 | done |
| 049 | [CI 修复：Netgen Linux 制品 ISA 基线与 Windows ASan 链接](task/049-ci-mesh-sigill-windows-asan.md) | 验证基础 | 038, 042, 010 | done |
| 058 | [项目包提交后的 CI 回归修复](task/058-ci-regression-after-project-package.md) | 验证基础 | 057, 032, 043 | done |
| 070 | [GoogleTest 三平台 SDK 制品 CI](task/070-googletest-sdk-ci.md) | 验证基础 | 019, 031, 038 | done |
| 071 | [GoogleTest SDK 消费接入](task/071-googletest-sdk-consumption.md) | 验证基础 | 070, 031 | done |
| 075 | [CI 修复：Windows GoogleTest ABI 与 Qt benchmark lint](task/075-ci-windows-gtest-and-qt-lint.md) | 验证基础 | 070, 071, 046 | done |
| 076 | [Cargo 测试与质量 runner 维护](task/076-panta-tests-runner-maintenance.md) | 验证基础 | 011, 043 | done |
| 082 | [CI 修复：VTK benchmark moc 前置与标题条焦点滚动](task/082-ci-vtk-benchmark-moc-and-focus-scroll.md) | 验证基础 | 048, 076, 029 | done |
| 083 | [Windows 开发裸启应用的 Qt 运行库部署](task/083-windows-app-runtime-deployment.md) | 应用平台扩展 | 005, 042 | done |
| 089 | [CI 修复：STL benchmark RSS 头文件识别](task/089-ci-stl-benchmark-rusage-include.md) | 验证基础 | 048, 080 | done |

## 仓库与文档维护

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 015 | [文档入口与一致性整理](task/015-documentation-structure.md) | 文档维护 | — | done |
| 016 | [忽略规则与通用开发规范](task/016-repository-conventions.md) | 仓库维护 | — | done |
| 017 | [代码生命周期与 commit 一致性规范](task/017-code-lifecycle-and-commits.md) | 仓库维护 | — | done |
| 021 | [重要模块说明与后续任务规划](task/021-important-module-planning.md) | 文档维护 | — | done |
| 028 | [QML 原子组件与主题 DSL 规划](task/028-qml-theme-planning.md) | 文档维护 | — | done |
| 066 | [重库适配边界与 Rust 领域模块规划](task/066-native-domain-boundaries.md) | 架构与规范 | 065 | done |
| 072 | [FSM DSL 与 Rust 状态机方案评估](task/072-flow-state-machine-planning.md) | 架构与文档准备 | 008, 066, 067 | done |
| 077 | [进行中任务状态盘点](task/077-active-task-status-audit.md) | 文档维护 | — | done |
| 078 | [QML 性能基准登记规范](task/078-qml-performance-benchmark-policy.md) | 文档维护 | 069, 048 | done |
| 084 | [进行中任务状态复查（2026-09-28）](task/084-active-task-status-followup.md) | 文档维护 | — | done |
| 101 | [进行中任务 CI 验收复查](task/101-active-task-ci-acceptance.md) | 文档维护 | — | done |
| 102 | [弹窗 GUI 与文档页签读屏验收](task/102-gui-and-screen-reader-acceptance.md) | 验证基础 | 091, 092, 094, 095, 096, 080 | done |
| 103 | [macOS 工程文件选择器打开状态复查](task/103-macos-project-open-dialog.md) | 验证基础 | 063, 102 | done |
| 104 | [Windows 工程选择器路径回归修复](task/104-ci-project-dialog-paths.md) | 验证基础 | 103 | done |
| 105 | [CI 静态检查与模块文档修正](task/105-ci-and-module-documentation-fixes.md) | 验证基础 | 014, 021, 042 | done |
| 106 | [Rust 代码结构与职责边界评审](task/106-rust-architecture-review.md) | 架构与文档准备 | 066, 067 | done |
| 107 | [Rust 工程服务边界与修订耗尽修复](task/107-rust-service-boundaries-and-revisions.md) | 应用平台扩展 | 106 | done |
| 108 | [工程包写入协调与磁盘修订复核](task/108-project-write-coordination.md) | Rust 架构整理 | 107 | done |
| 109 | [Rust 后台执行容量与生命周期收敛](task/109-bounded-background-execution.md) | Rust 架构整理 | 108 | done |
| 110 | [DSL AST、解析、校验与输出模块整理](task/110-dsl-module-boundaries.md) | Rust 架构整理 | 107 | done |
| 111 | [构建供给与质量命令模块整理](task/111-build-and-quality-module-boundaries.md) | Rust 架构整理 | 107 | done |
| 112 | [FFI 私有单元测试归属修正](task/112-ffi-unit-test-placement.md) | Rust 架构整理 | 107 | done |
| 129 | [仓库文档去重与实现对齐](task/129-repository-docs-alignment.md) | 仓库与文档维护 | 128 | done |
