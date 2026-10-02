# 023 — 跨平台路径与资源引用服务

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[005](005-qt-qml-shell.md)（已完成）、[006](006-rust-cpp-boundary.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-10-02

## 目标与背景

建立与 cwd 无关的路径服务和逻辑资源引用，为工程搬迁和运行时提供可验证边界。Rust 路径服务、FFI DTO 与 Qt `PathHost` 已落地；任务 086 将标准目录发现从 Bridge 的直接 `QStandardPaths` 调用迁入 `native/qt-adapter`，不改变本任务的路径语义。任务保持 in-progress，剩余平台验收见下方清单与验证记录。

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
- [ ] 覆盖空格、中文、file URL 编码、Windows 盘符/UNC、大小写与非 Unicode 策略，三平台记录实际结果。（既有 Rust / C++ 场景已有三平台 CI；新增盘符 / UNC URL 和反斜杠拒绝矩阵本地通过，新增场景三平台记录待补）
- [ ] 拒绝绝对路径冒充相对引用、`..` 越界及符号链接/junction 越界，覆盖尚不存在的写入目标。（绝对路径/`..`/未创建写目标/Unix 符号链接已测；新增 Windows junction 测试待 Windows CI 平台证据）
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

- 2026-10-02：盘点发现逻辑引用约定仅使用 `/`，但 Rust 校验尚未拒绝反斜杠；Windows `PathBuf::join` 可将其解释成盘符 / UNC / 父级分隔。范围补充为在 Rust 引用校验处统一拒绝反斜杠，增加稳定错误码及 Rust / CXX 端到端拒绝矩阵；本机路径与 file URL 转换不受此逻辑引用规则影响。

- 2026-10-02：继续 P1 路径验收。最新 CI 36982743570 已覆盖三平台 C++ PathHost；补充盘符 / UNC file URL 单次解码与 UNC 相对引用拒绝矩阵，增加 Windows 真实目录 junction 夹具，验证现存读目标和未创建写目标根外访问均被拒绝。UNC 网络共享访问不在本轮创建；区分 URL 语义验证与网络可用性，不将 macOS 结果冒充 Windows junction 通过。

- 2026-09-16：由任务 021 编排；长期设计见模块说明，不将文档完成等同功能完成。
- 2026-09-17：依赖 005、006 均已完成且范围/验收明确，状态调整为 ready。
- 2026-09-19：确认 macOS 上 `QStandardPaths::setTestModeEnabled(true)` 仍会落到用户 home 下的 `.qttest`，不适合作为受控环境的写入夹具；PathHost 测试统一通过 `createWithStandardRoots` 注入 `QTemporaryDir` 根，不改变生产路径发现和不可写目录拒绝语义。
- 2026-09-17（实施）：落地分层——`panta-core::path` 定义根类别（project/user-config/app-data/cache/session/qrc）、逻辑资源引用（`scheme:/relative` 结构化，qrc 只读不落本机路径）、Rust 工程引用规则（拒绝空引用/NUL/绝对路径（含 Unix 上伪装成相对组件的 `C:` 盘符）/词法 `..` 越界/Windows 保留名/尾随点或空格组件）与三层解析（`resolve` 纯逻辑、`resolve_existing` 存在性 + canonical 根内包含（拒符号链接/junction 越界）、`resolve_write_target` 以最深现存祖先做包含检查（覆盖未创建目标））；根必须为绝对路径且由宿主显式注入，解析与 cwd 无关（结构保证）。FFI（panta-ffi）新增 `PathService` 句柄与 `PathRef` DTO，跨边界仅传可往返 UTF-8。C++ 侧 `native/bridge` 新增 `PathHost`：QStandardPaths 类别注入（user-config→AppConfigLocation、app-data→AppDataLocation、cache→CacheLocation、session→TempLocation）、QString→UTF-8 往返校验（不可往返即拒绝，非 Unicode 策略落在此处）、file URL 单次解码（QUrl::toLocalFile，qrc/其它 scheme 不当本机路径）。Rust 单测覆盖引用规则矩阵与 unix 符号链接越界；Windows junction 实测由 CI 平台补（本地仅 macOS）。
- 2026-09-29：任务 086 将 `PathHost` 和 `ProjectViewModel` 的标准目录查询迁至 `panta::qt_adapter::standard_location`；类别映射、UTF-8 边界、根注入及错误语义保持不变。原有 QStandardPaths 行为验证仍适用于该 adapter 转发，平台覆盖缺口按本任务验收清单保留。

## 完成摘要
未完成（保持 in-progress）。已落地：Rust 路径层（根类别/逻辑引用/三层解析/拒绝矩阵/大小写不折叠，`panta-core::path`）、FFI DTO 与 `PathService`（`panta-ffi`，越界枚举拒绝）、Qt 宿主 `PathHost`（标准目录注入映射——同时作为 013 的安装布局接口、注入时缺失目录自动创建 + 写探针验证可写、UTF-8 往返校验、file URL 单次解码）及 macOS 全量测试（Rust 18+9、C++ 10 项，ctest 27/27），Rust 侧随 CI 三平台通过。既有 PathHost 测试已纳入 Cargo 聚合并通过三平台 CI；待补新增 Windows junction 与 UNC URL / 拒绝矩阵的三平台记录。


## 2026-10-02 平台回归补充

- CI [36985311142](https://github.com/Yuki-Nagori/panta/actions/runs/36985311142) 的 Windows Cargo / ASan 均在 junction 夹具目标断言失败，尚未执行该用例的 Rust 越界拒绝断言；其余失败日志未发现独立故障。本轮修复范围为使用 Qt 专用 `isJunction()` / `junctionTarget()` 核对真实 junction 与目标，替换错误的 `canonicalFilePath()` 直接比较；保留创建失败即失败、读 / 写越界拒绝及清理检查。任务仍待修复后的 Windows CI 验证。

- 最新 CI [36982743570](https://github.com/Yuki-Nagori/panta/actions/runs/36982743570) 的三平台 Cargo 聚合及 sanitizer 均成功，已执行 PathHost C++ 测试；旧记录中“C++ 尚未进 CI”仅是当时状态，当前缺口为新增 UNC / junction 场景的平台证据。
- 新增反斜杠逻辑引用矩阵在修复前于 macOS 失败（UNC 片段被错误接受），日志 `/tmp/panta-023-repro.log`。Rust 校验新增 `path.backslash_rejected`，错误码 / 详情遍历与 CXX 端到端测试同步；不把 Windows 本机路径字符串作为逻辑引用。
- file URL 测试覆盖盘符与 UNC 中的中文、空格、#、%20，要求单次解码且原值保留；不连接或创建网络共享，不能据此宣称网络共享 I/O 通过。
- Windows 测试通过真实 `mklink /J` 创建临时目录 junction，先用 `isJunction()` / `junctionTarget()` 核对类型与目标，再验证根外现存读目标及未创建写目标均拒绝；移除 junction 后检查目标文件仍在。创建失败直接失败，不跳过。[Microsoft mklink 文档](https://learn.microsoft.com/windows-server/administration/windows-commands/mklink) 定义 `/J` 创建目录 junction；[Qt QProcess 文档](https://doc.qt.io/qt-6/qprocess.html#setNativeArguments) 要求 cmd.exe 使用原生命令行；[Qt QFileInfo 文档](https://doc.qt.io/qt-6/qfileinfo.html#junctionTarget) 定义 junction 专用目标查询（2026-10-02 查阅）。测试使用固定相对参数及 QProcess working directory，不将临时路径插入 shell 文本。
- Unix 符号链接跨 CXX 测试补充未创建写目标拒绝，与原 Rust 行为回归一致。生产 UI、工程数据和原生 file URL 转换保持不变；无兼容例外。

- 初次新增 URL 测试识别出 Unix 解码盘符 URL 得到 `/C:/...`、Windows 得到 `C:/...` 的差异，已按平台分别断言；不通过宽松规范化吞掉差异。修复后 `cargo test --locked --workspace` 通过，69/69 CTest（macOS arm64，仓库根目录）；日志 `/tmp/panta-023-tests-reviewed.log`。
- 本轮未取得新 Windows / Linux 场景结果；UNC 网络共享 I/O 未测试。Windows junction 与新增 URL / 拒绝矩阵仍需下一次 CI 验证，任务保持 in-progress。

- 提交前评审：`cargo format --check`、`cargo lint --check` 与 `git diff HEAD --check` 通过，lint 日志 `/tmp/panta-023-lint.log`。Rust 负责逻辑引用规则，C++ 仅做 Qt URL 适配及跨语言测试；无重复规则、旧路径实现或兼容分支。

- CI 夹具修复后的 macOS arm64 回归：`cargo test --locked --workspace` 通过，69/69 CTest，日志 `/tmp/panta-023-ci-fix-tests.log`；`cargo format --check`、`cargo lint --check` 与 `git diff HEAD --check` 通过，质量检查日志 `/tmp/panta-023-ci-fix-lint.log`。评审核对保留原有 Rust 根包含规则及读 / 写越界、目标未被写入、清理后目标文件仍在的断言，无生产代码或 UI 改动。Windows 专用修复尚未本地执行，不以 macOS 聚合结果代替 Windows junction / ASan 验收；任务与索引继续 in-progress。
