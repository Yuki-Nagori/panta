# 117 — 将工程文档工作区状态迁移到 Rust

- 状态：done
- 阶段：Rust 架构整理
- 依赖：[067 Rust 网格领域迁移](067-rust-mesh-domain-migration.md)、[080 QML 视口文档页签](080-qml-viewport-document-tabs.md)、[107 Rust 工程服务边界](107-rust-service-boundaries-and-revisions.md)
- 优先级：P1
- 负责人：维护者
- 创建 / 更新：2026-10-16 / 2026-10-16

## 目标与背景

对应 [GitHub issue #35](https://github.com/Yuki-Nagori/panta/issues/35)。`ProjectViewModel` 当前管理文档页签列表、活动文档、加载结果合并、关闭后的回退选择、网格驻留投影及错误文案映射。这些决策仅能通过 Qt 测试，且不便由其他 Rust 调用方复用。本任务将文档工作区状态与决策移入 `panta-core`，保持现有 QML 行为和 Rust 网格激活服务边界。

## 必读

- [分层规则](../standards/layering.md)
- [Rust/C++ 边界审计](../architecture/native-domain-boundaries.md)
- [注释规范](../standards/comments.md)
- [验证与评审](../standards/validation-and-review.md)
- [提交规范](../standards/commits.md)
- [代码生命周期](../standards/code-lifecycle.md)

## 范围与非目标

- 在 `crates/panta-core/src/project/workspace.rs` 建立纯 Rust 文档工作区模型，负责 Welcome / 导入文档列表、状态、活动选择、激活结果校验与合并、驻留集合投影、关闭后的邻页回退、重排，以及稳定错误消息 key 映射。
- 由 `ProjectService` 协调工作区状态与现有网格激活、取消、缓存驻留 API；并发/迟到结果必须依据稳定文档 ID 和 attempt 标识判定，不能覆盖重开的同 ID 文档。
- 通过 `panta-ffi` 批量暴露工作区快照和粗粒度变更结果。`ProjectViewModel` 只负责调用服务、触发 Qt 定时排出、将快照映射为 Qt/QML 属性并发射通知；移除重复的页签状态和决策实现。
- 错误码到稳定 message key 的映射由 Rust 所有；Qt/QML 保留对 message key 的翻译。
- 将尺寸格式化从 `ProjectViewModel` 移至 Rust 的可测试纯函数，并保持当前两位小数、乘号分隔及可选单位的显示行为。
- 更新相关 Rust、FFI、C++ 行为测试及本任务记录；不调整任务 032 的覆盖率目标。

非目标：新增 CLI 命令、修改 QML 页签交互设计、改变工程存储格式、网格加载线程模型或 VTK 渲染行为；不搬迁 Qt 定时器、QObject 生命周期及信号/属性机制。

## 前置条件与待决策

- 依赖任务 067、080、107 已完成；本任务沿用现有 ProjectService 的激活与网格缓存能力。
- 保持用户可见错误文本及 Qt 翻译方式不变；稳定 key 的格式由实现统一确定并在 Rust 测试中穷举映射。

## 实施步骤

1. 用纯 Rust 状态与事件模型表达现有页签行为，并为激活、关闭、回退、重排、驻留、并发结果及错误 key 添加单元测试。
2. 通过 `ProjectService` / FFI 提供完整快照和状态转换结果，删除 `ProjectViewModel` 中重复的状态机、索引查找与错误映射。
3. 迁移尺寸格式化并增加边界/格式测试，保持现有显示结果。
4. 运行 Cargo 聚合测试、格式与 lint；单独运行现有 C++ bridge 测试及激活性能基准，检查关键 UI 行为与性能变化。
5. 完成独立只读评审，修复发现并复验；记录结果后同步任务索引。

## 预计改动

- 新增 `crates/panta-core/src/project/workspace.rs`；修改 `ProjectService`、`panta-ffi` 的工作区 DTO/接口、`ProjectViewModel` 及 Rust/C++ 测试。
- QML 公共快照字段保持兼容，避免改变用户可见行为；不修改工程文件 schema。

## 清理与兼容例外

- 删除 `ProjectViewModel` 的页签列表、活动文档选择、结果 attempt 簿记、索引/驻留决策、关闭回退逻辑与错误码到文案映射；同步删除旧测试中只覆盖这些 C++ 决策的重复断言，保留 Qt 投影、信号和异步调度测试。
- 删除 C++ `format_dimensions` 实现及其调用侧格式化逻辑，改用 Rust 返回的格式化结果。
- 无兼容例外。

## 验收标准

- [x] issue #35 列出的页签状态机与错误映射逻辑已迁出 `project_view_model.cpp`；C++ 仅保留 FFI 调用、Qt 投影/通知和异步调度。
- [x] Rust 测试覆盖就绪文档激活、活动文档关闭回退（右邻优先、再左邻、无可用邻页时的行为）、重排、驻留同步、并发/迟到激活结果、重开同 ID 文档隔离、所有错误码消息 key 映射及尺寸格式化。
- [x] `tests/cpp/bridge` 现有 bridge 测试通过，验证 QML 可见快照、消息翻译、信号与异步调度行为未回归。
- [x] `native/performance` 的 `panta_bridge_project_stl_activation_cpu_benchmark` 通过；已记录当前结果及无法取得可比迁移前基线的限制。
- [x] `cargo test --locked --workspace`、`cargo format --check` 与 `cargo lint --check` 通过；无旧实现、失效引用或新增死代码。
- [x] GUI 页签打开/关闭/拖动排序与失败提示保持迁移前行为；真实窗口验收已完成。
- [x] 独立评审完成，评审问题已修复并复核；任务记录与索引同步。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-10-16 | macOS 26.2 arm64；`cargo test --locked --workspace` | Rust 与 native/QML 聚合测试通过 | 通过；全部 Rust workspace 测试通过，CTest 76/76。首次打开已保存导入及切回 Welcome 后驻留同步的回归用例通过。 |
| 2026-10-16 | macOS 26.2 arm64、Qt 6.11.2、Release；`panta_bridge_project_stl_activation_cpu_benchmark`；合成 10k/20k 三角面资产，各 31 次热缓存重载 | 激活、页签切换、关闭释放正常；观察迁移后耗时与内存 | 通过；62 次切换 call/ready p95 均 0.403 ms；重载 p95 分别 11.772 / 12.370 ms；缓存估算 2.06 MiB，关闭后归零。当前无同负载迁移前基线，不能据此声称完成前后回归比较。 |
| 2026-10-16 | `cargo format --check`、`cargo lint --check` | 格式与八阶段质量检查通过 | 通过；lint 八阶段全部退出成功，包含 Clippy、Cargo unused-deps、CMake、native/QML metadata、qmllint、clang-tidy、include-cleaner 和 cppcheck。构建过程提示 `native/mesh/src/netgen/netgen_mesher.cpp:91` 有一条既有 signedness 警告。 |
| 2026-10-16 | macOS 26.2 arm64 调试窗口，重建后通过 CUA 无障碍树操作 | 检查真实窗口中的页签打开、关闭、拖动排序和失败提示 | 通过：成功打开工程并导入 STL；同名导入显示序号，关闭导入页签后工程记录仍保留。用户在真实窗口手动确认拖动排序正常。再打开仅位于 `/tmp` 的验证工程，其资产路径指向缺失文件；失败页签无障碍描述为 `The saved STL asset could not be opened.`，并能读出标题及关闭按钮。首次捕捉遇到 `SCStreamErrorDomain Code=-3811`，本次重试成功；窗口与 AX 状态均可复核。 |
| 2026-10-16 | PR #38 CI（macOS、Ubuntu、Windows） | 跨平台构建、测试、覆盖率与 lint 通过 | 通过：macOS、Ubuntu、Windows 构建，Rust/native/QML 测试、coverage、格式及适用 lint 全部通过；`cargo audit` 按工作流配置跳过。CI 首轮发现 Linux `uint64_t` 到 `QVariant` 转换歧义，改用显式 `QVariant::fromValue<qulonglong>` 后复跑通过。 |

## 风险与回退

- 页签状态与 ProjectService 网格缓存具有联动，关闭、失败、取消和迟到结果处理不一致会造成错误活动页或旧场景残留。先以状态快照明确状态转换，再集成调用点；测试需覆盖 attempt 替换与关闭后的迟到结果。
- 通过工作区快照一次性投影至 Qt，避免逐条 FFI 查询。性能基准若显示显著回退，检查复制与快照构造成本，不恢复双份业务状态。
- 回退时按提交撤销本次迁移；工程文件格式不变，不需数据转换。

## 决策与工作记录

- 2026-10-16：根据 issue #35 创建实施任务；采用 `panta-core` 纯 Rust 工作区模型，由现有 `ProjectService` 协调激活与缓存，C++ 只保留 Qt 适配和异步定时调度。
- 2026-10-16：独立评审发现首次打开文档未启动激活及缓存逐出后页签驻留未同步；已修复并新增服务级回归测试，评审者确认修复正确。
- 2026-10-16：真实窗口验收重试成功；用户确认拖动排序正常。失败资产页签显示 saved-asset 错误文案。复评发现同步激活失败的 message key 应使用 saved-asset 翻译上下文；已修复并增加 `project.manifest_invalid` 回归测试，复评确认修复及断言充分。
- 2026-10-16：PR #38 跨平台 CI 首轮暴露 Linux 对 `std::uint64_t` 到 `QVariant` 的重载歧义；改为显式 `qulonglong` QVariant 后，本地聚合验证、独立复评及 macOS/Ubuntu/Windows CI 均通过。

## 完成摘要

实现、独立评审、Cargo 聚合测试、质量检查、性能基准、真实窗口验收及 PR #38 跨平台 CI 均已完成。性能记录说明缺少可比的迁移前基线；CI 中 `cargo audit` 按配置跳过。任务验收完成。
