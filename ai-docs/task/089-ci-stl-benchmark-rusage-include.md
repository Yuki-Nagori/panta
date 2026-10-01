# 089 — CI 修复：STL benchmark RSS 头文件识别

- 状态：done
- 阶段：验证基础
- 依赖：[048 性能基线与性能测试体系](048-performance-testing.md)、[080 视口文档页签与 STL 按需激活](080-qml-viewport-document-tabs.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-29 / 2026-10-02

## 目标与背景

GitHub Actions run [36574286051](https://github.com/Yuki-Nagori/panta/actions/runs/36574286051) 的 `cargo lint (includes, ubuntu)` 失败：两个 080 STL benchmark 中的 `rusage` 用法被 LLVM include-cleaner 报告为缺少直接提供头。其他 17 项检查成功，1 项依赖审计按路径条件跳过。

本任务修复两个 benchmark 的 Linux 峰值 RSS 采样入口，改读内核公开的 `/proc/self/status` `VmHWM` 字段；它定义为进程 RSS 高水位，单位为 kB。macOS 继续使用 `getrusage`，Windows 继续使用 `GetProcessMemoryInfo`。Linux 分支不再引用 CI 无法归属头文件的 `rusage` 类型，不调整测量口径、lint 覆盖范围或 CI 工作流。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)
- [性能基线与性能测试体系](048-performance-testing.md)

## 范围与非目标

- CPU 与 GPU STL benchmark 在 Linux 上从 `/proc/self/status` 读取 `VmHWM`，校验字段单位后换算为字节。
- macOS 保留 `getrusage`，Windows 保留 `GetProcessMemoryInfo`；峰值 RSS 的含义不变。
- 不调整质量门禁、平台矩阵、benchmark 注册或测量口径。

## 前置条件与待决策

- CI 日志已将失败精确定位到 `tests/cpp/performance/project_stl_activation_cpu_benchmark.cpp` 与 `tests/cpp/performance/project_stl_viewport_gpu_benchmark.cpp` 中的 `rusage usage{}`。
- 修复后本地 include-cleaner 门禁可验证当前平台；Linux 远端复验需后续 push，本任务本身不 push。

## 实施步骤

1. Linux 分支改用内核文档定义的 `VmHWM` RSS 高水位字段，移除该分支不再需要的 `<sys/resource.h>`。
2. 运行格式检查、`cargo lint includes --check` 与适用的构建/测试入口。
3. 记录验证并按单 task commit 提交，不 push。

## 预计改动

- `tests/cpp/performance/project_stl_activation_cpu_benchmark.cpp`
- `tests/cpp/performance/project_stl_viewport_gpu_benchmark.cpp`
- 本任务文档与 `ai-docs/task-index.md`

## 清理与兼容例外

无废弃项；无兼容例外。

## 验收标准

- [x] 两个 benchmark 的 Linux 路径按 `VmHWM` 的 `kB` 单位返回字节，macOS 和 Windows 分支保持原样。
- [x] `cargo format --check` 与 `cargo lint includes --check` 在本机通过；两个 Release benchmark 目标构建通过。
- [x] GitHub Actions Linux includes job 在修复版本上通过。
- [x] task 与索引记录修复及验证结果；代码与 task 在同一 commit。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-29 | GitHub Actions run [36574286051](https://github.com/Yuki-Nagori/panta/actions/runs/36574286051) | 定位失败门禁与错误 | `cargo lint (includes, ubuntu)` 报两个 benchmark 的 `rusage` 缺少直接提供头；其余适用检查通过。 |
| 2026-09-29 | Linux 内核文档 [The /proc Filesystem](https://docs.kernel.org/filesystems/proc.html) | 核对 `VmHWM` 语义与单位 | 文档定义 `VmHWM` 为 peak resident set size；字段值以 kB 展示。 |
| 2026-09-29 | macOS arm64：`cargo format --check` | 格式入口通过 | 通过。 |
| 2026-09-29 | macOS arm64：`cargo lint includes --check` | include-cleaner 通过并覆盖两个 benchmark TU | 通过；自动生成 moc 后，相关 clang-tidy/includes 扫描成功。Linux 分支的最终远端复验待修复版本进入下一次 push。 |
| 2026-09-29 | macOS arm64 Release：`cmake --build target/native/release --target panta_bridge_project_stl_activation_cpu_benchmark panta_project_stl_viewport_gpu_benchmark --parallel 4` | 两个 benchmark 编译、链接通过 | 通过。 |

## 风险与回退

Linux `/proc` 不可用或字段格式异常时，基准将把峰值标为 unavailable；当前 resident RSS 也读取 `/proc/self/statm`，因此这与现有 Linux 指标的环境前提一致。macOS 与 Windows 的采样入口不变。

## 决策与工作记录

- 2026-09-29：根据 CI 日志登记修复。Linux `cargo lint (includes)` 将两个 benchmark 中的 `rusage` 判为无直接 provider，虽然源码条件分支列有 `<sys/resource.h>`。查阅 Linux 内核 proc 文档后，决定 Linux 直接读取语义一致的 `VmHWM` 字段；macOS 继续使用 POSIX `getrusage`，不增加 lint 豁免。

- 2026-10-02：通过 `gh run view 36890328640` 核验最新 [CI run 36890328640](https://github.com/Yuki-Nagori/panta/actions/runs/36890328640)，提交 `a0f0872061627a969d60e422a2c945e8fc22f7b7` 与当前 HEAD 一致，19 个 job 全部成功。Linux `cargo lint (includes, ubuntu)` 已通过；task 100 后续迁移性能文件不影响本次修复验收。

## 完成摘要

Linux RSS 采样修复已完成，本地格式、include-cleaner 和 Release benchmark 构建通过；最新 CI 的 Linux includes job 通过，剩余 CI 验收已闭环。证据：[CI run 36890328640](https://github.com/Yuki-Nagori/panta/actions/runs/36890328640)（`a0f0872`）。
