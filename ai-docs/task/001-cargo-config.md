# 001 — Cargo workspace 与 Rust 工具链

- 状态：done
- 阶段：M0
- 依赖：无
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

建立可检查的最小 Rust workspace，为后续统一构建入口提供可靠起点。

## 必读

- [通用规范：repository-hygiene](../standards/repository-hygiene.md)

- [规范：baseline](../standards/baseline.md)
- [规范：rust](../standards/rust.md)
- [规范：cargo](../standards/cargo.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [架构：repository-layout](../architecture/repository-layout.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不接 CMake/Qt，不创建所有空业务 crate，不安装全套 native 依赖。

## 前置条件与待决策

本任务无前置任务。实施前已核实：主验证平台为 macOS 26.3.1（arm64），PATH 上另有 Homebrew rustc/cargo 1.98.1，与 rustup stable 同版本；项目以 rustup + `rust-toolchain.toml` 为工具链权威来源。版本依据见下方决策记录。

## 实施步骤

1. 确认主验证环境并选择支持 edition 2024 的固定 stable 工具链；记录 rustc/cargo 版本及 rust-version 策略，并核对首选 CXX release 的 MSRV，不能只按 edition 的最低版本选工具链。
2. 建立最少必要成员，选定唯一默认 launcher package；集中声明 edition、resolver 3、公共依赖和 lint 继承。
3. 提交应用 Cargo.lock、工具链配置与格式约定；检查忽略规则覆盖产物且不忽略锁文件。
4. launcher 暂只提供明确的未接入桌面诊断；更新 README，区分 Rust 骨架可用和桌面尚未可用。

## 预计改动

Cargo.toml、Cargo.lock、rust-toolchain.toml、最小 crates/、按需 .cargo/ 与 README.md。实际改动：新增根 `Cargo.toml`、`rust-toolchain.toml`、`crates/launcher/{Cargo.toml,src/main.rs}` 与 `Cargo.lock`；未创建 `.cargo/`（暂无需要集中配置的 target/linker 设置，留给任务 003/004）；更新 README 与架构/规范文档。

## 清理与兼容例外

无废弃项（此前无可运行实现），无兼容层。rust.md 中原"当前没有 Cargo 工程"等失效表述已同步更新。

## 验收标准

- [x] cargo metadata 能正确解析实际成员，根目录运行目标唯一。
- [x] cargo build --locked、cargo test --locked 和 cargo fmt --all -- --check 对现有 Rust 骨架成功。
- [x] 新 checkout 可按记录安装工具链并重建；执行 cargo run 不会伪称启动了 GUI。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | macOS 26.3.1 arm64；`rustup show active-toolchain`（仓库内） | `1.98.1-aarch64-apple-darwin (overridden by '.../rust-toolchain.toml')`，固定生效；clippy/rustfmt 组件已装 |
| 2026-09-16 | `cargo metadata --no-deps --format-version 1`（rustup run 1.98.1） | workspace_members 与 default_members 均仅含 `panta-launcher@0.1.0`，运行目标唯一 |
| 2026-09-16 | `cargo build --locked`（rustup run 1.98.1） | 成功（先 `cargo generate-lockfile` 生成锁文件，随后所有检查 `--locked`） |
| 2026-09-16 | `cargo test --locked`（rustup run 1.98.1） | 2 passed, 0 failed（launcher 两个退出码单元测试） |
| 2026-09-16 | `cargo fmt --all -- --check`；`cargo clippy --locked --all-targets`（rustup run 1.98.1） | 均通过，无警告 |
| 2026-09-16 | `cargo run --locked`；`cargo run --locked -- --version`（rustup run 1.98.1） | 无参：输出"桌面可执行文件尚未接入…未启动 GUI"，launcher 退出码 69；带参：输出暂不接受参数，退出码 64；cargo 自身按惯例报 101，子进程码已在输出可见 |
| 2026-09-16 | PATH 上 Homebrew cargo 1.98.1 交叉复跑 `cargo test --locked` | 同样通过（与固定工具链同版本，仅作旁证） |
| 2026-09-16 | 干净目录重建：rsync 工作树（除 .git/target）到 /tmp 后 `rustup toolchain install 1.98.1 --profile minimal --component rustfmt,clippy` + locked build/test/fmt/run | 安装命令幂等成功（1.98.1 已在本地），重建、测试、格式、运行诊断全部通过 |
| 2026-09-16 | 忽略规则正反样例：`git check-ignore -v --no-index` | `Cargo.lock` 未被忽略（将提交）；`target/debug/panta-launcher` 命中 `.gitignore:2 /target/`；`.cargo/config.toml` 未被忽略（将来可提交） |
| 2026-09-16 | 本任务完成提交后以 `git clone` 复验干净检出 | locked build/test（2 passed）/fmt/clippy 全部通过；cargo run 输出未接入诊断、退出码 69，与 rsync 模拟一致 |

未覆盖：Windows/Linux 重建、非 rustup 环境（如纯 Homebrew 工具链对 rust-toolchain.toml 不生效）的行为未验证；本机 PATH 的 Homebrew cargo 会忽略 rust-toolchain.toml，但当前版本一致，升级后可能出现漂移——长期以 rustup 为准，平台矩阵归任务 002。

## 风险与回退

工具链可能支持 edition 2024 却不满足选定 CXX 的 MSRV；固定版本前核对两者。骨架检查通过只证明 Rust 入口可用。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-16（实施）工具链选型：edition 2024 仅要求 ≥1.85；经 crates.io API 核实首选 CXX 最新 release 1.0.202（2026-09-12 发布，cxx-build 同版本）`rust_version = 1.88` > 1.85，故不以 edition 最低值选型。固定 stable 1.98.1（2026-09-01）于 `rust-toolchain.toml`（profile minimal + rustfmt + clippy）；workspace `rust-version = "1.88"` 作为承诺下限并启用 resolver 3 的 MSRV 感知解析，依赖要求更高时上调。
- 2026-09-16（实施）成员与命名：仅建 `crates/launcher`（不建无用途 crate）；package 名 `panta-launcher`，bin 名 `panta-launcher`；桌面可执行文件名留给任务 005。`[workspace.dependencies]` 暂空，当前无外部依赖；lint 基线 `unsafe_code=deny` + clippy `all/unwrap_used/expect_used=warn`，最终策略归 011。
- 2026-09-16（实施）launcher 契约：无参数时 stderr 输出未接入诊断并以 69（EX_UNAVAILABLE）退出；收到参数时输出用法诊断并以 64（EX_USAGE）退出；退出码为任务 004 重写前的过渡约定。
- 2026-09-16（实施）范围调整：无。`.cargo/` 未创建（无需配置），未顺手引入 CXX 依赖（接入决策归 006，本次仅核对其 MSRV）。
- 待记录：后续任务引用本任务退出码或工具链版本时的变更原因。

## 完成摘要

已交付最小 Rust workspace：edition 2024 + resolver 3 的虚拟 workspace，唯一成员 `crates/launcher`（无外部依赖，含 2 个单元测试），`rust-toolchain.toml` 固定 stable 1.98.1，Cargo.lock 已提交。`cargo build --locked`、`cargo test --locked`、`cargo fmt --all -- --check`、`cargo clippy` 通过；`cargo run` 明确报告桌面未接入（退出码 69），不伪称启动 GUI。README 与架构/规范文档已同步"骨架可用、桌面未实现"的边界。剩余限制：仅验证 macOS arm64；Clippy 完整策略（011）、native 调度（003/004）未实施。后续：002 平台与依赖基线已就绪。
