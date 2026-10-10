# preview 分支：工作区充填演示

- 状态：实现与自动验证完成，真实窗口显示待用户复核。
- 使用说明：[MVP使用说明书.md](MVP使用说明书.md)。
- 本分支按用户约定只维护根目录文档，不新增 `ai-docs/task/` 文件。

## 当前流程与边界

以 main 已有 GUI 为主，使用普通工作区的 Import、Mesh、Analysis Sequence、Process Settings、Analyze、Logs 和 Results。独立 `FillingPreviewPanel`、固定四步入口和底部播放条已移除。

```text
导入 cover.STL → MMG 重划
  → Gate Location Analyze (--gate-only)
  → Fill Analyze (--mesh + 已推荐的 gate nodes)
  → Fill time / Pressure 云图 → Results Play
```

默认模板为 `target/Moldfill_HITL_v1/examples/cover/cover_fast.case.yaml`。GUI 的默认温度、流量、V/P 切换与边长和模板一致；其余材料、几何配对与数值设置来自模板。运行时保存独立 YAML，不锁定模板哈希。候选 `top_k: 3` 与实际单浇口数量分开；本轮不实现多浇口充填。

Rust `panta-solver` 在线程中管理外部进程、取消、事件和成功产物校验。`tools/moldfill/driver.py` 只将冻结参数转换成求解器 YAML / CLI，计算仍由外部引擎完成。CXX 转发请求和批量显示数据；C++ `AnalysisModel` 投影 Qt 状态，VTK 负责浇口标记、字段显示与逐帧动画，QML 只发语义命令。

浇口与充填各自产生工程树子方案和真实运行目录。子方案视图、日志列表、选中结果与活动求解状态目前限于当前会话；工程重开不自动恢复。结果树只发布真正读取到的 Fill time 与 Pressure。保压、纤维取向、结晶等配置保留在 main 界面中，未参与本次 Fill 计算。

## 2026-10-01 合并与迁移记录

- `preview` 已成功变基到 main 的 `b0d93e0`，保留 main 工艺设置、分析日志、结果界面和文档页签；3 处冲突均已解决。
- 网格、浇口推荐和充填分别执行；Fill 复用同一重划网格与 `resolved_gate_nodes`，不会再次自动选浇口。
- 播放控制迁到 Results；选择 Fill time / Pressure 显示真实字段及单位色标。运行状态和计时在日志摘要显示。
- 用户试用发现小写 `_remeshed.stl` 没有被原大写后缀匹配识别。已修复大小写匹配，仍检查产物唯一性、路径安全与 manifest 哈希。
- 网格工具增加循环进度条；MMG 当前无完成百分比，日志只显示真实阶段 / 产物写入和计时。
- 根据最新试用反馈删除视口浮动进度卡片与相关属性 / 渲染代码；日志分为参数 / 结果摘要和完整 Solver Log，长路径换行，摘要定时更新。
- 充填进度合并到阶段行，避免百分比重复；结果切换先更新显示快照，再通知文档。同一 Fill 页签重复激活保留当前字段，增加 Fill time / Pressure 色标往返回归。
- 清理旧 ViewModel、面板、播放条、资源注册、固定四步性能场景及失效插件定义；无兼容分支。

## 验证记录

2026-10-10：公共 `ActionButton` 统一判断 `clickAction`：已配置动作则执行，未配置则请求英文提示。`ThemedToolButton`、`RibbonTile` 和网格动作按钮共用该入口；菜单 / Ribbon / 任务按 key 分发的未实现分支同样请求提示。保留真实动作及运行前提禁用状态；本次验证待更新。

- 独立真实求解：Mesh → Gate Location → Fill 全流程通过，充填时间 **1.9150767935 s**，峰值压力 **2.1465074803 MPa**。节点时间与最终压力字段数量和三角面节点映射已校验。
- 自动浇口显示位置验证：真实 Mesh / Gate 推荐通过，确认选中的三个坐标分量来自推荐产物。
- 用户 `Frame.stl` 重划回归：真实求解器产出 **15,744 个三角面**，Rust 读取并发布成功；输出保留在 `target/moldfill-lowercase-regression/`。
- 单元回归覆盖分行 JSONL、成功 manifest / 哈希 / 路径安全、STL 后缀大小写、压力单位与节点映射、字段显示与回放切换、色标范围与隐藏。
- 最终 `cargo build --locked`、`cargo test --locked --workspace`（含 67 项 native / QML 检查）、`cargo format --check`、`cargo lint` 均通过。
- 按用户要求不启动真实 GUI 窗口验收；用户截图记录了试用问题，修复后的原生窗口显示仍需用户复核。本轮不运行 CPU/GPU 性能场景。

求解器和本机产物位于忽略的 `target/`，不会自动随仓库提交分发。本次外部求解器本机 HEAD 为 `ef062b4`；不将其源码或 ABI 链接进 Panta。
