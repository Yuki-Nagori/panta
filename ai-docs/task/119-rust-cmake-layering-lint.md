# 119 — Rust 与 CMake 分层依赖 lint

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-20

## 目标与背景

对应 GitHub issue #36 的自动边界检查部分。将任务 118 定义并核对过的 Rust crate 与 native CMake target 允许依赖表转成可重复执行的 lint，避免依赖方向回退。检查真实 Cargo metadata 和 CMake target link graph，不用易漏项的人工 grep 代替。

## 必读

- [分层标准](../standards/layering.md)
- [质量工具链](../modules/quality-tooling.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 读取 workspace Cargo metadata，拒绝未登记的仓库内 crate 依赖边。
- 在 CMake 配置/生成图中检查 native target 的直接项目内链接边，包括 `LINK_LIBRARIES`、`INTERFACE_LINK_LIBRARIES`、`INTERFACE_LINK_LIBRARIES_DIRECT` 及可解析的生成表达式/alias；第三方 imported targets 按其分类规则处理。
- 纳入质量命令与 CI，提供合法/非法 fixture 和清楚的来源、目标、允许边诊断。
- 格式与普通 lint 使用任务 120 的变更路径选择器：Rust 格式只运行直接变更包，Clippy 覆盖 L1 与 L2 受影响包；C++/CMake/QML 格式只处理变更文件。clang-tidy 对源文件变更按编译单元筛选，头文件、CMake 或工具配置变更扩大到完整 native 编译单元；未知路径和空 CI diff 触发完整质量检查。

分层图 lint 始终检查完整 Cargo/CMake 图；machete 与 cppcheck 保留工作区级扫描（后者依赖跨翻译单元信息）；qmllint 保留完整 QML target。非目标：重组 crate/target、自动修复依赖、在本任务实现 GUI/CLI 业务命令。

## 涉及分组与跨组依赖

- 涉及分组：Rust 基础、领域、工具链、FFI、native 适配器、bridge、UI、组合根及测试。
- 是否新增跨组依赖：否；只检查任务 118 的允许表。

## 验收标准

- [x] 合法现状图通过；人为加入 `panta-mesh → panta-core`、生成表达式前缀碰撞、隐藏/命名空间 alias 和 `INTERFACE_LINK_LIBRARIES_DIRECT` 等禁止 CMake target 边均失败。
- [x] 诊断标明依赖来源、目标和违反的规则；缺失或无法解析的项目 target 不被静默忽略。
- [x] `cargo lint` 单项与 CI 均执行该检查，且 lint 自身有行为测试。
- [x] `cargo format --changed` 与可安全缩小范围的 lint 只检查受影响文件/包；完整图及跨翻译单元检查保留完整扫描。
- [x] 完整适用验证、独立只读评审和 task-index 更新完成。

## 验证计划与结果

2026-10-20 复查修复：统一 native 静态检查的影响分类，纳入 QML CMake、公共 Rust 构建输入和 CXX 生成头；补齐 Objective-C++ 格式与 LLVM 检查，在构建前跳过明确无关的扫描。CI 增加 macOS 平台源码检查与独立质量工具缓存，选择器使用固定 Rust 工具链。本地适用验证、独立评审及本轮三平台 PR CI 均已通过；首次受信缓存保存/命中仍待 main 或 nightly 实际执行，不作为本 PR 通过的证据。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-20 | [PR #69 CI](https://github.com/Yuki-Nagori/panta/actions/runs/39386991774)，实现提交 `971c33a` | 三平台构建测试、所有适用质量检查与新增平台检查通过 | 15 个 job 全部成功；macOS 对 4 个 `.mm` TU 分别执行 clang-tidy/includes。cargo-deny/machete 精确缓存冷启动未命中，工具安装及检查成功；PR 未保存缓存，不声称已验证受信保存或后续命中。 |
| 2026-10-20 | macOS；`cargo verify-changed` 的格式、Clippy、machete、CMake、layering、qmllint、LLVM 阶段，修复后对两个受影响 TU 重跑 clang-tidy/includes，再执行 `cargo lint cppcheck --check` | 全部适用质量阶段通过 | 原聚合调用在 include-cleaner 报三处缺失 include 后终止；补齐直接依赖并完成受影响复验，其余 57 个 TU 不重复扫描。全量 LLVM 59 个 TU 已检查；Cppcheck、格式及其余阶段均通过，不将首次失败记录成聚合命令成功。 |
| 2026-10-20 | `cargo test --locked -p panta-tests --bin panta-tests`；`cargo test --locked -p panta-build database::tests` | 资源/生成头/构建输入、未知源码回退、删除源码及 `.mm` 选择正确 | runner 30 项通过、3 项性能入口专属检查按既有规则忽略；数据库 4 项通过。纯 Rust 领域与纯 QML 输入实跑确认 LLVM/Cppcheck 在构建前跳过。 |
| 2026-10-20 | `cargo lint clang-tidy --objcxx-only --check`、`cargo lint includes --objcxx-only --check` | 当前平台集合非空并完整分析 | 4 个 `.mm` TU 两项检查均通过；修复原 Cocoa 测试向非空事件参数传 `nil` 的问题，Qt WId 句柄转换精确例外登记在质量工具说明。 |
| 2026-10-20 | `actionlint`、缓存身份脚本正反例与 ShellCheck；`cargo test-scope all` | workflow 语法、缓存边界与测试通过 | 本地检查通过；选择器因本轮公共工具变更保守扩大为全部 Rust package，CTest 86/86 通过。独立 `gpt-6.1-sol` medium 分别评审工具/CI 和完整分支 diff，未发现未解决问题；评审未代替执行验证。 |
| 2026-10-16 | `cargo test --locked --package panta-tests test_scope::tests -- --nocapture` | 路径选择与完整回退分支通过 | 10 项通过；覆盖未知路径、公共配置、真实 `.clang-tidy` 子目录、Rust L1/L2 闭包。 |
| 2026-10-16 | `cargo format --changed --check` | 变更文件格式通过 | 通过；首次检查发现 CMake 文件格式差异，格式化后复验通过。 |
| 2026-10-16 | `cargo lint layering --check` | Cargo/CMake 实际依赖图与违规 fixtures 检查通过 | 通过；覆盖直接违规边、前缀碰撞、隐藏/命名空间 alias、`INTERFACE_LINK_LIBRARIES_DIRECT`、生成表达式拼接及未登记 `mesh_helper`；同时验证 Qt/CTest utility target 不误报，并完成 launcher 构建。 |
| 2026-10-16 | `cargo test --locked --package panta-tests` | 分层、选择器与聚合测试通过 | 通过：22 项单元测试、3 项 native 集成测试，CTest 76/76。 |
| 2026-10-16 | `cargo format --changed --check`、`git diff --check` | 变更格式与 diff 空白检查通过 | 均通过。 |
| 2026-10-16 | `cargo verify-changed` | 运行变更范围质量与 L1/L2 测试；未知/根配置变更完整回退 | 通过；完整质量检查与 L2 workspace 测试通过，CTest 76/76。 |
| 2026-10-16 | [PR #43 CI](https://github.com/Yuki-Nagori/panta/actions/runs/38809494521) | 运行受影响范围测试与质量检查 | 全部检查通过：macOS、Linux、Windows 构建和测试，格式、Clippy、CMake、layering、QML、C++ 静态检查及依赖审计均通过。 |

## 风险与回退

生成图可能包含条件 target、别名、生成表达式或第三方传递边；只对明确归属本仓库的直接边应用规则，并以 CMake 本身生成的数据判定。若检查误报，先修正分类/提取逻辑，不通过扩大 allowlist 消音。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；任务 118 与任务 120 已完成，开始实现实际 Cargo metadata 与配置后 CMake target graph 检查。
- 2026-10-16：加入 `cargo verify-changed` 本地聚合入口与 CI 变更范围；复核发现后修复 L1 测试遗漏、完整质量回退传递、工具配置路径漏检及 CMake 生成表达式 target/alias 解析，并补充回归。
- 2026-10-16：最终独立评审发现普通仓库 target 的登记检查缺口及 changed lint 重复构建；现已拒绝未登记的可链接仓库 target、略过不参与链接图的 CMake utility target，并复用一次 launcher 构建。`gpt-6.1-sol` medium 复核完整 diff 未发现剩余可操作问题。
- 2026-10-16：全部本地验收项、独立评审、索引更新与 PR CI 完成，任务验收通过。

## 完成摘要

本地验证、独立只读评审与 PR #43 CI 均通过；PR #43 已合并到 main。
