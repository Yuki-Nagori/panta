# QML 组件与声明式界面

查阅日期：2026-09-16。状态：Qt 6.11.2 预编译链路已由任务 005 落地验证（qmllint 经 `all_qmllint` 目标接入；手动注册类型对 qmllint 不可见为已知限制）。

适用于 `qml/`；Qt 对象模型参见 [Qt](qt.md)，视口参见 [VTK](vtk.md)。

## 官方依据

Qt 建议分离界面与业务逻辑，并使用资源系统组织应用资源；QML 文件与模块目录的关系影响 implicit imports。[QML Best Practices](https://doc.qt.io/qt-6/qtquick-bestpractices.html)

`qt_add_qml_module` 管理 QML 文件、资源和相关构建工具集成。[qt_add_qml_module](https://doc.qt.io/qt-6/qt-add-qml-module.html)

## 项目规则

- 组件文件 `PascalCase.qml`，id/属性/信号使用 `camelCase`；命名反映界面语义，不把库名当作用户功能名。
- 每个可复用组件声明输入属性与输出信号，通过显式属性注入 ViewModel；避免依赖任意上层 id 或散布全局 context properties。
- 状态优先用属性绑定表达；命令式赋值前检查是否会破坏原绑定。组件不直接读写工程文件，也不调用 OCCT、Netgen、VTK 算法。
- 长列表使用有明确角色的模型和 delegate；不将大型网格数组转换成 JavaScript 对象供 UI 管理。
- 将 `qt_add_qml_module` 放在与模块 QML 布局一致的位置，由 native 顶层纳入构建；若必须跨目录，任务 005 要验证 import 和资源别名，不能仅靠开发机 import path 可用。
- 图标和内置 QML 通过模块资源定位；用户输入文件通过应用服务处理 URL/path 转换。避免当前工作目录相关的相对资源路径。
- 主题、间距和色彩集中管理；文字通过翻译接口包裹；焦点、键盘操作与禁用原因可理解。第一版只搭壳，不提前堆砌业务面板。

## 验证

使用所锁定 Qt 提供的 qmllint/格式工具检查实际模块；验证必需属性、导入、绑定循环、窗口缩放和键盘焦点。打包后的资源加载必须独立验证，不能只用源码目录启动成功作为证据。
