# 092 — 方案分析序列与网格类型接线

- 状态：done
- 阶段：应用平台扩展
- 依赖：057, 067, 080, 091
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-30 / 2026-10-02

## 目标与范围

Rust 管理分析序列目录及各导入零件的已确认选择，通过 CXX 和 ProjectViewModel 投影到 QML。弹窗只维护候选值，确认后由 Rust 校验并事务写入 `.panta`。网格类型目录与导入默认值统一由 `panta-import` 提供，导入弹窗通过同一 ViewModel 消费；所选类型沿用导入记录 `mesh_type`；Tasks 根据当前导入文档展示，Welcome / 无活动导入时由 Rust 选择最近导入记录。所选类型不表示相应网格已生成，不提前建立求解执行 FSM。

## 实施与验收

- [x] Rust 序列目录替换 QML 固定列表；未知序列和不存在的记录拒绝修改。
- [x] 每个零件独立保存选择，重开恢复；持久化失败保留旧状态。
- [x] Tasks 切换零件时同步显示网格类型和分析序列。
- [x] QML 仅维护弹窗候选状态；C++ 只提供类型适配与通知。
- [x] 功能、资源、翻译、文档和测试一致，Cargo 聚合检查通过。
- [x] 真实窗口验收多零件切换、确认和取消后的最终显示。

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

- 2026-10-02：按优先级继续本任务，补齐双零件实际窗口切换与分析序列候选确认 / 取消验收。使用独立临时工程及已知 STL 夹具，分别设置 Dual Domain / Solid 3D 网格类型；不修改用户工程，不将无头回归代替窗口证据。
- 2026-09-30：Review 聚焦通知、绑定与持久化；快照 getter 不跨 FFI，领域投影只在离散状态变化刷新。覆盖表改为有序映射，避免元数据输出次序漂移。
- 2026-09-30：确认 `ImportRecord.mesh_type` 已由导入事务持久化，分析序列是方案配置，不将目录选项直接建模为 FSM 状态。
- 2026-09-30：网格类型目录和默认值集中到 `panta-import`，Qt 缓存只读目录；选中索引改为单一 ListView 属性。检查通知链：`planSettingsChanged` 不触发刷新或写入，快照差异门控避免重复通知；无渲染逐帧领域调用。

## 清理与兼容例外

移除 QML 领域目录和已确认选择；执行编排暂不接入。工程中的分析序列覆盖表允许省略，省略表示领域默认 Fill，不新增格式迁移分支。

## 2026-10-02 真实窗口收尾验收

环境为仓库根目录、macOS / Apple M4 / Qt 6.11.2、native Debug。`cargo build --locked` 通过；按仓库约定使用临时启动脚本 wrapper，切换到仓库目录并 exec `target/native/debug/app/panta-native`，通过 CUA 检查真实窗口。夹具为 `/private/tmp/panta-gui-092/gui-acceptance-092/gui-acceptance-092.panta`：复制此前临时验收工程与自制单三角形 STL，补第二导入记录及独立资产文件；初始第一零件为 Dual Domain / Gate Location，第二零件为 Solid 3D / 默认 Fill。夹具构造不作为 STL 导入流程验收，用户工程未改动。

| 操作 | 实际窗口结果 |
|---|---|
| 从原生选择器打开工程，停留 Welcome | Tasks 显示最近导入的 `part-b.stl`，Mesh (Solid 3D)、Fill |
| 双击 `part-a.stl` | 文档激活，Tasks 显示该零件、Mesh (Dual Domain)、Gate Location |
| 打开分析序列，选 Fill + Pack 后 Cancel | Tasks 仍为 Gate Location；重新打开候选也恢复 Gate Location |
| 再选 Fill + Pack 并 OK | 弹窗关闭，第一零件任务行更新为 Fill + Pack |
| 激活第二零件 | Tasks 仍为 Solid 3D / Fill，第一零件修改未串入第二零件 |
| 点击第一零件文档页签，再点击 Welcome | 第一零件恢复 Dual Domain / Fill + Pack；Welcome 返回第二零件 Solid 3D / Fill |

确认后磁盘清单的 `analysis_sequences` 只有 `import-1: fill-pack`，修订从 7 增至 8，两个网格类型保持原值。窗口 / AX 证据保存在忽略目录 `artifacts/task-092/`：`part-a-initial.ax.txt`、`part-a-cancel.ax.txt`、`part-a-confirmed.png`、`part-b-isolated.png`、`part-b-isolated.ax.txt`、`part-a-tab-return.ax.txt`、`welcome-latest.ax.txt`。已退出应用，进程查询无匹配，临时 wrapper 已删除；保留验收数据和证据。

复核既有 Rust 每记录事务测试、Bridge 方案投影及 QML 确认 / 取消回归；最近一次 `cargo test --locked --workspace` 全部通过（69/69 CTest），`plan_settings_are_per_record_and_transactional` 成功，日志 `/tmp/panta-103-tests-reviewed.log`；格式与静态检查已通过，详见 103。本批仅补窗口验收与文档，不修改产品代码，不重复运行代码测试。最新三平台 CI 证据为 [run 36932703454](https://github.com/Yuki-Nagori/panta/actions/runs/36932703454)（`bf0ecfb`），19 个 job 全部成功，已由本轮前序任务通过 gh 核验；不表示后续未 push 提交已在远端运行。

整体 review 确认候选取消、逐记录配置、Welcome 回退和文档切换与原任务规则一致；`git diff HEAD --check` 通过，任务与索引同步 done。未新增代码或兼容例外。

## 完成摘要

Rust 目录、逐记录持久化、网格类型投影及 QML 候选接线已完成；自动化回归与双零件真实窗口切换、确认 / 取消验收均通过。所选网格类型仍只表示导入配置，本任务不生成网格或执行求解。
