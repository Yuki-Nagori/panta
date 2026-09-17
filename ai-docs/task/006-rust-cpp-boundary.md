# 006 — Rust/C++ FFI 最小契约

- 状态：in-progress
- 阶段：基础平台
- 依赖：[004](004-cargo-native-orchestration.md)（已完成：Cargo 调度与运行入口就绪）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

以最小调用验证跨语言所有权、错误和链接顺序，为应用服务建立边界。

已开始实施最小 CXX 双向调用；完整 Project/Storage 仍不在本任务范围。

## 必读

- [通用规范：comments](../standards/comments.md)

- [规范：ffi](../standards/ffi.md)
- [规范：cxx](../standards/cxx.md)
- [规范：rust](../standards/rust.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [架构：application-and-storage](../architecture/application-and-storage.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不建设完整 Project/Storage，也不引入 Python binding。

## 前置条件与待决策

004 已完成且 Cargo→CMake 入口可用。实施前冻结 CXX 版本、桥接 DTO、错误编码和静态库传递；未验证的跨平台 ABI 仍保留在验收项中。

## 实施步骤

1. 优先验证 CXX，锁定 cxx 与生成器完全一致的版本并核实其 Rust 最低版本，更新规范及依赖基线。
2. 设计简单请求/响应、句柄释放和错误结构，明确编码、长度、线程和异常策略。
3. 分别实现 C++ 调 Rust 和 Rust 调 C++ 的最小路径，覆盖 DTO/opaque 所有权；不经 Rust 绕行渲染，不假定任意方向回调均可直接绑定。
4. 优先验证单个 Rust staticlib + 同版本 cxxbridge 生成代码 + CMake 最终链接；明确胶水只编译一次、产物传递和无环依赖，记录 allocator 与运行库要求。

## 预计改动

最小 FFI crate、native 应用服务适配、生成/链接规则、FFI 规范。执行前根据真实结构修订；不得顺手实现非目标功能。

本轮实际边界：新增 `crates/panta-ffi` 作为 `staticlib`/Rust 测试 crate；CXX 生成头由 Cargo build script 暴露给 native CMake，CMake 只消费该静态库并编译一个 GTest 边界测试。没有把 Qt、OCCT、Netgen 或 VTK 类型放入桥接。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [x] 跨语言成功/失败路径返回结构化结果；非 ASCII 文本与空输入行为定义明确。
- [ ] 重复创建/释放、无效输入和错误转换有可运行验证；不存在未经处理的跨边界异常展开。
- [ ] 干净构建与增量构建保持正确，没有第二套 Cargo 递归编译链。
- [x] CXX 两侧生成版本一致；双向调用及 CMake 最终链接通过，Result/异常与 panic-abort 行为被明确区分。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。
- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | CXX 双向最小路径：Rust `process` 调 C++ `cpp_prefix`；native GTest 调 Rust `process` | 待实现后验证 DTO、非 ASCII、空输入和错误转换 |
| 2026-09-17 | `cargo fmt --all -- --check`；`cargo metadata --locked --no-deps`；`git diff --check` | 通过；workspace 成员、CXX 1.0.202 依赖和锁文件结构可解析。 |
| 2026-09-17 | `CARGO_TARGET_DIR=/private/tmp/panta-ffi-dedicated cargo test -p panta-ffi --locked` | 未取得编译结果：本机新编译 Rust build script/可执行文件停留在 macOS `_dyld_start`，与项目源码无关；已终止本轮孤儿进程。需在 CI 或可正常启动新 Rust 二进制的环境补跑。 |
| 2026-09-17 | `cargo check -p panta-ffi --all-targets`；`cargo fmt --all -- --check` | 通过：桥接模块加 `#[allow(unsafe_code)]` 后，workspace `-D unsafe-code` 不再拦截 CXX 生成胶水（原 4 个错误清零）；格式检查通过。 |
| 2026-09-17 | `cargo clippy -p panta-ffi --all-targets` | 通过：仅测试代码 3 处 `expect_used` 警告（workspace 设为 warn，策略归任务 011）。 |
| 2026-09-17 | `cargo test -p panta-ffi --locked`（默认 target 目录） | 通过：3 个测试全部成功，含 Rust→C++ 调用与非 ASCII 往返；上一轮 dyld 卡挂未复现，补齐此前欠的运行证据。 |
| 2026-09-17 | 手动以等效 cargo 环境变量驱动 launcher build.rs 完成 CMake 全量构建（Qt staging 与 googletest 复用本地缓存，零下载；DEP 注入规则另由最小复现验证）。首跑暴露 build.rs E0382、DEP 不注入、boundary_test 缺 main 三处阻塞 | 修复后构建通过：84 个 ninja 目标全绿，panta_ffi_boundary_test 链接成功，产出 panta-native。 |
| 2026-09-17 | `ctest -R Ffi`（上条构建树，macOS 26 arm64 / c++ 20 / Qt 6.11.2 staging） | 通过：`Ffi.RustCppBoundary` 1/1，覆盖 C++ 调 Rust 非 ASCII 往返（"界"×2 → "ffi:界界"）与 Rust 结构化错误转 `rust::Error` 异常。 |
| 2026-09-17 | panic-abort 区分验证：桥接新增 `panic_probe` 验收探针；`cargo test -p panta-ffi`（4 测试含 `#[should_panic]`）+ 重建静态库后 `ctest -R Ffi` | 通过：Rust 侧 4/4；C++ 侧 `EXPECT_DEATH(panic_probe())` 确认 panic 中止进程而非以异常穿越，与 `Result`→`rust::Error` 可恢复路径构成对照。注意改 lib.rs 后须 `cargo build` 刷新 staticlib，`cargo test` 不重建该产物。 |

## 风险与回退

CXX 生成器版本不一致、glue 重复编译或双向符号未链接会破坏构建；先做最小双向调用，再扩展服务。不可将 panic 当作可恢复错误。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施；根据用户提出的 CXX 方案及官方文档，将 CXX 列为首选验证路线。
- 2026-09-17：开始实施；冻结 `cxx`/`cxx-build` 使用同一 `1.0.x` release，`FfiRequest`/`FfiResponse` 只含受支持的 UTF-8 字符串和整数；Rust 错误通过 CXX `Result` 转为 C++ 异常，调用方必须捕获，空输入和超大 repeat 明确拒绝。
- 2026-09-17：CMake 最终链接通过 launcher build dependency 传递 `panta-ffi` 静态库与生成头；CMake 不回调 Cargo，桥接 glue 只由 Rust build script 编译一次。
- 2026-09-17：修复 workspace `unsafe_code = "deny"` 拦截 CXX 生成胶水的问题。Cargo 禁止成员在 `lints.workspace = true` 下覆盖同名 lint（报 "cannot override workspace.lints"），故不改成员 Cargo.toml，而是在桥接模块上加模块级 `#[allow(unsafe_code)]` 并随代码注明安全前提；workspace 注释同步指向该放开点，其余 crate 维持 deny。后续方向是收窄而非移除：allow 只覆盖宏生成胶水，接口层保持纯安全签名（DTO + `Result`，两侧调用方不写 unsafe）；新增手写 unsafe 另设专用模块并逐块写 `// SAFETY:`。`unsafe extern` 声明与胶水内部 unsafe 属 FFI 固有成本，不作为清理目标。
- 2026-09-17：修复 fd0d228 引入的 launcher build.rs 借用错误（`out_dir` 移动后又被 `ffi_artifacts` 借用，E0382）。该提交落地时本地 `cargo test` 因 dyld 问题未完成、commit 未推送故 CI 未覆盖，问题在本次补跑验证时暴露。
- 2026-09-17：修复 `DEP_PANTA_FFI_INCLUDE` 不注入的问题。本机 cargo 1.95.0/1.98.1（Homebrew 与 rustup 双渠道）最小复现均证实：links 包的 metadata 只沿普通 `[dependencies]` 边注入下游构建脚本，仅列在 `[build-dependencies]` 时不注入；将 panta-ffi 移入 launcher 的 `[dependencies]`。判别过程与证据记录于验证表；该行为与既有 cargo 文档表述不一致，后续升级工具链时需复查。
- 2026-09-17：boundary_test 初次真实链接即失败：测试无自定义 `main` 却只链 `GTest::gtest`（fd0d228 从未本地链接验证过该目标）；按 foundation 先例改链 `GTest::gtest_main`。同轮发现 launcher build.rs 重建追踪缺 `native/ffi` 目录，一并补入。
- 2026-09-17：为 panic-abort 验收新增 `panic_probe` 探针（仅验收用，不承载业务功能）：C++ 侧 `EXPECT_DEATH` 证实 panic 中止进程、不以异常穿越，与 `Result` 错误路径区分。本地单平台已验证；三平台干净/增量构建待推送后由 CI 覆盖。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

未完成。最小双向路径与错误/panic 行为区分已在本机验证（Rust 4 测试 + native GTest 含 death test + CMake 最终链接）；剩余：三平台干净/增量构建（推送后 CI 覆盖）、以及重复创建/释放验收项——当前 DTO-only 范围尚无 opaque 句柄，待该场景引入时补验证。全部验收完成后再标 done。
