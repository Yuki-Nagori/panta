# 桌面入口（任务 005）：链接 QML 模块与 ViewModel 模块。
add_executable(panta_native_app main.cpp)
set_target_properties(panta_native_app PROPERTIES OUTPUT_NAME panta-native EXPORT_NAME native-app)
panta_native_defaults(panta_native_app)
target_link_libraries(
  panta_native_app
  PRIVATE panta::foundation
          panta::shell
          panta::i18n
          "${PANTA_FFI_STATIC_LIB}"
          Qt6::Gui
          Qt6::Qml
          Qt6::Quick
          Qt6::QuickControls2
          Qt6::QuickDialogs2)
target_include_directories(
  panta_native_app PRIVATE "${PANTA_FFI_INCLUDE_DIR}"
                           "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include")
if(UNIX)
  find_package(Threads REQUIRED)
  target_link_libraries(panta_native_app PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
elseif(WIN32)
  target_link_libraries(panta_native_app PRIVATE ws2_32 userenv ntdll)
  # 真实窗口验收直接裸启 exe（AGENTS 验收流程）：链接后把运行库部署到
  # exe 目录，不依赖手工 PATH。Qt 系由托管 windeployqt 覆盖（DLL、平台/
  # QML 插件与 Qt QML 模块；qmldir 指向仓库 QML 源供推导 Qt 模块集——
  # app 目标没有可扫描的源文件，见上方说明；Windows 的 qmlimportscanner
  # 不依赖平台 ICU）；VTK SDK 为共享库，整 bin 目录随包拷贝，新增 DLL
  # 无需改本文件。
  find_program(
    PANTA_WINDEPLOYQT_EXECUTABLE
    NAMES windeployqt
    PATHS "${QT_STAGING}/bin"
    NO_DEFAULT_PATH REQUIRED)
  panta_require_sdk("vtk" _panta_app_vtk_staging)
  add_custom_command(
    TARGET panta_native_app POST_BUILD
    COMMAND
      "${PANTA_WINDEPLOYQT_EXECUTABLE}" --no-translations --qmldir
      "${CMAKE_CURRENT_SOURCE_DIR}/../../qml" "$<TARGET_FILE:panta_native_app>"
    COMMENT "部署 Qt 运行库到 panta-native.exe 目录"
    VERBATIM)
  add_custom_command(
    TARGET panta_native_app POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy_directory "${_panta_app_vtk_staging}/bin"
            "$<TARGET_FILE_DIR:panta_native_app>"
    COMMENT "拷贝 VTK SDK 运行库到 panta-native.exe 目录"
    VERBATIM)
  # Dawn 的 HLSL 编译（native/d3d/PlatformFunctions.cpp EnsureFXC）要求
  # d3dcompiler_47.dll 在应用目录：仅存在 System32 副本时其 LoadLibraryEx
  # 报 Error 87、WebGPU 设备创建失败并崩溃；Windows 10/11 系统目录必有。
  add_custom_command(
    TARGET panta_native_app POST_BUILD
    COMMAND
      "${CMAKE_COMMAND}" -E copy "$ENV{windir}/System32/d3dcompiler_47.dll"
      "$<TARGET_FILE_DIR:panta_native_app>"
    COMMENT "拷贝 d3dcompiler_47.dll 到 panta-native.exe 目录"
    VERBATIM)
endif()
if(PANTA_ENABLE_BRIDGE_MODULE)
  # main.cpp 的进程级 crash handler 已在上方链接 FFI；这里仅追加 Bridge/VTK
  # 的插件和渲染依赖，关闭 Bridge 时不会把可选模块带入 app。
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
  target_link_libraries(
    panta_qml_shell_module_test
    PRIVATE panta::shell
            Qt6::Gui
            Qt6::Qml
            Qt6::Quick
            Qt6::QuickControls2
            Qt6::QuickDialogs2
            Qt6::Test)
  if(PANTA_ENABLE_BRIDGE_MODULE)
    target_link_libraries(
      panta_qml_shell_module_test PRIVATE panta::bridge panta::visualization panta_bridgeplugin
                                          panta_visualizationplugin)
    target_compile_definitions(panta_qml_shell_module_test PRIVATE PANTA_ENABLE_BRIDGE_MODULE=1)
  endif()
  panta_add_qml_test(Qml.ShellModuleLoads panta_qml_shell_module_test)

  if(PANTA_ENABLE_BRIDGE_MODULE)
    # 默认验证模块加载与条目析构；真实 GPU 生命周期用例需显式启用，
    # 不在 offscreen 环境触发（见性能模块说明）。
    add_executable(panta_qml_viewport_module_test ../../tests/cpp/app/viewport_module_load_test.cpp)
    panta_native_defaults(panta_qml_viewport_module_test)
    target_include_directories(panta_qml_viewport_module_test PRIVATE ../visualization/src/vtk)
    target_link_libraries(
      panta_qml_viewport_module_test PRIVATE panta::visualization panta_visualizationplugin
                                             Qt6::Gui Qt6::Qml Qt6::Test)
    panta_add_qml_test(Qml.ViewportModuleLoads panta_qml_viewport_module_test)
  endif()
endif()

install(
  TARGETS panta_native_app
  EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin)
