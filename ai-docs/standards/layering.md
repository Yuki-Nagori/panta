# 分层与依赖方向

更新日期：2026-10-16。来源：维护者给定结构与任务 066、118 的边界审计，对全部任务生效；违反即评审问题。当前实现与迁移顺序见 [重库适配与 Rust 领域模块](../architecture/native-domain-boundaries.md)。

## 结构

```text
QML → C++ ViewModel / Controller → CXX → Rust 应用服务 / 领域模型
                                         ↓ 自有后端接口
                                     CXX → C++ adapter → OCCT / Netgen

Rust 资产 / 显示快照 → CXX → C++ RenderScene / ViewportBackend → VTK
                           （原生窗口与逐帧显示留在 C++）
Rust Solver Client → 进程协议 → [External] MoldSolver
```

这是目标职责，不代表五个规划领域 crate 已建立。当前 STEP 摘要仍是 native 适配冒烟；Netgen Mesh IR 已由 067 接入 Rust 权威模型与校验。

## 当前分层与允许方向

以下层级依据当前 `Cargo.toml` workspace dependencies、native `target_link_libraries` 与 QML module 声明核对。层号分别在 Rust 图和 Native 图内解释；Rust staticlib、CXX、Qt/QML 是显式跨边界连接，不用层号伪装成同一种依赖。表内“→”表示左侧依赖右侧。测试 target 可依赖被测 target 与测试框架，不反向进入生产依赖图。

### Rust 层级

| 层级 | 当前成员 | 允许的仓库内依赖 | 说明 |
|---|---|---|---|
| Rust L1：基础与叶子库 | `panta-foundation`、`panta-build`、`panta-dsl-core`、`panta-mesh` | 不依赖其他 workspace crate | 基础设施、构建工具、DSL 核心与网格 IR/解析 |
| Rust L2：领域适配与工具入口 | `panta-import`、`panta-dslc` | `panta-import → panta-mesh`；`panta-dslc → panta-dsl-core` | 导入格式适配与 DSL 命令行工具；`panta-dslc` 是独立工具入口，不因此成为应用组合根 |
| Rust L3：应用 / 工程服务 | `panta-core` | `panta-core → panta-import, panta-mesh, panta-dsl-core` | 编排工程状态、资产和领域服务 |
| Rust L4：语言边界 | `panta-ffi` | `panta-ffi → panta-core, panta-mesh, panta-foundation, panta-build` | DTO、错误、句柄转换与服务转发，不承载领域算法 |
| Rust L5：桌面启动组合根 | `panta-launcher` | `panta-launcher → panta-ffi, panta-build, panta-dsl-core` | 组装并启动 native app |
| 测试工具（生产层外） | `panta-tests`、`tests/` | `panta-tests → panta-build`；测试可依赖被测 crate | 只作为验证消费者，不得被生产 crate 依赖 |

### Native 层级

| 层级 | 当前成员 | 允许的 Native target 依赖 | 跨语言边界 |
|---|---|---|---|
| Native L1：适配与基础服务 | `panta_foundation`、`panta_geometry`、`panta_mesh`、`panta_qt_adapter`、`panta_i18n`、`panta_visualization` | 这些 target 之间当前无直接自有 native 库链接；各自可依赖职责范围内的 Qt、OCCT、Netgen、VTK 或平台库 | `panta_mesh` 还链接 Rust FFI staticlib，属于显式 Rust/C++ 边界，不据此建立 Native L1 内部反向层级 |
| Native L2：UI bridge 与 QML | `panta_bridge` / `panta::bridge`、`panta_shell` / `Panta.Shell` | Bridge → `panta::visualization`、`panta::qt_adapter`；QML module 启用 Bridge 时 → `panta_bridge`、`panta_visualization` | Bridge 调用 Rust FFI；QML 只经 ViewModel/Bridge 使用服务 |
| Native L3：应用组合根 | `panta_native_app` | → `panta::foundation`、`panta::shell`、`panta_shellplugin`、`panta::i18n`；启用 Bridge 时还 → `panta::visualization`、Bridge/Visualization plugins | 直接链接 Rust FFI staticlib，并通过 Qt/CXX 连接运行时 |
| 测试与性能（生产层外） | native test、benchmark targets | 可依赖被测生产 target，不进入生产依赖图 | 性能基准由单独入口运行 |

Rust/CMake/QML 的当前允许边据实际清单核对；新增边须先更新分层归属和相应任务，lint 尚未更新时不得用“当前能构建”替代架构评审。跨语言链接须登记在边界表中，不能以新增反向依赖消除构建问题。

Cargo 表覆盖 workspace manifest 中声明的正常、build 与 dev 依赖；CMake 表针对自有 target 的直接链接边，QML module dependency 单独标出，第三方 imported target 按 adapter 边界归类。新增仓库内边需先判断职责归属；允许表或 lint 尚未更新时不得用“当前能构建”替代架构评审。任务 119 实现自动边检查，任务 120 独立实现测试影响范围选择。

### 应用命令层

命令 / 查询的输入、领域校验、编排、结果与稳定错误由 Qt 无关的 Rust 应用服务提供。GUI 通过 `panta-ffi`/Bridge 适配 Qt 值和展示；CLI 将参数适配为同一命令，不复制业务规则。当前 `panta-core::ProjectService` 已是 Rust 服务入口，`panta-dslc` 是 DSL 工具 CLI；仓库尚无面向应用领域的通用 `panta-cli`，不能把 build script 或两个调用方共用一个解析函数误记为共享命令。首个具体共享操作和 CLI 接入由任务 121 根据现有服务确定。

## 领域、适配与桥接

- Rust 拥有业务逻辑、领域数据、单位 / 引用 / 修订校验、轻量算法与任务编排。OCCT / Netgen 的几何和网格核心算法必须通过 C++ adapter 实际调用重库，不在 Rust 或 adapter 重写。
- C++ adapter 负责第三方类型转换、异常归一化、具体模板实例化、迭代器到数组、所有权和库调用次序 / 锁；不决定工程事务、材料 / 边界业务规则或分析质量阈值。索引起点、局部节点顺序和布局归一化属于转换，可进行必要计算。
- `#[cxx::bridge]` 只声明类型映射和函数签名。`panta-ffi` 的实现仅转换 DTO / 错误 / 句柄、组装并转发服务，不承载领域算法。CXX 不自动证明生命周期、线程安全或业务正确性。
- Rust 业务代码不直接链接调用重库 ABI，不手写 unsafe FFI；最终程序仍通过 native 构建链接重库。适配器对外不暴露 `TopoDS_Shape`、OCCT Handle、`netgen::Mesh`、`vtkActor` 等原始类型，包括把其 typedef 成自有名字。
- 小数据使用值语义；大对象可使用自有 opaque 所有者。接口必须说明创建、释放、跨调用有效期、借用、可否共享 / 修改、线程以及失效后的行为。只保存整数句柄时必须拒绝释放后复用导致的陈旧引用；进程句柄不能持久化为工程身份。
- 可恢复 C++ 错误在适配边界统一成结构化错误或受控 CXX Result；非标准库异常也须显式覆盖。panic 不作可恢复错误，CXX Result 不承诺捕获 Rust panic。具体约束见 [CXX](cxx.md) 与 [FFI](ffi.md)。
- 领域 crate 不依赖 Qt、VTK、CXX 或 `panta-ffi`；业务通过注入的后端接口使用 native 能力。禁止 `panta-core → panta-ffi → panta-core`，也禁止新增领域 crate 与 core 双向依赖；规划依赖图见架构文档。
- 当前 `panta-core` 内的 project/path/task 保持安全 Rust，不依赖 Qt、VTK 或 FFI，按真实职责演进。进程设施归 `panta-foundation`，不能因“属于 Rust”就放进领域 core；`panta-ffi` 可调用两者，反向依赖禁止。

## Rust 导入职责（067）

- `panta-import` 是格式入口：识别 STEP / IGES / STL、读取 / 固定来源快照、按格式选择实际后端、持有预览与导入准备状态、记录格式专属诊断和导入选项。当前已接入 STL；STEP / IGES 只有格式识别，读取仍待 `panta-geom` 与 OCCT adapter 的真实业务任务。
- `panta-mesh` 拥有表面 / 体 Mesh IR、STL 字节解析、网格领域校验及总体积摘要。它不管理工程包、Qt、VTK 或 OCCT；STL 不被冒充为 B-rep。
- `panta-core` 拥有工程清单、资产写入、修订和原子提交。它调用 `panta-import` 取得已验证的输入，提交成功才替换当前网格。工程包事务不由格式解析器自行完成。
- 运行期 `SurfaceMesh` 的预算、LRU、驻留 ID 和按页签关闭释放由 `panta-core::ProjectService` 管理；活动页签 ID 仍是 ViewModel 的 UI 选择状态，Rust 只接收 pin / unpin 提示并返回批量驻留状态。ViewModel 只保留当前活动显示 DTO，不能按文档另建 Mesh 缓存。
- 后续 `panta-geom` 拥有 Shape / Face / Edge 的领域身份及 STEP / IGES 结果契约；C++ OCCT adapter 真正执行这两种几何格式的读取、转换与所需底层算法。`panta-import` 负责分发，不在 Rust 复制 OCCT 解析器。
- 依赖方向为 `panta-core → panta-import → panta-mesh`；`panta-core` 在持有网格快照时可直接依赖 `panta-mesh`。`panta-ffi → panta-core / panta-mesh` 负责跨语言 DTO。反向依赖禁止。未来 `panta-import → panta-geom` 只在 STEP / IGES 服务接入时添加。
- 公共导入结果应保留格式、数据类别、单位、来源修订与诊断；共享导入编排不抹平 STL 表面网格和 STEP / IGES B-rep 的语义。新增格式需要实际后端和端到端消费者，不为统一名称创建空 parser。

## 显示与数据

- QML 表达用户意图，经 ViewModel / 原生视口提交；不得直接操作重库。ViewModel 负责 Qt 属性、信号与参数转换，不解析几何 / 网格文件或实现工程提交。
- VTK C++ 后端负责渲染对象缓存、投影与控件命中、相机插值、输入、定时器、窗口和 GPU 资源；Rust `ProjectService` 管领域 `SurfaceMesh` 缓存。`MeshSource::mesh_snapshot()` 只返回本地活动显示 DTO；导入、激活等粗粒度状态变化才跨 CXX 获取数据，渲染刷新与逐帧路径不得形成 QML → C++ → Rust → C++ → VTK 往返。
- Rust 管理可持久化的显示配置、字段语义、选择引用及工程相机书签；C++ `RenderScene` 是显示快照和可恢复 CPU 状态，不成为第二份工程领域模型。临时鼠标导航留在 C++，确需保存时提交快照。
- 几何解析与资产提交不放进 VTK 显示后端；067 已删除 VTK STL parser，由 Rust 网格后端解码，VTK 只构造显示对象。历史差异与审计依据见 066 / 067。
- 数据按资产 / 修订批量跨边界，不逐点、逐单元调用 FFI。借用缓冲区必须证明有效期；异步工作拥有快照，先确保所有权再按测量优化复制。第三方缓冲区和借用不能逃逸到 QML。
- 长任务不阻塞 GUI。每个 native 操作明确线程、串行 / 并行与取消阶段；不能为跨线程转移句柄盲目声明 Send/Sync。工程关闭或修订改变后拒绝迟到结果，失败 / 取消不覆盖有效资产。

## 平台与配置

- `qml/Themes/Theme.qml` 是 QML 唯一视觉 token 门面；`.pa` 的 variables / theme 按“基线 → 覆盖 → 校验 → C++/QML 快照”发布，不由 QML 直接解析。
- `native/qt-adapter` 集中 Qt 封装的系统服务与操作系统 API（例如标准目录发现、系统偏好）；Bridge/ViewModel 消费稳定的 Qt 值/属性并对接 QML 或 Rust DTO，不直接承载 AppKit、Win32、D-Bus 等平台查询。该 target 不依赖 Rust、Bridge 或 VTK。
- locale 支持集、校验和当前选择由 Rust 服务拥有；Qt adapter 管理 `QTranslator`，Bridge `Settings` 将 Rust locale 选择与 Qt resource 加载、QML 重翻译协调起来。Rust 不接触 Qt 类型、QML/Qt 事件循环或 QM resource URL；QML 不维护第二份 locale 列表。
- surface 坐标、窗口尺寸、设备像素以及 Wayland/Win32/Cocoa 对象归 C++ 平台适配层，不搬入主题 token 或 Rust 逐帧路径。
- Qt Quick 与 VTK hardware window 生命周期紧密耦合的原生 surface 代码归 `native/visualization`，不得为了平台 API 归类而让通用 Qt adapter 反向依赖 VTK。
- 用户设置由 Rust 定义 schema、默认值和业务校验；C++/Qt adapter 调用 QSettings，QML 与领域 core 不依赖 QSettings。
- 手写 unsafe 仅在登记的边界模块（当前 `panta-foundation::crash`），逐块说明安全前提；CXX 的 unsafe 声明与生成胶水按 FFI 规则审查，不扩散到业务调用方。
- External MoldSolver 只经进程协议接入，不链接其 ABI。

## 例外登记

新增例外须在本节登记或链接实际任务，不接受隐式例外。当前无新增例外；存量实现与目标差距在 066 架构审计中明确标记，不代表已完成迁移。
