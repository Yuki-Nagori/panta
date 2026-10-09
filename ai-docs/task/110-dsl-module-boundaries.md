# 110 — DSL AST、解析、校验与输出模块整理

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[107](107-rust-service-boundaries-and-revisions.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-09

## 目标与范围

在 panta-dsl-core 内分离 AST / 诊断、Pest 解析、语义校验、格式化和 TS 输出。保持共享 grammar、FSM 模块、公共 API、CLI 与构建脚本输出；不创建第二套 parser 或无消费者的新 crate。

本任务对应 [106 架构评审](106-rust-architecture-review.md) 的后续建议；确认导入后台化仍由 063 实施。保持 UI 与正常工程 schema，不将新基础接口描述为已经实现。

## 必读

- [分层规则](../standards/layering.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] AST / 诊断 / 格式化 / TS / FSM 既有回归通过，格式及生成输出保持一致，CLI 和编译期消费者通过。
- [x] 替换实现与失效引用已删除，无新增兼容分支。
- [x] Cargo 工作区聚合、构建、格式及完整 lint 通过；记录具体平台与未覆盖点。

## 验证计划与结果

先补定向回归定位，再执行 `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check`。涉及性能容量或真实窗口时按实际范围单独验证。实施前保存既有公开 API 的格式化 / TS / 诊断输出基线，模块拆分后逐字节比较；既有公共及私有回归继续作为验证入口。

## 本次验证结果

2026-10-09，仓库根目录，macOS arm64 / Rust 1.98.1：

| 检查 | 实际结果 |
|---|---|
| 拆分前后 CLI 快照 | 10 组源码的退出码、stdout / stderr、规范源码和 TS 字节一致；含现有语言字典、FSM 及多种错误输入 |
| 新增稳定性回归 | 固定源码 / 格式 / TS 夹具与诊断顺序、文本、位置断言通过；保留 Rule 的根路径编译回归 |
| `cargo test --locked --workspace` | 最终复验通过，含 13 个 DSL 单元、3 个 catalog、25 个 features、21 个 FSM、CLI 及编译期消费者，native / QML 72/72 通过 |
| `cargo build --locked` | 通过；领域 FSM 和 i18n 编译期消费者正常构建 |
| `cargo format --check` / `cargo lint --check` | 格式与完整 8 阶段 lint 通过 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --locked -p panta-dsl-core --no-deps` | 通过；移动后的文档链接无警告 |
| 独立 subagent review | 公共 API、serde、失败诊断、输出与文档检查通过；唯一发现的 ast 内 FSM 链接已修正并复验 |

上表为原实施阶段的本地验证，当时未执行 Linux / Windows CI、Miri、性能基准或真实窗口验收；源码未修改 UI / 渲染路径，不以常规 QML 回归冒充窗口或性能证据。

## 工作记录

- 2026-10-09：CI 收尾完成：项目级 LF 提交 `011e129` 的 [CI 37910040544](https://github.com/Yuki-Nagori/panta/actions/runs/37910040544) 共 19 个作业全部通过，包括三平台 Cargo / sanitizer、Rust / native 覆盖率、Miri 和质量检查；Windows 严格输出夹具回归通过。根属性统一项目文本 LF，二进制自动识别保留内容，根属性变更触发全量检查。原局部修复 `d739969` 的 CI 37908774773 也已通过；最终验收以项目级规则提交为准，独立 subagent 对验收记录的复核通过。

- 2026-10-09：按用户要求将换行属性移到根目录并统一项目文本为 LF，二进制按自动识别保持原始内容；根 .gitattributes 会影响检出字节，不再按文档路径跳过质量作业，进入保守全量分类。现有索引 / 工作区无 CRLF 或混合换行，不需批量改写。隔离检出在 core.autocrlf=true 下逐字节核对 600 个既有文件；新增 CRLF 文本在索引 / 检出中均为 LF，NUL 二进制样例保留 CRLF 字节。实际 changed-paths 脚本验证根属性变更五类输出均 true、纯 Markdown 均 false。Cargo 工作区聚合（72/72 native / QML）、格式、actionlint、差异及独立 subagent review 通过；推送后等待最新提交 CI。

- 2026-10-09：CI 收尾：run37906982451 仅 Windows Cargo 聚合失败，task110 固定格式夹具检出为 CRLF，与格式化器的 LF 字节输出不一致；其余 18 个作业通过。补充仅限 tests/fixtures/dsl 的 Git 换行属性，保持生产输出和严格断言；隔离 Git 检出模拟 core.autocrlf=true：修复前三个夹具均为 CRLF，修复后字节等于 LF blob，目录外样例仍为 CRLF。本地 Cargo 工作区聚合（native / QML 72/72）、格式及差异检查通过，独立 subagent review 无可行动问题，确认属性文件仍触发 Windows CI。推送后等待实际 Windows 复验，不把隔离检出记为 Windows 测试通过。

- 2026-10-09：完成七个私有模块及根公共门面；保留类型、字段、serde 与函数 API（包括原先公开的 Pest Rule）。FSM 复用唯一 grammar / 语法诊断，删除根文件旧实现和重复错误转换。增加拆分前生成的输出夹具、公开入口及诊断顺序回归；整体 review 后修复文档链接，最终聚合、构建、格式、完整 lint、严格 rustdoc 与快照比较通过。任务及索引同步 done，无新增兼容分支。

- 2026-10-09：开始实施。当前 lib.rs 共 1606 行，AST、Pest 解析、语义校验与两类输出混合；计划拆为私有 ast / diagnostic / syntax / parser / validation / format / ts 模块，根模块保持公共 API 重导出，FSM 复用唯一 grammar 与诊断。保持诊断顺序、输出字节及现有容量限制，不改变 UI、语言语法和依赖。

- 2026-10-08：按用户要求一次登记剩余 Rust 架构任务，先提交规划后实施；避免把后续建议混入已完成的 107。

## 完成摘要

已完成 DSL 内部职责拆分：模型、诊断、唯一 grammar、节点转换、语义校验、规范格式与 Qt TS 各归其模块。公共门面、CLI、FSM / i18n 编译期消费者、输出和既有容量限制保持；固定回归与独立 review 验证通过。
