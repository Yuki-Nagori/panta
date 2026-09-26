# 架构总览

[任务索引](../task-index.md) · [重要模块说明](../modules/README.md) · [技术规范](../standards/README.md)

## 主题导航

| 主题 | 文档 | 范围 |
|---|---|---|
| Rust 领域与 C++ 重库边界 | [适配边界与 crate 规划](native-domain-boundaries.md) | 本仓库；当前审计与目标 |
| 目录与模块归属 | [目录规划](repository-layout.md) | 本仓库 |
| Cargo/CMake、运行与部署 | [构建与开发](build-and-development.md) | 本仓库 |
| QML、ViewModel 与交互 | [界面与桥接](ui-and-bridge.md) | 本仓库 |
| STEP、拓扑与 OCCT | [几何模型](geometry.md) | 本仓库 |
| Mesh IR、Netgen 与质量 | [网格](mesh.md) | 本仓库 |
| VTK、视口与字段 | [可视化](visualization.md) | 本仓库 |
| Rust、工程与持久化 | [应用平台与存储](application-and-storage.md) | 本仓库 |
| FSM DSL 与 Rust 状态机 | [FSM 编译期声明与事务设计](../modules/fsm.md) | 本仓库；后续规划 |
| 材料、Study 与 Python | [分析配置与自动化](study-and-automation.md) | 本仓库 |
| 进程、事件与数据交换 | [求解器接入](solver-integration.md) | 本仓库客户端 |
| 物理求解引擎 | [[External / 非本仓库] MoldSolver](external-moldsolver.md) | 外部仓库 |
| 共享 schema 与协议 | [[External / 非本仓库] Mold Protocol](external-mold-protocol.md) | 候选外部仓库 |
| 交付阶段与验收 | [里程碑与验证](milestones-and-validation.md) | 本仓库 |

## 文档状态与命名

本组文档依据用户提供的架构讨论附件整理。附件将桌面平台称为 MoldCAE / `moldcae`；本仓库名称是 `panta`，因此以下以 panta 指代该桌面平台。MoldCAE 是原方案名称，不代表已存在的本地目录或可执行文件。

当前仓库已有 Rust workspace、Cargo 调度的 native 构建与 Qt Quick/C++ ViewModel
桌面骨架（任务 001–005）。`panta-foundation` 已承载进程级崩溃设施，
`panta-ffi` 负责 CXX 边界；VTK 视口的旧 Qt OpenGL/QQuickVTKItem 模块、适配器和
创建级测试曾落地，但由于 macOS 26 窗口化渲染路径崩溃，任务 007 已切换到
VTK WebGPU + 平台 hardware window 路线（Linux Wayland、macOS Cocoa、Windows Win32）。
`sdk-vtk-9.7.0-webgpu` 已由受信 CI 发布并登记，原生 surface 宿主也已接入 native 工程；真实窗口与三平台运行冒烟仍由 007 收口。`App.qml` 已改为实际创建 `CaeViewport`。其余业务服务尚未完成。本组文档
描述目标架构与已验证的边界，不把规划能力写成已实现；具体依赖版本、协议
编码和持久化格式仍须在实施时验证并确定。文档中的字段名和 API 名用于说明
语义，不是已发布接口。

## 第一阶段目标

先实现 Geometry / Preprocessor + CAE UI + Visualization Platform：STEP → Geometry → Mesh → Visualization → Study Setup → Save Project。第一阶段不依赖真实 CFD 引擎完成；使用带明确标识的合成标量场验证结果显示。

V1 应覆盖工程新建、打开、保存，STEP 导入，面/边/实体选择，Netgen 网格生成，
表面与体网格显示，属性编辑，相机、裁剪、色标和求解器占位入口。当前 VTK
视口已完成 WebGPU/platform-surface 适配边界、最小测试图元和 native configure/build 验证；
真实窗口化、高 DPI、事件和三平台运行证据仍待完成，见[任务 007](../task/007-vtk-quick-viewport.md)，不能据此宣称 V1 视口已交付。
V1 不实现 AI、自研 GPU 计算后端或物理求解内核。VTK/Qt 正常使用图形硬件
不属于排除范围中的 GPU 计算开发。

## 分层和依赖方向

```text
QML → C++ ViewModel → CXX → Rust 应用服务 / 领域模型
                              ↓ 自有后端接口
                          CXX → C++ adapter → OCCT / Netgen
Rust 资产 / 显示快照 → CXX → C++ RenderScene / ViewportBackend → VTK
Rust Solver Client → 进程协议 → [External] MoldSolver
```

这是目标职责；现有 native STEP 摘要、Mesh IR 与 Rust 服务的接入状态见 [重库适配与 Rust 领域模块](native-domain-boundaries.md)。Rust 拥有业务、数据与编排，C++ 适配器实际调用重库并封装其类型 / 异常 / 生命周期。VTK 后端还负责窗口、输入与逐帧显示，不为每帧形成 QML → C++ → Rust → C++ → VTK 的往返。

进程级设施单独归 Rust 基础设施层：`panta-foundation` 实现崩溃信号、日志
等需要操作系统边界的能力，`panta-ffi` 只把安全入口映射到 CXX；`panta-core`
保持领域模型职责和无手写 `unsafe`。VTK 只存在于
`native/visualization/src/vtk/` 适配器，`RenderScene`、`ViewportBackend` 和
QML 公共头不暴露 VTK 类型。当前 `App.qml` 已创建 `CaeViewport`，但 Linux 当前锁定 Wayland，纯 X11 需要单独制品变体；模块加载/类型创建测试仍保留，供
构建和边界回归使用。旧 QQuickVTKItem、Qt OpenGL 场景图依赖和失效 SDK 引用已从代码与消费配置删除，不保留双路径。

## 技术基线

技术选型、版本验证状态和责任任务集中维护在 [技术基线](../standards/baseline.md)，编码与库使用规则见 [规范索引](../standards/README.md)。Rust/C++ 桥接由任务 006 开始验证 [CXX](../standards/cxx.md) 的最小双向路径；进程级崩溃入口由任务 047 经 `panta-ffi` 接入，VTK 版本与线程/图形后端证据由任务 007 和 [VTK 规范](../standards/vtk.md)维护。架构文档只说明模块职责与交互，不另维护一份版本表。

## 外部边界

[求解器客户端](solver-integration.md) 属于本仓库；[MoldSolver](external-moldsolver.md) 的 FEM/FVM/DG、物理模型和 CPU/CUDA 后端不属于本仓库。[Mold Protocol](external-mold-protocol.md) 是可选的共享契约仓库，尚未确定独立创建。远程和云端求解只作为接口扩展方向，不列入 V1 已承诺能力。
