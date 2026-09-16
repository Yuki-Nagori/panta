# 架构总览

[任务索引](../task-index.md) · [技术规范](../standards/README.md)

## 主题导航

| 主题 | 文档 | 范围 |
|---|---|---|
| 目录与模块归属 | [目录规划](repository-layout.md) | 本仓库 |
| Cargo/CMake、运行与部署 | [构建与开发](build-and-development.md) | 本仓库 |
| QML、ViewModel 与交互 | [界面与桥接](ui-and-bridge.md) | 本仓库 |
| STEP、拓扑与 OCCT | [几何模型](geometry.md) | 本仓库 |
| Mesh IR、Netgen 与质量 | [网格](mesh.md) | 本仓库 |
| VTK、视口与字段 | [可视化](visualization.md) | 本仓库 |
| Rust、工程与持久化 | [应用平台与存储](application-and-storage.md) | 本仓库 |
| 材料、Study 与 Python | [分析配置与自动化](study-and-automation.md) | 本仓库 |
| 进程、事件与数据交换 | [求解器接入](solver-integration.md) | 本仓库客户端 |
| 物理求解引擎 | [[External / 非本仓库] MoldSolver](external-moldsolver.md) | 外部仓库 |
| 共享 schema 与协议 | [[External / 非本仓库] Mold Protocol](external-mold-protocol.md) | 候选外部仓库 |
| 交付阶段与验收 | [里程碑与验证](milestones-and-validation.md) | 本仓库 |

## 文档状态与命名

本组文档依据用户提供的架构讨论附件整理。附件将桌面平台称为 MoldCAE / `moldcae`；本仓库名称是 `panta`，因此以下以 panta 指代该桌面平台。MoldCAE 是原方案名称，不代表已存在的本地目录或可执行文件。

当前仓库已有 Rust workspace 骨架（任务 001：Cargo workspace、依赖锁与 launcher）；C++、Qt/渲染与 native 构建尚未开始。本组文档描述目标架构与建议契约，不宣称功能已实现；具体依赖版本、协议编码和持久化格式仍须在实施时验证并确定。文档中的字段名和 API 名用于说明语义，不是已发布接口。

## 第一阶段目标

先实现 Geometry / Preprocessor + CAE UI + Visualization Platform：STEP → Geometry → Mesh → Visualization → Study Setup → Save Project。第一阶段不依赖真实 CFD 引擎完成；使用带明确标识的合成标量场验证结果显示。

V1 应覆盖工程新建、打开、保存，STEP 导入，面/边/实体选择，Netgen 网格生成，表面与体网格显示，属性编辑，相机、裁剪、色标和求解器占位入口。V1 不实现 AI、自研 GPU 计算后端或物理求解内核。VTK/Qt 正常使用图形硬件不属于排除范围中的 GPU 计算开发。

## 分层和依赖方向

```text
QML 界面
  → C++ ViewModel / Controller
    → Application Services
      ├─ C++ Geometry / Mesh / RenderScene → OCCT / Netgen / VTK adapters
      └─ Rust Project / Workflow / Storage / Solver Client
                                                ↓ 进程协议
                                  [External] MoldSolver
```

界面表达用户意图，服务协调业务，领域模型表达几何、网格、工程和结果，适配器封装第三方库。库对象不能成为跨层公共数据模型。Rust 和 C++ 在应用服务边界协作，不应为每次渲染调用形成 QML → C++ → Rust → C++ → VTK 的往返链路。

## 技术基线

技术选型、版本验证状态和责任任务集中维护在 [技术基线](../standards/baseline.md)，编码与库使用规则见 [规范索引](../standards/README.md)。Rust/C++ 桥接优先验证 [CXX](../standards/cxx.md)，尚未完成集成；架构文档只说明模块职责与交互，不另维护一份版本表。

## 外部边界

[求解器客户端](solver-integration.md) 属于本仓库；[MoldSolver](external-moldsolver.md) 的 FEM/FVM/DG、物理模型和 CPU/CUDA 后端不属于本仓库。[Mold Protocol](external-mold-protocol.md) 是可选的共享契约仓库，尚未确定独立创建。远程和云端求解只作为接口扩展方向，不列入 V1 已承诺能力。
