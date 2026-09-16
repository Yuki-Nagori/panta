# 仓库工作路由

`chronos` 是规划中的 CAE 桌面平台。当前能力见 [README](README.md)，实施进度见任务索引；规划路径和接口不代表已实现。

| 工作内容 | 入口 |
|---|---|
| 开始任务、查依赖与状态 | [任务索引](ai-docs/task-index.md) / [任务模板](ai-docs/task/_template.md) |
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
- Cargo 提供统一入口，CMake 拥有 native 构建图；Rust/C++ 桥接优先验证 CXX。
- 外部求解器通过进程与版本化协议接入，不链接其 C++ ABI；大型数据走数据面。
- 外部文档只说明协作边界。未落地的接口、目录和命令标为规划，只运行实际存在的检查。
