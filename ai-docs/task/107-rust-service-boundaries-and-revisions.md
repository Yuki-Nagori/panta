# 107 — Rust 工程服务边界与修订耗尽修复

- 状态：done
- 阶段：应用平台扩展
- 依赖：[106](106-rust-architecture-review.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

## 目标与范围

实施 106 的第一批修复：在现有 crate 内拆分工程模型、错误、Mesh 缓存与服务门面，按职责拆分 FFI 转发实现，保留 CXX 声明入口及已有公共 API。统一工程修订递增策略，耗尽时在变更状态、写入资产或启动后台提交前拒绝操作，避免旧快照继续被接受。

UI、正常工程 schema 与导入业务行为保持现状；确认导入后台化继续归 063，文件级提交协调和执行器容量策略按 106 后续批次推进，本批不提前宣称这些建议已完成。

## 必读

- [分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] 工程模型 / 错误 / 缓存 / 服务模块职责明确，公共工程 API 与 FFI 生成入口保持一致。
- [x] FFI 转发按领域模块组织，旧根文件实现移除，无重复桥接或兼容入口。
- [x] 修订耗尽拒绝变更，内存、清单和资产不受影响；普通递增及无操作命令保持现有语义。
- [x] Rust / FFI 回归、Cargo 聚合、构建、格式和完整 lint 通过，记录真实证据。

## 验证计划与结果

补充修订上限、普通递增、方案配置、导入失败不污染资产及 CXX 诊断回归。定向检查定位问题，最终使用 `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check`。macOS、仓库根目录、锁定工具链：定向 `cargo test --locked -p panta-core -p panta-ffi` 通过；`cargo test --locked --workspace` 通过 Rust 工作区、qmllint 与 72/72 个 native / QML CTest；`cargo build --locked`、`cargo format --check`、`cargo lint --check` 均通过。修订耗尽回归覆盖重命名、导入、序列、材料、Fill / Gate Location 确认，验证状态 / 清单 / 资产保持不变，已确认值的无操作请求仍成功；FFI 验证稳定诊断与失败占位值。文档链接、空白及 diff 检查通过。本批未运行真实窗口、性能基准或新跨平台 CI，不据源码移动宣称性能提升。

## 工作记录

- 2026-10-08：完成工程与 FFI 模块拆分，工程子模块改为显式依赖，修订递增统一检查；保留 CXX 声明、opaque 类型的公共门面及已有生成入口，FFI Rust 回归曾移至 tests/rust；该私有单测归属问题由 112 修正，测试现保留在对应源码模块内。完成 Cargo 聚合及质量回归，状态 / 索引同步 done。

- 2026-10-08：先提交 105/106 文档（36bf598），登记本批实施边界与索引。修订耗尽策略由饱和成功变为显式拒绝，需同步既有饱和测试；不扩展工程文件格式或 UI。

## 完成摘要

第一批已完成：工程模型 / 错误 / 缓存 / 服务分别组织，FFI 转发与 DTO 映射按服务分模块，修订耗尽在写入前显式拒绝并保持旧状态。公共 API、UI 及工程 schema 保持原路径与格式；正常递增和无操作请求保持原语义。只有耗尽后的变更由饱和成功变为拒绝，既有测试已同步。

确认导入后台化、文件级提交协调、执行器容量 / 清理、DSL 和构建工具模块整理仍按 106 的后续顺序推进；107 done 不代表全部架构建议已经实施。
