# 020 — 托管引导：CMake/Ninja 二进制供给

- 状态：ready
- 阶段：M0
- 依赖：[004](004-cargo-native-orchestration.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-17

## 目标与背景

004 已接通 Cargo→CMake 调度，但仍要求本机预装 CMake/Ninja；[依赖获取](../standards/dependency-acquisition.md) 的托管原则要求"开发者只装 rustup 与平台编译器"。本任务实现工具二进制的自动供给：首次构建时按固定清单下载 CMake 与 Ninja 的预编译资产到构建树托管目录，缺失时引导，使 README 环境要求降为最终形态。依赖满足、任务已 ready，尚未实施。

## 必读

- [规范：依赖获取](../standards/dependency-acquisition.md)
- [规范：cargo](../standards/cargo.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：build.rs 引导逻辑（下载/校验/缓存 CMake 与 Ninja）、SHA256 校验与回填、跨平台资产（macOS/Linux/Windows）、缺失时的可定位诊断。

非目标：Qt/VTK/OCCT/Netgen 的获取与构建（各自任务）；CI 缓存共享（012）。

## 前置条件与待决策

- 固定版本已在依赖获取清单（CMake 4.4.3、Ninja 1.13.2）；实施时补齐各平台资产 URL 与 SHA256。
- Ninja 采用各平台预编译 release 资产；若上游没有可复核资产，另立制品任务，不在开发者构建中编译源码。缓存目录布局建议为 `target/` 下托管目录，随 profile 共享。

## 实施步骤

1. 在 build.rs 中实现"定位 → 缺失时按清单获取 → 校验 → 注入 PATH/显式路径"的供给链。
2. 记录 macOS/Linux/Windows 三平台资产与校验，回填依赖获取文档。
3. 验证：无 CMake/Ninja 的 PATH 下 `cargo build` 全链成功；损坏下载可检出并被拒绝。

## 预计改动

`crates/launcher/build.rs`（或新调度模块）、依赖获取文档、README 环境要求。

## 清理与兼容例外

移除"要求预装 CMake/Ninja"的临时说明；无兼容层。

## 验收标准

- [ ] 干净环境（PATH 无 cmake/ninja）`cargo build --locked` 全链成功且产物可运行。
- [ ] 下载校验（SHA256）生效，损坏产物被拒绝并给出可定位诊断。
- [ ] 三平台资产清单与校验和回填文档；README 环境要求同步收敛。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| — | 尚未执行 | 无实现证据 |

## 风险与回退

下载源不可达或校验失败会阻塞首次构建；保留"CMAKE 环境变量指定本机 cmake"的旁路并写明诊断。回退仅撤销本任务变更，恢复"要求本机预装"的 004 状态。

## 决策与工作记录

- 2026-09-16：自任务 004 的托管原则拆分立task；004 交付调度与诊断，本任务交付二进制供给。
- 2026-09-17：依赖 004 已完成且范围/验收明确，状态调整为 ready。根据预编译优先规则，CMake/Ninja 只消费带 SHA256 的官方或可信预编译资产；缺少资产时另立项目制品任务，不回退本地源码构建。

## 完成摘要

未完成。完成时填写实现行为、验证证据、剩余限制和后续 task；全部验收有证据后才标 done。
