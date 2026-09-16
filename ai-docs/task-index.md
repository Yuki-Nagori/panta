# Task 索引

采用“先写 task，再做实现”的工作方式。001、002、018 已完成（Rust 骨架可构建、可测试；主平台与 Cargo 托管的依赖固定清单已记录；三平台 CI 已建立）；003 已具备开始条件，其余基础设施任务未实现。仓库与文档维护任务单独列出。任务详情是范围、依赖、验收与证据的主记录，索引提供队列总览，状态变更时两处一起更新。

## 目录与使用方式

```text
ai-docs/
├── architecture/                 # README.md 为总览与主题导航
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
| 003 | [CMake/Ninja 原生构建骨架](task/003-cmake-native-skeleton.md) | M0 | 002 | ready |
| 004 | [Cargo 调度 CMake 与运行入口](task/004-cargo-native-orchestration.md) | M0 | 001, 003 | planned |
| 005 | [Qt/QML 主窗口与 C++ ViewModel](task/005-qt-qml-shell.md) | M0 | 004 | planned |
| 006 | [Rust/C++ FFI 最小契约](task/006-rust-cpp-boundary.md) | 基础平台 | 004 | planned |
| 007 | [VTK 原生 Qt Quick 视口](task/007-vtk-quick-viewport.md) | M0 | 005 | planned |
| 008 | [后台任务、错误与日志基础](task/008-tasks-errors-logging.md) | 基础平台 | 005, 006 | planned |
| 009 | [OCCT 依赖与 STEP 适配冒烟](task/009-occt-adapter-smoke.md) | CAE 接入基础 | 003, 008 | planned |
| 010 | [Netgen 接入与最小 Mesh IR](task/010-netgen-adapter-smoke.md) | CAE 接入基础 | 009 | planned |
| 011 | [统一测试与质量入口](task/011-test-quality-entrypoints.md) | 验证基础 | 007, 008, 010 | planned |
| 012 | [CI 与依赖缓存](task/012-ci-reproducibility.md) | 验证基础 | 011 | planned |
| 013 | [桌面安装布局与部署冒烟](task/013-desktop-deployment-smoke.md) | 交付基础 | 011 | planned |
| 014 | [后续 Python 工具环境](task/014-python-tooling-foundation.md) | 后续可选 | 001 | deferred |
| 018 | [三平台 CI 基础](task/018-cross-platform-ci.md) | 验证基础 | 001, 002 | done |

## 仓库与文档维护

| 编号 | 任务 | 阶段 | 依赖 | 状态 |
|---|---|---|---|---|
| 015 | [文档入口与一致性整理](task/015-documentation-structure.md) | 文档维护 | — | done |
| 016 | [忽略规则与通用开发规范](task/016-repository-conventions.md) | 仓库维护 | — | done |
| 017 | [代码生命周期与 commit 一致性规范](task/017-code-lifecycle-and-commits.md) | 仓库维护 | — | done |

## 执行顺序与交付边界

主线：001 → 002 → 003 → 004 → 005 → 007，完成 Cargo 启动 Qt/QML + VTK 的 M0 集成。

平台分支：004 → 006；005 + 006 → 008 → 009 → 010，先建立 FFI、任务生命周期，再验证 OCCT 和 Netgen。两个分支都准备好后，007 + 008 + 010 → 011 → 012 / 013，统一测试、CI 和部署检查。这里是依赖图，编号相邻不意味着必须等待不相关任务；是否并行执行由实际工作安排决定。

014 默认 deferred，只有开始 Python 工具工作时才推进，不阻塞桌面基础链。009/010 仅是适配器与小样例验证，完整 STEP UI、工程存储、网格编辑、Study、求解器客户端仍要另写业务 task；不包含外部 MoldSolver 或 Mold Protocol 的实现。

后续新任务使用当前最大编号加一，不复用已有编号。001、002 已完成；下一项建议执行 [003 CMake 原生构建骨架](task/003-cmake-native-skeleton.md)。技术规则见 [规范索引](standards/README.md)，产品目标见 [架构里程碑](architecture/milestones-and-validation.md)。
