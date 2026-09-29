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
- [ ] 页签可通过水平拖拽重排；拖动非活动标签时立即将其激活；拖动标签保持不透明、持续跟随指针且只沿 X 轴位移，跨越多个相邻标签期间拖拽不被中断，相邻标签平滑让位，松开后拖动标签动画归位；尊重减少动态效果偏好；垂直手势不触发重排，拖动关闭按钮不开始重排；重排后标签与其文档 ID、活动态及视口内容保持一致。
- [x] 文档页签宽度固定为 130px，长标题以省略号截断；文档标签带内边距为 `2px 2px 0 2px`，首个 tab 左侧仍缩进 2px；滚动裁切不截掉首末活动标签的圆弧，其他 `.tabs` 使用处维持原有间距。
- [x] 文件标签关闭只释放本次运行期视图数据，不删 ImportRecord / 资产、不设工程 dirty、不推进 revision；关闭 / 切换工程后所有旧 session 的页签、快照与结果失效。
- [x] 工程树 STL 的选中底色与对应 Part/Study Tasks 标题易辨认；树层级、任务行图标与文字对齐清楚，相关块的分隔、字号和密度与新文档页签一致。
- [x] 加载失败、取消、同记录重复请求、项目代次切换、旧 attempt 迟到成功 / 失败都有确定行为；旧结果不能覆盖当前视口或泄漏 mesh / VTK 资源。
- [x] 页签之间共用唯一的 `CaeViewport` 与 VTK render window；ViewModel 只保留仍打开的导入文档快照，重复切换不丢失已打开快照，关闭页签释放其独占快照。
- [ ] 在真实 VTK 窗口重复切换与关闭页签后，确认没有旧 actor/mapper、回调、snapshot 或设备资源残留。
- [ ] 代表性小 / 中 / 大 STL 已验证快照保留或逐出策略；记录内存高水位与切换/重载延迟，达到约定预算时行为明确，不发生无界增长。
- [x] 页签栏暴露 PageTabList / PageTab，标签和关闭按钮有可访问名称；Tab 获得键盘焦点后可用左右键导航并滚入可视区，Enter / Space 激活标签，关闭按钮可用 Space 或辅助技术 press action 执行。
- [x] 真实 VTK 窗口中，辅助技术激活 PageTab 会转移键盘焦点并显示焦点框；Close 按钮的辅助技术动作会关闭对应标签。
- [ ] 在 VoiceOver 真读屏环境确认标签名称、选中态和关闭操作的朗读；从全局搜索框按 Tab 进入标签栏的焦点路径也待验证。
- [x] 适用 `qmllint`、格式、Cargo 聚合测试和 native 真窗口测试通过。
- [x] 每个新增 QML 组件及影响更新/布局/绘制成本的 QML 均进入 069 harness 的 CPU / GPU 手动性能场景；记录输入规模、采样、p50/p95、环境与测量边界。
- [ ] 不保留旧视口分类切换路径；task、索引、HTML、C++/Rust 边界和性能记录相互一致。

## 验证计划与结果

根目录按仓库锁定工具链执行 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check` 与 `cargo lint`。另外以受控后端测试每一终态、取消/迟到竞态和对象释放；用真实图形窗口验证 WebGPU 单视口替换数据及关闭释放，手动运行 069 CPU / GPU 基准，不把这些性能基准注册为 CI 时间门禁。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-24 | `node --check ai-docs/qml-html/shell.js`、差异空白检查与交互源审阅 | Welcome 默认态、导入记录打开/去重、切换、关闭、全标签关闭后空白逻辑无语法/结构问题 | JS 语法与静态检查通过；本地浏览器预览未能打开，页面目视复核待后续真实窗口验收 |
| 2026-09-25 | `node --check ai-docs/qml-html/shell.js`、`git diff HEAD --check`、页签拖拽静态断言 | 普通点击保留 tab 事件目标；非活动标签拖动时先激活；被拖标签不透明并跟随指针，只沿 X 轴移动；稳定指针捕获让重排可持续至松手；tab 固定 130px、长标题截断 | JS 语法、差异空白及目标静态断言通过；拖动阈值前不捕获指针，开始拖动后捕获稳定列表；实际浏览器拖拽未验收，本地 file URL 被浏览器安全策略拦截 |
| 2026-09-28 | [GitHub Actions run 36265495455](https://github.com/Yuki-Nagori/panta/actions/runs/36265495455) | 当前分支 CI 全绿 | Ubuntu 构建测试、native coverage/QML、sanitizer 三处因同一 `ThemeComponentTest::document_tab_connectors_follow_html_reference` 像素断言失败：`(60,20)` 为灰色文字像素，非白色页签背景；其他已运行检查成功，Miri 作业取消。待修复并重跑。 |
| 2026-09-28 | `cargo build --locked`；临时 `PantaPreview.app` 真实窗口 | Welcome/导入页签与 GPU 网格可见；拖拽重排并归位 | 构建通过。真窗口显示白色 P、两侧页签圆弧；临时工程导入三角形 STL 后网格可见；保存 `.panta`、重开、双击树中 STL 后网格仍可见；两个页签拖动后换序并归位。 |
| 2026-09-28 | `cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、`git diff HEAD --check` | 聚合回归、格式、静态检查和差异空白检查通过 | 全部通过；CTest 62/62，包含页签像素、鼠标释放光标与重排回归；lint 完成 Clippy、依赖、CMake、qmllint、clang-tidy、include-cleaner、cppcheck。 |
| 2026-09-28 | 分支整体评审后 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、`git diff main --check` | 清理后构建、回归与静态检查通过 | 构建、聚合测试（CTest 62/62）、格式及差异检查通过。lint 首次受沙箱 TCP 锁限制，授权重跑后完整 8 阶段通过；clang-tidy 首轮自动修复与同时编辑冲突，源码已修复并以串行重跑通过。 |
| 2026-09-28 | 提交前 `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、暂存差异检查 | 以本次提交内容复核代码、测试和文档 | 构建与格式检查通过；聚合测试 CTest 62/62；lint 完整 8 阶段通过。活动页签关闭按钮和资产丢失消息的新断言均通过；本次没有重新执行真窗口或手动性能基准。 |
| 2026-09-28 | `gh run view 36402570741 --job ... --log`，并对照 run 36265495455 的 Miri 作业 | 定位本分支远端失败 | 两次 Miri 都在 `tests/rust/activation.rs::duplicate_begin_reuses_in_flight_attempt` 的 35 万三角形样本处失败或卡顿，最新作业在约 9 分钟时被取消；macOS ASan/UBSan 是动画 `x` 尚为 `2.087...` 时断言等于 `2`；Linux TSan 仅两个 ViewModel 异步激活用例报告 Rust 未插桩 `Mutex` 结果队列竞态，属于 042 已登记边界。clang-tidy 作业成功，两个手动 benchmark 源文件都进入其编译数据库并接受静态检查，程序本身未作为 CI 性能门禁运行。 |
| 2026-09-28 | `MIRIFLAGS=-Zmiri-disable-isolation CARGO_TARGET_DIR=target/miri cargo +nightly-2026-09-15 miri test --locked -p panta-core --test activation` | 缩小解释执行样本后仍覆盖成功、取消、去重和代次失效 | macOS arm64 本地 8/8 通过，约 6.93 秒；首次 2000 三角形仍在 10 秒等待限内失败，缩至 100 后通过；代次失效用例给故意分离的 worker 留出结束时间，避免测试进程退出时 Miri 报未结束线程。远端 CI 尚待新提交复跑。 |
| 2026-09-28 | `MIRIFLAGS=-Zmiri-disable-isolation CARGO_TARGET_DIR=target/miri cargo +nightly-2026-09-15 miri test --locked -p panta-core -p panta-dsl-core -p panta-foundation` | 全量纯 Rust Miri；容量压力按 032 边界处理 | macOS arm64 退出码 0：`panta-core` 40 单测、8 激活及 15 工程集成测试通过；`panta-dsl-core` 单测、catalog、features 和 FSM 全部通过，features/FSM 各 2 个超大容量用例按登记跳过；`panta-foundation` 无测试。首次运行时 512 边容量用例在 Pest 解释执行超过两分钟仍未完成，故新增该两项跳过。`cargo ub-check` 的 rustup 安装步骤因本机沙箱禁止写 `~/.rustup` 无法执行，直接运行的是 runner 后续完全相同的 Miri 命令。 |
| 2026-09-28 | `QT_QPA_PLATFORM=offscreen QT_QUICK_CONTROLS_STYLE=Basic target/native/debug/qml/panta_qml_cpu_benchmark` | 文档页签构造、32 次切换与关闭末项分别计时 | macOS 26.3.1 arm64、Qt 6.11.2、debug；1 次预热、31 次采样，最终源码重建后 3/3 通过。8 页签构造 p50/p95=974/1321 µs，切换=326/610 µs，关闭=910/1241 µs；24 页签分别为 2829/4823、482/2484、3165/5745 µs。仅为 QML CPU 更新成本，不推断 GPU 耗时。 |
| 2026-09-28 | `PantaBenchmark.app` 临时 wrapper 启动 `PANTA_BENCHMARK_DOCUMENT_TABS_ONLY=1 QT_QUICK_CONTROLS_STYLE=Basic` 的 `panta_qml_gpu_benchmark` | 真实窗口下比较页签静态、可见页签切换、首项关闭/重开 | macOS 26.3.1 arm64、Qt 6.11.2、Metal、debug；真实 1000×700 窗口，30 帧预热，每场景 3×60 帧。8 页签静态 p50/p95=16.64/24.24 ms、切换=16.67/23.65 ms、关闭/重开=16.61/23.04 ms；24 页签分别为 16.67/23.76、16.67/22.86、16.67/17.64 ms。三项均通过，数值含垂直同步与窗口合成，不能据此称为 GPU 内核耗时或性能改进。 |
| 2026-09-28 | `cargo build --locked`、`cargo test --locked --workspace`、`cargo test --locked -p panta-dsl-core --test fsm`、`cargo format --check`、`cargo lint`、`cargo sanitize` | 完成前的聚合与 CI 失败路径复核 | build、format、lint 八阶段均通过；Cargo 聚合 CTest 62/62（含页签动画断言），普通 FSM 20/20（含容量阈值）；macOS ASan/UBSan 62/62、TSan 47/47。TSan 排除两项经 FFI 拉取 Rust 异步结果的测试，其余五个 ViewModel 用例继续执行。远端三平台 CI 尚需新提交触发确认。 |
| 2026-09-28 | GitHub Actions run [36430454561](https://github.com/Yuki-Nagori/panta/actions/runs/36430454561)，commit `ca8ad8a` | 确认此前失败后的当前远端 CI 状态 | 全部 19 个 job 成功：三平台 build/test 与 sanitizer、Miri、覆盖率、格式及 lint 均通过。该 run 是旧失败记录的后续成功结果；它不包含本次尚未 push 的本地动画测试修正。 |
| 2026-09-28 | `cargo test --locked --workspace` | 验证页签动画断言修正及完整本机聚合测试 | 通过：Rust workspace 测试通过，native CTest 62/62；`ThemeComponentTest` 动画断言通过。固定 200ms 等待改为等待 x 坐标进入 0.1px 容差，避免 sanitizer 下按帧调度精确比较失败。 |
| 2026-09-28 | `cargo sanitize` | 验证页签改动在本机 sanitizer 构建下的 native 回归 | macOS ASan/UBSan CTest 62/62、TSan CTest 47/47 均通过。TSan 按 042 边界排除 TaskHost/Ffi/Qml 与两个经 Rust mutex drain 结果的 ProjectViewModel 异步激活用例；其余五个 ProjectViewModel 测试通过。 |
| 2026-09-28 | `cargo format --check`、`cargo lint` | 格式与静态质量验收 | format 通过；lint 在沙箱 TCP 锁限制导致首次失败后，于受限环境外重跑完整 8 阶段通过。 |
| 2026-09-28 | `cargo test --locked --workspace` | 验证同名来源的标签区分、PageTab 无障碍信息及键盘激活 | 通过：CTest 62/62。新增 QML 回归检查两个同名导入页签显示不同序号、Qt accessibility interface 暴露 PageTab 和完整名称、Enter 激活就绪页签；Space 使用相同处理分支。 |
| 2026-09-28 | `cargo format --check` | 检查同名标签回归及 QML 行为修改的格式 | 通过。 |
| 2026-09-28 | `cargo lint` | 静态检查本次 QML 与 C++ 回归 | 首轮 clang-tidy 因两个测试 API 缺少直接头文件失败；补齐 Qt accessibility 与键盘测试头文件后，完整 8 阶段通过，包括 qmllint、clang-tidy、include-cleaner 和 cppcheck。 |
| 2026-09-28 | `cargo test --locked --workspace` | 验证打开文档快照跨切换保留并在关闭后释放 | 通过：CTest 63/63。新增 ProjectViewModel 回归在两个已就绪页签间重复切换 32 次，再分别关闭非活动和活动页签；弱引用确认关闭后独占 `SurfaceMeshSnapshot` 已释放。 |
| 2026-09-28 | `cargo lint` | 静态检查快照释放生命周期回归 | 完整 8 阶段通过，包含 clang-tidy、include-cleaner 与 cppcheck。 |
| 2026-09-28 | `cargo test --locked --workspace` | 验证页签焦点导航、滚动可见、键盘激活及关闭操作 | 通过：CTest 63/63。QML 回归覆盖 PageTabList / PageTab / 关闭 Button 名称与角色、左右箭头导航并将目标滚入视区、Enter 和 Space 激活、Space 关闭；真窗口读屏验收仍待完成。 |
| 2026-09-28 | `cargo lint` | 检查页签键盘交互、focus 与 accessibility QML | 完整 8 阶段通过，含 qmllint、clang-tidy、include-cleaner 与 cppcheck。 |
| 2026-09-28 | `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、`git diff HEAD --check`；临时 `PantaPreview.app` 与 CUA AX 检查 | 验证新增 `DocumentTabBar` 翻译上下文可编译到 QM，并检查真窗口无障碍树 | build、format、差异检查通过；Cargo 聚合通过，CTest 63/63；lint 完整 8 阶段通过。QM 测试按 `DocumentTabBar + source` 命中新增中英文条目；AX 树暴露 PageTabList、Welcome PageTab 和 Close Welcome Button。真窗口截图捕获报 `SCStreamError -3811`，键盘焦点停在全局搜索框，焦点视觉与系统读屏操作仍未确认；临时 app 已关闭并清理。 |
| 2026-09-28 | `cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、`git diff HEAD --check` | 验证重排、切换、关闭时活动文档、辅助技术选中态和视口快照保持一致 | 通过：CTest 64/64。新增 ViewModel 回归重排稳定文档 ID 后活动 ID 和网格快照不变，关闭非活动页签不改快照，关闭活动页签转到右邻就绪文档，全部关闭后状态为空；QML 回归确认 `Accessible.selected` 随活动文档 ID 切换。format 与差异检查通过，lint 完整 8 阶段通过。 |
| 2026-09-28 | `cargo test --locked --workspace`、`cargo format --check`、`git diff HEAD --check` | 稳定拖拽换序后的动画终态断言 | 通过：CTest 64/64；将残留的固定 200ms 等待和精确浮点比较改为等待目标 x 坐标进入 0.1px 容差。format 与差异检查通过。 |
| 2026-09-28 | `cargo test --locked --workspace`、`cargo format --check`、`git diff HEAD --check` | 验证页签固定宽度、长标题省略和键盘导航后的可见范围 | 通过：CTest 64/64；QML 回归确认 tab 宽 130px、来源长标题已截断但完整文本仍进入无障碍名称，左右键将末尾 tab 的主体滚入可视区。首末活动页签圆弧在裁切边界处的像素检查仍待补。 |
| 2026-09-28 | `cargo test --locked --workspace`、`cargo format --check`、`cargo lint`、`git diff HEAD --check` | 检查首末活动页签滚动到边缘时的圆弧与白线像素 | 通过：CTest 64/64、format、lint 完整 8 阶段及差异检查通过；首个活动页签左下连接像素保持面板白色，末个活动页签滚到最右端时保留完整 8px 圆弧连接范围，圆弧外恢复标签带灰色，底部白线连续。 |
| 2026-09-29 | macOS arm64，`cargo format`；`cargo build --locked` | 适配层完成后，根 QML 的 reduced-motion 绑定通过格式和构建 | 通过。`Settings` QML 类型可用，`Settings.reducedMotion` 编译接入 `ViewportPane`；真实窗口动画效果仍待验收。 |
| 2026-09-29 | macOS arm64：`cargo build --locked`、`cargo format --check`、`cargo test --locked --workspace`、`cargo lint`；PantaPreview.app 真窗口及 AX 检查 | 验证 reduced-motion 下换序直接定位，并检查实际 VTK Welcome、页签角色/名称和键盘焦点 | build、format、聚合测试（CTest 66/66）及 lint 八阶段通过。QML 回归验证 `reducedMotion=true` 时页签重排同步到目标位置。真窗口截图显示 VTK `panta` Welcome 场景；AX 树包含 PageTabList、Welcome PageTab 和 Close Welcome 按钮。Tab 导航仍停在全局搜索框，真实页签焦点视觉与 VoiceOver 操作尚未验证；临时 app wrapper 已清理并关闭应用。 |
| 2026-09-29 | `panta_qml_cpu_benchmark`，offscreen / Qt Quick Basic；macOS 26.3.1 arm64、Qt 6.11.2、debug | 069 CPU harness 测文档页签构造、32 次切换与关闭末项 | 通过；每场景预热后 31 次，p50/p95 µs。1 页签构造 150/224；8 页签构造 932/1204、切换 323/598、关闭 894/1267；24 页签构造 2790/4998、切换 442/2079、关闭 2932/5721。计时覆盖 QML 构造/模型更新与事件处理，不代表 GPU 帧成本。 |
| 2026-09-29 | `PANTA_BENCHMARK_DOCUMENT_TABS_ONLY=1` 的 `panta_qml_gpu_benchmark` 真窗口；macOS 26.3.1 arm64、Qt 6.11.2、Metal、debug | 069 GPU harness 测静态、切换与关闭/重开时可见页签的帧呈现 | 通过；1000×700 窗口，30 帧预热，每场景 3×60 帧；p50/p95 ms/frame。空场景 16.65/18.93；静态 1/8/24 页签分别 16.70/25.60、16.69/25.27、16.70/24.90；切换 8/24 页签为 16.73/24.35、16.75/25.38；关闭/重开 8/24 页签为 16.71/24.59、16.67/18.96。测量包含 compositor/vsync，是 Qt Quick 端到端帧间隔，不是 GPU 内核耗时；本 harness 呈现独立 DocumentTabBar，不包括 VTK 网格场景。 |
| 2026-09-29 | `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`；`PantaViewportTest.app` 真窗口运行 `PANTA_TEST_NATIVE_VIEWPORT=1 panta_qml_viewport_module_test native_refresh_and_window_lifecycle`；macOS 26.3.1 arm64、Qt 6.11.2、VTK WebGPU | 视口隐藏时关闭 STL 文档，确认 CPU 网格快照立即释放，再显示并继续渲染 | 通过：构建、format、Cargo 聚合测试（CTest 66/66）及 lint 八阶段通过；真实桌面窗口日志为 3 passed、0 failed。弱引用在视口隐藏、场景清空后过期，随后重新显示仍提交帧。该用例确认 `SurfaceMeshSnapshot` 生命周期，不单独统计 VTK actor/mapper、回调或设备资源循环，完整资源验收仍待完成。 |
| 2026-09-29 | `cargo build --locked`、`cargo test --locked --workspace`、`cargo format --check`、`cargo lint`；PantaPreview.app 真窗口与 macOS AX 操作 | 验证 PageTab 焦点视觉、辅助技术激活与关闭 | 通过：构建、format、Cargo 聚合测试（CTest 66/66）及 lint 八阶段通过。真实窗口 AX 树暴露 PageTabList / Welcome PageTab / Close Welcome；AX 激活 Welcome 后焦点移到 PageTab，截图可见焦点框；AX 激活关闭按钮后该标签从树中移除。按 Tab 从全局搜索框没有移动焦点；本次未启动 VoiceOver，语音朗读与该焦点进入路径仍待验。 |

## 风险与回退

主要风险是为每个标签复制 native 场景导致 GPU / CPU 内存乘法增长，或异步结果越过工程切换覆盖当前视口。优先维持单 `CaeViewport`，让 Rust 以工程代次、ImportRecord ID 和 attempt 校验结果；性能基准决定打开标签快照缓存上限。若缓存释放或切换期间 VTK 生命周期不稳定，退回到单活动网格并在已打开但未驻留的页签重新加载，保留标签去重和用户关闭语义。

## 决策与工作记录

- 2026-09-24：按用户要求以浏览器标签替换旧 `Model / Mesh / Results` 视口条；Welcome 展示现有 panta 默认场景并有关闭按钮；全标签关闭后视口保持空白，STL 页签根据稳定导入记录打开、复用和关闭。
- 2026-09-24：复核当前实现发现 Rust ProjectService 仅在内存持有最新网格，工程记录已有稳定 `ImportRecord.id` 与相对资产引用；首次激活其他已保存 STL 是真实读取/解析操作，拟作为 073 的首个异步 FSM 消费者。标签点击/关闭本身不进入 FSM。
- 2026-09-24：本任务只登记 HTML 原型和实现边界，不修改 QML 产品代码；打开标签的快照缓存上限需由真实 STL 基准决定。
- 2026-09-25：完成 HTML 参考的标签切换、关闭、固定宽度与水平拖动跟手和邻项动画；普通点击不捕获指针，真实拖动才由稳定列表捕获，拖动开始时激活目标标签。QML、ViewModel、FSM 和资源生命周期仍待实施。
- 2026-09-26：审计 063/068/073 现状后冻结契约（实施前置）。①相关性 DTO：ProjectService 新增会话级 `generation`（create/open 各递增，不持久化、不等于 revision）；`attempt` 为进程内单调 u64；信封为 `(generation, attempt, ImportRecord.id)`。②流程图 `open-saved-stl`：`Idle → LoadingAsset → Parsing → Ready`，终态 `Failed / Cancelled / Expired`；guard `record-valid` 在 begin 提交边界求值（拒绝时同步返回错误、不建 attempt、不落 Failed 终态），guard `session-current` 在完成边界求值；取消检查点为分块读取阶段与解析完成边界，`parse_stl` 本身不可中断（073 设计允许），边界 guard 拒绝即释放过期快照。③begin/去重：同 `(generation, record)` 的在飞 attempt 直接复用返回同一 id；终态后可重新 begin；generation 变更时协调器立即对在飞 attempt 投递 `generation-invalidated`，drain 再按当前 generation 过滤兜底，过期/迟到快照在 Rust 侧释放、不跨 FFI。④UI 语义：页签状态 `Loading / Ready / Failed`；点击未打开记录立即建 Loading 页签但不激活，成功后激活；失败保留 Failed 页签与上一个可见视口；Loading/Failed 页签不可被激活（拖拽仅重排），活动文档恒为 Ready；关闭 Loading 页签取消 attempt；关闭全部页签后视口空白（隐藏 welcome 字样，复用 `RenderScene.primitive_visible` 区分 Welcome 与空白）。⑤快照保留基线：所有打开 Ready 页签在 ViewModel 保留 CPU 快照（上限=打开页签数），单一 `CaeViewport`/VTK actor 切换；069 基准测量内存高水位与切换延迟后再评估预算逐出。⑥`open()` 不再同步重载最新导入网格（消除同步/异步双路径），`current_mesh` 仅由会话内 `import_stl` 产出，重开工程后视口为 Welcome。⑦ViewModel 是活动文档 ID 与视口快照唯一权威；工程树选中行为活动文档投影，Part 检查器标题跟随活动导入页签（无活动时回退最新记录，保持 068 其余语义）。
- 2026-09-28：复核当前分支及 CI，追加本次收敛范围：修正 Ubuntu 像素测试误取文字颜色、对照 HTML 调整活动标签凹弧的几何和白线交界；审查并修复文档激活结果的 attempt 相关性与失败收敛、拖拽跟手等实际缺陷，精简偏离注释规范的逐句或不准确说明。所有修复以同一任务的行为验收和 Cargo 聚合入口为准。
- 2026-09-28：用户反馈释放后页签未归位、右侧圆弧错位、左侧抗锯齿接缝、Welcome 图标文字颜色，以及悬停误触发拖动。拖动改为稳定委托与槽位坐标动画，释放/取消清理按下身份，移动期间校验左键状态；右侧圆弧改为反向 `PathArc`，左右连接处与主体重叠 1px。Theme 的重复 12px 图标尺寸合并，删除无效说明性注释。
- 2026-09-28：用户反馈拖动释放后光标未恢复，光标形状现由当前 `MouseArea.pressed` 与当前页签拖动状态共同决定；QTest 覆盖悬停、左键拖动与释放后的形状。用户又发现 `.panta` 重开后双击 STL 不显示：排查出 `CaeViewport::refresh_mesh()` 把 Welcome 占位开关应用于网格 actor，已改为“有网格即显示，只有无网格时才看占位开关”。临时 `.panta` 在真窗口重开并双击 STL 后，网格实际显示。
- 2026-09-28：用户反馈活动页签的关闭 X 在选中时丢失底色。按 HTML 参考区分关闭按钮悬停与按下态，确保活动和非活动页签均显示背景反馈；补 QML 鼠标事件回归，并重跑适用的 Cargo 入口。
- 2026-09-28：用户进一步提供 `test_4 / mug.stl` 的重开场景，三角形 STL 的通过结果不足以证明该模型可显示。追加对真实模型的加载终态、页签激活及视口画面核查；如当前主机的 Documents 隐私权限阻止直接读文件，改由应用文件选择器复现并记录具体限制。
- 2026-09-28：用户确认复现条件是先关闭 Welcome 再双击工程树中的 STL。真窗口核查表明资产激活成功、活动页签已切到 `mug.stl`，但 VTK 画面仍空白。空白态会先隐藏 actor；恢复网格时将 actor 可见性更新前移到相机裁剪范围计算之前，避免按隐藏场景重设裁剪面。补 ViewModel 从全部页签关闭后的重新激活回归；用户随后确认该场景已修复。诊断日志仅临时用于定位，不保留在提交中。
- 2026-09-28：用户要求对当前分支相对 `main` 的完整差异做代码与注释评审。复核功能实现、测试、资源和任务记录的职责与一致性，删除死代码、临时诊断与逐句解释型注释；只修正与 080 行为相关且有证据的问题，再以 Cargo 聚合入口重新验收。
- 2026-09-28：完整差异评审清理了 ViewModel 未实现声明与空转信号封装，统一激活失败码解析；页签模型同步按 ID 建索引以保持委托身份并避免逐项线性查找；移除历史性注释。审计时发现同名页签可见消歧、性能高水位、反复切换资源释放、键盘可访问性、系统减少动态效果接线及手动基准尚无完整证据，将相关验收项恢复为未完成，不以本地构建通过代替这些结论。
- 2026-09-28：提交前复核分支完整差异和暂存边界；修正文档对 Loading 页签建立时机及 ViewModel 职责的不准确描述，补活动页签关闭按钮的悬停/按下回归，并将资产丢失测试改为验证具体用户消息。其余待验收项目保持 `in-progress`。
- 2026-09-28：最新远端 CI run 36430454561（commit `ca8ad8a`）19 个 job 全部成功，Miri、三平台 sanitizer、build/test、格式、覆盖率和 lint 均通过；较早的 run 36265495455 与 36402570741 失败记录由此更新为已恢复。该 run 不含未 push 的本地修正。
- 2026-09-28：将 `ThemeComponentTest` 的页签动画位置断言改为等待目标位置进入 0.1px 容差，删除固定 200ms sleep；完整 Cargo 聚合、ASan/UBSan、TSan、format 和 lint 本机通过。TSan 与 Miri 的平台边界和排除范围保留在当前 042 runner 契约中。
- 2026-09-28：同名 STL 页签在显示标题前加当前同名页签序号，并通过 PageTab accessibility name 暴露完整来源名与序号；就绪页签响应 Enter / Space 与辅助技术 press action。新增 QML 回归确认可见消歧、无障碍角色/名称及 Enter 激活，Cargo 聚合通过。
- 2026-09-28：补齐无障碍 QML 测试需要的 Qt 头文件后，`cargo lint` 完整 8 阶段通过，包含 clang-tidy、include-cleaner 和 cppcheck。
- 2026-09-28：新增 ViewModel 快照所有权回归，验证两个就绪文档反复切换 32 次后仍保留各自快照，关闭页签后其独占 `SurfaceMeshSnapshot` 释放；Cargo 聚合 CTest 63/63。真实 VTK 窗口中的 actor/mapper 与设备资源循环释放仍需验收。
- 2026-09-28：完善标签栏键盘交互和关闭控件可访问语义；左右键改变焦点并自动滚动到可视范围，Enter / Space 激活就绪页签，关闭按钮响应 Space 和辅助技术 press action。Cargo 聚合 CTest 63/63；真窗口读屏与焦点验收待完成。
- 2026-09-28：页签键盘与可访问性改动通过完整 `cargo lint` 8 阶段检查。
- 2026-09-28：在 `panta-en.pa` 与 `panta-cn.pa` 补充 `DocumentTabBar` 的页签名称、关闭、重复来源名和加载状态条目；扩展 QM 加载测试按 `DocumentTabBar + source` 验证中英文目录。Cargo 聚合 CTest 63/63，构建刷新后的 `panta_zh_CN.ts` 含对应上下文与译文。真窗口 AX 树已见 PageTabList / PageTab / 关闭按钮，但系统截图服务报错且键盘焦点未能进入页签，相关人工验收继续保持未完成。
- 2026-09-28：新增 ViewModel 文档重排/关闭行为测试和 QML PageTab 选中态测试；重排后活动 ID 与快照稳定，关闭非活动标签不变，关闭活动标签选右邻就绪标签，切换活动 ID 同步更新 accessible selected。Cargo 聚合 CTest 64/64、format 和 lint 八阶段通过。
- 2026-09-28：拖拽换序动画断言移除固定 200ms 等待和精确浮点比较，改为等待 0.1px 容差；Cargo 聚合 CTest 64/64、format 和差异检查通过。
- 2026-09-28：补页签宽度、长标题省略和末尾标签键盘滚动可见回归；完整来源仍保留在 PageTab/关闭按钮无障碍名称中。Cargo 聚合 CTest 64/64、format 和差异检查通过；圆弧贴近裁切边界仍需专项像素验收。
- 2026-09-28：页签末端滚动范围增加 8px 圆弧余量，并新增首尾活动页签连接处的像素回归；Cargo 聚合 CTest 64/64、format 与 lint 完整 8 阶段通过，固定宽度、长标题、省略、2px 首项位置及边缘圆弧布局验收完成。
- 2026-09-28：Qt 6.11 的 `QStyleHints` / `QAccessibilityHints` 未提供 reduced-motion 属性；采用平台适配：macOS `NSWorkspace.accessibilityDisplayShouldReduceMotion` 与选项变更通知，Windows `SPI_GETCLIENTAREAANIMATION` 与 `WM_SETTINGCHANGE`，Linux XDG Desktop Portal Settings v2 的 `org.freedesktop.appearance/reduced-motion` 与 `SettingChanged`。值未知或接口不可用时按无减少动态效果偏好处理，由 Bridge `Settings.reducedMotion` 注入视口页签。
- 2026-09-29：按维护者要求先建立独立 Qt 平台服务适配层，再继续页签动画接线。系统平台查询及标准目录发现由 086 收拢；086 已完成。根 QML 现使用 `Settings` 并将 `Settings.reducedMotion` 注入 ViewportPane 的页签动画。
- 2026-09-29：补充 reduced-motion 回归，验证开启时页签换序同步定位；Cargo 聚合 CTest 66/66、format 与 lint 八阶段通过。真窗口 CUA 截图和 AX 树确认 Welcome VTK 场景及 PageTab 语义；Tab 后焦点仍位于全局搜索，焦点视觉/VoiceOver 验收继续待办。
- 2026-09-29：排查关闭活动 STL 页签的 VTK 所有权时发现，视口隐藏期间 `sync_native_surface()` 提前返回，旧 actor mapper 与 `applied_mesh` 会一直保留到视口重新显示。隐藏分支现在同步替换 actor 管线并释放旧 CPU 快照；真实桌面窗口回归验证弱引用在隐藏状态下即过期，视口恢复后仍可继续提交帧。另修正测试对快照的局部强引用并补直接头文件。VTK actor/mapper、回调和设备资源的重复切换/关闭验收仍未完成。
- 2026-09-29：真窗口检查发现鼠标/辅助技术激活文档标签后，焦点仍留在全局搜索框。`DocumentTabBar` 现在在标签按下和 PageTab 辅助技术 press action 时显式转移焦点；QML 回归验证鼠标点击后 PageTab 获得 active focus。更新后的 PantaPreview 真窗口通过 AX 激活 Welcome 后焦点树指向 PageTab，截图显示焦点边框；AX 关闭动作从无障碍树移除 Welcome。按 Tab 从搜索框仍未进入标签栏；实际 VoiceOver 朗读及该键盘进入路径继续待验。

## 完成摘要

文档页签核心行为及此前 CI 回归已通过本机聚合、ASan/UBSan、TSan 与远端 run 36430454561（commit `ca8ad8a`）。动画测试改为容差等待；重复来源名已有可见与辅助技术消歧；ViewModel 活动快照切换/重排/关闭、键盘交互和 `DocumentTabBar` 中英文 QM 条目已有回归覆盖。086 已提供 Qt adapter API，任务保持 in-progress；本次修复并在真窗口验证隐藏视口关闭网格后 CPU 快照释放，也验证 AX 激活标签的焦点框与关闭动作。剩余 VoiceOver 朗读和从搜索框进入标签栏的焦点路径、VTK actor/mapper/回调/设备资源循环释放、代表性 STL 内存高水位/重载延迟、reduced-motion 动画效果验收，以及本地改动的后续远端 CI。
