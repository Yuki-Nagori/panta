# UI 模块（qml.md）：qt_add_qml_module 与 qml/ 目录布局一致，由 native 顶层纳入构建。
# URI Panta.Shell；资源前缀由 QTP0001(NEW) 统一为 /qt/qml。

# 主题单例声明必须先于 qt_add_qml_module（qmldir 的 singleton 标记在调用时读取）。
set_source_files_properties(Themes/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_qml_module(panta_shell
  URI Panta.Shell
  VERSION 1.0
  RESOURCE_PREFIX /qt/qml
  NO_PLUGIN
  QML_FILES
    App.qml
    Themes/Theme.qml
    Components/Atoms/ThemedLabel.qml
    Components/Atoms/ThemedButton.qml
    Components/Atoms/PanelSurface.qml
    Panels/PlaceholderPanel.qml
)

set_target_properties(panta_shell PROPERTIES EXPORT_NAME shell)
add_library(panta::shell ALIAS panta_shell)

# 资源扩展点：未来图标/内置资源在此追加 qt_add_resources（资源入口，任务 005 决策）。
