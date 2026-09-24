# 技术基线与版本决策

查阅日期：2026-09-16。状态：主验证平台、依赖获取方式与候选版本已由任务 002 记录；除 Rust 骨架外，各项集成都尚未实测，实测通过前全部视为候选。

## 当前基线

| 项目 | 项目选择 | 尚需确定/验证 | 责任任务 |
|---|---|---|---|
| 主验证平台 | macOS 26.3.1 / arm64，Apple clang 17.0.0（CLT）+ macOS 26.2 SDK + libc++ | CI 矩阵在 macOS/Linux/Windows 验证 Rust 层（018）；native 层跨平台矩阵随构建任务扩展 | 002 / 018 |
| C++ | 自有代码 C++20 | 部署目标版本、编译器选项矩阵 | 002 / 003 |
| C++ 测试 | GoogleTest v1.18.0（三平台固定 SDK，仅 `BUILD_TESTING`；规则见 [gtest](gtest.md)） | 聚合入口与门禁 | 019 / 070 / 071 / 011 |
| Rust | stable，edition 2024 | 版本与 MSRV 已固定（stable 1.98.1；`rust-version` 下限 1.88） | 001 |
| Cargo | 主开发入口 | launcher 骨架已落地（001）；native 调度待 004 | 001 / 004 |
| CMake + Ninja | CMake 4.4.3 + Ninja 1.13.2（Cargo 引导供给，均为上游最新） | 引导实装与首次 configure | 002 / 003 |
| Qt | 6.11.2 预编译包（qtbase+qtdeclarative，维护者决策不源码构建；Quick 运行时已由 005 实测） | 模块扩展（qtsvg 等）、部署 | 005 / 013 |
| VTK | V1 渲染后端：9.7.0 WebGPU；使用三平台预编译 hardware-window SDK | 原生 surface/view 与 Qt Quick 叠加、图形后端、ABI 与运行时加载 | 002 / 007 / 031 / 038 |
| OCCT | CAD/STEP：8.0.1；优先使用与目标 ABI 匹配的预编译 SDK（实测前保留 7.9.3 回退点） | STEP/元数据路径、模块裁剪、与 Netgen 组合实测 | 002 / 009 / 031 |
| Netgen | 自有 Mesh IR 的生成器：v6.2.2604；优先使用与 OCCT 匹配的预编译 SDK | 与 OCCT 8.0.1 组合实测、C++ 接口与导出 targets | 002 / 010 / 031 |
| Python | 3.12+，后续 | 解释器、环境与工具依赖锁 | 014 |
| Rust/C++ FFI | CXX 首选候选 | 固定版本、CMake 最终链接与所有权验证 | 006 |
| Python binding | pybind11 / Rust binding 候选 | 另建 task 决策 | 不在本轮必做范围 |

精确 tag、commit SHA、来源 URL、构建选项候选、依赖间版本关系、许可证入口和集成验证责任见 [依赖获取与主平台环境](dependency-acquisition.md)。

## 依赖获取与版本固定（2026-09-16 决策）

Cargo 统一托管：开发者只需安装 git、rustup 与 Apple 命令行工具，`cargo build` 触发的构建引导在首次构建时按固定清单获取预编译依赖到构建树。不以 Homebrew 等系统包管理器作为项目基线；本机同名包只是个人便利，不构成兼容性证据。git tag 以 commit SHA 校验，预编译包以 SHA256 及平台/架构/ABI 元数据校验（031 实装时回填）。

预编译优先是 native 依赖的默认策略：VTK、OpenCASCADE、Netgen 和 Qt 应优先使用上游或项目发布的预编译 SDK；无法获得匹配包时，由独立制品任务评估一次性构建并分发，不能让每个开发者在本地重复编译。源码构建只是记录充分理由后的兜底路径，不作为普通 `cargo build` 的隐式行为。

## 决策记录要求

主平台已由 002 记录（操作系统、架构、编译器及来源）；当前工作目录所在机器不自动构成所有发布平台的承诺。依赖升级或新增时记录精确版本/tag、来源与校验信息、链接方式、启用模块、CMake package/targets、许可证入口和对应测试。

依赖基线分“候选”和“实测通过”；安装完成或 configure 成功不能单独证明 ABI 与运行时兼容。Qt/VTK 需实际窗口验证；OCCT/Netgen 需实际几何到网格验证。Python 工具链可独立推进，不反向阻塞 M0。

优先稳定 release，不以 nightly/latest 作为锁定标识。升级任务要说明触发原因、接口变化、回退路径和最小验证集。第三方许可说明来自其发布包，不将本仓库 Apache-2.0 套用到依赖。

## 事实、约定与验证的边界

C++20、模块隔离、Cargo 主入口是项目选择；各工具实际支持的 API 由官方资料决定；组合是否可用由任务验收决定。任务索引中的 ready 只表示可开始，不代表任何依赖已经安装。

固定清单与升级记录维护在依赖获取文档；本页保留结论与责任分工，实施证据写入对应任务。各技术依据见 [规范索引](README.md)，开始工作见 [任务索引](../task-index.md)。
