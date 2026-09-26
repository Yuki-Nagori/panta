//! 有限状态机（FSM）消费者（073）：`.pa` 状态机声明 + 领域内手写实现。
//!
//! `fsm/*.pa` 是状态结构的唯一声明：build.rs 在构建期调用公共 DSL 内核
//! 生成转移元数据到 `OUT_DIR`，源码树只保留输入与手写实现。本目录的手写
//! 代码是规则、数据与副作用的唯一实现；两者不一致由 open_saved_stl 的
//! 双向结构测试暴露。

// 生成元数据保持声明全集（转移表、状态/事件集合与名称由双向结构测试
// 消费）；非测试构建不引用全部条目属预期，不视为死代码。
#[allow(dead_code)]
mod generated {
    // 由 build.rs 从 fsm/open-saved-stl.pa 确定性生成；不提交、不手改。
    include!(concat!(env!("OUT_DIR"), "/fsm/open_saved_stl.rs"));
}

pub(crate) mod open_saved_stl;
