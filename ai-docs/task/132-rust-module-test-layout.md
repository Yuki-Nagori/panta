# 132 — Rust 模块与私有测试文件布局

- 状态：in-progress
- 阶段：Rust 架构整理
- 依赖：107（Rust 服务边界）、112（FFI 私有单元测试归属）
- 优先级：P2
- 负责人：Codex
- 创建 / 更新：2026-10-28 / 2026-10-28

## 目标与背景

统一新建与重构 Rust 模块的源文件布局，并将适合独立维护的大型私有单元测试从生产实现文件移出，同时保留其对子模块私有实现的访问。拆分以职责和评审边界为依据，不做机械的行数清理。

## 必读

- [Rust 编码与工具链](../standards/rust.md)
- [测试目录与入口规范](../standards/testing.md)
- [模块分层](../standards/layering.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [Rust Reference：模块源文件名](https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames)
- [Rust Book：拆分模块](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)

## 范围与非目标

- 更新 Rust 与测试标准，明确 `module.rs` + 同级 `module/` 的模块组织方式、`tests.rs` 的私有测试归属和现有 `mod.rs` 的渐进迁移规则。
- 将职责已独立且测试块显著影响源文件可读性的测试迁到相应的 `tests.rs`；crate 根测试迁到 `src/tests.rs`。
- 仅整理文件布局，不改生产行为、测试断言或测试选择器语义。
- 不迁移未被本次重构涉及的既有 `mod.rs`，不拆分小型或职责混杂的测试。

## 涉及分组与跨组依赖

涉及 Rust 领域 crate、FFI crate、质量工具 crate 与根测试工具。仅调整同 crate 内模块文件布局，不新增依赖边或公开 API。

## 实施步骤

1. 先更新 Rust 与测试规范，消除“测试必须内联”的旧规则。
2. 将筛选出的私有测试移至父模块注册的 `tests.rs`；根 crate 测试使用 `src/tests.rs`。
3. 运行受影响范围验证并检查差异，记录独立评审结果。

## 预计改动

`ai-docs/standards/rust.md`、`ai-docs/standards/testing.md`、本任务与索引，以及 Rust crate 和测试工具中经检查适合迁移的测试文件。

## 清理与兼容例外

删除迁移前的内联测试副本；无兼容例外。既有 `mod.rs` 不属于清理对象，除非其模块本身在本任务内确有职责重构。

## 验收标准

- [x] Rust 与测试规范对模块路径、私有测试可见性、crate 根测试和既有 `mod.rs` 的处理一致。
- [x] 15 组大型且职责清楚的私有测试已移到同级 `tests.rs`；crate 根测试分别位于 `src/tests.rs`，模块内注册仍保持私有访问。
- [x] 未机械拆分小型测试，未迁移无关的既有 `mod.rs`，未改断言或生产行为。
- [ ] 受影响格式、测试和 lint 通过；依赖图、路径分类与测试选择器仍能识别新文件。
- [x] 独立只读评审完成，任务记录与索引同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-28 | `cargo test-plan` | 新路径归入原 Rust package 和原 native 测试闭包 | 通过：L1 识别 6 个直接受影响包，L2 增加 `panta-cli`、`panta-launcher`；native labels 为 app、bridge、ffi、i18n、mesh、qml、qt-adapter；未触发全量回退 |
| 2026-10-28 | `cargo fmt --all`、`cargo verify-changed` | 变更格式、受影响 lint 与测试通过 | Rust 格式、Clippy、分层 lint 与受影响 Rust 测试通过；79 项 native/QML 测试中 78 项通过。`QtAdapter.UserSettingsStore` 单独复跑仍因无法写入沙箱外 `~/Library/Preferences/com.pantatests...plist` 失败，与 Rust 文件布局无关，留待 CI 验收 |
| 2026-10-28 | `git diff --check` | 无空白错误 | 通过 |
| 2026-10-28 | 独立只读评审 | 无路径、边界或遗漏问题 | 通过：复核 15 组测试模块的私有访问、迁移前后测试内容、选择器分类与规范一致性；确认无需增加路径规则 |

## 风险与回退

移动私有测试可能改变其模块路径，导致私有项解析或测试夹具相对路径变化。以编译和原受影响测试验证；若某组测试无法保持私有访问或形成清晰职责，则保留内联布局并在记录中说明。

## 决策与工作记录

- 2026-10-28：按 Rust 官方文档推荐的新模块路径约定采用渐进整理；`mod.rs` 仍受支持，既有模块不为风格统一而单独迁移。

## 完成摘要

未完成。
