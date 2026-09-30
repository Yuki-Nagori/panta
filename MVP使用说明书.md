# 工作区充填 MVP 使用说明书

- 分支：`preview`
- 流程：创建工程 → 导入 Cover → Mesh → Gate Location Analyze → Fill Analyze → Results Play。
- 默认算例：`target/Moldfill_HITL_v1/examples/cover/cover_fast.case.yaml`。
- 现在使用 main 的正常工作区；独立 **Filling MVP** 页面已移除。

## 1. 准备与启动

当前工作区已有求解器和 `target/moldfill-venv`，无需重复安装。新工作区需要单独准备外部求解器；Panta 仓库不会分发忽略目录下的求解器、Python 环境或大模型文件。不要在演示前执行 `cargo clean`。

首次准备环境时，在仓库根目录执行（macOS / Linux）：

```sh
uv venv --python python3 target/moldfill-venv
uv pip install --python target/moldfill-venv/bin/python -e target/Moldfill_HITL_v1
cargo build --locked
cargo run --locked
```

Python 版本要求以求解器的 `pyproject.toml` 为准。Qt / VTK 构建环境见 [README](README.md)。代码修改后要退出旧窗口，再启动最新构建；已经打开的窗口不会自动加载新原生代码。

## 2. 按默认值演示

1. **New Project** 创建工程，进入 Home。
2. **Import** 导入 `target/Moldfill_HITL_v1/examples/cover/cover.STL`，单位选 **Millimeters**，网格类型使用 **Dual Domain**。点击导入零件，使其页签处于活动状态。
3. 点击 Home 的 **Mesh** 或 Tasks 中的 Mesh，打开 Tools 网格面板。保留 **Global edge length = 12 mm**，点击 **Mesh**。成功后视口显示真实重划网格与三角边线，Tasks 的 Mesh 显示完成标记。
4. 在 **Analysis Sequence** 选择 **Gate Location**。打开 **Process Settings**，保留模具 **40 °C**、熔体 **230 °C**、**Advanced gate locator**、**Number of gates = 1**，确认。
5. 点击 **Analyze**。求解器只执行配对与自动浇口推荐；成功后工程树出现 **cover.STL (Gate Location)** 子方案，红色球形标记显示选中的推荐位置。完整候选、节点和配对诊断见 Solver Log / `gate_recommend.yaml`。
6. 在 **Analysis Sequence** 切到 **Fill**。**Select Material** 确认默认 **POLYFLAM RIPP 3625 CS1**。打开 **Process Settings**，保留下面的默认值并确认；保压曲线保留两行：`0 s / 100%`、`10 s / 100%`。
7. 点击 **Analyze**。本次复用刚才的重划网格和推荐节点，真实执行充填。结束后出现 **cover.STL (Fill)** 子方案，结果目录提供 **Fill time** 和 **Pressure**。
8. 点击结果目录中的 **Fill time**，VTK 显示充填时间云图、最大时间、`[s]` 色标。切换到顶部 **Results**，点击 **Play** 回放充填前沿；支持 Pause、Stop、首帧、末帧与时间轴。
9. 点击 **Pressure** 查看最终节点压力云图与 `[MPa]` 色标。它是最终压力场，不是逐帧压力动画；峰值压力摘要是整个充填过程的峰值，两者数值可以不同。

浇口子方案只改变当前视图；后续 Analyze 仍使用同一导入零件的当前方案参数。建议一次演示只操作一个零件。求解产物会落盘，但子方案页签、运行日志列表和结果选择目前是会话状态；重新打开工程后需要重新运行流程。

## 3. 默认参数与生效范围

| 设置 | 默认值 | 当前计算行为 |
|---|---|---|
| 求解路线 | surface-pair | 实际调用外部求解器 |
| 网格边长 | 12 mm | 实际调用 MMG 重划；边长是目标尺寸，不是强制每条边都为 12 mm |
| 厚度范围 | 2.4–15 mm | 来自算例 |
| 模具 / 熔体温度 | 40 / 230 °C | 分别使用当前 Gate Location / Fill 工艺设置 |
| 流量 | 94.7 cm³/s | Fill 工艺设置映射到 YAML |
| V/P 切换 | 已充填体积 99% | 映射到 YAML 的 `by_volume: 0.99` |
| 压力 / 锁模限制 | 140 MPa / 350 t | 来自算例 |
| 自动浇口 | `mode: auto`, `top_k: 3` | 推荐最多 3 个候选，选择首个候选作为一个实际浇口 |
| 热计算 | 非等温，`nz: 12` | 来自算例 |
| 快速数值参数 | Picard 4、CFL 0.9、步长增长 0.5 | 与算例一致 |
| 保压 / 纤维取向 / 结晶 | 界面保留默认配置 | 当前只求解 Fill，不计算这些阶段 / 物理量 |

**Number of gates** 是实际浇口数量，不是候选 `top_k`。本次集成支持 1 个实际浇口；填入其他数量会给出明确错误，不会将多个候选伪装为多浇口充填。Gate Location 的算法标签映射到求解器的 `auto` 推荐方法。

每次运行由 `cover_fast.case.yaml` 生成独立 `case.yaml`；几何改为当前工程导入的 STL，材料路径转为绝对路径，界面中已生效的温度、流量、V/P 和网格尺寸覆盖对应参数。Fill 的 `gate.mode` 改为 `nodes`，使用 Gate Location 的 `resolved_gate_nodes`。检查本次 `case.yaml` 即可核对真实输入；不会锁定模板 YAML 哈希。

## 4. 进度、日志与结果

- 网格运行期间，Mesh 工具面板显示循环进度条。当前 MMG 不输出真实百分比；产物写入阶段来自求解器事件，不伪造线性进度。
- 下方 Mesh Log / Analysis Log 摘要显示当前阶段、已用时间和可用的充填百分比；视口不再显示浮动进度卡片。结束后日志保留终态与总耗时。
- 正在运行时再次点击 **Analyze** 可以停止；Mesh 面板的 **Cancel** 也可停止。关闭窗口时需先停止当前作业。
- Logs 的 **Mesh Log / Analysis Log** 显示参数、状态、耗时和结果摘要；**Solver Log** 显示真实求解器输出。长路径自动换行，可选择复制。
- 蓝 → 青 → 黄 → 红表示到达时间从早到晚；回放时灰色区域表示尚未充填。动画使用真实 `fill_time_s` 字段在线性三角面内插值，约 10 秒播完整段，不在 GUI 中重新求解。
- Pressure 使用求解器导出的最终 `pressure_Pa` 字段，转换成 MPa。只列出已读到的真实结果，不生成虚假的冷却、翘曲等结果。

默认 Cover 的本次独立验证结果为 **1.915 s** 物理充填时间、**2.147 MPa** 峰值压力。它们不等于电脑实际运行时间。本轮不做性能验收。

## 5. 输出、重试与限制

每次操作写入工程所在目录：

```text
analyses/<import-id>/run-<pid>-<timestamp>/
  request.json             # 冻结的 GUI 参数
  case.yaml                # 实际运行的 YAML
  console.log              # 完整 stdout / stderr
  gate-display.json        # 浇口运行的显示位置，mm
  runs/<run-id>/
    manifest.json
    logs/events.jsonl
    logs/run.log
    artifacts/...
```

每次使用独立目录，只有成功提交的 manifest 与哈希验证通过的产物才发布到界面。STL 后缀大小写都可读取。网格生成成功后再次划网格，会使旧浇口和充填结果失效；需要重新按 Gate Location → Fill 运行。失败或取消保留上一有效网格，不显示为成功。

| 现象 | 处理 |
|---|---|
| Generate the mesh first | 先在 Mesh 工具中成功生成网格 |
| Run Gate Location Analyze first | 先切到 Gate Location 并成功 Analyze |
| Open the imported part | 激活导入零件页签后再操作 |
| 缺少 solver environment | 准备 `target/Moldfill_HITL_v1` 与 `target/moldfill-venv` |
| 几何配对或充填失败 | 检查本次 Solver Log 和 `console.log`；默认演示请使用 cover.STL |
| 界面仍是旧布局 / 旧错误 | 退出旧窗口，重新构建并启动 |

其他毫米单位 STL 可以尝试真实重划，但充填仍使用 Cover 的材料、厚度范围和默认工艺，不能据此保证任意模型的配对和求解成功。当前不包含保压、冷却、翘曲、完整工程 Study 持久化、历史结果重新加载或性能调优。
