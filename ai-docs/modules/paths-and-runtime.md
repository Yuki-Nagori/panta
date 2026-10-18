# 跨平台路径与轻量运行时

[模块导航](README.md) · [023 路径服务](../task/023-cross-platform-paths.md) · [024 运行时上下文](../task/024-runtime-context.md)

## 边界

这里只描述应用内路径和会话状态。路径层不依赖 DSL；运行时不依赖 DSL 解析器；变量 DSL 将已校验的值提交给运行时。当前运行时不是虚拟 CPU、操作系统、字节码 VM 或安全沙箱。只有出现可测量的执行瓶颈、暂停脚本或第三方代码隔离需求时，才另行评估解释器、字节码或进程隔离。

## 路径层

| 类别 | 当前契约与实现 |
|---|---|
| 工程资产 | `project:/...` 绑定工程包根，不依赖 cwd。`ProjectService` 创建的运行时上下文注入当前工程根。 |
| 用户配置、应用数据、缓存、会话根 | 任务 023 的 Qt `PathHost` 通过 Qt adapter 获取标准目录，创建并验证目录后注入 `PathService`。当前 `ProjectService` 上下文只注入工程根。 |
| 内置资源 | `qrc:` 是只读类别，Rust 路径服务拒绝把它解析成本机路径；Qt 资源通过 Qt 入口读取。 |
| 临时工作目录 | `session` 根映射到 Qt `TempLocation`。当前没有通用的按任务创建、保留诊断和清理目录的管理器。 |
| 外部导入文件 | 本机路径或 file URL 通过显式导入流程读取；来源快照与工程内资产分开保存。 |

`panta-core::path` 定义 `project`、`user-config`、`app-data`、`cache`、`session`、`qrc` 六类根，使用 `scheme:/relative` 结构化引用，并提供三种解析：

- `resolve` 只校验引用并拼接绝对路径，不访问文件系统。
- `resolve_existing` 要求目标存在，规范化后仍位于对应根目录内。
- `resolve_write_target` 允许目标尚不存在，但要求最深现存祖先仍位于根目录内。

路径校验拒绝空引用、NUL、绝对路径、越界 `..`、Windows 保留设备名、尾随点或空格组件及逻辑引用中的反斜杠。文件系统检查覆盖符号链接/junction 越界；这不构成对恶意并发文件系统修改的安全隔离。拒绝矩阵和平台验收见[任务 023](../task/023-cross-platform-paths.md)。

Qt `PathHost` 将 user-config 映射到 `AppConfigLocation`、app-data 映射到 `AppDataLocation`、cache 映射到 `CacheLocation`、session 映射到 `TempLocation`。它在 `native/qt-adapter` 获取标准目录，检查绝对路径、创建缺失目录并用临时文件验证可写性。`QString` 转 UTF-8 必须往返一致；file URL 只通过 `QUrl::toLocalFile` 解码一次。`PathService`/`PathRef` 经 `panta-ffi` 传递；逻辑引用不是 QML URL handler，不能直接交给 QML 或操作系统打开。大文件通过资产 ID 或句柄传递，不放进路径 DTO。

## 运行时层

任务 024 在 `panta-core::runtime` 提供 `RuntimeContext`、`VariableSnapshot` 和 `RuntimeTaskScope`。`ProjectService` 拥有当前上下文：成功创建或打开工程时替换上下文，销毁服务时关闭上下文。当前没有显式关闭工程命令；现有服务生命周期关闭由销毁服务表示。

`RuntimeContext` 持有工程会话 generation、根解析服务及当前变量快照。`ProjectService` 当前只为上下文注入工程包根；标准用户目录等根仍由独立 `PathHost` 管理，尚未注入项目运行时。上下文只开放现存资源解析，不提供任意写入、进程启动或网络能力；持有 `ResourceRef` 不等于获得这些权限。

`VariableSnapshot` 以共享不可变有序表保存 bool、int、有限 real、string、resource 基础值。变量 revision 独立于工程清单 revision。提交时必须提供预期 revision；空名称、非有限数值、旧 revision 或已关闭上下文都会拒绝，失败时保留当前快照。DSL 标识符和类型语义校验由[任务 025](../task/025-variable-dsl.md)负责；024 不解析 DSL，也不把内存快照写入 `.panta`。

后台任务通过 `RuntimeTaskScope` 捕获会话身份、generation、变量 revision、不可变快照和只读路径服务。变量更新后，旧任务仍能读取提交时的值，但当前上下文拒收其结果。工程替换、上下文关闭或销毁会设置共享取消标志；任务需在安全检查点轮询，已进入的文件系统调用无法强制中断。取消信号当前没有接入 008 的演示型 `TaskManager`，也不代表真实任务已迁移到该管理器。

上下文完全由 Rust 管理，独立于 `QQmlEngine`。目前没有跨 CXX 的运行时消费者，因此不新增桥接句柄。C++ ViewModel 不得把 QObject、JS 值或裸指针存入运行时上下文；QML 重载不应隐式重复提交任务。开发热重载行为由[任务 027](../task/027-qml-state-reload.md)验收。

## 状态与后续

023 路径服务已完成；024 运行时基础正在实施。两者都不代表完整工程持久化或任务执行框架已实现。024 使用内存会话夹具验证快照、失效和取消；完整工程数据的读写由各自工程任务负责。打包与安装验收归任务 013。
