# 043 — 根目录质量入口与测试聚合

- 状态：done
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[032](032-cross-language-quality-gates.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-18 / 2026-09-18

## 目标与背景

原 native 聚合测试位于 `crates/launcher/tests`，依赖审计、未使用依赖检查、Rust 格式和 C++/QML 检查也曾分散在不同 CI step。本任务将根目录 `tests/` 建为唯一跨语言测试域：`src/` 放 Cargo 调度器，`integration/` 放跨语言聚合测试，`cpp/` 与 `qml/` 放测试源；通过 Cargo alias 统一执行。Rust 单元测试仍保留在实现文件，crate 黑盒测试保留在各 crate 的 `tests/`。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)
- [跨语言质量工具链](032-cross-language-quality-gates.md)
- [质量工具链模块](../modules/quality-tooling.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 范围与非目标

范围：根 `tests/` Cargo package、Cargo aliases、Rust/C++/QML/CMake 质量命令聚合、CI 入口、测试目录规范和使用说明；统一 `cargo quality`、`cargo lint`、`cargo test`、`cargo format`。lint 包含 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、IWYU 和 Cppcheck；format 另外包含由 uv 锁定的 cmake-format。

非目标：不把依赖私有实现的 crate 单元测试物理搬出所属 crate；不复制 CTest/GTest/QML 测试；不引入 Python 业务运行时；CMake 工具由 uv 锁定；coverage 仍由专用 CI job 调用固定工具。

## 前置条件与待决策

- `cargo deny` 与 `cargo machete` 版本已由 032 固定；CI 在质量入口前安装对应版本。
- native 构建仍由 launcher build.rs 复用生产构建图；根入口不得递归调用自身。
- 质量入口必须支持失败传播、空 CTest 套件失败、Release/Debug 和自定义 target-dir；格式工具缺失时明确失败。

## 实施步骤

1. 创建根 `tests/` package 与 build.rs，复用托管 CMake/clang-format 定位并注入当前 Cargo profile 的 native 构建树。
2. 将 launcher 的 native 聚合测试移入根 runner，删除旧入口；C++/QML 测试源统一迁移到 `tests/cpp/`、`tests/qml/`，保留 crate 私有 Rust tests 和 native CTest 注册。
3. 增加 Cargo aliases：`quality` 聚合格式、lint、依赖检查和测试；`lint` 聚合 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、IWYU 和 Cppcheck；根 `tests/integration/native.rs` 通过显式 manifest 注册接入标准 `cargo test`；`format` 聚合 Rust、C++/CXX、CMake 和 QML 格式检查，CMake 工具由 uv 锁定，QML 格式只准备 Qt 工具不构建 launcher。
4. CI 将每个 lint 作为独立可定位 step 调用 `cargo lint <tool>`，不以 `cargo quality` 代替；覆盖率仍使用独立 `cargo llvm-cov` job。
5. 更新 README、质量工具链模块、011/032 任务记录，记录真实命令和受控失败验证。

## 预计改动

`tests/Cargo.toml`、`tests/build.rs`、`tests/src/main.rs`、`tests/integration/native.rs`、`tests/cpp/`、`tests/qml/`、`Cargo.toml`、`.cargo/config.toml`、`.github/workflows/ci.yml`、README、质量模块、测试规范和相关 task。删除 `crates/launcher/tests/native_suite.rs` 旧聚合实现。

## 清理与兼容例外

删除 launcher 旧 native 聚合入口及失效引用；不保留双入口。crate 内部测试、native CTest 注册和 QML 行为测试不是废弃实现。无兼容例外。

## 验收标准

- [x] `cargo lint` 一条命令执行 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、IWYU 和 Cppcheck；工具缺失或任一检查失败返回非零。IWYU 与 Cppcheck 的职责不重复，Cppcheck 开启 unusedFunction。
- [x] `cargo quality` 一条命令执行格式、`cargo lint`、deny、Rust 测试、native CTest 和 QML 行为测试。
- [x] 标准 `cargo test` 执行 workspace Rust 测试和根 `tests/` 的完整 native/QML 聚合；失败、空套件、工具缺失返回非零；CI 使用 `cargo test --locked`。
- [x] `cargo format` 检查 Rust、C++、CMake、QML；官方 `cargo fmt` 保持 Rust-only 语义，Python 工具通过 uv 锁定。
- [x] CI 分别调用 `cargo lint clippy|machete|cmake|qmllint|clang-tidy|iwyu|cppcheck`，失败项可以单独定位；coverage job 的专用命令保持明确，不递归调用质量入口。
- [x] 根入口支持 Debug/Release、自定义 target-dir 和 Windows `ctest.exe`；不重复执行旧 launcher 聚合。
- [x] 文档、task、索引、Cargo aliases、测试目录规范和 CI 一致，旧入口引用清理完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-18 | 创建任务 | 已建立根质量入口任务 |
| 2026-09-18 | `cargo metadata --locked --no-deps --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked -p panta-tests --all-targets -- -D warnings`、`cargo test --locked -p panta-tests --no-run`、`actionlint .github/workflows/ci.yml`、`git diff --check` | 通过；根 `tests/` package、显式 `integration/native.rs`、CI 矩阵和 Rust 聚合器可解析/编译 |
| 2026-09-18 | `cargo check --locked --workspace --all-targets`、`cargo build --locked --workspace` | 通过；Cargo 驱动的 launcher 完成 CMake/Qt/GoogleTest 构建，并生成 native compile database |
| 2026-09-18 | `uv run --locked cmake-format --check ...`、`uv run --locked cmake-lint ...`（native/qml/tools 共 21 个 CMake 文件） | 通过；格式与 lint 均由根 Cargo runner 调用同一份 `pyproject.toml`/`uv.lock` 配置 |
| 2026-09-18 | 仓库根 `cargo test --locked` | 通过；Rust workspace、CTest/GTest、qmllint、QML 行为和 `Qml.FormatCheck` 均执行，CTest 28/28 通过 |
| 2026-09-18 | `cargo lint clippy`、`cargo lint cmake`、`cargo format`、`actionlint .github/workflows/ci.yml` | 通过；工具缺失和失败均由根入口传播，CI 每个 lint 工具独立成检查 |

## 风险与回退

Cargo runner 可能与 build.rs 使用不同 target-dir 或 profile；通过 Cargo metadata、编译期注入和受控失败验证路径。回退时删除根 runner 和 aliases，恢复 CI 直接命令；不删除各 crate/native 自有测试。

## 决策与工作记录

- 2026-09-18：按维护者要求将跨语言质量和测试聚合提升为根 `tests/` 入口；C++/QML 源统一按语言分类，Rust 私有单元测试继续留在实现文件。

## 完成摘要

根 `tests/` 已成为唯一跨语言测试与质量调度入口；native 测试源按 C++/QML 分类，Rust 私有单元测试仍遵循 Cargo 惯例保留在实现文件或 crate `tests/`。覆盖率报告继续由专用 CI job 独立执行。
