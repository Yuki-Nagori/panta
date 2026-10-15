# Rust 应用命令层

[架构总览](README.md) · [分层规则](../standards/layering.md) · [任务 118](../task/118-layering-lint-and-command-boundaries.md) · [任务 121](../task/121-rust-gui-cli-command-layer.md)

## 当前状态

Rust 应用服务位于 `panta-core`，例如 `ProjectService` 编排工程状态和网格激活。GUI 通过 `panta-ffi`/CXX 将请求转到 Rust，再由 C++ ViewModel 提供 Qt 属性、信号及异步调度。

`panta-dslc` 是 `.pa` parser/formatter 的命令行工具，但它不是桌面应用的通用 CLI；`panta-launcher` 负责启动 native app 与打包，也不是应用命令入口。当前没有正式的 `panta-cli`，也没有已完成的 GUI/CLI 共用领域命令。构建脚本与 CLI 共用 `panta-dsl-core` 函数不满足此边界验收。

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
- 不预先搭建命令总线、插件系统或空的 CLI crate。先以一个可从当前 GUI 与最小 CLI 实际触发的操作验证边界，再根据真实消费者决定接口归属。

## 后续实施

任务 121 需先盘点 `ProjectService` 等现有操作，选择最窄且具有用户价值的共享命令；为 Rust 命令行为和 GUI/CLI 调用路径增加测试。未实施部分仍是设计目标，不代表当前已有 `panta-cli`。
