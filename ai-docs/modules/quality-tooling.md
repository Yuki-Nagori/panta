# 跨语言质量工具链

[模块导航](README.md) · [实施任务 032](../task/032-cross-language-quality-gates.md) · [统一测试入口 011](../task/011-test-quality-entrypoints.md)

## 目标与范围

质量工具链覆盖已经存在的 Rust、C++20、QML/Qt 和 CMake。Python/Node 源码真正进入仓库后才引入相应工具。以下区分当前门禁与规划目标，工具或文档就绪不代表任务验收完成。

## 固定工具版本（任务 032）

| 工具 | 固定版本 | 安装/来源 | 当前用途 |
|---|---|---|---|
| cargo-deny | 0.20.2 | `cargo audit`（根 Cargo alias 自动安装到 `target/panta-tools/cargo`） | RustSec、许可证、重复/通配依赖；配置 `deny.toml` |
| cargo-machete | 0.9.2 | `cargo lint machete`（根 runner 自动安装到 `target/panta-tools/cargo`） | 未使用 Rust 依赖；launcher/panta-ffi 的 build.rs `DEP_*` 消费在 metadata 登记豁免 |
| cargo-llvm-cov | 0.9.1 | `cargo coverage`（根 runner 自动安装到 `target/panta-tools/cargo`），rustup `llvm-tools-preview` | Rust 函数及行覆盖率阶段门禁 |
| qmlformat / qmllint | 6.11.2 | Qt 托管预编译供给 | CTest `Qml.FormatCheck`（stdout diff）与 `all_qmllint` |
| LLVM clang/clang++/clang-cl/clang-format/clang-tidy | 22.1.7 | LLVM 官方三平台固定资产；URL/SHA256 固定于 `crates/launcher/src/provision.rs`，由 Cargo 缓存到 `target/panta-tools/llvm` | CMake 与 Cargo CXX 使用同一 LLVM；macOS/Linux 选择 clang++，Windows 选择 clang-cl；clang-format、clang-tidy 和 C++ 质量入口复用同一版本 |
| include-what-you-use | CI/本机固定 LLVM 工具链 | `IWYU_TOOL` 环境变量或 PATH 中的 `iwyu_tool.py`；使用 native compile database | 未使用/缺失 `#include` 检查；根 `cargo lint` 执行，不自动改写源码 |
| Cppcheck | CI/本机固定版本 | `CPPCHECK` 环境变量或 PATH | C++ 未使用函数（`unusedFunction`）、错误路径和可疑构造；以 `--error-exitcode=1` 阻断 |
| cmake-format / cmake-lint | 0.6.13 | 根 `pyproject.toml` + `uv.lock`；通过 `UV_CACHE_DIR=target/panta-tools/uv/cache UV_PROJECT_ENVIRONMENT=target/panta-tools/uv/venv uv run --locked` 调用 | CMakeLists/`.cmake` 格式与静态规则门禁；分别纳入 `cargo format` 与 `cargo lint cmake` |

LLVM 工具使用官方 22.1.7 三平台发布资产，下载归档按 SHA256 校验并按版本/摘要隔离缓存。升级时同步更新 Cargo 供给、CMake 编译器检查、CI cache key、task 042 和实际版本验证；不混用系统 LLVM 工具。

## 当前执行入口

- 除 Rust crate 外的质量工具统一安装到 `target/panta-tools/`：Cargo 扩展工具由根 runner 按固定版本安装到 `target/panta-tools/cargo`，uv 使用 `target/panta-tools/uv/cache` 和 `target/panta-tools/uv/venv`。根 `tests/` package 提供 Cargo alias：`cargo format`（Rust、C++/CXX、CMake、QML 格式）、`cargo lint`（Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、IWYU、Cppcheck）、`cargo audit`（cargo-deny）、`cargo coverage`（cargo-llvm-cov）和 `cargo quality`（全部质量入口）。`cargo lint <tool>` 可只运行一个工具，方便 CI 和本地定位；`cmake` 这一项由 uv 按 `uv.lock` 自动准备 `cmake-lint`。`cargo format` 直接调用格式工具，QML 只做 Qt 工具供给和文件检查，不触发 launcher/native 完整构建；完整链接和行为验证由 `cargo build`/`cargo test` 负责，`cargo run --locked --package panta-tests -- toolchain` 负责确认 build 后的 Cargo 托管工具链与共享产物路径。`tests/src/` 只调度已有测试，`tests/integration/` 只负责跨语言聚合，不复制 crate 私有测试或 CTest 用例。
- `cargo test` 保留 Cargo 原生 workspace 语义，同时由根 `tests/` package 的显式集成测试聚合 qmllint、完整 CTest/GTest/QtTest 和 QML 行为测试。C++/QML 测试源分别归档在 `tests/cpp/`、`tests/qml/`；构建树、Debug/Release 配置和托管 CMake/LLVM 工具链由根 runner 的 build.rs 注入；CTest 使用 `-C` 与 `--no-tests=error`。
- CI 将 `cargo audit`、`cargo format`、`cargo test` 以及每个 `cargo lint <tool>` 分成独立检查；coverage 拆成 Rust 门禁和 native C++ 插桩报告，QML 场景随 native 测试执行。CI 命令显式使用 `--locked`，本地入口保持简洁。
- IWYU、Cppcheck 和 uv 的路径可由环境变量覆盖；Clang-Tidy 默认来自 Cargo 托管 LLVM，也可通过 CLANG_TIDY 显式覆盖；没有工具时根 lint/format 明确失败。CMake 格式和 lint 共用 `pyproject.toml` 与 `uv.lock`，由 Cargo runner 调用 `uv run --locked`，不能绕过锁文件或静默跳过。Cppclean 不纳入门禁，IWYU 负责 include 建议，Cppcheck 负责错误路径与未使用函数等实现级检查。版本/来源、编译数据库路径和排除规则必须与 task 043 同步。

真实窗口、DPR、多显示屏、GPU、线程及 ABI 检查单独留证。无头组件测试不能代替所有平台的真实图形生命周期验证。

## 覆盖率门禁规则（2026-09-18 评审修订）

Rust 门禁与 CI 一致的命令（仓库根目录）：

```sh
cargo coverage
```

**当前下限为全局函数 89%、行 92%，两者都阻断。** 这是补齐历史缺口期间的阶段门禁，不是 100% 完成证明，也不是与父提交逐项比较的防下降机制。例如函数覆盖从 89.69% 降到 89.10% 仍可能通过；一个模块的增长也可能抵消另一模块的退步。新增/修改逻辑的行为覆盖仍需评审，032 后续补按模块统计与基线比较，不得把全局通过当作模块无缺口。

Rust 函数覆盖 100% 是函数维度目标，函数进入一次不代表其内部路径经过验证；保留行门禁，不用函数 100% 宣称行/分支完整。C++ line/branch 100% 为待落地目标；QML 以可执行绑定和关键状态场景的行为断言为准。固定 stable 工具链当前未采集 Rust branch 数据，该项记录为未测，不计作通过。目标完成须同步 032 验收证据，不能仅上调一个阈值就标 done。

native 覆盖率由独立 CI job 负责：`PANTA_NATIVE_COVERAGE=1` 让 Cargo 驱动的 CMake 构建使用 Clang 的 `-fprofile-instr-generate -fcoverage-mapping`，测试完成后用同一 rustup 工具链的 `llvm-profdata`/`llvm-cov` 合并并报告 C++ 对象。QML 只计入被执行场景对应的 C++/Qt 代码，暂不声称 QML 源码级覆盖率；当前 job 先产出报告，不设置百分比门禁，待稳定基线后再固定阈值。

**统计口径与工具约束：**

- 使用 `rust-toolchain.toml` 锁定 rustc，配套安装同一工具链的 `llvm-tools-preview`；核对 PATH 与 `LLVM_COV`/`LLVM_PROFDATA`，不混用系统 LLVM 解析不同版本的插桩数据。
- 默认的 tests/examples/benches、生成构建树、依赖源码排除见 [cargo-llvm-cov 0.9.1 规则](https://github.com/taiki-e/cargo-llvm-cov/tree/v0.9.1#exclude-file-from-coverage)（查阅 2026-09-18）。内联 `#[cfg(test)]` 模块未自动按内容排除，会影响分母；后续可移到独立测试文件并记录口径变化，不能通过删除断言提高覆盖率。
- 子进程使用 `CARGO_BIN_EXE_*`，继承工具设置的 `LLVM_PROFILE_FILE`。报告异常时核对实际二进制、profile 合并、工具版本及默认过滤；不能仅因某个 show 视图无零行就宣称全部生产代码已覆盖。
- 新排除项必须记录具体文件/符号、证明、配置位置、替代验证和复查条件。只写“豁免”不会改变实际统计，分母变更必须说明，不能混同比较前后数字。

**未覆盖点处理：**

1. 可达的业务、错误和资源生命周期逻辑：补行为测试，优先验证输出、不变量及失败后状态。
2. 难以稳定触发的系统失败：保留错误处理，采用可控故障或最小测试接缝；暂未验证的触发路径列为缺口。OS 线程创建失败属于可能发生的故障，不能称为逻辑不可达。
3. 有证据证明不可达：优先删除废弃实现；确需保留的防御逻辑单独记录证据与验证边界。低风险、低频或暂时没想到测试方法都不是排除理由。

当前特例与缺口：Rust 报告中的 `panta-launcher` 通过命令参数排除（其 native 调度不属于 Rust 插桩链），普通 Cargo 测试仍执行其测试；native 报告单独覆盖 C++ 目标。任务 032 负责复查两条报告边界。`task.rs` 线程创建失败的回滚函数已有直测，真实 spawn 失败触发未测且未从报告排除。dslc 内的 dsl-core 实例是可执行代码，不登记为“结构性不可执行”；其实际缺口继续核查。

## 测试编写约定

优先清晰的行为断言和有上下文的失败诊断。测试可返回 `Result<(), Box<dyn Error>>` 并用 `?`，不得吞掉设置夹具/提交任务的失败。负向断言可用 `matches!` 或明确的 `match`；`unwrap`/`expect` 仍遵守 workspace Clippy 规则。按错误构造成本选择 `ok_or`/`ok_or_else`，不为数字禁止正常控制流或有诊断意义的闭包。不添加仅重复调用 API 以提高某个二进制实例计数的测试。

## 提交门禁（git hooks）

`.githooks/pre-commit` 执行 `cargo check --locked --workspace --all-targets`、`cargo format` 与 `cargo lint clippy`。新机器需一次性启用：

```sh
git config core.hooksPath .githooks
```

原生 hooks 不额外引入 Node；本地 hook 可被跳过且检查工作树，不能替代 CI 或部分暂存时的提交自洽检查。

## 后续实施

011/032/043 继续补逐项测试发现与跳过检查、按模块覆盖率、工具版本和质量报告制品。每个新门禁先固定工具和命令、验证成功/失败场景，再同步 CI、规范与任务。所有排除和工具限制必须可追溯。
