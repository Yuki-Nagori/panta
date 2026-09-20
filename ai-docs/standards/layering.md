# 分层与依赖方向

更新日期：2026-09-19。来源：维护者给定结构，对全部任务生效；违反即评审问题。

## 结构

```text
QML 界面
  → C++ ViewModel / Controller
    → Application Services
      ├─ C++ Geometry / Mesh / RenderScene
      │    → OCCT / Netgen / VTK adapters
      └─ Rust Project / Workflow / Storage / Solver Client
           → panta-core（领域模型）
           → panta-foundation（进程级基础设施）
                                                ↓ 进程协议
                                  [External] MoldSolver

C++ shell / services ⇄ panta-ffi（CXX 边界）⇄ Rust crates
```

## 规则

- 依赖方向单向：QML → ViewModel/Controller → 应用服务 → 领域模型/适配器；
  禁止依赖循环与反向依赖。
- 界面表达用户意图，服务协调业务，领域模型表达几何、网格、工程和结果，
  适配器封装第三方库。
- `panta-core` 只承载可复用的领域模型与业务规则，不依赖 Qt、VTK、C++ 或
  FFI；它保持无手写 `unsafe`。进程级平台设施不因“属于 Rust”就塞进
  `panta-core`。
- 库对象不能成为跨层公共数据模型：`vtkActor`/`vtkDataSet`/OCCT/Netgen 等
  第三方类型不出现在 QML、ViewModel 或服务接口签名中，只存在于对应
  adapter 内部。
- 渲染等热路径不得形成 QML → C++ → Rust → C++ → VTK 的往返链路；渲染
  状态留在 C++ 后端内部（如 `RenderScene`），Rust 不参与逐帧路径。
- 进程级基础设施（崩溃信号、日志、生命周期编排）归 `panta-foundation`
  等 Rust 基础设施 crate；需要 C++ 壳初始化时，经 `panta-ffi` 暴露安全的
  CXX 入口。第三方 C++ 库对接仍归 C++ 适配器。
- `panta-ffi` 只负责 DTO、错误、句柄和生命周期等跨语言契约；不得把领域
  实现、VTK 对象或手写平台设施长期堆在 FFI crate 中。FFI 入口可以调用
  `panta-core` 与 `panta-foundation`，反向依赖禁止。
- `qml/Themes/Theme.qml` 是 QML 的唯一视觉 token 门面。默认 GUI 尺寸迁移到
  `.pa` 的 `kind: variables` 基线，主题颜色和允许覆盖的视觉 token 放在独立
  `kind: theme` 文件中，按“基线 → 覆盖 → 校验 → C++/QML 快照”发布；`.pa`
  不由 QML 直接解析。
- 运行时由窗口和 item 决定的 surface 坐标、实际宽高、设备像素尺寸及
  Wayland/Win32/Cocoa 原生对象属于 C++ 平台适配层，不是 Theme token，也不应
  为了“集中配置”搬进 Rust。Rust 只在未来需要持久化或校验用户/工程主题选择
  时，经既有服务与 FFI 边界承载数据，不进入 QML → C++ → Rust → C++ → VTK
  的热路径。
- 用户设置遵循同一边界：Rust 定义 schema、默认值和业务校验，C++/Qt adapter
  调用 `QSettings` 完成平台存取；`QSettings` 不进入 `panta-core`，QML 不直接
  依赖它。主题 `.pa` 是只读定义，设置只保存主题 ID 和用户偏好。
- 手写 `unsafe` 集中在明确登记的边界模块（当前为
  `panta-foundation::crash`）；每个块写明指针、FD、线程和信号处理前提，
  对外仍提供安全 API。CXX 生成胶水的 unsafe 属于工具边界，按 FFI 规范审查。
- External MoldSolver 只经进程协议接入，不链接其 ABI。

## 例外登记

新增例外必须在本节登记（或链接对应任务），不接受未登记的隐式例外。
当前无。
