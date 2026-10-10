# 032 — 跨语言质量工具链与覆盖率门禁

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[018](018-cross-platform-ci.md)、[019](019-gtest-native-testing.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-10-10

## 目标

为仓库实际使用的语言和构建描述建立可复现的格式、静态检查、测试、覆盖率、依赖与死代码检查，并纳入 CI。目标是门禁统计范围内的自有可执行代码达到 100% 覆盖率；未测业务逻辑不能靠排除项隐藏。

## 范围

- 当前范围：Rust/Cargo、C++/CMake/GTest、QML/Qt 与 CMake 脚本。Python/Node 工具仅在仓库实际引入对应工具链后纳入，不预建空配置。
- Rust 当前阶段门槛为函数 89%、行 92%；最终目标为函数 100%。C++ line/branch 100%、逐模块防回退及差异覆盖率尚未落地。QML 以绑定与关键状态行为测试为准；stable Rust branch coverage 未采集。
- 当前 Rust 统计保留内联测试，排除 `panta-launcher`、`panta-tests`、`panta-build`；排除理由和口径记录在 [质量工具链](../modules/quality-tooling.md)。固定全局阈值不能替代逐模块门禁或父提交比较。
- 不把硬件冒烟当作单元覆盖，不检查第三方/生成源码，不因追求数字增加不稳定故障注入或无依据排除。

## 验收标准

- [ ] Cargo 统一入口运行 Rust、C++、QML 与 CMake 的适用格式、静态检查和测试；缺工具、失败测试或报告生成失败时返回非零。
- [ ] Rust 函数覆盖达到 100% 并保留行门槛；C++ line/branch 门禁与按模块防回退落地；未测指标清楚标记。
- [ ] Rust、C++、QML 的关键错误路径、状态/绑定、线程/所有权和生命周期有行为测试；真实窗口/图形冒烟单独记录。
- [ ] 依赖与未使用代码/资源检查在实际目录运行；工具缺失、空测试、意外跳过和受控阈值失败均不能误报通过。
- [ ] macOS/Linux/Windows CI 上传带 commit、工具版本和排除口径的覆盖率报告；跨平台差异有验证记录。
- [ ] 任务、规范、CI 和入口命令一致；无重复脚本、失效配置或未登记排除项。

## 当前实现与待办

- 统一质量入口及 Rust 89% / 92% gate 已运行；Rust 覆盖率报告包含 JSON、crate 汇总、元数据和 CI artifact。
- `cargo coverage` 生成函数/行完整 JSON、可搜索的未覆盖行文本及 HTML 源码视图；报告使用 gate 收集的同一批 profile。
- `cargo coverage native` 合并 CTest profile 并生成汇总、逐行执行计数文本和 HTML 报告；额外生成排除 `tests/` 的产品源码汇总与逐行报告，CI 上传全部报告 artifact。2026-10-10 macOS arm64 最新报告的行覆盖为 84.51%、分支 46.10%，该汇总包含测试源码；产品 native 源码报告为行 74.84%、分支 55.99%，仅作为本机诊断基线。LLVM 22 的 `llvm-cov report` 不提供 missing-lines 选项，因此 native 文本报告保留所有源码行计数。当前 native job 只生成 artifact，尚无覆盖率阈值门禁；跨平台稳定口径与基线仍待确认。
- 尚待完成：Rust 100% 函数目标、C++ line/branch 门禁、逐模块防回退/差异覆盖率、完整跨平台 artifact 验证及所有受控失败验收。不要仅凭当前 89% / 92% gate 判定任务完成。

## 验证与证据

| 日期 / 证据 | 结果 |
|---|---|
| 2026-09-18 至 21：质量入口、工具门禁、Miri | Rust/C++/QML/CMake 适用工具和聚合入口逐步接入；空测试、失败测试与阈值夹具曾验证非零退出。Miri 仅跑适用的纯 Rust crate；Pest 容量压力用例因解释执行成本在 Miri 忽略，常规测试仍覆盖。 |
| 2026-09-24：macOS gate、[CI run 36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191) | 修复后函数 402/445（90.34%）、行 4270/4545（93.95%）；三平台当前 89% / 92% gate 全绿，不代表 100% 或 C++ line/branch 已达成。 |
| 2026-10-02：coverage artifact、[CI run 37010051058](https://github.com/Yuki-Nagori/panta/actions/runs/37010051058) | `rust-coverage` 与 `native-coverage` 上传成功；证明 artifact 路径可用，不表示逐模块门禁已存在。 |
| 2026-10-10：Rust crate 行为覆盖跟进 | 对 core、DSL、import、FFI 等可确定的业务行为补测试；逐项审阅剩余 LLVM 缺口，区分测试实例/泛型计数、不可达防御路径与不安全故障注入。内置材料解析抽为纯函数并覆盖 malformed JSON 和不支持 schema；独立 `gpt-6.1-sol` medium review 通过。`panta-dsl-core`、`panta-import` 等 crate 函数覆盖 100%，全局尚未达到。 |
| 2026-10-10：coverage 报告优化（macOS arm64） | 首次 `cargo coverage`：函数 803/842（95.37%）、行 9296/9687（95.96%）；生成 crate 汇总、完整 JSON、`uncovered-lines.txt` 与 HTML。`cargo coverage native` 的 CTest 72/72 通过，生成 summary、逐行计数与 HTML；LLVM 22 `llvm-cov report` 不支持 missing-lines 选项，改用 `llvm-cov show`。 |
| 2026-10-10：材料与工程失败路径覆盖 | 材料解析新增 malformed JSON / 不支持 schema 测试；新增锁文件不可打开、任务状态锁中毒恢复测试。`cargo coverage` 函数 813/850（95.65%）、行 9357/9748（95.99%）；core 393/418（94.02%）。`cargo test --locked --workspace`（72/72 native / QML CTest）、`cargo lint --check`、`cargo fmt --all -- --check` 与 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 批准。审计剩余缺口后，多数零计数为重复实例、崩溃子进程和手动性能基准，尚需继续逐项审查可稳定触达的生产行为。 |
| 2026-10-10：native 报告与图标尺寸回退 | 增加有效图标请求尺寸为空时回退 SVG 固有尺寸的行为断言；`cargo coverage native` 72/72 CTest 通过。报告新增排除 `tests/` 的产品汇总和逐行文件，并由 CI 上传；产品 native 行/分支为 74.84% / 55.99%（macOS arm64 本机诊断）。`cargo lint --check`、`cargo fmt --all -- --check` 与 `git diff --check` 通过；两轮独立 `gpt-6.1-sol` medium review 均未发现问题。C++ 阈值门禁仍未启用。 |
| 2026-10-10：标准目录映射覆盖 | 新增 Qt 标准目录五种映射的行为测试；`cargo coverage native` 73/73 CTest 通过。产品 native 行/分支升至 75.10% / 56.45%（macOS arm64 本机诊断）；`cargo lint --check`、clang-format、`cargo fmt --all -- --check` 和 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 未发现问题。C++ 覆盖门禁尚未启用。 |
| 2026-10-10：工程根 Unicode 错误路径 | `PathHost::setProjectRoot` 新增不可往返 UTF-8 的错误行为断言；`cargo coverage native` 73/73 CTest 通过，`path_host.cpp` 行覆盖 69.03%→70.32%、分支 57.69%→59.62%，产品汇总行覆盖为 75.13%。`cargo format --check` 与 `git diff --check` 通过；`gpt-6.1-sol` medium 独立 review 未发现问题。 |

## 重要边界

- LLVM 函数计数会受测试构建实例、泛型实例化和内联测试闭包影响；不得从单个 HTML/show 页面推断生产代码完整覆盖。
- 崩溃重发信号、OOM、mutex poison 等不能可靠刷盘或不应通过改变生产行为触发；只有存在可观察且稳定的契约时才增加测试。
- 曾观察到 macOS 与 rustc 插桩工具口径不一致，按 CI 固定工具链复核；报告工具版本变化时需重验统计。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)、[质量工具链](../modules/quality-tooling.md)
- [验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)、[代码生命周期](../standards/code-lifecycle.md)
- [Rust](../standards/rust.md)、[C++](../standards/cpp.md)、[GTest](../standards/gtest.md)、[QML](../standards/qml.md)、[CMake](../standards/cmake.md) 规范
