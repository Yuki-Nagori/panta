# 目录与模块规划

[架构总览](README.md)

## 当前与目标

下图只列出仓库中实际存在的主要目录；crate 与 native target 的依赖关系以[分层规范](../standards/layering.md)及其自动检查为准。几何、材料、工作流等尚未创建的 crate 不列入目录树。

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
├── rust-toolchain.toml        # 固定 Rust 工具链
├── crates/
│   ├── launcher/              # 应用入口与 native 构建调度
│   ├── panta-build/           # 工具供给、构建与质量 runner 支持
│   ├── panta-cli/、panta-dslc/ # 应用命令与 .pa CLI
│   ├── panta-core/            # 工程服务、运行时与操作编排
│   ├── panta-dsl-core/        # .pa parser、Artifact、诊断与生成
│   ├── panta-ffi/             # CXX 契约与 Rust staticlib
│   ├── panta-foundation/      # 进程级设施
│   ├── panta-import/          # 导入准备与来源快照
│   └── panta-mesh/            # Surface/Volume Mesh IR 与校验
├── native/
│   ├── CMakeLists.txt、CMakePresets.json
│   ├── app/、bridge/、ffi/、qt-adapter/、i18n/
│   ├── foundation/、geometry/、mesh/、visualization/
│   ├── cmake/                 # Qt / SDK 供给与构建辅助
│   └── performance/           # 手动运行的性能 target
├── qml/
│   ├── App.qml
│   ├── Components/{Atoms,Composites}/
│   ├── Dialogs/、Panels/{Ribbon,Sidebar}/、Themes/
│   ├── branding/、icons/
├── prototypes/shell/          # HTML/SVG 原型；shell.js 也是 QML 回归资源输入
├── resources/{i18n,materials}/
└── tests/                     # Cargo 编排及测试；详见 tests/README.md
    ├── rust/、cpp/、qml/       # 按语言 / 模块组织功能回归
    ├── integration/           # Cargo native/QML 聚合入口
    ├── support/               # 跨功能 / 性能测试共享的 Rust、Qt、QML 辅助代码
    ├── fixtures/              # 固定输入与来源 / 单位 / 预期
    └── performance/           # cpu/、gpu/、support/；cargo performance 手动运行
```

目录树只用于定位当前文件。crate、native target 与 QML module 的职责和允许依赖集中维护在[分层标准](../standards/layering.md)；`.pa` 工具链见[DSL 模块说明](../modules/dsl-engine-and-toolchain.md)，CXX 构建边界见[构建说明](build-and-development.md)。新增职责只在相应架构主题和 task 中记录，避免在目录图重复维护依赖表。

## 新增文件原则

按职责放置文件，避免按语言把所有业务堆进一个桥接层。访问私有实现的单元测试贴近源码；公开 API 的 Rust 黑盒测试放根 `tests/rust/` 并由所属 crate 注册，native/QML 测试和回归夹具按[测试规范](../standards/testing.md)归档。测试资产应小且能合法分发，大型 CAD/结果数据通过受控的外部资产机制管理。

工程名与 Rust package 名已统一：工程为 `panta`，Rust package 用 `panta-` 前缀，当前 launcher 的 bin 名为 `panta-launcher`（任务 001）。桌面可执行文件已统一命名 `panta`；Python package 名由后续 API 任务确定；不直接沿用附件中的 `moldcae-desktop` 等示意名称。
