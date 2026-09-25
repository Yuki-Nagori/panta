# native 层版本须与根 Cargo.toml 的 workspace.package.version 保持一致；
# 任务 004 的 launcher 诊断将依赖这一致性。3.22 是本树用到的 presets
# schema 3（build/test presets）所需的最低版本，不随在线文档的最新版浮动。
cmake_minimum_required(VERSION 3.22)
project(
  panta_native
  VERSION 0.1.0
  LANGUAGES C CXX)

option(PANTA_ENABLE_FFI_TEST "Build the CXX Rust/C++ boundary smoke test supplied by Cargo" OFF)
option(PANTA_ENABLE_BRIDGE_MODULE "Build and register the Panta.Bridge QML module" ON)
option(PANTA_ENABLE_COVERAGE "Instrument native C++ targets for LLVM coverage" OFF)
option(PANTA_USE_SYSTEM_TOOLS "Use explicitly selected host tools instead of Cargo-managed tools"
       OFF)
set(PANTA_LLVM_VERSION
    ""
    CACHE STRING "Expected managed LLVM version")

include(cmake/build-policy.cmake)

# include(CTest) 定义 BUILD_TESTING 选项（默认 ON）并启用测试；OFF 时不供给
# GoogleTest SDK、不注册测试，供 004 的发布链关闭测试面。
include(CTest)

# 输入错误应在依赖下载之前报告。
if(PANTA_ENABLE_FFI_TEST)
  if(NOT BUILD_TESTING)
    message(FATAL_ERROR "PANTA_ENABLE_FFI_TEST requires BUILD_TESTING=ON")
  endif()
  if(NOT EXISTS "${PANTA_FFI_INCLUDE_DIR}/panta_ffi.h"
     OR NOT EXISTS "${PANTA_FFI_STATIC_LIB}"
     OR IS_DIRECTORY "${PANTA_FFI_STATIC_LIB}")
    message(
      FATAL_ERROR
        "PANTA_ENABLE_FFI_TEST requires existing PANTA_FFI_INCLUDE_DIR/panta_ffi.h and PANTA_FFI_STATIC_LIB; build via Cargo"
    )
  endif()
endif()

# Qt 预编译产物供给（任务 005）：缓存于构建树，首次 configure 需联网。
# QT_STAGING 暴露路径供 find_package 和诊断使用。
include(cmake/qt-provision.cmake)
set(_panta_qt_components
    Core
    Gui
    Qml
    Quick
    QuickControls2
    QuickDialogs2
    Svg)
if(BUILD_TESTING)
  list(APPEND _panta_qt_components Test)
endif()
find_package(Qt6 6.11 REQUIRED COMPONENTS ${_panta_qt_components} PATHS "${QT_STAGING}"
                                          NO_DEFAULT_PATH)
qt_standard_project_setup()
qt_policy(SET QTP0001 NEW)
qt_policy(SET QTP0004 NEW)
qt_policy(SET QTP0005 NEW)

# 预编译 SDK 供给（任务 031）：manifest 登记与 staging 供给 API；VTK/OCCT/
# Netgen 供应用例引擎，GoogleTest 仅供 BUILD_TESTING；固定制品见
# dependency-acquisition.md，不回退系统库或源码获取。
include(cmake/sdk-provision.cmake)

if(BUILD_TESTING)
  add_test(
    NAME Build.MsvcAbiPolicy
    COMMAND
      "${CMAKE_COMMAND}" "-DTEST_BINARY_DIR=${CMAKE_CURRENT_BINARY_DIR}/abi-policy-test"
      "-DTEST_CXX_COMPILER=${CMAKE_CXX_COMPILER}" "-DTEST_MAKE_PROGRAM=${CMAKE_MAKE_PROGRAM}" -P
      "${CMAKE_CURRENT_SOURCE_DIR}/cmake/tests/check-abi.cmake")
  # SDK 供给逻辑（031）用 fixture 归档驱动：不联网（file://）、不编译
  # （consumer 为 project NONE），三平台 CI 与本地同路径。
  add_test(
    NAME Build.SdkProvision
    COMMAND
      "${CMAKE_COMMAND}" "-DTEST_BINARY_DIR=${CMAKE_CURRENT_BINARY_DIR}/sdk-provision-test"
      "-DTEST_MAKE_PROGRAM=${CMAKE_MAKE_PROGRAM}"
      "-DPANTA_SDK_MODULE=${CMAKE_CURRENT_SOURCE_DIR}/cmake/sdk-provision.cmake"
      "-DCONSUMER_SOURCE_DIR=${CMAKE_CURRENT_SOURCE_DIR}/cmake/tests/sdk/consumer" -P
      "${CMAKE_CURRENT_SOURCE_DIR}/cmake/tests/check-sdk-provision.cmake")
  # GoogleTest v1.18.0 通过固定 manifest 下载并缓存到共享 SDK 根目录。
  # 只消费 gtest targets；gmock 待模块边界场景出现时按对应 task 打开。
  panta_require_sdk(googletest PANTA_GOOGLETEST_STAGING)
endif()

add_subdirectory(foundation)
# QM 编译与资源入口（任务 034）：依赖 Qt 供给的 lrelease 与 Cargo 生成的 TS。
add_subdirectory(i18n)
# OCCT STEP 适配冒烟（任务 009）：经 031 固定供给消费 OpenCASCADE SDK；
# 无 Qt/QML 依赖，业务任务接入前仅由自身 gtest 消费。
add_subdirectory(geometry)
# Mesh IR 与 Netgen 适配（任务 010）：netgen 侧依赖 OCCT targets，
# 在 geometry 之后加入构建图。
add_subdirectory(mesh)
# lint 的 moc 前置聚合（任务 082）：EXCLUDE_FROM_ALL 的手动基准目标不进默认
# 构建，但 clang-tidy / cppcheck 的编译数据库引用其 AUTOMOC 产物；各基准
# 定义处必须把对应 _autogen 目标登记到 panta_benchmark_moc，否则 standalone
# lint 在未构建基准的工作树上缺 moc 失败。
if(BUILD_TESTING)
  add_custom_target(panta_benchmark_moc COMMENT "生成手动基准 AUTOMOC 产物")
endif()
# VTK 原生视口核心（任务 007）：bridge 经 Panta.Bridge 注册 CaeViewport；
# 诊断构建（AppNoBridge）不带 VTK。
if(PANTA_ENABLE_BRIDGE_MODULE)
  add_subdirectory(visualization)
  add_subdirectory(bridge)
endif()
# qml/ 位于仓库根（repository-layout）；二进制目录固定在 native 构建树内。
add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../qml" qml)
add_subdirectory(app)

if(PANTA_ENABLE_FFI_TEST)
  add_subdirectory(ffi)
endif()

set(PANTA_NATIVE_INSTALL_CONFIG_DIR lib/cmake/panta-native)

include(CMakePackageConfigHelpers)
configure_package_config_file(
  cmake/panta-native-config.cmake.in "${CMAKE_BINARY_DIR}/cmake/panta-native-config.cmake"
  INSTALL_DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}")
write_basic_package_version_file("${CMAKE_BINARY_DIR}/cmake/panta-native-config-version.cmake"
                                 COMPATIBILITY SameMajorVersion)

install(
  EXPORT pantaNativeTargets
  FILE panta-native-targets.cmake
  NAMESPACE panta::
  DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}")
install(FILES "${CMAKE_BINARY_DIR}/cmake/panta-native-config.cmake"
              "${CMAKE_BINARY_DIR}/cmake/panta-native-config-version.cmake"
        DESTINATION "${PANTA_NATIVE_INSTALL_CONFIG_DIR}")

# 等 Qt 的延迟目标创建完成后审计，包括 plugin_init/resource object targets。
cmake_language(DEFER CALL panta_verify_msvc_abi "${CMAKE_CURRENT_SOURCE_DIR}")
