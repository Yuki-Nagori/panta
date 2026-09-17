# 035 — `.pa` 格式化器与格式校验器选型

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[034](034-rust-panta-artifact-parser.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

让 `.pa` 保持 YAML 风格的可读性，同时避免 YAML 的隐式类型和复杂扩展。调查能否直接复用 `yaml-format` 或同类工具；若不能无损支持 `.pa`，实现基于 034 Rust AST 的小型确定性 formatter，并提供配套 `check`/`format --check` 校验入口。034 已提供 AST、诊断和唯一 parser，本任务在此基础上实现 formatter 并接入 CLI。

## 必读

- [`.pa` 规则](../standards/pa.md)
- [DSL 解析与 Artifact 引擎](../modules/dsl-engine-and-toolchain.md)
- [变量 DSL](../modules/variable-dsl.md)
- [国际化](../modules/internationalization.md)
- [Rust 规范](../standards/rust.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

包含 formatter 方案调研、YAML 工具兼容性矩阵、规范化输出、UTF-8/缩进/注释/无引号标量校验、重复键和域 schema 诊断、CLI `check` 与 `format --check`；不包含完整 YAML 兼容、IDE 插件、QML 编辑器或 QM 运行时。

## 前置条件与待决策

034 提供稳定 AST、源码 span 和错误 DTO。调查必须使用固定 `.pa` fixtures 覆盖 language/theme/variables、Unicode、URL、冒号、百分号、转义、注释和非法 YAML 特性；比较 `yaml-format` 的引号、排序、注释、行尾和隐式类型行为。选型结论以可重复命令和输出 diff 为证据。

## 实施步骤

1. 冻结 `.pa` canonical format 与 `pa.md` 规则，建立正反夹具。
2. 在隔离实验中评估 `yaml-format`/同类工具；若无损条件不成立，删除实验依赖，选择 AST formatter。
3. 实现确定性格式化、只读 `check`、`format --check` 退出码和原子写回；复用 034 parser/schema，不重复解析。
4. 将 formatter 接入 Cargo/CMake/CI 入口，记录 lint、失败路径和覆盖率证据。

## 预计改动

更新 `crates/panta-dsl-core/` 与 `crates/panta-dslc/` 的 formatter/validator 模块、fixtures、Cargo.lock（仅保留最终选型）和统一质量入口；更新 `.pa` 规范与任务文档。不得把格式化后的临时 TS/QM 或实验输出提交到仓库。

## 清理与兼容例外

删除未采用的 YAML formatter 依赖、实验代码和重复校验入口；默认无兼容例外。若未来需要完整 YAML 互操作，另立任务，不在本任务暗中扩大语法。

## 验收标准

- [x] 调研报告给出 `yaml-format`/同类工具与自研 formatter 的可重复对比，明确最终选择及原因。
- [x] `check` 能定位 UTF-8、Tab、缩进、未知 kind、重复键、非法值、占位符和域 schema 错误；失败不写文件。
- [x] `format` 输出 LF、两格缩进、无引号标量和单一末尾换行；重复运行幂等，语义、值、注释和变量声明顺序不变。
- [x] `format --check` 在 canonical 文件返回 0，在需要格式化的文件返回非零且不修改源文件；写回采用临时文件替换并同步临时文件。
- [x] language/theme/variables fixtures 和 Unicode/URL/冒号/百分号/转义覆盖，CLI 任意 cwd 可用。
- [ ] Cargo fmt/test/clippy 与 032 覆盖率门禁通过；022/025/030 只依赖共享 formatter/validator，不保留旁路实现。

## 验证计划与结果

记录 workspace 根目录的 `cargo test --locked`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、formatter 幂等/失败夹具、工具输出 diff 和任意 cwd CLI 运行。当前未执行。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-16 | 创建 `.pa` formatter/validator 任务并冻结 YAML 风格规则 | 有明确的复用工具评估和自研回退边界 | 仅文档完成，调研与实现未执行 |
| 2026-09-17 | `cargo search yaml-format --limit 5`；`cargo test -p panta-dsl-core`；CLI `format`/`format --check` 任意临时 cwd | 选型、格式化幂等、Tab/占位符校验和原子写回通过 | crates.io 未返回 `yaml-format` formatter；共享 AST formatter 已实现；9 个核心测试、2 个 fixture 测试、临时文件格式化与只读检查通过 |

## 风险与回退

通用 YAML formatter 可能自动加引号、改变 `true/1` 语义或重排注释。先在临时输出上比较 AST 与字节结果；不满足约束时移除依赖并回退到共享 AST formatter，原文件不被修改。

## 决策与工作记录

- 2026-09-16：创建任务；`.pa` 采用 YAML 风格的层级与冒号，但不承诺通用 YAML 兼容。
- 2026-09-16：formatter 与 validator 单独编排为 035，复用 034 AST，避免第三套 parser。
- 2026-09-17（选型）：不引入通用 `yaml-format`。`.pa` 具有 `src/tr`、`st`、`numerus`、`type = expression` 等专用节点，且要求禁止 YAML 隐式类型、锚点和 flow collection；直接格式化 YAML 会丢失这些语义。采用共享 AST 的小型 formatter，输出两格缩进、LF 和唯一末尾换行；`check` 继续使用 034 parser 作为唯一校验入口。

## 完成摘要

当前 formatter 保留独立 // 注释并将其置于规范输出的头部之后；条目 comment/extra 字段仍保留在对应消息上。剩余 032 统一质量门禁与 022/025/030 消费方接入完成后再标记 done。

CLI 读取源文件时先校验 UTF-8，非法字节返回稳定的 pa.invalid_utf8 诊断，不进入 parser 或写回流程。
