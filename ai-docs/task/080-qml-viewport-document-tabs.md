# 080 — 视口文档页签与 STL 按需激活

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[063](063-stl-import-and-mesh-workspace.md)、[068](068-qml-project-and-layers-docks.md)、[073](073-fsm-dsl-and-import-state-machine.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-26

## 目标与背景

将 `ViewportPane.qml` 底部目前用于 `Model / Mesh / Results` 分类切换的 `PanelTabBar` 替换为浏览器式文档页签。每次进入已打开的工程工作区时默认创建并选中 `Welcome`，显示现有 VTK `panta` 立体字样；成功导入 STL 后自动打开对应页签。用户点击工程树中的 STL 时，已打开的导入记录直接切到现有页签；尚未打开的记录从 `.panta` 工程资产异步读取并解析，成功后建立页签并显示网格。活动页签 ID 是视口显示内容的唯一选择状态：ViewModel 同步向 QML 投影选中项并向单个 CaeViewport 提供对应场景；QML 不并行维护另一份可分歧的选择值。每个页签（含 Welcome）均有关闭按钮；关闭只是结束当前 UI 会话中的打开视图，不删除工程记录、不改工程 dirty 状态。关闭最后一个页签后视口留空，用户可再从工程树打开 STL。配套整理左侧工程树与 Part / Study Tasks 检查器的行高、图标尺寸、选择底色、标题分隔线、缩进和文字对齐，让用户清楚看见所选 STL 与当前任务检查器的对应关系；不改变原工程树和检查器的数据/命令语义。

首次打开旧导入记录是本功能的异步领域操作：Rust 要按稳定导入 ID 解析工程资产并重建网格，操作可能受文件大小影响；工程切换、关闭待加载页签或快速连续选择还会产生取消和迟到结果。因此它作为 [073](073-fsm-dsl-and-import-state-machine.md) 的首个 FSM 消费者。纯粹激活已就绪页签、改变选中态和关闭非活动页签仍是同步 UI 操作，不经 FSM。

## 必读

- [STL 导入与工程存储](063-stl-import-and-mesh-workspace.md)、[视口导航](064-vtk-navigation-and-orientation.md)、[视口显示后端](../architecture/visualization.md)
- [SVG 图标规范](../standards/icons.md)：本区树项与检查器图标使用同一正式资源体系。
- [QML 规范](../standards/qml.md)、[FSM 设计](../modules/fsm.md)、[Qt / Rust 边界](../architecture/native-domain-boundaries.md)
- [分层规则](../standards/layering.md)、[注释规范](../standards/comments.md)、[测试规范](../standards/testing.md)、[性能测试模块](../modules/performance.md)
- [代码生命周期](../standards/code-lifecycle.md)、[文档规范](../standards/documentation.md)、[验证与评审](../standards/validation-and-review.md)、[提交规范](../standards/commits.md)

## 范围与非目标

包含：

- 将视口底部旧 `Model / Mesh / Results` 滑块完整替换成可切换、可关闭且可水平拖拽重排的文档页签；拖拽和相邻页签重排动画只沿 X 轴进行，垂直位移不改变标签位置或顺序；尊重系统减少动态效果偏好；不移除其他面板仍使用的 `PanelTabBar`。灰色标签带下缘绘制连续细白线，活动标签左右下角圆弧过渡并接入白线，视觉按用户浏览器参考处理交界。
- 文档页签固定宽度为 130px；标题超出可用空间时以省略号显示。只将文档标签带的内边距设为 `2px 2px 0 2px`，不改变其他共用 `.tabs` 面板的间距；滚动裁切区可覆盖标签带的 2px 两侧留白，但首个 tab 仍从左侧 2px 处开始。
- 同步打磨工程树 STL 选中行、Part / Study Tasks 检查器标题和任务行的密度、缩进、图标比例及区块分隔；STL 选择仍表示“请求打开/激活视口文档”，任务行不新增未实现操作。
- 每次进入工程工作区或切换到新工程 session 时创建并默认激活一个 `Welcome` 页签；不恢复上个 session 的活动文件页签。Welcome 与文件页签一样提供关闭按钮，所有标签关闭后视口保持空白，不自动重建 Welcome。打开文件页签按稳定身份去重，标签文字使用来源名；同名文件需要可访问的消歧标签。
- 成功导入新 STL 后立即新增并激活对应页签，尽量复用导入事务已产出的网格快照，避免再次解析；点击工程树中已打开的记录只激活现有页签，未打开记录经 073 的异步 FSM 加载。
- 以 `ProjectImport.id` 作为工程内记录身份，并与工程 session / generation 一起组成运行期文档键；不得只按 basename 或源文件绝对路径去重。
- 明确待加载、已就绪、失败、取消和过期结果的 UI / Rust 表示。加载失败保留上一个可见视口；过期结果不得覆盖新工程或之后激活的网格。
- 视口区域始终复用一个 `CaeViewport` / native VTK render window；每个文档标签不创建独立 GPU 窗口、VTK interactor 或场景树。
- 记录打开页签的网格快照、VTK 显示数据、取消工作和关闭操作的所有权；关闭文件标签后，其独占数据与渲染资源应能释放。
- 新增或拆分 QML 组件时按 [QML 性能基准规范](../standards/qml.md#性能基准) 把组件加入 CPU / GPU 手动基准场景；在真实图形窗口比较切换和关闭行为。

不包含：

- 不把点击树项、页签选中、关闭按钮或 hover 状态写成 Rust FSM；FSM 只覆盖异步读取/解析与结果相关性。
- 不改 `.panta` manifest schema，不把临时打开页签或活动页签持久化到工程，也不因视图切换推进工程 revision。
- 不为每个打开标签持有一套 `CaeViewport` / VTK actor 树，不把 mesh 数组交给 QML JavaScript。
- 不重设计 053 的 `panta` 字样、不增加网格编辑、结果云图、Study 多选择或视角动画。单个视口的每页相机状态是否恢复按前置决策确定；首期不承诺将相机状态写入工程。

## 前置条件与待决策

- 063 的项目导入、资产路径和失败回滚接口可供 Rust 服务读取；当前 ViewModel 仅公开导入名称列表和最新网格，需补齐包含稳定 `id` 与工程相对资产引用的只读记录模型。
- 073 冻结一个只读的工程资产激活契约：请求携带 project session / generation、导入记录 ID 与 attempt 相关性；Rust 校验工程记录并读取/解析已提交的 STL；返回拥有明确释放路径的 `SurfaceMesh` 快照；该路径不写清单、不推进修订。工程提交型导入 FSM 作为后续消费者另行登记。
- 冻结打开页签数据保留策略。至少比较“所有打开页签保留 CPU 网格快照”与“受预算约束的缓存、被逐出的打开页签再加载”两种策略，使用代表性 STL 测量内存高水位、切换延迟与解析耗时；不得未经测量地复制多个 VTK 场景或无限保留网格。
- 冻结重复来源名、同一记录并发加载、加载中关闭/切换、工程关闭或重开、导入成功但 UI 尚未接收快照时的语义。页面显示状态不能取代 Rust 对 session、记录身份及 attempt 的校验。
- ViewModel 是活动文档 ID 与当前 viewport snapshot 的单一权威；QML 将其投影为选中页签并发送 open / activate / close 意图，不另行储存一个可能不同步的“当前 VTK 内容”。打开工程时 ViewModel 建立并激活 Welcome；页签选择与关闭属于 UI 选择状态，不使用 Rust FSM 或 Qt QStateMachine。
- 073 是 STL 按需读取的异步任务、取消和迟到结果的权威来源；C++ ViewModel 适配 Qt 类型、相关性事件和显示快照。异步结果只可更新匹配的打开文档快照；是否激活仍由当前活动文档 ID 决定。

## 实施步骤

1. 对照 063 / 068 / 073 审计现有导入记录、当前网格、QML 工程树和 VTK 场景的所有权；先冻结记录 DTO、异步激活、页签缓存与工程切换契约。
2. 将 HTML 参考更新为浏览器式视口页签、默认 Welcome 立体字样、项目树打开/复用 STL、切换和关闭状态；人工检查标签栏直接替换旧类别切换条。
3. 先完成 073 的 FSM 元数据生成与只读 STL 工程资产激活消费者；Rust 返回经相关性校验的完成 / 失败 / 取消结果及可释放快照。
4. 通过 ViewModel 暴露导入记录模型和单一活动文档 ID；QML 用一个视口和文档标签组件呈现状态；集成导入成功后的新页签激活，并同步整理工程树选中反馈与 Part / Study Tasks 检查器排版。
5. 实现可测的资源保留/释放策略，处理项目代次失效、迟到结果、加载失败、重复点击、活动标签关闭与全标签关闭后的空白视口；删除旧 `Model / Mesh / Results` 视口切换路径和其仅由该路径使用的资源。
6. 完成 Rust / ViewModel / QML 行为测试、CPU / GPU 手动性能场景和真窗口 VTK 生命周期验收，同步 063、068、073、QML 规范或架构中确需更新的契约与索引。

## 预计改动

- `qml/Panels/ViewportPane.qml`：替换旧底部类别页签并承接新面板布局；必要时新增文档标签组合组件及 Theme token。
- `ai-docs/qml-html/imported-project/imported-project.html`、`ai-docs/qml-html/shell.js`、`ai-docs/qml-html/shell.css`、`ai-docs/qml-html/README.md`：维护交互参考和明确演示边界。
- `crates/panta-core` / `crates/panta-import` / `crates/panta-ffi`：按 073 设计加入基于已保存 ImportRecord 的异步只读资产激活；具体子模块和 DTO 以接口评审为准。
- `native/bridge/src/project_view_model.*` 与视口数据适配：提供稳定导入记录、session 失效和显示快照，不把 VTK 类型或网格数组暴露到 QML。
- `tests/` 与 QML 手动性能基准 harness：覆盖加载、切换、关闭、失败、迟到结果、内存释放和真实视口更新；记录真实窗口与 CPU / GPU 性能结果。
- `ai-docs/task/063-*`、`068-*`、`073-*` 和架构 / QML 文档：按最终行为同步现有“最新导入项”假设和 FSM 首个消费者说明。

## 清理与兼容例外

完整删除 `ViewportPane.qml` 中只服务 `Model / Mesh / Results` 视口切换的旧标签状态、翻译入口和死代码；先确认 `PanelTabBar` 仍有其他消费者，不整体删除该通用组件。关闭页签不删除 STL 持久资产。无兼容例外。

## 验收标准

- [ ] 每次进入工程工作区时只有活动 `Welcome` 页签并显示现有默认 `panta` 3D 场景；打开新项目不继承旧工程活动文件页签。Welcome 和 STL 页签都有关闭按钮，旧 `Model / Mesh / Results` 视口类别条不再出现。
- [ ] 新导入成功后自动出现并激活一个 STL 页签；工程树点击同一稳定 ImportRecord ID 不会重复创建标签，点击未打开记录时先异步加载成功再打开。
- [ ] 已打开页签之间切换时，单一 ViewModel 活动文档 ID、选中页签和 CaeViewport 场景始终对应；标签名冲突时仍能区分对象；关闭非活动标签不改变当前视口，关闭活动标签按定义选择相邻页签；关闭全部标签后视口及标签栏为空白。
- [ ] 灰色标签带底边存在一条连续白色细线；活动标签上沿圆角完整，左右下角圆弧对称接入白线，交界无凸点、露底或错位；关闭图标视觉居中，hover / active 背景圆角为 5px，标签选中和各交互态无颜色分裂。
- [ ] 页签可通过水平拖拽重排；拖动非活动标签时立即将其激活；拖动标签保持不透明、持续跟随指针且只沿 X 轴位移，跨越多个相邻标签期间拖拽不被中断，相邻标签平滑让位，松开后拖动标签动画归位；尊重减少动态效果偏好；垂直手势不触发重排，拖动关闭按钮不开始重排；重排后标签与其文档 ID、活动态及视口内容保持一致。
- [ ] 文档页签宽度固定为 130px，长标题以省略号截断；文档标签带内边距为 `2px 2px 0 2px`，首个 tab 左侧仍缩进 2px；滚动裁切不截掉首末活动标签的圆弧，其他 `.tabs` 使用处维持原有间距。
- [ ] 文件标签关闭只释放本次运行期视图数据，不删 ImportRecord / 资产、不设工程 dirty、不推进 revision；关闭 / 切换工程后所有旧 session 的页签、快照与结果失效。
- [ ] 工程树 STL 的选中底色与对应 Part/Study Tasks 标题易辨认；树层级、任务行图标与文字对齐清楚，相关块的分隔、字号和密度与新文档页签一致。
- [ ] 加载失败、取消、同记录重复请求、项目代次切换、旧 attempt 迟到成功 / 失败都有确定行为；旧结果不能覆盖当前视口或泄漏 mesh / VTK 资源。
- [ ] 任意时刻最多一个 `CaeViewport`、一个 render window 和一套活动 VTK actor/mapper；关闭/切换重复循环后无旧 actor、回调、snapshot 或设备资源残留。
- [ ] 代表性小 / 中 / 大 STL 已验证快照保留或逐出策略；记录内存高水位与切换/重载延迟，达到约定预算时行为明确，不发生无界增长。
- [ ] QML 键盘 / 焦点 / 可访问名称可用；适用 `qmllint`、格式、Cargo 聚合测试和 native 真窗口测试通过。
- [ ] 每个新增 QML 组件及影响更新/布局/绘制成本的 QML 均进入 069 harness 的 CPU / GPU 手动性能场景；记录输入规模、采样、p50/p95、环境与测量边界。
- [ ] 不保留旧视口分类切换路径；task、索引、HTML、C++/Rust 边界和性能记录相互一致。

## 验证计划与结果

尚未实施。根目录按仓库锁定工具链执行 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check` 与 `cargo lint`。另外以受控后端测试每一终态、取消/迟到竞态和对象释放；用真实图形窗口验证 WebGPU 单视口替换数据及关闭释放，手动运行 069 CPU / GPU 基准，不把这些性能基准注册为 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | `node --check ai-docs/qml-html/shell.js`、差异空白检查与交互源审阅 | Welcome 默认态、导入记录打开/去重、切换、关闭、全标签关闭后空白逻辑无语法/结构问题 | JS 语法与静态检查通过；本地浏览器预览未能打开，页面目视复核待后续真实窗口验收 |
| 2026-09-25 | `node --check ai-docs/qml-html/shell.js`、`git diff HEAD --check`、页签拖拽静态断言 | 普通点击保留 tab 事件目标；非活动标签拖动时先激活；被拖标签不透明并跟随指针，只沿 X 轴移动；稳定指针捕获让重排可持续至松手；tab 固定 130px、长标题截断 | JS 语法、差异空白及目标静态断言通过；拖动阈值前不捕获指针，开始拖动后捕获稳定列表；实际浏览器拖拽未验收，本地 file URL 被浏览器安全策略拦截 |
| — | Rust FSM、Cargo 聚合、真窗口和 CPU / GPU 基准 | 按以上验收验证实际实现 | 未实施 |

## 风险与回退

主要风险是为每个标签复制 native 场景导致 GPU / CPU 内存乘法增长，或异步结果越过工程切换覆盖当前视口。优先维持单 `CaeViewport`，让 Rust 以工程代次、ImportRecord ID 和 attempt 校验结果；性能基准决定打开标签快照缓存上限。若缓存释放或切换期间 VTK 生命周期不稳定，退回到单活动网格并在已打开但未驻留的页签重新加载，保留标签去重和用户关闭语义。

## 决策与工作记录

- 2026-09-24：按用户要求以浏览器标签替换旧 `Model / Mesh / Results` 视口条；Welcome 展示现有 panta 默认场景并有关闭按钮；全标签关闭后视口保持空白，STL 页签根据稳定导入记录打开、复用和关闭。
- 2026-09-24：复核当前实现发现 Rust ProjectService 仅在内存持有最新网格，工程记录已有稳定 `ImportRecord.id` 与相对资产引用；首次激活其他已保存 STL 是真实读取/解析操作，拟作为 073 的首个异步 FSM 消费者。标签点击/关闭本身不进入 FSM。
- 2026-09-24：本任务只登记 HTML 原型和实现边界，不修改 QML 产品代码；打开标签的快照缓存上限需由真实 STL 基准决定。
- 2026-09-25：完成 HTML 参考的标签切换、关闭、固定宽度与水平拖动跟手和邻项动画；普通点击不捕获指针，真实拖动才由稳定列表捕获，拖动开始时激活目标标签。QML、ViewModel、FSM 和资源生命周期仍待实施。
- 2026-09-26：审计 063/068/073 现状后冻结契约（实施前置）。①相关性 DTO：ProjectService 新增会话级 `generation`（create/open 各递增，不持久化、不等于 revision）；`attempt` 为进程内单调 u64；信封为 `(generation, attempt, ImportRecord.id)`。②流程图 `open-saved-stl`：`Idle → LoadingAsset → Parsing → Ready`，终态 `Failed / Cancelled / Expired`；guard `record-valid` 在 begin 提交边界求值（拒绝时同步返回错误、不建 attempt、不落 Failed 终态），guard `session-current` 在完成边界求值；取消检查点为分块读取阶段与解析完成边界，`parse_stl` 本身不可中断（073 设计允许），边界 guard 拒绝即释放过期快照。③begin/去重：同 `(generation, record)` 的在飞 attempt 直接复用返回同一 id；终态后可重新 begin；generation 变更时协调器立即对在飞 attempt 投递 `generation-invalidated`，drain 再按当前 generation 过滤兜底，过期/迟到快照在 Rust 侧释放、不跨 FFI。④UI 语义：页签状态 `Loading / Ready / Failed`；点击未打开记录立即建 Loading 页签但不激活，成功后激活；失败保留 Failed 页签与上一个可见视口；Loading/Failed 页签不可被激活（拖拽仅重排），活动文档恒为 Ready；关闭 Loading 页签取消 attempt；关闭全部页签后视口空白（隐藏 welcome 字样，复用 `RenderScene.primitive_visible` 区分 Welcome 与空白）。⑤快照保留基线：所有打开 Ready 页签在 ViewModel 保留 CPU 快照（上限=打开页签数），单一 `CaeViewport`/VTK actor 切换；069 基准测量内存高水位与切换延迟后再评估预算逐出。⑥`open()` 不再同步重载最新导入网格（消除同步/异步双路径），`current_mesh` 仅由会话内 `import_stl` 产出，重开工程后视口为 Welcome。⑦ViewModel 是活动文档 ID 与视口快照唯一权威；工程树选中行为活动文档投影，Part 检查器标题跟随活动导入页签（无活动时回退最新记录，保持 068 其余语义）。

## 完成摘要

未完成。HTML 交互参考已更新；QML、ViewModel、FSM、缓存/释放和性能验收仍待实施，任务保持 `in-progress`。
