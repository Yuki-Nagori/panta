# 108 — 工程包写入协调与磁盘修订复核

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[107](107-rust-service-boundaries-and-revisions.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-08

## 目标与范围

集中工程包写入租约：同一 manifest 的跨实例 / 跨进程写入互斥，提交前核对已保存修订，独立临时文件 create_new 与 RAII 清理，文件内容同步后原子替换。导入的资产写入和回滚纳入同一租约；失败不发布内存候选。锁为协作式，不承诺拦截绕过锁的外部写入；提交后不以目录同步失败伪装未提交，本批只承诺文件内容同步，不承诺所有平台断电后目录项持久性。

本任务对应 [106 架构评审](106-rust-architecture-review.md) 的后续建议；确认导入后台化仍由 063 实施。保持 UI 与正常工程 schema，不将新基础接口描述为已经实现。

## 必读

- [分层规则](../standards/layering.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] 并行写入排他、过期服务拒绝、临时文件冲突 / 清理、写入失败保留旧资产及普通保存 / 重开通过。
- [x] 替换实现与失效引用已删除，无新增兼容分支。
- [x] Cargo 工作区聚合、构建、格式及完整 lint 通过；记录具体平台与未覆盖点。

## 验证计划与结果

macOS / Rust 1.98.1、仓库根目录：定向 `cargo test --locked -p panta-core -p panta-build` 通过；最终 `cargo test --locked --workspace` 通过 Rust 工作区、qmllint 与 72/72 个 native / QML CTest（core 单元 55、工程集成 27、build 单元 17、FFI 单元 26）。`cargo build --locked`、`cargo format --check`、`cargo lint --check` 均通过。

并行保存只允许一个候选提交；过期服务的保存 / 导入被拒绝，内存、磁盘及资产保持；独立子进程持锁时保存立即拒绝，子进程绕过 Rust 析构退出后可重新取得租约。临时文件碰撞可重试且不覆盖旧文件，重试上限与编号耗尽可恢复失败，失败清理只删除自有候选。原有清单失败 / 导入资产回滚测试继续覆盖提交失败。

定向 Miri（nightly-2026-09-15、关闭隔离、独立 target/miri）工程集成 25 通过、2 个 OS 子进程测试明确跳过；repository 单元 3 通过。两项子进程行为已由 stable 原生测试运行。Miri nightly 对 fetch_update 有弃用提示；该 API 在仓库 stable / MSRV 下仍适用，不为新 nightly 改动承诺下限。未运行真实窗口、性能基准或本批跨平台 CI；文件内容同步不等于所有平台断电后的目录项持久性。

## 工作记录

- 2026-10-08：c6cfb5e 的 [CI 37741374442](https://github.com/Yuki-Nagori/panta/actions/runs/37741374442) 全部通过；后续 CI 暴露租约释放风险，在 [109 的 CI 收尾](109-bounded-background-execution.md) 中复现共享句柄延长锁寿命的机制，销毁改为显式 unlock 后关闭句柄；原 CI 的具体触发条件及新提交的 Linux 效果尚待复验。

- 2026-10-08：按用户要求一次登记剩余 Rust 架构任务，先提交规划后实施。
- 2026-10-08：用户要求已有文件锁一起迁移到 fs4；核对 [fs4 1.1.0 文档](https://docs.rs/fs4/1.1.0/fs4/)，同步特性 MSRV 1.75 满足仓库 1.88。既有 panta-build 安装锁和新工程写入租约统一使用显式 trait 调用及 WouldBlock / Error 分类，清除直接 fs2 依赖和锁文件项。
- 2026-10-08：集中 repository 租约，分离内存与已保存修订；导入写资产及回滚共享租约。以文件内容 sync 后 rename 为提交点，独立 staging 候选使用 create_new / RAII。完成并行、子进程、过期修订、碰撞及失败恢复回归和 Cargo 聚合质量检查，状态 / 索引同步 done。#7 的 FFI 私有测试归属由 112 独立提交。

## 完成摘要

工程写入协调及 fs4 迁移已完成本地验收，schema / CXX / UI 保持现状。稳定锁文件与空 staging 目录保留；过期写入显式拒绝，临时候选不覆盖遗留文件。执行容量与生命周期继续由 109 跟踪，确认导入后台化仍由 063 跟踪。
