# VTK 9.7.0 制品构建描述（任务 038）。
#
# 源码固定：tag v9.7.0 → commit 23f0a095621e91bbdbeace8451e22b950c8e5f46
# （2026-09-17 git ls-remote 解引用复核；annotated tag 对象 a78e2d95…）。
# CI 从 GitHub mirror 抓取；该 mirror 的 v9.7.0^{} 与上游 pin 一致，避免
# runner 到 VTK 上游服务的源码 clone 受网络策略影响。
# 配置开关冻结为 007 视口所需最小面：VTK WebGPU + 平台 hardware window
# （macOS 为 Cocoa hardware window）；不启用 Qt/GUISupportQtQuick，不把 Qt
# OpenGL scenegraph 集成带入制品。
# 许可证随源码树收集。

set(PANTA_VTK_VERSION 9.7.0)
set(PANTA_VTK_COMMIT 23f0a095621e91bbdbeace8451e22b950c8e5f46)
set(PANTA_VTK_INSTALL_DIR "${PANTA_SDK_OUT_ROOT}/vtk/${PANTA_VTK_VERSION}/${PANTA_SDK_TRIPLE}")

# VTK 9.7 的 RenderingWebGPU 在桌面平台依赖 Dawn；只打开
# VTK_ENABLE_WEBGPU 而不提供 Dawn_DIR 会在 configure 阶段失败。Dawn 使用
# GitHub native release，避免依赖 VTK 上游的外部 package endpoint。
set(PANTA_DAWN_VERSION 20260720.160313)
set(PANTA_DAWN_COMMIT 0bc38adde72b79013536f8ce354b639ae19ae195)
set(PANTA_DAWN_LICENSE_SHA256 0493f897193af1796d5054659f45ec7d4c5af648fa67a99f01d30e55cc805abc)
set(PANTA_DAWN_LICENSE_URL
    "https://raw.githubusercontent.com/google/dawn/${PANTA_DAWN_COMMIT}/LICENSE")
if(PANTA_SDK_TRIPLE STREQUAL "macos-arm64")
  set(PANTA_DAWN_PLATFORM macos-latest)
  set(PANTA_DAWN_SHA256 0729bc3f245584181238f5d2a6bd3423613d69a6c301d7b0d21a8c7516c058e3)
elseif(PANTA_SDK_TRIPLE STREQUAL "linux-x86_64")
  set(PANTA_DAWN_PLATFORM ubuntu-latest)
  set(PANTA_DAWN_SHA256 67e87bd8455256fefb81cc752ede0b4704289f6b49cf72ebd9e099e409f0e2f4)
elseif(PANTA_SDK_TRIPLE STREQUAL "windows-x86_64")
  set(PANTA_DAWN_PLATFORM windows-latest)
  set(PANTA_DAWN_SHA256 7af79f8525b15802d1438c6d2cd648cbea771c2cae56b43cae07870dd0f30130)
else()
  message(FATAL_ERROR "VTK WebGPU 制品没有登记 Dawn 平台资产：${PANTA_SDK_TRIPLE}")
endif()
set(PANTA_DAWN_ARCHIVE "Dawn-${PANTA_DAWN_COMMIT}-${PANTA_DAWN_PLATFORM}-Release.tar.gz")
set(PANTA_DAWN_URL
    "https://github.com/google/dawn/releases/download/v${PANTA_DAWN_VERSION}/${PANTA_DAWN_ARCHIVE}")

include(ExternalProject)
ExternalProject_Add(
  vtk_sdk
  GIT_REPOSITORY https://github.com/Kitware/VTK.git
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
             -DDawn_DIR=<SOURCE_DIR>/.dawn/dawn/lib/cmake/Dawn
             -DVTK_RELOCATABLE_INSTALL=ON
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

# VTK 源码已 checkout 后再下载/解包 Dawn，保证 VTK 的 configure step 能看到
# <SOURCE_DIR>/.dawn/dawn。Dawn 不是另一个可选
# 构建路径，而是 RenderingWebGPU 的固定外部依赖。
ExternalProject_Add_Step(
  vtk_sdk dawn
  COMMAND
    ${CMAKE_COMMAND} -DPANTA_DAWN_DOWNLOAD_DIR=<SOURCE_DIR>/.dawn -DPANTA_DAWN_URL=${PANTA_DAWN_URL}
    -DPANTA_DAWN_ARCHIVE=${PANTA_DAWN_ARCHIVE} -DPANTA_DAWN_SHA256=${PANTA_DAWN_SHA256}
    -DPANTA_DAWN_LICENSE_URL=${PANTA_DAWN_LICENSE_URL}
    -DPANTA_DAWN_LICENSE_SHA256=${PANTA_DAWN_LICENSE_SHA256} -P
    ${CMAKE_CURRENT_SOURCE_DIR}/download-dawn.cmake
  DEPENDEES update
  DEPENDERS configure
  USES_TERMINAL)

# 把 Dawn 的头文件、CMake package 和 native library 放进同一 VTK SDK，
# 避免生成只在 CI 能 configure、消费侧却缺 Dawn 依赖的半成品。
ExternalProject_Add_Step(
  vtk_sdk dawn_runtime
  COMMAND
    ${CMAKE_COMMAND} -DPANTA_DAWN_INSTALL_DIR=<SOURCE_DIR>/.dawn/dawn
    -DPANTA_VTK_INSTALL_DIR=${PANTA_VTK_INSTALL_DIR} -P
    ${CMAKE_CURRENT_SOURCE_DIR}/install-dawn.cmake
  DEPENDEES install
  USES_TERMINAL)

# panta-sdk.json：031 供给自检与 012 缓存键的机器可读依据。
file(
  WRITE "${CMAKE_BINARY_DIR}/vtk-panta-sdk.json.in"
  [=[
{
  "name": "vtk",
  "version": "@PANTA_VTK_VERSION@",
  "triple": "@PANTA_SDK_TRIPLE@",
  "source": {
    "repository": "https://github.com/Kitware/VTK.git",
    "tag": "v9.7.0",
    "commit": "@PANTA_VTK_COMMIT@"
  },
  "build_type": "Release",
  "shared": true,
  "relocatable_install": true,
  "graphics_backend": "WebGPU",
  "window_system": "VTK hardware window",
  "macos_surface": "CocoaHardwareWindow",
  "dawn": {
    "version": "@PANTA_DAWN_VERSION@",
    "commit": "@PANTA_DAWN_COMMIT@",
    "platform": "@PANTA_DAWN_PLATFORM@",
    "archive": "@PANTA_DAWN_ARCHIVE@",
    "sha256": "@PANTA_DAWN_SHA256@",
    "url": "@PANTA_DAWN_URL@",
    "license_url": "@PANTA_DAWN_LICENSE_URL@",
    "license_sha256": "@PANTA_DAWN_LICENSE_SHA256@"
  },
  "modules_highlights": ["RenderingWebGPU", "RenderingUI", "RenderingCore"],
  "cmake_package": ["lib/cmake/vtk-@PANTA_VTK_VERSION@", "vtk-config.cmake"],
  "generator": "@CMAKE_GENERATOR@",
  "license_files": ["share/licenses/VTK/Copyright.txt", "share/licenses/Dawn/LICENSE"],
  "license": "share/licenses/VTK/Copyright.txt (BSD-3)"
}
]=])
configure_file("${CMAKE_BINARY_DIR}/vtk-panta-sdk.json.in"
               "${CMAKE_BINARY_DIR}/vtk-panta-sdk-configure.json" @ONLY)
ExternalProject_Add_Step(
  vtk_sdk metadata
  COMMAND ${CMAKE_COMMAND} -E copy_if_different ${CMAKE_BINARY_DIR}/vtk-panta-sdk-configure.json
          ${PANTA_VTK_INSTALL_DIR}/panta-sdk.json
  DEPENDEES install dawn_runtime)
