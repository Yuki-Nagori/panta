# 目录与模块规划

[架构总览](README.md)

## 当前与目标

当前存在根目录文档、许可证、忽略配置、`ai-docs/`、任务 001 落地的 Cargo workspace 骨架（根 Cargo.toml、Cargo.lock、rust-toolchain.toml 与 `crates/launcher`）、任务 003/004 落地的 native 构建骨架，以及任务 005 落地的 Qt 桌面骨架（`native/app`+`native/bridge`、根 `qml/` 模块与 Qt 供给脚本）。下图中其余应用源码与构建部分为规划，不应据此创建无用途的占位模块。

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
│   ├── launcher/              # 统一运行入口（任务 001 骨架：仅未接入诊断）
│   ├── core/                  # 规划：通用 ID、错误和应用契约
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
│   ├── geometry/{core,occt}/
│   ├── mesh/{core,netgen}/
│   └── visualization/{core,vtk}/
├── qml/
│   ├── App.qml                # 主窗口（005 已落地；URI Panta.Shell，NO_PLUGIN 资源模块）
│   ├── Components/
│   ├── Panels/                # PlaceholderPanel 占位（005 已落地）
│   ├── Viewport/
│   └── Themes/                # Theme 单例（005 已落地）
├── python/                    # 后续 Python API，包名待定
├── schemas/                   # 本地工程 schema、外部契约版本引用
├── resources/                 # 图标、主题、样例等资源
└── tests/                     # 跨模块场景与回归数据
```

## 模块归属

`core` 保持小而稳定，只存放多模块确实共享的基础契约，不能变成所有业务逻辑的容器。`project` 管理实体关系、修订与命令；`workflow` 管理执行过程；`storage` 管理读写和资产引用；`solver-client` 管理外部进程及事件转换。

C++ 各模块的 `core` 定义自有类型与行为，`occt`、`netgen`、`vtk` 目录实现适配。ViewModel 不应承担几何修复、网格算法或文件格式解析。QML 组件处理布局、状态展示和交互绑定。

`schemas/` 若包含共享协议，应明确其上游来源、版本及生成方式，不能与候选外部协议仓库分别维护两个权威版本。跨语言 FFI 优先验证 CXX，实际 crate、生成工具及目录由任务 006 确定，不预设它们已经存在。QML 模块可在 qml/ 下设置自己的 CMakeLists.txt，由 native 顶层纳入构建。

## 新增文件原则

按职责放置文件，避免按语言把所有业务堆进一个桥接层。模块内部测试可贴近实现；跨模块端到端验证放在 `tests/`。测试资产应小且能合法分发，大型 CAD/结果数据通过受控的外部资产机制管理。

工程名与 Rust package 名已统一：工程为 `panta`，Rust package 用 `panta-` 前缀，当前 launcher 的 bin 名为 `panta-launcher`（任务 001）。Python package 名与最终桌面可执行文件名在相应脚手架任务（014 / 005）搭建时确定；不直接沿用附件中的 `moldcae-desktop` 等示意名称。
