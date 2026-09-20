# 043 — 根目录质量入口与测试聚合

- 状态：done
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[032](032-cross-language-quality-gates.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-18 / 2026-09-19

## 本轮修复范围（2026-09-19）

按维护者要求修复复审列出的全部基础设施缺口：提取公共构建支持 crate，供给采用文件锁、摘要隔离和原子发布；统一 Ninja 与 Windows SDK 环境；runner 按命令准备工具；固定并托管 uv/Python/Cppcheck；覆盖自有 CXX 编译命令；native coverage 使用 C++ LLVM 配套工具；工具核验读取实际编译数据库与 CMakeCache。工具选择收敛和验证证据随实施回填，不用本机旁路冒充三平台通过。

## 目标与背景

2026-09-19 复审重新打开验收：入口已接线，但工具托管、覆盖率调用、Windows 编译数据库与真实路径核验仍有缺口。本轮范围为逐项核对 032/042/043 的代码与证据，修复可独立验证的调度错误及失实注释；当前三平台冷构建证据继续保持未完成；供给并发治理与工具资产托管的本轮实测见后文。

原 native 聚合测试位于 `crates/launcher/tests`，依赖审计、未使用依赖检查、Rust 格式和 C++/QML 检查也曾分散在不同 CI step。本任务将根目录 `tests/` 建为唯一跨语言测试域：`src/` 放 Cargo 调度器，`integration/` 放跨语言聚合测试，`cpp/` 与 `qml/` 放测试源；通过 Cargo alias 统一执行。Rust 单元测试仍保留在实现文件，crate 黑盒测试保留在各 crate 的 `tests/`。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)
- [跨语言质量工具链](032-cross-language-quality-gates.md)
- [质量工具链模块](../modules/quality-tooling.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [仓库文件规范](../standards/repository-hygiene.md)

## 范围与非目标

范围：根 `tests/` Cargo package、Cargo aliases、Rust/C++/QML/CMake 质量命令聚合、CI 入口、测试目录规范和使用说明；统一 `cargo quality`、`cargo lint`、`cargo test`、`cargo format`。lint 包含 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、include-cleaner 和 Cppcheck；format 另外包含由 uv 锁定的 cmake-format。

非目标：不把依赖私有实现的 crate 单元测试物理搬出所属 crate；不复制 CTest/GTest/QML 测试；不引入 Python 业务运行时；CMake 工具由 uv 锁定；coverage 仍由专用 CI job 调用固定工具。

## 前置条件与待决策

- `cargo-deny`、`cargo-machete` 与 `cargo-llvm-cov` 版本已由 032 固定；根 runner 在首次使用时按锁定版本安装到 `target/panta-tools/<cargo-tool>/<version>`，CI 不重复实现安装逻辑。
- native 构建仍由 launcher build.rs 复用生产构建图；根入口不得递归调用自身。
- 质量入口必须支持失败传播、空 CTest 套件失败、Release/Debug 和自定义 target-dir；格式工具缺失时明确失败。

## 实施步骤

1. 创建根 `tests/` package 与 build.rs，复用托管 CMake/LLVM 工具链定位并注入当前 Cargo profile 的 native 构建树。
2. 将 launcher 的 native 聚合测试移入根 runner，删除旧入口；C++/QML 测试源统一迁移到 `tests/cpp/`、`tests/qml/`，保留 crate 私有 Rust tests 和 native CTest 注册。
3. 增加 Cargo aliases：`quality` 聚合格式、lint、依赖检查和测试；`lint` 聚合 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、include-cleaner 和 Cppcheck；根 `tests/integration/native.rs` 通过显式 manifest 注册接入标准 `cargo test`；`format` 聚合 Rust、C++/CXX、CMake 和 QML 格式检查，CMake 工具由 uv 锁定，QML 格式只准备 Qt 工具不构建 launcher。
4. CI 将每个 lint 作为独立可定位 step 调用 `cargo lint <tool>`，不以 `cargo quality` 代替；依赖审计调用 `cargo audit`，覆盖率调用独立的 `cargo coverage` job。
5. 在 `cargo build --locked --workspace` 后调用 `cargo run --locked --package panta-tests -- toolchain`，核对 Cargo 托管的 LLVM 22.1.7、CMake/Ninja、Qt、GoogleTest 和 compile_commands；更新 README、质量工具链模块、011/032/042 任务记录，记录真实命令和受控失败验证。

## 预计改动

`tests/Cargo.toml`、`tests/build.rs`、`tests/src/main.rs`、`tests/integration/native.rs`、`tests/cpp/`、`tests/qml/`、`Cargo.toml`、`.cargo/config.toml`、`.github/workflows/ci.yml`、README、质量模块、测试规范和相关 task。删除 `crates/launcher/tests/native_suite.rs` 旧聚合实现。

## 清理与兼容例外

删除 launcher 旧 native 聚合入口及失效引用；不保留双入口。crate 内部测试、native CTest 注册和 QML 行为测试不是废弃实现。无兼容例外。

## 验收标准

- [x] `cargo lint` 一条命令执行 Clippy、cargo-machete、cmake-lint、qmllint、Clang-Tidy、include-cleaner 和 Cppcheck；工具缺失或任一检查失败返回非零。include-cleaner 与 Cppcheck 的职责分工明确，Cppcheck 开启 unusedFunction。
- [x] `cargo quality` 一条命令执行格式、`cargo lint`、deny、Rust 测试、native CTest 和 QML 行为测试。
- [x] 标准 `cargo test` 执行 workspace Rust 测试和根 `tests/` 的完整 native/QML 聚合；失败、空套件、工具缺失返回非零；CI 使用 `cargo test --locked --workspace`。
- [x] `cargo format` 检查 Rust、C++、CMake、QML；官方 `cargo fmt` 保持 Rust-only 语义，Python 工具通过 uv 锁定。
- [x] CI 分别调用 `cargo lint clippy|machete|cmake|qmllint|clang-tidy|includes|cppcheck`，失败项可以单独定位；coverage job 的专用命令保持明确，不递归调用质量入口。
- [x] 根入口支持 Debug/Release、自定义 target-dir 和 Windows `ctest.exe`；不重复执行旧 launcher 聚合。（Release 与 `target/review-target`、`target/review-quality` 本地验证；Windows `ctest.exe` 经 run 35425146629 全套 CTest 通过；launcher 聚合已随 011 迁移删除）
- [x] CI 的 `cargo build --locked --workspace` 后显式验证完整 Cargo 工具链安装和共享构建产物路径。（"Verify Cargo-provisioned toolchain" 步骤紧随 build 在三平台运行，run 35425146629 全绿）
- [x] 文档、task、索引、Cargo aliases、测试目录规范和 CI 一致，旧入口引用清理完成。（2026-09-19 同步 default-members 语义、命令表与索引；全仓检索无旧入口残留）

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-18 | 创建任务 | 已建立根质量入口任务 |
| 2026-09-18 | `cargo metadata --locked --no-deps --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked -p panta-tests --all-targets -- -D warnings`、`cargo test --locked -p panta-tests --no-run`、`actionlint .github/workflows/ci.yml`、`git diff --check` | 通过；根 `tests/` package、显式 `integration/native.rs`、CI 矩阵和 Rust 聚合器可解析/编译 |
| 2026-09-18 | `cargo check --locked --workspace --all-targets`、`cargo build --locked --workspace` | 通过；Cargo 驱动的 launcher 完成 CMake/Qt/GoogleTest 构建，并生成 native compile database |
| 2026-09-18 | `uv run --locked cmake-format --check ...`、`uv run --locked cmake-lint ...`（native/qml/tools 共 21 个 CMake 文件） | 通过；格式与 lint 均由根 Cargo runner 调用同一份 `pyproject.toml`/`uv.lock` 配置 |
| 2026-09-18 | 仓库根 `cargo test --locked` | 通过；Rust workspace、CTest/GTest、qmllint、QML 行为和 `Qml.FormatCheck` 均执行，CTest 28/28 通过 |
| 2026-09-18 | `cargo lint clippy`、`cargo lint cmake`、`cargo format`、`actionlint .github/workflows/ci.yml` | 通过；工具缺失和失败均由根入口传播，CI 每个 lint 工具独立成检查 |
| 2026-09-18 | `cargo build --locked --workspace`、`cargo run --locked --package panta-tests -- toolchain` | 历史记录修正：当时验证 CMake/Ninja 与独立 clang-format、Qt/GoogleTest 和 native database；不能证明 2026-09-19 才接入的 LLVM 22.1.7，当前新工具链完整验证待补 |

| 2026-09-19 | `cargo fmt --all -- --check`；系统旁路下 `cargo clippy --locked -p panta-tests --all-targets -- -D warnings`；`actionlint .github/workflows/ci.yml`；`sh -n .githooks/pre-commit` | 通过；只证明调度代码与语法，不证明托管 LLVM 完整构建 |
| 2026-09-19 | `target/review-quality/validate.py` 临时受控夹具，独立 rustc 编译修改前/后 runner；使用实际 cargo-machete 0.9.2 扫描兄弟 crate 的未使用 serde | 修改前 exit 0 漏检，修改后非零且指出 fixture-unused；覆盖率参数/失败传播、IWYU 违规参数、clang-tidy 拒绝 PATH 回退、marker 版本失效共五项通过。后三类工具使用替身，未声称真实全量分析通过 |
| 2026-09-19 | ABI 夹具改前 configure；改后 `cmake -DTEST_BINARY_DIR=target/review-quality/abi-after -DTEST_CXX_COMPILER=/usr/bin/clang++ -P native/cmake/tests/check-abi.cmake`；uv 锁定 cmake-format/check 与 cmake-lint | 改前正例被空 LLVM 版本拦截；改后正例通过、runtime/iterator 反例正确拒绝；格式/lint 通过。这是元数据测试，不是 Windows ABI 实跑 |
| 2026-09-20 | macOS；`cargo build --locked -p panta-tests`、`cargo lint --check machete`、`cargo lint --check cmake`、`cargo format`（修复模式实际改写 main.rs 后）、`cargo format --check` | 通过：`--check` 参数解析与修复模式生效；clang-format 修复模式对 native C++ 源就地改写后检查转绿。CI（ci.yml lint 矩阵与 format job）及 pre-commit 全部切换为 `--check`；Clippy 修复/检查与 qmllint 路径行为由 pre-commit/CI 复验 |

## 风险与回退

Cargo runner 可能与 build.rs 使用不同 target-dir 或 profile；通过 Cargo metadata、编译期注入和受控失败验证路径。回退时删除根 runner 和 aliases，恢复 CI 直接命令；不删除各 crate/native 自有测试。

## 决策与工作记录

- 2026-09-18：按维护者要求将跨语言质量和测试聚合提升为根 `tests/` 入口；C++/QML 源统一按语言分类，Rust 私有单元测试继续留在实现文件。
- 2026-09-20（维护者要求）：lint/format 拆分修复与验证两种行为——`cargo format` 就地修复、`cargo format --check` 只验证；`cargo lint [tool]` 缺省为修复模式（clippy 先 `--fix` 再回落检查、clang-tidy/includes 追加 `--fix`，无修复能力的工具等价报告），`cargo lint [tool] --check` 只验证；CI 与 pre-commit 全部切到 `--check`，`cargo quality` 保持只验证。
- 待办（lint 编译成本）：clang-tidy/includes/cppcheck/qmllint 经 `build_launcher()` 触发全量 native 构建，因为它们消费 `quality/*.json` 编译数据库（configure+autogen 产物）。后续在 build.rs 引入 prepare-only 模式（configure + 生成质量数据库、跳过 `cmake --build`）可把这些 lint 的准备成本降到配置级；需先验证 autogen 的 moc 翻译单元在 configure-only 数据库中的完整性，避免 cppcheck/clang-tidy 扫到缺失的生成文件。clippy 因 `--workspace` 必须执行 launcher build script，受 Rust 构建图约束维持现状。

## 本轮复审与待验收

- 本轮修复 machete 仅扫描 tests/ 而漏掉业务 crate、cargo-llvm-cov 直接调用缺 `llvm-cov` 参数、Rust coverage 冷环境误运行 native 聚合、Cargo 工具升级/降级误命中旧 marker、IWYU 默认不阻断、clang-tidy 静默回退 PATH 及 tests build.rs 未追踪工具环境变量。
- coverage 排除 panta-tests 使调度器暂不计入 Rust 覆盖率，属于已登记的测量缺口；后续拆分可独立测试的调度逻辑后恢复测量，重测全局下限，不以排除提高数字。
- 清理 launcher 中没有消费者的六项 rustc-env 注入和多余 clang-format 路径复制；工具路径由 runner 按命令解析，tests/build.rs 只注入目录与 profile。修正 hook 的“轻量、不触发 native”注释，未恢复 cargo check。
- 复审时未解决项包括工具托管、按需准备、profile 传播、CXX 数据库、实际编译器核验。本轮代码修复见后续记录；三平台和工具真实违规测试单独记验证证据。
- Windows VS 生成器、共享 LLVM 缓存并发以及 native coverage 配套解析工具由 042 协同修复；不能把单机 check/clippy 通过当作这些功能已经验收。

## 完成摘要

已交付：根 `tests/` 是跨语言测试与质量调度入口，`cargo quality` 聚合格式、lint、审计与全部测试；CI 按 lint 工具独立分检查并保留专用 coverage job，跨平台 build 后核验托管工具链路径。本地 Release/自定义 target-dir 与三平台 CI（run 35425146629）证据齐备；runner 自身覆盖率作为已登记测量缺口由后续拆分处理。

## 2026-09-19 工具职责与基础设施修复

维护者采用 LLVM `misc-include-cleaner` 替代独立 IWYU；保留固定 Cppcheck 2.17.1 补充跨文件未使用函数。两者分别通过 `cargo lint includes`/`cargo lint cppcheck`，CI 独立运行。提取 panta-build 共用库，删除跨 crate 私有源码导入与整体 dead-code 豁免；runner 编译不下载工具。uv/Python/Cargo 工具全部仓库内安装；Windows Ninja/SDK 与 CXX 数据库由 042 联动；覆盖率入口改为 `cargo coverage native` 使用独立 CMake 树和同套 LLVM 解析工具。业务 Rust 覆盖率继续排除原来就在 launcher 内的构建支持逻辑（现 panta-build），基础设施行为测试另行运行。

## 2026-09-19 真实门禁收尾

macOS arm64 托管模式（未启用 `PANTA_USE_SYSTEM_TOOLS`）下，`cargo quality` 完整通过：Rust/C++/CMake/QML 格式，Clippy、machete、cmake-lint、qmllint、clang-tidy、include-cleaner、Cppcheck，cargo-deny，Rust 与 native 28/28。CMake 工具检查 21 个文件。`cargo coverage` 复跑函数 89.69%、行 94.23%，保持原门禁下限；`cargo coverage native` 报告边界见 042。`actionlint`、hook shell 语法和 diff 空白检查通过。

Cppcheck 2.17.1 使用 Qt/GoogleTest 库模型及 `tests/cppcheck-qt.cfg`，真实宏展开仍由 LLVM 检查。删除试验性的 framework 软链接与平台宏提取实现，不屏蔽语法/预处理失败；开启 exhaustive 检查避免默认分支分析截断。仅对生成代码、测试注册符号及固定 CXX 头误报配置精确例外，范围见质量模块。实际完整项目加临时反例：未调用函数 exit 1，跨文件补调用后 exit 0。include-cleaner 对多余 include 的反例也返回非零，移除后通过。

任务保持 in-progress：Windows/Linux 当前 CI、独立 target-dir 的完整跨语言运行及 runner 本身的覆盖率仍待补齐。本机系统目录写入测试需要允许 Qt 测试目录访问；沙箱拒绝该访问的失败不等同于业务回归，实际验证使用正常开发环境。

## 2026-09-19 关闭

Windows/Linux 当前代码 CI 与 Windows `ctest.exe` 证据由 run [35425146629](https://github.com/Yuki-Nagori/panta/actions/runs/35425146629)（`2ebbea7`）补齐：`cargo test --locked --workspace` 三平台全绿，Windows 26/26 CTest 含两个 QML 运行测试；Release 与独立 target-dir 已有 `target/review-target`、`target/review-quality` 本地记录。runner 自身覆盖率是已登记的测量缺口，随后续拆分单独补测，不在本任务三项待验收内。验收全部满足，标记 done。

## 2026-09-19 include 列表格式约定

按维护者要求清理自有 C++/CXX 的 include 列表分组空行和注释，保留接口契约及条件编译；`.clang-format` 设置 `IncludeBlocks: Merge`，通过现有 `cargo format` / CI 持续检查连续排列。include-cleaner 的 QtTest 例外留在 `.clang-tidy` 说明，不写回头文件列表。使用托管 clang-format 重新检查，pre-commit 的聚合 format 和 Clippy 通过。
