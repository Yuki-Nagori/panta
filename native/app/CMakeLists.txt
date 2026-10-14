# 桌面入口（任务 005）：链接 QML 模块与 ViewModel 模块。
qt_add_executable(panta_native_app main.cpp)
set_target_properties(panta_native_app PROPERTIES OUTPUT_NAME panta EXPORT_NAME native-app)
if(APPLE)
  # Qt 的 macOS 安装部署入口要求标准 .app 布局；开发期也从 bundle 内启动，
  # 让 cargo run 与安装后启动使用相同的可执行文件相对资源结构。
  set_target_properties(
    panta_native_app
    PROPERTIES MACOSX_BUNDLE TRUE
               MACOSX_BUNDLE_GUI_IDENTIFIER "com.yuki-nagori.panta"
               MACOSX_BUNDLE_BUNDLE_NAME "Panta")
endif()
panta_native_defaults(panta_native_app)
target_link_libraries(
  panta_native_app
  PRIVATE panta::foundation
          panta::shell
          panta_shellplugin
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
  # 真实窗口验收直接裸启 exe（AGENTS 验收流程）：部署机制与覆盖范围见
  # cmake/windows-app-runtime-deployment.cmake 头注。
  panta_deploy_windows_app_runtime(panta_native_app "${CMAKE_CURRENT_SOURCE_DIR}/../../qml")
endif()
if(PANTA_ENABLE_BRIDGE_MODULE)
  # main.cpp 的进程级 crash handler 已在上方链接 FFI；这里仅追加 Bridge/VTK
  # 的插件和渲染依赖，关闭 Bridge 时不会把可选模块带入 app。
  target_link_libraries(panta_native_app PRIVATE panta::visualization)
  target_link_libraries(panta_native_app PRIVATE panta_bridgeplugin panta_visualizationplugin)
  target_compile_definitions(panta_native_app PRIVATE PANTA_ENABLE_BRIDGE_MODULE=1)
  set(_panta_entry "${CMAKE_CURRENT_SOURCE_DIR}/../../qml/AppEntry.qml")
else()
  set(_panta_entry "${CMAKE_CURRENT_SOURCE_DIR}/../../qml/AppNoBridgeEntry.qml")
endif()
get_filename_component(_panta_alias "${_panta_entry}" NAME)
set_source_files_properties("${_panta_entry}" PROPERTIES QT_RESOURCE_ALIAS "${_panta_alias}")

# 应用入口包装 Shell 的实际页面。独立应用模块让 Qt 部署扫描从应用目标追踪
# Shell 及 QtQuick imports，不需要另外维护一份手写的 QML 依赖清单。
qt_add_qml_module(
  panta_native_app
  URI
  Panta.App
  VERSION
  1.0
  RESOURCE_PREFIX
  /qt/qml
  DEPENDENCIES
  TARGET
  panta_shell
  QML_FILES
  "${_panta_entry}")

if(BUILD_TESTING)
  add_executable(panta_qml_shell_module_test ../../tests/cpp/app/shell_module_load_test.cpp)
  panta_native_defaults(panta_qml_shell_module_test)
  target_link_libraries(
    panta_qml_shell_module_test
    PRIVATE panta::shell
            panta_shellplugin
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
  RUNTIME_DEPENDENCY_SET panta_native_runtime_dependencies
  BUNDLE DESTINATION . COMPONENT Runtime
  RUNTIME DESTINATION bin COMPONENT Runtime)

# QML 模块已静态编入程序；安装时只需复制扫描所得的运行期 QML imports 与插件。
# 动态库由 CMake dependency set 按 SDK 搜索路径解析并复制，确保 app 实际链接的
# native 依赖离开开发机路径后仍可加载。
qt_generate_deploy_qml_app_script(TARGET panta_native_app OUTPUT_SCRIPT
                                  panta_native_qml_deploy_script NO_TRANSLATIONS)
if(APPLE)
  install(
    RUNTIME_DEPENDENCY_SET
    panta_native_runtime_dependencies
    DIRECTORIES
    ${PANTA_RUNTIME_DEPENDENCY_DIRS}
    POST_EXCLUDE_REGEXES
    "^/System/Library/.*"
    "^/usr/lib/.*"
    "^${QT_STAGING}/.*"
    FRAMEWORK
    DESTINATION
    panta.app/Contents/Frameworks
    COMPONENT
    Runtime
    LIBRARY
    DESTINATION
    panta.app/Contents/Frameworks
    COMPONENT
    Runtime)
elseif(WIN32)
  install(
    RUNTIME_DEPENDENCY_SET
    panta_native_runtime_dependencies
    DIRECTORIES
    ${PANTA_RUNTIME_DEPENDENCY_DIRS}
    POST_EXCLUDE_REGEXES
    "^[A-Za-z]:[/\\\\]Windows[/\\\\].*"
    "^${QT_STAGING}/.*"
    RUNTIME
    DESTINATION
    bin
    COMPONENT
    Runtime)
  # Dawn 动态加载 D3DCompiler，依赖扫描无法发现它；沿用开发部署的系统 DLL 来源。
  install(
    FILES "$ENV{windir}/System32/d3dcompiler_47.dll"
    DESTINATION bin
    COMPONENT Runtime)
else()
  install(
    RUNTIME_DEPENDENCY_SET
    panta_native_runtime_dependencies
    DIRECTORIES
    ${PANTA_RUNTIME_DEPENDENCY_DIRS}
    POST_EXCLUDE_REGEXES
    "^/lib/.*"
    "^/lib64/.*"
    "^/usr/lib/.*"
    "^/usr/lib64/.*"
    "^${QT_STAGING}/.*"
    LIBRARY
    DESTINATION
    lib
    COMPONENT
    Runtime)
endif()

# macdeployqt 需要看到先复制进 bundle 的 native SDK 库，才能修正它们及 app
# 的 @rpath 引用；因此 Qt 部署步骤放在 runtime dependency set 之后执行。
install(SCRIPT "${panta_native_qml_deploy_script}" COMPONENT Runtime)
