# 042 — 三平台自有 C++ 统一 LLVM/Clang 工具链

- 状态：planned
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[032](032-cross-language-quality-gates.md)、[038](038-native-sdk-artifact-production.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-18 / 2026-09-18

## 目标与背景

维护者在本轮评审明确要求三平台统一 LLVM。拟将 macOS/Linux/Windows 的自有 C++（CMake native 与 Cargo CXX 桥接）统一到同一固定 LLVM release：macOS/Linux 使用上游 clang/clang++，Windows 使用 clang-cl。当前 macOS 的 Apple Clang、Linux 的 GCC 和 Windows 的 cl.exe 将不再作为自有 C++ 默认编译器。

“统一”指编译器来源、release 版本和选择策略；不是统一操作系统 ABI。macOS 保留 CLT/Xcode 提供的 SDK 与平台运行库，Linux 保留满足发布基线的 sysroot/glibc/C++ 运行库，Windows 保留 MSVC target、CRT/STL 和 Windows SDK。Rust 继续使用锁定 rustc（其 LLVM 后端不因此替换）；Rust 覆盖率工具仍与 rustc 配套，不能改用新 C++ LLVM 的 llvm-cov。

clang-cl 以 MSVC ABI 互操作为目标，但具体 C++ 特性、运行库及预编译包组合仍需实测。不能据此断言当前 Qt/VTK/OCCT/Netgen 组合已兼容，或上游均已验证本仓库的锁定版本。Windows 的 sanitizer 支持也不与 Linux/macOS 等同，TSan 官方支持平台不含 Windows。

官方依据（2026-09-18 查阅；版本落地时重新核对锁定版本）：[LLVM MSVC compatibility](https://clang.llvm.org/docs/MSVCCompatibility.html)、[clang-cl 使用方式](https://clang.llvm.org/docs/UsersManual.html#clang-cl)、[ThreadSanitizer 支持平台](https://clang.llvm.org/docs/ThreadSanitizer.html#supported-platforms)。

## 必读

- [技术基线](../standards/baseline.md)、[CMake 规范](../standards/cmake.md)、[C++ 规范](../standards/cpp.md)
- [依赖获取](../standards/dependency-acquisition.md)、[CXX 规范](../standards/cxx.md)
- [质量工具链](../modules/quality-tooling.md)、[验证与评审](../standards/validation-and-review.md)
- [注释](../standards/comments.md)、[仓库文件](../standards/repository-hygiene.md)、[文档](../standards/documentation.md)、[生命周期](../standards/code-lifecycle.md)、[提交规范](../standards/commits.md)

## 范围与非目标

范围：macOS arm64、Linux x86_64、Windows x64 的 CMake native、`crates/panta-ffi/build.rs` 经 cxx-build/cc 编译的 C++、本地开发与 CI 入口；固定同一 LLVM release 的三平台工具供给、编译器/公共告警策略和缓存身份，核对 Qt 与 SDK 消费边界。clang-format 一并评估对齐该 release，并按 032 更新供给与格式基线。

非目标：不更换 Rust 工具链/target，不引入 MinGW ABI 或强制三平台使用同一标准库/链接器，不承诺三平台 sanitizer 等价。第三方预编译库不因自有代码迁移就被视为 LLVM 产物；SDK 生产链是否同步迁移必须逐平台作出决策。若重产 SDK，先更新 038 范围及资产元数据，不覆盖已有发布资产。

## 前置条件与待决策

- 保留 018/032/038 的排期依赖；032/038 尚未完成，当前 planned，不意味着 clang-cl 技术上依赖覆盖率达 100%。开始前须有三平台可复现基线、消费资产及质量入口；若 032 的 C++ 覆盖率需依赖本任务，先拆分里程碑调整依赖，避免形成环。
- 选择一个同时具备三平台可验证资产的 LLVM release，固定版本、URL、摘要、许可与最低系统要求；优先复用托管 provision，缺失明确失败，不回退到系统编译器。没有可用官方资产时先评估自托管供给，不能用浮动 Homebrew/apt/runner 版本冒充统一版本。
- macOS 明确 SDK/sysroot、部署目标、libc++/链接器与架构；Linux 明确 sysroot、glibc/libstdc++ 基线及 SDK ABI；Windows 明确 MSVC Build Tools/STL/SDK 版本。编译器统一不能自动解决平台运行库差异。
- Windows CI 当前是 VS 17 2022 多配置生成器，本地默认是 Ninja：VS 路径拟用 `-T ClangCL` 并指定所锁定 LLVM 安装目录，不能意外选择 VS 内置不同版本；Ninja 显式选择 clang-cl 并初始化开发者环境。macOS/Linux Ninja 显式选择固定 clang/clang++。
- CMake 工具集不控制 Cargo 的 cxx-build/cc。需让两条 C++ 编译链选中同一固定 LLVM release 的编译器，记录实际命令与版本；Rust 链接器保持现状。
- Qt/SDK 各平台现有产物先作为拟验证组合，记录实际生产编译器。Windows 核对 x64、/MD、`_ITERATOR_DEBUG_LEVEL=0`、Debug 映射 Release Qt、异常/RTTI；Linux 核对 libstdc++ ABI/符号版本，macOS 核对 libc++/部署目标。SDK 管线逐平台评估统一 LLVM 的收益和成本，保留混合工具链须有消费证据。

## 实施步骤

1. 记录三平台现有编译器的干净/增量、Debug/Release、CTest 与 CXX 边界基线，选择固定 LLVM release 并落实三平台供给。
2. launcher/build.rs 按生成器接入编译器/工具集设置并追踪环境变化；同步 panta-ffi 的 cxx-build/cc 选择。C/C++ 编译器路径均显式指定，两条链记录版本与实际命令，工具缺失不回退。
3. 审查 `native/cmake/build-policy.cmake`：区分 MSVC ABI、编译器 ID 与前端；建立共同告警意图与平台参数映射，核对 CRT/STL、异常、RTTI、调试信息及链接器。记录平台专有检查，未识别参数不得静默忽略。
4. 使用干净 CMake 树；CI cache key 与 restore-key 纳入 LLVM 版本、平台/架构、工具集和运行库基线；不恢复旧编译器配置树。三平台各验证一次冷构建和增量构建。
5. 三平台 CI 与本地入口验证 Debug/Release 的 Cargo build/test（含完整 CTest、qmllint、格式与 Clippy）、FFI 异常/生命周期、Qt 模块与动态库加载。macOS 补部署目标、Linux 补运行库符号基线、Windows 补 VS/Ninja 两种配置入口的验证。
6. 为 ASan/UBSan/TSan 建立实际支持矩阵，只在工具链和运行库支持的平台验证；不支持项注明理由。C++ coverage 使用该 LLVM 配套工具，与 Rust llvm-tools 分开。此步向 032 提供选型证据，不将所有 sanitizer 功能接入都列为迁移前置。
7. SDK 生产链逐平台决定保留或迁移；消费验证只覆盖已接入项，未集成的引擎标未测。回写基线、构建说明、CMake/CXX 规范与本任务；新增供给/SDK 范围先更新 020/038。

## 预计改动

现存 `crates/launcher/build.rs`、`crates/panta-ffi/build.rs`、`.github/workflows/ci.yml`、`native/cmake/build-policy.cmake`、`crates/launcher/src/provision.rs`、必要的 presets/ABI 检查、clang-format 供给及相关规范。SDK workflow 仅在 038 范围确认需要重产时修改，不创建占位文件。

## 清理与兼容例外

三平台验收后移除自有代码默认 Apple Clang/GCC/cl.exe 选择和失效缓存配置，保留仍需的平台 SDK/运行库。无兼容例外，不为失败文件混用旧工具集；若验证失败，恢复该平台上一套完整可用配置。

## 验收标准

- [ ] 三平台 CMake 与 Cargo CXX 两条链的实际命令证明使用同一固定 LLVM release；版本/来源/摘要可追溯，缺失明确失败；Rust 工具链不变。
- [ ] macOS/Linux Ninja、Windows VS 多配置与 Ninja 有可复现本地步骤，均不意外使用系统默认编译器。
- [ ] 三平台干净/增量、Debug/Release 的 Cargo build/test、完整 CTest、qmllint、fmt/clippy 均通过，测试发现/跳过与基线无意外差异。
- [ ] 公共告警策略及平台参数映射、标准库/系统库、异常/RTTI、调试信息和当前 Qt/FFI/SDK 消费组合有验证证据；未集成项明确标未测。
- [ ] sanitizer 支持矩阵有实证或官方限制依据，C++/Rust coverage 工具各自配套，不宣称支持完全一致。
- [ ] 切换/回退不复用旧编译器构建树；缓存身份、工具供给与文档体现固定 LLVM release。
- [ ] SDK 三平台生产链与 clang-format 版本决策、清理项、task/索引和实际能力一致。

## 验证计划与结果

实施验证以三平台实机/CI run 为准，记录 commit、系统/编译器/SDK 版本与命令。本次仅评审任务，未执行编译器切换。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-18 | 仓库构建链与官方文档审查 | 按维护者要求扩为三平台统一固定 LLVM；记录 ABI/sanitizer 边界，补 CXX 链、供给、运行库、缓存与双配置验收。三平台迁移实测待实施；本地链接、索引状态/依赖无环检查通过 |

## 风险与回退

主要风险为预编译包的 CRT/STL 或异常边界差异、clang-cl 未识别参数与旧 CMake 缓存。失败时恢复受影响平台的完整构建配置和相应缓存身份，以全新构建树验证原 Apple Clang/GCC/cl.exe 基线；保留已有 SDK 发布资产及用户数据。

## 决策与工作记录

- 2026-09-18：从 Windows clang-cl 规划开始评审；维护者明确要求全部统一 LLVM，范围扩为三平台自有 C++ 和 CXX 桥接使用同一固定 LLVM release。稳定保留任务编号及文件路径，补三平台供给和运行库边界；本次仅更新规划，无实现变更、无兼容层。

## 完成摘要

未完成，等待依赖交付、版本选定、三平台供给与迁移验证。
