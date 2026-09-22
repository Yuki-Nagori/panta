# 063 — STL 导入、导入选项持久化与工程工作区

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[057](057-new-project-dialog.md)、[060](060-qml-project-workspace-reference.md)、[062](062-ribbon-tab-components.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-09-22

## 目标与背景

将 Home 工具栏的 Import 从视觉入口推进为第一条真实导入流程：当前先选择 `.stl` 文件，随后显示网格类型、单位、导入日志和近似尺寸确认窗口；确认后把源文件及影响几何解释的选项纳入 `.panta` 工程，并在工程树中展示零件、网格、填充、材料、注射位置、工艺设置、优化和分析等后续任务。右侧视口加载并渲染导入的 STL，后续再扩展其他格式。

本任务先完成可验收的 HTML 状态参考和持久化边界设计，再分阶段实现 QML、C++ ViewModel、Rust 导入服务与视口数据流。HTML 参考必须覆盖新建项目弹窗这个之前遗漏的状态。当前已落下首期 STL 运行时闭环，后续仍需补齐异步任务、真实网格生成及更完整的导入诊断。

## 必读

- [组件与主题模块](../modules/qml-components-and-theme.md)
- [应用平台、Rust 与工程存储](../architecture/application-and-storage.md)
- [外部求解器边界](../architecture/external-moldsolver.md)
- [QML 规范](../standards/qml.md)
- [分层与依赖方向](../standards/layering.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)
- [验证与评审](../standards/validation-and-review.md)

## 持久化决策

将导入选项记录在 `.panta` 内是正确的边界，但不应只保存 UI 当前值或原始绝对路径。导入确认后，工程 manifest 应保存可重放的 import record，至少包含：

- 工程内的源资产相对引用（首期将 STL 纳入目录包或由工程资产层托管），以及格式 / schema 版本；不能依赖用户电脑上的原绝对路径才能重新打开。
- `meshType`：`midplane`、`dual-domain`、`solid-3d`；这是影响后续网格解释的工程数据。
- `units`：首期支持界面中的明确单位选项，不能把“未指定”静默当成某个单位。
- `showImportLog`：记录用户对该次导入的日志可见性选择，但它只是展示偏好，不替代导入诊断和结果状态。
- 导入记录版本、STL 解析器版本、导入记录 ID，以及后续用于尺寸、拓扑和重建结果的稳定引用。

原始 STL 不应被修改；近似尺寸由解析结果计算并在确认窗只读展示。取消或导入失败不能写入工程 manifest、替换当前工程或清空已有视口。确认后的导入记录采用追加 / 版本化语义，避免改写其他已导入资产；具体 manifest schema 和资产复制策略在 native 实现阶段锁定并同步 034/048 的契约。

## 范围与非目标

包含：

- 首期文件选择器只接受 `.stl`，格式过滤和导入服务接口为可扩展集合，新增格式不复制一套流程。
- 导入确认窗口：Import 标题、网格类型选择、单位选择、近似尺寸、Show import log、OK / Cancel / Help；初始视觉状态参考用户提供的图一。
- 导入成功后的任务树与右侧 STL 视口状态，参考用户提供的图二；任务状态、错误、日志和后续 Create Mesh 入口先建立展示边界。
- `.panta` 内的 import record、源资产引用、失败回滚和重新打开恢复；QML 只发语义请求，C++ ViewModel 负责 Qt 适配，Rust 服务负责解析、校验、持久化和导入 DTO。
- 新建项目弹窗 HTML 参考，补齐 homepage 之前遗漏的 Create New Project 状态。
- 独立 HTML 状态参考：`open-project` 中点击 Import 后的导入弹窗，以及导入完成后的 `imported-project` 页面。

不包含：

- 不实现 Midplane、Dual Domain、Solid 3D 的实际网格生成，不把 Create Mesh 等任务伪装成已完成；首期只把 STL 三角面送入现有 VTK 视口。
- 不实现 STL 修复、单位推断、多个文件批量导入、其他 CAD 格式、导入日志编辑、材料和注射位置业务。
- 不把外部绝对路径、临时缓存或 UI 截图写入工程作为长期契约；不新建 target 或切换工具链。

## 实施步骤

1. 登记任务，补 homepage 新建项目弹窗、open-project 导入确认弹窗和 imported-project 工作区 HTML 参考。
2. 依据 HTML 锁定导入状态模型、manifest import record、资产引用和错误回滚边界。
3. 增加可扩展文件选择 / 导入请求 DTO，首期只实现 STL；解析器返回单位、尺寸、拓扑摘要和稳定资产引用。
4. 通过 C++ ViewModel 把导入确认、取消、日志和状态变更接到 Rust 服务；成功后追加工程记录并发布 Tasks / viewport 快照。
5. 增加 `.stl` 过滤、选项往返、失败不污染当前工程、重开恢复、版本不兼容和视口生命周期测试。
6. 按实际实现更新模块、manifest 规范、任务验证结果和索引状态；不要提前标记未落地的网格 / 渲染能力。

## 验收标准

- [x] task、持久化决策和首期 HTML 状态参考已登记；homepage 新建项目弹窗已补齐。
- [x] HTML 能表达 `.stl` 选择后的 Import 确认窗口，以及确认后的任务树和 STL 视口占位。
- [x] 首期文件选择只接受 `.stl`，且格式集合可在不复制导入流程的情况下扩展。
- [x] 取消 / 失败不修改工程；确认后 `.panta` 内有可重放 import record 和源资产引用。
- [x] 重新打开工程可恢复导入记录、任务树和 STL 视口；版本 / 单位 / 尺寸错误可诊断。
- [ ] QML / C++ / Rust 分层、静态检查、native 测试、真实窗口与视口生命周期验证通过。

## 验证计划与结果

已验证文档 / HTML 的相对资源路径、无网络依赖、公共壳层复用、模板完整性和 `git diff --check`；运行时阶段已在现有 `target/native/debug` 验证 Rust、CXX、QML 和 native。当前已通过 `cargo test -p panta-core -p panta-ffi --locked`、`cargo build -p panta-launcher --locked`、`cargo lint qmllint --check`、`cargo lint clippy --check`、两份 `.pa` 的 DSL 检查与格式检查、QML / C++ 格式检查，以及 7 个选定的 native / QML / i18n 回归测试；真实窗口下的视口生命周期复核仍待手工验收。

后续实现阶段在现有 `target/native/debug`、Qt 和 LLVM 基线下运行构建、QML lint、Rust/CXX 检查、导入单元 / 集成测试、四档缩放及 Cocoa 真窗口；不得用 HTML 预览替代 STL 解析或 VTK 生命周期验证。

## 风险与工作记录

- 2026-09-22：创建任务。用户确认当前先推进 `.stl`，后续支持更多格式；导入选项应写入 `.panta`，以便工程重开和结果复现。
- 2026-09-22：采用“源资产引用 + import record”而不是只保存绝对路径；`meshType` / `units` 属于可重放工程数据，`showImportLog` 保留为本次导入的展示偏好。
- 2026-09-22：先补 HTML 状态，不改运行时代码；新增新建项目弹窗、导入弹窗和导入后工作区参考。
- 2026-09-22：Rust 工程服务扩展 schema 2 和 `ImportRecord`，导入确认后复制 STL 到 `assets/imports/`，并为 schema 1 清单提供缺省 imports 读取。
- 2026-09-22：为 import record 固定 `record_version=1` 与 `parser_version=1`，使后续解析器升级可以显式拒绝或迁移旧记录。
- 2026-09-22：CXX `ProjectViewModel` 接入 STL 预检 / 导入 / 重开恢复，QML 接入 ImportDialog、任务树和视口资产路径；VTK 适配层增加 ASCII / 二进制 STL 到 `vtkPolyData` 的解析。
- 2026-09-22：将跨页面的导航、建模、分析、结果、报告、帮助和单位文案归并到顶部公共 i18n 分类，删除已无引用的页面专用重复上下文；`.pa` 的同长度源文案限制通过公共子分类保持显式隔离。
- 2026-09-22：尝试在现有 target 做无 Bridge 消融构建时发现旧生成 MOC 与配置切换不一致；未修改业务代码绕过，已恢复默认 Bridge=ON 构建配置并完成默认回归测试。

## 完成摘要

已完成任务登记、`.panta` 持久化边界说明、首期 HTML 状态参考，以及首期 STL 预检 / 导入 / 资产复制 / 重开恢复 / 任务树 / VTK 视口链路。剩余工作包括 qmllint 收口、真实窗口下的视口生命周期复核、异步任务化和实际 Create Mesh 流程。
