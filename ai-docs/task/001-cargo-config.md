# 001 — Cargo workspace 与 Rust 工具链

- 状态：in-progress
- 阶段：M0
- 依赖：无
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-10-20

## 目标与范围

维护 Cargo workspace 与固定 Rust 工具链。初始骨架于 2026-09-16 验收；本批按维护者要求从 1.98.1 升级至 1.99.0，保留 edition 2024 和依赖锁文件；按维护者确认迁移原子更新 API，MSRV 提至 1.95，升级与 GPU 基准修复分开提交。

规则见 [Rust 规范](../standards/rust.md)、[版本基线](../standards/baseline.md)和[构建职责](../architecture/build-and-development.md)。当前成员及依赖以 Cargo manifest 为准，不在任务中维护第二份清单。

## 验收

- [x] 原始 workspace 成员、默认 launcher、锁文件和 lint 规则落地；骨架 build / test / fmt / Clippy 及干净检出重建通过。
- [x] Rust 1.99.0 与 rustfmt / Clippy 安装确认；CI 从根工具链文件取 channel，缓存键包含其摘要。
- [x] Rust 1.99 格式、Clippy、workspace 构建与聚合测试通过；Rust 1.95 core all-targets 编译检查通过。
- [ ] 本批三平台 PR CI 通过，状态与索引同步。

## 验证记录

| 日期 / 范围 | 结果与边界 |
|---|---|
| 2026-09-16，初始骨架 | macOS 26.3.1 arm64、Rust 1.98.1；metadata、locked build、2 项单测、fmt、Clippy、临时目录与干净检出重建通过。当时 launcher 无 GUI，无参返回 69、未知参数返回 64；这是历史契约，不代表当前产品行为。 |
| 2026-10-20，工具链安装 | rustc `1.99.0 (b940084d7 2026-09-28)`、cargo `1.99.0 (5f94df478 2026-08-27)`，rustfmt / Clippy 已安装。未修改全局默认工具链。 |
| 2026-10-20，Rust 1.99 | `cargo format --check`、`cargo lint clippy --check`、`cargo build --locked --workspace`、`cargo test-scope all` 均退出 0，Rust package 回归与 native/QML CTest 93/93 通过。初轮 Clippy 报 4 处弃用，迁移后通过；独立 `gpt-6.1-sol` medium 只读评审核对 API / 内存序 / 溢出语义及工具链配置，无可操作发现。 |
| 2026-10-20，MSRV | `cargo +1.95.0 check --locked -p panta-core --all-targets` 退出 0，仅涵盖 core 及其依赖 / 测试 target。首次因工具链安装不完整失败，rustup 修复后通过；未宣称全 workspace 的 MSRV 实测。 |

## 决策与清理

- 首次选版同时考虑 edition 与 CXX 的最低版本；本批 `try_update` 的 1.95.0 稳定属性已从本机标准库源码核对，4 处迁移保持原子内存序及溢出拒绝语义；`workspace.package.rust-version` 表示 MSRV，不等同于日常锁定编译器。
- 1.99.0 由维护者明确指定。固定值以 `rust-toolchain.toml` 为权威，当前规范只说明使用规则；历史版本和测量记录保留原值。
- 不新增依赖、兼容层或全局 rustup override；无需改动按工具链文件取版本的 CI action 与缓存逻辑。
- 本批合并涉及文档的重复构建、版本、状态与验收叙述，配置值链接到源文件；保留历史环境、命令、测量边界及独有约束。8 份 Markdown 的 200 个本地链接、14 个表格及 diff 空白检查通过；`gpt-6.1-sol` medium 最终只读评审无新增可操作发现。

## 完成摘要

原始骨架已验收。当前工具链升级本地验证通过，CI 通过后恢复 done；本任务不重复后续 native / GUI 功能任务的实施记录。
