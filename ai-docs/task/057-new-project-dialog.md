# 057 — 新建项目对话框与工程命令边界

- 状态：done
- 阶段：应用平台扩展
- 依赖：[023](023-cross-platform-paths.md)（已落地路径宿主）、[029](029-qml-component-library.md)（已落地原子组件）、[055](055-qml-review-and-cleanup.md)（已完成 QML 整理）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

将左侧 `New Project` 入口从视觉占位推进为可用的现代化 QML 对话框。对话框提供项目名、保存目录、目录选择和说明，默认保存根目录为系统 `Documents/panta`，确认后生成 `Documents/panta/<name>/<name>.panta` 工程目录包，目录内可继续放置其他工程文件，底部只保留 `OK` 与 `Cancel`。新建项目命令必须经显式 ViewModel 信号进入 native 服务，QML 不直接访问文件系统。

同时把打开文件、创建项目、保存和工程模型命令的最小 Rust application service 一起落地，形成可复用的后端范例：QML 只表达用户意图，C++ ViewModel 负责 Qt/QML 适配、路径转换和错误呈现，工程模型、清单读写与命令校验归 Rust 服务。

## 必读

- [界面、C++ 桥接与交互](../architecture/ui-and-bridge.md)
- [应用平台、Rust 与工程存储](../architecture/application-and-storage.md)
- [跨平台路径与轻量运行时](../modules/paths-and-runtime.md)
- [原子组件库、Theme 与主题 DSL](../modules/qml-components-and-theme.md)
- [QML 规范](../standards/qml.md)
- [分层与依赖方向](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [C++ 编码与资源管理](../standards/cpp.md)
- [CXX 桥接规范](../standards/cxx.md)
- [提交规范](../standards/commits.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

包含：

- 将 `TasksPanel` 的新建入口改为语义信号；
- 以原子/组合组件拼装新建项目对话框，保持 Theme 尺寸和 Basic Controls 样式；
- 通过 C++ ViewModel 提供 Documents/panta 默认目录、目录选择结果和创建失败诊断；
- 为打开工程提供 `.panta` 文件选择、路径校验和 Rust 清单加载结果；
- 创建项目目录前校验名称、路径边界与已存在目标，成功后关闭对话框；
- 通过 CXX 暴露最小工程服务 DTO：创建、打开、保存和一个可验证的工程模型命令；覆盖所有权、错误和重复打开/保存生命周期。
- 新建项目使用无边框独立窗口：保留应用模态、默认居中和可移动能力，由 QML 标题栏提供拖动与关闭入口。
- 打开工程入口改为 `Open Project`，直接使用 Qt 系统文件选择器过滤 `.panta`，并调用已落地的 Rust 打开命令。
- 工程持久化契约采用目录包中的 `<name>.panta` 主文件；文件内部先使用 schema 1 JSON manifest，目录可承载后续几何、网格等资产。

不包含：完整工程清单格式、工程资产持久化、打开项目后的工程树模型、帮助按钮和额外对话框操作。

## 前置条件与待决策

- Qt 宿主能够发现并验证 `QStandardPaths::DocumentsLocation`；默认根目录为空或不可写时显示可恢复错误，不回退到 cwd。
- `FolderDialog` 的平台可用性需在锁定 Qt 供给上验证；若平台对话框不可用，保留可编辑的绝对路径输入并由 ViewModel 验证。
- 需要在当前 FFI 模块中新增真实消费者使用的 project service DTO；不把 Qt 类型、QObject 或 QML 状态放进 Rust。

## 实施步骤

1. 核对现有 TasksPanel、Shell ViewModel、PathHost 和 QML 模块构建边界，登记本任务。
2. 增加 QML 原子/组合组件所需的最小输入属性与语义信号，接入新建项目对话框。
3. 在 `panta-core` 建立最小工程模型和持久化契约，在 `panta-ffi` 暴露创建、打开、保存和模型命令的明确 DTO/错误结果。
4. 增加 C++ ProjectViewModel adapter：发现默认 Documents/panta、调用 Rust service、转换 Qt URL/QString 与错误摘要；Qt 文件夹选择只负责 UI 结果。
5. 为名称校验、默认路径、创建/打开/保存、非法命令和重复生命周期补充 Rust/CXX/native 测试，并运行 QML 静态检查与模块加载测试。
6. 更新任务验证记录和索引状态；不保留旧的无动作入口或未使用的兼容代码。

## 预计改动

- `qml/Components/Atoms/`、`qml/Components/Composites/`、`qml/Dialogs/`：新增可复用输入、按钮/表单和新建项目对话框；具体文件以实现中的独立职责为准。
- `qml/Components/Composites/`：无边框窗口标题栏组合组件，集中处理拖动、标题和关闭按钮。
- `qml/Panels/TasksPanel.qml`、`qml/App.qml`、`qml/CMakeLists.txt`：语义信号、ViewModel 注入和模块资源登记。
- `crates/panta-core/`、`crates/panta-ffi/`：最小工程模型、清单存储与 CXX DTO/错误契约。
- `native/bridge/src/` 与 `native/bridge/CMakeLists.txt`：QML 可见的项目 ViewModel 及 Qt adapter；不在 C++ 复制工程规则。
- `tests/cpp/bridge/` 与 Rust 单测：临时目录、非法名称、已存在目标、创建/打开/保存及模型命令测试。
- `ai-docs/task-index.md`：登记 057。

## 清理与兼容例外

删除 TasksPanel 中无动作的新建占位处理；不保留重复信号或兼容入口。无兼容例外。

## 验收标准

- [x] 点击 `New Project` 打开现代化对话框，字段、说明、目录选择和底部 `OK`/`Cancel` 均可见且尺寸来自 Theme。
- [x] 对话框无系统边框，默认相对主窗口居中，可通过自定义标题栏移动，打开后主窗口不可点击。
- [x] `Open Project` 入口直接打开系统文件选择器并过滤 `.panta`，确认后通过 ViewModel/Rust 加载已有工程清单。
- [x] 默认目录显示并使用 `Documents/panta`；名称合法时创建 `Documents/panta/<name>/<name>.panta`，非法、不可写或已存在目标显示可读错误且不关闭对话框。
- [x] QML 只发语义信号并通过显式 ViewModel 属性/命令工作，不直接调用文件系统或 Rust/Qt 私有对象。
- [x] 创建、打开、保存和模型命令均由 C++ ViewModel 适配到 Rust application service；路径、DTO 所有权、错误码和线程边界有测试证据。
- [x] 原子组件边界、翻译源文案、模块登记、测试和任务记录一致，无死代码或未登记兼容分支。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-22 | macOS arm64，`cargo run --locked -p panta-tests -- test` | Rust workspace、native CTest、QML 格式与模块加载通过 | 54/54 native CTest 通过；含 `.panta` `ProjectViewModelTest`、`Qml.FormatCheck`、`Qml.ShellModuleLoads`、i18n 与 FFI 测试 |
| 2026-09-22 | `cargo test -p panta-core --locked`、`cargo test -p panta-ffi --locked` | Rust 工程服务与 CXX 适配测试通过 | panta-core 34/34、panta-ffi 11/11；覆盖目录包创建、只接受 `.panta`、清单 schema、保存和重命名生命周期 |
| 2026-09-22 | `cargo test -p panta-dsl-core --locked`、`cargo clippy --locked --workspace --all-targets -- -D warnings` | 翻译产物和 Rust 质量门禁通过 | dsl-core 35/35；Clippy 无告警；修复 TS 全局重复 message ID 导致的 Qt 翻译丢失 |
| 2026-09-22 | `cargo lint qmllint --check`、`cargo lint includes --check`、`cargo lint clang-tidy --check`、`git diff --check` | QML/C++ 静态诊断与差异空白检查通过 | 通过 |
| 2026-09-22 | `CARGO_TARGET_DIR=target/ablation-no-bridge cargo build --locked --no-default-features -p panta-launcher` | 去掉 Bridge 后仍能构建最小 Shell，验证新建工程功能没有反向污染关闭路径 | 通过；Bridge-off 构建与默认 Bridge-on 集成测试相互独立 |

## 风险与回退

目录创建是用户可见的持久化副作用，仅在 `OK` 且校验通过后执行；失败保留对话框输入。若平台目录选择器不可用，使用可编辑路径并显示错误，不静默写入其他位置。回退时删除新对话框连接，保留已完成的原子组件和 ViewModel 单测。

## 决策与工作记录

- 2026-09-22：创建任务。根据分层文档确定 QML → C++ ViewModel → application service/Rust 的信号边界。
- 2026-09-22：用户要求将最小创建/打开/保存/模型命令作为本任务范例一并实现，移除“后续再接 Rust”的非目标。

## 完成摘要

完成新建工程独立无边框应用模态窗口：默认定位到 `Documents/panta`，自定义标题栏支持平台系统移动和关闭，底层主窗口在弹窗期间不可点击。工程目录包固定为 `Documents/panta/<name>/<name>.panta`，打开入口使用系统文件选择器过滤 `.panta`。QML 使用 `ThemedTextField` 与 `DialogTitleBar` 原子/组合组件；C++ `ProjectViewModel` 仅做 Qt 适配，创建、打开、保存和重命名命令由 Rust `ProjectService` 持有并经 CXX DTO 调用。schema 1 JSON manifest 与完整错误/生命周期测试同步更新。
