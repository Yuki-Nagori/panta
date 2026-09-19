# 011 — 统一测试与质量入口

- 状态：done
- 阶段：验证基础
- 依赖：[007](007-vtk-quick-viewport.md)、[008](008-tasks-errors-logging.md)、[010](010-netgen-adapter-smoke.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-19

## 目标与背景

让统一入口可证明 Rust、native 与 QML 的检查实际执行，形成 CI 可复用命令。

根 `tests/` 已落地跨语言聚合和 Cargo 质量入口；本任务仍保留真实窗口/图形冒烟与依赖任务未完成的边界，不把无头测试当作桌面生命周期验收。

## 必读

- [通用规范：validation-and-review](../standards/validation-and-review.md)
- [通用规范：documentation](../standards/documentation.md)

- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [规范：cpp](../standards/cpp.md)
- [规范：rust](../standards/rust.md)
- [规范：qml](../standards/qml.md)
- [规范：qt](../standards/qt.md)
- [规范：vtk](../standards/vtk.md)
- [架构：milestones-and-validation](../architecture/milestones-and-validation.md)

## 范围与非目标

范围：根 `tests/` 的 Rust/native/QML 聚合、Cargo aliases、失败传播和 CI 可复用质量命令。Rust 单元测试继续遵循 Cargo 惯例保留在实现文件或 crate `tests/`；真实窗口、平台图形和业务适配器不在本任务内实现。

非目标：不要求文档/无代码目录执行不存在的检查，不建设大型测试框架。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 清点已存在测试并分 Rust、CTest、QML lint 和图形冒烟，标注平台/显示服务要求。
2. 实现 cargo test 的原生聚合调度，防止从聚合测试递归调用自身；失败返回非零。
3. 接入 rustfmt、Clippy、C++ 格式/选定静态分析及文档链接检查，并记录真实命令。
4. 区分默认自动检查、目标平台图形检查和可选 sanitizer，规定跳过必须显式报告。
5. 当前 Rust workspace 以 `-D warnings` 运行 Clippy；测试与 build script 不通过 `expect`/`unwrap` 警告豁免掩盖问题。

## 预计改动

测试聚合入口、CTest 注册、质量工具配置、文档检查与使用说明。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [x] cargo test --locked 实际执行约定 Rust/native 套件；受控失败能导致顶层命令失败。
- [x] 当前 Rust workspace 的 Clippy warning 直接失败；既有 warning 已清理，CI 与本地命令保持一致。
- [x] 报告含测试数量和跳过原因；零个意外缺失的 native 测试不能视作通过；跨语言入口统一由根 `tests/` 调度。
- [x] QML 检查（qmllint 全量目标 + Qml.FormatCheck CTest）与行为测试经根 `tests/integration/native.rs` 聚合进 cargo test；真实窗口/图形冒烟仍为独立验证记录。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。
- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景按下表区分已验证与待执行。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | `cargo clippy --locked --workspace --all-targets -- -D warnings`（完整 workspace，复用已缓存 Qt staging）；`cargo fmt --all -- --check`；`cargo test --locked --workspace --exclude panta-launcher` | 完整 workspace Clippy 通过且无 warning；Rust 测试 15/15；fmt 通过 |
| 2026-09-18 | 仓库根，macOS arm64 / rustc 1.98.1；`cargo test --locked --workspace` | 101 项 Rust/聚合测试通过，含 CTest 28/28、qmllint 与 C++ 格式；Qt 路径测试需沙箱外测试配置目录写权限 |
| 2026-09-18 | `cargo test --locked --release --target-dir target/review-target -p panta-tests --test native` | 2/2 聚合、28/28 CTest 通过；验证 profile 与自定义目录，复用已缓存第三方依赖 |
| 2026-09-18 | 临时 CTest 夹具，编译并执行根 `tests/integration/native.rs`；空套件、失败用例、成功用例 | 前两者聚合 exit 101，成功 exit 0；Windows 后缀/多配置参数静态核对，Windows/Linux 本轮 CI 待跑 |
| 2026-09-18 | `cargo format`、`cargo lint cmake`、`cargo test --locked`、`cargo build --locked --workspace` | 根入口与 Cargo 驱动 native 构建通过；CTest 28/28，QML 格式、qmllint 与 QML 行为测试均执行 |
| 2026-09-19 | GitHub Actions run [35425146629](https://github.com/Yuki-Nagori/panta/actions/runs/35425146629)（`2ebbea7`，main push）三平台 | Windows/Linux 以同一聚合入口执行完整测试：三平台 success（6m29s），`cargo test --locked --workspace` 全绿，Windows 26/26 CTest 含两个 QML 运行测试与格式检查；此前"Windows/Linux 本轮 CI 待跑"证据补齐，任务关闭 |

## 风险与回退

聚合命令可能漏跑 native 测试、递归调用自身或将跳过当成功；用受控失败与执行数量核验实际覆盖。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-18（本轮评审）：复核暂存聚合入口：修复 Windows ctest.exe、多配置 -C、profile/target-dir 定位与零测试误报；补受控失败证据后提交。

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-17：根据质量要求先收敛 Rust workspace 门禁；workspace lints 将 Clippy warning 提升为 deny，CI 额外传入 `-D warnings`，并清理 FFI build script/测试中的 `expect` warning。
- 2026-09-18：CTest/QML 聚合落地（与 032 协同）：根 `tests/integration/native.rs` 聚合 `all_qmllint` 与全部 CTest，`cargo test` 一条命令覆盖 Rust + native + QML；ctest 定位经 build.rs 导出的 `PANTA_CMAKE` 同目录。评审后构建树与配置由 build.rs 注入，修复 Windows .exe、CTest -C、release/自定义 target-dir 以及空套件误报；qmllint/CTest 顺序执行。clang-format 已接入，C++ 覆盖率与逐项测试遗漏检测仍待后续增量。
- 2026-09-18：跨语言聚合迁移到根 `tests/` package；标准 `cargo test` 通过显式集成测试执行 qmllint/CTest，避免把聚合入口绑定在 launcher crate。
- 2026-09-18：根 `tests/` 聚合入口规划落地到任务 043；native 编排从 launcher 测试目录迁出，跨语言命令统一使用 Cargo aliases。
- 2026-09-19：run 35425146629 三平台全绿补齐 Windows/Linux 聚合证据；验收全部满足，标记 done。

## 完成摘要

根 `tests/` 已统一 Rust、C++、QML 和 CMake 的测试调度，失败与空套件会传播到 Cargo；真实窗口/图形冒烟和依赖任务的业务验收继续由 007/008/010 等任务负责。
