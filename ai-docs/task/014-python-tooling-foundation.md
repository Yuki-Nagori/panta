# 014 — 后续 Python 工具环境

- 状态：done
- 阶段：验证基础
- 依赖：[001](001-cargo-config.md)
- 优先级：P2
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-18

## 目标与背景

为质量门禁提供可复现的 Python 工具环境，当前只承载 `cmakelang` 的 CMake 格式检查。Python 工具通过 `uv` 和 `uv.lock` 管理，与桌面运行时、Python API 和 native ABI 保持边界。

## 必读

- [通用规范：comments](../standards/comments.md)
- [通用规范：repository-hygiene](../standards/repository-hygiene.md)

- [规范：python](../standards/python.md)
- [规范：ffi](../standards/ffi.md)
- [架构：study-and-automation](../architecture/study-and-automation.md)

## 范围与非目标

范围：`pyproject.toml`、`uv.lock`、Python 3.12+ 约束，以及由根 `cargo format`/`cargo lint cmake` 调用的 `cmake-format` 与 `cmake-lint`。

非目标：不嵌入解释器，不实现 pybind11/Rust binding，不为 MVP 提供 Python 入口，不把 Cppclean 引入质量门禁；Cppclean 与 IWYU/Cppcheck 职责重叠且其旧版包无法在 Python 3.14 构建。

## 前置条件与待决策

开始条件：质量工具链需要 CMake formatter，且 `uv` 可在开发机与 CI 获取。Python 最低版本、依赖版本和命令写入 `pyproject.toml`/`uv.lock`，不安装到系统 Python。

## 实施步骤

1. 确认 CMake 格式检查是首个实际 Python 工具用途，选择 uv 和 Python 3.12+。
2. 创建 `pyproject.toml`，固定 `cmakelang==0.6.13`，生成并提交 `uv.lock`。
3. 在 `tests/src/main.rs` 中通过 `uv run --locked cmake-format --check` 与 `uv run --locked cmake-lint` 检查所有 CMakeLists/`.cmake` 文件。
4. CI 安装 uv；本地和 CI 均从锁文件运行，不修改系统 Python。

## 预计改动

`pyproject.toml`、`uv.lock`、`tests/src/main.rs`、`.github/workflows/ci.yml`、README、质量工具链模块和 Python 规范。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [x] 新环境按记录重建依赖并运行 `UV_CACHE_DIR=target/panta-tools/uv/cache UV_PROJECT_ENVIRONMENT=target/panta-tools/uv/venv uv run --locked cmake-format --version` 与 `UV_CACHE_DIR=target/panta-tools/uv/cache UV_PROJECT_ENVIRONMENT=target/panta-tools/uv/venv uv run --locked cmake-lint --version`，Python 最低版本声明与实测版本一致。
- [x] 格式工具不启动 GUI 或修改工程；格式失败返回非零并保留文件路径上下文。
- [x] README 明确此任务只提供质量工具环境，不宣称 Python CAE API/headless 已实现。
- [x] 已同步 Python 规范、质量模块、当前可用命令和 task-index 状态。

- [x] 未引入旧实现或兼容代码；Cppclean 未纳入门禁，旧引用已清理。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-18 | `UV_CACHE_DIR=target/panta-tools/uv/cache UV_PROJECT_ENVIRONMENT=target/panta-tools/uv/venv uv lock`；`UV_CACHE_DIR=target/panta-tools/uv/cache UV_PROJECT_ENVIRONMENT=target/panta-tools/uv/venv uv run --locked cmake-format --version`；Python 3.14.0 | 依赖锁定成功，cmakelang 0.6.13 可执行；开发环境使用 uv 临时缓存以避免写入用户缓存目录 |
| 2026-09-18 | 根 `cargo format` 的 CMake 文件枚举与 `actionlint .github/workflows/ci.yml` | 已接入 `native/`、`qml/`、`tools/` 下 CMakeLists/`.cmake`；CI format job 安装 uv 并调用根入口 |

## 风险与回退

cmakelang 版本较旧，升级时必须重新核对 Python 支持矩阵和格式输出；uv lock 失败应阻止 CI，不回退到系统 pip。Python API、bindings 和业务脚本另立任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-18：质量门禁需要 CMake formatter，任务恢复；统一使用 uv，锁定 Python 3.12+ 与 cmakelang 0.6.13，根 `cargo format` 负责调度。
- 2026-09-18：评估 Cppclean 0.13；该包在 Python 3.14 的构建后端失败，且职责与 IWYU/Cppcheck 重叠，移除而不登记兼容例外。

## 完成摘要

已完成。`uv.lock` 固定 cmakelang 0.6.13，根 `cargo format` 检查 Rust、C++/CXX、CMake 和 QML；未引入 Python 运行时或业务 API。
