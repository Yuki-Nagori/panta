//! Panta 应用核心（见 architecture/application-and-storage.md）：Rust 拥有
//! 工程、命令、任务与作业的领域模型。已落地任务 008 的最小任务生命周期
//! 与任务 023 的跨平台路径/逻辑资源引用；任务 057 增加了 `.panta` 主文件的
//! schema 1 清单契约，完整工程资产模型和撤销/重做仍按后续任务接入。

pub mod path;
pub mod project;
pub mod task;
