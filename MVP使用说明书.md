# 充填演示 MVP 使用说明书

- 分支：`preview`
- 用途：使用固定 Cover 示例，展示「载入模型 → 生成网格 → 充填计算 → 结果回放」。
- 入口：Panta 窗口中，功能区下方的 **Filling MVP · default example**。
- 分析、实现范围与验证记录：[preview.md](preview.md)。

## 1. 本机准备

本次开发已在当前工作区准备好求解器环境。已有 `target/moldfill-venv` 时，不必重复安装。

如果重新准备工作区，需要先将相同版本的求解器放到 `target/Moldfill_HITL_v1`。本次验证的提交为 `022cfa39a6c15a8da1f7f33063a38e8dc422d7ad`，程序版本 `0.3.1.dev1`。求解器目录、Python 环境和运行结果均位于忽略的 `target/`，单独拉取 Panta 仓库不会自动获得它们，也不要在演示前运行 `cargo clean`。

在 Panta 根目录执行（macOS / Linux）：

```sh
uv venv --python python3 target/moldfill-venv
uv pip install --python target/moldfill-venv/bin/python -e target/Moldfill_HITL_v1
cargo build --locked
./target/native/debug/app/panta-native
```

Python 要求以求解器的 `pyproject.toml` 为准；本机已验证 Python 3.14.7、NumPy 2.5.3、SciPy 1.18.1、PyYAML 6.0.3、meshio 5.3.5、mmgpy 0.17.0。Panta 自身的 Qt / VTK 等构建环境按 [README](README.md) 准备。

本次实际验收平台为 macOS arm64。Windows 尚未进行真实窗口验收，启动路径和环境安装命令不能直接照搬上述 Unix 示例。

## 2. 按默认值完成一次演示

不需要先创建工程，不需要选择 STL、材料或填写边界条件。

| 操作 | 界面应出现什么 |
|---|---|
| 点击 **Filling MVP · default example** | 左侧四步流程、固定参数；右侧显示区域和底部回放条 |
| 点击 **Load default example** | Cover 模型，原始网格 164 个三角面 |
| 点击 **Generate default mesh** | 实际调用 MMG 重划；完成后显示网格边线，本机默认结果为 2,638 个三角面 |
| 点击 **Start filling with defaults** | 后台启动求解器；显示阶段、真实充填进度及运行耗时；运行时按钮禁用以防重复提交 |
| 等待 **Filling complete** | 显示已充满、物理充填时间、峰值压力及充填时间云图；底部回放按钮可用 |
| 点击 **Play filling** | 从起点回放充填过程；再次点击 **Pause** 可暂停 |
| 拖动底部时间轴 | 查看任意物理时刻的已充填区域；到结尾后再点击播放即可重播 |

左侧面板内容超出窗口高度时，可以滚动查看操作、结果和产物目录。**Back to workspace** 返回原工作区，当前演示状态保留；再次进入可继续。**Start over** 清空演示显示和步骤，重新开始；已经落盘的运行记录仍保留。

## 3. 固定参数与示例来源

直接复用求解器自带的 [快速非等温 Cover 算例](target/Moldfill_HITL_v1/benchmarks/cover_noniso_quick.case.yaml)，没有另外生成演示模型。

| 项目 | 值 |
|---|---|
| 模型 | [cover.STL](target/Moldfill_HITL_v1/examples/cover/cover.STL)，202 × 6 × 152 mm |
| 材料 | [POLYFLAM RIPP 3625 CS1](target/Moldfill_HITL_v1/materials/polyflam-ripp-3625-cs1.yaml) |
| 路线 / 网格 | surface-pair / MMG，目标边长 12 mm |
| 浇口 | nearest，(102, 3, 82) mm |
| 熔体 / 模具温度 | 230 / 40 °C |
| 注射流量 | 94.7 cm³/s |
| V/P 切换 / 限制 | 99% 体积；140 MPa；350 t |
| 热计算 | 非等温，厚度方向 12 层 |
| 输出 | 当前运行的最终结果及 VTK 场 |

启动器固定使用 CPU 快速预设：`MOLDFILL_DEVICE=cpu`、`MOLDFILL_PRESSURE_DEVICE=cpu`、`MOLDFILL_PRESSURE_SOLVER=direct`、`MOLDFILL_FAST_SOLVER=1`、`MOLDFILL_PICARD_MAX=4`、`MOLDFILL_CFL=0.9`、`MOLDFILL_DT_GROWTH=0.5`。无需用户设置环境变量；其他继承的 `MOLDFILL_*` 覆盖会被清除。

为保证只读摘要与真实输入一致，程序校验默认配置、模型和材料文件的 SHA-256。不要直接修改这三份文件来调参；文件被修改后会明确报错，不会继续按旧摘要运行。

## 4. 怎样读结果

- **Elapsed**：本次操作的电脑运行耗时，单位秒。
- **Filling time**：模拟中的物理充填时间，单位秒。
- **Peak pressure**：本次计算的峰值压力，单位 MPa。
- 云图颜色从蓝、青、黄到红表示熔体到达时间从早到晚，灰色表示当前回放时刻尚未充填。
- 回放依据真实结果中的 `fill_time_s` 字段，在三角面内部线性插值；不是在界面中重新计算流场。完整回放约 10 秒，时间轴显示的仍是物理时间。
- 首版先完成计算再回放；运行期间显示进度，网格视图不实时显示流动前沿。

本机首次命令行基线的纯求解阶段约 212 秒，完整 run 约 215 秒；结果为 `filled: true`、物理充填时间 **1.915 s**、峰值压力 **2.147 MPa**。GUI 验收的一次运行在并发构建/检查期间用了 **344.7 秒**，产出的 YAML 摘要和 VTK 文件与 CLI 基线逐字节一致。耗时会随硬件和并发负载变化，不作为固定性能保证。12 mm 粗网格和快速预设用于展示流程；工程精度评估及 3 mm 标准算例不属于本次 MVP。

CPU 没有占满所有核心是当前求解路径的正常现象：时间步及主要迭代按顺序执行，CPU 稀疏直接解法不会自动把整段求解变成多进程；部分 NumPy / SciPy 底层运算可使用库自身线程。增加线程环境变量不等于完整求解会按核心数加速，本次没有修改物理内核的并行实现。

## 5. 停止、重试与排查

计算时点击 **Stop** 会终止并回收当前求解进程；保留上一步有效状态，原按钮可重试该步骤。退出应用也会回收该应用启动的求解进程。失败和取消不会显示为“充填完成”。

每次生成网格或求解分别使用新的输出目录：

```text
target/moldfill-preview/gui-<进程号>-<时间戳>/
  preview-settings.json
  console.log
  runs/<run-id>/
    manifest.json
    logs/events.jsonl
    logs/run.log
    artifacts/...
```

左侧 **Run artifacts** 下方是本次目录，可以选中文本复制。重划产物会供下一步直接复用，充填阶段不会再重划网格。结果只从当前操作的成功 manifest 读取，并校验产物哈希；不会通过搜索“最近的结果”复用旧运行。

| 现象 | 处理 |
|---|---|
| 缺少 solver environment | 检查第 1 节的求解器目录和 Python 虚拟环境 |
| Default input changed | 恢复本说明指定版本的默认配置、STL 或材料文件 |
| 阶段进度暂时未变化 | 看 Elapsed 是否继续更新；查看该目录的 `console.log` 和 `logs/events.jsonl` |
| 求解失败或没有充满 | 查看界面错误和当前运行日志，再重试；不要靠放宽质量门禁来制造成功 |
| 看不到新入口或回放控件 | 重新构建并启动最新的 native 可执行文件；不要继续使用构建前已打开的窗口 |

本版不提供任意零件导入求解、参数编辑、保压/冷却/翘曲流程、工程包内仿真持久化或结果重新打开入口。保存的 VTK、YAML、CSV 和日志仍可在输出目录中检查。
