# 111 — 构建供给与质量命令模块整理

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[107](107-rust-service-boundaries-and-revisions.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-10-08 / 2026-10-09

## 目标与范围

分离 panta-build 的工具资产登记、安装 / 校验、编译器 / 平台环境；沿现有 coverage / performance 模块组织质量入口的 build / lint / sanitizer。保持 Cargo / CMake 职责、固定版本与供给锁。沿用现行测试布局：私有单元测试保留在源码内联 cfg(test) 模块，公共 API 黑盒测试在 tests/rust 注册；107 的 FFI 测试归属问题已由 112 修正，本任务不重复迁移。

本任务对应 [106 架构评审](106-rust-architecture-review.md) 的后续建议；确认导入后台化仍由 063 实施。保持 UI 与正常工程 schema，不将新基础接口描述为已经实现。

## 必读

- [分层规则](../standards/layering.md)
- [Rust 规范](../standards/rust.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [文件规范](../standards/repository-hygiene.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] 锁 / 摘要 / 缓存失效与平台解析回归通过，Cargo 原入口和失败码不变；测试分类及文档与实际布局一致。
- [x] 替换实现与失效引用已删除，无新增兼容分支。
- [x] Cargo 工作区聚合、构建、格式及完整 lint 通过；记录具体平台与未覆盖点。

## 验证计划与结果

先补定向回归定位，再执行 `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check`。涉及性能容量或真实窗口时按实际范围单独验证。先记录现有安装回归与质量 CLI 的错误输出 / 退出码基线；模块拆分后执行行为与公共入口复验。

## 本次验证结果

2026-10-09，仓库根目录，macOS arm64 / Rust 1.98.1：

| 检查 | 实际结果 |
|---|---|
| 固定资产源清单对比 | 12 个官方 URL、12 个 SHA256 与 CMake / Ninja / LLVM / uv 四个版本和拆分前一致；没有重新声明上游资产已经下载验证 |
| 质量 CLI 基线对比 | 7 组缺失 / 非法命令、format / lint / coverage / performance 参数的退出码与 stdout / stderr 字节一致 |
| 供给与 runner 定向回归 | panta-build 20 个单元测试通过（原 17 项保留，新增 marker 失效、PATH 合并和 OUT_DIR 解析）；runner 9 个常规单测通过，3 个性能 CLI 检查按原规则保持 ignored 并另外定向执行通过 |
| `cargo build --locked` | 通过；launcher / FFI 共用原公共供给 API |
| `cargo test --locked --workspace` | 聚合通过，包含新私有回归与 native / QML 72/72 |
| `cargo format --check` / `cargo lint --check` | 格式与完整 8 阶段 lint 通过 |
| `cargo run --locked -p panta-tests -- toolchain` | 实际 LLVM / CMake / Ninja / Qt / GoogleTest 路径、版本和编译数据库核验通过 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --locked -p panta-build --no-deps` | 通过 |
| 独立 subagent review | 公开 API、资产、平台路径、锁 / marker / 失败收尾、子进程参数及测试 / 文档检查通过，无可行动问题 |

本轮没有执行冷下载、Linux / Windows CI、完整 coverage / sanitizer / Miri 套件、CPU / GPU 性能基准或真实窗口验收。sanitizer 的平台矩阵和构建参数有纯函数回归，不能代替实际插桩运行；Windows SDK / CRT 的实际进程环境仍需 Windows 验证。本次为模块整理，不修改这些行为与现有排除边界。

## 工作记录

- 2026-10-09：完成供给库公共门面及六个私有模块；uv 资产统一登记，python 的公开版本 / 命令路径保留。质量入口分离命令运行、目录、构建、格式、lint、工具链、sanitizer、Miri，native 覆盖率归入 coverage；既有性能入口直接引用对应模块。私有测试随源码内联迁移，子进程精确过滤名称同步；新增缓存失效、平台路径、Cargo 透传及进程失败回归。聚合、构建、格式、完整 lint、工具链核验、文档检查和独立 review 通过，task / 索引同步 done；无新增兼容分支。

- 2026-10-09：开始实施。panta-build/lib.rs 1213 行，质量入口 main.rs 1094 行。按现有真实消费者拆分资产清单、安装 / 校验、编译器定位和平台环境；质量入口只保留分发，构建、命令执行、lint / 格式、工具链核验、sanitizer / Miri 与 native coverage 各归模块。既有内联私有测试随职责移动，保持固定版本、摘要、锁、目录、命令参数与失败码；不修改 UI 或依赖版本。

- 2026-10-08：按用户要求一次登记剩余 Rust 架构任务，先提交规划后实施；避免把后续建议混入已完成的 107。

## 完成摘要

已完成构建支持与质量命令的职责拆分。公共 API、固定资产、锁与发布语义、Cargo 原入口 / 失败码保持；源码内联私有回归、命令基线、资产源清单与实际工具链核验通过。测试布局及模块文档已同步。
