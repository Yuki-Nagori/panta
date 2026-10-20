# 技术基线与版本决策

更新日期：2026-10-20（Rust 工具链升级）。主验证平台、依赖获取方式与版本选择由任务 002 记录，各项集成的实际验收以责任任务为准。

## 当前基线

| 项目 | 项目选择 | 尚需确定/验证 | 责任任务 |
|---|---|---|---|
| 主验证平台 | macOS 26.3.1 / arm64，Apple clang 17.0.0（CLT）+ macOS 26.2 SDK + libc++ | 当前自有 C++ 使用托管 LLVM；三平台 CI 不代替实机验收 | 002 / 018 |
| C++ | 自有代码 C++20 | 部署目标版本、编译器选项矩阵 | 002 / 003 |
| C++ 测试 | GoogleTest v1.18.0（三平台固定 SDK，仅 `BUILD_TESTING`；规则见 [gtest](gtest.md)） | 具体测试仍按影响范围运行；完整矩阵由 nightly 验证 | 019 / 070 / 071 / 011 / 120 |
| Rust | stable 1.99.0，edition 2024 | MSRV 1.95；升级验证见任务 001 | 001 |
| Cargo | 主开发入口；提供 build、run、test、format、lint、coverage 和性能命令 | 命令职责与范围见[构建说明](../architecture/build-and-development.md)和[测试规范](testing.md) | 001 / 004 / 043 |
| CMake + Ninja | CMake 4.4.3 + Ninja 1.13.2（Cargo 引导供给，固定工具资产已接入） | 按任务记录验收各平台 native 配置与部署 | 002 / 003 / 020 |
| Qt | 6.11.2 预编译包，包含 qtbase、qtdeclarative、QtSvg 与 QtTools | 桌面部署冒烟仍由 013 跟踪 | 005 / 013 |
| VTK | V1 渲染后端：9.7.0 WebGPU；使用三平台预编译 hardware-window SDK | 原生 surface/view 与 Qt Quick 叠加、图形后端、ABI 与运行时加载 | 002 / 007 / 031 / 038 |
| OCCT | CAD/STEP：8.0.1；优先使用与目标 ABI 匹配的预编译 SDK | STEP/元数据路径、模块裁剪、与 Netgen 组合实测 | 002 / 009 / 031 |
| Netgen | 自有 Mesh IR 的生成器：v6.2.2604；优先使用与 OCCT 匹配的预编译 SDK | 与 OCCT 8.0.1 组合实测、C++ 接口与导出 targets | 002 / 010 / 031 |
| Python 工具环境 | uv 0.12.18 管理 CPython 3.14.7，锁定质量工具依赖；业务运行时 / binding 未接入 | 版本与 wheel 摘要见 `pyproject.toml` / `uv.lock` | 014 |
| Rust/C++ FFI | CXX | CXX 已接入；接口与所有权随功能验收 | 006 |
| Python binding | pybind11 / Rust binding 候选 | 另建 task 决策 | 不在本轮必做范围 |

## 版本与证据维护

精确版本、URL / SHA256、构建选项与许可证入口由[依赖获取规范](dependency-acquisition.md)链接到实际配置。Cargo 托管预编译依赖；系统包不构成项目基线，源码生产通过独立制品任务评估。

表格记录项目技术选择，实施状态和证据以责任任务为准。主验证机器不等同于发布支持范围；候选选版、configure / 链接通过与实际窗口或几何到网格验收须分开记录。

升级同一提交记录来源、ABI / 运行库、启用模块、验证和回退；优先稳定 release，不以浮动 latest / nightly 代替固定版本。第三方许可证按发布包保留，不适用本仓库许可证。详细编码约束见[规范索引](README.md)。
