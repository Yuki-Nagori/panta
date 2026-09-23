# 066 — 重库适配边界与 Rust 领域模块规划

- 状态：done
- 阶段：架构与规范
- 依赖：[065](065-vtk-zoom-and-cube-transition.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-23 / 2026-09-23

## 目标与背景

审计现有 VTK、OCCT、Netgen、CXX 与 Rust 职责，明确哪些逻辑应由 Rust 承担，哪些保留 native；评估 panta-geom、panta-mesh、panta-bc、panta-material、panta-visualization 的边界与依赖，补充长期规则。

## 必读

- [分层](../standards/layering.md)、[跨语言边界](../standards/ffi.md)、[CXX](../standards/cxx.md)
- [VTK](../standards/vtk.md)、[OCCT](../standards/occt.md)、[Netgen](../standards/netgen.md)
- [目录规划](../architecture/repository-layout.md)、[可视化](../architecture/visualization.md)
- [文档规范](../standards/documentation.md)、[仓库文件规范](../standards/repository-hygiene.md)、[提交规范](../standards/commits.md)

## 范围与非目标

本任务交付代码职责审计、crate 依赖规划与规则文档，同步冲突的架构表述。不迁移生产代码，不创建空 crate，不引入新的几何 / 网格算法或 FFI 接口。实现迁移必须另立 task，定义可验证的纵向功能后再实施。

## 实施步骤

1. 读取现有服务、桥接与 VTK 导航 / 几何显示代码，确认现状。
2. 明确重库调用、接口转换、显示编排与 Rust 业务层的责任及所有权。
3. 记录具体提取候选和保留理由，给出无循环 crate 依赖与渐进迁移门槛。
4. 核对文档链接、旧表述及 diff，完成后单独 commit，不 push。

## 清理与兼容例外

清理与新决策冲突的长期架构表述；无生产代码废弃项，无兼容例外。

## 验收标准

- [x] 按真实文件 / 函数列出 VTK 提取与保留结论，区分当前和规划。
- [x] 明确 Rust、cxx、C++ 适配器和重库的类型、异常、所有权、线程与算法边界。
- [x] 评估五个 crate 的职责、依赖方向和创建时机，不新增空壳代码。
- [x] 规则文档与架构入口一致，本地链接、索引和 diff 检查通过。

## 验证计划与结果

文档任务检查改动文档的本地链接、代码围栏、路由与源代码事实；不重复运行应用测试。

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-23 | 仓库根目录，Python 3 临时检查脚本 `/tmp/panta-066-check-docs.py`；遍历 `git diff HEAD --name-only` 与未跟踪文件，检查 Markdown 链接目标存在、围栏配对与 066 索引唯一 | 无断链、围栏配对、任务有唯一入口 | 通过：16 份改动 Markdown，159 个本地链接；退出码 0。脚本为本机一次性检查，不是新增仓库检查入口 |
| 2026-09-23 | 仓库根目录；`git diff HEAD --check` | 无差异空白错误 | 通过，退出码 0 |
| 2026-09-23 | 源码静态核对：VTK 导航 / STL / RenderScene、Rust project parser / ffi、native STEP / Mesh IR / Netgen | 结论有真实函数与调用依据 | 确认 STL 双解析与字段接受差异，确认重库调用、现有 native IR 及尚未接入 Rust 的业务边界；未运行新的行为用例 |
| 2026-09-23 | CXX 官方 Result、UniquePtr、extern C++ 文档 | 规则不夸大桥接保证 | 核对默认异常捕获、opaque 所有权和线程限制；来源已写入 CXX 规范，未改变锁定依赖 |

## 决策与工作记录

- 2026-09-23：065 已以 c63eed3 提交后登记本任务；范围限定为审计与规则规划。

- 2026-09-23：新增架构审计，给出五个领域 crate 的职责、无环依赖和实际功能驱动的创建顺序；同步 AGENTS、分层 / FFI / CXX / 重库规则与相关架构入口。清理旧 C++ 领域职责图、过时 VTK 回退说明和重复 core 目录规划。完成文档静态检查，无代码 / ABI / 依赖变更。

## 完成摘要

审计与规则文档完成。优先迁移项为 Rust 统一 STL 解析 / 网格数据通路，随后按实际业务迁移 Mesh IR 校验；相机动画、显示命中与窗口 / GPU 生命周期保留 C++。五个 crate 均为规划，未创建空壳，也未迁移生产实现。后续实施按架构文档所列步骤另建任务；本任务 done 仅表示审计与规则完成。
