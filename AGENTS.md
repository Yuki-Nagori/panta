# 仓库工作路由

`panta` 是规划中的 CAE 桌面平台。当前能力见 [README](README.md)，实施进度见任务索引；规划路径和接口不代表已实现。

| 工作内容 | 入口 |
|---|---|
| 开始任务、查依赖与状态 | [任务索引](ai-docs/task-index.md) / [任务模板](ai-docs/task/_template.md) |
| 查 i18n、路径/运行时、DSL、模块与重载设计 | [重要模块说明](ai-docs/modules/README.md) |
| 查模块职责、数据流与里程碑 | [架构总览及主题导航](ai-docs/architecture/README.md) |
| 查版本基线、编码及依赖规则 | [技术规范索引](ai-docs/standards/README.md) |
| 查外部物理引擎边界 | [[External / 非本仓库] MoldSolver](ai-docs/architecture/external-moldsolver.md) |
| 查共享契约边界 | [[External / 非本仓库] Mold Protocol](ai-docs/architecture/external-mold-protocol.md) |

工作约束：

- 先写或更新 `ai-docs/task/NNN-name.md` 并登记索引，再实施；元数据使用无序列表，任务保持单文件。完成后记录真实验证并同步状态。
- 修改代码遵循 [注释规范](ai-docs/standards/comments.md)，新增文件遵循 [仓库文件规范](ai-docs/standards/repository-hygiene.md)；验证要求见 [验证与评审](ai-docs/standards/validation-and-review.md)。
- 不保留死代码或废弃实现；替换时同步清理，默认不写前向/后向兼容代码。必要例外必须使用 `COMPAT(...)` 标记并登记清理任务，见 [代码生命周期](ai-docs/standards/code-lifecycle.md)。
- 每次 commit 保持功能、测试、配置、文档和 task 一致，遵循 [提交规范](ai-docs/standards/commits.md)；不把已知损坏状态留给下一提交修复。
- 只阅读任务相关的架构与规范；范围变化先更新任务，长期决策同步对应文档。
- QML 经 C++ ViewModel 调用服务，不直接操作 OCCT、Netgen、VTK；自有 C++ 使用 C++20。
- Rust 承担领域数据、业务规则与编排；CXX 仅声明映射和签名，C++ 适配层实际调用重库并封装类型、异常与所有权。VTK 的窗口、输入和逐帧显示留在 C++，具体边界与 crate 规划见 [分层规则](ai-docs/standards/layering.md) / [职责审计](ai-docs/architecture/native-domain-boundaries.md)。
- Cargo 提供统一入口，CMake 拥有 native 构建图；Rust/C++ 桥接优先验证 CXX。
- 常规构建、测试和质量验收直接使用仓库 Cargo 聚合入口：`cargo build`、`cargo test --workspace`、`cargo format --check`、`cargo lint`。定向 crate 测试、CTest 或单项 lint 用于定位问题，最终结果仍以适用的聚合入口为准；实际性能基准按对应 task 单独运行。
- `cargo format` 中的 Python 工具使用仓库锁定的 uv、托管 Python 和 `uv.lock`，不要切换到系统安装的 uv 或 Python。
  - 如果 macOS 沙箱里出现 `system-configuration-0.6.1/... Attempted to create a NULL object`，并伴随 `Tokio executor failed`，说明 uv 在访问系统配置服务时被沙箱拦截。旧版 `system-configuration` 没处理动态存储 API 返回的 NULL，导致 uv 在启动格式工具前 panic；这不是格式错误。
  - **在 Codex 中的处理方式**：保持仓库固定 uv 不变，对完整命令 `cargo format --check` 使用沙箱外执行权限（`require_escalated`）重跑。这样 uv 仍使用仓库托管的 Python 环境，同时可以访问 `com.apple.SystemConfiguration.configd`。2026-09-24 已按此方式验证通过。
  - 如果无法在沙箱外运行，可用 `cargo fmt --all -- --check` 单独检查 Rust 格式；这不能代替聚合格式检查。Ubuntu CI 也可验证完整入口。
  - 长期方案是升级固定 uv，并确认其依赖树使用 `system-configuration` 0.7.0 或更新版本（已加入 NULL 检查），然后在沙箱内重新验证 `cargo format --check`；不要只根据 uv 版本号判断是否修复。
  - 背景：[uv #16664](https://github.com/astral-sh/uv/issues/16664)、[uv #17484](https://github.com/astral-sh/uv/issues/17484)、[上游 NULL 检查修复](https://github.com/mullvad/system-configuration-rs/pull/59)。
- 需要真实窗口验收时直接运行 Cargo 构建出的 `panta-native` 二进制；有 computer use 能力时用它观察并操作 Panta 窗口，记录看到的界面与交互结果。不要为验收先连接 VS Code，也不要把无头 QML/CTest 或仅启动进程当作真实 GPU 窗口通过；图形会话不可用时记录具体限制，已有用户人工验收可单独记为用户证据。
- 外部求解器通过进程与版本化协议接入，不链接其 C++ ABI；大型数据走数据面。
- 外部文档只说明协作边界。未落地的接口、目录和命令标为规划，只运行实际存在的检查。
