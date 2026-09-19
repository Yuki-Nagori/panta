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
  target_link_libraries(panta_native_app PRIVATE panta_bridgeplugin)
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
  if(WIN32)
    # Keep the QML module DLL beside the test executable. This is also the
    # first Windows DLL search location when CTest starts a GUI process.
    add_custom_command(
      TARGET panta_qml_shell_module_test
      POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different "$<TARGET_FILE:panta_shell>"
              "$<TARGET_FILE_DIR:panta_qml_shell_module_test>"
      VERBATIM)
  endif()
  add_test(NAME Qml.ShellModuleLoads COMMAND panta_qml_shell_module_test)
  set_tests_properties(
    Qml.ShellModuleLoads
    PROPERTIES
      ENVIRONMENT
      "QT_QPA_PLATFORM=${PANTA_QML_TEST_QPA_PLATFORM};QT_QUICK_CONTROLS_STYLE=Basic;QT_PLUGIN_PATH=${QT_STAGING}/plugins;QT_QPA_PLATFORM_PLUGIN_PATH=${QT_STAGING}/plugins/platforms"
  )
endif()

install(
  TARGETS panta_native_app
  EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin)
