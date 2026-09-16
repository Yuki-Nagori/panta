# 依赖获取与主平台环境

查阅 / 决策日期：2026-09-16。状态：主平台与固定清单已由任务 002 记录；Cargo 托管引导尚未实装，实装与首次配置验证在任务 003/004。版本与兼容性结论的维护规则见 [技术基线](baseline.md)，本文只负责可复核的获取、重建、升级与回退步骤。

## 托管原则（2026-09-16 决策）

- Cargo 是唯一依赖入口：`cargo build` 触发的构建引导负责供给 CMake、Ninja、Qt、VTK、OCCT、Netgen，产物全部落在工作区构建树（根 `target/` 下托管目录），不写入源码树。
- 不把系统包管理器（Homebrew/MacPorts 等）作为项目基线；个人机器上已安装的同名包只是开发便利，不能当作兼容性证据，也不得进入提交的构建配置。
- 开发者前置只有三项：git；rustup（工具链由根 [rust-toolchain.toml](../../rust-toolchain.toml) 固定，缺失时 `rustup toolchain install 1.98.1`）；Apple 命令行工具（编译器、libc++、macOS SDK 与 `/usr/bin/perl`，最后者是 Qt 源码构建 syncqt 所需）。首次构建需要联网，此后可离线重复构建；跨机器缓存共享由任务 012 统筹。
- 版本固定：git tag 附 commit SHA 校验；下载的二进制附 SHA256，首次下载时校验并回填本文。升级必须修改固定清单、完成对应集成验证后同一 commit 提交。

## 主验证平台（002 记录）

| 项目 | 值 |
|---|---|
| 操作系统 | macOS 26.3.1，arm64（Apple Silicon） |
| 编译器 | Apple clang 17.0.0（/Library/Developer/CommandLineTools，含 macOS 26.2 SDK） |
| C++ 标准库 | libc++（随工具链） |
| Perl | /usr/bin/perl 5.34.1（Qt syncqt 需要） |
| CMake / Ninja | 不要求预装；由引导按下方清单供给 |

当前工作目录所在机器不自动构成所有发布平台的承诺；不承诺 Windows/Linux，跨平台矩阵另立任务。

## 固定清单（候选；"集成验证"列对应任务通过前不视为实测通过）

| 依赖 | 固定版本（tag） | 来源 | 校验 | 获取形态 | 关键候选选项 | CMake package / targets | 许可证入口 | 集成验证 |
|---|---|---|---|---|---|---|---|---|
| CMake | 4.3.3 | `github.com/Kitware/CMake` release 资产 `cmake-4.3.3-macos-universal.tar.gz` | SHA256 待 004 首次下载时记录 | 官方二进制解包 | — | 可执行工具（非 CMake 包） | BSD-3（发布包 Copyright.txt） | 003 |
| Ninja | 1.13.2 | `github.com/ninja-build/ninja` tag `v1.13.2` | tag commit；SHA256 待 004 记录 | 源码构建（备选 release 资产 `ninja-mac.zip`） | — | 可执行工具 | Apache-2.0（仓库 COPYING） | 003 |
| Qt | 6.11.1：qtbase `59c81a3c2247`、qtdeclarative `a02bed441965`、qtshadertools `b3b8537b3c5a`、qtsvg `2596f43da2dc`、qt5compat `c1e3fdd994e6`（均可选 SHA 前缀） | `github.com/qt/{qtbase,qtdeclarative,qtshadertools,qtsvg,qt5compat}` tag `v6.11.1` | 上列 commit SHA | 源码构建；qt5compat 是否需要由 005 按 QML import 裁剪 | examples/tests OFF；具体配置 004/005 定 | `find_package(Qt6)`、`qt_add_qml_module()` | LGPL-3.0（源码树 LICENSES/） | 005 |
| VTK | 9.7.0（commit `23f0a095621e`） | `github.com/Kitware/VTK` tag `v9.7.0`（官方镜像；上游 `gitlab.kitware.com/vtk/vtk`） | tag commit SHA | 源码构建 | testing OFF；启用 Qt 组（GUISupportQtQuick）；其余 007 定 | `find_package(VTK)` 模块化 targets | BSD-3（源码树 Copyright.txt） | 007 |
| OCCT | 7.9.3（tag `V7_9_3`，commit `a016080bf673`） | `github.com/Open-Cascade-SAS/OCCT` tag `V7_9_3` | tag commit SHA | 源码构建 | 渲染/DRAW/Tcl-TK 相关关闭、模块裁剪由 009 定 | `find_package(OpenCASCADE)` | LGPL-2.1 + OCCT 例外（源码树 LICENSE.txt） | 009 |
| Netgen | v6.2.2604（commit `3ee489c7d58f`） | `github.com/NGSolve/netgen` tag `v6.2.2604` | tag commit SHA | 源码构建 | `USE_OCC=ON`、GUI/Python 关闭等由 010 定；该 tag 默认 `USE_OCC=ON`，经 `find_package(OpenCASCADE)` 并适配 ≥7.8 的 TKDE 目标名 | 待 010 核实 | LGPL-2.1（仓库 LICENSE） | 010 |

"关键候选选项"是起点而非决定；实施任务按验证结果调整并回写本表。

## 干净重建步骤

目标流程（第 3 步的引导行为在 004 实装前不存在；当前 `cargo build` 只构建 Rust 骨架）：

1. 安装 git 与 Apple 命令行工具（`xcode-select --install`）。
2. 安装 rustup；在仓库内执行 `rustup toolchain install 1.98.1`，之后 `cargo` 自动命中固定工具链。
3. `cargo build --locked`：构建引导按固定清单拉取 CMake/Ninja 并构建 Qt/VTK/OCCT/Netgen 到构建树，再链接 native 与 Rust 产物；全部产物不离开构建树。

验证入口见 [构建与开发](../architecture/build-and-development.md)；每项依赖的集成验证任务见固定清单最后一列。

## 升级与回退

- 单个依赖升级：修改本文固定 tag/SHA → 对应集成验证任务执行最小验证集（几何/网格/视口冒烟）→ 回填 SHA256 与实测结果 → 清单、task 与受影响规范同一 commit 提交。
- OCCT 7.9 → 8.0 是大版本升级：上游 8.0.1 已发布（2026-07-30），但 Netgen v6.2.2604 未声明支持 8.0；升级前必须先确认 Netgen 侧兼容，否则保持 7.9.x。
- 回退：恢复上一版固定清单并按干净重建重建；构建树可整目录删除，不影响源码树。
- Rust 工具链升级规则见 [Rust 规范](rust.md)。

## 已知风险与显式未验证项

- Qt/VTK 源码构建为小时级耗时；解法是 012 的缓存与共享，不因慢退回系统包。
- CMake 4.x 移除了对 CMake < 3.5 的兼容；清单内项目声明的下限（如 Netgen 3.16）均高于该线，与 CMake 4.3 的首次 configure 仍由 003/004 实测。
- Qt Quick（macOS Metal 后端）与 VTK GL 上下文的交互是 007 的重点风险；本文只固定源码版本，不预支任何兼容结论。
- Netgen 与 OCCT 的 ABI/链接一致性在 010 用真实几何到网格验证；pin 一致不等于兼容证据。
- 二进制 SHA256 在实装时回填，此前不做无产物的形式校验。
