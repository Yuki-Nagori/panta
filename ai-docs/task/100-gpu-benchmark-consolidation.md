# 100 — GPU 基准共享与离屏验证

- 状态：in-progress
- 阶段：验证基础
- 依赖：048, 099
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-01 / 2026-10-01

## 目标与范围

按用户要求更新全部 GPU 性能基准，审阅实现、抽取实际共享的资源和统计代码，分批验证与提交。Qt Quick 首批离屏实现由 099 提供；本任务覆盖 `project_docks_gpu_benchmark`、`viewport_gpu_benchmark` 和 `project_stl_viewport_gpu_benchmark` 的后续统一。

离屏仍使用硬件 GPU；不得把 CPU 提交日志、屏幕呈现或同步完成混称 GPU 执行时间。用户授权：同机同场景的数值一致时可删除真实窗口性能模式；099 首轮比较未证明等价，暂保留。输入、窗口归属和渲染生命周期行为测试继续使用真实窗口。

## 必读

- [性能模块](../modules/performance.md)
- [原生职责边界](../architecture/native-domain-boundaries.md)
- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [仓库文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 实施步骤与验收

- [ ] 盘点全部 GPU 性能入口、后端、测量边界和资源释放顺序。
- [ ] 共享真实可复用的初始化、计时和统计代码；Qt Quick RHI 与 VTK WebGPU 的后端边界明确。
- [ ] 对可离屏的场景实施硬件离屏路径，记录不能迁移的具体条件。
- [ ] 原生窗口与离屏模式在相同配置下验证；不一致时保留独立基线。
- [ ] 每批聚合验证、性能实测、文档和 task 一致；无未使用辅助类或兼容占位。

## 验证与工作记录

- 2026-10-01：按用户要求登记扩展范围；099 的 Qt Quick 基准已有 Apple M4 / Metal 离屏和真实窗口通过证据，两种 GPU 时间戳不同，不能据此删除呈现模式。本任务尚未修改 VTK 基准。

## 清理与兼容

替换时清理重复基准实现，无兼容例外。性能工具不加入 CI 时间门禁。

## 完成摘要

进行中；先完成 099 提交，再按后端逐批实施本任务。
