# 047 — 崩溃信号处理与日志落地

- 状态：in-progress
- 阶段：验证基础
- 依赖：[008](008-tasks-errors-logging.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-19 / 2026-10-02

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
Windows 最小 SEH 记录写出）、经 `panta-ffi` 暴露、app 入口安装，以及经 Cargo 运行的平台专属测试（POSIX 子进程信号测试、Windows 异常过滤器验证）。
`panta-core` 保持领域模型职责，不承载该进程级平台设施。非目标：完整符号化（.ips 对照）、
minidump/WER 报告采集、Qt 消息处理、远程上报。

## 实施步骤

1. `panta::foundation::install_crash_handler(log_file)`：安装时预创建并持有
   日志 fd（处理器内不做路径/分配操作），登记 SIGSEGV/SIGBUS/SIGFPE/SIGILL/
   SIGABRT/SIGTRAP。
2. 处理器仅用 async-signal-safe 操作（write/整数格式化）；回溯
   `backtrace_symbols_fd` 为 best-effort（崩溃点在分配器内时可能缺失，
   头文件注明取舍）。
3. app main 入口首行安装；macOS/Linux 在 Rust 测试中独立启动子进程触发
   SIGSEGV，父进程断言原信号终止及日志内容；Windows 使用独立子进程验证
   异常记录写入后继续默认 WER 处置，不能在测试进程内直接触发未处理异常。

## 验收标准

- [ ] macOS/Linux 子进程触发 POSIX 崩溃信号时，stderr 与日志文件均有信号名/pid，子进程仍以原信号终止；macOS `.ips` 语义保留。
- [ ] Windows 子进程触发 SEH 后，stderr 与日志文件记录异常码/pid，并继续 Windows 默认 WER 处理；不在父测试进程内触发异常。
- [ ] macOS/Linux POSIX 行为测试与 Windows SEH 子进程测试经 Cargo 聚合入口通过，覆盖率与质量检查通过；测试注册方式、文档、task 与索引一致，无未登记兼容代码。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-19 | macOS arm64；`cargo test --locked -p panta-foundation --all-targets` | 通过，1/1；fork 子进程触发 `SIGSEGV` 后由处理器写入信号/pid/回溯和真实日志路径，恢复默认处置后仍以 `SIGSEGV` 终止；测试同时断言 `crash_log_path()` 返回实际文件路径，父进程读取日志后清理临时目录 |
| 2026-09-19 | macOS arm64；`cargo test --locked -p panta-ffi --all-targets` | 通过，11/11；CXX 边界、panic-abort、任务/路径服务回归通过，崩溃安装入口完成 foundation 转发 |
| 2026-09-19 | macOS arm64；`cargo build --locked` | 通过；panta-foundation → panta-ffi staticlib → Cargo 调度 native/VTK 构建链成功 |
| 2026-09-20 | GitHub Actions CI run [35459273420](https://github.com/Yuki-Nagori/panta/actions/runs/35459273420)（commit `a613a31`）Windows job `105940092071`、Rust coverage job `105940092069` | Windows 暴露实现只使用 Unix `std::os::fd`、POSIX 信号常量和 `libc::write`；Rust 覆盖率为 88.55%，新增 crash 模块仅 42.79% 行覆盖，低于现行 92% 门禁。保留 POSIX 完整信号路径，补 Windows 最小 SEH 日志路径，并为纯格式化/写出辅助函数增加同进程测试覆盖 |
| 2026-09-20 | macOS arm64；`cargo test --locked -p panta-foundation --all-targets`、`cargo check --locked -p panta-foundation --target x86_64-pc-windows-msvc`、`cargo coverage` | 通过：macOS crash 测试 2/2；Windows 目标交叉检查通过；覆盖率函数 89.18%、行 93.84%。Windows 使用 `SetUnhandledExceptionFilter` 写出最小 SEH 进程记录后继续 WER，POSIX 路径保留信号重发与 `.ips` 语义 |
| 2026-09-24 | GitHub Actions run [36001859191](https://github.com/Yuki-Nagori/panta/actions/runs/36001859191)，commit `48ea4b4`，macOS/Linux/Windows Cargo test | macOS 与 Linux 的 `handler_logs_segfault_and_reraises` 通过；Windows `handler_installs_windows_filter` 通过。Windows 测试只验证 filter 安装和日志路径创建，尚未在子进程触发 SEH 并验证异常记录/WER；POSIX stderr 仍缺捕获断言，因此保留 in-progress。 |

## 决策与工作记录

- 2026-09-19：007 取证时发现原生崩溃控制台零输出（证据仅 .ips），维护者
  指示优先落地。backtrace* 非严格 async-signal-safe 的取舍写入头文件；
  处理后恢复默认 disposition 重发，不以处理器替代 .ips。
- 2026-09-19：崩溃设施从 `panta-ffi/src/crash.rs` 收敛到独立的
  `panta-foundation` crate。`panta-ffi` 只保留 CXX/错误转换入口，
  `panta-core` 继续保持领域模型与无手写 unsafe；手写 unsafe 集中在
  `panta-foundation::crash` 专用模块，并逐块保留 `// SAFETY:` 前提。

## 2026-10-02 实现与验收补齐

当前进展：实现和自动化验收测试已补齐，本地检查通过，仅待本批提交的三平台 CI 验收。任务保持 in-progress；CI 必须对应包含本批改动的提交，不能沿用历史安装 smoke 的结果。

### 实现与评审

- POSIX stderr 补齐 PID；日志与 stderr 先写基本头部，再尝试 best-effort 回溯，避免回溯阻塞时连 stderr 的信号 / PID 都缺失。
- 用独立启动的测试子进程替换多线程 harness 中的 fork；父进程捕获 stderr，校验精确信号、PID、日志路径和原信号退出。移除为覆盖率在父进程调用处理器的做法，清理失败不再被忽略。
- Windows 读取系统提供的异常记录，记录固定八位十六进制异常码与 PID。生产过滤器仍返回 `EXCEPTION_CONTINUE_SEARCH`；测试子进程仅设置 `WER_FAULT_REPORTING_NO_UI`，触发非连续 SEH 异常，父进程校验 stderr、日志与异常退出码。替换原本只检查安装的 smoke。
- 崩溃子入口为 ignored 测试，由父测试显式启动；缺少专用环境变量时不会触发崩溃。Miri 不运行系统 FFI / 进程测试。整理注释，保留 ABI、句柄生命周期与回溯安全取舍。

参考：[EXCEPTION_POINTERS](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-exception_pointers)、[EXCEPTION_RECORD](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-exception_record)、[WerSetFlags](https://learn.microsoft.com/en-us/windows/win32/api/werapi/nf-werapi-wersetflags)。测试不使用会禁用 WER 的 `SEM_NOGPFAULTERRORBOX`。

### 本地验证

- macOS arm64：`cargo test --locked -p panta-foundation --all-targets` 通过，2 passed / 1 ignored。ignored 子入口已由父测试运行并验证。
- `cargo check --locked -p panta-foundation --tests --target x86_64-pc-windows-msvc` 通过；包含新增 Windows 测试的类型检查，尚未运行或链接 Windows 可执行文件。
- `cargo test --locked --workspace` 通过，包含 qmllint 与 CTest 69/69。
- `cargo coverage` 通过现行门禁：全局函数 624/687（90.83%）、行 6849/7359（93.07%）。foundation 为函数 6/13、行 83/171；信号退出前不能刷新部分子进程 profile，剩余缺口如实保留，不新增排除或在父进程触发处理器凑数。
- macOS DiagnosticReports 新增 `panta_foundation-9bfc76d2827a102c-2026-10-02-054925.ips`，记录 SIGSEGV / EXC_CRASH，系统诊断报告仍能生成；该系统文件不纳入仓库。
- 评审调整后再次运行 `cargo test --locked --workspace`、`cargo coverage`、`cargo format --check`、`cargo lint --check` 及上述 Windows 目标交叉检查，全部通过。三平台 CI 尚未取得本批提交的运行证据，待确认后同步验收项与任务 / 索引状态。
