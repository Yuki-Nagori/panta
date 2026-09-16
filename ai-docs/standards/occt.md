# OpenCASCADE / OCCT 几何适配

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 `native/geometry/occt/`。OCCT 版本尚未锁定，在线文档重定向后的 API 不能直接当成本地已可用 API。

## 官方依据

基础 STEP translator 与 XDE 路径的元数据能力不同；XDE 用于名称、颜色等附属信息。读取与转换是需检查结果的多个阶段。[OCCT STEP Translator](https://dev.opencascade.org/doc/overview/html/occt_user_guides__step.html)

OCCT 有自己的基础类型、内存与 handle 机制，不能以通用裸指针所有权习惯替代。[OCCT Foundation Classes](https://dev.opencascade.org/doc/overview/html/occt_user_guides__foundation_classes.html)

## 项目规则

- 所有 OCCT include 和类型限定在 adapter 内；公共几何契约使用工程 ID、单位、修订和自有诊断。
- 任务 009 根据是否需要装配/名称/颜色选择 STEPControl 或 STEPCAF/XDE 路径，记录不保留的元数据，不向用户承诺完整 STEP 语义保真。
- 每次导入检查读取状态、转换结果、有效 shape 数与空形状；记录单位和公差策略。修复行为必须留诊断，不能默默改变模型再报告无误。
- 正确使用所选版本的 handle 类型；复制拓扑句柄不能自动视为深拷贝。修改共享 shape 前明确共享关系，避免后台操作改变当前场景。
- 任务采用独立导入上下文；在确认线程安全前保守串行执行，不并发修改进程级 translator 参数。允许并发时逐项记录版本、API 和参数隔离证据。
- 拓扑 ID 由工程管理，OCCT 对象地址与遍历序号不作为持久化标识；重新导入后重新检查所有边界引用。
- B-rep、显示三角化与分析网格分开；公差与显示精度均记录单位，禁止通过调大公差掩盖任意坏模型。

## 验证

小模型测试覆盖读取失败、空输出、单位和装配/颜色保留策略；检查 Netgen 与桌面使用的 OCCT ABI/版本一致性。任务 009 只验证 adapter 和样例，完整导入产品流程另建业务 task。
