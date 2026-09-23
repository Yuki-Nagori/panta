# 分层与依赖方向

更新日期：2026-09-23。来源：维护者给定结构与任务 066 的边界审计，对全部任务生效；违反即评审问题。当前实现与迁移顺序见 [重库适配与 Rust 领域模块](../architecture/native-domain-boundaries.md)。

## 结构

```text
QML → C++ ViewModel / Controller → CXX → Rust 应用服务 / 领域模型
                                         ↓ 自有后端接口
                                     CXX → C++ adapter → OCCT / Netgen

Rust 资产 / 显示快照 → CXX → C++ RenderScene / ViewportBackend → VTK
                           （原生窗口与逐帧显示留在 C++）
Rust Solver Client → 进程协议 → [External] MoldSolver
```

这是目标职责，不代表五个规划领域 crate 已建立。native 现有 STEP 摘要与 Mesh IR 适配冒烟继续按已落地契约运行，迁移需独立任务。

## 领域、适配与桥接

- Rust 拥有业务逻辑、领域数据、单位 / 引用 / 修订校验、轻量算法与任务编排。OCCT / Netgen 的几何和网格核心算法必须通过 C++ adapter 实际调用重库，不在 Rust 或 adapter 重写。
- C++ adapter 负责第三方类型转换、异常归一化、具体模板实例化、迭代器到数组、所有权和库调用次序 / 锁；不决定工程事务、材料 / 边界业务规则或分析质量阈值。索引起点、局部节点顺序和布局归一化属于转换，可进行必要计算。
- `#[cxx::bridge]` 只声明类型映射和函数签名。`panta-ffi` 的实现仅转换 DTO / 错误 / 句柄、组装并转发服务，不承载领域算法。CXX 不自动证明生命周期、线程安全或业务正确性。
- Rust 业务代码不直接链接调用重库 ABI，不手写 unsafe FFI；最终程序仍通过 native 构建链接重库。适配器对外不暴露 `TopoDS_Shape`、OCCT Handle、`netgen::Mesh`、`vtkActor` 等原始类型，包括把其 typedef 成自有名字。
- 小数据使用值语义；大对象可使用自有 opaque 所有者。接口必须说明创建、释放、跨调用有效期、借用、可否共享 / 修改、线程以及失效后的行为。只保存整数句柄时必须拒绝释放后复用导致的陈旧引用；进程句柄不能持久化为工程身份。
- 可恢复 C++ 错误在适配边界统一成结构化错误或受控 CXX Result；非标准库异常也须显式覆盖。panic 不作可恢复错误，CXX Result 不承诺捕获 Rust panic。具体约束见 [CXX](cxx.md) 与 [FFI](ffi.md)。
- 领域 crate 不依赖 Qt、VTK、CXX 或 `panta-ffi`；业务通过注入的后端接口使用 native 能力。禁止 `panta-core → panta-ffi → panta-core`，也禁止新增领域 crate 与 core 双向依赖；规划依赖图见架构文档。
- 当前 `panta-core` 内的 project/path/task 保持安全 Rust，不依赖 Qt、VTK 或 FFI，按真实职责演进。进程设施归 `panta-foundation`，不能因“属于 Rust”就放进领域 core；`panta-ffi` 可调用两者，反向依赖禁止。

## 显示与数据

- QML 表达用户意图，经 ViewModel / 原生视口提交；不得直接操作重库。ViewModel 负责 Qt 属性、信号与参数转换，不解析几何 / 网格文件或实现工程提交。
- VTK C++ 后端负责渲染对象、缓存、投影与控件命中、相机插值、输入、定时器、窗口和 GPU 资源。渲染热路径不得形成 QML → C++ → Rust → C++ → VTK 的逐帧往返。
- Rust 管理可持久化的显示配置、字段语义、选择引用及工程相机书签；C++ `RenderScene` 是显示快照和可恢复 CPU 状态，不成为第二份工程领域模型。临时鼠标导航留在 C++，确需保存时提交快照。
- 几何解析与资产提交不放进 VTK 显示后端；当前 STL 双解析是已识别的迁移项，详见 066 审计。实施迁移后必须删除重复 parser，不能长期并存两套规则。
- 数据按资产 / 修订批量跨边界，不逐点、逐单元调用 FFI。借用缓冲区必须证明有效期；异步工作拥有快照，先确保所有权再按测量优化复制。第三方缓冲区和借用不能逃逸到 QML。
- 长任务不阻塞 GUI。每个 native 操作明确线程、串行 / 并行与取消阶段；不能为跨线程转移句柄盲目声明 Send/Sync。工程关闭或修订改变后拒绝迟到结果，失败 / 取消不覆盖有效资产。

## 平台与配置

- `qml/Themes/Theme.qml` 是 QML 唯一视觉 token 门面；`.pa` 的 variables / theme 按“基线 → 覆盖 → 校验 → C++/QML 快照”发布，不由 QML 直接解析。
- surface 坐标、窗口尺寸、设备像素以及 Wayland/Win32/Cocoa 对象归 C++ 平台适配层，不搬入主题 token 或 Rust 逐帧路径。
- 用户设置由 Rust 定义 schema、默认值和业务校验；C++/Qt adapter 调用 QSettings，QML 与领域 core 不依赖 QSettings。
- 手写 unsafe 仅在登记的边界模块（当前 `panta-foundation::crash`），逐块说明安全前提；CXX 的 unsafe 声明与生成胶水按 FFI 规则审查，不扩散到业务调用方。
- External MoldSolver 只经进程协议接入，不链接其 ABI。

## 例外登记

新增例外须在本节登记或链接实际任务，不接受隐式例外。当前无新增例外；存量实现与目标差距在 066 架构审计中明确标记，不代表已完成迁移。
