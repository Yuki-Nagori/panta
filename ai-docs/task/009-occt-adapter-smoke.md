# 009 — OCCT 依赖与 STEP 适配冒烟

- 状态：in-progress
- 阶段：CAE 接入基础
- 依赖：[003](003-cmake-native-skeleton.md)（已完成）、[008](008-tasks-errors-logging.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-20

## 目标与背景

证明固定 OCCT 依赖可被 native adapter 使用，并形成最小可验证 STEP 输入边界。

本任务尚未实施；拟改路径不代表文件已存在，执行前核对依赖任务的实际产物。

## 必读

- [规范：occt](../standards/occt.md)
- [规范：cpp](../standards/cpp.md)
- [规范：cmake](../standards/cmake.md)
- [架构：geometry](../architecture/geometry.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不做完整工程持久化、拓扑稳定命名、交互选择或几何修复产品；不保留装配/名称/颜色等 XDE 附属元数据（冒烟选 STEPControl 基础路径，见决策记录）；不做中途取消。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

已核实：031 manifest 三平台 OCCT 8.0.1 资产（Release `sdk-occt-netgen-8.0.1-6.2.2604`）齐备；本机 macOS arm64 经 `panta_require_sdk("occt")` 首次 configure 即完成下载/校验/供给/find_package。

## 实施步骤

1. 配置 imported targets/本地依赖封装；根据元数据需要选择基础 STEP 或 XDE 读取。
2. 用可分发小型样例读取/转换，生成自有摘要：单位、包围盒、拓扑数量及诊断。
3. 使用任务基础处理失败与取消边界，记录 shape 所有权和暂定串行策略。
4. 建立有效/无效样例测试，更新实际版本与限制，明确这是适配冒烟而非完整 UI 导入。

## 预计改动

native/geometry/（include/panta/geometry/step_import.hpp 公共契约 + src/occt/step_reader.cpp 适配器，均已创建）、tests/cpp/geometry/step_import_test.cpp、tests/fixtures/geometry/ 样例与 README（已创建）、native/CMakeLists.txt 挂接新模块（已改）；Windows 嵌套 bin 运行库目录收集（crates/panta-build/src/lib.rs，见决策记录）。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

无废弃项；无兼容例外。

## 验收标准

- [ ] 样例读入有非空有效结果且单位/摘要可核验，非法文件不导致进程异常退出。
- [ ] 名称/颜色/装配支持或不支持有记录；公共头没有泄露 OCCT 类型。
- [ ] 重复导入和失败清理通过，OCCT 运行库可正确定位。
- [ ] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-20 | 探针（树外 CMake 工程 include `sdk-provision.cmake` + `panta_require_sdk("occt")`，托管 CMake 4.4.3 + LLVM 22.1.7，macOS arm64） | 通过：首次 configure 约 5.5s 完成 Release 资产下载、SHA256 强校验、staging 原子发布与 `find_package(OpenCASCADE CONFIG)` + `TKernel`/`TKDESTEP` target 自检；缓存 marker 命中后离线幂等 |
| 2026-09-20 | OCCT 8.0.1 制品布局核对（macOS 归档 + Windows 归档清单） | macOS：lib/ 全量 dylib、`@rpath/libTK*.8.0.dylib` 安装名，CMake 构建 rpath 自动解析（与 VTK 消费同型），bin/ 仅 env 脚本；Windows：DLL 在嵌套 `win64/vc14/bin/`（非顶层 bin），原 `native_test_env` 的 `bin` 通配覆盖不到——已修（见决策记录）。TKDESTEP 依赖链含 TKV3d/TKService 等，均在同 staging lib 内 |
| 2026-09-20 | `cargo build --locked`（macOS arm64，托管 LLVM/Qt/OCCT） | 通过：native/geometry 首次编入构建图并静态链接 OCCT；公共头零第三方 include（includes 检查通过） |
| 2026-09-20 | `ctest --test-dir target/native/debug -R StepImport` | 7/7 通过：mm 夹具摘要（Millimetre/系数 1/bbox 0-10,20,30/solids 1/faces 6/edges 12）、metre 夹具单位换算（Metre/系数 1000/bbox 0-1000³）、缺失文件 kFileUnreadable、垃圾/空文件 kParseFailed 不崩溃、失败后导入恢复、重复导入稳定 |
| 2026-09-20 | `ctest --test-dir target/native/debug`（全量） | 36/36 通过：既有 29 项不受影响，新增 7 项 StepImport |
| 2026-09-20 | `cargo test --locked --workspace` | 通过：18 个测试目标全 ok，含 native_and_qml_suite 36 项聚合与 panta-build 的 `windows_sdk_dll_dirs` 单测（顶层 bin + 嵌套 win64/vc14/bin 布局） |
| 2026-09-20 | `cargo lint` + `cargo format --check`（macOS arm64） | 通过：clippy/machete/cmake-format/cmake-lint/qmllint/clang-tidy/includes/cppcheck 零告警。过程中修复：clang-tidy 要求枚举显式 `std::uint8_t` 基础、测试辅助函数相邻同型参数收敛为 `BoxExpectation` 结构体；cppcheck 对编译数据库内带引号 `-D` 宏转义误读为语法错误——改为注入裸路径 + 测试代码内字符串化宏，连带消除 `import_step_summary` 的跨 TU unusedFunction 误报 |
| 待 CI | push 后三平台 CI（018 矩阵） | Linux/Windows 首次 configure 将真实下载 OCCT 制品并编译/测试本模块；Windows 侧同时复验 `native_test_env` 嵌套 bin PATH 修复。此前 009 验收项不勾选、任务保持 in-progress |

## 风险与回退

STEP 读取成功仍可能得到空形状、错误单位或丢失元数据；逐项验证摘要和保留策略，失败时不发布候选几何。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-17：依赖 003、008 均已完成，但按 [031](031-prebuilt-native-dependencies.md) 的盘点与范围决策，OCCT 目前只有 Windows 官方预编译 SDK（V8.0.1 vc14-64），macOS arm64 与 Linux 无官方资产；在 031/038 提供全平台可缓存、可校验的 OCCT 制品前不启动第三方源码构建，也不以单平台 SDK 冒充三平台适配验证。阻塞解除条件：031/038 交付各平台固定 URL/SHA256/ABI 匹配的 OCCT SDK 并登记进固定清单。
- 2026-09-19：031 已登记 OCCT 全平台资产（Release `sdk-occt-netgen-8.0.1-6.2.2604`，OCCT 三平台统一自托管、与 Netgen 成对发布）并经 macOS 生产消费烟测，上述阻塞解除条件满足，状态转 ready。
- 2026-09-20（路线）：选 `STEPControl_Reader` 基础路径，STEPCAF/XDE 不引入；装配结构、名称、颜色、层级等附属元数据不保留（`DESTEP_Parameters` 的 ReadName/ReadColor 属 XCAF 消费面，基础路径不消费）。完整导入业务如需元数据再评估 XDE 路线。
- 2026-09-20（并发与所有权）：`Interface_Static` 与 translator 参数为进程级状态（standards/occt.md），适配器内进程级互斥锁串行全部导入，调用方无须加锁；并发隔离证据出现前不放开。读取器与形状逐次新建于单次调用内，返回后不保留 OCCT 状态，摘要为自有值类型；重复导入与失败后恢复由测试固定。
- 2026-09-20（失败边界）：OCCT 异常（`Standard_Failure`/`std::exception`）在适配器边界捕获转结构化 `StepImportStatus`。实测：缺失文件 → RetError→kFileUnreadable；垃圾/空文件 → RetFail→kParseFailed，同时 OCCT 向 stderr 打印 StepFile 解析错误（上游 printer 行为，冒烟不改写）。`FileUnits()` 在损坏文件上实测段错误——只在 `ReadFile==RetDone` 后调用，边界写入适配器注释。
- 2026-09-20（单位语义）：源单位取 `FileUnits()` 长度单位名（实测小写 ISO 拼写，如 `millimetre`/`metre`）；`StepModel::LocalLengthUnit`/`IsInitializedUnit` 是转移目标单位（恒 mm）且在转移前不初始化，均不表源单位。混合/未识别单位按 kUnknown 记系数 0，坐标仍由读取器转为毫米。
- 2026-09-20（公差策略）：`BRepBndLib::Add` 的 `Bnd_Box::Get` 含形状公差 gap（典型 `Precision::Confusion` ≈1e-7 mm）；摘要按含公差的保守包络报告并在公共头记录，测试断言容差 1e-6 mm。不通过调大公差掩盖坏模型。
- 2026-09-20（Windows 运行库）：OCCT 8 Windows 制品 DLL 位于嵌套 `win64/vc14/bin/`；`panta-build::windows_sdk_dll_dirs` 改为递归收集 `bin` 目录（深度上限 4）并更新单测，与 007 的 Qt runtime PATH 机制合并生效。产品 app 的 DLL 分发策略仍归 031/007 后续。
- 2026-09-20（夹具与模块边界）：mm/m 样例由临时生成器（`BRepPrimAPI_MakeBox` + `STEPControl_Writer`，`STEPControl_AsIs`，`write.step.unit=MM/M`）产出后入库，生成器不入仓库，来源/单位/预期记录于 `tests/fixtures/geometry/README.md`；`write.step.unit` 须在 controller 初始化（构造 writer）后设置，否则 SetCVal 失败。geometry 模块冒烟阶段不入安装导出集，消费业务任务接入时按 foundation 模式补装。
- 待记录：三平台 CI 复验（Linux/Windows configure 下载 OCCT、编译、测试；Windows 嵌套 bin PATH 行为）；多根/装配 STEP 样例与 XDE 元数据路线不在本任务内。

## 完成摘要

未完成。macOS arm64 已实现并通过全量本机验证（实现行为、验证证据、限制与取舍见上方记录）；三平台 CI 复验通过、验收项核实后，与索引一起标 done。
