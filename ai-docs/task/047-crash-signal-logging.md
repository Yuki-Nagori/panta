# 047 — 崩溃信号处理与日志落地

- 状态：in-progress
- 阶段：验证基础
- 依赖：[008](008-tasks-errors-logging.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-19 / 2026-09-19

## 目标与背景

原生崩溃（SIGSEGV/SIGTRAP 等）在控制台零输出，证据只进 macOS
`~/Library/Logs/DiagnosticReports/*.ips`（007 取证实证，维护者指出诊断盲区）。
本任务在 foundation 层落地崩溃信号处理器：崩溃时同步向 stderr 与日志文件
写入信号信息与 best-effort 回溯，随后恢复默认 disposition 重新 raise——
保持 .ips 崩溃报告与内核退出语义不变。

## 必读

- [规范：cpp](../standards/cpp.md)、[008 后台任务、错误与日志基础](008-tasks-errors-logging.md)

## 范围与非目标

范围：Rust `panta-foundation` 崩溃处理器（POSIX 全量：信号名/pid/回溯落盘；
Windows 最小信号写出）、经 `panta-ffi` 暴露、app 入口安装、fork 自验测试。
`panta-core` 保持领域模型职责，不承载该进程级平台设施。非目标：完整符号化（.ips 对照）、
minidump/WER、Qt 消息处理、远程上报。

## 实施步骤

1. `panta::foundation::install_crash_handler(log_file)`：安装时预创建并持有
   日志 fd（处理器内不做路径/分配操作），登记 SIGSEGV/SIGBUS/SIGFPE/SIGILL/
   SIGABRT/SIGTRAP。
2. 处理器仅用 async-signal-safe 操作（write/整数格式化）；回溯
   `backtrace_symbols_fd` 为 best-effort（崩溃点在分配器内时可能缺失，
   头文件注明取舍）。
3. app main 入口首行安装；fork 自验测试：子进程 raise(SIGSEGV)，父进程
   断言子进程死于 SIGSEGV 且日志含信号与 pid。

## 验收标准

- [ ] 崩溃信号触发时 stderr 与日志文件均有信号名/pid 记录，进程仍以原
      信号终止（.ips 照常生成）。
- [ ] fork 自验 CTest 三平台通过。
- [ ] 文档、task、索引同步；无未登记兼容代码。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | macOS arm64；`cargo test --locked -p panta-foundation --all-targets` | 通过，1/1；fork 子进程触发 `SIGSEGV` 后由处理器写入信号/pid/回溯，恢复默认处置后仍以 `SIGSEGV` 终止；父进程读取日志并断言内容后清理临时目录 |
| 2026-09-19 | macOS arm64；`cargo test --locked -p panta-ffi --all-targets` | 通过，11/11；CXX 边界、panic-abort、任务/路径服务回归通过，崩溃安装入口完成 foundation 转发 |
| 2026-09-19 | macOS arm64；`cargo build --locked` | 通过；panta-foundation → panta-ffi staticlib → Cargo 调度 native/VTK 构建链成功 |

## 决策与工作记录

- 2026-09-19：007 取证时发现原生崩溃控制台零输出（证据仅 .ips），维护者
  指示优先落地。backtrace* 非严格 async-signal-safe 的取舍写入头文件；
  处理后恢复默认 disposition 重发，不以处理器替代 .ips。
- 2026-09-19：崩溃设施从 `panta-ffi/src/crash.rs` 收敛到独立的
  `panta-foundation` crate。`panta-ffi` 只保留 CXX/错误转换入口，
  `panta-core` 继续保持领域模型与无手写 unsafe；手写 unsafe 集中在
  `panta-foundation::crash` 专用模块，并逐块保留 `// SAFETY:` 前提。

## 完成摘要

未完成。
