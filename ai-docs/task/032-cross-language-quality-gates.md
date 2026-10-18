# 032 — 跨语言测试质量门禁与行为契约

- 状态：deferred
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[018](018-cross-platform-ci.md)、[019](019-gtest-native-testing.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-10-19

## 目标

为仓库实际使用的语言和构建描述建立可复现的格式、静态检查、测试、覆盖率、依赖与死代码检查，并纳入 CI。测试首先描述公开行为、错误契约、不变量和可观察副作用；覆盖率用于发现测试盲区，不作为逐函数填满数字的交付目标。

## 范围

- 当前范围：Rust/Cargo、C++/CMake/GTest、QML/Qt 与 CMake 脚本。Python/Node 工具仅在仓库实际引入对应工具链后纳入，不预建空配置。
- Rust 保留函数 / 行各 85% 的全局下限；取消 100% 目标、逐文件阈值和差异覆盖率门禁。当前 Rust 统计保留内联测试，排除 `panta-launcher`、`panta-tests`、`panta-build`；排除理由和口径记录在 [质量工具链](../modules/quality-tooling.md)。
- C++ 与 QML 后续评估函数 / 行 85% 全局下限；先确认产品源码统计范围、工具支持和结果稳定性，不把现有 native 报告或 QML 行为测试写成源码覆盖率门禁。
- nightly 对 `panta-core`、`panta-dsl-core` 运行 `cargo-mutants` 并归档报告，仅用于诊断，不阻断常规 CI。
- 对 DSL parse/format 往返与路径规范化使用 `proptest` 验证公开行为和不变量。
- 全局下限只作质量护栏；新增或修改逻辑仍需评审其行为测试。Stable Rust branch coverage 未采集，作为未测指标记录。
- 不把硬件冒烟当作单元覆盖，不检查第三方/生成源码，不因追求数字增加不稳定故障注入或无依据排除。

## 验收标准

- [ ] Cargo 统一入口运行 Rust、C++、QML 与 CMake 的适用格式、静态检查和测试；缺工具、失败测试或报告生成失败时返回非零。
- [ ] Rust 仅保留函数 / 行 85% 全局下限；不设置 100% 目标、逐文件阈值或差异覆盖率门禁；C++ / QML 覆盖下限基于可靠统计口径决定。
- [ ] 测试以行为 / 契约命名；清理覆盖率导向的全变体遍历、非契约 Display 文案断言、视口内部状态探针和手拼 DSL 私有 key，保留公开 API 与可观察失败语义测试。
- [ ] `panta-core`、`panta-dsl-core` 的 nightly `cargo-mutants` 报告可生成并归档，且不阻断常规 CI。
- [ ] DSL parse/format 往返和路径规范化通过 `proptest` 检验公开不变量。
- [ ] Rust、C++、QML 的关键错误路径、状态/绑定、线程/所有权和生命周期有行为测试；真实窗口/图形冒烟单独记录。
- [ ] 依赖与未使用代码/资源检查在实际目录运行；工具缺失、空测试、意外跳过和受控阈值失败均不能误报通过。
- [ ] nightly 上传带 commit、工具版本和排除口径的 Rust/native 覆盖率报告；跨平台功能测试与差异有验证记录。
- [ ] 任务、规范、CI 和入口命令一致；无重复脚本、失效配置或未登记排除项。

## 当前实现与待办

- 统一质量入口及 Rust 89% / 92% gate 曾运行；Rust 覆盖率报告包含 JSON、crate 汇总、元数据和 CI artifact。2026-10-16 起 Rust 当前门槛为函数 / 行各 85%。
- `cargo coverage` 生成函数/行完整 JSON、可搜索的未覆盖行文本及 HTML 源码视图；报告使用 gate 收集的同一批 profile。
- `cargo coverage native` 合并 CTest profile 并生成汇总、逐行执行计数文本和 HTML 报告；额外生成排除 `tests/` 的产品源码汇总与逐行报告，nightly 上传全部报告 artifact。2026-10-10 macOS arm64 最新报告的行覆盖为 84.51%、分支 46.10%，该汇总包含测试源码；产品 native 源码报告为行 74.84%、分支 55.99%，仅作为本机诊断基线。LLVM 22 的 `llvm-cov report` 不提供 missing-lines 选项，因此 native 文本报告保留所有源码行计数。当前 native job 只生成 artifact，尚无覆盖率阈值门禁；跨平台稳定口径与基线仍待确认。
- 尚待完成：测试命名与行为断言整理、属性测试、nightly mutation 报告，以及 C++ / QML 覆盖统计口径评估。Rust 函数 / 行 85% 全局门槛已配置；本 task 整体保持 deferred，不代表其余规划已完成。
- 实施时逐项审计稳定可达的生产行为，只为有清晰契约的缺口补测试或小幅重构；若缺口只来自重复实例、不可稳定观察的崩溃路径或手动性能基准，记录证据并保留待办，不为覆盖率数字改变生产行为。
- CI 调度优化（2026-10-13 起）：PR 保留受影响范围测试、三平台构建检查和相关 lint。2026-10-16 起 PR 与 main push 都按 L1/L2 影响范围测试；完整 workspace 测试、Rust/native coverage 与全量 lint 移到每日 workflow，覆盖率报告上传 artifact；Miri、三平台 sanitizer 与 CPU 性能采样也由每日 workflow 执行。GPU 基准仍需可识别硬件/图形会话，暂不放入通用 runner。nightly 失败用于维护诊断，不替代 PR 必需检查。
- Miri 范围（2026-10-19）：沿用 2026-10-17 的 crate 与用例范围；Miri 下忽略 1 MiB 规范化输出边界，以及 128 层依赖 / 求值节点预算压力测试。普通测试仍覆盖这些资源上限，Miri 保留其余 DSL 语法、值语义、原子写入与失败保护用例。

## 验证与证据

| 日期 / 证据 | 结果 |
|---|---|
| 2026-10-19：Nightly Miri 运行 [run 39160415996](https://github.com/Yuki-Nagori/panta/actions/runs/39160415996) | 其他 nightly job 均通过；Miri 在执行 `normalized_source_over_limit_is_rejected_before_creating_file` 的 1 MiB DSL 输入时耗时约 42 分钟，随后达到 45 分钟 job 上限并被取消。普通测试继续覆盖该输入边界；Miri 仅运行满足 UB 检查价值的纯 Rust 行为用例。 |
| 2026-10-19：`cargo ub-check` | 本机完整 Miri 聚合入口通过；`panta-core`、`panta-dsl-core`、`panta-foundation` 及选定集成测试全部通过。纯资源预算压力样本按 Miri 条件忽略，普通 Cargo 测试仍覆盖边界。 |
| 2026-10-17：Nightly Miri 范围修正，[run 38960351402](https://github.com/Yuki-Nagori/panta/actions/runs/38960351402) | Miri 原本完整运行 `panta-core` 测试，在 `async_import_publishes_only_after_atomic_manifest_commit` 中进入 `zstd-safe` 的 `ZSTD_createCCtx`，该外部函数不受 Miri 支持。保留 `panta-core --lib`、单独的 `fsm_build`、`panta-dsl-core` 与 `panta-foundation`，并按测试名加入不访问 Mesh sidecar 的 `project` / `activation` 集成用例；调用 zstd 的四个 core 单元测试与构建夹具中的 Cargo 子进程用例仍仅在 Miri 下忽略。白名单中的 15 项直接 Miri 命令本机通过。`cargo ub-check` 本机未能启动，因为 rustup 无法在沙箱外创建工具链临时文件；需由 CI 验证聚合入口。 |
| 2026-10-13：GitHub Issue #11 规划更新 | [Issue #11](https://github.com/Yuki-Nagori/panta/issues/11) 的建议已并入本 task：Rust 全局函数 / 行 85%、取消 100% 与逐文件目标、行为测试清理、属性测试及 nightly mutation 报告。本次仅更新规划；状态保持 `deferred`。 |
| 2026-10-16：Rust 覆盖率门槛调整 | 按维护者要求将阻断门槛从函数 89% / 行 92% 改为各 85%；`032` 整体仍为 `deferred`，其余跨语言质量门禁未因此完成。 |
| 2026-10-13：CI 动态检查定时化 | `actionlint .github/workflows/ci.yml .github/workflows/nightly.yml` 与 `git diff --check` 通过。Miri、sanitizer 和 CPU 性能采样已拆到每日 workflow；实际 workflow/artifact 结果待 CI 验收，不把静态检查记为运行通过。 |
| 2026-09-18 至 21：质量入口、工具门禁、Miri | Rust/C++/QML/CMake 适用工具和聚合入口逐步接入；空测试、失败测试与阈值夹具曾验证非零退出。Miri 仅跑适用的纯 Rust crate；Pest 容量压力用例因解释执行成本在 Miri 忽略，常规测试仍覆盖。 |
| 2026-09-24：macOS gate、[CI run 36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | 修复后函数 402/445（90.34%）、行 4270/4545（93.95%）；三平台当时 89% / 92% gate 全绿，不代表 C++ line/branch 已达成。 |
| 2026-10-02：coverage artifact、[CI run 37010051058](https://github.com/Yuki-Nagori/panta/actions/runs/37010051058) | `rust-coverage` 与 `native-coverage` 上传成功；证明 artifact 路径可用，不表示逐模块门禁已存在。 |
| 2026-10-10：Rust crate 行为覆盖跟进 | 对 core、DSL、import、FFI 等可确定的业务行为补测试；逐项审阅剩余 LLVM 缺口，区分测试实例/泛型计数、不可达防御路径与不安全故障注入。内置材料解析抽为纯函数并覆盖 malformed JSON 和不支持 schema；独立 `gpt-6.1-sol` medium review 通过。`panta-dsl-core`、`panta-import` 等 crate 函数覆盖 100%，全局尚未达到。 |
| 2026-10-10：coverage 报告优化（macOS arm64） | 首次 `cargo coverage`：函数 803/842（95.37%）、行 9296/9687（95.96%）；生成 crate 汇总、完整 JSON、`uncovered-lines.txt` 与 HTML。`cargo coverage native` 的 CTest 72/72 通过，生成 summary、逐行计数与 HTML；LLVM 22 `llvm-cov report` 不支持 missing-lines 选项，改用 `llvm-cov show`。 |
| 2026-10-10：材料与工程失败路径覆盖 | 材料解析新增 malformed JSON / 不支持 schema 测试；新增锁文件不可打开、任务状态锁中毒恢复测试。`cargo coverage` 函数 813/850（95.65%）、行 9357/9748（95.99%）；core 393/418（94.02%）。`cargo test --locked --workspace`（72/72 native / QML CTest）、`cargo lint --check`、`cargo fmt --all -- --check` 与 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 批准。审计剩余缺口后，多数零计数为重复实例、崩溃子进程和手动性能基准，尚需继续逐项审查可稳定触达的生产行为。 |
| 2026-10-10：native 报告与图标尺寸回退 | 增加有效图标请求尺寸为空时回退 SVG 固有尺寸的行为断言；`cargo coverage native` 72/72 CTest 通过。报告新增排除 `tests/` 的产品汇总和逐行文件，并由 CI 上传；产品 native 行/分支为 74.84% / 55.99%（macOS arm64 本机诊断）。`cargo lint --check`、`cargo fmt --all -- --check` 与 `git diff --check` 通过；两轮独立 `gpt-6.1-sol` medium review 均未发现问题。C++ 阈值门禁仍未启用。 |
| 2026-10-10：标准目录映射覆盖 | 新增 Qt 标准目录五种映射的行为测试；`cargo coverage native` 73/73 CTest 通过。产品 native 行/分支升至 75.10% / 56.45%（macOS arm64 本机诊断）；`cargo lint --check`、clang-format、`cargo fmt --all -- --check` 和 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 未发现问题。C++ 覆盖门禁尚未启用。 |
| 2026-10-10：工程根 Unicode 错误路径 | `PathHost::setProjectRoot` 新增不可往返 UTF-8 的错误行为断言；`cargo coverage native` 73/73 CTest 通过，`path_host.cpp` 行覆盖 69.03%→70.32%、分支 57.69%→59.62%，产品汇总行覆盖为 75.13%。`cargo format --check` 与 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 未发现问题。 |
| 2026-10-13：崩溃日志文件打开失败 | 新增 Unix integration test：目标日志路径被目录占用时，公开安装 API 返回 `IsADirectory`，失败发生在安装处理器或修改全局状态前。`cargo test --locked -p panta-foundation` 与 `cargo coverage` 通过；Rust 汇总仍为函数 813/850（95.65%）、行 9357/9748（95.99%），integration test 不进入 crate 覆盖率分母。独立 review 未发现问题。 |
| 2026-10-13：工程网格激活成功桥接 | 新增完整成功场景，先释放导入时已驻留的 Mesh，再验证激活返回的 attempt/generation/import 标识、桥接坐标和缓存快照，确保成功结果实际提交进缓存。`cargo test --locked -p panta-ffi project_activation_success_commits_mesh_and_coordinates` 与 `cargo coverage` 通过；汇总为函数 814/851（95.65%）、行 9408/9794（96.06%）；`panta-ffi` 为函数 159/166（95.78%）、行 2067/2186（94.56%）。`gpt-6.1-sol` medium 独立 review 通过。 |
| 2026-10-13：QRC 路径桥接 | 增加 `qrc:/icons/panta.svg` 的 CXX 引用解析、枚举映射及逻辑路径往返断言；`cargo test --locked -p panta-ffi path_service_covers_existing_write_target_and_all_kinds` 与 `cargo coverage` 通过。汇总为函数 814/851（95.65%）、行 9413/9798（96.07%）；`panta-ffi` 为函数 159/166（95.78%）、行 2072/2190（94.61%）。`gpt-6.1-sol` medium 独立 review 未发现问题。 |
| 2026-10-13：DSL TS 文档种类拒绝 | 对可解析的 `variables` 文档调用 TS emitter，断言返回 `pa.ts_kind`，覆盖错误输入被拒绝的公开行为。定向 `panta-dsl-core` integration test、`cargo coverage`、`cargo fmt --all -- --check` 与 `git diff --check` 通过；汇总为函数 814/851（95.65%）、行 9421/9798（96.15%）；`panta-dsl-core` 行覆盖 2023/2117（95.56%），函数 132/132。`gpt-6.1-sol` medium 独立 review 未发现问题。 |
| 2026-10-13：DSL 手工 AST 消息顺序回退 | 验证 `format_document` 先按公开 AST 中的声明顺序输出，再按 map key 补齐遗漏项；重复与无效顺序项只会被忽略，各消息只输出一次。定向测试、全量 `cargo coverage`、格式与 diff 检查通过；汇总为函数 816/853（95.66%）、行 9475/9852（96.17%）；`panta-dsl-core` 函数 134/134、行 2079/2171（95.76%）。`gpt-6.1-sol` medium 独立 review 指出初版用例未区分声明顺序与 map 顺序；修订用例后复查通过。 |
| 2026-10-13：本地质量复验与 Clippy 修正 | `cargo test --locked --workspace` 通过，native/QML CTest 73/73；本批新增与调整的 crash handler、FFI activation、DSL formatter 定向测试及 `cargo fmt --all -- --check`、`git diff --check` 均通过。`cargo lint --check` 暴露测试中的 `expect`/不必要 clone，已修正；随后 Rust Clippy 阶段通过。因重复启动 lint 导致 native/QML 构建重跑并中断，未取得完整八阶段 lint 的最终结果，不记为通过。 |

## 重要边界

- LLVM 函数计数会受测试构建实例、泛型实例化和内联测试闭包影响；不得从单个 HTML/show 页面推断生产代码完整覆盖。
- 崩溃重发信号、OOM、mutex poison 等不能可靠刷盘或不应通过改变生产行为触发；只有存在可观察且稳定的契约时才增加测试。
- 曾观察到 macOS 与 rustc 插桩工具口径不一致，按 CI 固定工具链复核；报告工具版本变化时需重验统计。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)、[质量工具链](../modules/quality-tooling.md)
- [验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)
- [Rust](../standards/rust.md)、[C++](../standards/cpp.md)、[GTest](../standards/gtest.md)、[QML](../standards/qml.md)、[CMake](../standards/cmake.md) 规范
