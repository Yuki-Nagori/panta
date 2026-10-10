# 032 — 跨语言质量工具链与 100% 覆盖率门禁

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[018](018-cross-platform-ci.md)、[019](019-gtest-native-testing.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-10-10

## 目标与背景

为仓库内每种实际使用的语言和构建描述配置统一的格式化、静态检查、测试、覆盖率、依赖/死代码检查，并在 CI 形成可复核的质量门禁。目标是对门禁统计范围内的自有可执行代码达到 100% 覆盖率；生成文件、第三方源码、仅声明性资源和明确排除的启动胶水必须有登记理由，不能用排除项掩盖未测业务逻辑。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [Rust 规范](../standards/rust.md)、[C++ 规范](../standards/cpp.md)、[GTest 规范](../standards/gtest.md)、[QML 规范](../standards/qml.md)、[CMake 规范](../standards/cmake.md)
- [质量工具链模块](../modules/quality-tooling.md)
- [注释](../standards/comments.md)、[仓库文件](../standards/repository-hygiene.md)、[文档规范](../standards/documentation.md)

## 范围与非目标

范围：Rust/Cargo、C++/CMake/GTest、QML/Qt 工具、CMake 脚本，以及未来 Python tooling 进入仓库后的 pyproject 工具配置；为每种语言选择稳定、可离线复现或固定版本的 formatter、lint/static analysis、unit/integration test、coverage、dependency audit 和死代码/未使用文件检查。建立统一命令、报告格式、差异覆盖率与 CI 门禁。

非目标：不把所有工具无条件叠加，不检查第三方/生成树源码，不把图形硬件冒烟虚构成 100% 单元覆盖，不因 Python 任务 014 deferred 而现在引入 Python 运行时。Node.js 专用工具（如 ESLint/knip）只有仓库真正引入 Node/QML JavaScript tooling 后才启用；不为类比工具预建 package.json。

## 覆盖率定义与门禁

当前 Rust 使用函数 89% / 行 92% 双阶段下限，完整规则见 [质量工具链](../modules/quality-tooling.md)。函数目标为 100%，仍保留行门禁与错误路径测试；函数进入不等于内部路径完整。C++ line/branch 100% 为待落地目标，QML 以可执行绑定和关键状态行为断言为准。stable Rust branch 数据未采集，不计作通过。

全局固定阈值不能阻止所有下降，也不能保证单个模块达标；逐模块门禁与父提交比较仍待实施；本批新增 Rust 报告制品。排除需要证明和实际配置，不能把难触发系统故障、低风险或 bin 中可执行库实例称为不可达。内联测试代码与多二进制统计需要继续核对；不能根据单一 show 视图宣称生产代码全覆盖。

## 前置条件与待决策

011 统一入口、018 三平台 CI 和 019 GTest 规则需完成；先以当前实际语言清单为准（Rust、C++、QML、CMake），Python/Node 仅在对应目录和入口真正出现后纳入。实施前固定工具版本、安装来源、报告格式、排除清单和本地/CI 命令；核实 macOS/Linux/Windows coverage 工具差异，避免平台专用工具成为唯一门禁。

## 实施步骤

1. 清点源码、生成边界和现有测试，建立每种语言的工具矩阵及统一质量入口。
2. 接入格式化和 lint：Cargo fmt/Clippy；clang-format 与 clang-tidy/编译器告警；qmllint/qmlformat；CMake 格式/脚本检查；未来 Python 用 Ruff/pytest 等固定组合，未来 Node 用 ESLint/knip 等固定组合。
3. 接入测试与覆盖率：Rust cargo test + llvm-cov 或等价工具；C++ CTest/GTest + llvm-cov/平台等价物；QML/Qt Test 与 ViewModel/组件行为测试；报告统一上传并按模块、line/branch 阻断。
4. 接入依赖安全/死代码检查：Cargo tree/audit（实际工具可用性核实）、CMake target/链接审计、未引用 C++/QML 资源检查；只有真实 Node/Python 项目才加入其生态工具。
5. 在受控失败、空测试套件、覆盖率低于 100%、格式错误、lint 错误、未使用文件和工具缺失场景下验证非零退出；CI 区分必须门禁、平台图形冒烟和明确跳过。
6. 将工具版本、缓存、报告保留策略和排除清单写入规范；清理重复脚本与失效工具配置。

## 预计改动

Cargo/CMake/CI 配置、质量脚本、coverage 配置、工具版本清单、测试入口和 `ai-docs/standards/` 相关规范。仅在实际语言/工具栈出现后新增对应配置；不创建空的 `package.json`、`pyproject.toml` 或工具占位文件。

## 清理与兼容例外

移除重复或无人调用的质量脚本、旧命令和失效排除规则。工具版本切换在同一 commit 更新配置、锁文件、CI 和文档。无兼容例外；暂未达到目标时保持 in-progress 并列出具体缺口；有外部阻塞才标 blocked，不降低阈值伪装完成。

## 验收标准

- [ ] 统一入口实际运行 Rust、C++、QML 和 CMake 的格式化、静态检查与测试；工具缺失、测试失败或报告生成失败返回非零。
- [ ] 门禁统计范围与排除理由可审阅；Rust 函数达到 100% 并保留行门禁，C++ line/branch 目标落地；按模块及防下降检查完成，未测指标明确留档。
- [ ] Rust、C++、QML 的错误路径、状态/绑定、线程/所有权和关键生命周期有行为测试；真实窗口/图形冒烟作为独立检查并记录未覆盖。
- [ ] 依赖审计、未使用代码/资源检查可在真实目录运行；未引入 Node/Python 时不创建其生态配置，出现后可按任务扩展。
- [ ] macOS/Linux/Windows CI 产出可下载、含 commit/工具版本/排除规则的覆盖率与质量报告，平台差异有明确处理。
- [ ] 受控失败验证门禁真的阻断，空测试/意外跳过不能视作通过；质量入口不递归调用自身。
- [ ] 旧脚本、重复配置和未登记排除项已清理，task、规范、CI 和命令说明一致。

## 验证计划与结果

执行统一入口的全绿和受控失败场景，逐语言核对 formatter/lint/test/coverage/audit 输出；在三平台 CI 验证报告上传和 100% 门禁。以下保留早期探索记录；早期“结构性豁免”“阈值即防下降”“show 无零行即全覆盖”等推断不作为验收依据，以本轮评审及质量工具链规则为准。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-18 | Rust 质量工具、QML formatter 与统一测试入口 | `cargo deny`、`cargo machete`、Rust fmt/clippy、QML 格式受控失败检查通过；聚合测试曾通过 101 项测试与 CTest 28/28。初始 Rust line 覆盖率为 74.21%，此历史基线不代表当前门禁。 |
| 2026-09-18 | 覆盖率口径、阈值与受控失败 | 本地 Apple LLVM 与 rustc 插桩口径不一致，覆盖率以 CI 配套 `llvm-tools-preview` 为准。空测试集、失败用例及 100% 覆盖率阈值夹具均能使聚合入口失败；现行 gate 为函数 89% / 行 92%，launcher 按任务约定排除。 |
| 2026-09-20–21 | Rust 跨平台行为测试与 `cargo ub-check` / Miri | crash 模块补齐后本地覆盖率达函数 89.18% / 行 93.84%；Miri 对纳入范围的纯 Rust crate 通过，已知进程/FFI/压力测试限制按任务说明排除或跳过。 |
| 2026-09-28 | Miri 容量压力边界复核 | `crates/panta-dsl-core/tests/fsm.rs` 中精确上限和超限两个用例分别构造最多 512/513 条边。实测 Miri 在前者运行超过两分钟仍停在 Pest 解释执行，故与已有语言 DSL 容量压力用例同口径使用 `cfg_attr(miri, ignore)`；常规 `cargo test --locked --workspace` 继续覆盖阈值，其他 FSM 解析测试继续进入 Miri。复跑固定 nightly 的三 crate Miri 退出码 0，FSM 18/18 执行用例通过、2 个容量用例跳过。Pest/Miri 性能或容量阈值变化时复查此排除。 |
| 2026-09-24 | `cargo coverage` gate 回归与修复（macOS arm64） | 复现失败 86.47% / 90.00% 后补充领域逻辑行为测试；修复后为函数 90.34%（402/445）、行 93.95%（4270/4545），未降低 89% / 92% 阈值。 |
| 2026-09-24 | `cargo format --check`（uv 0.12.18 / Python 3.14.7） | 聚合 Rust、C++/CXX、CMake、QML 格式检查通过；升级固定 uv/Python 后，Seatbelt 内的 macOS 格式检查也通过。 |
| 2026-09-24 | GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4` | 三平台测试、格式检查和 89% / 92% Rust gate 全绿。该结果不满足尚未实现的 Rust 100% 函数覆盖及 C++ line/branch 门禁目标。 |


## 风险与回退

过度追求数字会导致无意义测试或大范围排除；按模块和分支指标审查，工具无法测量时补可观察行为测试。平台 coverage 工具差异由等价报告校验，暂不能统一时保持门禁缺口可见，不降低到“尽力而为”。回退只撤销本任务配置，保留已有测试和质量证据。

## 决策与工作记录

- 2026-09-16：确立跨语言统一 `cargo` 入口与 100% 覆盖率目标；尚未达到的目标不以降低门槛或排除代码代替。
- 2026-09-18：落地 Rust 依赖审计、fmt/clippy、QML formatter、native/QML 聚合测试和受控失败校验。测试及工具供给与任务 011 的统一入口相互依赖，按一个质量链维护。
- 覆盖率规则：本地 LLVM 与 rustc 插桩曾出现口径差异，CI 配套 LLVM 是权威依据；当前 89% 函数/92% 行为阶段门槛，目标仍为 Rust 函数 100% 和 C++ line/branch 分模块门禁。launcher 的排除保持显式登记。
- 2026-09-21：增加 `cargo ub-check` / Miri，仅覆盖适用的纯 Rust crate；FFI、进程和超大压力路径按任务中记录的边界由常规跨平台测试覆盖。
- 2026-09-24：修复 coverage gate 后本地达 90.34% / 93.95%，固定 uv/Python 升级后聚合格式检查通过；run 360018 确认三平台现行门禁通过。Rust 100% 与 C++ 覆盖率门禁仍未完成。

## 完成摘要

未完成：011 统一入口、语言级 format/lint/test、Rust 89%/92% 覆盖率门槛和 Miri job 已落地；run 36001859191 确认聚合格式与当前覆盖率门槛通过。任务目标仍包括 Rust 100% 函数覆盖率、C++ line/branch 覆盖与分模块防回退门禁、跨平台可下载报告及完整受控失败验收。

## 2026-10-10 Rust 函数覆盖率跟进

本轮先处理 Rust 函数覆盖率 100% 目标：以 `cargo coverage` 报告为依据定位未覆盖函数，优先补充能验证行为的测试或清理确实无用实现；不降低阶段门槛、不增加未证明的排除项，也不为覆盖数字重复调用 API。C++ line/branch 门禁和按模块防回退留待后续单独推进。

TS 输出原先在每个 XML 写入和 UTF-8 转换点各自创建错误映射闭包；`Writer<Vec<u8>>` 的写入失败无法从正常导出路径稳定触发，造成大量重复、不可达的闭包函数计数。本轮将其合并为两个具名诊断映射函数，新增直接错误映射断言，诊断码仍为 `pa.ts_write`，不改变成功输出。另为带引号的消息 ID 与手工构造 AST 中空的非复数翻译补上行为断言，覆盖解析器和格式化器可达的边界。

继续补测格式化器对手工 AST 的 locale 回退：文档语言没有对应翻译时，使用首个可用翻译；保持 formatter 的既有宽松 AST 契约。格式化器模块函数覆盖现为 23/23；DSL core 汇总仍为 132/135，新增测试自身计入分母，未覆盖项留在解析器/FSM 的防御路径。

补测后台执行器的 `Debug` 输出，确保诊断文本包含执行器名称、worker 数和容量。执行器拒绝/启动失败时，回调不执行仍是测试必须保持的语义；10 月 10 日后续记录通过复用已覆盖回调来避免把测试辅助闭包计入未覆盖函数。

覆盖口径复核发现，inline `#[cfg(test)]` 模块中的未调用闭包也进入函数分母，例如执行器拒绝/启动失败用例为了断言回调未运行而保留的闭包；这类记录不代表生产代码未测。本轮保留单元测试在源码内联模块中的归属，也不改覆盖过滤规则：拒绝/失败路径改为复用成功路径已执行的回调函数，并继续断言拒绝状态和未执行回调的 sender 断开。这样减少了三个测试辅助闭包的未覆盖函数计数，不改变执行器实现或行为。

本轮验证：`cargo test --locked -p panta-core execution::tests --lib` 通过（6 项）；`cargo coverage` 通过，全局函数 768/825（93.09%）、行 8752/9183（95.31%），比本轮开始前记录的 769/829 少 3 个未覆盖函数；`panta-core` 为 357/393（90.84%），剩余 36 个函数缺口。Rust 函数 100% 目标仍未完成，任务继续 in-progress。

按 crate 并行排查覆盖缺口后，复核发现不少零计数项在现有集成测试中已有行为验证，详细 LLVM 明细把不同 crate 编译实例和测试闭包分别列出；不据此重复调用 API。新增 `read_source_checked` 的中途取消行为验证：输入大于 256 KiB，首块读取后第二个检查点确定性取消，断言路径和检查次数。`cargo coverage` 通过，全局函数 769/826（93.10%）、行 8762/9193（95.31%）；panta-import 为 37/41（90.24%）。panta-core 的方案设置、分析序列、材料及工艺用例已存在；dsl-core 的主要零计数项也有解析和生成测试。Foundation 致命信号处理器的 profile 不能由退出子进程可靠刷盘，需与真实行为验收区分。该审查未发现可以直接宣称达成 100% 的依据，任务继续 in-progress。

验证（macOS arm64）：`cargo test --locked --workspace` 通过，native / QML CTest 72/72；`cargo coverage`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo clippy --locked -p panta-core --all-targets -- -D warnings` 和 `cargo format --check` 均通过。完整 `cargo lint` 因沙箱禁止绑定其锁管理 TCP listener，在 Clippy 阶段退出；因此其余 lint 阶段未运行。最终覆盖率为全局函数 769/829（92.76%）、行 8752/9190（95.23%）；`panta-core` 函数 358/397（90.18%），`panta-dsl-core` 函数 132/135（97.78%）、行 2001/2091（95.70%）。全局仍有 60 个 Rust 函数未覆盖，100% 目标尚未完成，本任务及索引保持 in-progress；其他 crate 与 DSL 的不可达/剩余路径留待继续按行为审查。

进一步复核 `open_saved_stl` 与仓储租约的 LLVM 明细：准入失败处理已由 worker 启动失败行为测试覆盖；仓储租约有两条可由真实磁盘状态触发、此前未直接断言的拒绝路径。新增目标文件在创建租约前已出现、以及已打开工程清单被外部破坏的测试，分别断言 `AlreadyExists` 与 `ManifestInvalid`。另将 worker panic 的终态映射提取为具名函数并断言失败码、分类、详情和 attempt，执行器 panic 回调行为由既有执行器测试覆盖。定向仓储测试 6/6、panic 映射测试、panta-core Clippy、格式与 diff 检查通过；完整 `cargo coverage` 通过，全局函数 774/830（93.25%）、行 8812/9230（95.47%），panta-core 函数 362/397（91.18%）、行 3534/3670（96.29%）。`cargo test --locked --workspace` 通过，native / QML CTest 72/72。仍有 56 个全局函数计数未覆盖，继续逐项区分真实行为缺口与测试构建实例/不可触发的诊断路径。

## 2026-10-02 优先级跟进：Rust 报告与门禁可审阅性

本批先为现有 Rust coverage 命令保存 LLVM JSON、逐 crate 函数/行汇总及提交/工具元数据，CI 在成功或门禁失败时均上传报告。保持既有三个 crate 排除和全局函数 89% / 行 92% 下限；新增报告校验与受控空数据/损坏输入/阈值失败回归，不将逐 crate 报告冒充逐模块门禁或防下降。原始统计口径仍包含内联测试，C++ 门禁、100% 目标和基线比较留待后续批次。

本批验证（macOS arm64，rustc 1.98.1 / cargo-llvm-cov 0.9.1）：

- `cargo coverage`：通过，函数 629/685（91.82%）、行 6907/7343（94.06%）；三个报告文件均已生成。初次运行发现工具版本命令需要 `llvm-cov` 子命令、当前 LLVM JSON 为 3.1.0，修正后重新实跑通过。
- `cargo test --locked --workspace`：通过，包含 qmllint 与 CTest 69/69；报告校验三项单测通过。追加精度边界测试后，定向 runner 单测再次通过（性能 CLI 三项按既有约定忽略）。
- `cargo format --check`、`cargo lint --check` 与 `git diff HEAD --check`：通过；Clippy、machete、CMake、qmllint、clang-tidy、include-cleaner、Cppcheck 均完成。依赖键保持 serde_json / pest_derive 原名，无版本变更。
- 新配置的 CI artifact 上传当时尚未取得当前提交证据；逐 crate 数字仅用于审阅，任务仍为 `in-progress`。随后 [run 37010051058](https://github.com/Yuki-Nagori/panta/actions/runs/37010051058)（`966b358`）成功上传 `rust-coverage` 与 `native-coverage`。那次上传只证明报告路径，不提高 89% / 92% 门槛，也不等于 100% 或分模块门禁。

## 2026-10-02 崩溃记录覆盖

`panta-foundation` 的崩溃记录此前大量落在信号处理器里。子进程以 SIGSEGV 退出时 LLVM profile 来不及刷盘，所以处理器里的写日志和回溯在覆盖率里一直显示未执行。本批把记录从“重发信号”里拆出：`record_crash` 只写标识和回溯，`crash_handler` 仍在记录后恢复默认处置并重发信号。生产终止语义不变。

新增直接测试：无效 fd 的 `write_all` 提前返回、六个崩溃信号名、`record_crash(SIGBUS)` 的日志与回溯文本、不可用目录安装失败，以及空目录安装后恢复默认信号处置。重发信号的那几行仍只在会杀死进程的子进程里执行，不把未刷盘的 profile 算作覆盖。Windows 路径不在这次 macOS 统计里。

`cargo coverage`（macOS arm64，rustc 1.98.1 / cargo-llvm-cov 0.9.1）通过，未改 89% / 92% 下限：

- 全局：函数 657/726（90.50%）、行 7567/8101（93.41%）
- `panta-foundation`：函数 14/19（73.68%）、行 215/241（89.21%）

同一份旧报告（`d0870c2`，树已脏）里 foundation 是函数 6/13（46.15%）、行 83/171（48.54%）。两次分母不同，只说明记录路径已经进入覆盖，不拿全局百分比和那份旧报告比高低。`panta-ffi` 行覆盖 1713/1920（89.22%）仍低于全局行下限，目前由其他 crate 补上。C++ line/branch、逐模块防下降和函数 100% 仍未完成，任务保持 in-progress。

本批还跑了 `cargo test -p panta-foundation --locked`（4 passed，1 ignored）和 `cargo clippy -p panta-foundation --locked --all-targets -- -D warnings`。未跑完整 `cargo test --locked --workspace`。

## 2026-10-02 DSL 头部与重复字段

`panta-dsl-core` 的 `source_kind`、重复 catalog/src、Pest 语法诊断、`Diagnostics` 显示、超限源、theme 格式化和 vanished 状态此前没有直接断言。补上这些路径后，阶段门禁仍是函数 89% / 行 92%。

`cargo coverage` 通过：全局函数 664/732（90.71%）、行 7645/8155（93.75%）。`panta-dsl-core` 为函数 123/145（84.83%）、行 1922/2058（93.39%）。上一轮同一命令是全局 657/726（90.50%），dsl-core 116/139（83.45%）。分母包含新增测试函数。`panta-ffi` 行覆盖仍是 1713/1920（89.22%）。C++ line/branch、逐模块防下降和函数 100% 仍未完成。

`cargo test -p panta-dsl-core --locked` 通过（lib 13、集成与 fsm 用例见该次输出）。`cargo clippy -p panta-dsl-core --locked --all-targets -- -D warnings` 通过。未跑完整 workspace 测试。

## 2026-10-02 FFI 确认响应

无工程时，打开、STL 导入、分析序列、材料确认、Fill 与 Gate Location 的 CXX 响应包装都返回诊断，并且不填成功载荷。`ProjectDiagnostic` 在 detail 为空时只显示 code，否则显示 `code: detail`。崩溃日志安装拒绝把普通文件当作目录，且不注册信号处理器。

`cargo coverage` 通过：全局函数 677/736（91.98%）、行 7818/8237（94.91%）。`panta-ffi` 为函数 152/160（95.00%）、行 1886/2002（94.21%），行覆盖越过 92%。上一轮全局是 664/732（90.71%），ffi 行覆盖是 1713/1920（89.22%）。门槛仍是 89% / 92%。函数 100%、C++ line/branch 和逐模块防下降仍未完成。

`cargo test -p panta-ffi --locked --lib` 25 passed。`cargo clippy -p panta-ffi --locked --all-targets -- -D warnings` 通过。未跑完整 workspace 测试。

## 2026-10-02 FSM 头部与导入错误显示

FSM 缺 version、缺 kind、重复 initial、重复 transitions 节，以及 Pest 语法错误都会进入既有诊断。可取消读取的 `CheckedReadError` 能格式化成文本。STL 读失败映射为 `ImportParseFailed`。

`cargo coverage` 通过：全局函数 681/737（92.40%）、行 7855/8245（95.27%）。上一轮是 677/736（91.98%）和 7818/8237（94.91%）。`panta-dsl-core` 为函数 125/145（86.21%），`panta-import` 为函数 36/40（90.00%）。门槛仍是 89% / 92%。函数 100%、C++ line/branch 和逐模块防下降仍未完成。

定向测试：`cargo test -p panta-dsl-core --locked --test fsm -- rejects_missing_headers`、`cargo test -p panta-import --locked --lib -- checked_read_supports_cancellation`、`cargo test -p panta-core --locked --lib -- read_failures_become_parse_failures` 均通过。三个 crate 的 `cargo clippy --locked --all-targets -- -D warnings` 通过。未跑完整 workspace 测试。

## 2026-10-10 项目校验与缓存失败路径

补充项目清单中分析序列与材料引用缺失/未知 ID 的拒绝行为；导入资产目标碰撞时，断言既有文件、工程清单、revision、导入记录和当前网格均保持不变；网格缓存未驻留时激活失败并保留当前活动网格。路径服务测试现在同时覆盖未配置 root 的诊断、注入后的 getter 和成功解析。

验证（macOS arm64）：定向项目与路径测试通过；`cargo format --check`、`cargo clippy --locked -p panta-core --all-targets -- -D warnings`、`git diff --check` 通过。`cargo coverage` 通过，全局函数 778/832（93.51%）、行 8840/9245（95.62%）；`panta-core` 函数 366/399（91.73%）、行 3562/3685（96.66%）。`cargo test --locked --workspace` 通过，native / QML CTest 72/72。100% 函数目标和 C++ line/branch 门禁尚未完成，任务保持 in-progress。

补充验证 FFI 的应用退出转发：子进程先接受材料写入、销毁 service，再调用 FFI 收尾包装，随后重开工程确认 revision 与材料确已持久化。用子进程隔离会关闭进程级写入执行器的 API，避免污染并行单测；并将临时目录测试夹具提升为 crate 共享测试模块，消除重复加载。`cargo test --locked -p panta-ffi --lib` 28 项通过，FFI Clippy 与格式检查通过。复跑 `cargo coverage`：全局函数 780/833（93.64%）、行 8882/9286（95.65%）；`panta-ffi` 函数 156/164（95.12%）、行 1986/2100（94.57%）。`cargo test --locked --workspace` 通过，native / QML CTest 72/72。剩余未覆盖计数仍需逐项审阅，100% 函数和 C++ 门禁未达成。
