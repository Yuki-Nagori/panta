# 034 — Rust Panta Artifact 解析与 TS/QM 编译入口

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[001](001-cargo-config.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

为变量 DSL、主题和国际化建立一个共享的 Rust `.pa` Artifact 解析/聚合内核及单文件 CLI。`.pa` 是人直接编写的简洁文本；国际化由 CLI 输出构建树临时 TS，CMake 使用锁定的 QtTools 预编译 `lrelease` 生成 QM。当前已开始实现解析内核与 TS 生成入口。

## 必读

- [模块设计](../modules/dsl-engine-and-toolchain.md)
- [变量 DSL](../modules/variable-dsl.md)
- [国际化](../modules/internationalization.md)
- [`.pa` 规则](../standards/pa.md)
- [Rust 规范](../standards/rust.md)
- [CMake 规范](../standards/cmake.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

包含公共 `.pa` grammar、pest parser、按 `kind` 聚合的 Artifact 模型、quick-xml TS 生成/校验、serde DTO、稳定诊断、资源上限、单文件 CLI 和 CMake 调用契约；不包含 Qt/QML runtime translator、主题业务 schema、完整变量求值、脚本执行、网络下载或本地编译 Qt。

## 前置条件与待决策

Rust workspace 入口 001 可用；实施前冻结 `.pa` 的 UTF-8、language/sourcelanguage 头部、`[Context]` 区块、短 ID、`src/tr` 字段、状态/复数元数据、YAML 风格缩进、无引号标量、转义规则和 TS 支持的 Qt TS 版本。首选 pest；若基准证明配置文件解析无法满足性能目标，才评估 nom。`quick-xml`、`serde` 和其他 crate 的版本、许可证与 MSRV 必须进入 Cargo.lock 并按依赖规范审核。

## 实施步骤

1. 建立 `panta-dsl-core` library、公共 AST/诊断 DTO、pest grammar 和限制常量，覆盖变量/主题/language 的 kind 分派。
2. 实现 Artifact 聚合和 quick-xml TS 生成/校验；输出规范化临时 TS，不修改 `.pa` 源文件。
3. 实现 `panta-dslc` CLI（validate、emit-ts 等最小子命令）及 Cargo/CMake 调用入口，记录 QtTools 预编译 `lrelease` 路径；格式化由 035 接入。
4. 添加语法往返、失败、上限、任意 cwd 和稳定输出测试，完成 022/025/030 的消费契约。

## 预计改动

新增 `crates/panta-dsl-core/`、`crates/panta-dslc/` 及 fixtures；更新 workspace `Cargo.toml`/`Cargo.lock`、CMake 构建入口和 `resources/i18n/panta-*.pa` 示例字典。具体模块/API 以实现时实际归属为准，不把生成的 TS/QM 或构建树放入仓库。

## 清理与兼容例外

删除任务 022/025 各自实现的重复 lexer、TS 生成器或源校验路径；当前尚无这些实现，记录为无废弃项。默认无兼容例外。

## 验收标准

- [ ] 同一 `.pa` 输入在三平台得到相同规范化 Artifact 快照；未知必需版本、重复 context/ID、非法 locale、语法错误和资源上限均拒绝且不覆盖旧产物。
- [x] pest grammar 提供准确 source span；kind variables/theme/language 分派明确，未知域与重复字段返回稳定诊断。
- [x] quick-xml 根据语言快照生成受支持的 Qt TS XML，context/source、自动生成的内部 id、占位符和 locale 基线不丢失。
- [x] CLI 为单文件可执行物，任意 cwd 可运行且不访问网络/系统 Qt；输出临时 TS 后由锁定的预编译 `lrelease` 生成 QM。
- [ ] Cargo 测试、fmt、clippy 与关键失败夹具通过；自有可执行代码覆盖率按 032 达到 line/branch 100%。
- [ ] 022、025、030 的文档和构建入口只依赖此共享核心，不保留第二套 parser 或 TS XML 生成实现。

## 验证计划与结果

在 workspace 根目录记录 `cargo test --locked`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、CLI 任意 cwd/失败夹具，以及 macOS/Linux/Windows 的确定性摘要。Qt QM 冒烟使用任务 022 锁定的 QtTools 预编译包。当前未执行。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-16 | 完成 Rust parser/CLI 架构与 `.pa` 简洁语法设计 | 明确 pest、quick-xml、serde、TS/QM 和 CMake 边界 | 设计已提交；实现与构建验证进行中 |
| 2026-09-17 | `cargo fmt --all`；`cargo test -p panta-dsl-core`；`cargo run -p panta-dslc -- check/emit-ts ...` | parser/TS 生成、kebab-case 约束、最长 source 规则和 CLI crate 通过 | 5 个核心单元测试、2 个 fixture 集成测试通过；CLI 成功校验 fixture 并生成 `zh_CN` TS。workspace 级 launcher/Qt 验证仍待预编译 Qt 供给可用后执行 |
| 2026-09-17 | QM 冒烟：`panta-dslc emit-ts resources/i18n/panta-cn.pa → TS`；任务 005 staging 的锁定 `lrelease 6.11.2`（macOS arm64）编译 QM | TS 被 lrelease 接受并产出 QM | 通过：TS 2.1/zh_CN/id/source/translation/%1 占位符完整；lrelease 报 4 翻译全部 finished，QM 245B。`-idbased` 在 Qt 6 已废弃（lrelease 直接拒绝），后续 CMake 接入使用默认参数 |
| 2026-09-17 | 任意 cwd（/tmp）执行 CLI：`check` cn/en、`format --check`、负例 tab 缩进/非法 UTF-8/未知 kind/重复 header | 校验通过返回 0；失败返回 2 且带稳定 span 诊断、不写文件 | 通过：`pa.tab_indentation at 3:5`、`pa.invalid_utf8 at byte 15`、未知 kind 的 pest span `1:7`、`pa.duplicate_header at 3:1`；失败路径均未修改文件 |

## 风险与回退

超大 `.pa` 或异常嵌套表达式可能消耗过多内存；读取前执行字节、token、深度和声明数量限制，失败保留上一份有效产物。QtTools 不可用时构建明确失败并给出预编译供给诊断，不回退系统 `lrelease` 或源码构建。

## 决策与工作记录

- 2026-09-16：创建任务；确认这是应用平台的配置/资源引擎工作，Rust library + 单文件 CLI 是唯一解析实现。
- 2026-09-16：首选 pest；quick-xml 将语言 Artifact 生成 Qt TS XML，serde 负责 DTO，QM 保持 Qt 运行期格式；`.pa` 源码不暴露 XML。
- 2026-09-17：根据 TS 兼容示例，语言 `.pa` 改为 `[Context]` + 短 ID + `src/tr` 元数据结构；支持 `st`、`oldsrc`、`comment`、`extra`、`numerus` 和 plural forms，`cn` 简写归一化为 `zh-CN`。
- 2026-09-17：开始实现 `panta-dsl-core` 与 `panta-dslc`，首期采用 YAML 风格缩进和无引号标量；变量/Theme 的类型表达式保留 `=`。

## 完成摘要

未完成，等待实现与跨平台验证。

## 当前进展（2026-09-17）

核心 parser、TS 生成入口和 formatter 消费契约已实现；实际 language 源文件已加入 resources/i18n/panta-en.pa 与 resources/i18n/panta-cn.pa。QM 冒烟已用任务 005 staging 的锁定 lrelease 打通（emit-ts → lrelease → QM），CLI 任意 cwd 与失败夹具验证完成。剩余：TS/QM 的 CMake 构建入口接入、三平台确定性快照、032 覆盖率门禁与 022/025/030 消费方接入。
