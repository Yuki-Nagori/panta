# 127 — Rust 与 Qt 职责边界收尾

- 状态：done
- 阶段：Rust 架构整理
- 依赖：057, 080, 088, 092, 107, 117, 124
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-20 / 2026-10-20

## 目标与范围

工程 GUI I/O 与大 Mesh 显示准备移到后台；core 拥有激活提交和工作区收尾，FFI / Qt 适配结果；GUI / CLI 共用 Rust 导入提醒。职责规则见[架构说明](../architecture/native-domain-boundaries.md)、[分层](../standards/layering.md)和[性能测量边界](../modules/performance.md)。

PR #70 已完成首批验收；合并后复查发现 GPU 基准仍把切换时的空帧算作目标帧，本批重新打开任务修复测量正确性。Rust 工具链升级另记任务 001。

## 验收

- [x] 创建 / 打开 / 保存 / 分析序列在后台准备，返回准入结果；完成后发布，失败保留旧状态和输入。
- [x] 过期 attempt、缓存提交与工作区终态归 core；FFI 只适配最终结果。
- [x] 后台构造拥有式显示 DTO，最多一个在途准备，按选择及 generation / revision 拒绝迟到结果；GUI 不等待线程退出。
- [x] Rust 统一展开字节估算与 2 GiB 非阻断提醒，无文件大小硬上限；同步工作区权威和 STEP shape 仓库说明。
- [x] GPU 基准确认目标 Mesh 身份及对应场景帧，拒绝空帧和旧帧；确定性回归、硬件渲染及独立评审通过。
- [x] 本批 PR CI 通过，任务与索引同步为 done。

## 验证证据

均在仓库根目录、macOS 26.3.1 arm64 / Qt 6.11.2 执行；下列初轮与 GPU 修复使用 Rust 1.98.1 / Debug，工具链升级后的结果见任务 001。

| 批次 / 检查 | 结果与限制 |
|---|---|
| PR #70：Rust / native | `cargo build --locked` 和 Rust package 回归通过；core 155 项单元、FSM 16、activation 集成 11、FFI 33、mesh 19、import 8。CTest 首轮 90/92，修正旧同步断言与缺失静态插件后精确复验 2/2；Bridge 30/30。 |
| PR #70：质量 / 窗口 | 格式、Clippy、machete、CMake lint、分层夹具、qmllint、clang-tidy / includes 59 个翻译单元及 cppcheck 通过；无用头文件修复后复验。真实窗口检查了创建失败保留输入、成功创建、cover.STL 预检 / 导入 / 显示、页签切换和分析序列保存；结束后确认无验收进程。 |
| PR #70：独立评审 / CI | `gpt-6.1-sol` medium 发现完成通知重入可能清除下一请求错误；修复并补回归后复核通过。实现提交 `fb90eab` 的 [CI 39418808793](https://github.com/Yuki-Nagori/panta/actions/runs/39418808793) 15/15 成功，[PR #70](https://github.com/Yuki-Nagori/panta/pull/70) 已合并。 |
| GPU 修复：确定性回归 | `ProjectViewModelTest.GpuSamplingRejectsEmptyFramesAndWaitsForTheTargetScene` 通过；真实 ViewModel 持有后台准备任务，依次模拟空帧、DTO、旧帧和目标帧，检查异 Mesh 拒绝及旧 DTO 弱引用释放。 |
| GPU 修复：硬件 / 窗口 | 10,000 / 20,000 面合成模型及 608,218 面真实 STL；工程 GPU 基准默认离屏与 `PANTA_BENCHMARK_PRESENTATION=1` 各运行每模型 3 样本，均退出 0，完成最终空场景等待。离屏确认 Metal 硬件 adapter。`PANTA_TEST_NATIVE_VIEWPORT=1 .../panta_qml_viewport_module_test native_refresh_and_window_lifecycle` 通过，100 次合并更新只提交一帧且报告修订 100，原有生命周期断言保留。 |
| GPU 修复：质量 / 聚合 | `cargo build --locked`、格式、Clippy、machete、CMake lint、分层、qmllint、完整 clang-tidy / includes 59 个翻译单元及 cppcheck 通过；新测试的直接包含缺失修复后复验。`cargo test-scope all` 退出 0，Rust package 回归及 CTest 93/93 通过。新共享 header 补选择器回归，验证范围保守扩大。 |
| GPU 修复：最终评审 | `gpt-6.1-sol` medium 只读核对完整 diff、Mesh / 修订关联、场景合并、信号重入、弱引用及所有等待点，无新增可操作发现。 |

实现提交 `e819de0` 的 [CI 39425918094](https://github.com/Yuki-Nagori/panta/actions/runs/39425918094) 15/15 成功；[PR #71](https://github.com/Yuki-Nagori/panta/pull/71) 文档收尾同步任务关闭，不重复运行已验证实现的测试。

质量聚合曾因已记录的直接包含缺失退出非零，修复后按阶段复验，不宣称一次聚合全绿。硬件运行先在沙箱内因无屏幕失败，沙箱外两模式及原生窗口测试通过。GPU 三样本验证测量流程，不建立性能基线或跨模式等值结论；窗口报告 VTK 提交，离屏报告队列完成，均不代表 GPU 内核时间。未宣称 Windows / Wayland 实机或最低配置帧预算通过。临时工程和日志在记录结果后清理。

## CPU 历史测量（PR #70）

使用 `panta_performance_fixture <新目录> /Users/yuki/Downloads/Large_Earth_Elemental.stl` 生成 10k / 20k 面合成模型及 608,218 面真实模型工程，再运行 `panta_bridge_project_stl_activation_cpu_benchmark <工程.panta> 31`；打开 / 保存 / 重载各 31 次、缓存命中选择 93 次。不含 VTK / GPU，不与旧 Release 结果直接比较。

| 范围 | 结果 |
|---|---|
| 打开 / 保存 | 准入 p50 / p95（打开包含 ViewModel 构造）：打开 0.158 / 0.232 ms，保存 0.020 / 0.026 ms；完成时延分别 0.469 / 0.584、4.797 / 10.974 ms。 |
| 大模型 / 缓存命中 | 首次激活墙钟 / 进程 CPU 为 1,244.082 / 1,234.953 ms，活动 DTO 41.76 MiB；热缓存重载墙钟 p50 / p95 为 1,219.203 / 1,358.273 ms，进程 CPU 为 1,221.257 / 1,320.823 ms。无缓存未命中的选择调用 p50 / p95 为 0.210 / 3.798 ms，就绪为 5.807 / 127.097 ms；仍含旧 DTO 释放和 Qt 更新，不能宣称所有同步调用达到 1 ms。 |
| 磁盘 / 内存 | 清单 3,459 B、sidecar 合计 17,761,070 B；峰值 RSS 186.86 MiB。关闭后 Rust 驻留估算归零、旧 C++ 快照弱引用释放；销毁服务后 RSS 182.52 MiB，逻辑释放不等于分配器立即向 OS 归还内存。 |

首次激活仅单次、关闭仅三个样本，重载使用热 OS 文件缓存；未单独分解解码与展开时间，不设时间门槛。

## 清理与完成摘要

已删除 GUI 同步工程 I/O、FFI 激活业务编排、重复导入估算和 GPU 基准仅按全局帧数增长的判定。无兼容层、无新增生产轮询器、crate 依赖或 Qt 运行时模块；工作区、路径后台校验、STEP 仓库与相关任务说明已同步。GPU 修复的本地硬件 / 回归验收及本批三平台 PR CI 均通过，任务关闭。
