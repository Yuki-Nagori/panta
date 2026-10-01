# 100 — tests 整体整理与性能入口统一

- 状态：in-progress
- 阶段：验证基础
- 依赖：048, 099
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-01 / 2026-10-01

## 目标与范围

按用户要求整体整理 tests 的目录、共享辅助代码、构建注册、运行入口和文档，并更新全部 GPU 性能基准；分批验证与提交。Qt Quick 首批离屏实现由 099 提供；本任务覆盖 `project_docks_gpu_benchmark`、`viewport_gpu_benchmark` 和 `project_stl_viewport_gpu_benchmark` 的后续统一。

离屏仍使用硬件 GPU；不得把 CPU 提交日志、屏幕呈现或同步完成混称 GPU 执行时间。用户授权：同机同场景的数值一致时可删除真实窗口性能模式；099 首轮比较未证明等价，暂保留。输入、窗口归属和渲染生命周期行为测试继续使用真实窗口。

新增范围：按用户要求将 CPU / GPU 性能源码集中到 `tests/performance/{cpu,gpu,support}`；QML 夹具仍放 `tests/fixtures/qml`。同步 Cargo/CMake 路径和质量工具源码清单，保留手动基准目标名与测量策略。CMake 性能目标集中到 `native/performance/CMakeLists.txt`；新增 Cargo 手动入口和合成 STL 工程生成器。共享 QML 功能测试辅助代码移入 `tests/support/qml`。辅助正确性测试与基准统一通过 `cargo performance`（support → CPU → GPU）运行，不注册到常规 Cargo/CTest 聚合测试，性能数值不纳入常规 CI 时间门禁。

## 必读

- [性能模块](../modules/performance.md)
- [原生职责边界](../architecture/native-domain-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 实施步骤与验收

- [x] 测试目录职责明确；跨功能测试 / 性能测试共享代码迁入 `tests/support`，性能专用代码留在 `tests/performance/support`。
- [x] 所有性能源码集中到 CPU / GPU 目录，去掉文件名重复分类；QML 夹具保留原目录。
- [x] 独立 CMake 维护性能目标；`cargo performance` 按 support → CPU → GPU 聚合，常规测试不执行性能辅助测试。
- [x] 更新质量检查源码清单、路径引用和长期文档，清理旧注册和重复实现。

- [x] 盘点全部 GPU 性能入口、后端、测量边界和资源释放顺序。
- [ ] 共享真实可复用的初始化、计时和统计代码；Qt Quick RHI 与 VTK WebGPU 的后端边界明确。
- [ ] 对可离屏的场景实施硬件离屏路径，记录不能迁移的具体条件。
- [ ] 原生窗口与离屏模式在相同配置下验证；不一致时保留独立基线。
- [ ] 每批聚合验证、性能实测、文档和 task 一致；无未使用辅助类或兼容占位。

## 验证与工作记录

- 2026-10-01：按用户要求登记扩展范围；099 的 Qt Quick 基准已有 Apple M4 / Metal 离屏和真实窗口通过证据，两种 GPU 时间戳不同，不能据此删除呈现模式。本任务尚未修改 VTK 基准。
- 2026-10-01：首批 `c4fe8d9` 已提交。两个 VTK 入口目前统计提交日志，且分别维护全局捕获与分位数实现；先共享捕获和统计，再提取 VTK 硬件离屏场景。使用现有 VTK 9.7.0 的 `SetOffScreenRendering` / `WaitForCompletion`，不新增 SDK 或 Rust 逐帧调用。

- 2026-10-01：第二批统一三个入口的最近秩分位数，替换两个 VTK 的全局日志捕获实现；保留原日志处理器/过滤器，修复默认处理器为空时吞掉其他日志的问题，禁止嵌套和复制捕获器。补充非法样本、分位数、同步/异步帧、超时和处理器恢复单测。`cargo test --locked --workspace` 通过（native/QML 68/68）；手动基准三个目标编译通过。离屏迁移尚未完成。

- 2026-10-01：按用户追加要求扩大为 tests 整体整理，统一性能目录 / 命名 / CMake 注册，新增 `cargo performance`，顺序为辅助代码 → CPU → GPU。跨功能 / 性能的 QML 树与目录辅助函数、VTK 日志捕获、Rust 临时目录迁入 `tests/support`。修复 Cargo 的 `--target-dir` 透传位置，避免改写外部日志规则或持锁调用原日志处理器。QML 夹具保留原目录；性能辅助测试不进入常规聚合入口。
- 2026-10-01：本批最终验证：`cargo lint --check` 全部 8 项通过，`cargo test --locked --workspace` 通过（native/QML 67/67，性能 CLI / Rust 微基准按要求 ignored），`cargo performance support` 的 CLI 3 项与 C++ 6 项通过；`cargo format --check`、`git diff HEAD --check` 通过。日志分别为 `/tmp/panta-100-batch2-{lint,tests,support}-final.log` 与 `/tmp/panta-100-batch2-format-check.log`。
- 2026-10-01：`cargo performance cpu --samples 3` 全部通过（`/tmp/panta-100-cpu2.log`）：Rust Release 网格 3 项；native Debug / Qt 6.11.2 的 QML 7 项、VTK 导航 3 项、合成 STL 工程基准通过。Control Gallery 24 项构造 p50/p95 为 8.612/9.226 ms；10k / 20k 三角形重载 p50/p95 为 13.057/13.074 与 24.716/24.788 ms。仅记录本机基线，不宣称性能改善；本批不包含 VTK 离屏迁移或 GPU 对照完成。

## 清理与兼容

替换时清理重复基准实现，无兼容例外。性能工具不加入 CI 时间门禁。

## 完成摘要

目录、共享辅助代码和独立性能入口已整理并验证；继续 VTK 硬件离屏、GPU 对照和最终 review。
