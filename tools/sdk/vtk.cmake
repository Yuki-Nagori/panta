# VTK 9.7.0 制品构建描述（任务 038）。
#
# 源码固定：tag v9.7.0 → commit 23f0a095621e91bbdbeace8451e22b950c8e5f46
# （2026-09-17 git ls-remote 解引用复核；annotated tag 对象 a78e2d95…）。
# 配置开关冻结为 007 视口所需最小面：VTK WebGPU + 平台 hardware window
# （macOS 为 Cocoa hardware window）；不启用 Qt/GUISupportQtQuick，不把 Qt
# OpenGL scenegraph 集成带入制品。
# 许可证随源码树收集。

set(PANTA_VTK_VERSION 9.7.0)
set(PANTA_VTK_COMMIT 23f0a095621e91bbdbeace8451e22b950c8e5f46)
set(PANTA_VTK_INSTALL_DIR "${PANTA_SDK_OUT_ROOT}/vtk/${PANTA_VTK_VERSION}/${PANTA_SDK_TRIPLE}")

include(ExternalProject)
ExternalProject_Add(
  vtk_sdk
  GIT_REPOSITORY https://gitlab.kitware.com/vtk/vtk.git
  GIT_TAG ${PANTA_VTK_COMMIT}
  GIT_SHALLOW TRUE
  GIT_PROGRESS TRUE
  # 子工程继承外层生成器（macOS/Linux 为 Ninja；Windows CI 传 Visual
  # Studio 17 2022 + x64，Ninja 子构建在无 vcvars 的 runner 上找不到 cl）。
  # 多配置生成器的构建/安装显式选 Release，单配置忽略 --config。
  BUILD_COMMAND ${CMAKE_COMMAND} --build . --config Release
  CMAKE_ARGS -DCMAKE_BUILD_TYPE=Release
             -DBUILD_SHARED_LIBS=ON
             -DBUILD_TESTING=OFF
             -DCMAKE_INSTALL_PREFIX=${PANTA_VTK_INSTALL_DIR}
             -DVTK_GROUP_ENABLE_Qt=NO
             -DVTK_ENABLE_WEBGPU=ON
             -DVTK_MODULE_ENABLE_VTK_RenderingUI=YES
             -DVTK_MODULE_ENABLE_VTK_RenderingWebGPU=YES
  # 安装后补齐统一布局的非构建产物：许可证与 SDK 元数据。
  INSTALL_COMMAND ${CMAKE_COMMAND} --install . --config Release
  COMMAND ${CMAKE_COMMAND} -E make_directory ${PANTA_VTK_INSTALL_DIR}/share/licenses/VTK
  COMMAND ${CMAKE_COMMAND} -E copy_if_different <SOURCE_DIR>/Copyright.txt
          ${PANTA_VTK_INSTALL_DIR}/share/licenses/VTK/Copyright.txt
  USES_TERMINAL_DOWNLOAD TRUE
  USES_TERMINAL_BUILD TRUE)

# panta-sdk.json：031 供给自检与 012 缓存键的机器可读依据。
file(
  WRITE "${CMAKE_BINARY_DIR}/vtk-panta-sdk.json.in"
  [=[
{
  "name": "vtk",
  "version": "@PANTA_VTK_VERSION@",
  "triple": "@PANTA_SDK_TRIPLE@",
  "source": {
    "repository": "https://gitlab.kitware.com/vtk/vtk.git",
    "tag": "v9.7.0",
    "commit": "@PANTA_VTK_COMMIT@"
  },
  "build_type": "Release",
  "shared": true,
  "graphics_backend": "WebGPU",
  "window_system": "VTK hardware window",
  "macos_surface": "CocoaHardwareWindow",
  "modules_highlights": ["RenderingWebGPU", "RenderingUI", "RenderingCore"],
  "cmake_package": ["lib/cmake/vtk-@PANTA_VTK_VERSION@", "vtk-config.cmake"],
  "generator": "@CMAKE_GENERATOR@",
  "license": "share/licenses/VTK/Copyright.txt (BSD-3)"
}
]=])
configure_file("${CMAKE_BINARY_DIR}/vtk-panta-sdk.json.in"
               "${CMAKE_BINARY_DIR}/vtk-panta-sdk-configure.json" @ONLY)
ExternalProject_Add_Step(
  vtk_sdk metadata
  COMMAND ${CMAKE_COMMAND} -E copy_if_different ${CMAKE_BINARY_DIR}/vtk-panta-sdk-configure.json
          ${PANTA_VTK_INSTALL_DIR}/panta-sdk.json
  DEPENDEES install)
