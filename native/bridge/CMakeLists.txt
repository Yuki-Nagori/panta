# C++ ViewModel 静态库，同时作为 Panta.Bridge QML 模块的 backing target。
# qt_add_qml_module 生成 typeinfo 和静态 plugin，应用通过 Panta.Shell 的
# TARGET 依赖导入 plugin；不在 main.cpp 里维护第二套手动注册入口。
if(UNIX AND NOT APPLE)
  # Qt Linux 预编译工具的 qtpaths 依赖其构建时 ICU ABI；当前官方归档面向
  # RHEL9，而 Ubuntu runner 没有同版本运行库。qmlls.build.ini 只是 IDE 辅助
  # 文件，不影响 typeinfo、qmlcachegen 或运行时，因此在 Linux 跳过它。
  set(PANTA_QML_MODULE_OPTIONS __QT_INTERNAL_NO_GENERATE_QMLLS_BUILD_INI)
endif()

add_library(panta_bridge STATIC)
add_library(panta::bridge ALIAS panta_bridge)

qt_add_qml_module(
  panta_bridge
  URI
  Panta.Bridge
  VERSION
  1.0
  OUTPUT_DIRECTORY
  "${CMAKE_BINARY_DIR}/Panta/Bridge"
  ${PANTA_QML_MODULE_OPTIONS}
  SOURCES
  src/import_interaction_controller.hpp
  src/import_interaction_controller.cpp
  src/settings.hpp
  src/settings.cpp
  src/path_host.hpp
  src/path_host.cpp
  src/project_view_model.hpp
  src/project_view_model.cpp)
panta_native_defaults(panta_bridge)
target_include_directories(panta_bridge PUBLIC src)
# Rust FFI 生成头与手写支持头（panta/ffi.hpp）公开给消费者；staticlib
# 及其系统库依赖随 target 传播到最终链接者。
target_include_directories(panta_bridge PUBLIC "${PANTA_FFI_INCLUDE_DIR}"
                                               "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include")
target_link_libraries(panta_bridge PUBLIC Qt6::Core Qt6::Qml Qt6::StateMachine panta::visualization
                                          panta::qt_adapter "${PANTA_FFI_STATIC_LIB}")
if(UNIX)
  find_package(Threads REQUIRED)
  target_link_libraries(panta_bridge PUBLIC Threads::Threads ${CMAKE_DL_LIBS})
elseif(WIN32)
  # Rust std 需要的 Windows 系统库不由 CMake target 传播，在此显式声明。
  target_link_libraries(panta_bridge PUBLIC ws2_32 userenv ntdll)
endif()

if(BUILD_TESTING)
  include(GoogleTest)
  # 路径宿主（023）覆盖标准目录注入、cwd 无关、拒绝矩阵、file URL 与
  # 非 Unicode 策略；symlink 用例仅在 Unix 编译（Windows junction 记录待测）。
  add_executable(panta_bridge_path_host_test ../../tests/cpp/bridge/path_host_test.cpp)
  panta_native_defaults(panta_bridge_path_host_test)
  target_link_libraries(panta_bridge_path_host_test PRIVATE panta::bridge GTest::gtest Qt6::Test)
  gtest_discover_tests(panta_bridge_path_host_test DISCOVERY_MODE POST_BUILD DISCOVERY_TIMEOUT 120
                       PROPERTIES LABELS bridge)

  add_executable(panta_bridge_project_view_model_test
                 ../../tests/cpp/bridge/project_view_model_test.cpp)
  panta_native_defaults(panta_bridge_project_view_model_test)
  target_include_directories(panta_bridge_project_view_model_test PRIVATE ../../tests/support/qt)
  target_link_libraries(panta_bridge_project_view_model_test PRIVATE panta::bridge GTest::gtest
                                                                     Qt6::Test)
  gtest_discover_tests(panta_bridge_project_view_model_test DISCOVERY_MODE POST_BUILD
                       DISCOVERY_TIMEOUT 120 PROPERTIES LABELS bridge)

  add_executable(panta_bridge_settings_test ../../tests/cpp/bridge/settings_test.cpp)
  panta_native_defaults(panta_bridge_settings_test)
  target_link_libraries(panta_bridge_settings_test PRIVATE panta::bridge panta::i18n Qt6::Test)
  # 该行为测试加载 QML 引擎，按 QML 测试注册：任务 042 已登记的 TSan 与
  # Windows ASan QML 第三方运行时边界会过滤 Qml.*；QtTest/add_test 也避免
  # 构建期 GTest discovery 在 Windows sanitizer 下提前启动 QML DLL。
  panta_add_qml_test(Qml.SettingsLanguageSwitch panta_bridge_settings_test bridge)
endif()
