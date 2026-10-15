# 120 — PR 按影响范围选择测试

- 状态：done
- 阶段：验证基础
- 依赖：[118 实际依赖图与应用命令层设计](118-layering-lint-and-command-boundaries.md)、[100 tests 整体整理与性能入口统一](100-tests-consolidation.md)
- 优先级：P0
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 GitHub issue #36 的 PR CI 影响范围部分。此前任一代码变更都会运行三平台 workspace 测试与覆盖率，反馈时间过长。本任务是 issue #36 的首个实现优先项：先把测试拆成可解释的层级，并按变更路径和依赖图选择 Rust package（含反向依赖）、CTest label 与 QML 检查；未知影响时保守扩大范围。PR 与 main push 都运行影响范围测试，完整测试与覆盖率由 nightly 承担。同分支同步修复 issue #40 中规范文档更新日期与来源说明未同步的问题。

本任务的 CI L1/L2 表示测试选择范围，不对应 [分层规则](../standards/layering.md) 中的 Rust L1–L5 或 Native L1–L3 架构编号：L1 是直接受影响的 Rust package，L2 是跨语言消费者闭包，无法安全缩小时升级到完整测试集。多个架构层可以共享同一执行范围。

## 必读

- [测试规范](../standards/testing.md)
- [质量工具链](../modules/quality-tooling.md)
- [任务 118](118-layering-lint-and-command-boundaries.md)
- [任务 100](100-tests-consolidation.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- 定义并实现 PR 测试层级：L1 为变更 crate/target 的快速局部验证；L2 运行依赖闭包中受影响的 Rust 反向依赖、native CTest 与 QML 行为测试；公共接口、构建图和无法分类的变更进入对应语言的完整测试矩阵。层级必须给出实际执行命令与选中结果，避免“分层”只成为标签。
- Rust 影响闭包不得止于 Cargo：沿 CXX/FFI staticlib 消费边继续扩展到 native targets、相关 CTest 和 QML module/行为测试。例如 `panta-mesh` 改动需考虑其 Rust 调用方以及 native mesh/CXX 消费者。
- Rust package 到 native 测试标签的生成代码、CXX/FFI 静态库及构建脚本边显式维护在 selector 映射和回归测试中；例如 `panta-ffi` 影响 native mesh，`panta-dsl-core` 影响构建生成的 i18n/Qt adapter 测试。
- native 影响闭包沿 CMake 项目内 link edge（含 PRIVATE）和 QML module dependency 递归扩展到消费者及其测试；仅凭 public header 或 CTest label 不能断定私有实现变更无 UI 影响。
- 变更路径转换为受影响 Rust package 及仓库内反向依赖，选择对应 package 测试；允许根据已验证的独立性拆开纯单元、native integration 与 GUI/QML 行为检查。
- 测试选择器由 `tests/src/test_scope.rs` 中的 Rust 实现提供；CI 与本地 `cargo test-scope l1|l2|all` 共用规则，避免 `.github` 脚本和本地执行范围分叉。
- 为 CTest 建立/使用稳定 labels，并映射 native 源码和测试变更；QML 变更运行 qmllint/QML 行为测试。依赖边、测试覆盖标签或变更分类无法可靠解析时，回退到受影响语言完整测试集；公共构建/协议边界不明时回退完整跨平台矩阵。
- PR 与 `main` push 无法分类、根配置/公共接口/依赖图变化时保守扩大范围；两者运行 L1/L2 与三平台构建及相关 lint。完整 workspace 测试、coverage 与全量 lint 由 nightly 执行，不作为常规 CI 门禁。

非目标：降低测试断言、覆盖率或 lint 门槛；跳过与变更相关的跨平台编译；性能基准纳入 PR 测试。

## 涉及分组与跨组依赖

- 涉及分组：CI、Rust workspace、native CTest、QML 测试。
- 是否新增跨组依赖：否；直接根据 Cargo metadata、CTest labels 和变更路径生成选择，不依赖任务 119 的 lint 实现。

## 验收标准

- [x] L1/L2/完整矩阵边界、实际命令和升级条件清楚；Rust、CMake、CXX/FFI 与 QML 依赖闭包都纳入消费者测试选择。
- [x] Rust crate 变更选择本 crate、所有受影响反向依赖及跨语言 native/QML 消费者；native 变更沿项目内 CMake/QML 消费边选择相关 target 和 CTest label；无法可靠追踪时回退对应语言完整测试集。
- [x] Rust 选择器测试覆盖单域、跨域、根配置、未知路径、重命名/删除和空 diff；未知/核心路径保守扩大。
- [x] Rust→native 的 CXX/FFI 与 build-dependency 映射具备回归测试，覆盖修改 Rust 输入后受影响的 native CTest labels。
- [x] 新增 native 模块、CTest label 或测试板块时，同步更新路径影响映射与 Rust 回归测试；未登记变更仍通过无过滤全量回退覆盖。
- [x] 本地通过 `cargo test-scope l1|l2|all [--base <ref>]` 使用与 CI 相同的选择规则；`cargo test-plan` 默认输出当前工作树的 L1/L2 选择结果，CI 显式使用路径输入模式。
- [x] PR 与 main push 的测试只运行所选范围；三平台 workspace check/build 与相关 lint 保留。nightly 运行完整 workspace 测试、Rust/native coverage 与全量 lint 并归档 coverage 报告；Miri/sanitizer/CPU performance 诊断不受影响。
- [x] CI 输出可读的路径分类和选择结果，结果可由提交与 workflow run 复核。
- [x] 完整适用验证、独立只读评审和 task-index 更新完成。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | `cargo format --check`、`cargo lint clippy --check`、选择器 9 项与 native 聚合测试、`cargo test --locked --workspace`、`actionlint`、`git diff --check`；直接输入 launcher build script、未知路径、空输入、README、FFI C++ 与 QML 路径运行 `cargo test-plan --ci`；实跑 `mesh\|ffi` CTest 标签 | 选择范围和触发域与输入一致；未知/空输入完整回退，纯文档跳过；完整聚合测试、lint、格式、workflow 语法与受影响 CTest 通过 | 选择器回归 9/9、native 聚合测试 3/3、完整 workspace 测试通过，CTest 76/76；Clippy、actionlint、格式与 diff 检查通过。输入探测确认 `crates/launcher/build.rs` 和未知/空输入均输出 `code/cargo_deps/rust/native/qml=true`、全部 Cargo 包及 CTest 标签；README 仅文档跳过；FFI C++ 命中 `panta-ffi`、launcher 与 native labels；QML 命中 qml/bridge labels。代表性 `mesh\|ffi` 实跑只构建相关目标并通过 16 个 CTest，用时 1.91 秒。PR #41 的三平台构建、L1/L2、native/QML 与 lint 全部通过（[run 38800066250](https://github.com/Yuki-Nagori/panta/actions/runs/38800066250)）。首次 Windows CI 暴露 JSON 行尾污染 Cargo package 名的问题，已改为 NUL 分隔并通过重跑。独立 `gpt-6.1-sol` medium 只读评审最终代码 diff，未发现剩余可操作问题。 |

## 风险与回退

影响范围误判会造成假绿。核心构建文件、Cargo/CMake 图、共用 FFI 和未分类路径必须触发完整回退；PR 与 main 的三平台构建及相关 lint 保留，nightly 执行固定的三平台完整测试、全量 lint 和覆盖率检查。任何路径解析失败都不得静默缩小范围。

## 决策与工作记录

- 2026-10-16：从 issue #36 拆分；按用户优先级提升为 P0，先解决 PR 测试耗时；按实际依赖/CTest label 设计 L1/L2 与完整矩阵回退。
- 2026-10-16：独立评审指出测试闭包还需覆盖 CMake、CXX/FFI 与 QML 消费边；已将跨语言影响传播及无法解析时的保守回退加入验收条件。
- 2026-10-16：task 118 文档 PR 合并后，在专用分支开始实施。
- 2026-10-16：按维护者意见将影响范围选择器从 `.github` Python 脚本迁入 `panta-tests` Rust 命令；CI 和本地通过同一规划逻辑，新增 `cargo test-plan` 与 `cargo test-scope` 入口。
- 2026-10-16：独立评审发现 Rust→native 映射漏掉 FFI→mesh 与 DSL build-dependency→i18n/Qt adapter，以及本地 native-only 命令缺少构建树准备；已补映射回归测试，并在执行选定 native 测试前构建 launcher。
- 2026-10-16：同一分支顺带处理 issue #40，更新分层规范来源与测试规范日期；并修复 QML lint 重新配置后 GTest `*_NOT_BUILT` 无标签导致受影响 CTest 被静默跳过的问题。
- 2026-10-16：按维护者确认将 Rust 函数/行覆盖率门槛调整为 85%/85%；随后确定完整 workspace 测试、coverage 与全量 lint 放入 nightly。PR 与 main push 都只运行 L1/L2 影响范围测试，三平台构建与相关 lint 保留。
- 2026-10-16：独立 `gpt-6.1-sol` medium 最终只读评审完整 diff；修复 i18n→QtAdapter 与测试 manifest 的漏选、macOS Bash 3.2 `mapfile` 不兼容、全量 CTest 标签遗漏和 Markdown 夹具触发问题后复核通过。评审未发现剩余可操作问题；真实三平台 Actions 尚未运行。
- 2026-10-16：严格复核 selector 最终快照，独立评审提出的 launcher build script 后置升级漏开依赖域已修复并以直接输入确认；选择器 9 项回归、完整 workspace 聚合测试（CTest 76/76）、actionlint、格式和 diff 检查均通过。实际三平台 Actions 仍是剩余验收项。
- 2026-10-16：Clippy 发现选择器与 placeholder 测试中的 `unwrap/expect`，已改为 `Result` 错误传播；Clippy、选择器回归、native 聚合测试和完整 workspace 聚合测试重新通过。
- 2026-10-16：PR #41 的三平台 Actions 全部通过。Windows L2 曾因 Git Bash/JQ 行尾将 CR 混入 Cargo 包名而失败；改为 NUL 分隔解析后重跑通过，且全量回退 native/QML 测试通过。维护者授权不等待最新文档提交触发的 CI，已将任务与索引标为 done。

## 完成摘要

实现、严格评审、本地验收和 PR 三平台 CI 已通过；文档同步已完成。PR #41 按维护者指示不等待最新文档提交触发的 CI，直接合并。
