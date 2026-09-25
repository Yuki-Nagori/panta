# Task 索引

采用“先写 task，再做实现”的工作方式。001–006、008–012、014、018–020、026、036、039–041、043–046 已完成，对应实现与验证见各任务记录。GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)（commit `48ea4b4`）三平台成功，覆盖 Cargo 测试、native CTest、覆盖率门槛和格式检查。2026-09-24 `cargo coverage` 函数/行覆盖率为 90.34% / 93.95%，超过当前 89% / 92% 门槛，但任务 032 要求的 100% 目标、C++ 覆盖门禁及分模块回归门禁仍未完成。031 已登记三平台 VTK WebGPU 与 OCCT/Netgen SDK manifest；007 仍待真实窗口/硬件渲染生命周期验收，010 的 Netgen 适配与调用侧销毁 workaround 已由维护者确认通过 commit `376f338` 三平台 CI。任务 079 跟踪 SDK 修复进入消费基线后的兼容特例清理。其他未完成项及其剩余条件见下方 task 表和任务 077 的逐项审计。任务详情是范围、验收与证据的主记录，索引提供队列总览，状态变更时两处一起更新。

## 目录与使用方式

```text
ai-docs/
├── architecture/                 # README.md 为总览与主题导航
├── modules/                      # 重要模块设计与任务导航
├── standards/                    # 技术规范与官方依据
├── task-index.md                 # 全部任务索引
└── task/
    ├── _template.md              # 创建任务时复制
    └── 001-cargo-config.md       # 一个任务一个 Markdown 文件
```

新增任务使用未占用的三位递增编号和 kebab-case 名称；编号稳定，不因执行顺序调整而重排。每个任务直接保存为 `task/NNN-name.md`，无需单独创建目录。设计、决策和验证记录写在任务文件内；必要时链接外部证据，不要提交临时大型日志或构建产物。

1. 从 [模板](task/_template.md) 建立任务，填写目标、边界、依赖、必读规范和可判断的验收条件。
2. 在本索引登记，核对依赖无环；信息充分且依赖完成后标 ready。日常“开始任务 001”即可指向对应任务，不需要另建应用里的 task。
3. 开始实现时标 in-progress；只按该任务范围工作。范围明显变化先更新任务，额外功能先创建新任务。
4. 每次 commit 更新 task 的实际进展、验证与清理/兼容记录；状态、依赖或标题变化时同步索引。全部验收完成才标 done，文档已写好不等于实现完成。

提交的最小完整性及消息格式见 [提交规范](standards/commits.md)，代码删除和兼容例外见 [代码生命周期](standards/code-lifecycle.md)。

## 状态约定

| 状态 | 含义 |
|---|---|
| draft | 缺少范围/验收，不能开始 |
| planned | 内容已编排，依赖尚未满足 |
| ready | 依赖完成且可开始，尚无实现 |
| in-progress | 正在实施 |
| blocked | 有具体阻塞，记录解除条件 |
| done | 验收完成且有证据 |
| deferred | 暂不排入当前交付 |
| cancelled | 已取消，保留编号和原因 |

## 验证节奏

每个基础设施任务随实现完成自己的验收与失败路径检查，不等待 011。018 已建立三平台 CI，push/PR 时自动执行当前可用检查；011 已把已有检查聚合到统一入口（043 收敛根质量入口），012 已交付依赖缓存与可复现 CI 基础。任务记录是验证证据的来源，索引状态只是摘要。

## 基础设施队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 001 | [Cargo workspace 与 Rust 工具链](task/001-cargo-config.md) | M0 | — | done |
| 002 | [平台、工具链与 native 依赖基线](task/002-dependency-baseline.md) | M0 | 001 | done |
| 003 | [CMake/Ninja 原生构建骨架](task/003-cmake-native-skeleton.md) | M0 | 002 | done |
| 004 | [Cargo 调度 CMake 与运行入口](task/004-cargo-native-orchestration.md) | M0 | 001, 003 | done |
| 005 | [Qt/QML 主窗口与 C++ ViewModel](task/005-qt-qml-shell.md) | M0 | 004 | done |
| 006 | [Rust/C++ FFI 最小契约](task/006-rust-cpp-boundary.md) | 基础平台 | 004 | done |
| 007 | [VTK WebGPU 硬件窗口原生视口](task/007-vtk-quick-viewport.md) | M0 | 005, 031, 038 | in-progress |
| 008 | [后台任务、错误与日志基础](task/008-tasks-errors-logging.md) | 基础平台 | 005, 006 | done |
| 009 | [OCCT 依赖与 STEP 适配冒烟](task/009-occt-adapter-smoke.md) | CAE 接入基础 | 003, 008 | done |
| 010 | [Netgen 接入与最小 Mesh IR](task/010-netgen-adapter-smoke.md) | CAE 接入基础 | 009 | done |
| 079 | [清理 Netgen mesh 销毁 workaround](task/079-remove-netgen-mesh-cleanup-workaround.md) | CAE 接入基础维护 | 010, 038 | planned |
| 011 | [统一测试与质量入口](task/011-test-quality-entrypoints.md) | 验证基础 | 007, 008, 010 | done |
| 012 | [CI 与依赖缓存](task/012-ci-reproducibility.md) | 验证基础 | 011 | done |
| 013 | [桌面安装布局与部署冒烟](task/013-desktop-deployment-smoke.md) | 交付基础 | 011 | ready |
| 014 | [Python/uv 质量工具环境](task/014-python-tooling-foundation.md) | 验证基础 | 001 | done |
| 018 | [三平台 CI 基础](task/018-cross-platform-ci.md) | 验证基础 | 001, 002 | done |
| 019 | [GTest 测试配置与规则](task/019-gtest-native-testing.md) | 验证基础 | 003 | done |
| 020 | [托管引导：CMake/Ninja 二进制供给](task/020-toolchain-provisioning.md) | M0 | 004 | done |
| 031 | [预编译 native 依赖供给与 CMake package](task/031-prebuilt-native-dependencies.md) | 交付基础 | 002, 004 | in-progress |
| 036 | [三平台 CI native 构建修复](task/036-ci-native-build-fix.md) | 验证基础 | 004, 005, 018 | done |
| 038 | [Native SDK 制品生产与发布](task/038-native-sdk-artifact-production.md) | 交付基础 | 031, 020 | in-progress |
| 039 | [CI FFI 构建链修复](task/039-ci-ffi-build-fix.md) | 验证基础 | 006, 018, 036 | done |
| 040 | [统一 Cargo 构建编排入口](task/040-cargo-build-orchestration.md) | 验证基础 | 004, 039 | done |
| 041 | [构建产物归一与第三方缓存共享](task/041-build-artifact-consolidation.md) | 验证基础 | 004, 020 | done |

## 应用平台扩展队列

设计入口：[重要模块说明](modules/README.md)。下列任务部分已实施或完成，状态以表内为准。

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 022 | [UI 英文源文案与语言字典](task/022-ui-internationalization.md) | 应用平台扩展 | 005, 034 | planned |
| 023 | [跨平台路径与资源引用服务](task/023-cross-platform-paths.md) | 应用平台扩展 | 005, 006 | in-progress |
| 024 | [工程运行时上下文与变量快照](task/024-runtime-context.md) | 应用平台扩展 | 008, 023 | planned |
| 025 | [变量 DSL 解析、求值与存储](task/025-variable-dsl.md) | 应用平台扩展 | 024, 034 | planned |
| 026 | [C++ 静态库边界与 QML 自动注册](task/026-static-qml-modules.md) | 应用平台扩展 | 005 | done |
| 027 | [开发模式 QML 重载与状态恢复](task/027-qml-state-reload.md) | 应用平台扩展 | 007, 024, 026 | planned |
| 029 | [QML 原子组件库与 Theme 尺寸参数化](task/029-qml-component-library.md) | 应用平台扩展 | 005 | done |
| 030 | [DSL 主题配置与运行期主题切换](task/030-theme-dsl.md) | 应用平台扩展 | 025, 029 | planned |
| 048 | [设置服务与 Qt 持久化适配](task/048-settings-service-and-qt-adapter.md) | 应用平台扩展 | 006, 023, 030 | planned |
| 050 | [QML 页面设计 HTML 先行复刻](task/050-qml-html-page-replica.md) | 应用平台扩展 | 028, 029 | done |
| 051 | [无边框外观与一体化窗口标题栏](task/051-integrated-window-titlebar.md) | 应用平台扩展 | 005, 029 | ready |
| 052 | [QML 图标规范与首页布局优化](task/052-qml-icon-and-layout-polish.md) | 应用平台扩展 | 029, 050 | done |
| 053 | [默认视口立体 panta 字样](task/053-default-panta-wordmark.md) | 应用平台扩展 | 007, 081 | in-progress |
| 054 | [顶部折叠图标与搜索框引导](task/054-titlebar-search-details.md) | 应用平台扩展 | 052 | done |
| 055 | [QML 组件评审与整理](task/055-qml-review-and-cleanup.md) | 应用平台扩展 | 052, 054 | done |
| 056 | [QML 周边 C++ 简化与性能评审](task/056-qml-native-review.md) | 应用平台扩展 | 055, 007 | done |
| 057 | [新建项目对话框与工程命令边界](task/057-new-project-dialog.md) | 应用平台扩展 | 023, 029, 055 | done |
| 059 | [打开工程后的 QML HTML 参考同步](task/059-open-project-html-reference.md) | 应用平台扩展 | 050, 052, 055, 057 | done |
| 060 | [已验收 HTML 的 QML 工程工作区同步](task/060-qml-project-workspace-reference.md) | 应用平台扩展 | 059, 057, 055 | done |
| 061 | [Home 与 Start & Learn 工具栏切换](task/061-ribbon-tab-navigation.md) | 应用平台扩展 | 060, 057 | done |
| 062 | [Ribbon 页签内容与公共渲染拆分](task/062-ribbon-tab-components.md) | 应用平台扩展 | 061, 060 | done |
| 063 | [STL 导入、导入选项持久化与工程工作区](task/063-stl-import-and-mesh-workspace.md) | 应用平台扩展 | 057, 060, 062 | in-progress |
| 064 | [VTK 视口导航、笛卡尔坐标系与六面体定位](task/064-vtk-navigation-and-orientation.md) | 应用平台扩展 | 007, 063 | in-progress |
| 065 | [VTK 视口缩放、右键旋转与方向过渡](task/065-vtk-zoom-and-cube-transition.md) | 应用平台扩展 | 007, 064 | done |
| 067 | [Rust 统一 STL 解析与 Mesh IR 领域校验](task/067-rust-mesh-domain-migration.md) | CAE 领域模块迁移 | 010, 063, 066 | done |
| 068 | [QML 工程 / 任务 Dock 与 Layers Dock](task/068-qml-project-and-layers-docks.md) | 应用平台扩展 | 063, 060, 062 | done |
| 069 | [工程 / Tasks / Layers 面板整体 Review 与性能消融](task/069-project-docks-review-and-ablation.md) | 应用平台扩展 | 068 | done |
| 073 | [Flow DSL 与首个异步 STL 视口资源激活](task/073-flow-dsl-and-import-state-machine.md) | CAE 业务编排 | 008, 034, 035, 067, 072；首个消费者 080 | planned |
| 074 | [Qt StateMachine 与导入窗口交互编排](task/074-qt-interaction-state-machine.md) | 应用平台扩展 | 026, 063, 072 | planned |
| 080 | [视口文档页签与 STL 按需激活](task/080-qml-viewport-document-tabs.md) | 应用平台扩展 | 007, 063, 068, 073 | in-progress |
| 081 | [QML 视觉语言与图标体系统一](task/081-qml-visual-language-and-iconography.md) | 应用平台扩展 | 029, 050, 069, 078, 080 | planned |

## 验证与质量扩展队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 032 | [跨语言质量工具链与 100% 覆盖率门禁](task/032-cross-language-quality-gates.md) | 验证基础 | 011, 018, 019 | in-progress |
| 033 | [高 DPI 缩放与多显示屏基础](task/033-display-scaling-and-multi-monitor.md) | 应用平台扩展 | 005, 007 | planned |
| 034 | [Rust Panta Artifact 解析与 TS/QM 编译入口](task/034-rust-panta-artifact-parser.md) | 应用平台扩展 | 001 | in-progress |
| 035 | [`.pa` 格式化器与格式校验器选型](task/035-pa-formatter-and-validator.md) | 应用平台扩展 | 034 | in-progress |
| 037 | [软件内增量更新基础](task/037-incremental-update-foundation.md) | 交付基础 | 005, 008, 013, 023 | planned |
| 042 | [三平台自有 C++ 统一 LLVM/Clang 工具链](task/042-unified-llvm-toolchain.md) | 验证基础 | 018, 032, 038 | in-progress |
| 043 | [根目录质量入口与测试聚合](task/043-root-quality-runner.md) | 验证基础 | 011, 032 | done |
| 044 | [Windows CI 停滞诊断与修复](task/044-windows-ci.md) | 验证基础 | 018, 042 | done |
| 045 | [Windows CI 分支代码审查与收敛](task/045-branch-code-review.md) | 验证基础 | 044 | done |
| 046 | [CI 触发拆分与缓存预算](task/046-ci-trigger-split-cache-budget.md) | 验证基础 | 018, 012 | done |
| 047 | [崩溃信号处理与日志落地](task/047-crash-signal-logging.md) | 验证基础 | 008 | in-progress |
| 048 | [性能基线与性能测试体系](task/048-performance-testing.md) | 验证基础 | 011, 032 | in-progress |
| 049 | [CI 修复：Netgen Linux 制品 ISA 基线与 Windows ASan 链接](task/049-ci-mesh-sigill-windows-asan.md) | 验证基础 | 038, 042, 010 | done |
| 058 | [项目包提交后的 CI 回归修复](task/058-ci-regression-after-project-package.md) | 验证基础 | 057, 032, 043 | done |
| 070 | [GoogleTest 三平台 SDK 制品 CI](task/070-googletest-sdk-ci.md) | 验证基础 | 019, 031, 038 | done |
| 071 | [GoogleTest SDK 消费接入](task/071-googletest-sdk-consumption.md) | 验证基础 | 070, 031 | done |
| 075 | [CI 修复：Windows GoogleTest ABI 与 Qt benchmark lint](task/075-ci-windows-gtest-and-qt-lint.md) | 验证基础 | 070, 071, 046 | done |
| 076 | [Cargo 测试与质量 runner 维护](task/076-panta-tests-runner-maintenance.md) | 验证基础 | 011, 043 | done |
| 082 | [CI 修复：VTK benchmark moc 前置与标题条焦点滚动](task/082-ci-vtk-benchmark-moc-and-focus-scroll.md) | 验证基础 | 048, 076, 029 | in-progress |

## 仓库与文档维护

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 015 | [文档入口与一致性整理](task/015-documentation-structure.md) | 文档维护 | — | done |
| 016 | [忽略规则与通用开发规范](task/016-repository-conventions.md) | 仓库维护 | — | done |
| 017 | [代码生命周期与 commit 一致性规范](task/017-code-lifecycle-and-commits.md) | 仓库维护 | — | done |
| 021 | [重要模块说明与后续任务规划](task/021-important-module-planning.md) | 文档维护 | — | done |
| 028 | [QML 原子组件与主题 DSL 规划](task/028-qml-theme-planning.md) | 文档维护 | — | done |
| 066 | [重库适配边界与 Rust 领域模块规划](task/066-native-domain-boundaries.md) | 架构与规范 | 065 | done |
| 072 | [Flow DSL 与 Rust 状态机方案评估](task/072-flow-state-machine-planning.md) | 架构与文档准备 | 008, 066, 067 | done |
| 077 | [进行中任务状态盘点](task/077-active-task-status-audit.md) | 文档维护 | — | done |
| 078 | [QML 性能基准登记规范](task/078-qml-performance-benchmark-policy.md) | 文档维护 | 069, 048 | done |

## 执行顺序与交付边界

主线：001 → 002 → 003 → 004 → 005 → 031 → 038 → 007，完成 Cargo 启动 Qt/QML + 预编译 VTK WebGPU 硬件窗口 SDK 的 M0 集成；若 031 找到可直接消费的全平台官方 SDK，038 可只完成资产登记与自检决策。

平台分支：004 → 006；005 + 006 → 008 → 009 → 010，先建立 FFI、任务生命周期，再验证 OCCT 和 Netgen。两个分支的集成冒烟接入已交付的统一聚合（011、012 已完成），013 继续部署检查。这里是依赖图，编号相邻不意味着必须等待不相关任务；是否并行执行由实际工作安排决定。

主题分支：005 → 029；025 + 029 → 030。先迁移组件及尺寸参数，再接主题 DSL；主题切换不依赖工程打开或引擎重载。

新增分支：001 → 034；005 + 034 → 022；005 → 026；005 + 006 → 023；008 + 023 → 024，024 + 034 → 025；007 + 024 + 026 → 027。022（语言切换）和 025（变量提交）不依赖热重载；这些扩展不阻塞原有 M0 主线。031 为 007、009、010 提供预编译 native SDK；当上游没有完整资产时由 038 生产可缓存制品；032 在 011 统一入口上补齐跨语言质量工具与 100% 覆盖率门禁。

014 已完成最小 Python/uv 质量工具环境：仅锁定 cmakelang 并供 `cargo format` 调用，不接入 Python 运行时/API，也不阻塞 OpenCASCADE、Netgen、自研 CFD 与 VTK 主链路。009/010 仅是适配器与小样例验证，完整 STEP UI、工程存储、网格编辑、Study、求解器客户端仍要另写业务 task；不包含外部 MoldSolver 或 Mold Protocol 的实现。

Flow 分支由 [072 设计评估](task/072-flow-state-machine-planning.md) 与 [073 实施规划](task/073-flow-dsl-and-import-state-machine.md) 跟踪：复用 034/035 的 DSL 内核，首个消费者是 [080](task/080-qml-viewport-document-tabs.md) 的只读 STL 视口资源激活，不改写工程或修订。后续 STEP 等写入型导入事务另行登记消费者并冻结提交边界；不追加 067 同步 STL 改造，也不阻塞当前 M0 主线。

Qt 交互分支由 [074](task/074-qt-interaction-state-machine.md) 跟踪，先接入 Qt StateMachine 模块与现有导入窗口交互；它与 073 的 Rust 核心实现没有互相完成依赖。073 提供实际异步能力后再联调，Qt 只协调意图与展示，提交 / 取消决定权保持在 Rust。

后续新任务使用当前最大编号加一，不复用已有编号。主线仍有 [007 VTK WebGPU 硬件窗口原生视口](task/007-vtk-quick-viewport.md) 的真实窗口验收，以及 [031 预编译 native 依赖供给](task/031-prebuilt-native-dependencies.md) / [038 SDK 制品生产](task/038-native-sdk-artifact-production.md) 的运行时分发、Linux 基线与 SBOM/provenance 收尾。任务 010 已完成，Netgen SDK 特例的移除由任务 079 跟踪。全部 in-progress 项已在 [077 状态盘点](task/077-active-task-status-audit.md) 对照验收与 CI 证据逐项检查。技术规则见 [规范索引](standards/README.md)，产品目标见 [架构里程碑](architecture/milestones-and-validation.md)。
