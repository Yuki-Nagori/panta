# 重要模块设计

[任务索引](../task-index.md) · [架构总览](../architecture/README.md) · [技术规范](../standards/README.md)

本目录保存跨任务的重要模块设计：职责、数据流、生命周期、方案取舍与演进门槛。架构文档维护全局分层；规范维护通用约束；实施进展与验证证据仍以 task 为准，不在这里复制状态表。

以下模块均为规划，接口、语法和新目录均未落地；已有桌面骨架不代表这些能力已经实现。

| 方向 | 设计说明 | 实施任务 |
|---|---|---|
| 原子组件、统一尺寸与主题切换 | [组件库与主题 DSL](qml-components-and-theme.md) | [029](../task/029-qml-component-library.md)、[030](../task/030-theme-dsl.md) |
| 英文源文案、语言字典与切换 | [国际化](internationalization.md) | [022](../task/022-ui-internationalization.md) |
| 跨平台路径、逻辑资源地址与运行上下文 | [路径与运行时](paths-and-runtime.md) | [023](../task/023-cross-platform-paths.md)、[024](../task/024-runtime-context.md) |
| 变量声明、表达式与快照 | [变量 DSL](variable-dsl.md) | [025](../task/025-variable-dsl.md) |
| 静态库、QML 注册与状态重载 | [模块与热重载](qml-modules-and-reload.md) | [026](../task/026-static-qml-modules.md)、[027](../task/027-qml-state-reload.md) |
| 高 DPI 缩放与多显示屏 | [显示缩放与多屏](display-scaling-and-multi-monitor.md) | [033](../task/033-display-scaling-and-multi-monitor.md) |
| `.pa` 解析、聚合与 TS/QM 工具链 | [DSL 解析与 Artifact 引擎](dsl-engine-and-toolchain.md) | [034](../task/034-rust-panta-artifact-parser.md) |
| `.pa` 格式化与校验 | [DSL 解析与 Artifact 引擎](dsl-engine-and-toolchain.md) | [035](../task/035-pa-formatter-and-validator.md) |

优先推进 022、023、026；024 在路径与基础任务服务就绪后推进，025 建立在运行时之上，027 最后验证 UI 重载。它们不成为现有 M0 视口主线的额外前置条件。

组件库 029 可基于已有桌面骨架先推进；025 + 029 → 030 完成主题 DSL 接入，不依赖 027 引擎热重载。
