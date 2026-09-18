# 跨语言质量工具链

[模块导航](README.md) · [实施任务 032](../task/032-cross-language-quality-gates.md) · [统一测试入口 011](../task/011-test-quality-entrypoints.md)

## 目标与范围

质量工具链覆盖已经存在的 Rust、C++20、QML/Qt 和 CMake。Python/Node 源码真正进入仓库后才引入相应工具。以下区分当前门禁与规划目标，工具或文档就绪不代表任务验收完成。

## 固定工具版本（任务 032）

| 工具 | 固定版本 | 安装/来源 | 当前用途 |
|---|---|---|---|
| cargo-deny | 0.20.2 | `cargo install cargo-deny --locked --version 0.20.2` | RustSec、许可证、重复/通配依赖；配置 `deny.toml` |
| cargo-machete | 0.9.2 | `cargo install cargo-machete --locked --version 0.9.2` | 未使用依赖；launcher/panta-ffi 的 build.rs `DEP_*` 消费在 metadata 登记豁免 |
| cargo-llvm-cov | 0.9.1 | `cargo install cargo-llvm-cov --locked --version 0.9.1`，rustup `llvm-tools-preview` | Rust 函数及行覆盖率阶段门禁 |
| qmlformat / qmllint | 6.11.2 | Qt 托管预编译供给 | CTest `Qml.FormatCheck`（stdout diff）与 `all_qmllint` |
| clang-format | 20.1.0 | `muttleyxd/clang-tools-static-binaries` release `master-796e77c`；资产 URL/SHA256 固定于 `crates/launcher/src/provision.rs` | native_suite 检查 native 与 panta-ffi 自有 C++；缓存按版本/摘要隔离并校验复用文件 |
| cmake-format | 未引入 | 待确认供给 | 不属于当前门禁 |

clang-format 使用第三方构建，摘要固定保证所取资产一致，不证明构建来源可信。升级时记录源码版本、构建流程与成功 run 的可定位链接，更新各平台摘要并验证实际执行版本与格式结果。同源码版本的独立构建比对可补充来源验证；不同版本对当前文件输出一致只证明这些文件的格式结果一致。

## 当前执行入口

- `cargo test --locked`：Rust 测试，launcher 集成测试顺序执行 `all_qmllint` 与完整 CTest，并检查自有 C++ 格式。构建树和 Debug/Release 配置由 build.rs 注入，CTest 使用 `-C` 与 `--no-tests=error`。尚未自动核对逐项意外缺失/跳过，011 保留该验收缺口。
- CI `rust`：三平台 build/test/fmt/clippy；`quality`：Ubuntu 上 deny/machete；`coverage`：Ubuntu 上 Rust 函数及行覆盖率阶段下限。
- C++ 覆盖率、clang-tidy、CMake 格式、actionlint、未引用资源审计和带 commit 的报告制品上传仍由 032 后续实施。

真实窗口、DPR、多显示屏、GPU、线程及 ABI 检查单独留证。无头组件测试不能代替所有平台的真实图形生命周期验证。

## 覆盖率门禁规则（2026-09-18 评审修订）

当前与 CI 一致的命令（仓库根目录）：

```sh
cargo llvm-cov --locked --workspace --exclude panta-launcher --summary-only --fail-under-functions 89 --fail-under-lines 92
```

**当前下限为全局函数 89%、行 92%，两者都阻断。** 这是补齐历史缺口期间的阶段门禁，不是 100% 完成证明，也不是与父提交逐项比较的防下降机制。例如函数覆盖从 89.69% 降到 89.10% 仍可能通过；一个模块的增长也可能抵消另一模块的退步。新增/修改逻辑的行为覆盖仍需评审，032 后续补按模块统计与基线比较，不得把全局通过当作模块无缺口。

Rust 函数覆盖 100% 是函数维度目标，函数进入一次不代表其内部路径经过验证；保留行门禁，不用函数 100% 宣称行/分支完整。C++ line/branch 100% 为待落地目标；QML 以可执行绑定和关键状态场景的行为断言为准。固定 stable 工具链当前未采集 Rust branch 数据，该项记录为未测，不计作通过。目标完成须同步 032 验收证据，不能仅上调一个阈值就标 done。

**统计口径与工具约束：**

- 使用 `rust-toolchain.toml` 锁定 rustc，配套安装同一工具链的 `llvm-tools-preview`；核对 PATH 与 `LLVM_COV`/`LLVM_PROFDATA`，不混用系统 LLVM 解析不同版本的插桩数据。
- 默认的 tests/examples/benches、生成构建树、依赖源码排除见 [cargo-llvm-cov 0.9.1 规则](https://github.com/taiki-e/cargo-llvm-cov/tree/v0.9.1#exclude-file-from-coverage)（查阅 2026-09-18）。内联 `#[cfg(test)]` 模块未自动按内容排除，会影响分母；后续可移到独立测试文件并记录口径变化，不能通过删除断言提高覆盖率。
- 子进程使用 `CARGO_BIN_EXE_*`，继承工具设置的 `LLVM_PROFILE_FILE`。报告异常时核对实际二进制、profile 合并、工具版本及默认过滤；不能仅因某个 show 视图无零行就宣称全部生产代码已覆盖。
- 新排除项必须记录具体文件/符号、证明、配置位置、替代验证和复查条件。只写“豁免”不会改变实际统计，分母变更必须说明，不能混同比较前后数字。

**未覆盖点处理：**

1. 可达的业务、错误和资源生命周期逻辑：补行为测试，优先验证输出、不变量及失败后状态。
2. 难以稳定触发的系统失败：保留错误处理，采用可控故障或最小测试接缝；暂未验证的触发路径列为缺口。OS 线程创建失败属于可能发生的故障，不能称为逻辑不可达。
3. 有证据证明不可达：优先删除废弃实现；确需保留的防御逻辑单独记录证据与验证边界。低风险、低频或暂时没想到测试方法都不是排除理由。

当前特例与缺口：`panta-launcher` 通过命令参数排除（build.rs 的 native 调度尚未接入 Rust 插桩链），普通 Cargo 测试仍执行其测试；任务 032 负责复查该边界。`task.rs` 线程创建失败的回滚函数已有直测，真实 spawn 失败触发未测且未从报告排除。dslc 内的 dsl-core 实例是可执行代码，不登记为“结构性不可执行”；其实际缺口继续核查。

## 测试编写约定

优先清晰的行为断言和有上下文的失败诊断。测试可返回 `Result<(), Box<dyn Error>>` 并用 `?`，不得吞掉设置夹具/提交任务的失败。负向断言可用 `matches!` 或明确的 `match`；`unwrap`/`expect` 仍遵守 workspace Clippy 规则。按错误构造成本选择 `ok_or`/`ok_or_else`，不为数字禁止正常控制流或有诊断意义的闭包。不添加仅重复调用 API 以提高某个二进制实例计数的测试。

## 提交门禁（git hooks）

`.githooks/pre-commit` 执行 `cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets -- -D warnings`。新机器需一次性启用：

```sh
git config core.hooksPath .githooks
```

原生 hooks 不额外引入 Node；本地 hook 可被跳过且检查工作树，不能替代 CI 或部分暂存时的提交自洽检查。

## 后续实施

011/032 继续补逐项测试发现与跳过检查、按模块覆盖率、C++ 度量、质量报告制品及受控失败验证。每个新门禁先固定工具和命令、验证成功/失败场景，再同步 CI、规范与任务。所有排除和工具限制必须可追溯。
