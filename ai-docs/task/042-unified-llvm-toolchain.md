# 042 — 三平台自有 C++ 统一 LLVM/Clang 工具链

- 状态：in-progress
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[032](032-cross-language-quality-gates.md)、[038](038-native-sdk-artifact-production.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-18 / 2026-09-19

## 本轮修复范围（2026-09-19）

按维护者要求修复复审列出的全部基础设施缺口：提取公共构建支持 crate，供给采用文件锁、摘要隔离和原子发布；统一 Ninja 与 Windows SDK 环境；runner 按命令准备工具；固定并托管 uv/Python/Cppcheck；覆盖自有 CXX 编译命令；native coverage 使用 C++ LLVM 配套工具；工具核验读取实际编译数据库与 CMakeCache。工具选择收敛和验证证据随实施回填，不用本机旁路冒充三平台通过。

## 目标与背景

维护者在本轮评审明确要求三平台统一 LLVM。已实现 macOS/Linux/Windows 自有 C++（CMake native 与 Cargo CXX 桥接）选择固定 LLVM 22.1.7 的初版供给：macOS/Linux 使用 clang/clang++，Windows 使用同一发行包内的 clang-cl。当前 macOS 的 Apple Clang、Linux 的 GCC 和 Windows 的 cl.exe 将不再作为自有 C++ 默认编译器。

“统一”指编译器来源、release 版本和选择策略；不是统一操作系统 ABI。macOS 保留 CLT/Xcode 提供的 SDK 与平台运行库，Linux 保留满足发布基线的 sysroot/glibc/C++ 运行库，Windows 保留 MSVC target、CRT/STL 和 Windows SDK。Rust 继续使用锁定 rustc（其 LLVM 后端不因此替换）；Rust 覆盖率工具仍与 rustc 配套，不能改用新 C++ LLVM 的 llvm-cov。

clang-cl 以 MSVC ABI 互操作为目标，但具体 C++ 特性、运行库及预编译包组合仍需实测。不能据此断言当前 Qt/VTK/OCCT/Netgen 组合已兼容，或上游均已验证本仓库的锁定版本。Windows 的 sanitizer 支持也不与 Linux/macOS 等同，TSan 官方支持平台不含 Windows。

官方依据（2026-09-18 查阅；版本落地时重新核对锁定版本）：[LLVM MSVC compatibility](https://clang.llvm.org/docs/MSVCCompatibility.html)、[clang-cl 使用方式](https://clang.llvm.org/docs/UsersManual.html#clang-cl)、[ThreadSanitizer 支持平台](https://clang.llvm.org/docs/ThreadSanitizer.html#supported-platforms)。

## 必读

- [技术基线](../standards/baseline.md)、[CMake 规范](../standards/cmake.md)、[C++ 规范](../standards/cpp.md)
- [依赖获取](../standards/dependency-acquisition.md)、[CXX 规范](../standards/cxx.md)
- [质量工具链](../modules/quality-tooling.md)、[验证与评审](../standards/validation-and-review.md)
- [注释](../standards/comments.md)、[仓库文件](../standards/repository-hygiene.md)、[文档](../standards/documentation.md)、[生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

范围：macOS arm64、Linux x86_64、Windows x64 的 CMake native、`crates/panta-ffi/build.rs` 经 cxx-build/cc 编译的 C++、本地开发与 CI 入口；固定同一 LLVM release 的三平台工具供给、编译器/公共告警策略和缓存身份，核对 Qt 与 SDK 消费边界。clang-format、clang-tidy 一并使用该 release；格式和静态分析不再消费独立 LLVM 版本。

非目标：不更换 Rust 工具链/target，不引入 MinGW ABI 或强制三平台使用同一标准库/链接器，不承诺三平台 sanitizer 等价。第三方预编译库不因自有代码迁移就被视为 LLVM 产物；SDK 生产链是否同步迁移必须逐平台作出决策。若重产 SDK，先更新 038 范围及资产元数据，不覆盖已有发布资产。

## 前置条件与待决策

- 保留 018/032/038 的排期依赖；032/038 尚未完成，本任务已进入 in-progress，不意味着 clang-cl 技术上依赖覆盖率达 100%。开始前须有三平台可复现基线、消费资产及质量入口；若 032 的 C++ 覆盖率需依赖本任务，先拆分里程碑调整依赖，避免形成环。
- 已选择 LLVM 22.1.7：三平台 URL、SHA256、许可证和缓存键已固定于 provision 与依赖获取清单；受支持平台缺失资产直接失败，不回退系统编译器。没有可用官方资产时先评估自托管供给，不能用浮动 Homebrew/apt/runner 版本冒充统一版本。
- macOS 明确 SDK/sysroot、部署目标、libc++/链接器与架构；Linux 明确 sysroot、glibc/libstdc++ 基线及 SDK ABI；Windows 明确 MSVC Build Tools/STL/SDK 版本。编译器统一不能自动解决平台运行库差异。
- 三平台统一 Ninja；Windows 从 MSVC Build Tools 读取 SDK/CRT 环境，并显式选择 target 下的 clang-cl 编译 C/C++。macOS/Linux 显式选择固定 clang/clang++；macOS 两条链均显式传 Apple SDK。
- Cargo 的 cxx-build/cc 与 CMake 分别显式接收同一托管 LLVM 目录中的编译器；CMake 配置阶段检查 clang 版本，Rust 链接器保持现状。
- Qt/SDK 各平台现有产物先作为拟验证组合，记录实际生产编译器。Windows 核对 x64、/MD、`_ITERATOR_DEBUG_LEVEL=0`、Debug 映射 Release Qt、异常/RTTI；Linux 核对 libstdc++ ABI/符号版本，macOS 核对 libc++/部署目标。SDK 管线逐平台评估统一 LLVM 的收益和成本，保留混合工具链须有消费证据。

## 实施步骤

1. 选定 LLVM 22.1.7，登记 macOS ARM64、Linux x86_64、Windows x64 官方资产及 SHA256；缓存键位于 `target/panta-tools/llvm`，与 CMake/Ninja 同一托管根。
2. launcher/build.rs 接入 LLVM 编译器、clang-format 和版本注入；panta-ffi 的 cxx-build/cc 显式选择 clang++ 或 clang-cl，工具缺失不回退。
3. `native/cmake/build-policy.cmake` 在非系统旁路下检查 C/C++ 编译器均为 LLVM 22.1.7；保留 Windows /MD、`_ITERATOR_DEBUG_LEVEL=0` 和 Qt Release ABI 策略。
4. CI cache key 纳入 `llvm-22.1.7`、平台、架构、生成器和运行库标签；编译器路径变化触发干净 configure。
5. 三平台 CI 与本地入口验证 Debug/Release 的 Cargo build/test（含完整 CTest、qmllint、格式与 Clippy）、FFI 异常/生命周期、Qt 模块与动态库加载。macOS 补部署目标、Linux 补运行库符号基线、Windows 补 Ninja 下 SDK/CRT 环境发现与实际 clang-cl 路径验证。
6. 为 ASan/UBSan/TSan 建立实际支持矩阵，只在工具链和运行库支持的平台验证；不支持项注明理由。C++ coverage 使用该 LLVM 配套工具，与 Rust llvm-tools 分开。此步向 032 提供选型证据，不将所有 sanitizer 功能接入都列为迁移前置。
7. SDK 生产链逐平台决定保留或迁移；消费验证只覆盖已接入项，未集成的引擎标未测。回写基线、构建说明、CMake/CXX 规范与本任务；新增供给/SDK 范围先更新 020/038。

## 预计改动

现存 `crates/launcher/build.rs`、`crates/panta-ffi/build.rs`、`.github/workflows/ci.yml`、`native/cmake/build-policy.cmake`、`crates/panta-build/src/lib.rs`、tests 工具链核验、相关规范与任务索引。SDK workflow 仅在 038 范围确认需要重产时修改，不创建占位文件。

## 清理与兼容例外

已移除自有 C++ 对 Apple Clang/GCC/cl.exe 的默认选择；受支持平台在显式设置 `PANTA_USE_SYSTEM_TOOLS=1` 时使用系统工具；无固定资产的平台明确失败，不再隐式回退。平台 SDK/运行库仍由宿主提供。无兼容例外，不为失败文件混用旧工具集；若验证失败，恢复该平台上一套完整可用配置。

## 验收标准

- [ ] CMake 与 Cargo CXX 两条链显式使用同一 LLVM 22.1.7 目录；版本、来源、SHA256 可追溯，缺失明确失败；Rust 工具链不变。
- [ ] macOS/Linux 选择 clang++，Windows 选择 clang-cl；三平台使用 Ninja，Windows 注入 SDK 环境并显式选择 clang-cl。
- [ ] 三平台干净/增量、Debug/Release 的 Cargo build/test、完整 CTest、qmllint、fmt/clippy 均通过，测试发现/跳过与基线无意外差异。
- [ ] 公共告警策略及平台参数映射、标准库/系统库、异常/RTTI、调试信息和当前 Qt/FFI/SDK 消费组合有验证证据；未集成项明确标未测。
- [ ] sanitizer 支持矩阵有实证或官方限制依据，C++/Rust coverage 工具各自配套，不宣称支持完全一致。
- [x] LLVM 版本进入工具 marker、CI cache key、CMake 版本检查和文档；切换编译器会因 CMake 缓存/版本检查而要求重新配置。
- [ ] SDK 三平台生产链与 clang-format 版本决策、清理项、task/索引和实际能力一致。

## 验证计划与结果

实施验证记录实际命令、工具版本、缓存路径和 CI 平台结果；三平台完整构建仍由本次 CI 运行补齐。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | `cargo fmt --all -- --check`、`cargo check --locked -p panta-ffi --all-targets`、`cargo clippy --locked -p panta-ffi --all-targets -- -D warnings`（系统工具旁路仅用于本机快速编译） | 通过；Cargo CXX build.rs 已显式选择统一 LLVM resolver，代码与锁文件可解析。|
| 2026-09-19 | LLVM 22.1.7 官方 release 资产核对、依赖清单与 CI cache key 审查 | 通过；macOS ARM64、Linux x86_64、Windows x64 URL/SHA256 已固定，Windows 资产为 clang+llvm 归档，保留 clang-cl。|

| 2026-09-19 | ABI 元数据夹具：改前 configure 失败；改后 `check-abi.cmake` 的 valid/runtime/iterator 三场景 | 通过；隔离供给检查后正例成功、两种错误覆盖均被拒绝；使用 Apple Clang，仅验证 CMake 元数据策略 |
| 2026-09-19 | `gh run list --limit 5`，核对最近远程 CI | 最新可见 run 35356240701 为旧提交 6ae8ed3，结论 failure；不能将旧绿灯或本机系统旁路用作当前 LLVM 实现的三平台证据 |

## 2026-09-19 复审时发现的问题（修复记录见下）

- **P1：Windows 工具链未闭合。** CI 使用 Visual Studio 生成器，launcher 仅对 Ninja 供给 Ninja，核验器却强制要求 Ninja 和 compile_commands；CMake 官方明确 VS 不支持该数据库。`-T ClangCL` 与显式编译器参数尚不足以证明 MSBuild 使用 target 中的 LLVM，需要固定安装根并实跑核验。
- **P1：冷构建供给存在竞争。** panta-ffi 与 panta-tests 的 build.rs 共用 `llvm`、归档和 marker，没有互斥与原子发布，可能在对方下载或解包时删除目录。
- **P1：native coverage 工具错配。** C++ 已指定 LLVM 22.1.7，但 CI 仍通过 rustup 找 profdata/cov；应使用 C++ 发行包配套工具，Rust 覆盖率保留 rustc 配套工具。
- **P1（本轮修复）：ABI 元数据夹具被新版本检查提前拦截。** 夹具未设置 LLVM 版本，原本正例也失败；现明确只在该夹具启用系统工具旁路，生产门禁不变。
- **P2：验证器只检查预期路径。** 未读取 CMakeCache/实际编译命令，未校验 CXX 编译器与 clang-tidy；不能将工具文件存在等同于已被构建使用。
- **P2：配置切换与平台边界。** 系统旁路和 coverage 仅向 CMake 写 ON，取消环境变量后缓存不自动回 OFF；无资产平台仍隐式回退。Release、sanitizer、平台运行库及未来 SDK 消费验证尚未完成。

## 风险与回退

主要风险为预编译包的 CRT/STL 或异常边界差异、clang-cl 未识别参数与旧 CMake 缓存。失败时恢复受影响平台的完整构建配置和相应缓存身份，以全新构建树验证原 Apple Clang/GCC/cl.exe 基线；保留已有 SDK 发布资产及用户数据。

## 决策与工作记录

- 2026-09-18：从 Windows clang-cl 规划开始评审；维护者明确要求三平台统一 LLVM。
- 2026-09-19：任务文件重命名为 `042-unified-llvm-toolchain.md`；固定 LLVM 22.1.7，macOS/Linux 使用 clang++，Windows 使用 clang-cl；Cargo provision、CXX、CMake、CI、质量工具与文档同步实现，无兼容层。

## 完成摘要

统一 LLVM 供给和编译器选择已落地：Cargo 默认下载并校验 LLVM 22.1.7 到 `target/panta-tools/llvm`，CMake 与 panta-ffi CXX 使用同一目录；macOS/Linux 选择 clang++，Windows 选择 clang-cl；clang-format 与 clang-tidy 复用该版本。本轮已补无资产平台拒绝、共享供给互斥与版本隔离、Windows Ninja/SDK 配置、CXX 数据库和实际编译器核验、native coverage 配套 LLVM 工具。三平台冷/增量和 Debug/Release 全矩阵仍需当前代码 CI 证据，不能用本机系统旁路替代。

## 2026-09-19 修复验证

- macOS arm64 使用官方 LLVM 22.1.7 冷安装，摘要验证通过；`cargo build --locked --workspace`、`panta-tests toolchain` 成功，CMakeCache 与 CMake/CXX 数据库指向同一托管目录。未启用系统工具旁路。
- 两条 C++ 链显式使用 Apple SDK libc++ 头与 sysroot，修复 LLVM 发行包 libc++ 头引用平台运行库尚无的 `__hash_memory` 符号。部署目标取 `MACOSX_DEPLOYMENT_TARGET`，未指定时与 cc 的 SDK 默认值一致。
- `cargo test --locked --release` 通过，Rust 与 native CTest 28/28；Debug 聚合由 `cargo quality` 通过。`cargo coverage native` 使用 LLVM 22 配套 profdata/cov 成功生成报告（函数 93.51%、行 92.93%、分支 50.26%，包含自有测试源，不能视为纯业务代码指标）。QML 仅有行为验证，CXX adapter 插桩与 native 百分比门禁仍待 032。
- panta-build 的 12 项测试通过：并发安装只发布一次、失败升级保留旧版本、进程退出后重试成功、数据库筛选和实际编译器拒错、Cppcheck 依赖模型投影。Rust coverage 对象目录复用根 target LLVM 缓存，不再重复下载。
- Linux/Windows 当前代码未在本机运行，远程 CI 尚未触发；完整平台运行库、sanitizer 与后续 CAE SDK 消费矩阵仍未完成，保持 in-progress。
