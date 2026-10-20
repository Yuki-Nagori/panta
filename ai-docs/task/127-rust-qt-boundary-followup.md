# 127 — Rust 与 Qt 职责边界收尾

- 状态：in-progress
- 阶段：Rust 架构整理
- 依赖：057, 080, 088, 092, 107, 117, 124
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-20 / 2026-10-20

## 目标与范围

修复职责审计发现的工程 GUI 同步 I/O、FFI 激活收尾编排、导入提醒重复策略，以及已登记的大网格显示 DTO 同步物化成本。Rust 应用服务拥有任务、提交和工作区结果；Qt 负责展示快照、QObject 生命周期及排队通知。同步 CLI 服务继续共用领域实现。

必读：[职责边界](../architecture/native-domain-boundaries.md)、[分层](../standards/layering.md)、[CXX](../standards/cxx.md)、[注释](../standards/comments.md)、[验证与评审](../standards/validation-and-review.md)。

## 实施与验收

- [x] 创建、打开、保存和分析序列确认的 GUI 路径后台执行 I/O；返回值明确表示准入，完成通知再更新界面；失败保留旧工程及输入。
- [x] 资产激活的过期判断、缓存提交、attempt 收尾和工作区更新集中在 core；FFI 只适配最终结果，非 FFI 消费者无需重写规则。
- [x] 大网格显示 DTO 在后台准备，以拥有式不可变结果交付；过期工程、关闭或较新选择拒绝迟到显示结果；GUI 不等待线程退出，无新增轮询器。
- [x] Rust 预览统一返回展开大小估算及非阻断警告，GUI 与 CLI 共用策略；无文件大小硬上限。
- [x] 同步文档中的工作区权威和 STEP 进程内 shape 仓库描述，保留 Qt/VTK 原生生命周期职责。
- [x] 受影响回归、格式、lint、窗口适用验收与独立只读评审完成；更新任务和索引。

## 清理与依赖

移除 GUI 同步 I/O 接线、FFI 中重复的激活业务裁决、GUI/CLI 重复资源估算及旧的 GUI 显示 DTO 物化路径。不保留旧 API 别名，无兼容例外；不新增 crate 依赖或 Qt 运行时模块。新增模块同步检查测试/质量选择器归属。

## 验证计划与结果

以 `cargo test-plan` / `cargo verify-changed` 的实际影响范围为准，覆盖后台启动/容量拒绝、失败回滚、迟到结果、关闭释放、警告边界及 GUI 完成通知。真实窗口验收区分平台可用条件；性能只报告实际测量口径。

2026-10-20，仓库根目录，macOS arm64 / Qt 6.11.2 / 仓库固定 Rust 工具链：

| 场景 | 结果与边界 |
|---|---|
| Rust 行为测试 | core 工程后台命令 11/11；后续整合 core 155 项单元、FSM 激活 16/16、activation 集成 11/11、FFI 33 项单元通过。Mesh 19/19、import 8/8，含估算阈值及真实编码布局核对。 |
| 构建 / 原生回归 | `cargo build --locked` 通过。`cargo test-scope all` 的 Rust package 测试均通过；原生 CTest 首轮 90/92 通过，含 Bridge 30/30。两处失败为旧同步失败断言和新增 QML 对话框缺静态 Bridge plugin；修复后精确重跑 `Qml.SettingsLanguageSwitch` / `Qml.ThemeComponentParameters`，2/2 通过。测试辅助目录和公共 CMake 输入使选择器保守扩大到完整范围；修复后只重跑受影响项。 |
| 质量检查 | `cargo verify-changed` 执行的格式、Clippy、machete、CMake lint、分层图及禁止边夹具、qmllint 均通过；完整 clang-tidy 59 个自有翻译单元通过，完整 includes 扫描发现一处无用头文件并移除。后续受影响翻译单元的 clang-tidy / includes、完整 cppcheck、CMake lint / layering、`cargo format --changed --check` 通过。原聚合命令曾因已修复项退出非零，不记作一次完整成功。 |
| 性能目标 | CPU 激活、GPU 工程视口和 fixture 三个手动目标按最终代码重新链接通过；GPU 基准本轮未运行，不以编译代替 GPU 实测。 |
| macOS 真实窗口 | 按 AGENTS 绑定 `target/native/debug/app/panta.app` 并检查 AX/截图；临时工程位于 `/private/tmp/panta-127-gui/BoundaryReview12`。无效名称创建失败保留输入，成功创建进入 Home；原生选择器选择 `cover.STL` 后预检、导入、模型显示及 Welcome/cover 切换正常；分析序列从 Fill 改为 Fill + Pack，完成后关闭窗口并更新 Tasks。未测最低配置延迟或 Windows/Wayland 实机。验收后关闭应用，沙箱外进程查询确认无验收进程。 |
| 独立只读评审 | `gpt-6.1-sol` medium 检查完整 diff、新模块及文档。首轮发现完成通知重入会清除下一请求错误；已先清理旧错误再释放 pending，并新增确定性启动失败回归。最终复核确认该修复、过期显示结果的释放顺序及两处测试接线成立，未发现新增可操作问题；评审不替代执行测试。 |

本地适用验收已完成。状态保持 `in-progress`，PR 的 CI 验收通过后同步为 `done`；不宣称 Windows / Wayland 实机或最低配置帧预算通过。

### 后台路径性能测量

macOS 26.3.1 arm64、Qt 6.11.2、Debug。用最终重新链接的 `panta_performance_fixture` 在临时目录建立 10,000 / 20,000 面合成模型与用户提供的 `Large_Earth_Elemental.stl`（608,218 面）工程；CPU 基准 `panta_bridge_project_stl_activation_cpu_benchmark <临时工程.panta> 31` 退出 0。大文件只读，临时工程及日志在记录结果后清理。

| 测量范围 | 结果 |
|---|---|
| 工程打开，31 次 | 包含 ViewModel 构造的请求准入 p50 / p95 为 0.158 / 0.232 ms；包括后台读取、校验及排队发布的完成时延为 0.469 / 0.584 ms。 |
| 工程保存，31 次 | 请求准入 p50 / p95 为 0.020 / 0.026 ms；包括原子写入、同步及排队发布的完成时延为 4.797 / 10.974 ms。 |
| 大模型首次激活 | 1,244.082 ms 墙钟 / 1,234.953 ms 进程 CPU，包含 sidecar 激活、后台显示准备及排队发布；活动显示 DTO 容量 41.76 MiB。未单独分解解码与展开时间。 |
| 缓存命中选择，93 次 | 无缓存未命中；选择调用 p50 / p95 为 0.210 / 3.798 ms，选择至显示快照就绪为 5.807 / 127.097 ms。仍包含旧显示缓冲释放和 Qt 元数据更新，不能宣称全部同步调用达到 1 ms。 |
| 大模型关闭重开，31 次 | 热 OS 文件缓存下墙钟 p50 / p95 为 1,219.203 / 1,358.273 ms，进程 CPU 为 1,221.257 / 1,320.823 ms。 |
| 磁盘与释放 | 清单 3,459 B、sidecar 合计 17,761,070 B；进程峰值 RSS 186.86 MiB。全部关闭后 Rust 驻留估算归零，旧 C++ 快照弱引用释放断言通过；进程销毁服务后的 RSS 为 182.52 MiB，不能把逻辑释放等同于分配器立即向 OS 归还内存。 |

测量范围为 ViewModel、Rust 激活和 CPU 显示 DTO，不含 VTK / GPU。首次激活仅单次，保存 / 重载使用热文件缓存；关闭仅三个样本，不设时间门槛。本机 Debug 数据不与历史 Release 结果直接比较，不替代最低配置、真实 GPU 或跨平台性能验收。

## 决策与工作记录

- 2026-10-20：维护者授权修复完整审计结果，使用专用分支；不自动提交或推送。
- 2026-10-20：补充实际调用链发现的路径检查 I/O：导入列表仅投影已保存相对路径，读取前的包含/symlink 校验移到激活 worker，保留原稳定错误分类。新增模块和共享 Qt 测试辅助目录补选择器回归；当前公共测试/CMake 输入会保守扩大验证范围。
- 2026-10-20：维护者授权最终评审后提交、创建 PR 并等待 CI 合并。追加的 `gpt-6.1-sol` medium 独立只读评审覆盖完整 diff、新文件、消费者及文档，未发现新增可操作问题；按已记录的本地验证结果提交，远程 CI 待验收。

## 完成摘要

工程 GUI I/O 与大 Mesh 显示准备已迁到后台，资产激活提交由 core 收尾；GUI / CLI 共用 Rust 导入提醒策略，架构说明和相关任务已同步。测试覆盖请求准入和最终完成、失败保留状态、通知重入、迟到结果、所有者销毁及显示快照释放。未新增生产轮询器、crate 依赖或运行时 Qt 模块。剩余为提交与 PR CI 验收。
