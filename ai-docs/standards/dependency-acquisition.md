# 依赖获取与主平台环境

查阅 / 决策日期：2026-09-16。状态：主平台与固定清单已由任务 002 记录，同日按维护者要求升级至各上游最新稳定版并核对依赖间版本关系；CI 三平台矩阵由任务 018 建立。Cargo→CMake 调度已由任务 004 接通；工具二进制供给拆分为任务 020。Qt 已验证采用预编译包，VTK/OCCT/Netgen 的应用接入必须优先消费预编译 SDK，统一供给任务见 [031](../task/031-prebuilt-native-dependencies.md)。版本与兼容性结论的维护规则见 [技术基线](baseline.md)，本文只负责可复核的获取、重建、升级与回退步骤。

## 托管原则（2026-09-16 决策）

- Cargo 是唯一依赖入口：`cargo build` 触发的构建引导负责把固定的预编译 CMake、Ninja、Qt、VTK、OCCT、Netgen 包缓存到工作区构建树（根 `target/` 下托管目录），不写入源码树。正常开发构建不得把第三方源码加入本地构建图。
- 预编译优先：优先使用上游或项目发布的、与目标平台、架构、编译器 ABI、Qt 版本和所需模块匹配的 SDK/二进制包。只有不存在合适的预编译包时，才可另立任务评估由 CI/维护者生成可复用制品；本地构建源码不是默认回退，也不能静默触发。
- 包不可用或匹配检查失败时立即给出版本、平台、架构、ABI、缺失 target 和获取入口诊断；不能退回系统库或偷偷开始长时间源码编译。
- 不把系统包管理器（Homebrew/MacPorts/apt 等）作为项目基线；个人机器上已安装的同名包只是开发便利，不能当作兼容性证据，也不得进入提交的构建配置。
- 开发者前置：git、rustup（工具链由根 [rust-toolchain.toml](../../rust-toolchain.toml) 固定，缺失时 `rustup toolchain install`）与所在平台的 C++ 编译器工具链（macOS：Apple 命令行工具；Linux：gcc/clang；Windows：MSVC 构建工具）——编译器和标准库无法由 Cargo 供给。macOS 上 Qt 源码构建另需系统自带 `/usr/bin/perl`。首次构建需要联网，此后可离线重复构建；跨机器缓存共享由任务 012 统筹。
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
| CMake | 4.4.3 | `github.com/Kitware/CMake` release 资产：macOS `cmake-4.4.3-macos-universal.tar.gz`（universal，SHA256 `0c5d65251c14cc884bfa16bdbed3c263ce5bffe2e21c0d0d00962cb0610464fa`）、Linux x86_64 `cmake-4.4.3-linux-x86_64.tar.gz`（`d6c83076c575bc00b823522ac974bda66d0af05d6ddc30e739c12385cf32c6cc`）、Windows x86_64 `cmake-4.4.3-windows-x86_64.zip`（`4d52ebab7193a698651639ed80d8d04fd903358843572cf44c7fd234cb7c26ab`）；SHA256 为 2026-09-17 下载实测（020） | 官方二进制解包（020）：定位 `CMAKE` 旁路/PATH → 托管下载校验；解包用平台自带 `tar` | — | 可执行工具（非 CMake 包） | BSD-3（发布包 Copyright.txt） | 003/004/020 |
| Ninja | 1.13.2 | `github.com/ninja-build/ninja` release 资产：`ninja-mac.zip`（SHA256 `c99048673aa765960a99cf10c6ddb9f1fad506099ff0a0e137ad8960a88f321b`）、`ninja-linux.zip`（`5749cbc4e668273514150a80e387a957f933c6ed3f5f11e03fb30955e2bbead6`）、`ninja-win.zip`（`07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65`）；SHA256 为 2026-09-17 下载实测（020） | 预编译 release 资产（020）；解包用 `cmake -E tar`（libarchive，三平台零额外工具） | — | 可执行工具 | Apache-2.0（仓库 COPYING） | 003/004/020 |
| Qt | 6.11.2（qtbase + qtdeclarative + qttools 预编译包，含 Qml/Quick/QuickControls2/Test 与 qttools 的 lrelease/lupdate；qtsvg/qt5compat 等未取，需要时按 task 扩展） | `download.qt.io/online/qtsdkrepository/{mac_x64,linux_x64,windows_x86}/desktop/qt6_6112/`（与在线安装器同源、免账号；三端 URL 与归档名见 `native/cmake/qt-provision.cmake`，以脚本为准） | SHA256（2026-09-16/17 下载实测，硬编码于供给脚本并强校验） | 预编译 7z 解包（`cmake -E tar`，三平台零额外工具）；维护者决策（2026-09-16）：不源码构建。qttools 归档（034，SHA256 于 2026-09-17 实测）提供锁定 `lrelease` 供 QM 编译，不使用系统 Linguist | 供给缓存于 `target/panta-deps/qt/staging`（任务 041：跨 profile、Cargo 与 presets 共享；每归档解包指纹在 `qt/extracted/`，升级只补新归档）；安装版布局为平铺 bin/lib | `find_package(Qt6 6.11 COMPONENTS Core Gui Qml Quick QuickControls2 Test)`；`bin/lrelease` 由 `native/i18n` 消费 | LGPL-3.0（发布包 LICENSES/） | 005/034 |
| Qt Linux ICU runtime | 73.2（与 Qt 6.11.2 Linux 工具 ABI 匹配） | Qt 官方 qtsdkrepository `6.11.2-0-202608131018icu-linux-Rhel8.6-x86_64.7z` | SHA256 `111bdae30a66fff6ef65620e95766170fa5a6f425c360ea33b79cb2ec7e2fd86` | 预编译 7z 解包至 `qt/staging/lib`；仅 Linux 使用，不使用系统 ICU 或源码构建 | `rcc`、`qtpaths`、`qmlimportscanner` 运行时依赖 | ICU 许可证随归档 | 005/036 |
| VTK | 9.7.0（commit `23f0a095621e91bbdbeace8451e22b950c8e5f46`，tag `v9.7.0` 解引用，unmodified 上游源码） | 038 受信 CI 制品已发布（Release `sdk-vtk-9.7.0`，2026-09-17 三平台生产全绿）：macOS `…/vtk-9.7.0-macos-arm64.tar.gz`（SHA256 `eb3c2f298c1640d347b42cfbd0715fb05b95eb403e0cff2cbb1d2139f6feca58`）、Linux `…/vtk-9.7.0-linux-x86_64.tar.gz`（`d2b54fb37eb82bc27c3ce1840556a4c92129b6c0fb9d07ac4f0407ac66e2f80c`）、Windows `…/vtk-9.7.0-windows-x86_64.tar.gz`（`023931171a60b1a74bb97766d26713dabab73d4ff0ed4b6f706aba67cf60a710`）；完整 URL 见 031 sdk-provision.cmake manifest，合规说明见 `tools/sdk/releases/vtk-9.7.0.md` | 预编译归档 SHA256（2026-09-17 Release `.sha256` 实测，manifest 强校验）；源码 tag/commit 作为来源依据 | 预编译制品消费（031 `panta_require_sdk(vtk)` 三平台登记；macOS 生产 consumer 烟测 + 离线复用通过）。Release/共享库/Qt6/GUISupportQtQuick，Qt 为锁定预编译 6.11.2；Linux 由 ubuntu-24.04 gcc13 生产，有效 glibc 基线待 007 运行验证回写 | `GUISupportQtQuick`/`GUISupportQt`/`RenderingQt`/`ViewsQt` 已在制品中确认（038 自检）；窗口运行时行为由 007 验证 | `find_package(VTK CONFIG)`：`VTK::GUISupportQtQuick` 等 `VTK::` 命名空间 targets | BSD-3（随包 `share/licenses/VTK/Copyright.txt`） | 031/038（configure 级）；007（窗口级） |
| OCCT | 8.0.1（tag `V8.0.1`，commit `b8f597c67781`，2026-07-30，当前最新 release） | Windows x86_64 官方归档已登记（031 manifest）：`opencascade-release-no-pch.zip`（SHA256 `307f694f1d4a280c7f58ee2ddb69a7f7e2b78d82749339efd40ce8b8b116d76c`，内层 `opencascade-8.0.1-vc14-64.zip` SHA256 `24d947bf045e8da43f559592d28eee4df389dd8034754d70f3ab341346a22ca8`）；macOS/Linux 无官方 SDK，待 038 | 预编译包 SHA256（Windows 已登记）；tag SHA 作为来源依据 | 预编译 SDK 优先（供给入口 031 sdk-provision.cmake）；Windows 候选与 Qt/编译器 ABI 集成尚未验证（009/038），其余平台请求按缺资产失败 | 渲染/DRAW/Tcl-TK 相关关闭、模块裁剪由 009 定；SDK 必须提供 `find_package(OpenCASCADE)` | `find_package(OpenCASCADE CONFIG)` | LGPL-2.1 + OCCT 例外（发布包 LICENSE.txt） | 009/031 |
| Netgen | v6.2.2604（commit `3ee489c7d58f`，当前最新 release） | NGSolve 官方 release/SDK 或项目制品源（具体平台包由 031 固定） | 预编译包 SHA256；tag SHA 作为来源依据 | 供给入口已落地（031 sdk-provision.cmake）；上游无 release 预编译资产，任何平台请求均按缺资产失败并指向 038，不源码构建 | `USE_OCC=ON`、GUI/Python 关闭等由 010 定；SDK 与 OCCT ABI 必须匹配 | 由 010/031 核实 `find_package` targets | LGPL-2.1（发布包/仓库 LICENSE） | 010/031 |
| GoogleTest（仅测试） | v1.18.0（commit `063de7e9578f82b369302001269680b4b1553359`，当前最新 release） | `github.com/google/googletest` tag `v1.18.0` | tag commit SHA（即 FetchContent `GIT_TAG`） | CMake FetchContent：仅 `BUILD_TESTING=ON` 拉取，`EXCLUDE_FROM_ALL`、`INSTALL_GTEST=OFF` | `gtest_force_shared_crt=ON`；`BUILD_GMOCK=OFF`（需要时按 task 打开） | `GTest::gtest` / `GTest::gtest_main` | BSD-3（源码树 LICENSE） | 019 |

"关键候选选项"是起点而非决定；实施任务按验证结果调整并回写本表。

## 依赖间版本关系（2026-09-16 核实）

- **Netgen ↔ OCCT**：netgen v6.2.2604 的 OCCT 适配经 `find_package(OpenCASCADE)`，源码内版本守卫为 `NETGEN_OCC_VERSION_AT_LEAST` 风格（如 ≥7.4、≥7.8 走 TKDE 新目标名），对 8.0 为"大于等于"语义，未发现排除 8.x 的守卫或上游 issue；因此采用 OCCT 8.0.1。该组合**未经构建实测**：010 必须用真实几何到网格验证；若不兼容，回退固定 OCCT `V7_9_3`（commit `a016080bf673`）并在此记录原因。**制品层配对（2026-09-18 维护者决策）**：OCCT 不承诺跨版本 C++ ABI 稳定，Netgen 制品只与构建时所链接的 OCCT 兼容，混用会在加载/链接期失败或静默出错；因此两者由 038 同管线、同工具链生产并**合并为一个 Release 成对发布**（`sdk-occt-netgen-<occt>-<netgen>`），升级必须成对重建发布，不允许只换一侧。
- **VTK ↔ Qt**：VTK 9.7.0 的预编译 SDK 必须明确包含 `GUISupportQtQuick`/`QQuickVTKItem`、Qt 6.11.2 兼容范围和目标 ABI；007 验证实际窗口与运行时加载。没有匹配 SDK 时，先由 031 评估项目制品，不在开发机直接编译 VTK。2026-09-17（038 增量一）已在 macOS arm64 构建级证实：9.7.0 × Qt 6.11.2 预编译包可产出 `GUISupportQtQuick` 并经 `find_package(VTK CONFIG)` 以 `VTK::GUISupportQtQuick` 消费；窗口运行时行为仍归 007。
- **全体 ↔ CMake 4.4**：清单内项目声明的 CMake 下限均 ≥3.10，高于 CMake 4 移除的 <3.5 兼容线；首次 configure 由 003/004 实测。
- Rust 侧无 native 版本耦合；CXX 的 MSRV（1.88）低于固定工具链 1.98.1，约束见 [Rust 规范](rust.md)。

## CI（018）

- GitHub Actions workflow [ci.yml](../../.github/workflows/ci.yml)：push 到 main 与全部 pull request 触发；矩阵 `macos-latest` / `ubuntu-latest` / `windows-2022`。Windows 使用 Visual Studio 17 2022 与 Qt MSVC2022 预编译包；Ubuntu 在 configure 前安装 `libgl1-mesa-dev`，仅补齐 Qt Gui 的 OpenGL 开发文件，不替代项目托管的 Qt 供给。
- Qt 6.11.2 Linux 预编译归档面向 RHEL9，Qt 工具（包括 `rcc`、`qtpaths`、`qmlimportscanner`）需要 ICU 73。`native/cmake/qt-provision.cmake` 同步下载并校验 Qt 官方的 ICU 73 预编译归档，解包到 `qt/staging/lib`，让 Ubuntu 使用与 Qt 工具匹配的 ABI；不使用系统 ICU、不伪造 SONAME，也不源码编译 ICU。Linux 仍跳过仅供 IDE 使用的 `.qmlls.build.ini` 和当前 app 的空 import scan，保留 QML typeinfo、cachegen、资源和运行时验证。
- 每个平台执行与本地一致的最小检查：按 rust-toolchain.toml 安装固定工具链（minimal + rustfmt + clippy）→ `cargo build --locked` → `cargo test --locked` → `cargo fmt --all -- --check` → `cargo clippy --locked --all-targets`。
- runner 需要镜像自带的平台编译器与 rustup；Ubuntu 额外安装上面列出的 OpenGL 开发包，以满足预编译 Qt 的 CMake 探测。
- 边界：当前 CI 通过 Cargo 同步验证 Rust 与已有 native/Qt 构建；VTK、OCCT、Netgen 的 SDK 供给与集成仍由 031 及后续任务扩展，测试聚合与质量门禁归 011，依赖缓存归 012。

## 干净重建步骤

当前流程（004 已接通调度；Qt 预编译供给由 005 落地，VTK/OCCT/Netgen 预编译供给由 031 负责，工具二进制由 020 负责）：

1. 按 [README 环境要求](../../README.md#环境要求) 安装平台前置（macOS：Apple CLT；Linux：gcc/clang；Windows：MSVC）与 rustup；CMake/Ninja/Qt/VTK/OCCT/Netgen 由固定预编译供给处理。
2. 在仓库内执行 `rustup toolchain install 1.98.1`（或首次 cargo 命令时按 rustup 提示安装）。
3. `cargo build --locked`：launcher 的 build.rs 调度 CMake/Ninja，消费构建树内固定的预编译依赖（缺失或 ABI 不匹配时立即失败并给出诊断）；`cargo run` 启动 `panta-native` 并转发参数与退出码。VTK/OCCT/Netgen 的 SDK 供给模块已落地（031 `native/cmake/sdk-provision.cmake`，缓存于 `target/panta-deps/sdk/`），但当前仅 OCCT Windows 有固定资产；消费方任务（007/009/010）接入前生产构建不触发 SDK 下载。

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
