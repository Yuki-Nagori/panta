# 034 — Rust Panta Artifact 解析与 TS/QM 编译入口

- 状态：done
- 阶段：应用平台扩展
- 依赖：[001](001-cargo-config.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-10-13

## 目标与背景

为变量 DSL、主题和国际化提供共享的 Rust `.pa` Artifact 解析/聚合内核及单文件 CLI。`.pa` 是人直接编写的简洁文本；国际化由共享核心生成构建树临时 TS，CMake 使用锁定的 QtTools 预编译 `lrelease` 生成 QM。核心、CLI 和 TS/QM 构建链已落地；本轮补齐多文档快照、批量 TS 发布和来源诊断。

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

包含公共 `.pa` grammar、pest parser、按 `kind` 聚合的 Artifact 模型、quick-xml TS 生成、serde DTO、带来源位置的稳定诊断、资源上限、单文件 CLI 和 CMake 调用契约；不包含 Qt/QML runtime translator、主题业务 schema、完整变量求值、脚本执行、网络下载或本地编译 Qt。

## 前置条件与待决策

Rust workspace 入口 001 可用；实施前冻结 `.pa` 的 UTF-8、language/sourcelanguage 头部、`[Context]` 区块、短 ID、`src/tr` 字段、状态/复数元数据、YAML 风格缩进、无引号标量、转义规则和 TS 支持的 Qt TS 版本。首选 pest；若基准证明配置文件解析无法满足性能目标，才评估 nom。`quick-xml`、`serde` 和其他 crate 的版本、许可证与 MSRV 必须进入 Cargo.lock 并按依赖规范审核。

## 实施步骤

1. [x] 建立 `panta-dsl-core` library、公共 AST/诊断 DTO、pest grammar 和限制常量，覆盖 variables/theme/language 分派。
2. [x] 实现 Artifact 聚合、确定性快照和 quick-xml TS 生成；构建产物不修改 `.pa` 源文件。
3. [x] 实现 `panta-dslc` CLI 与 Cargo/CMake 调用入口，使用锁定的 QtTools 预编译 `lrelease`；formatter 由 035 共享核心实现。
4. [x] 添加语法、失败、资源上限、任意 cwd 和稳定输出测试；同步 022/025/030 的核心依赖边界。

## 预计改动

新增 `crates/panta-dsl-core/`、`crates/panta-dslc/` 及 fixtures；更新 workspace `Cargo.toml`/`Cargo.lock`、CMake 构建入口和 `resources/i18n/panta-*.pa` 示例字典。具体模块/API 以实现时实际归属为准，不把生成的 TS/QM 或构建树放入仓库。

## 清理与兼容例外

删除任务 022/025 各自实现的重复 lexer、TS 生成器或源校验路径；当前尚无这些实现，记录为无废弃项。默认无兼容例外。

## 验收标准

- [x] 同一 `.pa` 输入在三平台得到相同规范化 Artifact 快照；未知必需版本、重复 context/ID、非法 locale、语法错误和资源上限均拒绝且不覆盖旧产物。固定快照测试已加入；本批次 macOS/Linux/Windows CI 已通过。
- [x] Pest 语法诊断保留准确 UTF-8 字节位置/范围；版本、locale、消息与 language-only 头部语义诊断定位到相关声明。kind variables/theme/language 分派明确，未知域与重复字段返回稳定诊断。
- [x] quick-xml 根据语言快照生成受支持的 Qt TS XML，context/source、自动生成的内部 id、占位符和 locale 基线不丢失。
- [x] CLI 为单文件可执行物，任意 cwd 可运行且不访问网络/系统 Qt；输出临时 TS 后由锁定的预编译 `lrelease` 生成 QM。
- [x] Cargo workspace 测试、format、lint、Clippy 与关键失败夹具通过；任务 032 当前 deferred，其 100% 覆盖率目标不阻塞本任务，仍以行为/失败契约测试验收。
- [x] 022、025、030 文档明确复用本共享 parser；当前语言构建入口直接依赖 `panta-dsl-core`，仓库没有第二套 parser 或 TS XML 生成实现。025/030 的业务消费者仍由各自 planned task 实施。

## 验证计划与结果

在 workspace 根目录记录 `cargo test --locked --workspace`、`cargo format --check`、`cargo lint --check`、DSL core Clippy、CLI 任意 cwd/失败夹具，以及 macOS/Linux/Windows 的确定性快照测试。Qt QM 冒烟使用任务 022 锁定的 QtTools 预编译包。提交后需以本批次三平台 CI 作为最终平台验收证据。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-16 | 完成 Rust parser/CLI 架构与 `.pa` 简洁语法设计 | 明确 pest、quick-xml、serde、TS/QM 和 CMake 边界 | 设计已提交；实现与构建验证进行中 |
| 2026-09-17 | `cargo fmt --all`；`cargo test -p panta-dsl-core`；`cargo run -p panta-dslc -- check/emit-ts ...` | parser/TS 生成、kebab-case 约束、最长 source 规则和 CLI crate 通过 | 5 个核心单元测试、2 个 fixture 集成测试通过；CLI 成功校验 fixture 并生成 `zh_CN` TS。workspace 级 launcher/Qt 验证仍待预编译 Qt 供给可用后执行 |
| 2026-09-17 | QM 冒烟：`panta-dslc emit-ts resources/i18n/panta-cn.pa → TS`；任务 005 staging 的锁定 `lrelease 6.11.2`（macOS arm64）编译 QM | TS 被 lrelease 接受并产出 QM | 通过：TS 2.1/zh_CN/id/source/translation/%1 占位符完整；lrelease 报 4 翻译全部 finished，QM 245B。`-idbased` 在 Qt 6 已废弃（lrelease 直接拒绝），后续 CMake 接入使用默认参数 |
| 2026-09-17 | 任意 cwd（/tmp）执行 CLI：`check` cn/en、`format --check`、负例 tab 缩进/非法 UTF-8/未知 kind/重复 header | 校验通过返回 0；失败返回 2 且带稳定 span 诊断、不写文件 | 通过：`pa.tab_indentation at 3:5`、`pa.invalid_utf8 at byte 15`、未知 kind 的 pest span `1:7`、`pa.duplicate_header at 3:1`；失败路径均未修改文件 |
| 2026-09-17 | Qt 6.11.2 在线仓库基础包归档清单核查（mac/linux/windows Updates.xml）与 qttools 归档下载实测 | 基础包内含 qttools；三平台归档 SHA256 记录进 `qt-provision.cmake` | mac `415b5008…07d3719`（41M）、windows `5f2b387a…56946`（26M）、linux `42d5f3db…71c0ca`（28M）；三归档均含平铺 `bin/lrelease`（另有 lrelease-pro） |
| 2026-09-17 | `cargo build --locked`（macOS arm64，共享树 `target/native/debug`） | TS 生成 → lrelease QM → qrc 嵌入全链接通 | 通过：`i18n/panta_en.ts`、`panta_zh_CN.ts` 由 build.rs 写出（`PANTA_I18N_TS_DIR`），staging 锁定 lrelease 6.11.2 编译 QM 并经 qrc 嵌入 `:/i18n/`；qttools 归档按新解包指纹一次性迁移（旧缓存无指纹，三归档全量重解，之后按指纹跳过），幂等二跑 0.03s |
| 2026-09-17 | `ctest --preset debug`（macOS arm64） | 既有测试不受影响，i18n 链路有端到端证据 | 17/17 全绿，含新增 `I18n.CompiledQmLoads`（QTranslator 从 `:/i18n/` 加载 zh_CN/en QM，`Revision %1 → 修订 %1`、`Error → 错误` 断言通过） |
| 2026-09-17 | 增量与失败路径：`touch resources/i18n/panta-cn.pa` 后重建；向 panta-cn.pa 追加语法错误后 `cargo build`；纯净目录 `cmake -S native -B /tmp/fresh-boundary` | .pa 变更传播到 TS；解析失败保留旧 TS 并给稳定诊断；缺 TS 的 configure 立即失败 | 通过：touch 后 TS mtime 更新；损坏 .pa 构建失败（`panta-cn.pa：pa.syntax at 23:1`）且旧 TS 未动，恢复后全绿；无 `PANTA_I18N_TS_DIR` 的 configure 报错并指引先 `cargo build`（与 FFI 同边界） |
| 2026-09-17 | `cargo test --locked`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`git diff --check` | Rust 侧检查全绿 | 通过 |
| 2026-09-17 | 三平台 CI（push 2f62a69，run [35233366640](https://github.com/Yuki-Nagori/panta/actions/runs/35233366640)） | 三平台走通 qttools 下载 + TS/QM 链路 | macOS/Windows 全绿（含锁定 lrelease 编 QM、qrc 嵌入与链接）；Linux 暴露 qtdeclarative 历史记录哈希笔误（`…db6e92…` 应为 `…dbb6e92…`，此前被 CI 缓存掩盖，新的每归档指纹强制复验后失败）。修复：实测回填正确哈希，并在两处 `file(DOWNLOAD)` 后立即复验 + FATAL——该命令的哈希不符是延迟错误，会继续跑完 configure 并写入带错哈希的解包指纹，毒化后续缓存判定。 |
| 2026-09-17 | 三平台 CI 复跑（push 0c16681，run [35234986184](https://github.com/Yuki-Nagori/panta/actions/runs/35234986184)） | 哈希修正后三平台全绿 | windows-2022 / macos-latest / ubuntu-latest 全部 success：qttools 下载校验、build.rs TS 生成、锁定 lrelease 编 QM、qrc 嵌入与链接在三平台干净环境成立（QTranslator 加载为 ctest，CI 执行待 011 聚合）。 |
| 2026-10-13 | `cargo test --locked -p panta-dsl-core` | 重复 context 稳定拒绝；language message 纳入声明上限，刚好达到上限可通过、超限拒绝 | 通过：新增重复 context/跨 context 同 ID 行为测试；language 16,384 条消息可解析，第 16,385 条返回 `pa.too_many_declarations`。 |
| 2026-10-13 | 最终复验：`cargo format --check`、`cargo lint --check`、`cargo test --locked --workspace`、`git diff --check` | workspace 与 DSL core 检查通过 | 通过：Rust workspace 测试、聚合 lint 全绿，CTest 75/75；Clippy 包含在聚合 lint 中。macOS arm64。 |
| 2026-10-13 | PR #14 三平台 CI（run [38364866559](https://github.com/Yuki-Nagori/panta/actions/runs/38364866559)） | macOS、Linux、Windows 构建及适用质量检查通过 | 通过：三平台 Cargo check/build、格式、lint、Rust 与 native 覆盖率、QML 测试、依赖审计全部通过；CMake/QML 单项 lint 按变更路径跳过。 |
| 2026-10-13 | 重复 `numerus` 字段回归用例；`cargo test --locked -p panta-dsl-core`、`cargo format --check`、DSL core Clippy | 重复元数据不得静默覆盖，定向质量门禁通过 | 通过：第二个 `numerus` 返回 `pa.duplicate_field`；core 单元/集成测试、格式与 Clippy 全绿。 |
| 2026-10-13 | Artifact 快照、确定性 JSON、来源关联诊断、TS 冲突与多来源资源边界测试；`cargo test --locked -p panta-dsl-core` | 输入顺序无关、失败包含 source ID、冲突和资源超限稳定拒绝 | 通过：固定 JSON fixture、source ID 错误关联、16 MiB 聚合输入、256 来源和 65,536 声明上限均由测试覆盖。 |
| 2026-10-13 | launcher `cargo build --locked`；`panta-build` 目录发布单测 | build.rs 通过共享 Artifact 核心生成 locale TS；失败时保留既有输出，成功后移除陈旧 TS | 通过：本机 macOS arm64 构建成功；TS 产物出现在 `target/native/debug/i18n`，完整目录替换及非法文件名不触碰旧目录由单测覆盖。 |
| 2026-10-13 | Pest 与语义诊断位置回归；DSL core 定向测试 | 语法诊断保留字节范围，缺失源、非法版本、非法 locale 指向对应声明 | 定向回归通过；完整 workspace 门禁与平台 CI 结果见本轮收尾记录。 |

## 风险与回退

单个 `.pa` 上限 1 MiB，Artifact 聚合上限 16 MiB / 256 个来源 / 65,536 条声明；launcher 与 CLI 在读取前进行有界读取。每份文档最多 16,384 条声明；grammar 没有嵌套表达式或脚本执行。解析和 TS 生成失败时保留上一份有效 TS 集合。QtTools 不可用时构建明确失败并给出预编译供给诊断，不回退系统 `lrelease` 或源码构建。

## 决策与工作记录

- 2026-09-16：创建任务；确认这是应用平台的配置/资源引擎工作，Rust library + 单文件 CLI 是唯一解析实现。
- 2026-09-16：首选 pest；quick-xml 将语言 Artifact 生成 Qt TS XML，serde 负责 DTO，QM 保持 Qt 运行期格式；`.pa` 源码不暴露 XML。
- 2026-09-17：根据 TS 兼容示例，语言 `.pa` 改为 `[Context]` + 短 ID + `src/tr` 元数据结构；支持 `st`、`oldsrc`、`comment`、`extra`、`numerus` 和 plural forms，`cn` 简写归一化为 `zh-CN`。
- 2026-09-17：开始实现 `panta-dsl-core` 与 `panta-dslc`，首期采用 YAML 风格缩进和无引号标量；变量/Theme 的类型表达式保留 `=`。
- 2026-09-17（TS/QM 构建入口）：查明 Qt 6.11.2 在线仓库基础包（clang_64 / linux_gcc_64 / win64_msvc2022_64）的归档清单内含 `qttools-*.7z`（lrelease/lupdate 所在），此前 qt-provision 只取 qtbase/qtdeclarative。实施：qt-provision 增补三平台 qttools 固定归档（下载实测 SHA256）；launcher build.rs 以 build-dependency 复用 `panta-dsl-core` 解析 `resources/i18n/*.pa` 并通过 staging/backup 目录发布构建树 TS（失败保留旧文件集），传 `PANTA_I18N_TS_DIR` 给 CMake；新增 native/i18n 模块用 staging 锁定 `lrelease` 编 QM、`qt_add_resources` 嵌入并附 QTranslator 加载冒烟 ctest。presets 纯净 configure 依赖先 `cargo build` 生成 TS（与 FFI 同边界）。

## 完成摘要

实现、文档、本地验证、独立评审及本批次三平台 CI 均已完成。032 覆盖率工作保持 deferred，不作为本任务阻塞项。

## 历史进展（截至 2026-09-17）

核心 parser、Artifact 快照、TS 生成入口和 formatter 消费契约已实现；实际 language 源文件已加入 resources/i18n/panta-en.pa 与 resources/i18n/panta-cn.pa。QM 冒烟已用任务 005 staging 的锁定 lrelease 打通。TS/QM 构建入口通过 launcher build.rs 调用共享 core，再由 `native/i18n` 编 QM、以 qrc 嵌入并由 `I18n.CompiledQmLoads` 端到端验证。旧记录中的 032 覆盖率与业务消费者实施要求已按当前边界修订：032 是 deferred，022/025/030 负责各自运行时业务；034 提供唯一 parser、Artifact 与语言构建基础。

## 本轮实现与验收（2026-10-13）

独立审查发现并修复重复 context、language 声明计数、隐式 Context 往返和重复 `numerus` 问题。本轮新增 Artifact 聚合与固定 JSON 快照、确定性顺序、source ID 诊断、聚合来源/字节/声明上限、按 locale 生成 TS 和输出名冲突拒绝。launcher 在解析全部来源、生成全部 TS 后才通过 `panta-build` 的 staging/backup 目录发布完整文件集。诊断现保留 UTF-8 字节范围，并将消息、版本和 locale 相关语义错误定位到源声明。022/025/030 及 DSL 模块文档已按实际职责更正；032 deferred 不阻塞 034。macOS arm64 的 format、聚合 lint、workspace 测试及 75/75 CTest 均通过；独立最终评审未发现阻塞提交的问题；PR #14 的三平台构建与必需检查全部通过。
