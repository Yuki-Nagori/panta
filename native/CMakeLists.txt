# native 层版本须与根 Cargo.toml 的 workspace.package.version 保持一致；
# 任务 004 的 launcher 诊断将依赖这一致性。3.22 是本树用到的 presets
# schema 3（build/test presets）所需的最低版本，不随在线文档的最新版浮动。
cmake_minimum_required(VERSION 3.22)
project(panta_native VERSION 0.1.0 LANGUAGES CXX)

# enable_testing() 必须先于 add_subdirectory，使子目录的 add_test 生效。
enable_testing()
add_subdirectory(foundation)

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
