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
| 2026-09-18 | Rust 质量工具、QML formatter 与统一测试入口 | `cargo deny`、`cargo machete`、Rust fmt/clippy、QML 格式受控失败检查通过；聚合测试曾通过 101 项测试与 CTest 28/28。初始 Rust line 覆盖率为 74.21%，此历史基线不代表当前门禁。 |
| 2026-09-18 | 覆盖率口径、阈值与受控失败 | 本地 Apple LLVM 与 rustc 插桩口径不一致，覆盖率以 CI 配套 `llvm-tools-preview` 为准。空测试集、失败用例及 100% 覆盖率阈值夹具均能使聚合入口失败；现行 gate 为函数 89% / 行 92%，launcher 按任务约定排除。 |
| 2026-09-20–21 | Rust 跨平台行为测试与 `cargo ub-check` / Miri | crash 模块补齐后本地覆盖率达函数 89.18% / 行 93.84%；Miri 对纳入范围的纯 Rust crate 通过，已知进程/FFI/压力测试限制按任务说明排除或跳过。 |
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
