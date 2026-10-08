# 112 — FFI 私有单元测试归属修正

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[107](107-rust-service-boundaries-and-revisions.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

## 目标与范围

修复 [issue #7](https://github.com/Yuki-Nagori/panta/issues/7)：将经 path 属性挂载到根 tests/rust 的 FFI 私有单元测试归还对应源模块的 cfg(test) 内联模块，保留断言、私有访问及 Cargo 发现方式。删除旧文件和挂载路径，修正 107 记录与 111 规划，不修改测试目录规范或生产接口。

## 必读

- [测试规范](../standards/testing.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] 私有测试按职责归属源码文件，旧 tests/rust/ffi.rs 与挂载引用清除，断言保持。
- [x] Cargo 工作区聚合、构建、格式及 lint 通过，任务与文档一致。

## 验证计划与结果

macOS / Rust 1.98.1、仓库根目录：`cargo test --locked -p panta-ffi` 的 26 个单元回归通过；比对原文件的 20 个测试名全部保留，其余原模块测试仍在。`cargo test --locked --workspace` 通过 Rust 工作区、qmllint 和 72/72 个 native / QML CTest；`cargo build --locked`、`cargo format --check` 与 `cargo lint --check` 通过。未改生产行为，未运行真实窗口或性能基准；本批跨平台 CI 尚未执行。

## 工作记录

- 2026-10-08：核对 issue #7，采用源模块内联单测，沿用现行规范。

## 完成摘要

私有单元测试按 support / path / task / language / project / mesh 归属源码内联模块，删除根 tests/rust/ffi.rs 及 path 挂载；保留测试行为与 CXX 入口。107 记录与 111 规划同步修正，现行测试规范保持。
