# 网格模型、Netgen 与 Mesh IR

[架构总览](README.md)

## 自有 Mesh IR

网格模块拥有独立于 Netgen 和 VTK 的中间表示（Mesh IR）。基本信息包括节点坐标、单元类型与连接关系、节点/单元 ID、区域与边界分组、单位、来源几何修订及生成参数。

混合单元必须明确各单元的类型和局部节点顺序；边界面、体单元及其对应关系应可验证。索引宽度、浮点精度和内存布局在实现时依据规模确定并写入格式说明，不能依赖某个库的隐式默认值。

当前权威 Mesh IR、STL 解码、体网格校验与总体积计算在 `crates/panta-mesh/`。`native/mesh/` 调用 Netgen，将库对象转换成明确命名的 native DTO，再经 CXX 委托 Rust 校验。C++ 只保留库结果转换、安全边界检查与局部节点顺序归一化；它不再保存第二套领域校验规则或重复计算网格总体积。STL 文件读取由 `panta-import` 编排、`panta-mesh` 解码，VTK 只接收已提交快照。职责与依赖方向见 [适配边界](native-domain-boundaries.md)。

## 从 STL 到显示和计算的路线（A 已有，B/C 规划）

```text
STL source asset → panta-import → panta-mesh::SurfaceMesh（Rust 权威）
                                      ├─ A. 直接显示：批量 CXX 快照 → C++ vtkPolyData → VTK
                                      ├─ B. 几何修复任务（规划）：C++ OCCT adapter → faceted B-rep
                                      │       → Rust 提交新 GeometryAsset / revision
                                      │       → OCCT 三角化 → vtkPolyData → VTK
                                      └─ C. 体网格任务（规划）：C++ Netgen adapter → native DTO
                                              → panta-mesh 体 Mesh 校验 / 提交
                                              → vtkUnstructuredGrid → VTK
```

Rust 拥有资产身份、来源 revision、转换 / 网格任务策略、参数和成功提交；C++ adapter 调用 OCCT / Netgen、转换输入输出并管理 native 所有权；VTK 只承担显示对象与 GPU 生命周期。场景 A 使用单个批量快照，不让 QML 持有数组，也不将 VTK 类型穿过 Rust。当前 SurfaceMesh 以每个三角面三个 f64 顶点表示；C++ 直接生成面法线，绕过 VTK 上重复拆点 / 三角化的处理。

场景 B 从 STL 三角面重建的是离散面片构成的 faceted B-rep；这不会恢复原始 CAD 的解析曲面、参数和设计历史。需要曲面拟合或拓扑修复时，任务必须定义容差、诊断、实体有效性、失败保留旧 GeometryAsset 和映射失效行为。GeometryAsset 的 ID 与 revision 由 Rust 持有；C++ OCCT shape 是运行期 native 对象，不能当成跨 FFI 的 Rust 领域对象。

场景 C 首期按用户提出的路线以明确 `SurfaceMesh` revision 与参数快照为输入。固定 SDK Netgen 6.2.2604 的 `nglib.h` 提供 `Ng_STL_NewGeometry` / `Ng_STL_AddTriangle` / `Ng_STL_InitSTLGeometry` 接收三角面、`Ng_STL_GenerateSurfaceMesh` 生成表面网格，再由 `Ng_GenerateVolumeMesh` 生成体网格的 API 路径；这证明 C++ adapter 有直接传入三角面数据的 API 边界，但不证明任意 STL 都闭合、可定向或可成功生成体网格，需通过真实失败样例验收。Netgen 只在 C++ adapter 中运行；输出经 native DTO 交 Rust `panta-mesh` 校验，再按新资产 revision 提交为体 Mesh。当前领域类型为 `TetMesh`，接口若对产品使用 `VolumeMesh` 名称需在实施任务中统一术语，不维护第二份 Mesh IR。A/B/C 操作都不能覆盖其来源资产。

Netgen API 依据：[v6.2.2604 `nglib.h`](https://github.com/NGSolve/netgen/blob/v6.2.2604/nglib/nglib.h)。最终能否对特定输入直接体网格化，仍由未来网格任务的封闭性、方向、容差及失败诊断测试决定。

Mesh IR 的旁置持久化和页签按需读取由 [088](../task/088-mesh-asset-sidecar-storage.md) 规划：`.panta` 保存索引，Mesh payload 使用工程内相对引用，打开项目不把所有网格读入内存。持久文件格式不依赖 VTK / Netgen 对象布局；内容格式版本和来源 revision 不匹配时拒绝作为当前资产使用。

## 生成流程

应用服务确认几何有效，冻结输入修订和网格参数，创建后台任务，再由 Netgen 适配器生成输出。完成后转换为 Mesh IR、检查有效性、形成质量摘要，最后一次性提交新的网格资产。

最初提供全局尺寸等少量可解释参数；局部加密、曲率控制等选项在验证后逐步增加。界面显示参数单位、有效范围和预计代价，避免暴露未经封装的库内部配置。

生成中不能覆盖当前有效网格。取消、失败或输入几何已变更时，丢弃或隔离候选输出；成功提交才触发视口刷新、工程 dirty 状态和旧 Study/结果的依赖失效检查。

## 有效性与质量

至少检查连接索引范围、非有限坐标、重复/退化单元、单元方向、区域分组完整性，以及适用于目标分析的体积或面积条件。质量指标及阈值必须声明适用单元类型，不能用一个未定义的“quality”数字覆盖所有网格。

几何到网格的边界映射是分析配置的关键资产。记录每个分组对应的几何实体和网格实体；转换后验证分组没有意外丢失。生成完成不等于适合所有求解器，还需经过求解器能力和单元支持检查。

## 显示与缓存

体网格可派生外表面用于高效显示，裁剪和截面可按需要访问体单元。用于显示的抽稀结果或缓存不能回写为分析输入。缓存以网格修订和显示参数为键，在资产更换后失效。

验收覆盖一个简单闭合实体、失败/取消场景、边界映射和存储往返。性能指标在有代表性样例后设定，不在当前文档承诺未经测量的规模或速度。
