# 063 — STL 导入、导入选项持久化与工程工作区

- 状态：done
- 阶段：应用平台扩展
- 依赖：[057](057-new-project-dialog.md)、[060](060-qml-project-workspace-reference.md)、[062](062-ribbon-tab-components.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-22 / 2026-10-13

## 目标与背景

将 Home 工具栏的 Import 从视觉入口推进为第一条真实导入流程：选择一个或多个 `.stl` 文件，随后显示网格类型、单位、导入日志和近似尺寸确认窗口；确认后将导入资产、来源信息及影响几何解释的选项纳入当前 `.panta` 工程，并在工程树中展示多个独立零件。左侧工程工作区上下分区：上方展示工程与导入文件，下方展示当前选择零件的任务和属性；右侧视口显示所选 STL，后续再扩展其他格式。

2026-10-16，任务 088 将本任务原有的工程内 STL 副本存储迁移为 `SurfaceMesh` sidecar。当前 schema 4 不保留原始 STL 副本；来源快照保留导入时所选路径、文件名、格式、SHA-256、mtime 和 size，仅用于追溯与诊断。下方 `assets/imports/` 与 schema 2/3 的说明记录 063 实施时的历史行为及兼容迁移输入，不代表当前新导入行为。

本任务先完成可验收的 HTML 状态参考和持久化边界设计，再分阶段实现 QML、C++ ViewModel、Rust 导入服务与视口数据流。HTML 参考覆盖新建项目弹窗。首期 STL 运行时闭环、结构化导入 / 工程诊断、后台导入事务、`display_name` 重名策略、四档缩放行为验证与 Metal 硬件离屏性能基准均已完成；实际网格生成不在本任务范围。

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

- 独立的用户可见 `display_name`，与来源文件名、资产 ID 分离；默认由来源文件名去除已识别扩展名得到。名称冲突规则在实现前确认。
- 来源快照记录导入时所选路径、原文件名、格式、内容摘要（SHA-256）、mtime 和 size；它们仅用于追溯与诊断，不能成为工程重开或加载 Mesh 的依赖，也不代表工程内保留源文件副本。
- `meshType`：`midplane`、`dual-domain`、`solid-3d`；这是影响后续网格解释的工程数据。
- `units`：首期支持界面中的明确单位选项，不能把“未指定”静默当成某个单位。
- `showImportLog`：记录用户对该次导入的日志可见性选择，但它只是展示偏好，不替代导入诊断和结果状态。
- 导入记录版本、STL 解析器版本、导入记录 ID，以及后续用于尺寸、拓扑和重建结果的稳定引用。

原始 STL 不应被修改；近似尺寸由解析结果计算并在确认窗只读展示。取消或导入失败不能写入工程 manifest、替换当前工程或清空已有视口。确认后的导入记录采用追加 / 版本化语义，避免改写其他已导入资产。当前 schema 与复制行为记录在本任务验收结果中；不保留源副本并改用 Mesh sidecar 的最终存储由 088 负责。

063 实施期间曾将原始 STL 复制到 `assets/imports/`，manifest 保存 import record；这不是解析后的 `SurfaceMesh` sidecar。当前 schema 4 由 [任务 088](088-mesh-asset-sidecar-storage.md) 管理 `.panta` Mesh 索引及按页签加载，迁移保留 `display_name` 和来源快照并移除工程内 STL 副本。063 不实现 Mesh sidecar，也不把 VTK 快照持久化。

### 资产显示名与来源

来源文件、工程内资产和运行期数据是三个相关但不同的概念。063 最初复制并保存原始 STL，激活时由 `panta-mesh` 解析为内存 `SurfaceMesh`；当前 schema 4 已由 088 改为持久化 Mesh sidecar。历史 STL 副本只有 schema 2/3 迁移时会从工程包读取，成功提交 sidecar 索引后再清理；导入时用户选定的外部 STL 不再复制或作为运行依赖。

导入记录区分用户看到的 `display_name` 与来源快照：STL 的默认显示名取文件名主体（例如 `model_a.stl` 显示为 `model_a`）；`source_name` 保留原始文件名及扩展名，格式、路径、SHA-256、mtime 和 size 仅用于来源诊断。当前 `ImportRecord.id` 标识导入记录；Mesh sidecar 的 `asset_id` / `revision` 标识 Mesh 资产。显示名和源文件名都不是身份键。088 迁移保留 063 建立的命名契约和来源快照，并将 Mesh 索引从导入记录中拆开。

显示名允许重复。记录持久化相同的基础 `display_name`，工程树和文档页签按当前导入顺序显示序号以消除歧义；来源名与导入记录 ID 不因显示冲突而改写。

## 范围与非目标

包含：

- 首期文件选择器只接受 `.stl`，格式过滤和导入服务接口为可扩展集合，新增格式不复制一套流程。
- 导入确认窗口：Import 标题、网格类型选择、单位选择、近似尺寸、Show import log、OK / Cancel / Help；初始视觉状态参考用户提供的图一。
- 导入成功后的任务树与右侧 STL 视口状态，参考用户提供的图二；任务状态、错误、日志和后续 Create Mesh 入口先建立展示边界。
- 工程工作区左栏有两个独立 Dock：上部工程 / 任务 Dock 内部分为上下两区，顶部工程树展示多个导入 study，中间显示当前 study 的任务；底部保留独立的 Layers Dock、截图中的工具按钮和带 Layers 图标的单一页签，内容先留空，具体图层行为待后续确认。
- `.panta` 工程打开后可再次导入一个或多个 STL；每个文件作为工程下独立零件加入，不覆盖已有零件，文件选择器允许多选。
- `.panta` 内的 import record、源资产引用、失败回滚和重新打开恢复；QML 只发语义请求，C++ ViewModel 负责 Qt 适配，Rust 服务负责解析、校验、持久化和导入 DTO。
- 将显示名与来源快照分开：STL 默认显示名不带 `.stl`，来源名保留原扩展名；路径、格式、SHA-256、mtime 和 size 仅用于追溯诊断，不作为稳定资产身份或 Mesh 加载依赖。
- 新建项目弹窗 HTML 参考，补齐 homepage 之前遗漏的 Create New Project 状态。
- 独立 HTML 状态参考：`open-project` 中点击 Import 后的导入弹窗，以及导入完成后的 `imported-project` 页面。

不包含：

- 不实现 Midplane、Dual Domain、Solid 3D 的实际网格生成，不把 Create Mesh 等任务伪装成已完成；首期只把 STL 三角面送入现有 VTK 视口。
- 不实现 STL 修复、单位推断、其他 CAD 格式、导入日志编辑、材料和注射位置业务。
- 不把外部绝对路径、临时缓存或 UI 截图写入工程作为长期契约；不新建 target 或切换工具链。

## 实施步骤

1. 登记任务，补 homepage 新建项目弹窗、open-project 导入确认弹窗和 imported-project 工作区 HTML 参考。
2. 依据 HTML 锁定导入状态模型、manifest import record、资产引用和错误回滚边界。
3. 增加可扩展文件选择 / 导入请求 DTO，首期只实现 STL；解析器返回单位、尺寸、拓扑摘要和稳定资产引用。
4. 通过 C++ ViewModel 把导入确认、取消、日志和状态变更接到 Rust 服务；成功后追加工程记录并发布 Tasks / viewport 快照。
5. 增加 `.stl` 过滤、选项往返、失败不污染当前工程、重开恢复、版本不兼容和视口生命周期测试。
6. 按实际实现更新模块、manifest 规范、任务验证结果和索引状态；不要提前标记未落地的网格 / 渲染能力。

### 本轮范围（2026-10-13）

- 将确认导入的源文件复核、工程资产写入和 manifest 提交移入 Rust 有界后台写入执行器；GUI 线程只发起请求、轮询回执和应用成功快照。
- 明确提交截止点：后台事务到 manifest 原子替换前可取消，取消 / 失败不得留下可见资产或修改工程；原子替换成功后视为已提交，即使窗口关闭或宿主销毁也必须完成收尾并记录结果。
- 同一工程同一时间只接受一个导入写入；开始、完成、取消、容量拒绝和 worker 断开都需稳定请求关联与诊断，不得让迟到回执污染后来打开的工程。
- 保留现有同步领域入口仅供无 GUI 测试 / 调用方使用时，需由本轮验证确认是否仍有实际消费者；不保留没有消费者的重复路径。
- 当时的异步写入批次不包含 `display_name` 重名策略和四档缩放验证；后续批次已补齐这两项。实际网格生成仍不属于本任务范围。

### 本轮范围（2026-10-13，显示名）

- 在 ImportRecord 持久化 `display_name`，STL 默认使用移除 `.stl` 扩展名后的文件名主体；`source_name` 仍保留原始名称。当前只接受 STL，格式识别依据由 Rust 导入领域层负责。
- 新导入记录保存源字节快照的 SHA-256（`sha256:<lowercase hex>`）；旧 schema 2 记录迁移时摘要保留缺失，打开清单不额外读取 STL 资产。
- 工程树与文档页签显示 `display_name`；重复名称保留相同基础名，并按完整导入记录顺序加序号。页签序号不依赖打开 / 激活顺序，关闭其他同名页签后仍保持稳定，不写回领域名。Layers Dock 当前没有资产行。
- 通过 schema / import record 版本化迁移现有 schema 2 记录：从 `source_name` 派生显示名，保存时写入新版本。schema 1 无导入记录的打开行为保持不变；未知版本仍明确拒绝。
- 增加文件名大小写、Unicode、缺失扩展名 / 非预期扩展名、重复名称、逆序打开 / 关闭后页签序号稳定性、schema 1/2/3 无效导入版本拒绝且不替换当前工程，以及旧 schema 迁移回归；不涉及 Mesh sidecar（088）或四档缩放性能基准。

## 验收标准

- [x] task、持久化决策和首期 HTML 状态参考已登记；homepage 新建项目弹窗已补齐。
- [x] HTML 能表达 `.stl` 选择后的 Import 确认窗口，以及确认后的任务树和 STL 视口占位。
- [x] HTML 展示左栏两个 Dock：工程 / 任务 Dock 内含工程树和所选 study 任务两个分区；导入后底部显示带工具按钮和 Layers 图标页签、内容留空的独立 Dock。
- [x] HTML 展示同一 `.panta` 工程含多个 STL 零件，Import 文件选择支持多选并表达“追加到当前工程”；空工程保留 Layers Dock 外框与工具栏，未导入时隐藏 Layers 页签行。
- [x] 首期文件选择只接受 `.stl`，且格式集合可在不复制导入流程的情况下扩展。
- [x] 取消 / 失败不修改工程；确认后 `.panta` 内有可重放 import record 和源资产引用。
- [x] 重新打开工程可恢复导入记录、任务树和 STL 视口；版本 / 单位 / 尺寸错误可诊断。
- [x] Import record 和 Tasks / 文档页签显示独立的 `display_name`；默认从来源文件名去除已识别的格式扩展名，保留原始 `source_name`、格式、工程内来源资产引用及 SHA-256 内容摘要。同名记录保留同一基础显示名，界面按导入顺序加序号，并有回归测试。Layers Dock 当前没有资产行，内容仍按本任务既定范围留空。
- [x] 导入及工程服务错误通过结构化 DTO 跨 CXX 传递稳定错误码、类别和必要上下文；C++ 适配 Qt 展示字段与本地化消息，QML 不解析拼接错误文本。
- [x] STL 预检 / 写入导入迁为 Rust 后台任务，定义取消与提交截止点；GUI 线程不执行文件读取和格式解析。
- [x] QML / C++ / Rust 分层、静态检查、native 测试、四档缩放行为与 Metal 硬件离屏性能基准、真实窗口及视口生命周期验证通过。

## 清理与兼容例外

- `COMPAT(task 063; remove-task 088)`: 读取当前受支持的 schema 2 工程时，将旧 ImportRecord 的 `display_name` 从 `source_name` 派生并迁移到当前 schema 3；schema 1 继续按无 imports 打开。迁移发生在 Rust 工程读取边界，保存只写 schema 3 / ImportRecord v2；未知版本明确拒绝。该读取路径用于保留用户已有工程，088 切换到 Mesh 资产索引时复核并清除过期 schema 支持；验证覆盖 schema 1、2、3 的打开及保存往返。

## 验证计划与结果

### 结构化诊断批次（提交 2f6260c）

验证平台为 macOS，cwd 为仓库根目录，沿用仓库锁定工具链与现有 target：

- `cargo test --locked --workspace`：通过 Rust 工作区测试、qmllint 与 70/70 个 native / QML CTest。新增覆盖全部 21 类工程诊断、八种结果 DTO、Unicode / 冒号 / Windows 路径上下文、失败占位值隔离、全部资产激活错误类别与 STL 解析错误、四类过期元数据请求、资产复制冲突、失败回滚、异步失败及修复后诊断清空。
- `cargo build --locked`、`cargo format --check`、`cargo lint --check`：通过；质量入口覆盖格式、Clippy、QML、CMake、clang-tidy、include-cleaner、cppcheck。
- `cargo sanitize`：通过 native ASan/UBSan 70/70 与 TSan 53/53 个 CTest；记录分别位于 `target/native/debug-sanitizer-address`、`debug-sanitizer-thread`。
- 跨平台 CI：[运行 36994873700](https://github.com/Yuki-Nagori/panta/actions/runs/36994873700)，对应提交 `2f6260c15d4d5ffccaad599d24cee631cb40be44`，结论 success。macOS / Linux / Windows 的 Cargo 检查、构建、聚合测试及 sanitizer 均通过；Rust coverage、native C++ coverage / QML 测试、格式、Clippy、qmllint、CMake、clang-tidy、include-cleaner、cppcheck、machete 与 Miri 通过。cargo audit 因路径过滤跳过，不记为已执行通过。
- 真实 Cocoa / GPU 窗口：使用隔离临时工程，确认新建、STL 文件选择、1 × 1 × 0 mm 预览、三角面显示、取消不改动原页签 / 视口，以及关闭后从工程树重新激活并恢复显示。截图及 AX 证据位于忽略目录 `artifacts/task-063/import-cancel.*`、`reactivated.*`；临时 app wrapper 验收后删除。

本批不改 manifest schema、UI 样式或 STL 几何算法。已删除 `ProjectViewModel` 对 `rust::Error` 的冒号文本解析；保存资产激活的失败页签另有 code / category / detail，成功或重试时清空。

该批次当时尚未覆盖四档缩放；后续已按最终壳层实现补测。该提交中的 `inspectStl` / `importStl` 当时仍同步执行文件 I/O 与格式解析，之后的异步预检 / 写入批次已迁移到后台执行器。

### 只读预检批次（写入后台化前置批次）

Rust 后台预检、Qt 非阻塞轮询与弹窗取消已实现；确认导入的写入事务仍同步。评审补充属性 / 错误通知中替换或取消请求的重入回归，完成事件仅发布仍有效的请求；整理快照取消契约、测试断言与 ViewModel 声明，同步 Qt 交互模块中的预检回执说明。`cargo test --locked --workspace` 通过 Rust 工作区测试、qmllint 与 72/72 个 native / QML CTest，包含取消 / 替换 / 工程切换、快照来源变化、桥接失败占位值及通知重入回归。`cargo build --locked`、`cargo format --check`、`cargo lint --check` 均通过。真实 Cocoa 窗口通过 STL 文件选择、1 × 1 × 0 mm 尺寸预览与 OK 启用、取消后空工程 / Welcome 不变，以及重开后文件 / 尺寸清空、OK 禁用；取消后的 manifest imports 仍为空。截图与 AX 证据位于 `artifacts/task-063/async-preview.*`、`async-cancel.*`、`async-reopen.*`，临时 app wrapper 已退出并删除。`cargo sanitize` 通过 ASan/UBSan 72/72、TSan 55/55 个 CTest（包括新增预检取消与通知重入回归）；本批未测四档缩放及性能基准，也未将小夹具窗口验收视为大文件性能证据。

### 确认写入异步批次（2026-10-13）

`ProjectService` 将源文件复核、资产复制和 manifest 原子替换放入 109 提供的有界写入执行器。取消在替换前通过共享原子状态裁决；进入提交门槛后拒绝取消并继续消费回执。pending 期间阻止同一服务的其他工程写入；执行器拒绝时恢复可重试的预览会话，失败时回滚未提交资产。`ProjectViewModel` 只启动请求、轮询回执和发布已提交快照，`ImportDialog` 展示进行状态并在成功后关闭；新增中英文状态译文。同步导入入口保留为非 QML 方法，当前仅供原有测试调用。

- `cargo test --locked --workspace`：Rust 工作区通过；native / QML CTest 74/74 通过，包含异步提交、取消、容量拒绝恢复、Qt ViewModel 后台完成，以及旧完成回执不得关闭新 Import 弹窗的回归。
- `cargo build --locked`、`cargo format --check`、`cargo lint --check`：通过；lint 覆盖 Rust、CMake/native、qmllint、clang-tidy、include-cleaner 与 cppcheck。
- 真实 Cocoa 窗口：通过创建隔离工程、打开 Import、选择 STL 并完成导入；任务列表和文档页签显示导入记录，视口显示网格。日志未见 QML 或导入错误；GPU 设备销毁信息发生在退出时。
- `git diff --check HEAD`：通过。独立 gpt-6.1-sol medium review 未发现阻断问题；发现的 P2（旧请求完成回执关闭新弹窗）已通过请求编号关联修复，并由 shell 回归覆盖。
- 该批次当时未运行 sanitizer 或三平台 CI，也未完成四档缩放和 `display_name` 持久化 / 展示；后两项在之后的批次验收完成。

评审指出异步文件写入失败、接收通道断开、服务 teardown 和提交截止点竞争尚无确定性专用回归；当前测试覆盖正常提交、取消、容量拒绝及弹窗旧回执隔离。以上为后续补充测试的边界，不作为已验证路径。

### 显示名批次（2026-10-13）

将导入记录显示名从来源文件名拆分并持久化为 schema 3 / ImportRecord v2；导入时记录源字节 SHA-256，旧 schema 2 迁移时摘要缺省。工程树和文档页签显示去除 `.stl` 后的名称；同名页签序号按完整导入顺序投影，逆序打开或关闭其他同名页签不会改变序号。迁移拒绝未知 ImportRecord 版本及不符合 schema 约束的记录，并确保失败不替换当前工程。

- `cargo test --locked --workspace`：通过 Rust 工作区测试、qmllint 与 75/75 个 native / QML CTest；新增覆盖逆序打开及关闭后同名页签序号稳定、schema 1 非空 imports 和 schema 2/3 不支持记录的拒绝及当前工程状态保留。
- `cargo build --locked`、`cargo format --check`、`cargo lint --check`：通过。lint 覆盖 Clippy、依赖检查、CMake、qmllint、clang-tidy、include-cleaner 与 cppcheck。
- 独立 `gpt-6.1-sol` medium review 指出页签序号曾基于当前打开文档，且缺少若干 schema 拒绝回归；修复为全量导入顺序投影并补齐回归。修复后复审确认两项闭环，无新增 actionable findings。
- 该批次当时未运行 sanitizer、跨平台 CI、真实窗口或四档缩放；这些未被记作该批次的通过项，真实窗口和四档缩放随后分别记录在本节相应验证项中。

### 四档缩放与最终收尾（2026-10-13）

- macOS arm64、macOS 26.3.1、Qt 6.11.2；分别设置 `QT_QPA_PLATFORM=offscreen` 和 `QT_SCALE_FACTOR=1 / 1.25 / 1.5 / 2` 运行 `target/native/debug/app/panta_qml_shell_module_test`：每档 15/15 通过，覆盖 Shell、工程工作区、导入预检 / 取消及页签状态。无头运行会跳过 WebGPU 原生 surface，不作为图形生命周期证据。
- 四档硬件离屏性能：Apple M4、macOS 26.3.1、arm64、Qt 6.11.2；逻辑视口 1280 × 800，Metal hidden hardware surface，native Debug 构建。由 `panta_performance_fixture` 创建合成工程，再将第二个导入资产替换为 500,000 三角形的二进制 STL（25,000,084 字节），并同步更新该临时 schema 2 清单的三角形数与尺寸；较小的 10,000 三角形模型保留作对照。每个模型先各完成一次初始激活与渲染，没有独立的预热循环；之后每模型采集 31 次页签切换样本。下表记录 500k 模型的场景切换至 VTK 队列完成时间；右列为独立 VTK 渲染提交 / 队列完成时间（均为 p50/p95，毫秒）：

  | `QT_SCALE_FACTOR` | 场景切换 → 队列完成 | VTK 提交 | VTK 队列完成 |
  | --- | ---: | ---: | ---: |
  | 1 | 174.305 / 184.414 | 66.490 / 71.320 | 72.049 / 78.880 |
  | 1.25 | 175.036 / 190.323 | 66.067 / 68.452 | 72.665 / 75.864 |
  | 1.5 | 180.149 / 186.702 | 68.483 / 72.005 | 76.909 / 79.808 |
  | 2 | 179.636 / 181.954 | 66.273 / 67.780 | 77.141 / 78.791 |

  命令为 `QT_QPA_PLATFORM=cocoa QT_SCALE_FACTOR=<scale> target/native/debug/performance/panta_project_stl_viewport_gpu_benchmark /Users/yuki/eit/panta/target/performance/task-063-fixture/Benchmark/Benchmark.panta 31`；四档均正常结束，完整生成工程保留在忽略目录 `target/performance/task-063-fixture/`。此基准测量已持久化 STL 的激活 / 视口更新，不测导入耗时；使用 Metal 硬件离屏 surface，不是 Cocoa 可见窗口测量；时间不含 GPU 内核时间或显示呈现时间，任务没有规定性能阈值，因此记录实测数据，不宣称达成某个未定义门槛。
- Cocoa 真实窗口导入、任务 / 页签与 STL 视口显示，以及关闭后重新激活恢复的证据见上方“结构化诊断批次（提交 2f6260c）”；确认写入异步批次另有导入成功后的列表、页签及视口证据。视口资源销毁与生命周期由此前真实窗口验收及既有 007 / 064 / 080 测试覆盖。
- 本轮尝试重新启动可见 Cocoa 窗口时，当前图形会话返回 `Cannot create window: no screens available`，没有把此次启动尝试记为通过；使用上述此前已记录的 Cocoa 实际窗口证据验收可见窗口路径。四档 Shell 定向测试、四档 Metal 硬件离屏性能基准及 `git diff --check HEAD` 通过。`cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check` 已在显示名批次通过。

## 风险与工作记录

- 2026-10-13：按维护者确认，重复导入保留相同 `display_name`，工程树与文档页签按导入顺序加序号；`source_name` 与 ID 不改写。当前导入仍复制 STL 并在激活时解析为内存 `SurfaceMesh`；Mesh sidecar 属于 088。OCCT 几何修复与 Netgen 网格优化边界同步澄清，具体几何修复流程不纳入 063。
- 2026-10-13：显示名实现评审发现页签序号受打开顺序影响；改为依据完整导入记录顺序计算，并补充关闭后稳定、逆序打开以及旧 schema 无效记录拒绝回归。macOS `cargo test --locked --workspace`、`cargo build --locked`、`cargo format --check`、`cargo lint --check` 全部通过；75/75 个 native / QML CTest 通过。复审无新增 findings；未运行 sanitizer、跨平台 CI、真实窗口或四档缩放基准。
- 2026-10-13：显示名提交 `a2d7716` 后，四档缩放在 Qt 6.11.2 / macOS 26.3.1 下分别运行 `panta_qml_shell_module_test`，1、1.25、1.5、2 倍均 15/15 通过。四档 Metal 硬件离屏性能基准以 500k 三角形 STL 各测 31 次并正常结束，实测数据及计时边界见上方；可见 Cocoa 窗口重启因当前会话无可用屏幕未能复测，既有真实窗口证据见结构化诊断批次。无头 Shell 结果不描述为 GPU 通过。
- 2026-10-13：063 收尾的独立 `gpt-6.1-sol` medium review 核实四档指标边界、确认 Cocoa 恢复证据引用正确，并要求补机器 / GPU 与预热边界；已登记 Apple M4、单次初始激活 / 渲染和 31 次测量样本。没有剩余 actionable findings。
- 2026-10-08：规划后续写入导入后台化按 108 的工程写入协调和 109 的执行基础契约接入；沿用本任务的基础依赖，避免把历史导入基础与后续架构整理连成循环依赖。当时只读预检已完成，写入取消 / 提交截止点仍待实现；该缺口已由本次异步写入批次补齐。
- 2026-10-13：确认导入写入后台化已完成 macOS 本地验收；取消 / 提交截止点、失败清理、容量拒绝恢复和真实窗口结果记于“确认写入异步批次”。本批未运行 sanitizer / CI；6.1 medium 独立复审未发现阻断，异步失败 / 通道断开 / teardown 的专用回归仍待补充。

- 2026-10-02：CI 运行 [37005793824](https://github.com/Yuki-Nagori/panta/actions/runs/37005793824) 的 Linux TSan 在四个新接入后台预检的 ViewModel 用例中报告 Rust std mpsc 交付 / 释放竞态。当前 sanitizer 仅插桩自有 C++，Rust std 同步不可见，属于 042 已登记的工具边界；本轮按精确用例名补齐 TSan 组合排除，普通聚合及 ASan/UBSan 继续执行全部预检回归，Rust 并发正确性由既有 Miri 入口覆盖。不改业务同步实现或新增运行时抑制；修复后 macOS `cargo test --locked --workspace` 通过 Rust 测试及 72/72 个 native / QML CTest；`cargo sanitize` 通过 ASan/UBSan 72/72、TSan 51/51，`cargo format --check` 通过。用 CTest `--show-only=json-v1 -E` 对比实际清单，确认相对原名单仅减少这四个用例。`cargo lint --check` 全部通过；Linux 最终结果待新 CI 验证。

- 2026-10-10：GitHub Actions run [38037921716](https://github.com/Yuki-Nagori/panta/actions/runs/38037921716) 的 macOS workspace 测试中，非 UTF-8 文件名夹具在临时文件系统创建阶段返回 EILSEQ；验证继续检查该路径被拒绝。测试现识别 macOS 文件系统对此名称的拒绝；定向测试、`cargo test --locked --workspace`、`cargo lint --check` 均通过，修复提交 `afba765` 的三平台 CI run [38039422235](https://github.com/Yuki-Nagori/panta/actions/runs/38039422235) 全绿。sanitizer 运行时边界见任务 042。

- 2026-10-02：异步迁移按可验证边界分批实施。首批将 STL 预检迁入 Rust 后台；请求拥有独立编号、取消标志和字节 / 网格快照，GUI 线程只发起、取消和拉取结果。替换文件选择、关闭弹窗及成功切换工程均废弃旧请求，旧结果不得重新启用 OK 或污染新表单；预检成功后保留同一来源快照供确认时检查源文件变化。补充 Rust / CXX / Qt 的失败、取消、迟到结果与工程不变回归。确认导入的写入事务及取消截止点留在下一批，当前不标异步导入验收完成。

- 2026-10-02：经 gh 核实提交 `2f6260c` 的 CI 运行 36994873700 成功，补齐三平台测试、sanitizer、覆盖率及质量检查证据；移除已完成的 CI 待验收描述。异步导入迁移与四档缩放仍未完成，任务及索引保持 in-progress。

- 2026-10-02：推进剩余诊断边界。Rust `ProjectError` 显式提供稳定 code / category / detail，CXX 工程操作返回带诊断的值 DTO；覆盖创建、打开、保存、命令、STL 预检 / 导入、元数据确认及资产激活入口，删除 ViewModel 按冒号拆分异常文本的逻辑。Qt 暴露分类与上下文，用户摘要继续按稳定码本地化；失败保留既有工程、导入记录及有效视口。补充跨语言失败 / 修订 / 上下文回归，已执行 Cargo 聚合、sanitizer、质量检查与真实窗口复核。UI 样式、manifest schema 与已保存资产保持现状。

- 2026-09-22：创建任务。用户确认当前先推进 `.stl`，后续支持更多格式；导入选项应写入 `.panta`，以便工程重开和结果复现。
- 2026-09-22：采用“源资产引用 + import record”而不是只保存绝对路径；`meshType` / `units` 属于可重放工程数据，`showImportLog` 保留为本次导入的展示偏好。
- 2026-09-22：先补 HTML 状态，不改运行时代码；新增新建项目弹窗、导入弹窗和导入后工作区参考。
- 2026-09-22：Rust 工程服务扩展 schema 2 和 `ImportRecord`，导入确认后复制 STL 到 `assets/imports/`，并为 schema 1 清单提供缺省 imports 读取。
- 2026-09-22：为 import record 固定 `record_version=1` 与 `parser_version=1`，使后续解析器升级可以显式拒绝或迁移旧记录。
- 2026-09-22：CXX `ProjectViewModel` 接入 STL 预检 / 导入 / 重开恢复，QML 接入 ImportDialog、任务树和视口资产路径；VTK 适配层增加 ASCII / 二进制 STL 到 `vtkPolyData` 的解析。
- 2026-09-22：将跨页面的导航、建模、分析、结果、报告、帮助和单位文案归并到顶部公共 i18n 分类，删除已无引用的页面专用重复上下文；`.pa` 的同长度源文案限制通过公共子分类保持显式隔离。
- 2026-09-22：尝试在现有 target 做无 Bridge 消融构建时发现旧生成 MOC 与配置切换不一致；未修改业务代码绕过，已恢复默认 Bridge=ON 构建配置并完成默认回归测试。
- 2026-09-24：按用户反馈，底部 Layers Dock 外框与工具栏保持显示；只有导入 STL 后显示带图标的 Layers 页签行，图层内容留空。
- 2026-09-24：架构评审补充 Qt / Rust 结构化错误边界；现有 C++ ViewModel 仍解析 `rust::Error` 文本前缀，记录为本任务未完成的导入诊断迁移，不视作已满足该约定。

## 完成摘要

已完成任务登记、`.panta` 持久化边界说明、HTML 状态参考，以及 STL 预检 / 确认导入 / 资产复制 / 重开恢复 / 任务树 / VTK 视口链路。结构化错误迁移、失败回归、异步预检与确认写入、`display_name` 持久化与重名展示、Cargo 聚合 / 质量检查、四档 Qt 缩放行为、四档 Metal 硬件离屏性能基准和既有 macOS 可见窗口验收均已完成。保存资产的异步激活由 073/080 实现；实际 Create Mesh 不属于本任务。任务验收完成。

## 契约更新（080，2026-09-26）

打开工程不再同步重载最新导入网格（消除同步 / 异步双路径）；已保存 STL 由 073 的只读激活 FSM 按 `ImportRecord.id` 异步恢复，服务 080 视口文档页签。记录与资产契约（稳定 ID、相对资产引用、失败回滚）不变。
