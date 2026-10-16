# 123 — Rust CLI 检查 STL

- 状态：done
- 阶段：CAE 领域模块迁移
- 依赖：[121 GUI 与 CLI 共享 Rust 命令入口](121-rust-gui-cli-command-layer.md)、[063 STL 导入与工程工作区](063-stl-import-and-mesh-workspace.md)、[067 Rust 统一 STL 解析与 Mesh IR 领域校验](067-rust-mesh-domain-migration.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-17

## 目标与背景

提供不启动 GUI 的 STL 预检入口，便于脚本在导入前读取解析诊断、面片数量和包围盒等已有信息。检查只读，不创建或修改工程资产。

## 必读

- [应用命令层](../architecture/command-layer.md)
- [分层标准](../standards/layering.md)
- [测试规范](../standards/testing.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

- CLI 使用现有 Rust STL 解析与验证能力，输出稳定、可诊断的预检摘要。
- 明确格式错误、空网格、无效几何等失败结果及退出码；路径支持空格和非 UTF-8 平台路径。
- 验证输出与 GUI 导入预检使用同一解析/诊断规则。

非目标：修复或改写 STL、网格质量优化、导入工程、调用 OCCT/Netgen、引入独立解析器。

## 前置条件与待决策

- 依赖 063、067 的当前解析与诊断接口；实施时核对面片数、包围盒等字段是否均可由公共 Rust API 提供。
- 选择人类可读摘要与机器可读输出的最小范围；若需要 JSON，复用项目已有序列化约定，不增加临时私有格式。

## 决策

- CLI 使用 `stl inspect <path>`，通过 `ProjectService::inspect_stl` 复用 GUI 的 STL 解析和预检规则；第一版输出人类可读的来源名、面片数和三个方向的尺寸，不新增 JSON 格式或 STL 算法。
- 命令路径始终以平台原生 `Path` 读取；来源文件名进入 UTF-8 元数据时使用有损替换显示，不改变文件访问路径或字节内容。
- STL 不包含单位；尺寸按来源坐标原样输出，不推断单位，也不把解析成功描述为网格质量合格。

## 验收标准

- [x] CLI 只读解析 STL，并复用 GUI 所用 Rust 解析和验证路径。
- [x] 有效文件输出可核对的摘要；损坏、空或不支持的输入返回非零退出码和诊断。
- [x] 单元与真实进程测试覆盖路径、解析成功和失败；任务记录适用验证和独立评审结果。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-17 | `cargo test-plan`、`cargo verify-changed` | 仅运行 CLI / 导入 parser 影响包、消费者及映射的 native 检查 | 选择器选中 L1 `panta-cli`、`panta-import`；L2 `panta-core`、`panta-ffi`、`panta-launcher`；native labels `app`、`bridge`、`ffi`、`i18n`、`mesh`、`qml`、`qt-adapter`。CLI 单元 5/5、CLI 进程 10/10、panta-core 115/115、activation 8/8、FSM build 4/4、project 39/39、FFI 32/32、panta-import 7/7、launcher 4/4、选择器回归 23 项通过 / 3 项忽略，native CTest 67/67；格式与适用 Clippy 通过。完整回退未触发。 |
| 2026-10-17 | macOS 非 UTF-8 文件系统夹具 | 在可创建该路径的文件系统上验证 CLI 原生路径读取及 UTF-8 来源名替换显示 | 当前 macOS 沙箱创建非 UTF-8 目录名返回 `PermissionDenied (os error 1)`，夹具明确跳过；Linux CI 将执行实际非 UTF-8 父目录与文件名进程测试。 |
| 2026-10-17 | PR #53 CI（run 38945520247） | macOS、Windows、Linux 构建与测试及适用质量检查通过 | 三平台 Cargo 构建、L1/L2 Rust 测试、受影响 native/QML 测试，以及格式、Clippy、CMake、include、cppcheck、clang-tidy、qmllint、machete 和分层检查全部通过；cargo audit 按变更范围跳过。 |

## 风险与回退

不要把解析成功描述为网格质量合格或可生成体网格。输出字段应来自已有领域结果；不在 CLI 中另写 STL 诊断算法。

## 决策与工作记录

- 2026-10-16：根据任务 121 的命令层盘点登记；该只读用例可作为导入脚本和诊断流程的前置检查。
- 2026-10-17：任务 122 合并后开始实施；新增 `stl inspect <path>`，直接调用 `ProjectService::inspect_stl`，摘要复用 GUI 预检的来源名、面片数和尺寸。
- 2026-10-17：非 UTF-8 路径保持原生 `Path` 供文件系统访问；只有用于 UI/CLI 的来源名元数据执行有损 UTF-8 显示，不将显示名重新用于路径访问。
- 2026-10-17：`gpt-6.1-sol` medium 独立只读评审未发现可操作问题；对非 UTF-8 basename 的有损显示复审确认实际读取仍使用原始 `PathBuf`，来源路径已有同类有损元数据策略。
- 2026-10-17：PR #53 独立最终评审通过，CI run 38945520247 全部适用检查通过。

## 完成摘要

CLI 预检命令、跨平台路径与失败诊断测试、README 和命令层文档均已完成；本地受影响范围验证、独立评审及 PR #53 三平台 CI 通过。
