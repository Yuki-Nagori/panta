# 100 — tests 整体整理与性能入口统一

- 状态：done
- 阶段：验证基础
- 依赖：048, 099
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-01 / 2026-10-02

## 目标与范围

整体整理仓库测试体系：盘点 Rust、C++、QML 功能测试、性能基准、夹具和辅助代码，统一目录职责、命名、构建注册、运行入口与文档；清理重复实现和失效引用，分批验证与提交。GPU 基准统一作为其中一个子项，Qt Quick 首批离屏实现由 099 提供。

功能测试按现有语言与模块组织，通过 `cargo test --locked --workspace` 聚合。跨测试复用的辅助代码放 `tests/support`；性能专用代码放 `tests/performance/support`；QML 夹具保留 `tests/fixtures/qml`。逐项检查现有文件的归属与使用者，迁移有明确复用价值的代码，避免为目录整齐引入重复封装。

离屏仍使用硬件 GPU；不得把 CPU 提交日志、屏幕呈现或同步完成混称 GPU 执行时间。用户授权：同机同场景的数值一致时可删除真实窗口性能模式；099 首轮比较未证明等价，暂保留。输入、窗口归属和渲染生命周期行为测试继续使用真实窗口。

CPU / GPU 性能源码集中到 `tests/performance/{cpu,gpu,support}`，文件名去掉目录已表达的分类。同步 Cargo/CMake 路径和质量工具源码清单，保留手动基准目标名。CMake 性能目标集中到 `native/performance/CMakeLists.txt`；通过 `cargo performance` 按性能辅助代码正确性测试 → CPU → GPU 运行，提供合成 STL 工程夹具。性能辅助正确性测试与基准不注册到常规 Cargo/CTest 聚合测试，性能数值不纳入常规 CI 时间门禁。

## 必读

- [性能模块](../modules/performance.md)
- [原生职责边界](../architecture/native-domain-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 实施步骤与验收

- [x] 完成 tests 全目录及相关 Cargo/CMake 注册的归属审计，确认功能测试、性能测试、夹具和辅助代码职责一致。
- [x] 测试目录职责明确；跨功能测试 / 性能测试共享代码迁入 `tests/support`，性能专用代码留在 `tests/performance/support`。
- [x] 所有性能源码集中到 CPU / GPU 目录，去掉文件名重复分类；QML 夹具保留原目录。
- [x] 独立 CMake 维护性能目标；`cargo performance` 按 support → CPU → GPU 聚合，常规测试不执行性能辅助测试。
- [x] 更新质量检查源码清单、路径引用和长期文档，清理旧注册和重复实现。
- [x] 盘点全部 GPU 性能入口、后端、测量边界和资源释放顺序。
- [x] 共享真实可复用的初始化、计时和统计代码；Qt Quick RHI 与 VTK WebGPU 的后端边界明确。
- [x] 对可离屏的场景实施硬件离屏路径，记录不能迁移的具体条件。
- [x] 原生窗口与离屏模式在相同配置下验证；不一致时保留独立基线。
- [x] 每批聚合验证、性能实测、文档和 task 一致；无未使用辅助类或兼容占位。
- [x] 整体 review 测试代码与注释，核查夹具所有权、清理顺序、失败传播和测试隔离；清理失效路径与重复注册。

## 决策与工作记录

- 2026-10-01：任务从 GPU 基准统一扩展为 tests 整体整理，并从 `100-gpu-benchmark-consolidation.md` 更名为当前文件。099 首批 Qt Quick 离屏与窗口计时未证明等价，保留窗口对照。
- 2026-10-01：`55436fb` 完成性能目录 / 命名 / 独立 CMake / Cargo 入口整理，迁移共享 QML、Qt、Rust 辅助代码。性能辅助测试退出常规聚合入口；该批 lint 8 项、native/QML 67 项、CLI 3 项、C++ 支持测试 6 项及 CPU 套件通过。
- 2026-10-01：全目录审计确认功能测试按模块注册，Rust 私有项测试保留所属模块，CMake configure 夹具按规范保留 `native/cmake/tests`。修正 testing 规范的旧归档表述；STEP 空文件改用只读零字节夹具，清理固定临时路径。
- 2026-10-01：两个 VTK 基准共用隐藏硬件 surface；提取产品私有几何、材质、相机配置供离屏共用，新增 CPU 回归。初始化和 VTK 错误明确失败，错误标志由回调命令持有。Qt Quick 校验硬件设备与可视负载，修复呈现变量值 `0` 的误判；窗口先析构，再释放渲染回调状态。
- 2026-10-01：VTK 窗口首批少一个样本的检查暴露旧相邻日志计时边界不准确，改为逐次场景更新至提交；捕获器只保留计数与等待，删除时间戳接口和缓存，补充重置断言。Qt Quick 行数原先在预热前读取，改为预热后统计并断言可视负载，内容尺寸显式同步；两种模式均实现 Tasks 23 行 / 组合场景 19 行。
- 2026-10-01：窗口退出曾在晚到鼠标事件中崩溃，定位为 AppKit 保留的 view 引用已销毁 hardware window。解绑时先清空反向指针并隐藏 view，补充保留 view 后销毁窗口、派发晚到事件的回归。已核对 SDK 精确版本 [VTK Cocoa view 源码](https://github.com/Kitware/VTK/blob/23f0a095621e91bbdbeace8451e22b950c8e5f46/Rendering/UI/vtkCocoaHardwareView.mm)，不修改第三方 SDK。

## 最终验证

仓库根目录；macOS 26.3.1 / Apple M4 / Metal，Qt 6.11.2、VTK 9.7.0。native 采用 Debug，Rust 网格微基准采用 Release；合成 STL 为 10k / 20k 平面三角形，工程采样各 3 次。Qt Quick / VTK 固定场景预热 30 帧，正式采样 3 × 60 帧。图形命令使用实际 Cocoa 会话；沙箱无屏幕的启动失败不计为通过。

| 日期 | 命令 / 场景 | 实际结果 / 证据 |
|---|---|---|
| 2026-10-01 | `cargo test --locked --workspace` | Rust / doc-tests 通过，native/QML 69/69；日志 `/tmp/panta-100-tests-final2.log`。原生图形场景需单独启用，见下两项。 |
| 2026-10-01 | `cargo performance --samples 3` | 按 support → CPU → GPU 全部通过；CLI 3 项、C++ 支持 7 项，Rust mesh 3 项，QML CPU 7 项、导航 3 项，QML GPU 5 项、VTK GPU 3 项及工程 CLI 通过；日志 `/tmp/panta-100-performance-final.log`。 |
| 2026-10-01 | `cargo performance gpu --presentation --samples 3` | 支持测试、QML GPU 5 项、VTK GPU 3 项及工程窗口 CLI 全部通过，无跳过；日志 `/tmp/panta-100-presentation-final.log`。 |
| 2026-10-02 | `PANTA_TEST_NATIVE_VIEWPORT=1 QT_QPA_PLATFORM=cocoa target/native/debug/app/panta_qml_viewport_module_test` | 5 项通过，无跳过；日志 `/tmp/panta-100-viewport-final.log`。 |
| 2026-10-02 | `PANTA_TEST_NATIVE_VIEWPORT=1 QT_QPA_PLATFORM=cocoa target/native/debug/visualization/panta_native_surface_lifecycle_test` | 3 项通过，无跳过；日志 `/tmp/panta-100-cocoa-final.log`。 |
| 2026-10-02 | `cargo lint --check` | 全部 8 项通过；日志 `/tmp/panta-100-quality-final.log`。 |
| 2026-10-02 | `cargo format --check`、`git diff HEAD --check` | 通过；格式日志 `/tmp/panta-100-format-final.log`。 |

本机 p50/p95（ms），只记录本轮结果，不宣称性能改善或设 CI 时间门槛：

| 场景 / 指标 | 离屏 | 窗口 |
|---|---|---|
| Control Gallery 24 项，GPU 时间戳 | 0.127 / 0.129 | 1.085 / 1.197 |
| Control Gallery 24 项，CPU 更新至离屏完成 / 窗口呈现间隔 | 0.642 / 0.751 | 16.655 / 17.246 |
| VTK 欢迎场景，CPU 更新至提交 | 6.861 / 7.920 | 16.662 / 16.728 |
| VTK 欢迎场景，CPU 更新至队列完成 | 8.360 / 12.442 | 不采集此指标 |

STL CPU 重载 10k / 20k 的 p50/p95 为 13.323/13.550 与 24.752/25.248 ms。CPU Gallery 本轮 p95 波动明显，原始数据保留于日志；不据此推断代码回归。不同模式的调度与同步边界不同，数值未等价，保留独立基线和显式窗口入口。Windows / Linux 硬件运行与完整应用视觉验收未在本轮执行；099 的剩余验收继续独立跟踪。

## 清理与兼容

替换时清理重复基准实现，无兼容例外。性能工具不加入 CI 时间门禁。

## 完成摘要

已完成 tests 归属审计、共享辅助代码、独立性能目录与 Cargo/CMake 入口，三个 GPU 基准默认硬件离屏。整体 review 清理重复实现与失效接口，补充场景、日志及 Cocoa 生命周期回归；本机功能、质量、CPU/GPU 和窗口对照均通过。窗口模式保留独立指标，无兼容例外。
