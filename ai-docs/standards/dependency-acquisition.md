# 依赖获取与主平台环境

查阅 / 决策日期：2026-09-16。状态：主平台与固定清单已由任务 002 记录，同日按维护者要求升级至各上游最新稳定版并核对依赖间版本关系；CI 三平台矩阵由任务 018 建立。Cargo→CMake 调度已由任务 004 接通；工具二进制供给拆分为任务 020。Qt 已验证采用预编译包，VTK/OCCT/Netgen 的应用接入必须优先消费预编译 SDK，统一供给任务见 [031](../task/031-prebuilt-native-dependencies.md)。版本与兼容性结论的维护规则见 [技术基线](baseline.md)，本文只负责可复核的获取、重建、升级与回退步骤。

## 托管原则（2026-09-16 决策）

- Cargo 是唯一依赖入口：`cargo build` 触发的构建引导负责把固定的预编译 CMake、Ninja、Qt、VTK、OCCT、Netgen 包缓存到工作区构建树（根 `target/` 下托管目录），不写入源码树。正常开发构建不得把第三方源码加入本地构建图。
- 预编译优先：优先使用上游或项目发布的、与目标平台、架构、编译器 ABI、Qt 版本和所需模块匹配的 SDK/二进制包。只有不存在合适的预编译包时，才可另立任务评估由 CI/维护者生成可复用制品；本地构建源码不是默认回退，也不能静默触发。
- 包不可用或匹配检查失败时立即给出版本、平台、架构、ABI、缺失 target 和获取入口诊断；不能退回系统库或偷偷开始长时间源码编译。
- 不把系统包管理器（Homebrew/MacPorts/apt 等）作为项目基线；个人机器上已安装的同名包只是开发便利，不能当作兼容性证据，也不得进入提交的构建配置。
- 开发者前置：git、rustup（工具链由根 [rust-toolchain.toml](../../rust-toolchain.toml) 固定，缺失时 `rustup toolchain install`）与所在平台的 SDK/运行库（macOS：Apple 命令行工具和 SDK；Linux：glibc/sysroot、C 编译器（Rust 链接驱动）与 OpenGL 前置——供给 Qt 的 `find_package(Qt6 Gui)` 经 `WrapOpenGL` 强依赖宿主 GL 开发文件，运行 QML 另需 libEGL/libxkbcommon 运行库与软件渲染驱动，Debian/Ubuntu 对应 `libgl-dev`、`libegl1`、`libxkbcommon0`、`libgl1-mesa-dri`，CI 由 ci.yml 平台前置步骤安装；Windows：MSVC Build Tools、Windows SDK 和 CRT）。自有 C++ 编译器由 Cargo 托管的 LLVM 22.1.7 供给，不依赖 PATH 中的 Apple Clang、GCC 或 cl.exe。macOS 上 Qt 源码构建另需系统自带 `/usr/bin/perl`。首次构建需要联网，此后可离线重复构建；跨机器缓存共享由任务 012 统筹。
- 版本固定：源码 tag 附 commit SHA 校验；下载的预编译包附 SHA256，并记录目标平台、架构、编译器/运行库 ABI、Qt 兼容范围和模块清单。升级必须修改固定清单、完成对应集成验证后同一 commit 提交。
- 选版原则（2026-09-16 维护者决策）：尽量采用各上游最新稳定版，同时核对依赖间版本关系；受关系约束不能升级时，记录原因与重评条件。

## 主验证平台（002 记录）

| 项目 | 值 |
|---|---|
| 操作系统 | macOS 26.3.1，arm64（Apple Silicon） |
| 编译器 | Apple clang 17.0.0（/Library/Developer/CommandLineTools，含 macOS 26.2 SDK） |
| C++ 标准库 | libc++（随工具链） |
| Perl | /usr/bin/perl 5.34.1（Qt syncqt 需要） |
| CMake / Ninja | 不要求预装；由引导按下方清单供给 |

当前工作目录所在机器不自动构成所有发布平台的承诺。CI 在 macOS/Linux/Windows 三平台验证 Rust 层（见下文 CI 一节）；native 依赖的跨平台矩阵随对应构建任务扩展。

## 固定清单（候选；"集成验证"列对应任务通过前不视为实测通过）

| 依赖 | 固定版本（tag） | 来源 | 校验 | 获取形态 | 关键候选选项 | CMake package / targets | 许可证入口 | 集成验证 |
|---|---|---|---|---|---|---|---|---|
| CMake | 4.4.3 | `github.com/Kitware/CMake` release 资产：macOS `cmake-4.4.3-macos-universal.tar.gz`（universal，SHA256 `0c5d65251c14cc884bfa16bdbed3c263ce5bffe2e21c0d0d00962cb0610464fa`）、Linux x86_64 `cmake-4.4.3-linux-x86_64.tar.gz`（`d6c83076c575bc00b823522ac974bda66d0af05d6ddc30e739c12385cf32c6cc`）、Windows x86_64 `cmake-4.4.3-windows-x86_64.zip`（`4d52ebab7193a698651639ed80d8d04fd903358843572cf44c7fd234cb7c26ab`）；SHA256 为 2026-09-17 下载实测（020） | 官方二进制解包（020）：默认托管缓存/下载并校验；不支持固定资产的平台才允许显式旁路；解包用平台自带 `tar` | — | 可执行工具（非 CMake 包） | BSD-3（发布包 Copyright.txt） | 003/004/020 |
| Ninja | 1.13.2 | `github.com/ninja-build/ninja` release 资产：`ninja-mac.zip`（SHA256 `c99048673aa765960a99cf10c6ddb9f1fad506099ff0a0e137ad8960a88f321b`）、`ninja-linux.zip`（`5749cbc4e668273514150a80e387a957f933c6ed3f5f11e03fb30955e2bbead6`）、`ninja-win.zip`（`07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65`）；SHA256 为 2026-09-17 下载实测（020） | 预编译 release 资产（020）；解包用 `cmake -E tar`（libarchive，三平台零额外工具） | — | 可执行工具 | Apache-2.0（仓库 COPYING） | 003/004/020 |
| LLVM/Clang | 22.1.7 | `github.com/llvm/llvm-project` release 资产：macOS ARM64 `LLVM-22.1.7-macOS-ARM64.tar.xz`（SHA256 `4177245188b0a30a6539c96b361dea56f253485756bfd8927a6a59e7301e7806`）、Linux x86_64 `LLVM-22.1.7-Linux-X64.tar.xz`（`edb0522b41e261819c06ea437d249f9b8acfa413d3805bc9920eec6fb76ff830`）、Windows x64 `LLVM-22.1.7-win64.exe`（`e091fcf965ce589c83c0f7c5356b2fcf3e658a8ec990bfcf79cce4389a0d1eb3`） | Cargo build.rs 下载到 `target/panta-tools/llvm` 并校验；Unix 解包归档，Windows 通过官方 NSIS 安装包静默安装；Unix 使用 clang/clang++，Windows 使用 clang-cl；clang-format/clang-tidy 与编译器来自同一目录 | — | Apache-2.0 with LLVM exceptions（发布包 LICENSE） | 042 |
| Qt | 6.11.2（qtbase + qtdeclarative + qttools + qtsvg 预编译包，含 Qml/Quick/QuickControls2/Test、qttools 的 lrelease/lupdate 与 qtsvg 的 qsvg 图像格式插件（029 图标资源）；qt5compat 等未取，需要时按 task 扩展） | `download.qt.io/online/qtsdkrepository/{mac_x64,linux_x64,windows_x86}/desktop/qt6_6112/`（与在线安装器同源、免账号；三端 URL 与归档名见 `native/cmake/qt-provision.cmake`，以脚本为准） | SHA256（2026-09-16/17 下载实测，硬编码于供给脚本并强校验） | 预编译 7z 解包（`cmake -E tar`，三平台零额外工具）；维护者决策（2026-09-16）：不源码构建。qttools 归档（034，SHA256 于 2026-09-17 实测）提供锁定 `lrelease` 供 QM 编译，不使用系统 Linguist。qtsvg 归档（029，SHA256 于 2026-09-22 下载实测，并经官方 .meta4 交叉核对）提供 `plugins/imageformats/qsvg` 供 Image 渲染 SVG 图标资源 | 供给缓存于 `target/panta-deps/qt/staging`（任务 041：跨 profile、Cargo 与 presets 共享；每归档解包指纹在 `qt/extracted/`，升级只补新归档）；安装版布局为平铺 bin/lib | `find_package(Qt6 6.11 COMPONENTS Core Gui Qml Quick QuickControls2 Test)`；`bin/lrelease` 由 `native/i18n` 消费 | LGPL-3.0（发布包 LICENSES/） | 005/034 |
| Qt Linux ICU runtime | 73.2（与 Qt 6.11.2 Linux 工具 ABI 匹配） | Qt 官方 qtsdkrepository `6.11.2-0-202608131018icu-linux-Rhel8.6-x86_64.7z` | SHA256 `111bdae30a66fff6ef65620e95766170fa5a6f425c360ea33b79cb2ec7e2fd86` | 预编译 7z 解包至 `qt/staging/lib`；仅 Linux 使用，不使用系统 ICU 或源码构建 | `rcc`、`qtpaths`、`qmlimportscanner` 运行时依赖 | ICU 许可证随归档 | 005/036 |
| VTK | 9.7.0（commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`，tag `v9.7.0` 解引用，unmodified 上游源码） | 038 受信 CI 制品已发布（Release `sdk-vtk-9.7.0-webgpu`，2026-09-20 三平台生产全绿）：macOS `vtk-9.7.0-macos-arm64.tar.gz`（SHA256 `191f93371d6129780ff7c1363faf860c5e2bb279989ebe1d66e3ed0b5b932114`）、Linux `vtk-9.7.0-linux-x86_64.tar.gz`（`01a84e97d35b0a0f1ae443bea7215e18ed0617e7e186277f384cc139a4011602`）、Windows `vtk-9.7.0-windows-x86_64.tar.gz`（`771877f2c8cb170131d863c7791b450ab7c8f09306eaf1f328445dcd5fad3c78`）；完整 URL 见 031 sdk-provision.cmake manifest，合规说明见 `tools/sdk/releases/vtk-9.7.0.md` | 预编译归档 SHA256（2026-09-20 Release `.sha256` 实测，manifest 强校验）；源码 tag/commit 作为来源依据 | 预编译制品消费（031 `panta_require_sdk(vtk)` 三平台登记；macOS 真实 consumer 烟测、native configure/build 与离线复用通过）。WebGPU/共享库；Linux 由 ubuntu-24.04 gcc13 生产，Wayland-only，glibc 与三平台运行冒烟由 007 回写 | `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn`；原生窗口分别为 Cocoa、Wayland、Win32 | `find_package(VTK CONFIG)` 与 `find_package(Dawn CONFIG)`；`VTK::`/`dawn::` targets | BSD-3（VTK）与 Dawn 上游许可证，均随包 | 031/038（configure/package 级）；007（窗口级） |
| OCCT | 8.0.1（tag `V8.0.1`，commit `b8f597c677811d1f9f4d8a97f5ae2825c0353a42`，2026-07-30，当前最新 release） | 038 受信 CI 制品已发布（Release `sdk-occt-netgen-8.0.1-6.2.2604`，2026-09-18 三平台生产全绿；维护者决策：官方仅 Windows 有归档、跨平台工具链不一致，三平台统一自托管，官方归档不再消费）：macOS `…/occt-8.0.1-macos-arm64.tar.gz`（SHA256 `db6d4a878cc3f1c4ccf693e2d1408c35b10fa844b9793a02c38379bcbc157161`）、Linux `…/occt-8.0.1-linux-x86_64.tar.gz`（`04a33d7a5aa1c122da8fb0ec775fffb5f8872a90db717b800e7561d52a7cf563`）、Windows `…/occt-8.0.1-windows-x86_64.tar.gz`（`d0162ff98100741f63d6f4103e8c98d6b7e4c7fa32c6ff4ba9ad570c66634751`）；完整 URL 见 031 sdk-provision.cmake manifest，配对约束见"依赖间版本关系" | 预编译归档 SHA256（2026-09-18 Release `.sha256` 实测，manifest 强校验）；tag SHA 作为来源依据 | 预编译制品消费（031 manifest 三平台登记；macOS 生产 consumer 烟测通过）。Release/Shared；模块 Draw/Visualization/DETools 与 USE_FREETYPE/USE_XLIB 关闭（渲染归 VTK、零系统第三方依赖），其余默认启用（含 STEP 所需 DataExchange）；模块进一步裁剪由 009 定 | SDK 必须提供 `find_package(OpenCASCADE)`（已验证：targets 无命名空间，`TKernel`/`TKDESTEP` 等） | `find_package(OpenCASCADE CONFIG)` | LGPL-2.1 + OCCT 例外（随包 `share/licenses/OCCT/`） | 031/038（configure 级）；009（STEP 集成级，2026-09-20 三平台 CI 通过：geometry 模块 imported targets 链接、单位/摘要冒烟测试全绿；Windows DLL 位于嵌套 `win64/vc14/bin/`，测试运行经 `native_test_env` 递归收集解析） |
| Netgen | v6.2.2604（commit `3ee489c7d58fdbc2a6708cca3cbaefaae506dc17`，当前最新 release） | 038 受信 CI 制品已发布（与 OCCT 同 Release `sdk-occt-netgen-8.0.1-6.2.2604` 成对发布，2026-09-18 三平台生产全绿）：macOS `…/netgen-6.2.2604-macos-arm64.tar.gz`（SHA256 `51d067f057143044fb8feb8501f47973832c92a359281833ff7be996018f384e`）、Linux `…/netgen-6.2.2604-linux-x86_64.tar.gz`（`9be1ac3a2d8f16bc86c2c52d51c7aab821aaeee9848e2c3b85d55bea6eb4079a`）、Windows `…/netgen-6.2.2604-windows-x86_64.tar.gz`（`3e8c5204fc1977c4ce4ff53e66cb32c4a2092e408b45ad7b5da173922ce9fd1d`）；完整 URL 见 031 manifest，配对约束见"依赖间版本关系" | 预编译归档 SHA256（2026-09-18 Release `.sha256` 实测，manifest 强校验）；tag SHA 作为来源依据 | 预编译制品消费（031 manifest 三平台登记；macOS 生产 consumer 烟测通过）。`USE_OCC=ON` 链接同 Release 的 OCCT（成对升级）；GUI/Python/MPI/JPEG/MPEG 关闭；**包配置文件为 `NetgenConfig.cmake`（大写 N）——`find_package` 必须用 `Netgen`**（Linux ext4 大小写敏感，038 实证）；运行期依赖同平台 OCCT 资产（009/010 消费侧处理加载路径） | `ngcore`/`nglib` imported targets（无命名空间，038 自检已验证）；与 OCCT ABI 匹配由成对发布保证 | `find_package(Netgen CONFIG)` | LGPL-2.1（随包 `share/licenses/Netgen/LICENSE`） | 031/038（configure 级）；010（网格集成级） |
| GoogleTest（仅测试） | v1.18.0（上游 commit `063de7e9578f82b369302001269680b4b1553359`） | 上游 `github.com/google/googletest`；仓库 Release `sdk-googletest-1.18.0` | 三平台归档 SHA256（2026-09-24 下载实测并与 `.sha256` sidecar、Release API digest 对照）：macOS arm64 `f1c28c7121cd2b34beaa0fdbec660b32b2e45ba8d252f14073eb93e357c73579`；Linux x86_64 `ae6bf4752d4e95893d81ce316efd0dc37c433b87cad243c87103f7c78bcf948f`；Windows x86_64 `a395b0f227254509f7df1dbd562287556df8d5192c1a940d0cf06e6813c70f96` | 三平台自建静态 SDK（任务 070 workflow run 35972333653）；CMake `panta_require_sdk(googletest)` 仅在 `BUILD_TESTING=ON` 时下载/校验/解包到 `target/panta-deps/sdk/googletest/1.18.0/<triple>`，可由 Cargo 缓存目录共享 | 上游 CMake 安装；C++17、static、`BUILD_GMOCK=OFF`、Windows `gtest_force_shared_crt=ON` | `find_package(GTest CONFIG)`；`GTest::gtest` / `GTest::gtest_main` | BSD-3-Clause：归档内 `share/licenses/GoogleTest/LICENSE` | 019/070/071 |

"关键候选选项"是起点而非决定；实施任务按验证结果调整并回写本表。

## 依赖间版本关系（2026-09-16 核实）

- **Netgen ↔ OCCT**：netgen v6.2.2604 的 OCCT 适配经 `find_package(OpenCASCADE)`，源码内版本守卫为 `NETGEN_OCC_VERSION_AT_LEAST` 风格（如 ≥7.4、≥7.8 走 TKDE 新目标名），对 8.0 为"大于等于"语义，未发现排除 8.x 的守卫或上游 issue；因此采用 OCCT 8.0.1。该组合**未经构建实测**：010 必须用真实几何到网格验证；若不兼容，回退固定 OCCT `V7_9_3`（commit `a016080bf673`）并在此记录原因。**制品层配对（2026-09-18 维护者决策）**：OCCT 不承诺跨版本 C++ ABI 稳定，Netgen 制品只与构建时所链接的 OCCT 兼容，混用会在加载/链接期失败或静默出错；因此两者由 038 同管线、同工具链生产并**合并为一个 Release 成对发布**（`sdk-occt-netgen-<occt>-<netgen>`），升级必须成对重建发布，不允许只换一侧。
- **VTK ↔ Qt**：VTK 9.7.0 WebGPU SDK 不依赖 Qt VTK GUI 模块；Qt 6.11.2 只提供 QML/Quick 宿主，原生 view/layer/surface 由 `src/vtk/` 平台桥接接入。SDK 必须提供 `VTK::RenderingWebGPU`、`VTK::RenderingUI`、`dawn::webgpu_dawn` 和对应 hardware window；007 负责实际窗口、事件与运行时加载验证。没有匹配 SDK 时，先由 031/038 评估项目制品，不在开发机把 VTK 源码构建作为普通消费路径。
- **全体 ↔ CMake 4.4**：清单内项目声明的 CMake 下限均 ≥3.10，高于 CMake 4 移除的 <3.5 兼容线；首次 configure 由 003/004 实测。
- Rust 侧无 native 版本耦合；CXX 的 MSRV（1.88）低于固定工具链 1.98.1，约束见 [Rust 规范](rust.md)。

## CI（018）

- GitHub Actions workflow [ci.yml](../../.github/workflows/ci.yml)：push 到 main 与全部 pull request 触发；矩阵 `macos-latest` / `ubuntu-latest` / `windows-2022`。三平台统一 Ninja；Windows 使用托管 clang-cl、MSVC Build Tools/Windows SDK 环境与 Qt MSVC2022 预编译包；CI 不自行安装项目工具。CMake/Ninja/Qt 由 Cargo 构建或质量入口按需供给；GoogleTest 在 `BUILD_TESTING=ON` 时由 `sdk-provision` 下载并校验对应平台固定 SDK。Linux 上 Qt 预编译包的 configure 与 QML 运行依赖宿主 OpenGL 前置（见"开发者前置"），由 workflow 平台前置步骤以 apt 安装，属宿主能力而非项目依赖。构建引导的 curl 下载带 connect/speed/max 上限，任务级 `timeout-minutes` 兜底传输停滞。
- Qt 6.11.2 Linux 预编译归档面向 RHEL9，Qt 工具（包括 `rcc`、`qtpaths`、`qmlimportscanner`）需要 ICU 73。`native/cmake/qt-provision.cmake` 同步下载并校验 Qt 官方的 ICU 73 预编译归档，解包到 `qt/staging/lib`，让 Ubuntu 使用与 Qt 工具匹配的 ABI；不使用系统 ICU、不伪造 SONAME，也不源码编译 ICU。Linux 仍跳过仅供 IDE 使用的 `.qmlls.build.ini` 和当前 app 的空 import scan，保留 QML typeinfo、cachegen、资源和运行时验证。
- 每个平台安装 rust-toolchain.toml 中的固定 Rust 工具链后执行 workspace check、完整 build、`panta-tests toolchain` 实际路径核验与 `cargo test --locked --workspace`；lint、format、audit 和两类 coverage 在 Ubuntu 独立运行，并按变更路径域触发。
- runner 需要镜像自带的平台编译器与 rustup；平台编译器和标准库属于 Cargo 无法替代的宿主能力，项目依赖仍由 Cargo 驱动的构建引导供给。
- 边界：当前 CI 通过 Cargo 同步验证 Rust 与已有 native/Qt 构建；VTK、OCCT、Netgen 的 SDK 供给与集成仍由 031 及后续任务扩展，测试聚合与质量门禁归 011，依赖缓存归 012。

## 干净重建步骤

当前流程（004 已接通调度；Qt 预编译供给由 005 落地，VTK/OCCT/Netgen 预编译供给由 031 负责，工具二进制由 020 负责）：

1. 按 [README 环境要求](../../README.md#环境要求) 安装平台前置（macOS：Apple CLT；Linux：C 编译器与 OpenGL 前置；Windows：MSVC）与 rustup；CMake/Ninja/Qt/VTK/OCCT/Netgen 由固定预编译供给处理。
2. 在仓库内执行 `rustup toolchain install 1.98.1`（或首次 cargo 命令时按 rustup 提示安装）。
3. `cargo build --locked`：launcher 的 build.rs 调度 CMake/Ninja，消费构建树内固定的预编译依赖（缺失或 ABI 不匹配时立即失败并给出诊断）；`cargo run` 启动 `panta-native` 并转发参数与退出码。VTK/OCCT/Netgen SDK 由 031 manifest 供应用例引擎消费；GoogleTest v1.18.0 也已在 `native/cmake/sdk-provision.cmake` 登记，仅构建测试时从 Release 下载并强校验，统一缓存于 `target/panta-deps/sdk/`。

验证入口见 [构建与开发](../architecture/build-and-development.md)；每项依赖的集成验证任务见固定清单最后一列。

## 升级与回退

- 单个依赖升级：修改本文固定 tag/SHA → 核对与其它依赖的版本关系（尤其 Netgen ↔ OCCT）→ 对应集成验证任务执行最小验证集（几何/网格/视口冒烟）→ 回填 SHA256 与实测结果 → 清单、task 与受影响规范同一 commit 提交。
- 回退：恢复上一版固定清单并按干净重建重建；构建树可整目录删除，不影响源码树。OCCT 组合的既定回退点是 `V7_9_3`。
- Rust 工具链升级规则见 [Rust 规范](rust.md)。

## 已知风险与显式未验证项

- VTK/OCCT/Netgen 的平台 SDK 可能没有统一的官方分发格式；在 031 解决前不得把源码构建塞进普通开发流程。必要的源码生产应在独立 CI/制品任务执行并缓存可复用结果。
- Netgen v6.2.2604 × OCCT 8.0.1 只有源码守卫级证据，构建与网格正确性由 010 实测；这是当前清单中关系最紧的组合。
- Qt Quick（macOS Metal 后端等）与 VTK GL 上下文的交互是 007 的重点风险；本文只固定源码版本，不预支任何兼容结论。
- 二进制 SHA256 在实装时回填，此前不做无产物的形式校验。

## 质量工具供给（042/043）

`panta-build` 是 launcher、FFI 与根质量 runner 共享的支持 crate。LLVM/CMake/Ninja/uv 固定资产按版本和 SHA256 隔离并加锁安装；Cargo 扩展按固定版本安装到 `target/panta-tools/<tool>/<version>`。uv 0.8.22 和 CPython 3.13.7 在仓库内托管；cmakelang 0.6.13、Cppcheck wheel 1.5.1（Cppcheck 2.17.1）由 `uv.lock` 锁定，禁止源码安装回退。Python 工具/解释器许可证沿各分发包保留；Cppcheck 为 GPL-3.0 工具，仅开发/CI 使用，不链接进应用。详细边界与入口见 [质量工具链](../modules/quality-tooling.md)。

CI 缓存只保存 Cargo registry/git 和 `target/panta-tools`、`target/panta-deps` 这类可验证的依赖资产；`target/native` 与 Cargo 编译产物每次在当前 checkout 重新生成，避免旧 compile database、CMakeCache 或增量对象从 restore key 泄漏。缓存键包含平台、架构、LLVM 版本、Cargo/Python/原生供给清单；回退命中只能复用目录内带版本和摘要校验的资产。Windows 的 LLVM 官方安装包及其已校验安装目录因此会随 `target/panta-tools` 缓存复用。

托管工具归档下载以父进程限制含重试的总耗时为 30 分钟，解包或 Windows 安装上限为 20 分钟；超时终止并回收直接子进程，保留临时目录供下次持锁清理，不发布 `.complete`。安装锁、归档检查、摘要校验、解包和发布均有阶段日志；长时间运行的下载、解包或安装每 30 秒报告耗时。

上述整体上限不能仅用 curl `--max-time` 替代：该计时在每次重试时重置，`--retry-max-time` 也不截断已经开始的传输。机制依据：[curl 手册](https://curl.se/docs/manpage.html#--max-time)、[Cargo build script 输出](https://doc.rust-lang.org/cargo/reference/build-scripts.html#life-cycle-of-a-build-script)（2026-09-19 查阅）；Windows 停滞的实跑诊断与验收见 [044](../task/044-windows-ci.md)。
