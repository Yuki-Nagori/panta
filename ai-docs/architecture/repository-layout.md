# 目录与模块规划

[架构总览](README.md)

## 当前与目标

当前存在根目录文档、许可证、忽略配置、`ai-docs/`、Cargo workspace、
`panta-foundation` 进程基础设施、`panta-core` 领域模型、`panta-ffi` CXX
边界，以及任务 034 正在实施的 `.pa` DSL parser/CLI（`crates/panta-dsl-core`、
`crates/panta-dslc`）。任务 003/004 落地 native 构建骨架，任务 005 落地 Qt
桌面骨架；任务 007 的 VTK WebGPU 原生视口已接入默认窗口，065 补齐导航。
真实窗口的跨平台覆盖仍见各任务。下图标为规划的部分不应据此
创建无用途的占位模块。

```text
panta/
├── AGENTS.md                  # 任务路由与基本约束
├── README.md                  # 使用入口与当前状态
├── ai-docs/
│   ├── architecture/          # README.md 总览与分主题架构
│   ├── modules/               # 重要模块设计；实施进展见 task
│   ├── standards/             # 技术规范及官方依据
│   ├── task-index.md          # 任务队列与状态
│   └── task/                 # 模板与 NNN-name.md
├── Cargo.toml / Cargo.lock    # Rust workspace 与依赖锁（任务 001 已落地）
├── rust-toolchain.toml        # 固定 stable 工具链（任务 001 已落地）
├── crates/
│   ├── launcher/              # 统一运行入口与 native 构建调度
│   ├── panta-foundation/      # 进程级设施；当前为崩溃信号与日志（047）
│   ├── panta-core/            # Rust 领域模型：任务、路径等（008/023）
│   ├── panta-ffi/             # CXX DTO/句柄/错误边界与 staticlib（006/047）
│   ├── panta-dsl-core/         # .pa parser、Artifact 聚合、诊断与 TS 生成（034 实施中）
│   ├── panta-dslc/             # 单文件 .pa 校验/格式化/TS 输出 CLI（034/035 实施中）
│   ├── panta-geom/            # 规划：几何身份、修订与后端契约
│   ├── panta-mesh/            # 规划：Mesh IR、导入与轻量校验
│   ├── panta-bc/              # 规划：边界条件与目标引用
│   ├── panta-material/        # 规划：材料数据、单位与版本
│   ├── panta-visualization/   # 规划：显示配置、字段与选择语义
│   ├── project/               # 规划：工程模型与命令
│   ├── workflow/              # 规划：任务、作业和流程编排
│   ├── solver-client/         # 规划：外部求解器客户端
│   └── storage/               # 规划：持久化与数据资产索引
├── native/
│   ├── CMakeLists.txt         # native 顶层构建（003/004/005 演进：Qt 供给、defaults 函数）
│   ├── CMakePresets.json      # 单配置 Ninja presets：debug / release（任务 003 已落地）
│   ├── cmake/                 # 安装包配置模板与 Qt 供给脚本（qt-provision.cmake，005 已落地）
│   ├── foundation/            # 基础契约与构建链验证 target（003 已落地）
│   ├── bridge/                # ViewModel（ShellViewModel + GTest 信号测试，005 已落地）
│   ├── app/                   # Qt 桌面入口 panta-native（005 已落地 Qt 实现）
│   ├── geometry/{include,src/occt}/       # 当前：自有摘要契约与 OCCT adapter
│   ├── mesh/{include,src/netgen}/         # 当前：native Mesh IR 与 Netgen adapter
│   └── visualization/{include,src/vtk}/   # 当前：显示契约与 VTK 后端（含 navigation/）
├── qml/
│   ├── App.qml                # 主窗口（005 已落地；URI Panta.Shell，NO_PLUGIN 资源模块）
│   ├── Components/
│   ├── Panels/                # PlaceholderPanel 占位（005 已落地）
│   ├── Viewport/
│   └── Themes/                # Theme 单例（005 已落地）
├── python/                    # 后续 Python API，包名待定
├── schemas/                   # 本地工程 schema、外部契约版本引用
├── resources/                 # 图标、.pa 源文件与样例；TS/QM 只进构建树
└── tests/                     # 跨模块场景与回归数据
```

## 模块归属

当前 `panta-core` 已包含 project/path/task 与工程存储服务；`panta-foundation` 拥有进程设施，`panta-ffi` 负责跨语言契约和服务转发。规划的五个领域 crate 按实际功能建立，其职责、依赖方向和迁移门槛以 [重库适配与 Rust 领域模块](native-domain-boundaries.md) 为准；领域 crate 不反向依赖承载应用服务的 core，避免循环。

`project`、`workflow`、`storage` 与 `solver-client` 是后续职责拆分方向，不是已存在的独立 crate。当前不再规划另一份含义重叠的 `crates/core/`；共享基础类型有真实消费者后才确定归属。C++ `include/panta/<module>/` 暴露自有 native 契约，`src/occt`、`src/netgen` 封装重库；`src/vtk` 同时承载显示后端，导航内部归 `navigation/`。业务状态逐步归 Rust；保留本地相机、命中和窗口生命周期。ViewModel 只适配 UI，QML 处理布局、展示与绑定。

`schemas/` 若包含共享协议，应明确其上游来源、版本及生成方式，不能与候选外部协议仓库分别维护两个权威版本。跨语言 FFI 优先验证 CXX；任务 006 的 `crates/panta-ffi` 生成桥接头和 Rust staticlib，native CMake 通过 launcher 传入它们，不在 CMake 中回调 Cargo。QML 模块可在 qml/ 下设置自己的 CMakeLists.txt，由 native 顶层纳入构建。

`.pa` 源文件由 `panta-dsl-core` 统一解析，按 `kind` 聚合成 language、theme 或 variables Artifact；`panta-dslc` 编排 check、format/format --check 和 TS 输出。格式化复用核心 AST，写回前通过临时文件同步并替换；国际化构建树中的临时 TS 由 quick-xml 生成、QM 由预编译 QtTools 生成，源码目录不保存这两类产物。格式化/校验规则和 fixtures 归 035，不能在各业务目录复制 parser。

## 新增文件原则

按职责放置文件，避免按语言把所有业务堆进一个桥接层。模块内部测试可贴近实现；跨模块端到端验证放在 `tests/`。测试资产应小且能合法分发，大型 CAD/结果数据通过受控的外部资产机制管理。

工程名与 Rust package 名已统一：工程为 `panta`，Rust package 用 `panta-` 前缀，当前 launcher 的 bin 名为 `panta-launcher`（任务 001）。Python package 名与最终桌面可执行文件名在相应脚手架任务（014 / 005）搭建时确定；不直接沿用附件中的 `moldcae-desktop` 等示意名称。
