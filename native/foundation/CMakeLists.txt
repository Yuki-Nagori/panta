# 构建链验证与 native 基础契约的最小目标；业务模块（geometry/mesh/…）
# 由后续任务按 repository-layout 增加，不在此预建占位目录。
add_library(panta_foundation)
add_library(panta::foundation ALIAS panta_foundation)

target_sources(panta_foundation PRIVATE src/version.cpp)

# BUILD/INSTALL 双接口：源码树绝对路径不可进入导出集，安装后改用相对 include。
target_include_directories(
  panta_foundation PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                          $<INSTALL_INTERFACE:include>)

# 导出名用短名：消费者链接 panta::foundation；构建树 target 仍为 panta_foundation。
set_target_properties(panta_foundation PROPERTIES EXPORT_NAME foundation)
panta_native_defaults(panta_foundation)

# 版本号单一来源是顶层 project(VERSION)，测试期望值同样由此注入。
target_compile_definitions(
  panta_foundation
  PRIVATE PANTA_NATIVE_VERSION_MAJOR=${PROJECT_VERSION_MAJOR}
          PANTA_NATIVE_VERSION_MINOR=${PROJECT_VERSION_MINOR}
          PANTA_NATIVE_VERSION_PATCH=${PROJECT_VERSION_PATCH})

# 测试注册由 gtest_discover_tests 承担：每个用例成为独立 CTest 项（gtest.md）。
# include(GoogleTest) 模块随 CMake 提供；链接 gtest_main 复用其 main()。
if(BUILD_TESTING)
  include(GoogleTest)
  add_executable(panta_foundation_version_test ../../tests/cpp/foundation/version_test.cpp)
  panta_native_defaults(panta_foundation_version_test)
  target_link_libraries(panta_foundation_version_test PRIVATE panta::foundation GTest::gtest_main)
  target_compile_definitions(
    panta_foundation_version_test
    PRIVATE PANTA_TEST_EXPECT_MAJOR=${PROJECT_VERSION_MAJOR}
            PANTA_TEST_EXPECT_MINOR=${PROJECT_VERSION_MINOR}
            PANTA_TEST_EXPECT_PATCH=${PROJECT_VERSION_PATCH})
  # 多配置生成器（Windows Visual Studio）在构建阶段可能尚未准备好 Qt
  # runtime；延后发现用例到 ctest，避免 configure/build 被测试启动阻断。
  gtest_discover_tests(panta_foundation_version_test DISCOVERY_MODE PRE_TEST)
endif()

install(
  TARGETS panta_foundation
  EXPORT pantaNativeTargets
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION bin)
install(DIRECTORY include/ DESTINATION include)
