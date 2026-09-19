# 038 — Native SDK 制品生产与发布

- 状态：in-progress
- 阶段：交付基础
- 依赖：[031](031-prebuilt-native-dependencies.md)、[020](020-toolchain-provisioning.md)
- 优先级：P0
- 负责人：待分配
- 创建 / 更新：2026-09-17 / 2026-09-20

## 目标与背景

VTK、OpenCASCADE、Netgen 在目标平台没有统一、完整且可直接消费的官方 C++ SDK。为了坚持预编译优先，由受信 CI 按固定源码 commit 和工具链生产可复用 SDK，开发者和普通 CI 只下载校验后的归档，不在本地构建第三方源码。当前仅完成任务编排，尚未生成制品。

## 必读

- [Native 依赖供给模块](../modules/native-dependency-supply.md)
- [依赖获取与主平台环境](../standards/dependency-acquisition.md)
- [技术基线](../standards/baseline.md)
- [CMake 规范](../standards/cmake.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

范围：为 macOS arm64、Linux x86_64/aarch64、Windows x86_64 生成 VTK/OCCT/Netgen SDK；固定源码 commit、构建器镜像、CMake 选项和 ABI；运行 package 自检；产出 manifest、SHA256、SBOM、许可证集合、provenance 和可供 031 下载的归档。

非目标：不在开发者工作区编译第三方源码，不实现 VTK/OCCT/Netgen 适配器，不替换 031 的下载/缓存逻辑，不把 Python wheel 或系统包管理器作为 C++ SDK。

## 前置条件与待决策

031 必须先冻结每个依赖的版本、模块和 CMake package 入口；020 必须提供可复核的 CMake/Ninja 工具链。开始生产前核实各上游许可证、目标平台编译器/运行库 ABI、Netgen × OCCT 组合和 VTK `RenderingWebGPU`/`RenderingUI` 是否能在制品中启用。VTK 制品不再链接 Qt；发布位置、签名服务和 runner 隔离策略待项目维护者确定。

## 实施步骤

1. 为每个依赖建立可审计的构建描述：源码 URL/commit、工具链版本、配置开关、依赖版本、禁用项和许可证来源。
2. 在隔离的 CI runner 生产各平台 SDK，构建结果只上传到制品存储，不自动写回源码仓库或开发者缓存。
3. 运行 package 自检，确认头文件、库、运行时、CMake config/imported targets、所需模块和许可证完整；检查架构、ABI 与 Qt 兼容元数据。
4. 生成归档 manifest、SHA256、SBOM 和 provenance，按依赖/版本/triple 隔离发布；更新 031 的固定清单和下载入口。
5. 让 007、009、010 使用真实制品完成最小集成，再记录未覆盖 GPU/平台并决定是否发布。

## 预计改动

CI workflow、构建描述、制品 manifest/校验脚本、许可证汇总、031 供给清单及发布文档。归档和构建树不提交源码仓库；具体存储路径和签名配置实施时确定。

## 清理与兼容例外

删除普通开发流程中出现的 FetchContent、系统路径或隐式源码 fallback；若当前不存在则记录无废弃项。无兼容例外；历史制品升级通过新版本 manifest 管理，不在构建逻辑中保留双路径。

## 验收标准

- [ ] 三平台每个目标依赖都有可下载、可校验的 SDK 归档，包含版本、triple、ABI、Qt 兼容范围、模块、CMake package、许可证和 SHA256。
- [ ] 受信 CI 可从固定源码 commit 和工具链重建相同 manifest；开发者 `cargo build`/CMake 不编译第三方源码。
- [ ] 缺包、哈希错误、架构/ABI 不匹配、缺 CMake target、缺 `RenderingWebGPU`/`RenderingUI` 或许可证时立即失败并给出诊断。
- [ ] 031 清除缓存后能按 manifest 重建，正确缓存支持离线 configure；不同版本缓存互不覆盖。
- [ ] 007、009、010 使用 imported targets 完成对应平台的最小链接/运行冒烟，未覆盖环境明确记录。
- [ ] SBOM、许可证和 provenance 随制品发布，升级/回退不破坏上一有效版本。
- [ ] 代码、CI、固定清单、文档和 task 自洽，废弃源码入口已清理且无未登记兼容分支。

## 验证计划与结果

先在隔离 runner 运行依赖包自检和哈希复现，再在三平台清除缓存执行 configure/package smoke；以 031、007、009、010 的真实集成命令作为最终证据。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | `git ls-remote --tags https://gitlab.kitware.com/vtk/vtk.git "*9.7.0*"` | `v9.7.0` 为 annotated tag（对象 `a78e2d95…`），解引用 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`（"Update version number to 9.7.0"）——规范原 pin `23f0a095621e` 为其前缀，无误 |
| 2026-09-17 | `cmake -S tools/sdk -B target/sdk-production/superbuild -G Ninja -DQT_PROVISION_DIR=<共享 Qt 缓存>` → `cmake --build`（macOS arm64，Apple Silicon 全并行） | 全程约 13 分钟：clone ≈2min、configure ≈1min、编译+安装 ≈10min。安装树 `out/vtk/9.7.0/macos-arm64/`（295MB）：bin/include/lib/share/licenses + `panta-sdk.json`；Qt 模块 dylib 齐备（GUISupportQt/GUISupportQtQuick/GUISupportQtSQL/RenderingQt/ViewsQt）。**构建级证实 VTK 9.7.0 × Qt 6.11.2 可产出 GUISupportQtQuick**（窗口运行时行为仍归 007） |
| 2026-09-17 | 自检 `cmake -S tools/sdk/selfcheck -B … -DPSDK_ROOT=<安装树> -DPSDK_PACKAGE=VTK -DPSDK_REQUIRED_TARGETS="VTK::GUISupportQtQuick VTK::RenderingQt" -DCMAKE_PREFIX_PATH=<Qt staging>` | 通过。两轮教训记档：`cmake -P` 脚本模式无法执行 `add_library(IMPORTED)`，VTK config 加载半途而断且脚本仍退出 0（假阳性）——自检必须是真实 configure 工程；VTK config 会调用 FindThreads 等编译探测，工程需启用 CXX。必需 target 以 `VTK::` 命名空间断言 |
| 2026-09-17 | 打包 `cmake -DPKG_ROOT=<安装树> … -P tools/sdk/package.cmake` | `vtk-9.7.0-macos-arm64.tar.gz`（平铺布局，56MB），SHA256 `0cc143dc6545d96f25d537b4ee31f76f4d6e3cc7dfb147bc205c7fdd1e1b6a0f`，与 `.sha256` 文件一致 |
| 2026-09-17 | 增量二本地回归：vtk.cmake 去除子工程硬编码 Ninja（继承外层生成器）后 `cmake -S tools/sdk …` 重配 + `cmake --build`；`python3 -c "yaml.safe_load(...)"` 校验 `sdk-vtk.yml` 与 `ci.yml` | 重配/幂等构建通过（已产出的 stamp 不重编）；两个 workflow YAML 解析通过。dispatch 级验证（三平台真实生产/发布）待推送后执行 |
| 2026-09-18 | 用户 dispatch 实测：workflow run [35242622228](https://github.com/Yuki-Nagori/panta/actions/runs/35242622228)（三平台，1h58m） | 旧版 Release `sdk-vtk-9.7.0` 落地；仅满足 `GUISupportQtQuick`/`RenderingQt` 原型，保留为历史验证证据，不作为当前 WebGPU 硬件窗口路线消费输入 |
| 2026-09-19 | WebGPU hardware-window 制品配置更新 | `tools/sdk/vtk.cmake` 改为 `VTK_ENABLE_WEBGPU=ON`、`RenderingWebGPU` + `RenderingUI`、Qt 关闭；workflow 自检目标与新 Release tag 改为 `sdk-vtk-9.7.0-webgpu`，待维护者 dispatch 生产 |
| 2026-09-19 | VTK SDK workflow 首次 WebGPU dispatch 失败（run [35447552846](https://github.com/Yuki-Nagori/panta/actions/runs/35447552846)） | macOS/Linux 在 VTK configure 阶段报 `Could not find the Dawn external dependency`；根因是 WebGPU 构建描述缺少 `Dawn_DIR`。Windows 旧 workflow 仍进入 `netgen_sdk` 的 MSBuild，人工取消前未进入 VTK configure。已按 VTK 9.7.0 上游固定 Dawn 资产补充下载、SHA256 校验、安装树合并和 provenance，并让 VTK workflow 不再编译无关的 OCCT/Netgen；待新 commit 重新 dispatch 验证三平台 |
| 2026-09-19 | 本地 review：VTK-only、OCCT/Netgen-only、全禁用三种 `cmake -S tools/sdk -B … -G Ninja` 配置；VTK `cmake --build … --target help`；Dawn 下载/安装脚本 tar.gz 两种根目录 fixture；三个 workflow YAML 与 `git diff --check` | 全部通过；fixture 覆盖 Dawn 归档 SHA256、解包、`DawnConfig.cmake`、include/lib/bin/share 合并及许可证复制。提交前 pre-commit 的 cargo format/lint、cmake-format/lint、QML 格式和 qmllint 全部通过；真实 VTK 三平台生产待新 commit dispatch |
| 2026-09-19 | 新 VTK CI 构建图在 `ninja` 阶段失败：`dawn_runtime`, needed by `vtk_sdk-metadata`, missing and no known rule | 根因是 `ExternalProject_Add_StepDependencies` 把 step 名当成了独立 Ninja target；改为 metadata 的 `DEPENDEES install dawn_runtime`。本地新目录 `cmake -S tools/sdk … -G Ninja` + `ninja -n vtk_sdk` 通过，展开顺序为 `install → dawn_runtime → metadata` |
| 2026-09-19 | `fb9a10b` 的 VTK CI 在 Windows 源码下载阶段失败 | `gitlab.kitware.com:443` 连续三次 clone 超时，尚未进入 Dawn/VTK configure；GitHub VTK mirror 的 `refs/tags/v9.7.0^{}` 已复核为同一 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`。构建描述切换至 GitHub mirror；待新 commit 验证源码和 Dawn 资产下载 |
| 2026-09-19 | GitHub-only 依赖切换 | Dawn 改用 GitHub `google/dawn` native Release `v20260720.160313`（三平台 Release 资产、固定 commit、LICENSE URL/SHA256）；VTK 与 Dawn 的 active 下载和 provenance 均不再依赖 GitLab，待 CI 验证 VTK 9.7 与该 Dawn native package 的 configure/build 兼容性 |
| 2026-09-19 | GitHub 依赖本地回归：真实 Dawn native archive/license fixture；VTK 9.7.0 GitHub 源码 + Dawn native package configure | Dawn 下载、SHA256、解包、许可证补齐和安装树合并通过；VTK configure 通过，`Dawn_DIR` 成功解析 `dawn::webgpu_dawn`。当前未在本机完成 VTK 全量编译，三平台 CI 仍需验证源码下载与 build |
| 2026-09-19 | Ubuntu VTK CI run [35451808688](https://github.com/Yuki-Nagori/panta/actions/runs/35451808688) 在 Dawn step 失败；本地修复回归 | Ubuntu Dawn native archive 使用 `lib64/cmake/Dawn/DawnConfig.cmake`，而下载脚本只检查统一路径 `lib/cmake/Dawn`；归档下载与 SHA256 已通过，VTK 尚未 configure。解包后对单独的 `lib64` 归一化为 `lib`；真实 Ubuntu Dawn archive 下载/校验/安装 fixture、superbuild configure、`ninja -n vtk_sdk` 和 JSON/diff 检查通过，待 CI 重跑验证三平台 |
| 2026-09-19 | `a7d9cca` 后续 Ubuntu CI run [35452752611](https://github.com/Yuki-Nagori/panta/actions/runs/35452752611) 在 VTK configure 失败；本地修复回归 | Dawn 归档的 `DawnTargets-release.cmake` 仍引用 `${_IMPORT_PREFIX}/lib64/libwebgpu_dawn.a`；仅移动目录导致 imported target 指向不存在的旧路径。现已在 `lib64 → lib` 归一化时同步改写 Dawn 包内 `${_IMPORT_PREFIX}/lib64/` 相对前缀；真实 Ubuntu Dawn 包、VTK configure、macOS/Windows Dawn fixture、格式/lint 均通过 |
| 2026-09-19 | 端到端 review：VTK 安装可搬迁性与 Dawn 消费依赖 | 发现 VTK relocatable install 不能只依赖上游默认值，且 selfcheck 未把制品根加入 `CMAKE_PREFIX_PATH`，可能出现生产 configure 通过而安装树消费时 `find_package(Dawn)` 失败；现显式冻结 `VTK_RELOCATABLE_INSTALL=ON`、selfcheck 注入制品根并断言 `dawn::webgpu_dawn`。模拟安装树 selfcheck、superbuild configure/dry-run、metadata JSON、workflow YAML、格式/lint 均通过 |
| 2026-09-20 | VTK CI run [35453562309](https://github.com/Yuki-Nagori/panta/actions/runs/35453562309)：macOS job `105924788622`、Ubuntu job `105924788655` | 两个平台均在 `RenderingWebGPU` 编译阶段失败；Ubuntu 日志明确显示 VTK 生成的命令仍带 `-std=c++17`，随后 Dawn `webgpu_cpp.h` 报 `std::span`、`requires`、dependent `typename` 等 C++20 错误，macOS 报同类错误。根因是当前 GitHub Dawn native 资产已要求 C++20，而 VTK 9.7 的 `RenderingWebGPU` 目标只声明 `PUBLIC cxx_std_17`；单独设置全局 `CMAKE_CXX_STANDARD=20` 仍会被 VTK 目标级 feature 降回 C++17，现通过 `CMAKE_PROJECT_VTK_INCLUDE` 延迟覆盖该目标为 C++20，并在子工程中显式固定 C++20/required/no extensions。真实 VTK 本地生成命令已从 `-std=c++17` 变为 `-std=c++20`，待 CI 重跑 |
| 2026-09-20 | 同一 VTK CI run [35453562309](https://github.com/Yuki-Nagori/panta/actions/runs/35453562309) 的 Windows job `105924788412`（`gh run view 35453562309 --job 105924788412 --log`） | Windows 在 `dawn/include/dawn/webgpu_cpp.h` 编译阶段报多组 MSVC `C2989`：`wgpu::ExternalTextureBindingLayout`、`Future`、`InstanceLimits`、`MemoryHeapInfo`、`MultisampleState`、`Origin2D` 等类型“class template has already been declared as a non-class template”，随后 MSB8066 退出。该错误与三平台使用 Dawn `v20260720.160313` 的版本/API/头文件契约不匹配同属依赖问题，不是 Windows 网络失败；当前改用 VTK 官方指定 Dawn source tag `v20260421.125655`，并以统一 RTTI/C++20 配置重建 |
| 2026-09-20 | 本地 macOS arm64 真实 VTK 9.7.0 `RenderingWebGPU` 全量编译（C++20 hook + GitHub Dawn `v20260720.160313`） | C++20 标准修复生效，1,797 个编译步骤完成且不再出现 `std::span`/`requires` 语法错误；最后 2 个 Dawn 回调模板实例化错误暴露 VTK 9.7.0 与该 Dawn 版本的 API 不匹配。VTK 自带 `Rendering/WebGPU/README.md` 明确要求 Dawn `v20260421.125655`，下一步将依此固定 GitHub Dawn 资产后重跑完整构建 |
| 2026-09-20 | 对照 VTK 9.7 官方 WebGPU 文档与本地源码 review | 官方固定 Dawn `v20260421.125655`，支持 GitHub 源码构建；VTK 的 `DawnMemoryDump` 派生类需要 RTTI，而 GitHub native release 默认按 Dawn `DAWN_ENABLE_RTTI=OFF` 构建，导致静态链接缺 `typeinfo for dawn::native::MemoryDump`。删除不稳妥的 RTTI 假 anchor，改为固定 GitHub Dawn 源码构建并显式 `DAWN_ENABLE_RTTI=ON`；同时关闭 Dawn 无关测试、工具、protobuf/IR 构建，待完整 VTK 链接与安装验证 |
| 2026-09-20 | VTK 9.7 Linux Wayland 官方文档/源码核对 | VTK 9.7 已提供原生 `vtkWaylandHardwareWindow`、`vtkWaylandRenderWindowInteractor` 和 WebGPU Wayland surface；`VTK_USE_Wayland` 仅在 `VTK_USE_X=OFF` 时启用，当前构建描述若沿用默认值会实际产出 X11 路径。Linux WebGPU SDK 改为显式 `VTK_USE_X=OFF`、`VTK_USE_Wayland=ON`，CI 补 `libwayland-dev`（含 `wayland-scanner`）、`wayland-protocols` 和 `libxkbcommon-dev`，X11 不再作为该 SDK 的编译后端；Qt Quick 与 Wayland surface 的嵌入/事件协调仍由 007 的应用集成阶段验证。参考 [VTK 9.7 Wayland/WebGPU 架构说明](https://docs.vtk.org/en/latest/release_details/9.7/hardware-windows-and-wayland.html) 与 [VTK 9.7 build settings](https://docs.vtk.org/en/v9.7.0/build_instructions/build_settings.html) |
| 2026-09-20 | 本地 Dawn source tag `v20260421.125655` + `DAWN_ENABLE_RTTI=ON` 构建/安装；VTK 9.7.0 macOS arm64 WebGPU target 与完整 install；自检/打包 | 通过：Dawn `webgpu_dawn` 734/734 编译，`nm -C` 可见 `typeinfo/vtable for dawn::native::MemoryDump`；VTK `vtkRenderingWebGPU` 编译链接通过，完整 VTK install 通过；合并 Dawn 运行时与许可证后，真实 selfcheck 找到 `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn`，归档内含 `DawnConfig.cmake`、`libwebgpu_dawn.a`、两套许可证与 `panta-sdk.json` |
| 2026-09-20 | VTK 9.7.0 WebGPU 最小模块集 review：新目录配置 `VTK_GROUP_ENABLE_{Rendering,StandAlone,Imaging,Parallel,Views,Web}=DONT_WANT`、`VTK_ENABLE_WRAPPING=OFF`，直接启用 `RenderingUI`/`RenderingWebGPU`，保留平台 hardware window 与 WebGPU 依赖 | 通过：配置和 macOS arm64 直接目标编译均成功，1818/1818 步骤完成；目标仍包含 `RenderingCore`、必要 Filters、FreeType 等传递依赖，但不再启用无关 OpenGL/Qt/Views/Parallel/Web group 或 C++ wrapping。关闭 wrapping 后完整 install、自检和打包均通过；临时归档未包含 OpenGL、Qt 或 wrapping 库 |
| 2026-09-20 | 仓库级回归：`cargo test --locked --workspace` | 通过：Rust 单元/集成测试全部通过；native_and_qml_suite 的 CTest `29/29` 全部通过，其中包含 crash 日志、FFI、路径、QML 和任务服务测试 |
| 2026-09-19 | `.githooks/pre-commit` | 未通过：仓库聚合 formatter 在 macOS 环境调用 `uv` 时触发 `system-configuration` 动态存储 NULL object panic，尚未进入本次代码 lint；独立 `cargo fmt --check`、`cmake-format --check`、`cmake-lint`、JSON/CMake 图/diff 检查通过 |
| 2026-09-18 | 闭环验证（031 侧）：manifest 按发布资产登记后，macOS 生产 consumer 从 Release 真实下载消费，`find_package(VTK CONFIG)` + required-target 自检通过，离线二跑零下载；ctest 27/27 | 通过。VTK 从生产到消费全链路闭环 |
| 2026-09-18 | macOS 本机生产 occt→netgen（Apple Silicon 全并行，OCCT 编译约 18 分钟）：`cmake -S tools/sdk -B … -DQT_PROVISION_DIR=<Qt 缓存>` → `cmake --build … --target occt_sdk netgen_sdk` | 通过，经三轮实证修正：① Netgen 必须显式指向 OCCT **安装树** config（`OpenCASCADE_DIR=<prefix>/lib/cmake/opencascade`），否则撞上构建树 config 而 `*Targets.cmake` 缺失失败；② ExternalProject 的 CMAKE_ARGS 变更需父工程重 configure 才生效；③ Netgen 在 Apple 把安装前缀 FORCE 成 `.app` bundle——经其 `INSTALL_DIR` 变量接管并把 `NG_INSTALL_DIR_*`（CACHE）压平为标准 Unix 布局。产物：OCCT 安装树 111MB（归档 28MB，SHA256 `6175fae77f8ca385…`）、Netgen 9.5MB（归档 3.2MB，SHA256 `3440438b7f5bcb58…`） |
| 2026-09-18 | 制品自检与打包：selfcheck `OpenCASCADE`（`TKernel TKDESTEP`，OCCT 8 targets 无命名空间）、`netgen`（`ngcore nglib`，无命名空间）；package 平铺归档 | 双双通过；LGPL-2.1+例外（OCCT）/LICENSE（Netgen）随包，`panta-sdk.json` 齐。Netgen 的 `libnglib.dylib` 链接同管线 OCCT（同工具链，ABI 约束满足）；运行期 rpath 跨目录解析留待 009/010 消费侧处理（已记录）。workflow YAML 解析通过（维护者后续决策：VTK 独立管线不与 OCCT/Netgen 绑定；OCCT/Netgen 合并为一个管线一个 Release） |
| 2026-09-18 | 首次 dispatch 失败诊断与修复（runs 35304311839/35304323682：Linux/Windows OCCT configure 终止于 Freetype 头文件缺失） | 根因：OCCT 在 Linux/Windows 默认启用 Freetype（Linux 另有 Xlib），configure 检查到"used but not found"即终止；macOS 未默认启用故本机实证未暴露。修复 `USE_FREETYPE=OFF`/`USE_XLIB=OFF`（Netgen 上游 SuperBuild 同传，证明可关）。本地复测按维护者指示跳过，以修复后 dispatch 作为验证；推送前需 dispatch `SDK artifacts (OCCT+Netgen)`（publish 可先 false 验证三平台，再 true 发布） |
| 2026-09-18 | 二次 dispatch（run [35304961896](https://github.com/Yuki-Nagori/panta/actions/runs/35304961896)）：macOS/Windows success（Freetype/Xlib 修复生效），Ubuntu 在 Netgen 自检失败 | 根因：Netgen 安装的包配置文件名为 `NetgenConfig.cmake`（大写 N，CI 日志证实），而 `find_package(netgen)` 只检索小写变体——macOS/Windows FS 大小写不敏感故通过，Ubuntu ext4 敏感即失败。修复：自检包名改 `Netgen`（本地最小工程复现 + CI 日志双重验证）。**约束登记**：031 manifest 的 occt/netgen 条目 PACKAGE 登记为 `Netgen`；010 消费侧 `find_package(Netgen)`；OCCT 侧 `OpenCASCADE` 全大写无此问题。待三次推送后重 dispatch 验证 |
| 2026-09-18 | OCCT/Netgen 管线零 Qt 化：tools/sdk 顶层把 qt-provision include 与 VTK 描述一起包进新开关 `PANTA_SDK_ENABLE_VTK`（默认 ON，sdk-vtk.yml 行为不变；include 在 configure 期执行，单纯挪文件无用——本地验证发现后改为开关方案）；sdk-occt-netgen.yml 传 OFF。配置级回归：OFF 模式 0 次 Qt 下载/解包，默认模式正常供给（命中缓存） | 通过；该管线 dispatch 不再白下 1.7GB Qt |

## 风险与回退

构建器差异、许可证遗漏、Netgen/OCCT ABI 不匹配或 VTK QtQuick 模块缺失会产生不可用 SDK。发布前 package 自检失败即阻止上传；升级保留上一 manifest 和归档，供给脚本只切换到完整校验通过的版本。

## 决策与工作记录

- 2026-09-17：由 031 官方资产盘点拆分；上游缺少全平台 C++ SDK 时，采用受信 CI 生成一次、开发者复用的制品路线。任务不授权普通本地构建源码。
- 2026-09-17（增量一，已实施）：建立可审计构建描述 `tools/sdk/`——CMake superbuild（ExternalProject）+ 自检/打包脚本，首个目标 VTK 9.7.0。源码固定 tag `v9.7.0` 解引用 commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`（git ls-remote 复核）；原制品曾启用 Qt/GUISupportQtQuick，现已由 007 决策替换为 WebGPU hardware-window 配置。安装布局含 `panta-sdk.json` 与 `share/licenses/`；开发机生产仅为构建描述验证与 007 前置证据，不改变"受信 CI 生产、开发者只下载"的目标形态。
- 2026-09-17（历史增量二，CI 生产管线）：新增 `.github/workflows/sdk-vtk.yml`（workflow_dispatch，三平台矩阵：macos-latest/Ninja、ubuntu-latest/Ninja + Wayland 开发包、windows-2022/VS17-x64——生成器经 CMAKE_GENERATOR 继承进 ExternalProject 子构建，vtk.cmake 不再硬编码 Ninja 并显式 `--config Release` 应对多配置）。旧流程曾包含 qt-provision；现行 WebGPU hardware-window 流程不消费 Qt，直接生产 → selfcheck 真实 configure 消费 → package → upload-artifact；`publish` 输入开启时 `gh release` 幂等发布。工具链取 runner 预装 cmake/ninja（受信生产环境，非开发者基线）。旧版 dispatch 证据见验证表。
- 2026-09-18（增量三，OCCT/Netgen 构建描述）：维护者决策——OCCT 放弃消费官方 Windows SDK，三平台统一走本管线自托管（官方仅 Windows 有归档，跨平台工具链不一致）。`tools/sdk/occt.cmake`：tag V8.0.1 → commit `b8f597c677811d1f9f4d8a97f5ae2825c0353a42`（ls-remote 复核），Release/Shared，模块 Draw/Visualization/DETools 关闭（渲染归 VTK，免除 Tcl/freetype），保留 DataExchange（STEP）；许可证 LGPL-2.1+例外随包。`tools/sdk/netgen.cmake`：tag v6.2.2604 → commit `3ee489c7d58fdbc2a6708cca3cbaefaae506dc17`，USE_OCC=ON 链接同管线 OCCT（同 triple 同工具链，满足 Netgen↔OCCT 匹配约束），GUI/Python/MPI 关闭；依赖 occt_sdk ExternalProject。031 manifest 的 occt/netgen 登记及 Windows 官方 SDK 条目替换，待 Release 落地后按实际 URL/SHA256 回填。
- 2026-09-18（增量四，CI 失败修复与管线合并）：首次 dispatch（run 35304311839/35304323682）Linux/Windows 同因失败——OCCT 在两平台默认启用 Freetype（Linux 另有 Xlib），缺系统头文件即 configure 终止（macOS 未默认启用故本机未暴露）。修复：occt.cmake 显式 `USE_FREETYPE=OFF`/`USE_XLIB=OFF`（渲染归 VTK 后无用；Netgen 上游 SuperBuild 构建 OCCT 时同传 OFF，制品保持零系统第三方依赖）。维护者决策：Netgen↔OCCT 为硬 ABI 锁定，**合并为一个管线一个 Release**——sdk-occt.yml/sdk-netgen.yml 合并为 sdk-occt-netgen.yml（tag `sdk-occt-netgen-8.0.1-6.2.2604`，6 资产=3 平台×2 依赖），发布说明合并为 occt-netgen-8.0.1-6.2.2604.md，升级成对进行。031 manifest 的 occt/netgen 登记及 Windows 官方 SDK 条目替换，待 Release 落地后按实际 URL/SHA256 回填。

## 完成摘要

未完成（保持 in-progress）。旧 VTK Qt/OpenGL Release 已完成历史生产/消费闭环；现行 VTK 构建描述已切换到 WebGPU hardware-window（Linux Wayland、macOS Cocoa、Windows Win32），待维护者 dispatch `sdk-vtk-9.7.0-webgpu` 并把真实 URL/SHA256/targets 回填 031 manifest。OCCT/Netgen 合并 workflow 经 macOS 本机生产、自检与打包实证，仍需按既有计划完成其余交付证据。剩余：新 VTK 制品三平台发布与消费、OCCT/Netgen 三平台 Release/manifest 完整证据、SBOM/provenance 自动化、007/009/010 的真实链接/运行冒烟、Linux glibc 有效基线回写。
