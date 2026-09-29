# 080 — 视口文档页签与 STL 按需激活

- 状态：in-progress
- 阶段：应用平台扩展
- 依赖：[007](007-vtk-quick-viewport.md)、[063](063-stl-import-and-mesh-workspace.md)、[068](068-qml-project-and-layers-docks.md)、[073](073-fsm-dsl-and-import-state-machine.md)、[086](086-qt-platform-adapter.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-24 / 2026-09-29

## 目标与背景

将 `ViewportPane.qml` 底部目前用于 `Model / Mesh / Results` 分类切换的 `PanelTabBar` 替换为浏览器式文档页签。每次进入已打开的工程工作区时默认创建并选中 `Welcome`，显示现有 VTK `panta` 立体字样；成功导入 STL 后自动打开对应页签。用户点击工程树中的 STL 时，已打开的导入记录直接切到现有页签；尚未打开的记录先建立 Loading 页签，再从 `.panta` 工程资产异步读取并解析，成功后激活并显示网格。活动页签 ID 是视口显示内容的唯一选择状态：ViewModel 同步向 QML 投影选中项并向单个 CaeViewport 提供对应场景；QML 不并行维护另一份可分歧的选择值。每个页签（含 Welcome）均有关闭按钮；关闭只是结束当前 UI 会话中的打开视图，不删除工程记录、不改工程 dirty 状态。关闭最后一个页签后视口留空，用户可再从工程树打开 STL。配套整理左侧工程树与 Part / Study Tasks 检查器的行高、图标尺寸、选择底色、标题分隔线、缩进和文字对齐，让用户清楚看见所选 STL 与当前任务检查器的对应关系；不改变原工程树和检查器的数据/命令语义。

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
- 收敛本分支 CI 回归：Miri 大 STL 激活测试的解释执行耗时、macOS 页签动画时序断言，以及 Linux TSan 对未插桩 Rust `Mutex` 的跨语言误报；保留相应行为在 Rust、ASan/UBSan 和普通 QML 测试中的验证。

不包含：

- 不把点击树项、页签选中、关闭按钮或 hover 状态写成 Rust FSM；FSM 只覆盖异步读取/解析与结果相关性。
- 不改 `.panta` manifest schema，不把临时打开页签或活动页签持久化到工程，也不因视图切换推进工程 revision。
- 不为每个打开标签持有一套 `CaeViewport` / VTK actor 树，不把 mesh 数组交给 QML JavaScript。
- 不重设计 053 的 `panta` 字样、不增加网格编辑、结果云图、Study 多选择或视角动画。单个视口的每页相机状态是否恢复按前置决策确定；首期不承诺将相机状态写入工程。

## 前置条件与待决策

- 063 的项目导入、资产路径和失败回滚接口可供 Rust 服务读取；当前 ViewModel 仅公开导入名称列表和最新网格，需补齐包含稳定 `id` 与工程相对资产引用的只读记录模型。
- 073 冻结一个只读的工程资产激活契约：请求携带 project session / generation、导入记录 ID 与 attempt 相关性；Rust 校验工程记录并读取/解析已提交的 STL；返回拥有明确释放路径的 `SurfaceMesh` 快照；该路径不写清单、不推进修订。工程提交型导入 FSM 作为后续消费者另行登记。
- 打开页签的 Rust `ProjectService` 实现 128 MiB Mesh 缓存预算：保留活动文档 pin，超预算时按 LRU 逐出非活动 `SurfaceMesh`；页签及文档 ID 保留，驻留查询投影为 `unloaded`，再次选择时异步重载。单个活动 Mesh 可超预算，其他非活动 Mesh 可逐出。预算只计 Rust 三角数组的 `capacity × size_of::<triangle>()`，不代表 allocator RSS、C++ 活动显示 DTO、VTK 数据或 GPU 显存上限。ViewModel 只保留单份当前活动显示 DTO，不另建按文档索引的 Mesh 缓存。以 3 个真实 STL 合计约 45.66 MiB 为基线，并通过多份大模型副本比较 128 / 256 MiB：128 MiB 将 Rust Mesh 常驻控制在约 125 MiB，超预算选择调用 p95 约 0.04 ms、就绪 p95 约 22 ms；256 MiB 常驻约 213 MiB，缓存命中仍需 GUI 线程生成活动显示 DTO，选择调用 p95 约 11 ms。选定 128 MiB 以限制 Mesh 常驻并让超限切换走异步路径；常驻缓存命中的 DTO 物化目前仍同步执行，真实三资产选择 p95 约 9.5 ms。
- 冻结重复来源名、同一记录并发加载、加载中关闭/切换、工程关闭或重开、导入成功但 UI 尚未接收快照时的语义。页面显示状态不能取代 Rust 对 session、记录身份及 attempt 的校验。
- ViewModel 是活动文档 ID 与当前 viewport snapshot 的单一 UI 权威；QML 将其投影为选中页签并发送 open / activate / close 意图，不另行储存一个可能不同步的“当前 VTK 内容”。Rust `ProjectService` 管理 Mesh 数据驻留预算、LRU 与释放，并接收活动文档 pin；ViewModel 只投影批量驻留结果和持有当前活动显示 DTO。打开工程时 ViewModel 建立并激活 Welcome；页签选择与关闭属于 UI 选择状态，不使用 Rust FSM 或 Qt QStateMachine。
- 073 是 STL 按需读取的异步任务、取消和迟到结果的权威来源；C++ ViewModel 适配 Qt 类型、相关性事件和显示快照。异步结果只可更新匹配的打开文档快照；是否激活仍由当前活动文档 ID 决定。

## 实施步骤

1. 对照 063 / 068 / 073 审计现有导入记录、当前网格、QML 工程树和 VTK 场景的所有权；先冻结记录 DTO、异步激活、页签缓存与工程切换契约。
2. 将 HTML 参考更新为浏览器式视口页签、默认 Welcome 立体字样、项目树打开/复用 STL、切换和关闭状态；人工检查标签栏直接替换旧类别切换条。
3. 先完成 073 的 FSM 元数据生成与只读 STL 工程资产激活消费者；Rust 返回经相关性校验的完成 / 失败 / 取消结果及可释放快照。
4. 通过 ViewModel 暴露导入记录模型和单一活动文档 ID；QML 用一个视口和文档标签组件呈现状态；集成导入成功后的新页签激活，并同步整理工程树选中反馈与 Part / Study Tasks 检查器排版。
5. 在 Rust `ProjectService` 实现可测的 Mesh 缓存预算、LRU、驻留查询和关闭释放；ViewModel 只投影驻留状态并持有活动显示 DTO。处理项目代次失效、迟到结果、加载失败、重复点击、活动标签关闭与全标签关闭后的空白视口；删除旧 `Model / Mesh / Results` 视口切换路径和其仅由该路径使用的资源。
6. 完成 Rust / ViewModel / QML 行为测试、CPU / GPU 手动性能场景和真窗口 VTK 生命周期验收；实测大网格重复切换后的 mapper / WebGPU 资源回收、128 MiB 缓存超限逐出与重载，比较 128 / 256 MiB 后冻结预算和超限行为；同步 063、068、073、QML 规范或架构中确需更新的契约与索引。
7. 用 `gh` 核对失败作业与前次 Miri 日志，缩小激活测试样本且保持取消/去重/过期语义；修正动画断言等待；按 042 已登记的边界收窄 TSan 排除；为文档页签增加构造、切换、关闭的 CPU/GPU 消融场景并实测。

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

- [x] 每次进入工程工作区时只有活动 `Welcome` 页签并显示现有默认 `panta` 3D 场景；打开新项目不继承旧工程活动文件页签。Welcome 和 STL 页签都有关闭按钮，旧 `Model / Mesh / Results` 视口类别条不再出现。
- [x] 新导入成功后自动出现并激活一个 STL 页签；工程树点击同一稳定 ImportRecord ID 不会重复创建标签，点击未打开记录时先创建 Loading 页签，异步加载成功后激活。
- [x] 已打开页签之间切换时，ViewModel 活动文档 ID、PageTab 选中态与 CaeViewport 共用的数据源一致；关闭非活动标签不改变当前活动快照，关闭活动标签优先选右邻就绪页签；关闭全部标签后活动 ID、快照与页签模型为空。
- [x] 同名 STL 页签显示序号以区分标签，并将完整来源名和序号作为 Qt PageTab 无障碍名称；就绪页签可用 Enter / Space 激活。
- [x] 灰色标签带底边存在一条连续白色细线；活动标签上沿圆角完整，左右下角圆弧对称接入白线，交界无凸点、露底或错位；关闭图标视觉居中，hover / active 背景圆角为 5px，标签选中和各交互态无颜色分裂。
- [x] 页签可通过水平拖拽重排；拖动非活动标签时立即将其激活；拖动标签保持不透明、持续跟随指针且只沿 X 轴位移，跨越多个相邻标签期间拖拽不被中断，相邻标签平滑让位，松开后拖动标签动画归位；尊重减少动态效果偏好；垂直手势不触发重排，拖动关闭按钮不开始重排；重排后标签与其文档 ID、活动态及视口内容保持一致。
- [x] 文档页签宽度固定为 130px，长标题以省略号截断；文档标签带内边距为 `2px 2px 0 2px`，首个 tab 左侧仍缩进 2px；滚动裁切不截掉首末活动标签的圆弧，其他 `.tabs` 使用处维持原有间距。
- [x] 文件标签关闭只释放本次运行期视图数据，不删 ImportRecord / 资产、不设工程 dirty、不推进 revision；关闭 / 切换工程后所有旧 session 的页签、快照与结果失效。
- [x] 工程树 STL 的选中底色与对应 Part/Study Tasks 标题易辨认；树层级、任务行图标与文字对齐清楚，相关块的分隔、字号和密度与新文档页签一致。
- [x] 加载失败、取消、同记录重复请求、项目代次切换、旧 attempt 迟到成功 / 失败都有确定行为；旧结果不能覆盖当前视口或泄漏 mesh / VTK 资源。
- [x] 页签之间共用唯一的 `CaeViewport` 与 VTK render window；Rust `ProjectService` 管理 128 MiB `SurfaceMesh` 缓存与 LRU，ViewModel 只投影驻留态并保留当前活动显示 DTO；关闭页签通过服务释放对应运行期 Mesh。
- [x] 在真实 VTK 窗口重复切换与关闭页签后，旧 mapper 与 snapshot 弱引用释放；换宿主后旧 actor、interactor、render window、hardware window 和 WebGPU configuration 均释放，当前 interactor 观察器保持注册。日志确认 WebGPU device 销毁。
- [x] 代表性小 / 中 / 大 STL 已验证 128 MiB Rust Mesh 缓存的 LRU 逐出策略；活动文档固定驻留，单个活动 Mesh 可超预算；被逐出页签再次选择会异步重载。多资产结果及 128 / 256 MiB 比较见下方记录；选择 128 MiB 以限制领域 Mesh 常驻并避免超预算时同步读取 / 解析大文件。缓存命中时生成活动 C++ 显示 DTO 仍同步执行，真实大 STL 选择 p95 约 9.5 ms，作为已知延迟记录。重复替换大网格后旧 mapper / WebGPU 资源回收或稳定在有界高水位；记录 CPU / GPU 进程 RSS、切换 / 重载延迟，并冻结预算及超限行为。GPU 测量不单独报告专用显存。
- [x] 页签栏暴露 PageTabList / PageTab，标签和关闭按钮有可访问名称；Tab 获得键盘焦点后可用左右键导航并滚入可视区，Enter / Space 激活标签，关闭按钮可用 Space 或辅助技术 press action 执行。
- [x] 真实 VTK 窗口中，辅助技术激活 PageTab 会转移键盘焦点并更新无障碍焦点/选中态；页签焦点边框透明，不绘制黑色外框；Close 按钮的辅助技术动作会关闭对应标签。
- [x] 从全局搜索框沿 Shell 的 Tab 焦点顺序可到达文档 PageTab；页面真实辅助树暴露 Open documents / Welcome / Close Welcome。
- [ ] 在 VoiceOver 真读屏环境确认标签名称、选中态和关闭操作的朗读。
- [x] 适用 `qmllint`、格式、Cargo 聚合测试和 native 真窗口测试通过。
- [x] 每个新增 QML 组件及影响更新/布局/绘制成本的 QML 均进入 069 harness 的 CPU / GPU 手动性能场景；记录输入规模、采样、p50/p95、环境与测量边界。
- [x] 不保留旧视口分类切换路径；task、索引、HTML、C++/Rust 边界和性能记录相互一致。

## 验证计划与结果

常规验收使用 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check` 和 `cargo lint`。真窗口用例通过临时 `.app` wrapper 在 macOS 图形会话运行；069 CPU / GPU 基准仅手动执行，不作为 CI 门禁。

| 日期 | 验证 | 结果与边界 |
|---|---|---|
| 2026-09-24–25 | HTML 原型 `node --check`、交互静态审阅 | JS 和拖拽静态断言通过；当时浏览器拦截本地 file URL，后续改由真窗口验收。 |
| 2026-09-28 | GitHub Actions runs [36265495455](https://github.com/Yuki-Nagori/panta/actions/runs/36265495455)、[36402570741](https://github.com/Yuki-Nagori/panta/actions/runs/36402570741)、[36430454561](https://github.com/Yuki-Nagori/panta/actions/runs/36430454561)；本机 Miri / `cargo sanitize` | 两次旧 run 的问题分别为页签像素断言取到文字、动画断言过于精确、Miri 35 万面样本超时，以及 TSan 报告未插桩 Rust Mutex 队列。修复后 Miri 激活用例 8/8、本机 ASan/UBSan 62/62、TSan 47/47；TSan 按 042 边界排除两个 FFI 异步结果用例。run 36430454561 的 19 个 job 全通过（commit `ca8ad8a`），不包含其后的本地提交。 |
| 2026-09-29 | `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint` | 本次修改最终复验全部通过，CTest 66/66；格式、构建和 lint 八阶段通过。新增 Rust 缓存策略及重复 Import ID 的 FFI outcome 回归均通过。Miri 的 512 边容量压力用例因解释执行超过两分钟按既有边界跳过。 |
| 2026-09-29 | Release CPU / GPU STL 手动基准；显式释放旧 mapper 图形资源后再跑 GPU；`cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint --check` | 两个 CMake 手动目标构建通过；Cargo 聚合测试通过，CTest 66/66；格式及 lint 检查模式八阶段通过。默认 `cargo lint` 修复模式在 sandbox 中无法绑定本地 TCP 锁监听器，故使用仓库支持的 `--check` 完成全 lint。三资产完成 CPU 每文件 100 次热缓存重载和真实 Cocoa VTK WebGPU 窗口每文件 31 次切换。显式调用 `ReleaseGraphicsResources` 后 GPU 峰值仍为 1,651.12 MiB；关闭文档时 RSS 从未显式释放时的 1,299.62 降至 1,063.62 MiB；这是后续 VTK 法线过滤优化前的阶段结果。 |
| 2026-09-29 | STL 面法线直写消融；`cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint --check`；Release 真窗口基准两次 | Cargo 聚合测试和 CTest 66/66、格式、lint 八阶段通过。三个 STL 各 31 次切换，峰值 RSS 为 456.28 / 471.25 MiB；第二次关闭所有文档后 RSS 422.62 MiB、快照容量 0。`footprint` 切换后约 502 MiB，普通 malloc 大/小区 303 MiB、图形类约 190 MiB；真窗口确认 608,218 面模型外观。阶段快照与帧计时见下表。 |
| 2026-09-28–29 | PantaPreview 真窗口：导入、保存/重开 `.panta`、树项重新激活、页签拖动；VTK lifecycle 与 AX | STL 重开后可显示；拖动重排、AX 激活/关闭、焦点框均通过。新增真窗口回归连续替换 12 次网格，每帧后旧 `SurfaceMeshSnapshot` 弱引用均过期；测试 3 passed、0 failed。未单独统计 actor/mapper、回调或 GPU 设备对象。早前人工 Tab 尝试未进入标签栏；后续 Shell 焦点链回归通过。VoiceOver 朗读未验。 |
| 2026-09-29 | reduced-motion 与页面一致性审计 | QML 回归确认 `reducedMotion=true` 时重排直接定位；真窗口尚未确认动画观感。HTML、任务索引、单 `CaeViewport` / `DocumentTabBar`、Rust/C++ 职责和 069 场景一致，旧视口分类路径已审计。 |
| 2026-09-29 | Shell 焦点链与拖拽边界回归 | `cargo test --locked --workspace` 通过，CTest 66/66。Shell 从全局搜索按 Tab 遍历到 Welcome PageTab；四标签 QTest 鼠标事件跨越三项重排并验证跟手、仅 X 位移、不透明、活动/身份保留和松手归位；垂直手势及关闭按钮拖出不触发重排。PantaPreview AX 树含 PageTabList、Welcome 与 Close Welcome；VoiceOver 朗读仍未验。 |
| 2026-09-29 | 真窗口 VTK 生命周期与页签焦点外观 | 图形会话用例无 skip：12 次网格替换后旧 mapper / snapshot 释放，换宿主后旧 actor、interactor、render window、hardware window、WebGPU configuration 弱引用均过期；VTK 日志记录 device 销毁，3 passed / 0 failed。像素回归确认 PageTab 焦点边框透明；辅助树仍报告焦点与选中态。QML 透明色字面量统一引用 `Theme.colorTransparent`。 |
| 2026-09-29 | Release CPU 缓存预算性能与消融；Release GPU 基准尝试 | macOS 26.3.1 arm64、Qt 6.11.2；三资产各 100 次热页缓存重载，完整数据见下表。7 页签 / 5 个大 STL、217 次循环选择对比 128 与 256 MiB；128 MiB 峰值 RSS 326.34 MiB、Mesh payload 估值 125.29 MiB，选择 miss 217/217、调用 p95 0.035 ms、就绪 p95 21.715 ms；256 MiB 峰值 RSS 409.16 MiB、payload 估值 212.71 MiB、命中 217/217、同步选择 p95 10.840 ms。修正基准字段名，明确报告 Mesh payload 估值，不暗示 C++ 按页签保留 DTO。GPU 基准构建通过，但本次真实窗口运行失败：`Cannot create window: no screens available`，故没有新增 GPU 结果；面法线直写 GPU 消融见下表的既有真窗口实测。 |

### 页签手动性能

| 场景 | 配置与结果（p50/p95） | 测量边界 |
|---|---|---|
| CPU 构造 / 切换 / 关闭 | macOS 26.3.1 arm64、Qt 6.11.2、debug；1 次预热、31 次采样。1 页签构造 150/224 µs；8 页签构造 932/1204、切换 323/598、关闭 894/1267；24 页签构造 2790/4998、切换 442/2079、关闭 2932/5721。 | QML 构造、模型更新和事件处理，不含 GPU。 |
| GPU 帧间隔 | macOS 26.3.1 arm64、Qt 6.11.2、Metal、debug；1000×700，30 帧预热、每场景 3×60 帧。空场景 16.65/18.93 ms；静态 1/8/24 页签 16.70/25.60、16.69/25.27、16.70/24.90；切换 8/24 为 16.73/24.35、16.75/25.38；关闭/重开 8/24 为 16.71/24.59、16.67/18.96。 | 包含 vsync / compositor；只呈现 DocumentTabBar，不包含 VTK 网格场景或 GPU 内核耗时。 |

### STL 激活与显示 DTO 基线

真实工程测量使用用户提供的 `test_1.panta` 及其三个已保存资产：`Frame.stl`（47,946 面，约 2.3 MiB）、`Large_Earth_Elemental.stl`（608,218 面，约 29 MiB）和 `mug.stl`（8,834 面，约 431 KiB）。CPU 基准负责 ViewModel 工程激活/重载延迟、Rust Mesh 驻留估值和活动显示 DTO 容量；驻留估值用该资产的 DTO payload 代替 Rust `Vec` 实际容量，不表示 C++ 按页签保留 DTO。独立 GPU 基准在真实窗口测量单个 VTK 视口的帧提交及进程 RSS。两个手动目标和 CPU / GPU 源码分别集中于 `native/performance/CMakeLists.txt` 与 `tests/cpp/performance/`，不加入默认构建或 CTest。目标为 `panta_bridge_project_stl_activation_cpu_benchmark` 与 `panta_project_stl_viewport_gpu_benchmark`，通过 `cmake --build target/native/release --target panta_bridge_project_stl_activation_cpu_benchmark panta_project_stl_viewport_gpu_benchmark --parallel 4` 构建；CPU 目标接受工程路径和重载次数，GPU 目标接受工程路径及可选采样数，并要求可见原生图形窗口。CPU 重载重复运行时操作系统文件缓存为热状态；GPU 切换复用已加载 Mesh。进程 RSS 不代表专用 GPU 显存。

| 输入 | 解析 / 展开中位数 | 数据量与边界 |
|---|---|---|
| Netgen `part1.stl`（小型 ASCII） | 0.129 / 0.001 ms | 380 面，99,118 输入字节；Rust Mesh capacity 36,864 字节，坐标快照 27,360 字节。 |
| Netgen `hinge.stl`（小型 ASCII） | 0.423 / 0.002 ms | 1,212 面，318,065 输入字节；Rust Mesh capacity 147,456 字节，坐标快照 87,264 字节。 |
| 合成 binary STL | 0.592 / 0.198 ms | 100k 面，输入 5,000,084 字节；Rust Mesh 与展开坐标各 7,200,000 字节。仅为规模参照。 |

使用 `PANTA_BENCH_STL=<文件路径> cargo test -p panta-mesh --release --test mesh_performance supplied_stl_parse_and_snapshot_copy_cost -- --ignored --nocapture` 可测指定 STL。解析计时不含磁盘读取和应用激活链路。C++ 当前快照为每面三个未共享的 `double` 顶点，payload 约 72 字节/面；前表的小型 SDK 样本和 100k 合成模型本身不足以确定缓存预算，预算依据补充的用户工程和多资产测量见下文。

#### 用户工程 CPU / GPU 实测

环境：macOS 26.3.1、arm64、Release、Qt 6.11.2。CPU 目标以 100 次逐文件关闭 / 重开测量 ViewModel 与 Rust 激活链路，另以 31 轮页签选择测缓存命中与未命中。GPU 目标此前通过临时 `.app` 在真实 Cocoa 窗口中驱动同一个 VTK WebGPU viewport，每个模型测 31 次实际文档切换至 `frame submitted`；本次运行因没有可用屏幕未能启动。测试工程与资产均来自用户工作区，没有复制进仓库。

| CPU 阶段 | 进程 RSS / 峰值 | Rust Mesh payload 估值 | 说明 |
|---|---:|---:|---|
| Service baseline | 30.47 / 30.47 MiB | 0 MiB | 工程尚未激活资产。 |
| 三个 Rust Mesh 驻留 | 200.09 / 200.09 MiB | 45.66 MiB | 小 3.29、中 0.61、大 41.76 MiB；ViewModel 只保留当前活动 DTO。 |
| 各资产 100 次重载后 | 201.45 / 201.50 MiB | 45.66 MiB | RSS 在第 50 至 100 次间持平；文件页缓存热。 |
| 关闭全部文档后 | 201.45 / 201.50 MiB | 0 MiB | Rust Mesh 驻留归零；进程 RSS 未回落，符合 allocator 保留工作集的表现。 |

| 资产 | 初次激活 | 100 次重载 p50 / p95 | 口径 |
|---|---:|---:|---|
| `Frame.stl` | 12.420 ms | 12.818 / 13.372 ms | 100 次关闭 / 重开；文件页缓存热。 |
| `Large_Earth_Elemental.stl` | 34.981 ms | 20.270 / 20.628 ms | 100 次关闭 / 重开；文件页缓存热。 |
| `mug.stl` | 11.326 ms | 11.261 / 11.296 ms | 100 次关闭 / 重开；文件页缓存热。 |

三资产全部驻留时，以 31 轮顺序切换共 93 次，缓存命中页签选择 p50 / p95 为 0.725 / 9.472 ms。切换操作会同步把 Rust `SurfaceMesh` 转成活动 C++ 显示 DTO；计时不含 QML 事件派发或 VTK 帧。关闭 / 重开数据中，大 STL 每次约 20.6 ms。

#### 128 MiB 缓存压力场景

另建临时工程，包含用户原有三个资产及四份 `Large_Earth_Elemental.stl` 副本，共 7 个打开页签、5 个 608,218 面模型。macOS 26.3.1 arm64、Release、Qt 6.11.2 的 CPU 基准顺序循环 31 次，共 217 次选择。

| Rust 缓存预算 | 打开全部后的 Mesh 驻留 | 217 次选择命中 | 同步选择调用 p50 / p95 | 选择至就绪 p50 / p95 | 峰值进程 RSS |
|---|---:|---:|---:|---:|---:|
| 128 MiB | 3/7 页签，125.29 MiB | 0/217 | 0.024 / 0.035 ms | 20.273 / 21.715 ms | 326.34 MiB |
| 256 MiB | 7/7 页签，212.71 MiB | 217/217 | 9.655 / 10.840 ms | 9.657 / 10.845 ms | 409.16 MiB |

128 MiB 下 217 次选择均未命中，由异步路径处理；选择调用本身 p95 为 0.035 ms，选择至就绪 p95 为 21.715 ms。256 MiB 将估算 Rust Mesh payload 增加 87.42 MiB、峰值 RSS 增加 82.82 MiB；命中路径仍在 GUI 线程生成 C++ 显示 DTO，同步选择 p95 为 10.840 ms。预算取舍是限制 Mesh 常驻，同时让超限选择快速返回并异步恢复；在预算内的缓存命中路径仍有约 9.5 ms 的大模型 DTO 物化延迟。全部关闭后两种预算的逻辑 Mesh 容量均归零，进程 RSS 保留 allocator 工作集。该 CPU 基准不创建 VTK 数据，也未测专用显存；VTK 生命周期与帧提交高水位见上方真实窗口记录。

| 原管线 GPU 阶段 / 资产 | RSS / 峰值 | 快照 payload capacity | 帧提交或激活时间 |
|---|---:|---:|---:|
| 原生 viewport baseline | 218.02 / 218.28 MiB | 0 MiB | — |
| 三个快照驻留、大 STL 活动 | 569.22 / 614.30 MiB | 45.66 MiB | — |
| 完成三组各 31 次切换 | 1,348.17 / 1,651.12 MiB | 45.66 MiB | — |
| 关闭全部文档 | 1,063.62 / 1,651.12 MiB | 0 MiB | — |
| `Frame.stl` | — | — | 初次激活 130.347 ms；帧提交 p50 / p95 16.678 / 17.121 ms。 |
| `Large_Earth_Elemental.stl` | — | — | 初次激活 130.097 ms；帧提交 p50 / p95 83.336 / 83.402 ms。 |
| `mug.stl` | — | — | 初次激活 16.440 ms；帧提交 p50 / p95 16.660 / 16.738 ms。 |

| 面法线直写 GPU 阶段 / 资产 | 首轮 RSS / 峰值 | 复测 RSS / 峰值 | 帧提交 p50 / p95（首轮） |
|---|---:|---:|---:|
| 原生 viewport baseline | 213.03 / 213.03 MiB | 212.00 / 212.77 MiB | — |
| 三个快照驻留、大 STL 活动 | 411.78 / 453.50 MiB | 412.91 / 447.72 MiB | — |
| 各文件完成 31 次切换 | 413.09 / 456.28 MiB | 420.03 / 459.39 MiB | — |
| 关闭全部文档 | — | 422.62 / 471.25 MiB | — |
| `Frame.stl` | — | — | 16.677 / 16.759 ms |
| `Large_Earth_Elemental.stl` | — | — | 33.278 / 33.346 ms |
| `mug.stl` | — | — | 16.784 / 16.840 ms |

帧计时从 ViewModel 激活到 VTK 的 `frame submitted` 日志，不证明 GPU 执行完成或已呈现在显示器上；RSS 包含 CPU 工作集及图形栈进程内存，不包含可单独核对的专用显存。原管线中显式释放旧 mapper 图形资源，使关闭后的 RSS 相对未释放对照下降约 236 MiB，但 31 次切换峰值基本不变（1,653.05 → 1,651.12 MiB）。完整 malloc 栈记录指向 `vtkPolyDataNormals` / `vtkTriangleFilter` 的重复临时分配；STL 顶点本来按面独立，改为直接生成面法线并预分配 VTK 数组后，两次 31 轮的峰值降至 456 / 471 MiB，大 STL 帧提交 p50 由约 83 ms 降至约 33 ms。图形类 footprint 仍约 190 MiB，不能将这部分视作已量化的独立显存。基准可用 `PANTA_BENCH_PAUSE_PHASE=after_frame_cycles,all_documents_closed PANTA_BENCH_PAUSE_MS=60000` 在指定阶段暂停供 `footprint` / `heap` 采样。多资产 GPU 高水位与专用显存仍未测量；CPU Mesh 缓存逐出和异步重载的实测见下方压力场景。

## 风险与回退

主要风险是为每个标签复制 native 场景导致 GPU / CPU 内存乘法增长，或异步结果越过工程切换覆盖当前视口。优先维持单 `CaeViewport`，让 Rust 以工程代次、ImportRecord ID 和 attempt 校验结果；性能基准决定打开标签快照缓存上限。若缓存释放或切换期间 VTK 生命周期不稳定，退回到单活动网格并在已打开但未驻留的页签重新加载，保留标签去重和用户关闭语义。

## 决策与工作记录

- **2026-09-24–26：产品语义与边界。** 以浏览器式页签替换视口底部 `Model / Mesh / Results` 分类；工程会话默认打开 Welcome，全部关闭后留白。以 ImportRecord ID + session generation 标识文档；073 负责已保存 STL 的异步读取、取消和迟到结果校验，QML 只表达意图，C++ ViewModel 持有 Qt 页签状态与显示快照，Rust 保持领域权威。全页签共用一个 `CaeViewport`；就绪页签暂保留 CPU 快照，缓存上限待代表性测量。
- **2026-09-25–26：原型与实现契约。** HTML 原型完成切换、关闭、固定宽度与水平拖动。冻结 Loading / Ready / Failed 状态、稳定身份去重、失败保留旧视口、关闭 Loading 时取消、工程代次变化丢弃迟到结果；打开工程不同步重载旧网格。
- **2026-09-28：交互和视口修复。** 稳定委托与槽位坐标修正拖动跟手、释放归位和左右圆弧；关闭按钮补 hover / active 背景，Welcome 图标文字及光标恢复问题均补回归。针对用户提供的 `mug.stl`，确认“先关 Welcome 再双击树项”会激活页签但不显示；调整网格 actor 可见性与相机裁剪顺序后，真窗口显示恢复并获用户确认。该日完整评审清理死代码、修正激活 attempt 相关性并收敛 CI 问题。
- **2026-09-28：状态、可访问性与 CI。** 同名 STL 加序号并进入 PageTab 无障碍名称；ViewModel 覆盖切换、重排、关闭与快照释放；QML 覆盖 PageTab 角色、选中态、左右键、Enter / Space 和关闭。补齐 `DocumentTabBar` 中英文翻译及边缘像素测试。动画断言改为 0.1px 容差；远端 run 36430454561 的 19 个 job 全通过。
- **2026-09-29：平台偏好与焦点。** Qt 6.11 无统一 reduced-motion 属性，平台查询由 086 的适配层提供，QML 用 `Settings.reducedMotion` 控制重排；086 已完成。AX 激活标签后焦点转移与关闭动作有效；按页面外观要求，PageTab 焦点边框使用 `Theme.colorTransparent`，辅助树仍报告焦点/选中态。Shell Tab 链已通过回归，从全局搜索沿 Tab 到标签栏；VoiceOver 朗读仍待验。QML 的透明色统一使用主题 token。
- **2026-09-29：资源生命周期与性能。** 视口隐藏时也替换旧 VTK actor 管线，确保关闭页签后快照立即释放；真窗口验证隐藏清理及 12 轮连续替换。补充可指定真实输入的 STL Release 微基准，取得两个小型 SDK 样本与 100k 合成数据；目前只得到解析、展开和 payload 基线，不足以确定缓存预算或证明 VTK 对象无残留。
- **2026-09-29：用户工程真实 STL 实测。** 使用 `test_1.panta` 的三个资产完成 Release CPU 重载与真实窗口 GPU 切换测量。CPU 侧全部快照容量 45.66 MiB；三文件分别 100 次热缓存重载后 RSS 在第 50 至 100 次间稳定，关闭后快照容量归零。GPU 侧测得帧提交 p50/p95，真实窗口显示 608,218 面模型；增加旧 mapper 显式释放后，31 次切换峰值仍为 1,651.12 MiB，关闭后 RSS 从 1,299.62 降到 1,063.62 MiB。当时高水位仍未解释，Mesh 缓存预算仍待多资产 CPU 测量。CPU 与 GPU 基准源文件同置 `tests/cpp/performance/`，目标统一登记在 `native/performance/CMakeLists.txt`。
- **2026-09-29：高水位归因与修复。** `footprint` 的两次原管线 31 轮切换快照为 1,329 / 1,485 MiB，其中 MALLOC_LARGE + MALLOC_REALLOC 为 991 / 1,134 MiB，图形类约 190 MiB；`heap` 活跃 malloc 节点约 118 MiB。完整 malloc 栈记录的 5 轮累计分配 6.41 GiB，高水位调用树包括 `vtkPolyDataNormals::RequestData` 约 180 MiB、内部 `vtkTriangleFilter` 约 60 MiB。STL 快照每面独占顶点，C++ VTK 转换现直接生成面法线、精确预分配并跳过该 filter；Welcome 保留原管线。两次 31 轮峰值 RSS 降至 456 / 471 MiB，图形类约 190 MiB，真窗口大模型正常；普通 allocator 保留部分工作集。独立显存和未来多资产预算仍待单独测量，不能将本次修复等同于缓存策略已冻结。
- **2026-09-29：后续资产存储边界。** 当前工程保存原始 STL sidecar，首次打开页签仍按需解析；用户提出 `.panta` 元数据 + Mesh revision 索引、规范 Mesh 旁置并按页签读取。该设计能免重复 STL 解析但不替代 VTK / GPU 数据创建，独立实施计划见 [088](088-mesh-asset-sidecar-storage.md)。080 仍负责运行期页签快照生命周期与缓存预算，不改 manifest schema。
- **2026-09-29：Mesh 缓存策略。** 三个真实资产同时驻留约 45.66 MiB，大 STL 单项约 41.76 MiB。Rust `ProjectService` 按 128 MiB `SurfaceMesh` 容量预算运行，ViewModel 提供活动文档 pin，Rust 按 LRU 逐出非活动 Mesh 并批量报告驻留 ID；单个活动 Mesh 允许超限，预算不含 allocator RSS、C++ 活动显示 DTO、VTK 或 GPU 资源。7 页签、5 个大模型的 Release 临时工程验证了 128 MiB 超限逐出与异步重载，并与 256 MiB 对照：128 MiB 少约 87 MiB Mesh payload、峰值 RSS 少约 83 MiB；超预算选择调用 p95 约 0.04 ms、就绪 p95 约 22 ms。预算内命中会在 GUI 线程构造活动 DTO，三真实资产下 p95 约 9.5 ms；据常驻量与超限异步体验选定 128 MiB，完整口径见缓存压力表。
- **2026-09-29：Rust / C++ 边界复核。** 缓存预算、LRU、resident set 和 Mesh 关闭释放迁入 `ProjectService`。C++ ViewModel 仅持有页签 UI 状态和当前活动显示 DTO；QML 的 `unloaded` 是 Rust 驻留集合的展示投影。热路径核查确认 `MeshSource::mesh_snapshot()` 只返回本地 `m_activeMesh`；绑定与 `meshChanged` 才触发场景快照更新，VTK 相机/绘制循环不调用 Rust / FFI。批量 activation outcome 现先校验每条记录的当前 attempt，过期成功结果不会进入缓存或携带大网格跨 FFI；有效 DTO 由对应 outcome 自身的 Mesh 生成，缓存提交也会再次校验 attempt。
- **2026-09-29：一致性复核。** `ViewportPane` 仅连接一个 `CaeViewport` 和 `DocumentTabBar`；`PanelTabBar` 仍供 Tasks 面板使用。HTML 顶部 Mesh / Results 是全局导航，不是旧视口切换；任务索引、Rust/C++ 边界及 069 手动基准记录一致。

## 完成摘要

文档页签核心行为及此前 CI 回归已通过本机聚合、ASan/UBSan、TSan 与远端 run 36430454561（commit `ca8ad8a`）。动画测试改为容差等待；重复来源名已有可见与辅助技术消歧；ViewModel 活动快照切换/重排/关闭、Shell Tab 焦点链、四标签鼠标拖拽边界和 `DocumentTabBar` 中英文 QM 条目已有回归覆盖。086 已提供 Qt adapter API；真窗口检查覆盖 12 轮网格替换、隐藏关闭、换宿主后的 VTK 对象与 WebGPU configuration 回收；页签仍提供无障碍焦点状态，焦点边框引用 `Theme.colorTransparent`。代表性 STL 的 CPU 重载和 GPU 帧提交测量已完成；反复切换时的 RSS 高水位已归因于 STL 法线管线的重复大额分配，并通过直接面法线消融确认下降。128 MiB Rust Mesh 缓存预算经 7 页签压力场景与 256 MiB 对照后冻结；唯一未完成验收是 VoiceOver 真读屏朗读检查。
