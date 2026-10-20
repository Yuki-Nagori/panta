# 仓库协作指南

`panta` 是规划中的 CAE 桌面平台。当前能力见 [README](README.md)，实施进度见[任务索引](ai-docs/task-index.md)；规划路径和接口不代表已经实现。

## 工作入口

| 工作内容 | 入口 |
|---|---|
| 开始任务、查依赖与状态 | [任务索引](ai-docs/task-index.md) / [任务模板](ai-docs/task/_template.md) |
| 查 i18n、路径与运行时、DSL、模块与重载设计 | [重要模块说明](ai-docs/modules/README.md) |
| 查模块职责、数据流与里程碑 | [架构总览及主题导航](ai-docs/architecture/README.md) |
| 查版本基线、编码及依赖规则 | [技术规范索引](ai-docs/standards/README.md) |
| 查外部物理引擎边界 | [MoldSolver](ai-docs/architecture/external-moldsolver.md) |
| 查共享协议边界 | [Mold Protocol](ai-docs/architecture/external-mold-protocol.md) |

只阅读当前任务相关的架构与规范。任务范围变化时先更新所属任务；长期设计决策同步到对应说明。

## 任务与代码约束

- 实施前创建或更新 `ai-docs/task/NNN-name.md` 并登记 `ai-docs/task-index.md`。任务元数据使用无序列表，每个任务保持一个文件。CI 修复等小范围工作更新所属任务的范围与验证记录，不另建任务。验收后记录实际验证结果并同步任务状态和索引。
- 代码注释遵循[注释规范](ai-docs/standards/comments.md)，新增文件遵循[仓库文件规范](ai-docs/standards/repository-hygiene.md)，验证要求见[验证与评审](ai-docs/standards/validation-and-review.md)。
- 不保留死代码或废弃实现；替换时同步清理，默认不写前向或后向兼容代码。确需例外时用 `COMPAT(...)` 标记并登记清理任务，见[代码生命周期](ai-docs/standards/code-lifecycle.md)。
- 每次提交需让功能、测试、配置、文档和任务记录保持一致，并遵循[提交规范](ai-docs/standards/commits.md)。不把已知损坏状态留给后续提交修复。

## 架构边界

- Rust 承担领域数据、业务规则与编排。CXX 只声明映射和签名；C++ 适配层调用重型库并封装类型、异常与所有权。
- VTK 窗口、输入和逐帧显示由 C++ 管理。QML 经 C++ ViewModel 调用服务，不直接操作 OCCT、Netgen 或 VTK。自有 C++ 使用 C++20。详细边界见[分层规则](ai-docs/standards/layering.md)和[职责审计](ai-docs/architecture/native-domain-boundaries.md)。
- 外部求解器通过进程和版本化协议接入，不链接其 C++ ABI；大型数据走数据面。
- Cargo 提供统一入口，CMake 拥有 native 构建图；Rust/C++ 桥接优先验证 CXX。
- 外部协议文档只定义本仓库的协作边界。未落地的接口、目录和命令要标为规划，只运行仓库中实际存在的检查。

## 构建、测试与窗口验收

- 本地日常开发先运行 `cargo test-plan` 查看影响范围，再用 `cargo test-scope l1` 验证直接受影响的 Rust package，或用 `cargo test-scope l2` 验证其消费者闭包及相关 native/QML 测试；选择器遇到空输入、未知路径或公共构建边界时会自动升级为完整范围。修改后和评审修复后优先重跑受影响范围，不重复运行 workspace 全量测试。
- `cargo test --locked --workspace` 是完整 Rust/native/QML 聚合测试入口，适用于明确要求全量验收、影响无法安全缩小的改动，以及 nightly CI；日常开发无需每次重复运行。常规构建和质量检查使用 `cargo build`、`cargo format --check`、`cargo lint`，并按改动范围执行相应检查。
- 本地日常验证使用 `cargo verify-changed`，聚合检查变更范围内的格式、适用 lint 和受影响测试。它复用 `cargo test-plan` 的变更路径与 L1/L2 包选择：Rust 格式检查直接变更包，Clippy 检查 L1/L2 受影响包，测试运行 L1 与 L2；C++、CMake、QML 格式检查变更文件；clang-tidy 对变更源文件选择编译单元，头文件（含删除）、CMake 配置、公共 Rust 构建或 CXX 生成输入变更扩大到全部 native 单元；Objective-C++ 同时纳入格式和 macOS LLVM 检查。分层图、workspace 依赖、跨翻译单元 cppcheck 和完整 qmllint 保留全量扫描。未知路径和公共构建边界会保守扩大质量检查；明确要求完整质量门禁时使用 `cargo quality` 或不带 `--changed` 的 lint/format 入口。
- 新增或迁移代码板块、Cargo package、CMake target、QML 模块或测试支持目录时，同一改动内更新 `tests/src/test_scope.rs` 的路径分类、L1/L2 依赖闭包和 native label 映射，以及对应 format/lint 的受影响范围；同步补选择器回归测试和必要的任务/架构文档。依赖关系以 `Cargo.toml`、配置后的 CMake target link 属性和仓库实际目录为准，不维护脱离实现的静态副本；路径或依赖无法识别时必须保守扩大范围，禁止让未覆盖板块静默跳过。
- CPU/GPU 性能基准由对应任务单独运行，不纳入常规单测或 CI 时间门禁。
- 需要真实窗口验收时先运行 `cargo build --locked`。macOS 绑定 `target/native/debug/app/panta.app`，或启动其 `Contents/MacOS/panta`；Windows/Linux 启动 `target/native/debug/app/panta.exe` 或 `target/native/debug/app/panta`。macOS 用 `cua.getApp` 绑定 `.app` 并调用 `getAXStateAndScreenshot()` 检查真实窗口。每次 UI 操作后刷新 `getAXState()`，按最新状态定位控件。
- 验收结束时关闭应用。不要为验收先连接 VS Code，也不要把无头 QML/CTest 或仅启动进程当作真实 GPU 窗口通过。Linux / Windows 使用窗口清单中的实际窗口 ID 绑定。图形会话不可用时记录具体限制；用户人工验收可作为独立证据。

## Subagent 使用规则

### 何时调用

- 每个非平凡任务在交付前都要安排一次独立评审。评审者根据需求、风险和完整 diff 检查，不复述实现者自检。简单的拼写、格式或单行修复不因此强制启动评审，用户明确要求时除外。
- 覆盖率缺口按[测试规范](ai-docs/standards/testing.md)拆分给 subagent 调查或修复；主代理负责整合并运行完整验证。不要为了达到数字而机械补断言或豁免行。
- 复杂问题能拆成相互独立的调查（例如跨平台 CI 根因、模块边界或文档一致性）时，可以并行委派。先确认每个子任务有独立产出，避免多个代理同时修改同一处。
- 常规小改动、主线工作可直接完成且没有独立审查要求时，不要为形式而启动 subagent。

### 默认模型

- 代码与架构的独立评审默认使用 `gpt-6.1-sol`，reasoning effort 为 `medium`。用户对某次工作明确指定其他模型或强度时，以该指示为准。
- 调查与实现子任务沿用当前代理可用的默认模型；只有用户或任务要求指定时才覆盖。工具支持时，委派前明确传入模型和 reasoning effort。

### 委派与评审边界

- 委派说明列明目标、范围、相关路径、验收条件、需要运行的检查，以及只读评审或获准修改。独立评审默认只读，不让评审者顺手重写被评代码。
- 评审报告应提供可操作的问题、文件与行号、触发条件和影响。没有发现问题时也要说明检查范围与限制。主代理判断发现是否成立，负责修复、验证和更新文档，并将评审结果记录在所属任务中；不把评审结论直接当作通过证明。
- 等待评审时，主代理继续处理不依赖评审结论的工作。修复评审发现后，重跑受影响测试及适用的交付门禁；只有影响无法安全缩小或明确要求完整验收时才运行全量测试，必要时请评审者复核。
- Subagent 不自行提交、推送、创建或合并 PR，也不代表用户对外沟通，除非用户明确授权。

## 分支、GitHub 与提交

- GitHub issue、PR、Actions 与仓库状态的查询或操作统一使用已认证的 `gh` CLI（如 `gh issue`、`gh pr`、`gh run`、必要时 `gh api`），不要用网页手工操作或自写 HTTP 请求。写操作前核对仓库、目标编号、当前分支和对话中的授权范围。
- 涉及代码、测试、依赖、构建或 CI 配置的工作，以及同时包含实现与配套文档的工作，都在专用工作分支完成。Task 实现与 issue 修复默认创建或继续对应分支，验证后通过 PR 合并到 `main`。
- 只有纯文档 issue 修复，且用户明确要求直接提交到 `main`（或本会话已有同等明确授权）时，才可在 `main` 上修改并提交。提交前确认工作区没有混入其他改动。纯文档例外不适用于实现任务中的文档同步。
- 不确定是否属于纯文档时按需要分支处理；不要为了省略 PR，把测试、脚本、配置或生成逻辑归为文档改动。
- 提交、推送、合并 PR 或改写共享分支历史，均须获得当前会话的明确授权；完成评审本身不构成授权。获准推送实现前，先完成独立评审。
- CI 修复等小改动复用所属任务。本地验证和评审后，按当前授权提交或推送，并等待对应 CI；CI 失败时继续定位、修复和复验，全部通过后结束。
