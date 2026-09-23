# 跨语言质量工具链

[模块导航](README.md) · [实施任务 032](../task/032-cross-language-quality-gates.md) · [统一测试入口 011](../task/011-test-quality-entrypoints.md) · [性能测试 048](../task/048-performance-testing.md)

## 目标与范围

质量工具链覆盖已经存在的 Rust、C++20、QML/Qt 和 CMake。Python/Node 源码真正进入仓库后才引入相应工具。以下区分当前门禁与规划目标，工具或文档就绪不代表任务验收完成。

## 完成情况（2026-09-19 代码复审）

**覆盖率与 sanitizer 门禁尚未完成全部验收。** Cargo 入口、CMake 原生构建图与根质量入口已接通；当前托管 LLVM 22.1.7 代码的三平台 CI 已全绿（run 35425146629），032/042 仍为 `in-progress`，100% 覆盖率门禁与 sanitizer/平台运行库矩阵未关闭。

| 检查面 | 当前实现 | 验证边界 |
|---|---|---|
| Rust | rustfmt、Clippy、deny、machete、测试、覆盖率 | 全局阶段下限不保证逐模块或逐提交不下降 |
| C++ / CXX | 托管 LLVM、clang-tidy、include-cleaner、Cppcheck、GTest/QtTest | 合并 CMake/CXX 编译命令，分析自有翻译单元；仍需三平台冷构建证据 |
| C++ 动态检测 | sanitizer 矩阵：ASan+UBSan 三平台、TSan 仅 Linux/macOS；`cargo sanitize` 每组合独立插桩树并完整执行 CTest | Qt/SDK 预编译库与 Cargo CXX adapter 不插桩，检测边界为自有 C++；错误经非零退出阻断，矩阵依据见任务 042 |
| Rust 动态检测 | Miri（固定日期 nightly）解释纯 Rust crate 测试：`cargo ub-check`；关闭隔离放行文件系统夹具 | CXX FFI 与进程/构建类 crate 不可解释；crash 模块进程测试在 Miri 下跳过，真实平台照常执行 |
| QML / CMake | qmlformat、qmllint、行为测试、cmake-format/lint | QML 源码级覆盖率未实现 |
| 工具供给 | 固定 LLVM/CMake/Ninja/uv/Python/Cargo 工具；锁文件固定 Python wheels | 工具缓存加锁、版本隔离、临时安装后发布；平台 SDK 仍是开发者前置 |
| CI | 三平台 Ninja check/build/test 与实际编译器核验；Ubuntu 独立 lint/format/audit/coverage；三平台 sanitizer 与 Ubuntu Miri 按路径域触发 | 本机验证不能代替当前提交的 Linux/Windows CI |

## 工具版本与来源（任务 032/042/043）

| 工具 | 固定版本 | 供给与入口 | 职责 |
|---|---|---|---|
| cargo-deny | 0.20.2 | `cargo audit`；`target/panta-tools/cargo-deny/<version>` | RustSec、许可证、重复/通配依赖 |
| cargo-machete | 0.9.2 | `cargo lint machete`；同类版本目录 | 扫描仓库根 workspace 的未使用 Rust 依赖 |
| cargo-llvm-cov | 0.9.1 | `cargo coverage`；同类版本目录 | 使用 rustup 配套工具测量 Rust 业务代码 |
| LLVM | 22.1.7 | 官方资产及 SHA256：`crates/panta-build/src/lib.rs` | clang/clang++/clang-cl 编译；clang-format 格式；clang-tidy 分析；llvm-cov/profdata 解析 native 覆盖率；sanitizer 编译开关与运行库（compiler-rt，随 `lib/clang` 资源目录保留） |
| Miri | nightly-2026-09-15（rustc 1.100.0-nightly 574ff7d98） | `cargo ub-check`；rustup 组件，日期固定于 runner 常量，升级同步回填 032 | 解释执行纯 Rust crate 测试，检出越界、悬垂引用、数据竞争等 UB |
| include-cleaner | 随 LLVM 22.1.7 | `cargo lint includes`，仅启用 `misc-include-cleaner` | 缺失/多余 include 建议变为错误；取代独立 IWYU |
| Cppcheck | 2.17.1（wheel 1.5.1） | `cargo lint cppcheck`；`pyproject.toml`/`uv.lock` | 补充 `unusedFunction`，`--error-exitcode=1` 阻断 |
| qmlformat / qmllint | Qt 6.11.2 | Cargo/CMake 供给 Qt | 应用及测试 QML 格式、类型检查 |
| uv / CPython | 0.8.22 / 3.13.7 | `crates/panta-build/src/python.rs`；官方固定 uv 资产校验 SHA256 | 禁止选用系统 Python；解释器、venv、缓存均在 target |
| cmake-format / cmake-lint | cmakelang 0.6.13 | `cargo format` / `cargo lint cmake` | `uv run --locked --managed-python --no-build`，仅消费锁定 wheels |

LLVM、CMake、Ninja、uv 按版本与摘要隔离到 `target/panta-tools/<tool>/<version-sha256>/`；安装持有 OS 文件锁，下载到临时文件并校验，解包成功后才发布目录。进程中断释放锁，下次持锁重试清理未发布目录；旧版本不受失败升级影响。Cargo 扩展按版本隔离并沿用同一发布机制。Qt 供给也串行化，防止 build/format 同时解包。升级同步供给清单、CMake 版本检查、CI 缓存、文档与验收。

Cppcheck wheel 来自 [cppcheck-wheel](https://github.com/msclock/cppcheck-wheel)，不是 LLVM 发行包；锁定包和平台 wheel 摘要，不使用 apt/Homebrew/PATH。其 `unusedFunction` 用于补足跨文件未使用函数分析，Qt 元对象和 Rust 调用的边界须有明确规则，不能将静态分析等同于检出所有死代码。include-cleaner 按[上游限制](https://clang.llvm.org/extra/clang-tidy/checks/misc/include-cleaner.html)诊断主文件，头文件仍需被真实翻译单元包含。独立 IWYU、Cppclean 和无 Node 需求的 knip 不纳入工具链。

### Cppcheck 与 include 检查边界

Cppcheck 从真实数据库保留编译宏、自有头文件及 moc/CXX 生成翻译单元，但移除 Qt SDK 的 include 路径，改用内置 `qt`、`googletest` 和 `tests/cppcheck-qt.cfg` 库模型。Qt 6.11.2 的 moc revision 与静态插件宏在该配置中固定；Qt 真实头文件、宏展开与类型检查继续由 LLVM 编译/clang-tidy 验证。升级 Qt/Cppcheck 时复查模型，不能以库模型检查替代真实构建。使用 `--check-level=exhaustive` 避免默认分支分析截断，解析错误仍阻断。

仅排除生成目录的 `unusedFunction`、GoogleTest 模型生成的 `__*` 测试注册符号（`tests/cppcheck-suppressions.xml`）及固定 CXX 生成头中的 `eraseDereference` 误报。moc/CXX 的实际调用关系仍参与分析，禁止排除整个自有源目录。真实工具反例已验证：新增无调用函数返回非零，补充另一个翻译单元中的调用后通过。动态 QML/Qt 注册仍有静态模型局限，不能宣称检出所有未使用代码。

`.clang-tidy` 对 Qt 基础转发链 `QtGlobal/qglobal.h/qtypes.h/qminmax.h/qnumeric.h` 以及 `qstringliteral.h` 配置精确 include-cleaner 例外：Qt 公开 API 约定允许通过聚合/转发头使用 `qreal`、`qMax`、`qRound`、`QStringLiteral` 等基础定义，检查器不应迫使源码改用标准库替代品；范围不扩展到 Qt 业务模块或自有头。源码仍优先写稳定的 Qt 聚合入口（如 `<QtGlobal>`、`<QString>`）。另对 QtTest `qtest.h` 保留 `QTRY_*` 宏例外，因为检查器不跟踪宏内部声明。头文件列表保持连续且无注释；这些例外不屏蔽其他缺失或多余 include。

## 当前执行入口

- `cargo build` 准备构建所需 LLVM、CMake/Ninja、Qt/GoogleTest 并完整链接；`cargo test --workspace` 保留原生 Cargo 语义，根集成测试执行 qmllint 和完整 CTest。根目录默认成员是 `panta-launcher`，裸 `cargo build`/`cargo test` 只覆盖该包及其依赖。Debug/Release 与显式本机 `--target` 的目录保持一致；跨目标构建明确拒绝。
- `cargo format` 就地修复 Rust、C++/CXX、CMake、应用及测试 QML 的格式；`cargo format --check` 只验证不改动。Qt 格式工具直接供给，不配置或编译 native 工程。
- `cargo lint clippy|machete|cmake|qmllint|clang-tidy|includes|cppcheck` 按需准备工具；不带工具名顺序执行全部，并在每个阶段开始时打印进度。clang-tidy 对自有翻译单元逐文件检查，最多 8 个 worker；成功的文件默认不输出，长时间停留在该阶段时可查看 worker 活动或单独重跑 `cargo lint clang-tidy --check` 定位慢项。不要按“测试代码复杂”整体排除测试文件；只有确认工具本身无法处理的单文件才可按具体诊断设置精确例外。缺省为修复模式（clippy/clang-tidy 应用可自动修复项后回落检查，其余工具只报告），`--check` 只验证不改动；CI 与 pre-commit 一律使用 `--check`。audit/machete 编译 runner 时不下载 LLVM/CMake。
- `cargo audit`、`cargo coverage`、`cargo coverage native` 分别负责依赖审计、Rust 门禁和 native 覆盖率报告。`cargo quality` 聚合格式、lint、审计、测试，不包含 coverage。
- `cargo sanitize` 按平台固定 sanitizer 矩阵（Linux/macOS：ASan+UBSan 与 TSan 独立树；Windows：ASan），每组合一个 `target/native/debug-sanitizer-<组合>` 插桩树并完整执行 CTest；组合合法性在 configure 期校验，TSan 互斥与平台缺口按官方文档注明，矩阵与依据见任务 042。`cargo ub-check` 以固定 nightly 解释执行 panta-core、panta-dsl-core、panta-foundation 测试，产物在 `target/miri`；CXX FFI 与进程类 crate 不在其语义内。
- `cargo run --locked -p panta-tests -- toolchain` 检查托管工具版本、Qt/GoogleTest 文件、CMakeCache 的 C/C++ 编译器/CMake/Ninja 路径，以及合并编译数据库中包括手写 CXX adapter 在内的实际编译器。系统旁路不能作为该检查的通过证据。
- runner、FFI 与 launcher 共用 `panta-build`，无需通过 `#[path]` 导入其他 crate 私有文件或整体关闭 dead-code 告警。质量数据库位于 `target/native/<profile>/quality/compile_commands.json`；只选自有翻译单元，头文件不单独伪造编译命令。
- CI 三平台单 job 顺序执行 check、build、工具核验与完整测试套件；lint 按工具与变更路径域拆分触发（machete→rust、cmake→native、qmllint→qml/native），dependency-audit 仅随依赖清单触发，纯文档变更整场跳过。Cargo 工具缓存仅由 main push 的 check job 保存，其余 job 只恢复；缓存瘦身在供给代码完成——panta-build 安装归档发布即删并按白名单裁剪 LLVM，Qt/SDK CMake 供给发布即删归档，三平台条目合计控制在仓库 10 GB 配额内。CI 不单独安装非 Rust 质量工具。Cargo aliases 和内部 Cargo 调用默认 `--locked`，直接 `cargo build/test/check` 按原生 Cargo 语义由调用者选择 `--locked`。

真实窗口、DPR、多显示屏、GPU、线程及 ABI 检查单独留证。无头组件测试不能代替所有平台的真实图形生命周期验证。

## 覆盖率门禁规则（2026-09-18 评审修订）

Rust 门禁与 CI 一致的命令（仓库根目录）：

```sh
cargo coverage
```

**配置下限为全局函数 89%、行 92%，两者都阻断；2026-09-19 macOS 托管工具实跑为函数 89.69%、行 94.23%。** 这是补齐历史缺口期间的阶段门禁，不是 100% 完成证明，也不是与父提交逐项比较的防下降机制。例如函数覆盖从 89.69% 降到 89.10% 仍可能通过；一个模块的增长也可能抵消另一模块的退步。新增/修改逻辑的行为覆盖仍需评审，032 后续补按模块统计与基线比较，不得把全局通过当作模块无缺口。

Rust 函数覆盖 100% 是函数维度目标，函数进入一次不代表其内部路径经过验证；保留行门禁，不用函数 100% 宣称行/分支完整。C++ line/branch 100% 为待落地目标；QML 以可执行绑定和关键状态场景的行为断言为准。固定 stable 工具链当前未采集 Rust branch 数据，该项记录为未测，不计作通过。目标完成须同步 032 验收证据，不能仅上调一个阈值就标 done。

`cargo coverage native` 在独立 `target/native/debug-coverage` CMake 树启用 `-fprofile-instr-generate -fcoverage-mapping`，共用托管依赖缓存；清空本次 raw profiles 后执行 CTest，使用同一 C++ LLVM 的 `llvm-profdata`/`llvm-cov` 生成 `target/native-coverage/summary.txt`，CI 上传报告。它统计 native C++（当前也包含自有测试源），尚不统计未插桩的 Cargo CXX adapter；QML 场景只验证行为，不声称源码级覆盖率。此 job 暂产报告，待稳定基线后再固定百分比门禁。

**统计口径与工具约束：**

- Rust 覆盖率使用 `rust-toolchain.toml` 锁定 rustc，配套安装同一工具链的 `llvm-tools-preview`；核对 PATH 与 `LLVM_COV`/`LLVM_PROFDATA`，不混用系统 LLVM 解析不同版本的插桩数据。
- 默认的 tests/examples/benches、生成构建树、依赖源码排除见 [cargo-llvm-cov 0.9.1 规则](https://github.com/taiki-e/cargo-llvm-cov/tree/v0.9.1#exclude-file-from-coverage)（查阅 2026-09-18）。内联 `#[cfg(test)]` 模块未自动按内容排除，会影响分母；后续可移到独立测试文件并记录口径变化，不能通过删除断言提高覆盖率。
- 子进程使用 `CARGO_BIN_EXE_*`，继承工具设置的 `LLVM_PROFILE_FILE`。报告异常时核对实际二进制、profile 合并、工具版本及默认过滤；不能仅因某个 show 视图无零行就宣称全部生产代码已覆盖。
- 新排除项必须记录具体文件/符号、证明、配置位置、替代验证和复查条件。只写“豁免”不会改变实际统计，分母变更必须说明，不能混同比较前后数字。

**未覆盖点处理：**

1. 可达的业务、错误和资源生命周期逻辑：补行为测试，优先验证输出、不变量及失败后状态。
2. 难以稳定触发的系统失败：保留错误处理，采用可控故障或最小测试接缝；暂未验证的触发路径列为缺口。OS 线程创建失败属于可能发生的故障，不能称为逻辑不可达。
3. 有证据证明不可达：优先删除废弃实现；确需保留的防御逻辑单独记录证据与验证边界。低风险、低频或暂时没想到测试方法都不是排除理由。

当前特例与缺口：Rust 报告中的 `panta-launcher` 通过命令参数排除（其 native 调度不属于 Rust 插桩链），普通 Cargo 测试仍执行其测试。`panta-tests` 也从 Rust coverage 调度排除：它的集成测试需要 launcher 先生成 native 构建树，冷环境下无法随排除 launcher 的 Rust 插桩运行。此项同时排除了 runner 自身 Rust 代码，属于真实覆盖缺口，不是不可执行代码；043 需拆分可独立测量的调度逻辑后复查，不能与历史分母直接比较。`panta-build` 从原本不计量的 launcher 构建支持迁出，继续不计入业务门禁分母；安装并发/失败恢复和数据库选择有独立单元测试，但未获得完整工具支持代码覆盖率。native 报告单独覆盖 C++ 目标。任务 032 负责复查两条报告边界。`task.rs` 线程创建失败的回滚函数已有直测，真实 spawn 失败触发未测且未从报告排除。dslc 内的 dsl-core 实例是可执行代码，不登记为“结构性不可执行”；其实际缺口继续核查。

## 测试编写约定

优先清晰的行为断言和有上下文的失败诊断。测试可返回 `Result<(), Box<dyn Error>>` 并用 `?`，不得吞掉设置夹具/提交任务的失败。负向断言可用 `matches!` 或明确的 `match`；`unwrap`/`expect` 仍遵守 workspace Clippy 规则。按错误构造成本选择 `ok_or`/`ok_or_else`，不为数字禁止正常控制流或有诊断意义的闭包。不添加仅重复调用 API 以提高某个二进制实例计数的测试。

## 提交门禁（git hooks）

`.githooks/pre-commit` 只执行 `cargo format --check`（秒级、不触发 native 构建、不改动文件）；lint 全套、依赖审计、覆盖率与测试套件由 CI 承担——lint 工具经 build.rs 触发全量 native 构建支撑编译数据库，放进 hook 会让每次提交阻塞数分钟。新机器需一次性启用：

```sh
git config core.hooksPath .githooks
```

原生 hooks 不额外引入 Node；本地 hook 可被跳过且检查工作树，不能替代 CI 或部分暂存时的提交自洽检查。

## 架构评估与后续实施

Cargo 统一用户入口、CMake 管理 native 图、CXX 管理 Rust/C++ 边界，以及 `tests/cpp` / `tests/qml` 分类是合理基础。工具种类已覆盖当前语言的主要需求，无需继续堆叠 Cppclean 或尚无 Node 源码需求的 knip。

当前 CAE 依赖仍由 031/038 逐步交付，VTK/OCCT/Netgen 的全平台供给和消费验证不能因 CMake 接口已存在就标完成。cargo-deny 只审计 Cargo 依赖图，Qt 和 native SDK 的许可证、漏洞与制品来源仍需独立清单和更新机制。

本轮已实现共享安装互斥、原子发布、按命令准备工具、Windows Ninja/SDK 环境、CXX 数据库合并和实际工具路径核验。043 的三平台证据已由 run 35425146629 补齐并关闭；sanitizer 矩阵（`cargo sanitize`）与 Rust Miri 入口（`cargo ub-check`）已接线并完成 macOS 本机实证与受控失败验证，042 保持 in-progress 等待三平台 CI 的 sanitizer 证据（Windows 仅 ASan），032 继续补按模块覆盖率、CXX/QML 测量缺口与 native 百分比基线。每个排除与工具限制须可追溯，实际验证结果以任务记录为准。

性能测试（Criterion、火焰图、QML Profiler、Massif 等）按维护者决策不纳入本工具链的 CI 门禁，属于开发侧工作台；工具矩阵与命令见[性能测试与剖析](performance.md)与任务 048。
