# 105 — CI 静态检查与模块文档修正

- 状态：in-progress
- 阶段：验证基础
- 依赖：[014](014-python-tooling-foundation.md)、[021](021-important-module-planning.md)、[042](042-unified-llvm-toolchain.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-07 / 2026-10-07

## 目标与范围

修复提交 `28948f6` 的 [CI 37023906309](https://github.com/Yuki-Nagori/panta/actions/runs/37023906309) 中 WebGPU adapter 预检的 clang-tidy 与 include-cleaner 失败，处理 [issue #5](https://github.com/Yuki-Nagori/panta/issues/5) 的 Python/uv 文档版本滞后和 [issue #6](https://github.com/Yuki-Nagori/panta/issues/6) 的模块入口错误规划描述。

回调仅检查 adapter，不保留所有权，改用常量引用；超时时间使用直接声明的标准整数类型。文档以实现固定版本及 task 状态为准，历史实测保留当时版本。同步依赖供给规范中的同类旧版本描述，清理模块入口中已落地能力的规划用语。保持 UI、adapter 预检行为及依赖版本不变，不恢复 deferred 的 007/038，也不增加 lint 抑制或兼容层。

## 必读

- [注释规范](../standards/comments.md)
- [文档规范](../standards/documentation.md)
- [文件规范](../standards/repository-hygiene.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)

## 验收标准

- [x] adapter 预检使用 const 引用，整数类型直接包含声明头；clang-tidy / include-cleaner 通过。
- [x] 当前 Python/uv 文档与 `panta-build/src/python.rs` 的 0.12.18 / 3.14.7 一致；历史验证明确标为历史。
- [x] 模块入口区分已实现基础与后续规划，路径层和 QML 模块注册不再被统称为规划。
- [x] Cargo 聚合、构建、格式及完整 lint 通过；本地文档链接及差异检查通过。
- [ ] 新提交的 Linux CI clang-tidy / include-cleaner 通过。

## 验证计划与结果

在 macOS 仓库根目录沿用锁定工具链执行 `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check`。文档检查本地链接和固定版本。`cargo test --locked --workspace` 通过 Rust 工作区及 72/72 个 native / QML CTest；`cargo build --locked`、`cargo format --check` 通过。修改文档的 189 个本地链接、代码围栏和当前固定版本检查通过。`cargo lint --check` 全部通过，包括原 CI 失败的 clang-tidy 与 include-cleaner；本批未重新执行真实 GPU 窗口，Linux 修复效果待新 CI，不将本地结果标为三平台通过。

## 工作记录

- 2026-10-07：通过 gh 核实最新 CI 的失败栈与两个 open issues，登记本任务及索引后实施。替代旧文档描述，无废弃运行时实现，无兼容例外；issues 的修复状态以本地验证及后续提交为准。

## 完成摘要

本地修复及验证完成：adapter 回调使用常量引用，补齐标准整数声明头；issue #5/#6 对应文档与实现边界已同步。未改变依赖版本、UI 或预检行为，无新增抑制或兼容层。状态保持 in-progress，剩余新提交的 Linux CI 验收；GitHub issues 仍开放，待修复提交发布后关闭。
