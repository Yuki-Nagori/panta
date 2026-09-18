# 032 — 跨语言质量工具链与 100% 覆盖率门禁

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[018](018-cross-platform-ci.md)、[019](019-gtest-native-testing.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-18

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

全局固定阈值不能阻止所有下降，也不能保证单个模块达标；按模块/与父提交比较和报告制品仍待实施。排除需要证明和实际配置，不能把难触发系统故障、低风险或 bin 中可执行库实例称为不可达。内联测试代码与多二进制统计需要继续核对；不能根据单一 show 视图宣称生产代码全覆盖。

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
| 2026-09-16 | 仅完成规划 | 未执行；当前仓库尚无统一质量入口与覆盖率配置 |
| 2026-09-18 | `cargo deny check`（macOS arm64，deny 0.20.2，deny.toml 新建） | 初跑三项失败并全部修复复跑通过：advisories——quick-xml 0.38.4 RustSec 告警升 0.41.0（panta-dsl-core/dslc 测试 11 项通过，TS 生成不变）；bans multiple-versions——syn 2(pest)/3(cxx) 锁定组合 skip 登记；bans wildcards——workspace 内部 path 依赖补 `version = "0.1.0"` 并统一 `.workspace = true` 引用。最终 `advisories ok, bans ok, licenses ok, sources ok` |
| 2026-09-18 | `cargo machete`（0.9.2） | 初跑报 launcher/panta-ffi 未使用——实为 build.rs `DEP_PANTA_FFI_INCLUDE` 环境变量消费（machete 无法识别），按官方 `[package.metadata.cargo-machete]` 登记豁免；复跑干净（exit 0） |
| 2026-09-18 | CTest `Qml.FormatCheck`（qmlformat 6.11.2，Qt 供给） | 全部 qml/ 文件初检合规；受控失败两分支验证：追加合法但格式差文件 → 失败并列出文件名；追加非法语法 → 解析失败路径同样阻断；恢复后通过。空目录防呆（无 QML 即 FATAL） |
| 2026-09-18 | `cargo llvm-cov --locked --workspace --exclude panta-launcher --summary-only`（0.9.1，LLVM_PROFDATA/LLVM_COV 指向 Apple CLT） | Rust line 基线 74.21%（path 87.15 / task 92.16 / dsl-core 66.94 / dslc 0.00 / ffi 83.96）；Branches 无数据（stable rustc 限制，已记录）。llvm-cov 驱动的 native 重编在 launcher build.rs 失败 → 印证 launcher 排除理由（启动胶水） |
| 2026-09-18 | `ci.yml` 新增 `quality`（deny+machete，ubuntu 单平台）与 `coverage`（llvm-cov 报告，非门禁）job；YAML 解析通过 | 待推送后 CI 实证 |
| 2026-09-18 | `cargo test --locked --workspace --exclude panta-launcher`（8 组 ok）、`cargo build --locked`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings` | 全部通过 |
| 2026-09-18 | dslc CLI 覆盖缺口补齐（0%→88.01% line；测试 0→14 项）：main.rs 单测（check/validate/emit-ts/format 全命令、kind 拒绝、usage、缺参、缺文件、非法 UTF-8、原子写回成功/创建失败/目录 rename 失败、无扩展名路径）+ tests/cli.rs 集成测试（CARGO_BIN_EXE 真二进制，main() 行随子进程 profdata 并入覆盖，退出码 0/2 契约） | 两项模式决策实证：①负向断言用 `matches!(result, Err(ref e) if …)`——新版 clippy 的 `unwrap_used` 涵盖 `unwrap_err`，且 match+panic 分支是结构性永不可达的未覆盖行；②测试统一 `?` 传播（`Result<(), Box/E>`）——`unwrap_or_else(|e| panic!())` 错误闭包同样是不执行的未覆盖行。llvm-cov 工具口径矛盾已记录：summary 报 dslc 88.01%（宏展开区域计 0 的行）而 lcov/show 行数据无零计数行，门禁启用前须先固定权威口径 |
| 2026-09-18 | path.rs 覆盖缺口攻坚（28 行缺失起步）：`validate_relative` 弃用平台 `Path::components`，改为按逻辑引用格式 `/` 手工切分——Windows 盘符 Prefix 分支此前在 mac/Linux 永不可达（平台解析器差异），且反斜杠分隔语义随宿主漂移；现三平台同一套判定（尾随/连续分隔符容忍、首分隔符拒绝、盘符/保留名/尾点空格不变）。`resolve_write_target` 的防御性死分支以 `ancestors().find + unwrap_or(根)` 消除。新增全变体 `code()/detail()` 遍历测试、`root()` 访问器断言、分隔符容忍测试；负向断言转 `matches!`。补 `From<PathError> for String`。测试 20 项通过；clippy/fmt 干净 |
| 2026-09-18 | **关键发现（影响 100% 门禁的口径）**：llvm-cov summary 的分母包含 `#[cfg(test)]` 测试代码自身的错误闭包与 panic 分支——仓库旧测试普遍使用 `unwrap_or_else(panic!)`，这些从不执行的闭包全部计为"未覆盖行"（path.rs 攻坚后"覆盖率下降"即因新增测试的闭包）。结论：100% 门禁的真实工作量 = ①全仓库测试风格统一为 `?` 传播（stable rustc 无 `#[coverage(off)]`，E0658 实证）；②补真实逻辑缺口。`allow-unwrap-in-tests` 被 011 记录的决策排除。约定已写入本表供后续 crate 复用 |
| 2026-09-18 | task.rs 缺口补齐（8 项新测试，全套 29 项通过）：SubmitError Display 全变体、TaskEventKind label 全变体、`TaskManager::default()` 行为、日志环 256 容量淘汰（超量提交后 `recent_logs().len() == 256`）、cancel 对未知/终态任务拒绝、**迟到位事件拒绝**（经 `manager.inner` 直接驱动 `transition`/`publish_progress`：未知 id 与终态任务均不产生事件）、spawn 回滚（`rollback_spawn` 从 submit Err 臂提取为独立函数以便测试：移除记录+写日志+SpawnFailed）、`wait_for` 超时/命中两态；`io_other_error` clippy 修正 |
| 2026-09-18 | **本地覆盖率工具链不可靠（已证实）**：同一 profdata 下，Apple llvm-cov 17.x 的 summary/lcov/show 三种口径互相矛盾（task.rs run_task 明明执行却报 0 计数；show 无任何 0 行而 lcov 有几十条）——rustc 1.98 插桩格式与 Apple 17.x 解析存在版本错配。**权威口径定为 CI**（rustup `llvm-tools-preview` 与 rustc 严格配套，coverage job 已配置）；本地 lcov 数据仅作方向参考（dsl-core ~369 行缺口的量级可信，具体行清单以 CI 报告为准）。032 增量二的门禁启用以 CI 报告为唯一依据 |

| 2026-09-18 | 本轮 Rust 验证，cwd 仓库根，macOS arm64，rustc 1.98.1 / cargo-llvm-cov 0.9.1 + 同工具链 llvm-tools；`cargo test --locked --workspace --exclude panta-launcher` | 90 项测试通过；新增 DSL features 24 项。移除 CLI 重复实例测试后覆盖数字不变 |
| 2026-09-18 | `cargo llvm-cov --locked --workspace --exclude panta-launcher --summary-only --fail-under-functions 89 --fail-under-lines 92` | 通过：函数 89.69%（30/291 未进入），行 94.23%（163/2826 未覆盖）；仍有实际报告缺口，未宣称 100% |

| 2026-09-18 | 完整 `cargo test --locked --workspace`，macOS arm64 / rustc 1.98.1；CMake 4.3.3 / clang-format 20.1.0 / Qt 6.11.2，使用本地固定/缓存供给 | 101 项 Rust/聚合测试通过，含 CTest 28/28、qmllint、自有 native + CXX 格式。沙箱内初跑 Qt 测试配置目录不可写，扩大执行权限后全部通过；新增纳管的两个 CXX 源文件已格式化 |
| 2026-09-18 | `cargo test --locked --release --target-dir target/review-target -p panta-tests --test native`（复用第三方缓存） | Release 与自定义 target-dir 的聚合 2/2、CTest 28/28，通过；从 build.rs 导出当前构建树/配置，不再读取 Debug 旧产物 |
| 2026-09-18 | 同一覆盖率数据分别运行 `cargo llvm-cov report --summary-only --fail-under-functions 100` 与 `--fail-under-lines 100`；临时 CTest 夹具编译根 `tests/integration/native.rs` | 两个覆盖率受控失败均 exit 1；空套件与失败用例均使聚合测试 exit 101，成功夹具 exit 0 |
| 2026-09-18 | `cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、差异空白检查 | 通过。Linux/Windows 的本轮 CI 尚未运行；未将其写成本地已验证 |

## 风险与回退

过度追求数字会导致无意义测试或大范围排除；按模块和分支指标审查，工具无法测量时补可观察行为测试。平台 coverage 工具差异由等价报告校验，暂不能统一时保持门禁缺口可见，不降低到“尽力而为”。回退只撤销本任务配置，保留已有测试和质量证据。

## 决策与工作记录

- 2026-09-18（native 与门禁批次）：聚合测试与 formatter 供给、既有 C++ 格式归一一起交付；修复 profile/target-dir、Windows ctest.exe 与多配置 -C、空套件误报，串行执行 qmllint/CTest。formatter 按版本/摘要隔离缓存，复用也校验，下载完成后原子落位；格式扫描纳入自有 CXX 并传播目录读取失败。032 与 011 在此不可分割（工具供给与统一执行入口相互依赖）。清除硬编码 Debug、PATH ctest 回退和失实覆盖率结论；无兼容层。未完成项仍保留，后续统一 LLVM 供给时须同步工具决策。
- 2026-09-18：任务 043 将 Clang-Tidy、IWYU、Cppcheck、cargo-machete 和 cmake-lint 纳入根 `cargo lint`；IWYU 专注 include，Cppcheck 开启 unusedFunction，移除与两者重叠且 Python 3.14 不兼容的 Cppclean；cmake-format 由 uv 锁定并纳入 `cargo format`。

- 2026-09-18（本轮评审）：修正指标与 CI 矛盾、全局阈值防下降和无依据排除；补齐阶段门禁说明。native 聚合须修复 Windows 后缀、配置/target-dir、空套件、源码扫描吞错与 formatter 缓存身份；无兼容例外。
- 2026-09-18（Rust 测试批次）：新增 DSL 24 项公共 API 行为测试、FFI 取消/失败事件和路径类别/读写链验证；测试错误使用 Error + ? 传播。评审修复关闭线程测试吞掉 submit 失败，恢复 CLI 完整 usage 断言，删除只为覆盖率实例重复调用的 CLI 测试和冗余 dev-dependency。原始暂存中的重复/矛盾工作日志已收敛；未更改生产业务契约。

- 2026-09-16：新增跨语言质量任务；用户要求各语言配置 format/test/lint/依赖与死代码工具，并以 100% 覆盖率作为门禁目标。
- 2026-09-18（增量一，Rust 质量完备 + QML 格式门禁 + CI 接线；维护者指示 032 优先于 007）：工具版本固定入 `modules/quality-tooling.md`（cargo-deny 0.20.2 / cargo-machete 0.9.2 / cargo-llvm-cov 0.9.1 / qmlformat 6.11.2）。发现并修复三类真实问题：quick-xml 0.38.4 有 RustSec 告警（升 0.41，DSL 测试全过、TS 生成语义不变）；syn 2/3 双版本为 pest↔cxx 锁定组合的传递依赖（deny skip 登记）；内部 path 依赖无版本号触发 wildcard 拒绝（workspace 表补 version、成员统一 `.workspace = true`）。launcher 的 panta-ffi 依赖被 machete 误报（仅经 build.rs 的 DEP_* 环境变量消费），按官方机制登记豁免。QML 格式门禁落地为 CTest `Qml.FormatCheck`（qmlformat stdout diff，两分支受控失败均验证）。Rust 覆盖率基线：line 74.21%（path 87.15/task 92.16/dsl-core 66.94/dslc 0/ffi 83.96），launcher（启动胶水）按 032 允许条款登记排除；stable rustc 无分支覆盖数据，门禁先以 line 执行（工具限制已记录）；**100% 门禁在缺口清零前不启用**。CI 新增 `quality`（deny+machete，单平台）与 `coverage`（报告非门禁）两个 job。
- 2026-09-18（增量二补充，提交门禁）：维护者要求 commit 强制 fmt+lint。husky 依赖 Node.js/package.json，与"未引入 Node 时不创建其生态配置"规则冲突，经说明后采用原生 git hooks：`.githooks/pre-commit` 执行与 CI 同命令的 fmt --check 与 clippy -D warnings，失败即拒绝提交；`git config core.hooksPath .githooks` 一次性启用（git 不携带 hooks 配置，README 与 quality-tooling 已记录）。本机已启用并验证。
- 2026-09-18（增量二，覆盖率缺口清零与门禁启用）：按 llvm-cov 未覆盖行清单逐 crate 补测试（基线 631 行缺口：dsl-core 406/ffi 60/path 50/task 29/dslc 86）；全部清零后启用 line 覆盖率门禁（CI coverage job 转阻断）。分支覆盖继续受 stable rustc 工具限制记录在案。门禁范围：workspace 除 launcher（启动胶水，已登记）。

## 完成摘要

未完成，等待 011 统一入口和实际语言工具链接入。
