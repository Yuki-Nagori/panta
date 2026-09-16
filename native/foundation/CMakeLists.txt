# 构建链验证与 native 基础契约的最小目标；业务模块（geometry/mesh/…）
# 由后续任务按 repository-layout 增加，不在此预建占位目录。
add_library(panta_foundation)
add_library(panta::foundation ALIAS panta_foundation)

target_sources(panta_foundation PRIVATE src/version.cpp)

# BUILD/INSTALL 双接口：源码树绝对路径不可进入导出集，安装后改用相对 include。
target_include_directories(panta_foundation PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:include>
)

# PUBLIC 的 cxx_std_20 是对消费者的等价 C++20 约束（cpp.md 允许的形式）。
target_compile_features(panta_foundation PUBLIC cxx_std_20)

set_target_properties(panta_foundation PROPERTIES
  CXX_EXTENSIONS OFF
  # 导出名用短名：消费者链接 panta::foundation；构建树 target 仍为 panta_foundation。
  EXPORT_NAME foundation
)

# 告警只作用于自有 target，不经 PUBLIC 泄漏给消费者。
target_compile_options(panta_foundation PRIVATE
  $<$<CXX_COMPILER_ID:MSVC>:/W4 /permissive->
  $<$<NOT:$<CXX_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic>
)

# 版本号单一来源是顶层 project(VERSION)，测试期望值同样由此注入。
target_compile_definitions(panta_foundation PRIVATE
  PANTA_NATIVE_VERSION_MAJOR=${PROJECT_VERSION_MAJOR}
  PANTA_NATIVE_VERSION_MINOR=${PROJECT_VERSION_MINOR}
  PANTA_NATIVE_VERSION_PATCH=${PROJECT_VERSION_PATCH}
)

# 测试注册由 gtest_discover_tests 承担：每个用例成为独立 CTest 项（gtest.md）。
# include(GoogleTest) 模块随 CMake 提供；链接 gtest_main 复用其 main()。
if(BUILD_TESTING)
  include(GoogleTest)
  add_executable(panta_foundation_version_test tests/version_test.cpp)
  target_compile_features(panta_foundation_version_test PRIVATE cxx_std_20)
  set_target_properties(panta_foundation_version_test PROPERTIES CXX_EXTENSIONS OFF)
  target_compile_options(panta_foundation_version_test PRIVATE
    $<$<CXX_COMPILER_ID:MSVC>:/W4 /permissive->
    $<$<NOT:$<CXX_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic>
  )
  target_link_libraries(panta_foundation_version_test PRIVATE
    panta::foundation
    GTest::gtest_main
  )
  target_compile_definitions(panta_foundation_version_test PRIVATE
    PANTA_TEST_EXPECT_MAJOR=${PROJECT_VERSION_MAJOR}
    PANTA_TEST_EXPECT_MINOR=${PROJECT_VERSION_MINOR}
    PANTA_TEST_EXPECT_PATCH=${PROJECT_VERSION_PATCH}
  )
  gtest_discover_tests(panta_foundation_version_test)
endif()

install(TARGETS panta_foundation EXPORT pantaNativeTargets
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION bin
)
install(DIRECTORY include/ DESTINATION include)
