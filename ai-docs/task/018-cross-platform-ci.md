# 018 — 三平台 CI 基础

- 状态：done
- 阶段：验证基础
- 依赖：[001](001-cargo-config.md)（已完成）、[002](002-dependency-baseline.md)（已完成）
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

维护者要求在基础阶段即建立跨平台验证：以 GitHub Actions 在 macOS/Linux/Windows 三平台持续验证当前可用命令，避免 Rust 骨架只在本机可用的回归。聚合测试入口（011）与依赖缓存（012）仍按原计划后置。

## 必读

- [通用规范：validation-and-review](../standards/validation-and-review.md)
- [规范：baseline](../standards/baseline.md)
- [规范：依赖获取](../standards/dependency-acquisition.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：一个 GitHub Actions workflow，三平台矩阵执行 pinned 工具链安装与当前全部可用 Rust 检查。

非目标：native 依赖的托管构建接入 CI（随 004 之后扩展）；测试聚合与质量门禁（011）；依赖缓存（012）；额外测试框架。

## 前置条件与待决策

仓库托管于 GitHub（origin Yuki-Nagori/panta），使用 GitHub Actions。workflow 内不硬编码工具链版本，从 rust-toolchain.toml 读取 channel，避免与工具链清单漂移。

## 实施步骤

1. 新增 `.github/workflows/ci.yml`：push main 与全部 PR 触发；矩阵 macos-latest / ubuntu-latest / windows-latest。
2. 每平台执行：安装 pinned 工具链（minimal + rustfmt + clippy）→ `cargo build --locked` → `cargo test --locked` → `cargo fmt --all -- --check` → `cargo clippy --locked --all-targets`。
3. README 加 CI 徽章与环境要求；依赖获取文档记录 CI 环境与边界；task-index 同步验证节奏描述。

## 预计改动

`.github/workflows/ci.yml`（新增）、README.md、ai-docs/standards/dependency-acquisition.md、ai-docs/task-index.md、本文件。

## 清理与兼容例外

无废弃项。012 的"接入 CI"范围收窄为聚合与缓存扩展，边界记录于本文与依赖获取文档。

## 验收标准

- [x] workflow 三平台矩阵覆盖 pinned 工具链与当前全部可用命令，且不重复定义工具链版本。
- [x] 不引入本机路径或未登记的第三方依赖；action 版本为当前最新（checkout v7）。
- [x] README 环境要求与 runner 实际前置一致；文档/索引同步，011/012 边界清晰。
- [ ] Actions 实际三平台绿灯（待下次 push 后观察确认；不通过则修复并回填证据）。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | workflow YAML 解析与关键字段检查（python3 yaml.safe_load + 字段断言） | 语法有效；矩阵含三平台；四条检查命令与本地验收一致 |
| 2026-09-16 | actions/checkout 最新版本核实（GitHub API） | v7.0.1（2026-07-20），workflow 引用 @v7 |
| 2026-09-16 | 本地等价命令复跑（rustup run 1.98.1 build/test/fmt/clippy） | 全部通过（与 001 证据同源） |
| 待 push | GitHub Actions 首次三平台运行 | 未执行：配置已提交，运行观察待 push 后回填 |

## 风险与回退

runner 的 rustup 若不支持按 rust-toolchain.toml 自动解析，安装步骤已显式读取 channel，不依赖隐式行为。Windows 的 CRLF 检出可能影响 fmt 检查，rustfmt 默认 newline_style=Auto 可接受；若失败再在 workflow 内显式 `git config core.autocrlf false`。回退仅删除 workflow 与文档引用，不影响本地流程。

## 决策与工作记录

- 2026-09-16：维护者要求基础阶段配三平台 CI；本任务从 012 的前置范围中拆出并立即实施。
- 2026-09-16（实施）工具链安装从 rust-toolchain.toml 读取 channel（awk 提取，bash 兼容三平台），不在 workflow 中重复版本号。
- 2026-09-16（实施）不加 `--deny warnings` 等门禁参数：质量门禁由 011 统一定义，当前 CI 只做可用性验证。

## 完成摘要

已建立 `.github/workflows/ci.yml`：macOS/Linux/Windows 矩阵按 rust-toolchain.toml 安装固定工具链并执行 build/test/fmt/clippy 四项检查；README 增加徽章与三平台环境要求；依赖获取文档记录 CI 环境与 011/012 边界。限制：Actions 实际运行绿灯待 push 后确认（验收第 4 项未勾），确认前不宣称三平台验证通过。
