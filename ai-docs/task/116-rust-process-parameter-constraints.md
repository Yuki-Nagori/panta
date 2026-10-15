# 116 — Rust 工艺参数约束统一暴露

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：094, 095, 099
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

处理 [GitHub issue #34](https://github.com/Yuki-Nagori/panta/issues/34)：Fill、Gate Location 与保压曲线的数值边界目前分别写在 Rust 校验和 QML 输入控件中，容易发生定义漂移。工艺参数约束应由 `panta-core` 定义并用于领域校验，再通过 FFI 供 QML 绑定。

## 必读

- [分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [Fill 工艺设置](094-fill-process-settings.md)
- [Gate Location 工艺设置](095-gate-location-process-settings.md)

## 范围与非目标

- 为温度、流量、切换体积、保压时长、保压压力与浇口数量定义领域约束，包括边界开闭、整数属性与单位。
- Rust 持久化校验与草稿逐字段验证共用约束；只读约束及验证结果经 FFI/C++ 暴露给三个 QML 对话框。
- QML 输入范围、范围文案、确认状态和保压曲线绘图归一化均从暴露的约束与草稿验证结果派生。
- 不改变现有数值和业务含义，不重做 UI，不扩展到其他对话框，也不实现 issue #35 的状态机。

## 验收标准

- [x] QML 不再维护工艺参数的数值边界常量；Rust 约束经只读投影传播到输入、文案、绘图和确认状态。
- [x] 草稿验证返回稳定的字段错误键与整体有效状态；Rust 会拒绝的值不能通过弹窗确认。
- [x] Rust 边界测试覆盖闭区间端点、开区间流量下界、整数浇口数量及非法值。
- [x] 相关 QML / native / Rust 测试及本机 Cargo 聚合门禁通过。
- [x] 独立评审完成，问题已修复；任务记录和索引与交付状态一致。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | macOS arm64；`cargo test --locked --workspace` | Rust、native、QML 聚合测试通过 | 通过；CTest 76/76，其中 QML 与 ViewModel 约束回归通过 |
| 2026-10-16 | `cargo build --locked -p panta-launcher --no-default-features`；`ctest --test-dir target/native/debug -R Qml.ThemeComponentParameters --output-on-failure` | Bridge 关闭时 QML 组件测试可编译并运行 | 通过；CMake 确认 `PANTA_ENABLE_BRIDGE_MODULE=OFF`，组件测试通过 |
| 2026-10-16 | `cargo format --check`; `cargo lint --check` | Rust、C++、QML 格式与质量检查通过 | 通过；Clippy、依赖/CMake 检查、构建/QML lint、clang-tidy、include-cleaner、cppcheck 均通过 |
| 2026-10-16 | 工艺 QML 边界扫描与 `git diff --check` | 目标对话框无重复业务边界，差异无空白错误 | 通过；原有范围值只定义在 `panta-core` 约束中 |

## 清理与兼容例外

移除三个目标 QML 对话框中的重复边界判断与字面量；工程持久化格式不变。无兼容例外。

## 决策与工作记录

- 2026-10-16：登记 issue #34，开始将工艺参数约束统一到 Rust 领域模型。
- 2026-10-16：`panta-core` 为工艺数值定义区间、开闭边界、整数属性和单位；持久化校验与字段错误键共用约束。FFI/C++ 将约束投影到 `planSettings`，并提供 Fill、Gate Location 与曲线草稿验证接口。
- 2026-10-16：三个 QML 对话框从投影绑定输入范围、单位、范围文案、曲线归一化与确认状态；验证器不支持的开区间由 Rust 草稿验证处理，浇口数小数原样传入验证。
- 2026-10-16：独立评审发现备用空行提示、压缩曲线索引映射和边界存在标志传播不完整；已修复并增加回归测试。保压范围文案支持仅有上界。
- 2026-10-16：final review 发现无 Bridge 诊断构建的 QML 测试依赖未受 feature guard 保护；已增加条件编译和明确跳过，并验证 Bridge OFF 与默认 ON 两种测试路径。复核通过，无未解决发现。
- 2026-10-16：`cargo test --locked --workspace`（CTest 76/76）、Bridge OFF 目标构建/测试、`cargo lint --check` 通过。分支尚未推送，CI 验收待运行。

## 完成摘要

实现、本机验证和独立评审完成；尚未取得 CI 结果，保持 in-progress。
