# 重库适配与 Rust 领域模块

[架构总览](README.md) · [分层规则](../standards/layering.md) · [任务 066](../task/066-native-domain-boundaries.md)

更新日期：2026-09-23。本文记录目标边界与当前代码审计；新增 crate、服务接口及迁移均为规划，066 只交付文档。

## 分层与调用

```text
QML → C++ ViewModel → CXX → Rust 应用服务 / 领域模型
                                  ↓ 通过注入的后端接口提交粗粒度操作
                              CXX → C++ adapter → OCCT / Netgen

Rust 已提交的资产 / 显示配置 → CXX → C++ RenderScene / ViewportBackend
                                             ↓
                                C++ VTK 后端 → VTK / WebGPU
                                （窗口、输入、逐帧更新在本地完成）
```

Rust 管理工程身份、单位、修订、引用、任务与成功提交，C++ 适配器负责实际调用重库、转换输入输出及管理 native 资源。CXX 声明受支持的类型和签名、生成胶水，不决定业务行为。调用流程不等同于 crate 依赖图；领域层通过自有后端接口接收能力，不反向依赖桥接 crate。

“不直接链接 OCCT/Netgen”在此指 Rust 业务代码不得直接声明或调用重库 ABI、不得暴露其对象布局；桌面最终产物仍需链接重库。当前最终链接由 CMake 拥有，CXX 不消除链接和运行库部署要求。

适配器对外使用小型值 DTO、连续数组或自有 opaque 所有者。大型 B-rep 无须强行完整复制到 Rust：未来可由 native 持有真实 shape，通过自有句柄暴露受限操作；Rust 保存 Shape/Face 的领域身份与修订。句柄不是 OCCT Handle 的别名，也不是可序列化的裸地址。当前 STEP 冒烟只返回摘要，没有持久 shape 服务。

## 适配器允许做什么

| 内容 | 责任 |
|---|---|
| STEP 读取、拓扑操作、几何修复、几何三角化 | C++ 调用 OCCT；修复选项和是否提交由 Rust 决定，不自行重写内核 |
| 网格生成、库提供的网格优化 | C++ 调用 Netgen；Rust 选择参数、输入修订与任务策略 |
| 异常归一化、类型转换、数组展开、索引起点与单元节点顺序转换 | C++ adapter，属于接口语义转换 |
| 工程网格有效性、应用质量阈值、区域 / 边界引用、资产失效 | Rust 领域层；边界仍保留必要长度 / 溢出 / 范围检查 |
| VTK 数据对象、mapper/filter/actor、相机及显示命中、GPU 资源 | C++ 显示后端；调用 VTK 的绘制和数据处理能力 |
| 导入格式解析、领域网格 / 字段、显示选项与选择状态 | Rust；轻量解析 / 校验可自有实现，不替代 OCCT/Netgen 核心算法 |

“薄适配”限制领域策略与重库算法重写，不禁止必要转换中的循环和计算。Netgen 输出索引归一化、异常封装和 native 锁是适配职责。VTK 的 C++ 模块还包含原生显示控制器；将每个鼠标事件、像素命中和相机插值搬到 Rust 不会使接口更清楚。

## 当前 VTK 审计

路径均相对于仓库根目录；结论基于 065 提交后的源码。

| 现有位置 / 符号 | 结论与理由 |
|---|---|
| `native/visualization/src/vtk/stl_mesh.cpp`：`read_binary`、`read_ascii`、`finite_triangle`、`load_stl_mesh` 的文件读取 | 优先提取。Rust `crates/panta-core/src/project.rs` 的 `parse_stl` / `parse_binary_stl` / `parse_ascii_stl` 已做另一套解析与摘要；应统一为 Rust 网格导入入口，返回经校验的表面 Mesh 数据 |
| 同文件 `make_poly_data` | 保留 C++：只把 Mesh 坐标 / 连接关系转换成 `vtkPoints`、`vtkCellArray`、`vtkPolyData`，检查索引转换和缓冲区有效期 |
| `RenderScene::mesh_path`、`CaeViewport::setMeshPath`、`VtkViewport::apply_state` / `update_mesh_actor` | 未来随资产通路改为修订化 Mesh 引用 / 显示快照；工程路径解析、I/O、导入提交归 Rust。当前仍按路径载入 STL，尚未改造 |
| `navigation/viewport_camera.cpp`：`ViewportCamera`、`ViewportCameraTransition` | 保留 C++：操作 VTK 相机，状态仅用于即时导航；数学虽可移植，但当前无独立 Rust 消费者，不增加每帧 FFI |
| `navigation/viewport_input.cpp`：`classify_viewport_input` | 保留 C++：把 VTK 事件映射到本地动作。若将来支持可配置按键，Rust 管理偏好数据，C++ 使用一次下发的配置 |
| `navigation/viewport_orientation.cpp`：`pick_cube_face`、`ViewportOrientation` | 保留 C++：方向控件的显示命中、坐标投影和 VTK 标记同步。工程实体选择的 ID、修订和引用校验未来归 Rust |
| `vtk_viewport.cpp` 的 timer / refresh / camera / window 管理；`vtk_native_surface.*` | 保留 C++：Qt 线程、像素尺度、VTK/GPU 生命周期与平台窗口强耦合 |
| `default_wordmark.cpp`：`create_default_wordmark`、`wordmark_color` | 保留显示模块：临时欢迎图形和装饰颜色，不是分析网格或物理场，不抽成 Rust 领域算法；最终设计仍由 053 决定 |

两套 STL 解析存在静态可见差异：Rust ASCII 路径取 `vertex` 后前三个数而不拒绝额外字段；C++ 路径要求该行恰好四个字段。Rust 用严格 UTF-8 解码，C++ 用 `QString::fromUtf8`。这里只记录源码差异，未新增运行用例证明某个完整文件的最终结果。迁移必须定义唯一接受规则并用同一组合法 / 非法输入回归；不能把当前任一宽松实现直接当作完整 STL 规范。

提取时应让预览、导入与显示消费同一解析结果或同一内容修订的缓存，避免二次读盘后二次解析，也避免源文件变化导致预览和提交不一致。完整的读取、快照和提交方案在实施 task 中确定。

## OCCT / Netgen 当前边界

- `native/geometry/src/occt/step_reader.cpp` 的 `import_step_summary` 已实际调用 STEP reader、`TopExp` 和 `BRepBndLib`，保留适配器。几何资产、稳定 ID 和导入事务未来归 `panta-geom` / Rust 应用服务；摘要中的遍历计数不等于持久拓扑身份。
- `native/mesh/src/netgen/netgen_mesher.cpp` 的 `generate_tet_mesh_from_step` 已调用 Netgen 的 STEP/OCC、边网格、表面网格、体网格流程。`convert_to_ir`、分组压缩和节点顺序归一化留在边界；映射须可追溯，不凭压缩序号建立持久身份。
- `native/mesh/src/mesh_ir.cpp` 的 `validate_tet_mesh` 与 `include/panta/mesh/mesh_ir.hpp` 的自有网格契约适合随 `panta-mesh` 迁移为 Rust 权威数据与校验。实施时保留必要的 C++ 转换安全检查，移除重复的领域校验；不能同时维护两套权威规则。
- 现有 native Mesh IR、STEP 摘要只是适配冒烟，尚未经 CXX 接入完整 Rust 几何 / 网格服务。Netgen 当前资源释放的已知制品问题仍按 010/038 跟踪，本文不把所有权规则写成该问题已经修复。

## Rust crate 划分（规划）

五个名称可以采用，按实际功能逐步建立。`geom` 表示几何领域，`bc` 明确表示 boundary conditions；公共文档和接口注释使用完整含义。

| 规划 crate | 拥有的内容 | 创建触发条件 |
|---|---|---|
| `panta-geom` | Shape/Face/Edge ID、几何修订、单位、导入请求与摘要、拓扑引用有效性、后端操作契约 | 实现 STEP 导入到工程资产的完整通路；不存 `TopoDS_Shape` |
| `panta-mesh` | 表面 / 体 Mesh IR、连接关系、分组、来源几何修订、导入解析与轻量校验、网格后端契约 | 首选从统一 STL 解析与显示数据通路落地；随后接 Netgen |
| `panta-material` | 材料定义、属性及单位、来源版本、表格数据和校验 | 首次材料编辑 / 持久化功能；不实现外部求解器的本构计算 |
| `panta-bc` | 边界条件定义、目标实体 / 分组引用、参数单位、修订与客户端校验 | 首次 Study 边界配置；不负责组装方程或施加数值边界 |
| `panta-visualization` | 后端中立显示配置、字段关联与范围策略、实体选择语义、可保存相机书签 | 首次字段显示 / 选择 / 显示配置持久化的真实消费者；不含 VTK/GPU/Qt 或逐帧导航 |

网格区域到材料的分配、边界条件与网格版本的组合由 Study 应用模型拥有；材料库不依赖网格，网格也不依赖材料或边界条件。VTK 收到已解析的显示快照，不理解求解器参数。

### 编译依赖方向

以下是目标上限，只有实际使用类型才添加依赖：

```text
panta-ffi（组装后端实现、DTO 转换） → Rust 应用服务
Rust 应用服务 → geom / mesh / material / bc / visualization
mesh → geom（仅来源几何契约）
bc → geom / mesh（仅目标引用契约）
visualization → geom / mesh（仅显示来源契约）
geom、material → 无其他上述领域 crate
```

当前 `panta-core` 包含 project/path/task 和工程存储实现，不能同时被当成底层共享类型库又依赖这些领域 crate。过渡期若由 `panta-core` 应用服务消费新领域 crate，领域 crate **不得反向依赖 `panta-core`**。通用类型优先留在真正拥有它们的领域；有跨领域复用证据后才另建低层契约 crate，或先将 project/workflow/storage 分出，再收缩 core。禁止为五个空目录先造一个万能公共包。

`panta-ffi` 当前依赖 `panta-core`，因此不能让 core 再依赖 ffi 来调用重库。规划做法是在领域 / 服务层声明窄后端接口，ffi 的组装层实现该接口并注入服务；CXX 模块只声明绑定，组装函数只转发与转换。编译依赖无环，运行时服务仍可请求 native 工作。具体接口、线程和最终链接须在首个实现任务验证。

## 实施顺序与门槛

1. 另立实现 task，统一 Rust STL 解析并返回表面 Mesh，通过一次批量数据交付给 C++ VTK 转换；同步删除 C++ 文件解析与失效路径 API，补预览 / 导入 / 显示一致性、非法输入、资产更换及复制成本验证。由此建立有真实消费者的 `panta-mesh`。
2. STEP 资产闭环引入 `panta-geom` 与自有 native shape 所有者；Netgen 接入 Rust 任务服务，定义取消、安全串行、失败保留旧网格和迟到修订拒绝；届时迁移 Mesh IR 领域验证。
3. 随材料 / Study 边界和字段显示功能分别建立余下三个 crate。每次迁移包含实现、旧代码删除、测试、构建和规则同步，不保留双权威模型或空包装层。

相机 CPU 基准见 [065](../task/065-vtk-zoom-and-cube-transition.md)：测量不含 GPU，不能据此宣称所有 VTK 算法都在 GPU 执行。当前明确的是 WebGPU 绘制；未增加自研 GPU 计算。
