//! Panta 应用核心（见 architecture/application-and-storage.md）：Rust 拥有
//! 工程、命令、任务与作业的领域模型。当前只落地任务 008 的最小任务生命
//! 周期；工程模型、序列化与撤销/重做另行接入。

pub mod task;
