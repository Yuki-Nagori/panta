# 092 — 方案分析序列与网格类型接线

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：057, 067, 080, 091
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-09-30

## 目标与范围

Rust 管理分析序列目录及各导入零件的已确认选择，通过 CXX 和 ProjectViewModel 投影到 QML。弹窗只维护候选值，确认后由 Rust 校验并事务写入 `.panta`。网格类型目录与导入默认值统一由 `panta-import` 提供，导入弹窗通过同一 ViewModel 消费；所选类型沿用导入记录 `mesh_type`；Tasks 根据当前导入文档展示，Welcome / 无活动导入时由 Rust 选择最近导入记录。所选类型不表示相应网格已生成，不提前建立求解执行 FSM。

## 实施与验收

- [x] Rust 序列目录替换 QML 固定列表；未知序列和不存在的记录拒绝修改。
- [x] 每个零件独立保存选择，重开恢复；持久化失败保留旧状态。
- [x] Tasks 切换零件时同步显示网格类型和分析序列。
- [x] QML 仅维护弹窗候选状态；C++ 只提供类型适配与通知。
- [x] 功能、资源、翻译、文档和测试一致，Cargo 聚合检查通过。
- [ ] 真实窗口验收多零件切换、确认和取消后的最终显示。

## 验证计划与结果

2026-09-30，在仓库根目录、macOS 26.3.1 arm64 / Qt 6.11.2 执行：

| 检查 | 实际结果 |
|---|---|
| `cargo test --locked --workspace` | 通过，CTest 66/66；Rust 覆盖每记录保存、重开、无效命令、同值确认和写盘失败，Bridge 覆盖多零件 / Welcome 投影，QML 覆盖键盘确认、取消和重新打开 |
| `cargo format --check`、`cargo lint`、`cargo build --locked` | 全部通过；lint 需要本机锁服务的 TCP 监听权限 |
| `node --check ai-docs/qml-html/shell.js`、`git diff HEAD --check` | 通过 |
| CPU / GPU 基准目标构建 | 通过；目录由真实 ProjectViewModel 注入，不复制测试目录 |
| `QT_QPA_PLATFORM=offscreen QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/qml/panta_qml_cpu_benchmark measures_analysis_sequence_construct` | 通过，31 次采样；空白 / 面板 / 完整弹窗 p50/p95 为 12.9/15.5、164.4/300.4、523.0/779.9 µs。仅构造与布局，无可见窗口；不据此比较版本性能或推断 GPU 表现 |
| 真实窗口 / GPU | 同 091：临时 app wrapper 两次连接超时，GPU 启动无可用 screen；未取得真实窗口及帧呈现证据，保持 in-progress |

## 决策与工作记录

- 2026-09-30：Review 聚焦通知、绑定与持久化；快照 getter 不跨 FFI，领域投影只在离散状态变化刷新。覆盖表改为有序映射，避免元数据输出次序漂移。
- 2026-09-30：确认 `ImportRecord.mesh_type` 已由导入事务持久化，分析序列是方案配置，不将目录选项直接建模为 FSM 状态。
- 2026-09-30：网格类型目录和默认值集中到 `panta-import`，Qt 缓存只读目录；选中索引改为单一 ListView 属性。检查通知链：`planSettingsChanged` 不触发刷新或写入，快照差异门控避免重复通知；无渲染逐帧领域调用。

## 清理与兼容例外

移除 QML 领域目录和已确认选择；执行编排暂不接入。工程中的分析序列覆盖表允许省略，省略表示领域默认 Fill，不新增格式迁移分支。
