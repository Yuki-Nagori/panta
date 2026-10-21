# 重要模块设计

[任务索引](../task-index.md) · [架构总览](../architecture/README.md) · [技术规范](../standards/README.md)

当前 crate、native target 与 QML 的依赖分组见[分层标准](../standards/layering.md)和[仓库布局](../architecture/repository-layout.md)；Rust 应用命令入口及 GUI/CLI 边界见[命令层设计](../architecture/command-layer.md)。

本目录保存跨任务的重要模块设计：职责、数据流、生命周期、方案取舍与演进门槛。架构文档维护全局分层；规范维护通用约束；实施进展与验证证据仍以 task 为准，不在这里复制状态表。

本目录同时记录已实现基础和后续规划。路径服务（023）、工程运行时与变量 DSL（024/025）、静态 QML 模块注册（026）、原子 / 组合组件（029）、`.pa` parser / formatter（034/035）及 Rust FSM 的只读 STL 视口激活（073/080）已落地。主题 DSL（030，ready）、QML 重载（027）、设置界面与持久化（087/115/125）及增量更新（037）尚未实现；状态以任务记录为准。

| 方向 | 设计说明 | 实施任务 |
|---|---|---|
| 原子组件、统一尺寸与主题切换 | [组件库与主题 DSL](qml-components-and-theme.md) | [029](../task/029-qml-component-library.md)、[030](../task/030-theme-dsl.md) |
| 用户设置、平台菜单入口与语言设置 | [国际化](internationalization.md)、[应用与存储架构](../architecture/application-and-storage.md) | [115](../task/115-platform-application-menu-and-settings-entry.md)、[125](../task/125-settings-service-and-qt-adapter.md)、[087](../task/087-language-settings-ui.md) |
| 英文源文案、语言字典与切换 | [国际化](internationalization.md) | [022](../task/022-ui-internationalization.md) |
| 跨平台路径、逻辑资源地址与运行上下文 | [路径与运行时](paths-and-runtime.md) | [023](../task/023-cross-platform-paths.md)、[024](../task/024-runtime-context.md) |
| 变量声明、表达式与快照 | [变量 DSL](variable-dsl.md) | [025](../task/025-variable-dsl.md) |
| 静态库、QML 注册与状态重载 | [模块与热重载](qml-modules-and-reload.md) | [026](../task/026-static-qml-modules.md)、[027](../task/027-qml-state-reload.md) |
| 高 DPI 缩放与多显示屏 | [显示缩放与多屏](display-scaling-and-multi-monitor.md) | [033](../task/033-display-scaling-and-multi-monitor.md) |
| 跨语言格式、测试、审计与质量门禁 | [质量工具链](quality-tooling.md) | [032](../task/032-cross-language-quality-gates.md) |
| `.pa` 解析、聚合与 TS/QM 工具链 | [DSL 解析与 Artifact 引擎](dsl-engine-and-toolchain.md) | [034](../task/034-rust-panta-artifact-parser.md) |
| `.pa` 格式化与校验 | [DSL 解析与 Artifact 引擎](dsl-engine-and-toolchain.md) | [035](../task/035-pa-formatter-and-validator.md) |
| 编译期 FSM 声明、Rust 状态机与异步事务 | [FSM 与状态机](fsm.md) | [072 评估](../task/072-flow-state-machine-planning.md)、[073 实施](../task/073-fsm-dsl-and-import-state-machine.md) |
| Mesh 资产 revision、旁置存储与按需加载 | [工程存储](../architecture/application-and-storage.md)、[网格架构](../architecture/mesh.md) | [088](../task/088-mesh-asset-sidecar-storage.md) |
| Qt 交互状态机准则与导入窗口 | [Qt 交互状态机](qt-interaction-state-machines.md) | [074](../task/074-qt-interaction-state-machine.md) |
| 打包后软件内增量更新、签名、回滚 | [软件内增量更新](incremental-updates.md) | [037](../task/037-incremental-update-foundation.md) |
| VTK/OCCT/Netgen 预编译 SDK 与受信制品 | [Native 依赖供给](native-dependency-supply.md) | [031](../task/031-prebuilt-native-dependencies.md)、[038](../task/038-native-sdk-artifact-production.md) |
| 性能基线、剖析与回归对比（开发侧，非 CI 门禁） | [性能测试与剖析](performance.md) | [048](../task/048-performance-testing.md) |
