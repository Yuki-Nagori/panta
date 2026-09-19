# 桌面入口（任务 005）：链接 QML 模块与 ViewModel 模块。
add_executable(panta_native_app main.cpp)
set_target_properties(panta_native_app PROPERTIES OUTPUT_NAME panta-native EXPORT_NAME native-app)
panta_native_defaults(panta_native_app)
target_link_libraries(
  panta_native_app
  PRIVATE panta::foundation
          panta::shell
          panta::i18n
          Qt6::Gui
          Qt6::Qml
          Qt6::Quick
          Qt6::QuickControls2)
if(PANTA_ENABLE_BRIDGE_MODULE)
  # main.cpp 在 QGuiApplication 前调用 prepare_graphics_environment()。
  target_link_libraries(panta_native_app PRIVATE panta::visualization)
  target_link_libraries(panta_native_app PRIVATE panta_bridgeplugin panta_visualizationplugin)
  target_compile_definitions(panta_native_app PRIVATE PANTA_ENABLE_BRIDGE_MODULE=1)
endif()

# 开启 Panta.Bridge 时，其静态 plugin 由 main.cpp 中的
# Q_IMPORT_QML_PLUGIN 显式保留；关闭时使用无 Bridge 的 Shell fallback。
# App.qml 或 AppNoBridge.qml 已作为 Panta.Shell 模块资源编译，app 目标没有可供 qmlimportscanner
# 扫描的源文件；调用扫描器只会产生空结果，并要求工具额外加载平台 ICU。

if(BUILD_TESTING)
  add_executable(panta_qml_shell_module_test ../../tests/cpp/app/shell_module_load_test.cpp)
  panta_native_defaults(panta_qml_shell_module_test)
  target_link_libraries(panta_qml_shell_module_test PRIVATE panta::shell Qt6::Gui Qt6::Qml
                                                            Qt6::Test)
  if(PANTA_ENABLE_BRIDGE_MODULE)
    target_link_libraries(panta_qml_shell_module_test PRIVATE panta_bridgeplugin)
    target_compile_definitions(panta_qml_shell_module_test PRIVATE PANTA_ENABLE_BRIDGE_MODULE=1)
  endif()
  panta_add_qml_test(Qml.ShellModuleLoads panta_qml_shell_module_test)

  # 视口模块加载测试（任务 007）：只验证 QML 注册与类型可创建，渲染线程
  # 管线不在无头环境触发（VTK 对软件 scenegraph 会显式报错，见 vtk.md）。
  add_executable(panta_qml_viewport_module_test ../../tests/cpp/app/viewport_module_load_test.cpp)
  panta_native_defaults(panta_qml_viewport_module_test)
  target_link_libraries(
    panta_qml_viewport_module_test PRIVATE panta::visualization panta_visualizationplugin Qt6::Gui
                                           Qt6::Qml Qt6::Test)
  panta_add_qml_test(Qml.ViewportModuleLoads panta_qml_viewport_module_test)
endif()

install(
  TARGETS panta_native_app
  EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin)
