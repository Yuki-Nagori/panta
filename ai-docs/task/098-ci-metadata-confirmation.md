# 098 — 工艺确认跨平台 CI 修复

- 状态：done
- 阶段：质量与构建
- 依赖：094, 095, 097
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-10-01 / 2026-10-02

## 目标与范围

修复 CI run [36744497800](https://github.com/Yuki-Nagori/panta/actions/runs/36744497800) 的失败：Windows 工艺确认重开快照断言，以及 Linux TSan 报告 Rust 后台元数据通道读写竞争。同时补齐 Rust 覆盖率门禁发现的 Gate Location / FFI 回归缺口，保持业务校验、异步写盘和覆盖率门槛。

## 必读

- [分层规则](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)

## 验收标准

- [x] 工艺配置重开回归按字段语义比较，保留持久化与失败保留状态断言。
- [x] 按任务 042 的既有插桩边界，仅在 TSan 排除新增的 Rust 异步 Fill 桥接用例；普通测试、ASan/UBSan 与 Rust Miri 保留，复查条件为双侧 TSan 插桩。
- [x] Cargo 聚合测试、格式与相关质量检查通过，远端平台证据如实记录。
- [x] 修复提交后的 Windows / Linux CI 通过。

## 验证与工作记录

- 2026-10-01：核对 CI run 36744497800 的全部 job，失败原因包括 Windows Fill 重开快照比较、Linux TSan 未观测到 Rust 通道同步，以及 Rust 覆盖率低于函数 89% / 行 92% 门槛。其余质量 job、macOS 构建与 sanitizer、Linux 构建及 Miri 均通过。
- Windows 创建路径由 Rust `PathBuf::join` 组合，重开路径由 QUrl / QDir 转换，分隔符文本可能不同。测试先比较非空 canonical 文件路径，再完整比较其余快照字段，保留工程身份、配置持久化与失败保留状态断言；不改变生产路径契约。
- TSan runner 仅补充 `FillSettingsConfirmAsynchronouslyAndReopenFromRust` 的精确排除，沿用任务 042 已登记的 Rust 未插桩边界。普通 CTest 和 ASan/UBSan 保留该测试；没有修改 suppression 文件。Rust 后台交付继续使用安全标准库通道，纯 Rust 侧由 Miri 验证。
- 补充 Gate Location 的默认值、参数/引用/修订校验、后台确认、重开、无效清单、写盘失败与解锁回归；补充 FFI 目录、材料、Fill、Gate Location 和语言适配的成功与错误路径。FFI 只做类型映射，生产业务职责未变。
- macOS 仓库根目录、Rust 1.98.1：`cargo test --locked --workspace` 通过（native / QML CTest 67/67）；`cargo coverage` 通过，函数 91.82%、行 94.06%，保持现有门槛；`cargo sanitize` 通过（ASan/UBSan 67/67，TSan 50/50）；`cargo ub-check` 通过（nightly-2026-09-15，含新增工程服务测试）；`cargo format --check` 与差异检查通过。
- `cargo build --locked` 与 `cargo lint --check` 全部八项通过。远端 Windows / Linux 的修复后 CI 尚未运行；本机结果不作为这两个平台的通过证据，任务保持 in-progress 待后续 CI 确认。

## 清理与兼容例外

无兼容例外。TSan 覆盖例外按任务 042 登记；不增加按堆栈通配的竞态抑制。

- 2026-10-02：通过 `gh run view 36890328640` 核验最新 [CI run 36890328640](https://github.com/Yuki-Nagori/panta/actions/runs/36890328640)，提交 `a0f0872061627a969d60e422a2c945e8fc22f7b7` 与当前 HEAD 一致，19 个 job 全部成功。Windows / Linux build/test 与 sanitizer、Rust coverage 及 Miri 均通过，修复后平台验收完成。

## 完成摘要

Windows 工艺配置重开比较、既有插桩边界下的 Linux TSan 排除及 Rust 覆盖率回归补充已完成；最新 CI 的三平台构建与测试、sanitizer、Rust coverage 和 Miri 均通过。既有双侧 TSan 插桩复查条件继续由任务 042 跟踪。证据：[CI run 36890328640](https://github.com/Yuki-Nagori/panta/actions/runs/36890328640)（`a0f0872`）。
