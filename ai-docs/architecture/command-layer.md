# Rust 应用命令层

[架构总览](README.md) · [分层规则](../standards/layering.md) · [任务 118](../task/118-layering-lint-and-command-boundaries.md) · [任务 121](../task/121-rust-gui-cli-command-layer.md) · [任务 122](../task/122-rust-cli-project-create.md)

## 当前状态

Rust 应用服务位于 `panta-core`，例如 `ProjectService` 编排工程状态和网格激活。GUI 通过 `panta-ffi`/CXX 将请求转到 Rust，再由 C++ ViewModel 提供 Qt 属性、信号及异步调度。工程打开由 Qt 无关的 `ProjectCommand::Open` 表达，GUI FFI 与 CLI 都经 `ProjectService::execute` 调用同一校验和读取路径。

`panta-dslc` 是 `.pa` parser/formatter 的命令行工具；`panta-launcher` 负责启动 native app 与打包。`panta-cli` 提供窄应用操作：`project inspect <path>` 读取工程摘要，`project create <parent> <name>` 创建工程；它不负责启动 GUI。inspect 与 GUI 打开操作共用 `ProjectCommand::Open`；create 与 GUI 共用 `ProjectService::create` 及名称、路径和覆盖保护规则。构建脚本与 CLI 共用 `panta-dsl-core` 函数不等同于应用命令复用。

## 目标职责

```text
GUI 参数 → Qt ViewModel / Bridge ─┐
                                 ├→ Rust 应用命令/查询 → 领域服务 → 结果 / 错误
CLI 参数 → Rust CLI adapter ─────┘
```

- 应用命令定义可执行的用户意图；查询定义只读结果。输入、结果与错误使用 Qt 无关的 Rust 类型。
- Rust 应用入口负责业务校验与编排，调用既有领域服务。领域不依赖 UI、命令行、Qt 或 CXX。
- FFI 只转换 DTO、错误与句柄并转发；Bridge/ViewModel 只做 Qt 生命周期、属性/信号和线程适配。
- CLI 解析参数、调用同一 Rust 入口并呈现结果；不得另行复制业务校验和流程。
- 不预先搭建命令总线或插件系统。优先让应用命令表达用户意图，由真实消费者决定后续接口扩展。

## 命令选择

首个共享命令是打开工程：GUI 用它加载活动工程状态；CLI 用隔离的 `ProjectService` 实例检查 `.panta` 文件并显示名称、路径和修订号。打开命令会复用文件存在性、扩展名、清单解析和 schema 校验，不复制业务规则。工程创建也复用 `ProjectService::create`，由 CLI 解析父目录和名称后调用，不复制目录创建、名称验证及目标冲突规则。参数/格式错误返回非零退出码，领域错误保留稳定的 `ProjectError` code 供诊断。

开发时可运行 `cargo run -p panta-cli -- project inspect <工程文件.panta>` 检查工程，或运行 `cargo run -p panta-cli -- project create <父目录> <工程名>` 创建工程。新增跨 GUI/CLI 的用户操作时，先定义 Qt 无关输入/输出和错误，再由 FFI 与 CLI 各自做薄适配。
