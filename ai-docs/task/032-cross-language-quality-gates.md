# 032 — 跨语言质量工具链与 100% 覆盖率门禁

- 状态：in-progress
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)、[018](018-cross-platform-ci.md)、[019](019-gtest-native-testing.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

为仓库内每种实际使用的语言和构建描述配置统一的格式化、静态检查、测试、覆盖率、依赖/死代码检查，并在 CI 形成可复核的质量门禁。目标是对门禁统计范围内的自有可执行代码达到 100% 覆盖率；生成文件、第三方源码、仅声明性资源和明确排除的启动胶水必须有登记理由，不能用排除项掩盖未测业务逻辑。

## 必读

- [统一测试与质量入口](011-test-quality-entrypoints.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [Rust 规范](../standards/rust.md)、[C++ 规范](../standards/cpp.md)、[GTest 规范](../standards/gtest.md)、[QML 规范](../standards/qml.md)、[CMake 规范](../standards/cmake.md)
- [质量工具链模块](../modules/quality-tooling.md)

## 范围与非目标

范围：Rust/Cargo、C++/CMake/GTest、QML/Qt 工具、CMake 脚本，以及未来 Python tooling 进入仓库后的 pyproject 工具配置；为每种语言选择稳定、可离线复现或固定版本的 formatter、lint/static analysis、unit/integration test、coverage、dependency audit 和死代码/未使用文件检查。建立统一命令、报告格式、差异覆盖率与 CI 门禁。

非目标：不把所有工具无条件叠加，不检查第三方/生成树源码，不把图形硬件冒烟虚构成 100% 单元覆盖，不因 Python 任务 014 deferred 而现在引入 Python 运行时。Node.js 专用工具（如 ESLint/knip）只有仓库真正引入 Node/QML JavaScript tooling 后才启用；不为类比工具预建 package.json。

## 覆盖率定义与门禁

“100% 覆盖率”按语言选择可解释的指标执行：Rust 至少 line + branch，C++ 至少 line + branch，QML 至少可执行绑定/JavaScript 分支与关键状态场景；声明性 QML 布局、生成的 moc/rcc/qmlcache、第三方和 vendor 目录排除并登记。门禁同时要求测试发现数量无意外缺失、覆盖率报告可追溯到 commit，并保留平台图形检查的未覆盖说明。

新代码不得降低基线；历史代码接入时先补齐到 100%，不能用“全局平均 100%”掩盖单个文件/模块空洞。工具无法可靠测量某类 QML 行为时，建立等价的可观察 ViewModel/组件行为测试，并在 task 记录工具限制和替代断言。覆盖率达标不替代错误路径、线程、资源生命周期、ABI 和真实窗口验证。

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

移除重复或无人调用的质量脚本、旧命令和失效排除规则。工具版本切换在同一 commit 更新配置、锁文件、CI 和文档。无兼容例外；暂时无法达到 100% 时保持 planned/blocked 并列出具体缺口，不能降低门禁伪装通过。

## 验收标准

- [ ] 统一入口实际运行 Rust、C++、QML 和 CMake 的格式化、静态检查与测试；工具缺失、测试失败或报告生成失败返回非零。
- [ ] 门禁统计范围、生成/第三方排除清单和理由可审阅；纳入范围的自有可执行代码 line + branch 覆盖率均为 100%，新代码不得降低基线。
- [ ] Rust、C++、QML 的错误路径、状态/绑定、线程/所有权和关键生命周期有行为测试；真实窗口/图形冒烟作为独立检查并记录未覆盖。
- [ ] 依赖审计、未使用代码/资源检查可在真实目录运行；未引入 Node/Python 时不创建其生态配置，出现后可按任务扩展。
- [ ] macOS/Linux/Windows CI 产出可下载、含 commit/工具版本/排除规则的覆盖率与质量报告，平台差异有明确处理。
- [ ] 受控失败验证门禁真的阻断，空测试/意外跳过不能视作通过；质量入口不递归调用自身。
- [ ] 旧脚本、重复配置和未登记排除项已清理，task、规范、CI 和命令说明一致。

## 验证计划与结果

执行统一入口的全绿和受控失败场景，逐语言核对 formatter/lint/test/coverage/audit 输出；在三平台 CI 验证报告上传和 100% 门禁。当前未执行，等待 011 与实际源码规模稳定。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | 仅完成规划 | 未执行；当前仓库尚无统一质量入口与覆盖率配置 |
| 2026-09-18 | `cargo deny check`（macOS arm64，deny 0.20.2，deny.toml 新建） | 初跑三项失败并全部修复复跑通过：advisories——quick-xml 0.38.4 RustSec 告警升 0.41.0（panta-dsl-core/dslc 测试 11 项通过，TS 生成不变）；bans multiple-versions——syn 2(pest)/3(cxx) 锁定组合 skip 登记；bans wildcards——workspace 内部 path 依赖补 `version = "0.1.0"` 并统一 `.workspace = true` 引用。最终 `advisories ok, bans ok, licenses ok, sources ok` |
| 2026-09-18 | `cargo machete`（0.9.2） | 初跑报 launcher/panta-ffi 未使用——实为 build.rs `DEP_PANTA_FFI_INCLUDE` 环境变量消费（machete 无法识别），按官方 `[package.metadata.cargo-machete]` 登记豁免；复跑干净（exit 0） |
| 2026-09-18 | CTest `Qml.FormatCheck`（qmlformat 6.11.2，Qt 供给） | 全部 qml/ 文件初检合规；受控失败两分支验证：追加合法但格式差文件 → 失败并列出文件名；追加非法语法 → 解析失败路径同样阻断；恢复后通过。空目录防呆（无 QML 即 FATAL） |
| 2026-09-18 | `cargo llvm-cov --locked --workspace --exclude panta-launcher --summary-only`（0.9.1，LLVM_PROFDATA/LLVM_COV 指向 Apple CLT） | Rust line 基线 74.21%（path 87.15 / task 92.16 / dsl-core 66.94 / dslc 0.00 / ffi 83.96）；Branches 无数据（stable rustc 限制，已记录）。llvm-cov 驱动的 native 重编在 launcher build.rs 失败 → 印证 launcher 排除理由（启动胶水） |
| 2026-09-18 | `ci.yml` 新增 `quality`（deny+machete，ubuntu 单平台）与 `coverage`（llvm-cov 报告，非门禁）job；YAML 解析通过 | 待推送后 CI 实证 |
| 2026-09-18 | `cargo test --locked --workspace --exclude panta-launcher`（8 组 ok）、`cargo build --locked`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings` | 全部通过 |

## 风险与回退

过度追求数字会导致无意义测试或大范围排除；按模块和分支指标审查，工具无法测量时补可观察行为测试。平台 coverage 工具差异由等价报告校验，暂不能统一时保持门禁缺口可见，不降低到“尽力而为”。回退只撤销本任务配置，保留已有测试和质量证据。

## 决策与工作记录

- 2026-09-16：新增跨语言质量任务；用户要求各语言配置 format/test/lint/依赖与死代码工具，并以 100% 覆盖率作为门禁目标。
- 2026-09-18（增量一，Rust 质量完备 + QML 格式门禁 + CI 接线；维护者指示 032 优先于 007）：工具版本固定入 `modules/quality-tooling.md`（cargo-deny 0.20.2 / cargo-machete 0.9.2 / cargo-llvm-cov 0.9.1 / qmlformat 6.11.2）。发现并修复三类真实问题：quick-xml 0.38.4 有 RustSec 告警（升 0.41，DSL 测试全过、TS 生成语义不变）；syn 2/3 双版本为 pest↔cxx 锁定组合的传递依赖（deny skip 登记）；内部 path 依赖无版本号触发 wildcard 拒绝（workspace 表补 version、成员统一 `.workspace = true`）。launcher 的 panta-ffi 依赖被 machete 误报（仅经 build.rs 的 DEP_* 环境变量消费），按官方机制登记豁免。QML 格式门禁落地为 CTest `Qml.FormatCheck`（qmlformat stdout diff，两分支受控失败均验证）。Rust 覆盖率基线：line 74.21%（path 87.15/task 92.16/dsl-core 66.94/dslc 0/ffi 83.96），launcher（启动胶水）按 032 允许条款登记排除；stable rustc 无分支覆盖数据，门禁先以 line 执行（工具限制已记录）；**100% 门禁在缺口清零前不启用**（dslc CLI 与 dsl-core 错误路径为下一增量补测目标）。CI 新增 `quality`（deny+machete，单平台）与 `coverage`（报告非门禁）两个 job。011 的 CTest/QML 聚合与其余静态分析（clang-tidy、clang-format 供给）留在增量二。

## 完成摘要

未完成，等待 011 统一入口和实际语言工具链接入。
