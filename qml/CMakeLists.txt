# UI 模块（qml.md）：qt_add_qml_module 与 qml/ 目录布局一致，由 native 顶层纳入构建。
# URI Panta.Shell；资源前缀由 QTP0001(NEW) 统一为 /qt/qml。

# 主题单例声明必须先于 qt_add_qml_module（qmldir 的 singleton 标记在调用时读取）。
set_source_files_properties(Themes/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
# 输出根目录需在创建模块前设定；Qt 会在其下追加 URI 的 Panta/Shell
# 路径，便于 qmllint、qmltestrunner 和 IDE 通过标准 import 解析模块。
set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/..")

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

if(BUILD_TESTING)
  add_executable(panta_qml_theme_component_test Tests/theme_component_test.cpp)
  panta_native_defaults(panta_qml_theme_component_test)
  target_link_libraries(panta_qml_theme_component_test PRIVATE
    panta_shell
    Qt6::Gui
    Qt6::Qml
    Qt6::Test
  )
  add_test(
    NAME Qml.ThemeComponentParameters
    COMMAND panta_qml_theme_component_test
  )
  set_tests_properties(Qml.ThemeComponentParameters PROPERTIES
    ENVIRONMENT "QT_QPA_PLATFORM=offscreen"
  )
endif()

# 资源扩展点：未来图标/内置资源在此追加 qt_add_resources（资源入口，任务 005 决策）。
