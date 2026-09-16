# C++ ViewModel 静态库。
# 注册方式偏离 qt.md 的"模块化优先"：Panta.Shell 为 NO_PLUGIN 纯资源模块，
# 跨静态 QML 模块插件的链接依赖 qmlimportscanner 扫描 exe 自身 QML，与
# "App.qml 独立于 exe 目录（repository-layout 的根 qml/）"冲突，故在
# native/app/main.cpp 中用 qmlRegisterType 显式注册（任务 005 决策记录）。
add_library(panta_bridge STATIC)
add_library(panta::bridge ALIAS panta_bridge)

target_sources(panta_bridge PRIVATE
  src/shell_view_model.hpp
  src/shell_view_model.cpp
)
panta_native_defaults(panta_bridge)
# main.cpp 的显式注册与测试按 <shell_view_model.hpp> 引用头文件。
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
