# Qt 对象、线程与部署

查阅日期：2026-09-16。状态：项目规范草案，尚未完成工具链集成验证。

适用于 Qt 6 C++ bridge、应用入口和资源部署。具体 Qt 次版本由任务 002 锁定。显示缩放和多屏状态遵循[显示缩放与多屏模块](../modules/display-scaling-and-multi-monitor.md)。

## 官方依据

QObject 有线程归属，事件在对象所属线程处理；QObject 可重入不等于同一实例可被任意线程并发操作。[Threads and QObjects](https://doc.qt.io/qt-6/threads-qobject.html)

屏幕、窗口和 DPI 变化通过 `QScreen`/`QWindow` 的公开信号观察；逻辑像素与物理像素转换使用运行期 `devicePixelRatioF()`，不能将 `QScreen*`、屏幕下标或整数缩放比写入持久化状态。[QScreen](https://doc.qt.io/qt-6/qscreen.html)、[High DPI](https://doc.qt.io/qt-6/highdpi.html)

C++ 类型可以通过 QML 注册宏与模块构建暴露到 QML；属性、方法与信号需符合类型系统要求。[C++ QML Types](https://doc.qt.io/qt-6/qtqml-cppintegration-definetypes.html)

QML 部署脚本参与 install 流程，其平台支持和行为依 Qt 版本而定。[QML Deployment](https://doc.qt.io/qt-6/qt-generate-deploy-qml-app-script.html)

## 项目规则

- 只有一个应用对象和清晰的入口初始化序列。ViewModel 在 GUI 线程管理，长任务交给 worker/service，通过排队事件更新可观察状态。
- 类型注册优先使用模块化方式。每个可变 Q_PROPERTY 配置正确通知，值未改变时不重复发通知；模型增删使用相应模型 API，避免静默修改底层容器。
- QObject parent 表达唯一销毁关系；无 parent 对象另定 RAII 所有者，不能两者重复删除。回调捕获 QObject 时使用受控上下文/存活检查。
- 明确 worker、QThread 对象、事件循环的生命周期；停止线程前完成取消与对象清理，不能把 QThread 对象自身当成已运行于 worker 线程。
- 可观察错误进入 UI，详细诊断进入日志；后台错误不打开任意线程的对话框。关闭窗口时阻止迟到任务更新已销毁 ViewModel。
- 发布检查 Qt plugins、QML imports 和 native 动态库。部署脚本只是流程的一部分，不能推断会自动打包所有非 Qt 依赖。
- 屏幕迁移、DPI 改变和热插拔发布完整快照；不要在 GUI 线程之外直接访问 `QScreen`，也不要让单个组件自行处理平台 DPI 分支。渲染像素转换、截图 DPR 和拾取坐标必须在明确的边界完成。

## 验证

任务 005 验证属性通知和资源加载，任务 008 验证线程/取消与关闭时序，任务 013 验证无开发环境路径的安装产物。Qt Quick 图形线程不能按 GUI 线程规则直接推断，另读 [VTK](vtk.md)。
