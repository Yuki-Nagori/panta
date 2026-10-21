//! Panta 应用核心（见 architecture/application-and-storage.md）：Rust 拥有
//! 工程、命令与作业的领域模型。任务 023 提供跨平台路径/逻辑资源引用；
//! 任务 057 增加了 `.panta` 主文件，
//! 当前工程清单契约为 schema 4，撤销 / 重做仍按后续任务接入。

mod execution;
mod fsm;
pub mod language;
pub mod path;
pub mod project;
pub mod runtime;
pub mod theme;

/// 宿主事件循环退出、工程服务销毁后关闭写入准入并等待已接受的清单提交。
/// 不在 UI 交互或写入 worker 内调用。
pub fn finish_background_writes() {
    execution::writes().close_and_wait();
}
