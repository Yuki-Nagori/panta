# 104 — Windows 工程选择器路径回归修复

- 状态：in-progress
- 阶段：验证基础
- 依赖：103
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-02 / 2026-10-02

## 目标与范围

CI run [36939813938](https://github.com/Yuki-Nagori/panta/actions/runs/36939813938) 的 Windows Cargo 聚合在工程选择器 plain / encoded 两组回归失败：Qt file URL 解码后的路径使用 `/`，新建工程的 Rust 快照含 Windows `\`，直接字符串比较误报不同。修正测试的预期路径表达，不修改生产路径服务、QML 选择器配置或工程格式。

## 必读

[验证与评审](../standards/validation-and-review.md)、[注释规范](../standards/comments.md)、[文件规范](../standards/repository-hygiene.md)、[提交规范](../standards/commits.md)。

## 验收标准

- [x] 接线断言采用所选 file URL 解码后的本地路径，保留 plain / 中文、空格、#、% 的准确路径与工程状态断言。
- [x] Cargo 聚合、格式、lint 与 diff 检查通过，记录 Windows 验证限制。
- [x] 任务与索引同步；无生产接口或兼容分支改动。
- [ ] 修复提交后的 Windows Cargo 聚合 CI 通过。

## 验证与工作记录

- 2026-10-02：读取最新失败日志，Windows 仅 `project_file_dialog_routes_final_selection` 两组在第 652 行路径字符串断言失败；材料异步保护回归通过。完整日志保存于 `/tmp/panta-ci-36939813938.log`。开始修正接线测试的路径预期。

- 2026-10-02：将预期路径取自实际所选 `QUrl::toLocalFile()`；两处路径仍逐字比较，不折叠大小写、不移除中文或特殊字符。生产 `openProject()` 已通过 `QDir::fromNativeSeparators` / `cleanPath` 规范化输入，测试预期现与这条接线一致。取消及坏文件保护、工程名称、dirty、页签和 Ribbon 断言保持。
- 2026-10-02：macOS arm64 仓库根目录 `cargo test --locked --workspace` 通过，包含 69/69 CTest；`cargo format --check`、`cargo lint --check`、`git diff HEAD --check` 通过。日志 `/tmp/panta-104-tests.log`、`/tmp/panta-104-lint.log`。失败运行的其他 18 个 CI 作业均成功；本机没有 Windows 执行环境，修复后的 Windows 结果待远端 CI，任务保持 in-progress。

## 清理与兼容例外

无废弃生产实现，无兼容例外。

## 本批进展

测试路径预期已修正，本地聚合与质量检查通过；等待修复提交后的 Windows CI，尚未标 done。
