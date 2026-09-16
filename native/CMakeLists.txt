# native 层版本须与根 Cargo.toml 的 workspace.package.version 保持一致；
# 任务 004 的 launcher 诊断将依赖这一致性。3.22 是本树用到的 presets
# schema 3（build/test presets）所需的最低版本，不随在线文档的最新版浮动。
cmake_minimum_required(VERSION 3.22)
project(panta_native VERSION 0.1.0 LANGUAGES CXX)

# 默认值单一来源（cargo.md：Cargo 调度与直接诊断构建共用一套有效配置）：
# presets 与 crates/launcher/build.rs 只传显式差异项（构建类型），未指定时
# 用 Debug 与构建目录内 install，避免两处默认值漂移。
if(NOT DEFINED CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Debug CACHE STRING "构建类型（Debug/Release）" FORCE)
endif()
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install" CACHE PATH "安装前缀" FORCE)
endif()
if(NOT DEFINED CMAKE_EXPORT_COMPILE_COMMANDS)
  set(CMAKE_EXPORT_COMPILE_COMMANDS ON CACHE BOOL "导出 compile_commands.json" FORCE)
endif()

# 自有 target 公共约束（cpp.md）：C++20 必需（PUBLIC 向消费者传播）、无编译器
# 扩展、告警基线仅作用于自身。须在 add_executable/add_library 之后调用。
function(panta_native_defaults target)
  target_compile_features(${target} PUBLIC cxx_std_20)
  set_target_properties(${target} PROPERTIES CXX_EXTENSIONS OFF)
  target_compile_options(${target} PRIVATE
    $<$<CXX_COMPILER_ID:MSVC>:/W4 /permissive->
    $<$<NOT:$<CXX_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic>
  )
endfunction()

# include(CTest) 定义 BUILD_TESTING 选项（默认 ON）并启用测试；OFF 时不拉取
# googletest、不注册测试，供 004 的发布链关闭测试面。
include(CTest)
if(BUILD_TESTING)
  # GoogleTest 固定点见 ai-docs/standards/dependency-acquisition.md：
  # GIT_TAG 用完整 commit SHA（v1.18.0 → 063de7e9578f82b369302001269680b4b1553359）。
  # configure 阶段联网拉取到构建树，与 Cargo 托管原则一致（gtest.md）。
  include(FetchContent)
  FetchContent_Declare(googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG 063de7e9578f82b369302001269680b4b1553359
  )
  # Windows 下与自有 target 的 CRT 运行库保持一致，避免混链。
  set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
  # 只消费 gtest；gmock 待模块边界场景出现时按对应 task 打开。
  set(BUILD_GMOCK OFF CACHE BOOL "" FORCE)
  set(INSTALL_GTEST OFF CACHE BOOL "" FORCE)
  FetchContent_MakeAvailable(googletest)
endif()

add_subdirectory(foundation)
add_subdirectory(app)

set(PANTA_NATIVE_INSTALL_CONFIG_DIR lib/cmake/panta-native)

include(CMakePackageConfigHelpers)
configure_package_config_file(
  cmake/panta-native-config.cmake.in
  "${CMAKE_BINARY_DIR}/cmake/panta-native-config.cmake"
  INSTALL_DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}"
)
write_basic_package_version_file(
  "${CMAKE_BINARY_DIR}/cmake/panta-native-config-version.cmake"
  COMPATIBILITY SameMajorVersion
)

install(EXPORT pantaNativeTargets
  FILE panta-native-targets.cmake
  NAMESPACE panta::
  DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}"
)
install(FILES
  "${CMAKE_BINARY_DIR}/cmake/panta-native-config.cmake"
  "${CMAKE_BINARY_DIR}/cmake/panta-native-config-version.cmake"
  DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}"
)
