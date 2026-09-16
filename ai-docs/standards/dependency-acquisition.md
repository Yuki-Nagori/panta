# 依赖获取与主平台环境

查阅 / 决策日期：2026-09-16。状态：主平台与固定清单已由任务 002 记录，同日按维护者要求升级至各上游最新稳定版并核对依赖间版本关系；CI 三平台矩阵由任务 018 建立。Cargo→CMake 调度已由任务 004 接通（本机需具备 CMake/Ninja，缺失诊断可定位）；工具二进制的自动供给拆分为任务 020，Qt/VTK/OCCT/Netgen 的托管获取随对应任务落地。版本与兼容性结论的维护规则见 [技术基线](baseline.md)，本文只负责可复核的获取、重建、升级与回退步骤。

## 托管原则（2026-09-16 决策）

- Cargo 是唯一依赖入口：`cargo build` 触发的构建引导负责供给 CMake、Ninja、Qt、VTK、OCCT、Netgen，产物全部落在工作区构建树（根 `target/` 下托管目录），不写入源码树。
- 不把系统包管理器（Homebrew/MacPorts/apt 等）作为项目基线；个人机器上已安装的同名包只是开发便利，不能当作兼容性证据，也不得进入提交的构建配置。
- 开发者前置：git、rustup（工具链由根 [rust-toolchain.toml](../../rust-toolchain.toml) 固定，缺失时 `rustup toolchain install`）与所在平台的 C++ 编译器工具链（macOS：Apple 命令行工具；Linux：gcc/clang；Windows：MSVC 构建工具）——编译器和标准库无法由 Cargo 供给。macOS 上 Qt 源码构建另需系统自带 `/usr/bin/perl`。首次构建需要联网，此后可离线重复构建；跨机器缓存共享由任务 012 统筹。
- 版本固定：git tag 附 commit SHA 校验；下载的二进制附 SHA256，首次下载时校验并回填本文。升级必须修改固定清单、完成对应集成验证后同一 commit 提交。
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
| CMake | 4.4.3 | `github.com/Kitware/CMake` release 资产 `cmake-4.4.3-macos-universal.tar.gz` | SHA256 待 020 首次下载时记录 | 官方二进制解包（020）；当前要求本机可用（如 Homebrew），缺失诊断由 004 调度给出 | — | 可执行工具（非 CMake 包） | BSD-3（发布包 Copyright.txt） | 003/004 |
| Ninja | 1.13.2 | `github.com/ninja-build/ninja` tag `v1.13.2`（当前最新 release） | tag commit；SHA256 待 020 记录 | 源码构建（备选 release 资产 `ninja-mac.zip`） | — | 可执行工具 | Apache-2.0（仓库 COPYING） | 003/004 |
| Qt | 6.11.2（qtbase + qtdeclarative 预编译包，含 Qml/Quick/QuickControls2/Test；qtsvg/qt5compat 等未取，需要时按 task 扩展） | `download.qt.io/online/qtsdkrepository/{mac_x64,linux_x64,windows_x86}/desktop/qt6_6112/`（与在线安装器同源、免账号；三端 URL 与归档名见 `native/cmake/qt-provision.cmake`，以脚本为准） | SHA256（2026-09-16 下载实测，硬编码于供给脚本并强校验） | 预编译 7z 解包（`cmake -E tar`，三平台零额外工具）；维护者决策（2026-09-16）：不源码构建 | 供给缓存于构建树 `qt/staging`；安装版布局为平铺 bin/lib | `find_package(Qt6 6.11 COMPONENTS Core Gui Qml Quick QuickControls2 Test)` | LGPL-3.0（发布包 LICENSES/） | 005 |
| VTK | 9.7.0（commit `23f0a095621e`，当前最新稳定 tag） | `github.com/Kitware/VTK` tag `v9.7.0`（官方镜像；上游 `gitlab.kitware.com/vtk/vtk`） | tag commit SHA | 源码构建 | testing OFF；启用 Qt 组（GUISupportQtQuick）；其余 007 定；`cmake_minimum_required 3.12...3.21` | `find_package(VTK)` 模块化 targets | BSD-3（源码树 Copyright.txt） | 007 |
| OCCT | 8.0.1（tag `V8.0.1`，commit `b8f597c67781`，2026-07-30，当前最新 release） | `github.com/Open-Cascade-SAS/OCCT` tag `V8.0.1` | tag commit SHA | 源码构建 | 渲染/DRAW/Tcl-TK 相关关闭、模块裁剪由 009 定；`cmake_minimum_required 3.10` | `find_package(OpenCASCADE)` | LGPL-2.1 + OCCT 例外（源码树 LICENSE.txt） | 009 |
| Netgen | v6.2.2604（commit `3ee489c7d58f`，当前最新 release） | `github.com/NGSolve/netgen` tag `v6.2.2604` | tag commit SHA | 源码构建 | `USE_OCC=ON`、GUI/Python 关闭等由 010 定；该 tag 默认 `USE_OCC=ON`，经 `find_package(OpenCASCADE)`；`cmake_minimum_required 3.16` | 待 010 核实 | LGPL-2.1（仓库 LICENSE） | 010 |
| GoogleTest（仅测试） | v1.18.0（commit `063de7e9578f82b369302001269680b4b1553359`，当前最新 release） | `github.com/google/googletest` tag `v1.18.0` | tag commit SHA（即 FetchContent `GIT_TAG`） | CMake FetchContent：仅 `BUILD_TESTING=ON` 拉取，`EXCLUDE_FROM_ALL`、`INSTALL_GTEST=OFF` | `gtest_force_shared_crt=ON`；`BUILD_GMOCK=OFF`（需要时按 task 打开） | `GTest::gtest` / `GTest::gtest_main` | BSD-3（源码树 LICENSE） | 019 |

"关键候选选项"是起点而非决定；实施任务按验证结果调整并回写本表。

## 依赖间版本关系（2026-09-16 核实）

- **Netgen ↔ OCCT**：netgen v6.2.2604 的 OCCT 适配经 `find_package(OpenCASCADE)`，源码内版本守卫为 `NETGEN_OCC_VERSION_AT_LEAST` 风格（如 ≥7.4、≥7.8 走 TKDE 新目标名），对 8.0 为"大于等于"语义，未发现排除 8.x 的守卫或上游 issue；因此采用 OCCT 8.0.1。该组合**未经构建实测**：010 必须用真实几何到网格验证；若不兼容，回退固定 OCCT `V7_9_3`（commit `a016080bf673`）并在此记录原因。
- **VTK ↔ Qt**：VTK 9.7.0 的 Qt 集成（GUISupportQtQuick / QQuickVTKItem）与 Qt 6.11.2 的实际编译/运行兼容由 007 验证；同源源码构建（同一引导、同一编译器）降低 ABI 漂移风险。
- **全体 ↔ CMake 4.4**：清单内项目声明的 CMake 下限均 ≥3.10，高于 CMake 4 移除的 <3.5 兼容线；首次 configure 由 003/004 实测。
- Rust 侧无 native 版本耦合；CXX 的 MSRV（1.88）低于固定工具链 1.98.1，约束见 [Rust 规范](rust.md)。

## CI（018）

- GitHub Actions workflow [ci.yml](../../.github/workflows/ci.yml)：push 到 main 与全部 pull request 触发；矩阵 `macos-latest` / `ubuntu-latest` / `windows-latest`。
- 每个平台执行与本地一致的最小检查：按 rust-toolchain.toml 安装固定工具链（minimal + rustfmt + clippy）→ `cargo build --locked` → `cargo test --locked` → `cargo fmt --all -- --check` → `cargo clippy --locked --all-targets`。
- runner 只需镜像自带的平台编译器与 rustup，无额外系统包——与 README 环境要求一致。
- 边界：当前 CI 只验证 Rust 骨架层。native 托管构建接入 CI 随 004 之后扩展；测试聚合与质量门禁归 011，依赖缓存归 012。

## 干净重建步骤

当前流程（004 已接通调度；Qt/VTK/OCCT/Netgen 与二进制自动供给分别随任务 005+ 与 020 落地）：

1. 按 [README 环境要求](../../README.md#环境要求) 安装平台前置（macOS：Apple CLT；Linux：gcc/clang；Windows：MSVC）、rustup 与 CMake/Ninja（暂时手动，020 后免除）。
2. 在仓库内执行 `rustup toolchain install 1.98.1`（或首次 cargo 命令时按 rustup 提示安装）。
3. `cargo build --locked`：launcher 的 build.rs 调度 CMake/Ninja 构建 native 骨架（构建树在 `target/` 内 OUT_DIR 下）；`cargo run` 启动 `panta-native` 并转发参数与退出码。

验证入口见 [构建与开发](../architecture/build-and-development.md)；每项依赖的集成验证任务见固定清单最后一列。

## 升级与回退

- 单个依赖升级：修改本文固定 tag/SHA → 核对与其它依赖的版本关系（尤其 Netgen ↔ OCCT）→ 对应集成验证任务执行最小验证集（几何/网格/视口冒烟）→ 回填 SHA256 与实测结果 → 清单、task 与受影响规范同一 commit 提交。
- 回退：恢复上一版固定清单并按干净重建重建；构建树可整目录删除，不影响源码树。OCCT 组合的既定回退点是 `V7_9_3`。
- Rust 工具链升级规则见 [Rust 规范](rust.md)。

## 已知风险与显式未验证项

- Qt/VTK 源码构建为小时级耗时；解法是 012 的缓存与共享，不因慢退回系统包。
- Netgen v6.2.2604 × OCCT 8.0.1 只有源码守卫级证据，构建与网格正确性由 010 实测；这是当前清单中关系最紧的组合。
- Qt Quick（macOS Metal 后端等）与 VTK GL 上下文的交互是 007 的重点风险；本文只固定源码版本，不预支任何兼容结论。
- 二进制 SHA256 在实装时回填，此前不做无产物的形式校验。
