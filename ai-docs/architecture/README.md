# 架构总览

[任务索引](../task-index.md) · [重要模块说明](../modules/README.md) · [技术规范](../standards/README.md)

## 主题导航

| 主题 | 文档 | 范围 |
|---|---|---|
| Rust 领域与 C++ 重库边界 | [适配边界与 crate 规划](native-domain-boundaries.md) | 本仓库；当前审计与目标 |
| 目录与模块归属 | [目录规划](repository-layout.md) | 本仓库 |
| Rust 应用命令层 | [GUI / CLI 共享命令设计](command-layer.md) | 当前 GUI / CLI 边界与后续扩展 |
| Cargo/CMake、运行与部署 | [构建与开发](build-and-development.md) | 本仓库 |
| QML、ViewModel 与交互 | [界面与桥接](ui-and-bridge.md) | 本仓库 |
| STEP、拓扑与 OCCT | [几何模型](geometry.md) | 本仓库 |
| Mesh IR、Netgen、资产 revision 与旁置存储 | [网格](mesh.md) | 本仓库；持久化与按需加载见 [088](../task/088-mesh-asset-sidecar-storage.md) |
| VTK、视口与字段 | [可视化](visualization.md) | 本仓库 |
| Rust、工程与持久化 | [应用平台与存储](application-and-storage.md) | 本仓库 |
| FSM DSL 与 Rust 状态机 | [FSM 编译期声明与事务设计](../modules/fsm.md) | 本仓库；后续规划 |
| 材料、Study 与 Python | [分析配置与自动化](study-and-automation.md) | 本仓库 |
| 进程、事件与数据交换 | [求解器接入](solver-integration.md) | 本仓库客户端 |
| 物理求解引擎 | [[External / 非本仓库] MoldSolver](external-moldsolver.md) | 外部仓库 |
| 共享 schema 与协议 | [[External / 非本仓库] Mold Protocol](external-mold-protocol.md) | 候选外部仓库 |
| 交付阶段与验收 | [里程碑与验证](milestones-and-validation.md) | 本仓库 |

## 当前实现与目标边界

仓库已有 Qt/QML 桌面应用、Rust 工程服务、STL 导入与 Mesh sidecar、`.pa` 工具链、GUI/CLI 命令以及 VTK WebGPU 视口。STEP 的 native shape 仓库和 Netgen 复用尚未形成持久化几何资产或完整网格生成流程；Study 求解闭环与外部引擎也未实现。平台验收和具体功能状态以[任务索引](../task-index.md)及责任任务为准；本组文档说明模块边界和目标，不维护重复的验收记录。产品阶段目标见[里程碑](milestones-and-validation.md)。

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
