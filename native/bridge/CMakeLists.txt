# C++ ViewModel 静态库，同时作为 Panta.Bridge QML 模块的 backing target。
# qt_add_qml_module 生成 typeinfo 和静态 plugin，应用通过 Panta.Shell 的
# TARGET 依赖导入 plugin；不在 main.cpp 里维护第二套手动注册入口。
add_library(panta_bridge STATIC)
add_library(panta::bridge ALIAS panta_bridge)

qt_add_qml_module(panta_bridge
  URI Panta.Bridge
  VERSION 1.0
  OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/Panta/Bridge"
  SOURCES
  src/shell_view_model.hpp
  src/shell_view_model.cpp
)
panta_native_defaults(panta_bridge)
# 测试和 C++ 消费者按 <shell_view_model.hpp> 引用头文件。
target_include_directories(panta_bridge PUBLIC src)
target_link_libraries(panta_bridge PUBLIC Qt6::Core)

if(BUILD_TESTING)
  include(GoogleTest)
  add_executable(panta_bridge_shell_view_model_test tests/shell_view_model_test.cpp)
  panta_native_defaults(panta_bridge_shell_view_model_test)
  target_link_libraries(panta_bridge_shell_view_model_test PRIVATE
    panta::bridge
    GTest::gtest
    Qt6::Test
  )
  gtest_discover_tests(panta_bridge_shell_view_model_test)
endif()
