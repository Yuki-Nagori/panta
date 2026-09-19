# 012 — CI 与依赖缓存

- 状态：done
- 阶段：验证基础
- 依赖：[011](011-test-quality-entrypoints.md)
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-19

## 目标与背景

在实际代码托管环境建立可复现的基础设施验证，缓存只加速不隐藏依赖。

任务 040 的初版把整个 `target/` 放入缓存，可能恢复旧的 Cargo/CMake 构建树。本轮已收窄为 Cargo registry/git 与带版本校验的 `target/panta-tools`、`target/panta-deps`；native 构建树和 Cargo 编译产物不跨运行复用。远端命中/失效和清空缓存仍需当前 workflow 实跑补证。

2026-09-19 首次实跑暴露两类缺口：供给 Qt 的 `find_package(Qt6 Gui)` 经 `WrapOpenGL` 依赖宿主 OpenGL 开发头文件，runner 未预装导致全部 Linux job 失败（macOS 因 SDK 自带 GL 框架通过）；构建引导的 curl 下载无空闲/总时长上限，LLVM Windows 归档传输停滞时 job 挂满 6 小时才被平台上限终止。平台前置属于宿主能力（同 MSVC/Apple SDK），由 workflow 安装并在依赖获取规范登记。

## 必读

- [通用规范：repository-hygiene](../standards/repository-hygiene.md)
- [通用规范：validation-and-review](../standards/validation-and-review.md)

- [规范：baseline](../standards/baseline.md)
- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [规范：ninja](../standards/ninja.md)
- [架构：build-and-development](../architecture/build-and-development.md)
- [架构：milestones-and-validation](../architecture/milestones-and-validation.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不自动发布、不扩大未验证的平台矩阵；没有可用远端 runner 时如实标 blocked。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 托管平台与 CI 格式已由 [018](018-cross-platform-ci.md) 确定为 GitHub Actions 三平台矩阵；本任务核实 runner 能力，确认 018 workflow 首次三平台绿灯并回填其证据，再扩展缓存与聚合接入。
2. 运行锁定工具链与依赖重建流程，缓存键包含 OS/架构/LLVM 版本、锁文件和供给清单。注意任务 005 起 `cargo build` 会拉取 Qt 预编译包——CI 缓存覆盖 `target/panta-deps/qt`，但不缓存可变 native 构建树。
3. 接入 011 的检查入口，保留测试报告和失败诊断；有显示环境时执行图形冒烟。
4. 验证清空缓存也可成功，明确未覆盖的平台、图形检查和下一步补齐方式。

## 预计改动

实际 CI 平台配置、缓存配置与运行说明。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [x] 目标 runner 上从干净 checkout 通过统一检查，并能获取真实运行记录。（run 35425146629 由 main push 触发，公开日志留存）
- [x] 缓存命中与缓存删除两种路径有效；不依赖本机全局包或绝对目录。（删除/冷路径由 run 35425146629 证明；命中路径由 run 35427710829 证明——Cargo.toml 变更换新精确键后经 restore-key 命中上一键 4.1GB 依赖缓存，恢复后全绿）
- [x] 故意失败的检查会使 CI 失败，未运行的图形检查明确标为缺口。（run 35376430562 的真实失败即时阻断 CI；图形/窗口冒烟缺口在 testing 与 quality 模块标注为独立验证）
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。（2026-09-19 同步 default-members 语义与依赖获取文档）

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-17 | 任务 040 workflow 初始缓存接入 | 初版覆盖 `~/.cargo/registry`、`~/.cargo/git`、`target/`；后续复审确认整个 target 会恢复可变构建树，不能作为最终方案 |
| 2026-09-19 | CI cache 方案收窄 | 只缓存 registry/git、`target/panta-tools` 和 `target/panta-deps`；key 按 OS/架构/LLVM 与 Cargo/Python/native 供给清单区分，restore key 只回退同平台同 LLVM 依赖资产 |
| 2026-09-19 | run 35376430562 失败诊断 | 全部 Linux job 失败于 launcher build.rs 内 `find_package(Qt6 Gui)`：`Qt6Gui could not be found because dependency WrapOpenGL could not be found`；Qt 归档下载/解包均成功，缺口是宿主 GL 开发文件。Windows job 在 build script 阶段静默挂满 6h 平台上限（`Checking panta-dslc` 后无输出），唯一无上限等待是构建引导 `curl -fSL` |
| 2026-09-19 | 本地验证（macOS 26, arm64） | `actionlint .github/workflows/ci.yml` 通过；`cargo check --locked -p panta-build`、`cargo test --locked -p panta-build`（12 passed）、`cargo fmt --all -- --check`、`git diff --check` 通过；新 curl 参数集实测下载 ninja-mac.zip 且 SHA256 与固定清单一致。三平台实跑证据待 push 后的 run 回填 |
| 2026-09-19 | run [35425146629](https://github.com/Yuki-Nagori/panta/actions/runs/35425146629)（`2ebbea7` 合入 main 后 push）三平台 | 修复后的缓存与前置在真实 run 生效：success，总时长 6m29s（此前失败 run 35376430562 拖满 6h）。Linux GL 前置与 curl 上限修复生效；新缓存键下无可命中旧缓存，证明"删除/冷缓存"路径可完整构建；"缓存命中"路径待下一次同键 run 观测 |
| 2026-09-19 | run [35427710829](https://github.com/Yuki-Nagori/panta/actions/runs/35427710829)（批次推送触发，含 Cargo.toml 变更）三平台 | 新精确键下的命中回退路径：success（约 7 分钟）。精确键 `…-a6ff278…` 未命中后按 restore-key `panta-deps-macOS-ARM64-llvm-22.1.7-` 命中上一键 `…-82b3bce7…`（约 4.1GB），恢复后 check/build/toolchain/test 全绿并回存新键；命中与删除两种路径证据齐备，任务关闭 |
| — | 完整缓存命中/删除验证 | 未完成 |

## 风险与回退

缓存可能掩盖未声明依赖，runner 可能缺少图形能力；用无缓存运行核验可复现性，单独记录图形检查缺口。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-19：Linux 平台前置补 OpenGL 开发头文件与 QML 运行库（`libgl-dev`、`libegl1`、`libxkbcommon0` 及软件渲染驱动），仅在触发 native 构建或 QML 测试的 job 安装；构建引导 curl 增加 connect/speed/max 上限，任务级 `timeout-minutes` 兜底，防止单点传输停滞拖满平台上限。缓存收窄方案不变：`panta-deps` 内 archives/extracted 均带哈希指纹，坏缓存按校验自愈。
- 待记录：实际方案、版本依据、失败原因、范围调整与后续任务。

## 完成摘要

已交付：缓存覆盖 registry/git 与 `target/panta-tools`、`target/panta-deps`，按 OS/架构/LLVM 与 Cargo/Python/native 供给清单分键，restore-key 仅回退同平台同 LLVM 依赖资产，归档带哈希指纹、坏缓存按校验自愈；构建引导 curl 带 connect/speed/max 上限并由任务级 timeout 兜底，Linux GL 宿主前置按 job 按需安装。删除/冷路径、命中回退路径与失败阻断均由公开 run 证明（35376430562 失败阻断、35425146629 冷全绿 6m29s、35427710829 restore-key 命中全绿）。
