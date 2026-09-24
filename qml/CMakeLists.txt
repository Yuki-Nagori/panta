# UI 模块（qml.md）：qt_add_qml_module 与 qml/ 目录布局一致，由 native 顶层纳入构建。
# URI Panta.Shell；资源前缀由 QTP0001(NEW) 统一为 /qt/qml。

# 主题单例声明必须先于 qt_add_qml_module（qmldir 的 singleton 标记在调用时读取）。
set_source_files_properties(Themes/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
# 输出根目录需在创建模块前设定；Qt 会在其下追加 URI 的 Panta/Shell
# 路径，便于 qmllint、qmltestrunner 和 IDE 通过标准 import 解析模块。
set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/..")

set(PANTA_SHELL_QML_FILES
    App.qml
    Themes/Theme.qml
    Components/Atoms/ThemedLabel.qml
    Components/Atoms/ThemedToolButton.qml
    Components/Atoms/ThemedTextField.qml
    Components/Atoms/ThemedIcon.qml
    Components/Atoms/PanelSurface.qml
    Components/Composites/ToolGroup.qml
    Components/Composites/RibbonTile.qml
    Components/Composites/RibbonGroup.qml
    Components/Composites/RibbonContent.qml
    Components/Composites/HorizontalToolStrip.qml
    Components/Composites/PanelTabBar.qml
    Components/Composites/PaneCloseButton.qml
    Components/Composites/DialogTitleBar.qml
    Panels/TopChromePanel.qml
    Panels/RibbonPanel.qml
    Panels/Ribbon/HomeRibbon.qml
    Panels/Ribbon/StartLearnRibbon.qml
    Panels/TasksPanel.qml
    Panels/LayersPanel.qml
    Panels/ViewportPane.qml
    Panels/PlaceholderPanel.qml
    Dialogs/NewProjectDialog.qml
    Dialogs/ImportDialog.qml)
set(PANTA_SHELL_QML_DEPENDENCIES)
if(PANTA_ENABLE_BRIDGE_MODULE)
  list(APPEND PANTA_SHELL_QML_DEPENDENCIES DEPENDENCIES TARGET panta_bridge panta_visualization)
endif()
if(NOT PANTA_ENABLE_BRIDGE_MODULE)
  # 提供无 Bridge 的最小 Shell，避免关闭可选模块后残留 Panta.Bridge import；
  # ViewportPane 依赖 Panta.Visualization（CaeViewport），一并剔除。
  list(REMOVE_ITEM PANTA_SHELL_QML_FILES App.qml Panels/ViewportPane.qml
       Dialogs/NewProjectDialog.qml)
  list(APPEND PANTA_SHELL_QML_FILES AppNoBridge.qml)
endif()

if(UNIX AND NOT APPLE)
  # 与 Panta.Bridge 保持一致：Linux runner 的预编译 Qt qtpaths 需要未提供的
  # ICU 73；qmlls.build.ini 是编辑器辅助产物，跳过不影响 QML 资源和 typeinfo。
  set(PANTA_QML_MODULE_OPTIONS __QT_INTERNAL_NO_GENERATE_QMLLS_BUILD_INI)
endif()

# 纯资源模块静态链接，避免 Windows 丢弃无导出符号引用的 DLL 及其资源初始化。
qt_add_qml_module(
  panta_shell
  STATIC
  URI
  Panta.Shell
  VERSION
  1.0
  RESOURCE_PREFIX
  /qt/qml
  NO_PLUGIN
  ${PANTA_SHELL_QML_DEPENDENCIES}
  ${PANTA_QML_MODULE_OPTIONS}
  QML_FILES
  ${PANTA_SHELL_QML_FILES})

set_target_properties(panta_shell PROPERTIES EXPORT_NAME shell)
panta_native_defaults(panta_shell)
add_library(panta::shell ALIAS panta_shell)
target_sources(panta_shell PRIVATE ../native/app/icon_provider.cpp ../native/app/icon_provider.hpp)
target_include_directories(panta_shell PUBLIC ../native/app)
target_link_libraries(panta_shell PRIVATE Qt6::Svg Qt6::Quick Qt6::QuickDialogs2)

# ThemedIcon 使用 image://panta-icons；provider 读取下列 qrc 资源并着色。
# CONFIGURE_DEPENDS 收集资源；视觉、功能及来源清单见 ai-docs/standards/icons.md。
file(
  GLOB PANTA_SHELL_ICON_FILES CONFIGURE_DEPENDS
  RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}"
  "${CMAKE_CURRENT_SOURCE_DIR}/icons/*.svg")
qt_add_resources(panta_shell "panta_shell_icons" PREFIX "/qt/qml/Panta/Shell" FILES
                 ${PANTA_SHELL_ICON_FILES})

if(BUILD_TESTING)
  add_executable(panta_qml_theme_component_test ../tests/qml/theme_component_test.cpp)
  panta_native_defaults(panta_qml_theme_component_test)
  target_link_libraries(panta_qml_theme_component_test PRIVATE panta_shell Qt6::Gui Qt6::Qml
                                                               Qt6::Test Qt6::Quick)
  panta_add_qml_test(Qml.ThemeComponentParameters panta_qml_theme_component_test)
  # 开发侧 QML 性能工作台：按场景扩展，手动运行，不注册为 CTest 或 CI 时间门禁。
  add_executable(panta_qml_cpu_benchmark EXCLUDE_FROM_ALL
                 ../tests/qml/project_docks_cpu_benchmark.cpp)
  panta_native_defaults(panta_qml_cpu_benchmark)
  target_link_libraries(panta_qml_cpu_benchmark PRIVATE panta_shell Qt6::Gui Qt6::Qml Qt6::Test
                                                        Qt6::Quick)
  add_executable(panta_qml_gpu_benchmark EXCLUDE_FROM_ALL
                 ../tests/qml/project_docks_gpu_benchmark.cpp)
  panta_native_defaults(panta_qml_gpu_benchmark)
  target_link_libraries(panta_qml_gpu_benchmark PRIVATE panta_shell Qt6::Gui Qt6::Qml Qt6::Test
                                                        Qt6::Quick)
  # QML 格式门禁（任务 032）：qmlformat 来自 Qt 预编译供给（与 qmllint
  # 同源、同版本锁定），三平台同路径。
  find_program(
    PANTA_QMLFORMAT_EXECUTABLE
    NAMES qmlformat
    PATHS "${QT_STAGING}/bin"
    NO_DEFAULT_PATH REQUIRED)
  add_test(
    NAME Qml.FormatCheck
    COMMAND
      "${CMAKE_COMMAND}" "-DQMLFMT=${PANTA_QMLFORMAT_EXECUTABLE}"
      "-DQML_DIR=${CMAKE_CURRENT_SOURCE_DIR}" -P
      "${CMAKE_CURRENT_SOURCE_DIR}/../native/cmake/tests/check-qml-format.cmake")
endif()
