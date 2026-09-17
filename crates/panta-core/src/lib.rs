//! Panta 应用核心（见 architecture/application-and-storage.md）：Rust 拥有
//! 工程、命令、任务与作业的领域模型。已落地任务 008 的最小任务生命周期
//! 与任务 023 的跨平台路径/逻辑资源引用；工程模型、序列化与撤销/重做另行
//! 接入。

pub mod path;
pub mod task;
