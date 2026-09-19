# 分层与依赖方向

更新日期：2026-09-19。来源：维护者给定结构，对全部任务生效；违反即评审问题。

## 结构

```text
QML 界面
  → C++ ViewModel / Controller
    → Application Services
      ├─ C++ Geometry / Mesh / RenderScene → OCCT / Netgen / VTK adapters
      └─ Rust Project / Workflow / Storage / Solver Client
                                                ↓ 进程协议
                                  [External] MoldSolver
```

## 规则

- 依赖方向单向：QML → ViewModel/Controller → 应用服务 → 领域模型/适配器；
  禁止依赖循环与反向依赖。
- 界面表达用户意图，服务协调业务，领域模型表达几何、网格、工程和结果，
  适配器封装第三方库。
- 库对象不能成为跨层公共数据模型：`vtkActor`/`vtkDataSet`/OCCT/Netgen 等
  第三方类型不出现在 QML、ViewModel 或服务接口签名中，只存在于对应
  adapter 内部。
- 渲染等热路径不得形成 QML → C++ → Rust → C++ → VTK 的往返链路；渲染
  状态留在 C++ 后端内部（如 `RenderScene`），Rust 不参与逐帧路径。
- 进程级基础设施（崩溃信号、日志、生命周期编排）归属 Rust 实现，经既有
  FFI 边界暴露给 C++ 壳（047 决策）；第三方 C++ 库对接仍归 C++ 适配器。
- External MoldSolver 只经进程协议接入，不链接其 ABI。

## 例外登记

新增例外必须在本节登记（或链接对应任务），不接受未登记的隐式例外。
当前无。
