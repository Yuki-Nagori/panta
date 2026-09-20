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
  src/shell_view_model.hpp
  src/shell_view_model.cpp
  src/task_host.hpp
  src/task_host.cpp
  src/path_host.hpp
  src/path_host.cpp)
panta_native_defaults(panta_bridge)
# 测试和 C++ 消费者按 <shell_view_model.hpp> 引用头文件。
target_include_directories(panta_bridge PUBLIC src)
# TaskHost 消费 Rust TaskService：生成头路径与其引用的手写支持头
# （panta/ffi.hpp）公开给消费者，staticlib 及其系统库依赖随 target
# 传播到最终链接者（与 native/ffi 测试同规则）。
target_include_directories(panta_bridge PUBLIC "${PANTA_FFI_INCLUDE_DIR}"
                                               "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include")
target_link_libraries(panta_bridge PUBLIC Qt6::Core "${PANTA_FFI_STATIC_LIB}")
if(UNIX)
  find_package(Threads REQUIRED)
  target_link_libraries(panta_bridge PUBLIC Threads::Threads ${CMAKE_DL_LIBS})
elseif(WIN32)
  # Rust std 需要的 Windows 系统库不由 CMake target 传播，在此显式声明。
  target_link_libraries(panta_bridge PUBLIC ws2_32 userenv ntdll)
endif()

if(BUILD_TESTING)
  include(GoogleTest)
  add_executable(panta_bridge_shell_view_model_test
                 ../../tests/cpp/bridge/shell_view_model_test.cpp)
  panta_native_defaults(panta_bridge_shell_view_model_test)
  target_link_libraries(panta_bridge_shell_view_model_test PRIVATE panta::bridge GTest::gtest
                                                                   Qt6::Test)
  # POST_BUILD 发现会在链接后立即启动测试可执行文件：Cargo 统一入口由
  # launcher 构建脚本给构建子进程注入托管 Qt runtime PATH（Windows 必需）。
  # 不用 PRE_TEST 发现——其测试清单依赖 CMake 4.4 的 discover_tests 内建
  # 命令，PATH 上较旧的 ctest 无法消费。
  gtest_discover_tests(panta_bridge_shell_view_model_test DISCOVERY_MODE POST_BUILD)

  add_executable(panta_bridge_task_host_test ../../tests/cpp/bridge/task_host_test.cpp)
  panta_native_defaults(panta_bridge_task_host_test)
  target_link_libraries(panta_bridge_task_host_test PRIVATE panta::bridge GTest::gtest Qt6::Test)
  gtest_discover_tests(panta_bridge_task_host_test DISCOVERY_MODE POST_BUILD)

  # 路径宿主（023）覆盖标准目录注入、cwd 无关、拒绝矩阵、file URL 与
  # 非 Unicode 策略；symlink 用例仅在 Unix 编译（Windows junction 记录待测）。
  add_executable(panta_bridge_path_host_test ../../tests/cpp/bridge/path_host_test.cpp)
  panta_native_defaults(panta_bridge_path_host_test)
  target_link_libraries(panta_bridge_path_host_test PRIVATE panta::bridge GTest::gtest Qt6::Test)
  gtest_discover_tests(panta_bridge_path_host_test DISCOVERY_MODE POST_BUILD)
endif()
