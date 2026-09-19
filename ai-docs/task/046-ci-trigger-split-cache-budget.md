# 046 — CI 触发拆分与缓存预算

- 状态：in-progress
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[012](012-ci-reproducibility.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-19 / 2026-09-19

## 目标与背景

三平台缓存在 GitHub 10 GB 仓库配额内互相驱逐（每平台压缩后约 4 GB，合计超限），冷构建反复出现；同时纯文档提交也触发全部约 14 个 job。本任务把 CI 触发按路径拆分，并把缓存压回配额内，保证质量门禁不因路径过滤被静默跳过。

本地实测（2026-09-19，macOS arm64，`target/` 与 `~/.cargo`）：

- `panta-tools/llvm` 7.4 GB：`bin` 5.3 GB（官方全量发布含 mlir-opt 等约 200 个各 100–290 MB 工具，实际使用仅 clang 系、clang-tidy、llvm-profdata/llvm-cov、lld-link）、`lib` 1.9 GB（LLDB/MLIR 静态库与 dylib 均未使用；`lib/clang/22` 45 MB 为编译器资源目录，必须保留）、`include` 203 MB（LLVM 开发头文件，非运行时所需）。
- `panta-tools/archives` 1.6 GB（工具下载归档，解包后可重下载）、`cmake` 531 MB、`cargo-build` 596 MB。
- `panta-deps/qt` 1.5 GB（含 196 MB 归档）、`panta-deps/sdk` 501 MB（含 86 MB 归档）。
- `~/.cargo/registry` 1.6 GB（`src/` 为解包副本，可由 cache 重生成）。

## 必读

- [验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)
- [质量工具模块](../modules/quality-tooling.md)、[依赖获取](../standards/dependency-acquisition.md)
- 任务 [012](012-ci-reproducibility.md)（缓存键与 restore-key 语义）、[042](042-unified-llvm-toolchain.md)（LLVM 工具集约束：验证器要求 clang-tidy/llvm-cov/llvm-profdata，Windows 链接使用 lld-link）

## 范围与非目标

范围：`.github/workflows/ci.yml` 触发拆分与缓存结构；索引与相关模块文档同步。不改变任何本地构建/供给实现（`panta-build`、`native/cmake` 不动），不在本轮做 sanitizer、LLVM 换源或制品外置。跳过判定必须保守：未知路径一律视为代码触发全量检查。

## 实施步骤

1. 新增 `changes` job：按事件取基线（PR 取 `origin/<base>`，push 取 `event.before`，新分支/空 diff 回退全量），把变更文件分为 code / cargo_deps / rust / native / qml。
2. 既有 job 挂 `needs: changes` 与保守门控：纯文档 → 全部跳过；dependency-audit → 触发 `cargo_deps`；machete → `rust`；cmake-lint → `native`；qmllint → `qml` 或 `native`；其余（check 矩阵、format、clippy、clang-tidy、includes、cppcheck、两类 coverage）→ 非纯文档即运行。
3. 缓存拆为两键（cargo-home 仅 index/cache/git、panta-cache 含 tools+deps），lint/audit/format/coverage 全部 restore-only，仅 main push 的 check job 保存。
4. 缓存瘦身下沉到供给代码（维护者要求不在 CI 打补丁）：panta-build 安装归档发布即删、解包后按白名单裁剪 LLVM（bin 留 clang 系/lld 系/coverage/AR，lib 仅留 lib/clang，include 移除）；Qt/SDK CMake 供给发布即删归档，Qt 复用判定不再依赖归档存在。
5. 公共步骤封装为 `.github/actions/*` 复合 action：changed-paths、panta-toolchain、linux-gl-prereqs、panta-cache-restore、panta-cache-save。
6. check job 输出裁剪后体积（`du -sh`），为后续预算调整留证据。

## 预计改动

- `.github/workflows/ci.yml` 与 `.github/actions/*`：changes 门控、缓存两键、复合 action。
- `crates/panta-build/src/lib.rs`：归档发布即删、`slim_llvm` 裁剪及单元测试。
- `native/cmake/qt-provision.cmake`、`native/cmake/sdk-provision.cmake`：归档发布即删，Qt 复用条件改为指纹 + staging 完整性。
- `ai-docs/task-index.md`、`ai-docs/modules/quality-tooling.md`、`ai-docs/standards/dependency-acquisition.md`：登记与 CI 结构描述同步。

## 清理与兼容例外

无兼容例外；旧的单条 `panta-deps-*` 大缓存条目随 LRU 驱逐自然淘汰，不保留双写。

## 验收标准

- [ ] 纯文档（`*.md`、`ai-docs/**` 等）push/PR 仅运行 `changes` job，其余 job 显示 skipped 且 run 绿。
- [ ] 未知或代码路径变更触发全量检查；deny.toml 仅触发 dependency-audit；machete/cmake-lint/qmllint 按域触发。
- [ ] 三平台新缓存条目合计 ≤ 8 GB（按 job 日志 Cache Size 汇总），main push 后 restore 命中。
- [ ] 缓存瘦身由供给代码完成：panta-build 发布即删归档并裁剪 LLVM（单元测试覆盖白名单与资源目录保留），Qt/SDK 供给发布即删归档且复用不依赖归档；CI 保存动作不做内容清理。
- [ ] LLVM 裁剪后 `panta-tests toolchain` 核验与全量测试在三平台通过。
- [ ] 文档、task、索引与 CI 实际行为一致；无未登记的兼容分支。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-19 | 本地 `du -sh` 实测缓存目录；actionlint；过滤脚本对样例文件清单（docs-only/rust-only/native-only/deny-only/未知）五场景断言 | 门控输出符合矩阵 | 待回填 |
| 2026-09-19 | push 后 run：changes 输出、各 job 触发矩阵、三平台 check（含 toolchain 核验）、Cache Size 汇总 | 全量触发 + 新条目 ≤ 8 GB | 待回填 |

## 风险与回退

裁剪误删构建所需工具会让三平台 check 失败——白名单覆盖 clang 系全部前缀、llvm-ar/llvm-ranlib/llvm-symbolizer、lld 系，并以 `panta-tests toolchain` 与完整 CTest 为验收；失败即回退裁剪步骤，仅保留路径拆分。门控过宽漏检由保守默认（未知路径全量）兜底。回退仅撤销本任务 workflow 变更。

## 决策与工作记录

- 2026-09-19：创建任务。选 DIY diff 脚本而非第三方 paths-filter action（少一个供应链面，未知路径默认全量）。
- 2026-09-19（撤销 test 拆分）：曾把 test 拆为独立 job 经 artifact 传递构建树，实测 upload-artifact 保留构建时 mtime、新 checkout 源码反而更新，cargo 全量重编（run 35429575198 macOS/Ubuntu test 失败于此），收益为负；按维护者决定回到 check/build/verify/test 单 job。
- 2026-09-19（瘦身下沉供给层）：维护者指出可在代码层优化的不要放进 CI。CI 保存动作只负责存取，裁剪下沉：panta-build 发布即删归档 + `slim_llvm` 白名单裁剪（本地开发机同步受益），Qt/SDK CMake 供给发布即删归档并把复用判定改为指纹 + staging 完整性。legacy `panta-deps-*` 恢复键随首代新键落地移除。

## 完成摘要

未完成。
