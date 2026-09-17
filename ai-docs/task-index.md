# Task 索引

采用“先写 task，再做实现”的工作方式。001–005、018、019、036 已完成（Rust 骨架、主平台与依赖固定清单、native 构建骨架、三平台 CI、GTest 规则、Cargo 调度 CMake、Qt Quick 主窗口和跨平台 native CI 修复均落地并验证）；006 最小 CXX 边界双向调用已本机验证、收尾验收进行中，031 已开始官方预编译资产盘点，039 正在修复 FFI 接入后的三平台干净构建，040 正在把 FFI staticlib 顺序收回统一 Cargo 编排入口，007 等待 031 提供匹配的预编译 VTK SDK，其余基础设施任务未实现。仓库与文档维护任务单独列出。任务详情是范围、验收与证据的主记录，索引提供队列总览，状态变更时两处一起更新。

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

每个基础设施任务随实现完成自己的验收与失败路径检查，不等待 011。018 已建立三平台 CI，push/PR 时自动执行当前可用检查；011 负责把已有检查聚合到统一入口，012 负责依赖缓存与 CI 扩展。任务记录是验证证据的来源，索引状态只是摘要。

## 基础设施队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 001 | [Cargo workspace 与 Rust 工具链](task/001-cargo-config.md) | M0 | — | done |
| 002 | [平台、工具链与 native 依赖基线](task/002-dependency-baseline.md) | M0 | 001 | done |
| 003 | [CMake/Ninja 原生构建骨架](task/003-cmake-native-skeleton.md) | M0 | 002 | done |
| 004 | [Cargo 调度 CMake 与运行入口](task/004-cargo-native-orchestration.md) | M0 | 001, 003 | done |
| 005 | [Qt/QML 主窗口与 C++ ViewModel](task/005-qt-qml-shell.md) | M0 | 004 | done |
| 006 | [Rust/C++ FFI 最小契约](task/006-rust-cpp-boundary.md) | 基础平台 | 004 | in-progress |
| 007 | [VTK 原生 Qt Quick 视口](task/007-vtk-quick-viewport.md) | M0 | 005, 031 | planned |
| 008 | [后台任务、错误与日志基础](task/008-tasks-errors-logging.md) | 基础平台 | 005, 006 | planned |
| 009 | [OCCT 依赖与 STEP 适配冒烟](task/009-occt-adapter-smoke.md) | CAE 接入基础 | 003, 008 | planned |
| 010 | [Netgen 接入与最小 Mesh IR](task/010-netgen-adapter-smoke.md) | CAE 接入基础 | 009 | planned |
| 011 | [统一测试与质量入口](task/011-test-quality-entrypoints.md) | 验证基础 | 007, 008, 010 | in-progress |
| 012 | [CI 与依赖缓存](task/012-ci-reproducibility.md) | 验证基础 | 011 | planned |
| 013 | [桌面安装布局与部署冒烟](task/013-desktop-deployment-smoke.md) | 交付基础 | 011 | planned |
| 014 | [后续 Python 工具环境](task/014-python-tooling-foundation.md) | MVP 后续能力 | 001 | deferred |
| 018 | [三平台 CI 基础](task/018-cross-platform-ci.md) | 验证基础 | 001, 002 | done |
| 019 | [GTest 测试配置与规则](task/019-gtest-native-testing.md) | 验证基础 | 003 | done |
| 020 | [托管引导：CMake/Ninja 二进制供给](task/020-toolchain-provisioning.md) | M0 | 004 | ready |
| 031 | [预编译 native 依赖供给与 CMake package](task/031-prebuilt-native-dependencies.md) | 交付基础 | 002, 004 | in-progress |
| 036 | [三平台 CI native 构建修复](task/036-ci-native-build-fix.md) | 验证基础 | 004, 005, 018 | done |
| 038 | [Native SDK 制品生产与发布](task/038-native-sdk-artifact-production.md) | 交付基础 | 031, 020 | planned |
| 039 | [CI FFI 构建链修复](task/039-ci-ffi-build-fix.md) | 验证基础 | 006, 018, 036 | in-progress |
| 040 | [统一 Cargo 构建编排入口](task/040-cargo-build-orchestration.md) | 验证基础 | 004, 039 | in-progress |

## 应用平台扩展队列

设计入口：[重要模块说明](modules/README.md)。下列任务均未实施。

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 022 | [UI 英文源文案与语言字典](task/022-ui-internationalization.md) | 应用平台扩展 | 005, 034 | planned |
| 023 | [跨平台路径与资源引用服务](task/023-cross-platform-paths.md) | 应用平台扩展 | 005, 006 | planned |
| 024 | [工程运行时上下文与变量快照](task/024-runtime-context.md) | 应用平台扩展 | 008, 023 | planned |
| 025 | [变量 DSL 解析、求值与存储](task/025-variable-dsl.md) | 应用平台扩展 | 024, 034 | planned |
| 026 | [C++ 静态库边界与 QML 自动注册](task/026-static-qml-modules.md) | 应用平台扩展 | 005 | in-progress |
| 027 | [开发模式 QML 重载与状态恢复](task/027-qml-state-reload.md) | 应用平台扩展 | 007, 024, 026 | planned |
| 029 | [QML 原子组件库与 Theme 尺寸参数化](task/029-qml-component-library.md) | 应用平台扩展 | 005 | in-progress |
| 030 | [DSL 主题配置与运行期主题切换](task/030-theme-dsl.md) | 应用平台扩展 | 025, 029 | planned |

## 验证与质量扩展队列

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 032 | [跨语言质量工具链与 100% 覆盖率门禁](task/032-cross-language-quality-gates.md) | 验证基础 | 011, 018, 019 | planned |
| 033 | [高 DPI 缩放与多显示屏基础](task/033-display-scaling-and-multi-monitor.md) | 应用平台扩展 | 005, 007 | planned |
| 034 | [Rust Panta Artifact 解析与 TS/QM 编译入口](task/034-rust-panta-artifact-parser.md) | 应用平台扩展 | 001 | in-progress |
| 035 | [`.pa` 格式化器与格式校验器选型](task/035-pa-formatter-and-validator.md) | 应用平台扩展 | 034 | in-progress |
| 037 | [软件内增量更新基础](task/037-incremental-update-foundation.md) | 交付基础 | 005, 008, 013, 023 | planned |

## 仓库与文档维护

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 015 | [文档入口与一致性整理](task/015-documentation-structure.md) | 文档维护 | — | done |
| 016 | [忽略规则与通用开发规范](task/016-repository-conventions.md) | 仓库维护 | — | done |
| 017 | [代码生命周期与 commit 一致性规范](task/017-code-lifecycle-and-commits.md) | 仓库维护 | — | done |
| 021 | [重要模块说明与后续任务规划](task/021-important-module-planning.md) | 文档维护 | — | done |
| 028 | [QML 原子组件与主题 DSL 规划](task/028-qml-theme-planning.md) | 文档维护 | — | done |

## 执行顺序与交付边界

主线：001 → 002 → 003 → 004 → 005 → 031 → 038 → 007，完成 Cargo 启动 Qt/QML + 预编译 VTK SDK 的 M0 集成；若 031 找到可直接消费的全平台官方 SDK，038 可只完成资产登记与自检决策。

平台分支：004 → 006；005 + 006 → 008 → 009 → 010，先建立 FFI、任务生命周期，再验证 OCCT 和 Netgen。两个分支都准备好后，007 + 008 + 010 → 011 → 012 / 013，统一测试、CI 和部署检查。这里是依赖图，编号相邻不意味着必须等待不相关任务；是否并行执行由实际工作安排决定。

主题分支：005 → 029；025 + 029 → 030。先迁移组件及尺寸参数，再接主题 DSL；主题切换不依赖工程打开或引擎重载。

新增分支：001 → 034；005 + 034 → 022；005 → 026；005 + 006 → 023；008 + 023 → 024，024 + 034 → 025；007 + 024 + 026 → 027。022（语言切换）和 025（变量提交）不依赖热重载；这些扩展不阻塞原有 M0 主线。031 为 007、009、010 提供预编译 native SDK；当上游没有完整资产时由 038 生产可缓存制品；032 在 011 统一入口上补齐跨语言质量工具与 100% 覆盖率门禁。

014 默认 deferred，当前 MVP 不接入 Python；只有开始第一个真实 Python 工具工作时才推进，不阻塞 OpenCASCADE、Netgen、自研 CFD 与 VTK 主链路。009/010 仅是适配器与小样例验证，完整 STEP UI、工程存储、网格编辑、Study、求解器客户端仍要另写业务 task；不包含外部 MoldSolver 或 Mold Protocol 的实现。

后续新任务使用当前最大编号加一，不复用已有编号。001–005 已完成；主线下一项先执行 [031 预编译 native 依赖供给](task/031-prebuilt-native-dependencies.md)，完成后再执行 [007 VTK 原生 Qt Quick 视口](task/007-vtk-quick-viewport.md)（006 可并行）。技术规则见 [规范索引](standards/README.md)，产品目标见 [架构里程碑](architecture/milestones-and-validation.md)。
