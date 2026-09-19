# 023 — 跨平台路径与资源引用服务

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)（已完成）、[006](006-rust-cpp-boundary.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

建立与 cwd 无关的路径服务和逻辑资源引用，为工程搬迁和运行时提供可验证边界。 当前仅规划，未实施。

## 必读

- [模块设计](../modules/paths-and-runtime.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [文档规范](../standards/documentation.md)
- [验证与评审](../standards/validation-and-review.md)
- [代码生命周期](../standards/code-lifecycle.md)
- [qt](../standards/qt.md)
- [rust](../standards/rust.md)
- [cpp](../standards/cpp.md)
- [cxx](../standards/cxx.md)

## 范围与非目标

包含标准目录发现、工程相对引用、file URL 转换、读写权限错误和 Rust/C++ DTO；不包含虚拟文件系统挂载、远程存储或 VM。

## 前置条件与待决策

006 提供可用 FFI 字符串/错误与所有权契约；明确非 Unicode 路径处理，拒绝有损转换。以临时工程根夹具验证，不等待完整工程存储。

## 实施步骤

1. 定义根类别、资源引用、解析结果与诊断契约，Qt 宿主注入标准目录。
2. 实现 Rust 工程引用规则和 C++ URL 适配，按真实调用点收敛已有路径处理。
3. 覆盖平台路径、根外访问与失败场景，并给部署任务 013 记录安装布局接口。

## 预计改动

现存 native/bridge/、native/app/、crates/；具体服务目录依 006 决定，新建路径夹具与测试，不创建空占位 crate。 上述新增路径/类型均为规划，以实施时实际模块归属为准。

## 清理与兼容例外

删除本任务替代的旧实现、引用及配置，不保留重复路径；未涉及替代的现存功能保持。无兼容例外。

## 验收标准

- [x] 切换 cwd 与搬迁工程根后，相对资产仍正确解析；内置 qrc 资源只读且不当作本机路径。
- [ ] 覆盖空格、中文、file URL 编码、Windows 盘符/UNC、大小写与非 Unicode 策略，三平台记录实际结果。（空格/中文/file URL/非 Unicode/大小写不折叠已实测——大小写断言随 CI 三平台执行；盘符拒绝为跨平台规则本地已测，UNC 与 C++ 侧三平台记录待补）
- [ ] 拒绝绝对路径冒充相对引用、`..` 越界及符号链接/junction 越界，覆盖尚不存在的写入目标。（绝对路径/`..`/未创建写目标/Unix 符号链接已测；Windows junction 待 Windows 平台证据）
- [x] 标准目录为空、不可写、资产缺失返回明确错误，不静默退到 cwd；FFI 往返无有损编码。（空/相对路径注入、不可写目录（Unix 只读 + 写探针）、缺失目录自动创建且无探针残留、资产缺失 `path.not_found`、非 Unicode 往返均有夹具；注入语义：标准目录缺失即创建、写探针验证真实可写）
- [x] 代码、测试、配置和文档一致，删除废弃实现；记录真实验证并同步索引。（无被替代的旧路径实现，记录为无废弃项）

## 验证计划与结果

Rust/native 路径行为测试与三平台 CI，使用隔离临时目录；Windows junction 等场景无法运行时记录未覆盖，不以字符串测试替代平台实测。

| 日期 | 场景 | 实际结果 |
|---|---|---|
| 2026-09-16 | 本次仅完成规划 | 实现与功能验证未执行 |
| 2026-09-17 | `cargo test -p panta-core --locked`（macOS arm64） | 17 项通过：逻辑引用往返与结构错误（缺/未知 scheme）、相对片段规则矩阵（空/NUL/绝对路径含 Unix 上 `C:` 盘符/`..` 词法越界/Windows 保留名/尾随点或空格）、受控 `a/../b` 折叠、三层解析、qrc 拒绝本机化、注入校验（相对根/qrc 根）、缺失根、未创建写目标、Unix 符号链接越界（读 + 写目标均 `path.not_contained`） |
| 2026-09-17 | `cargo test -p panta-ffi --locked`（macOS arm64） | 9 项通过（8 项既有 + 路径服务 1 项）：`PathRef` 往返、未知 scheme、中文/空格路径解析、`../`/`C:`/`COM1` 错误码前缀、qrc → `path.qrc_not_native` |
| 2026-09-17 | `ctest --preset debug`（macOS arm64） | 24/24 全绿，含 7 项 `PathHostTest`：QStandardPaths 测试模式注入（cache 落 `.qttest` 隔离目录）、切换 cwd 结果不变、工程根搬迁后引用跟随新根、写目标解析→落盘→读解析 canonical 一致、拒绝矩阵（`../`、`C:/`、`CON`、尾随点、未知 scheme、缺 scheme、qrc、未注入根 `path.root_missing`）、file URL 单次解码（`%20` 不二次解码、qrc 拒绝）、未配对代理项 `path.non_unicode`、Unix 符号链接越界 |
| 2026-09-17 | `cargo build/test --locked`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`git diff --check` | 通过 |
| 2026-09-17 | 三平台 CI（push 81e98e3，run [35237992065](https://github.com/Yuki-Nagori/panta/actions/runs/35237992065)；含 7919c7f 的 023 代码，前序 run 35237494179 因并发被该 push 取消） | 三平台 success：cargo build/test 在 Windows（`C:/win` 走 Prefix 分支拒绝）与 Linux/macOS 跑通全部路径单测；PathHost C++ 测试不在 CI（CTest 聚合归 011），Windows junction 证据仍待补 |
| 2026-09-17 | 标准目录注入语义收尾（macOS arm64）：`createWithStandardRoots` 夹具——缺失多级目录自动创建且写探针无残留、空/相对路径条目 `path.standard_dir_unavailable`、Unix 只读目录写探针 `path.standard_dir_unwritable`；Rust 大小写断言（`Assets/GearBox.PA` ≠ `assets/gearbox.pa`，服务层不折叠） | `cargo test -p panta-core` 18 项通过；`ctest --preset debug` 27/27（新增 3 项注入夹具）；cargo build/fmt/clippy、`git diff --check` 通过 |
| 2026-09-19 | managed CTest 全量复跑（macOS arm64，修复前） | 23/29；6 项使用 `PathHost::create()` 的用例因 Qt test mode 仍将 macOS 标准目录解析到用户 home 下的 `.qttest`，受控环境无法写入 user-config；显式根夹具和 VTK/FFI/QML 用例通过 |
| 2026-09-19 | 修复后 managed CTest 全量复跑（托管 CMake 4.4.3，macOS arm64） | 29/29 通过；PathHost 测试改用 `QTemporaryDir` 显式根夹具，生产 `QStandardPaths` 路径发现未改动 |

## 风险与回退

规范化规则误改真实文件名；保留原引用和用户资产，失败拒绝操作而非改写文件。

## 决策与工作记录

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-17：依赖 005、006 均已完成且范围/验收明确，状态调整为 ready。
- 2026-09-19：确认 macOS 上 `QStandardPaths::setTestModeEnabled(true)` 仍会落到用户 home 下的 `.qttest`，不适合作为受控环境的写入夹具；PathHost 测试统一通过 `createWithStandardRoots` 注入 `QTemporaryDir` 根，不改变生产路径发现和不可写目录拒绝语义。
- 2026-09-17（实施）：落地分层——`panta-core::path` 定义根类别（project/user-config/app-data/cache/session/qrc）、逻辑资源引用（`scheme:/relative` 结构化，qrc 只读不落本机路径）、Rust 工程引用规则（拒绝空引用/NUL/绝对路径（含 Unix 上伪装成相对组件的 `C:` 盘符）/词法 `..` 越界/Windows 保留名/尾随点或空格组件）与三层解析（`resolve` 纯逻辑、`resolve_existing` 存在性 + canonical 根内包含（拒符号链接/junction 越界）、`resolve_write_target` 以最深现存祖先做包含检查（覆盖未创建目标））；根必须为绝对路径且由宿主显式注入，解析与 cwd 无关（结构保证）。FFI（panta-ffi）新增 `PathService` 句柄与 `PathRef` DTO，跨边界仅传可往返 UTF-8。C++ 侧 `native/bridge` 新增 `PathHost`：QStandardPaths 类别注入（user-config→AppConfigLocation、app-data→AppDataLocation、cache→CacheLocation、session→TempLocation）、QString→UTF-8 往返校验（不可往返即拒绝，非 Unicode 策略落在此处）、file URL 单次解码（QUrl::toLocalFile，qrc/其它 scheme 不当本机路径）。Rust 单测覆盖引用规则矩阵与 unix 符号链接越界；Windows junction 实测由 CI 平台补（本地仅 macOS）。

## 完成摘要

未完成（保持 in-progress）。已落地：Rust 路径层（根类别/逻辑引用/三层解析/拒绝矩阵/大小写不折叠，`panta-core::path`）、FFI DTO 与 `PathService`（`panta-ffi`，越界枚举拒绝）、Qt 宿主 `PathHost`（QStandardPaths 注入映射——同时作为 013 的安装布局接口、注入时缺失目录自动创建 + 写探针验证可写、UTF-8 往返校验、file URL 单次解码）及 macOS 全量测试（Rust 18+9、C++ 10 项，ctest 27/27），Rust 侧随 CI 三平台通过。待补：Windows junction 实测、UNC/C++ 侧三平台记录（PathHost 测试进 CI 归 011）。
