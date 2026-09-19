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

# VTK 9.7 的 RenderingWebGPU 在桌面平台依赖与 VTK 同步的 Dawn 预编译
# 运行时；只打开 VTK_ENABLE_WEBGPU 而不提供 Dawn_DIR 会在 configure 阶段失败。
# 版本、平台资产和 SHA256 取自 VTK v9.7.0 自带的 .gitlab/ci/download_dawn.cmake。
set(PANTA_DAWN_VERSION 20251002.162335)
set(PANTA_DAWN_BUILD_DATE 20260130.0)
if(PANTA_SDK_TRIPLE STREQUAL "macos-arm64")
  set(PANTA_DAWN_PLATFORM macos-arm64)
  set(PANTA_DAWN_EXTENSION tar.gz)
  set(PANTA_DAWN_SHA256 1e4537f51cc39500fee35cb0ab8f40b0491fe8cd9b1d8c6cc87eba35dcdf16eb)
elseif(PANTA_SDK_TRIPLE STREQUAL "linux-x86_64")
  set(PANTA_DAWN_PLATFORM linux-x86_64)
  set(PANTA_DAWN_EXTENSION tar.gz)
  set(PANTA_DAWN_SHA256 a0f846e06f0ebdfe5f74b0fc193e3b74421ae9c9524a9d37cf9bf39c52110921)
elseif(PANTA_SDK_TRIPLE STREQUAL "windows-x86_64")
  set(PANTA_DAWN_PLATFORM windows-x86_64)
  set(PANTA_DAWN_EXTENSION zip)
  set(PANTA_DAWN_SHA256 500afd33a3b3ab1e3a7153900a18f32e9979ec4c9021d645c133f0e2db3ba198)
else()
  message(FATAL_ERROR "VTK WebGPU 制品没有登记 Dawn 平台资产：${PANTA_SDK_TRIPLE}")
endif()
set(PANTA_DAWN_ARCHIVE "dawn-v${PANTA_DAWN_VERSION}-${PANTA_DAWN_PLATFORM}.${PANTA_DAWN_EXTENSION}")
set(PANTA_DAWN_URL
    "https://gitlab.kitware.com/api/v4/projects/6955/packages/generic/dawn/v${PANTA_DAWN_VERSION}-${PANTA_DAWN_BUILD_DATE}/${PANTA_DAWN_ARCHIVE}"
)

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
             -DDawn_DIR=<SOURCE_DIR>/.gitlab/dawn/lib/cmake/Dawn
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

# VTK 源码已 checkout 后再按上游固定脚本下载/解包 Dawn，保证 VTK 的
# configure step 能看到 <SOURCE_DIR>/.gitlab/dawn。Dawn 不是另一个可选
# 构建路径，而是 RenderingWebGPU 的固定外部依赖。
ExternalProject_Add_Step(
  vtk_sdk dawn
  COMMAND
    ${CMAKE_COMMAND} -DPANTA_DAWN_DOWNLOAD_DIR=<SOURCE_DIR>/.gitlab
    -DPANTA_DAWN_URL=${PANTA_DAWN_URL} -DPANTA_DAWN_ARCHIVE=${PANTA_DAWN_ARCHIVE}
    -DPANTA_DAWN_SHA256=${PANTA_DAWN_SHA256} -P ${CMAKE_CURRENT_SOURCE_DIR}/download-dawn.cmake
  DEPENDEES update
  DEPENDERS configure
  USES_TERMINAL)

# VTK 的 WebGPU 模块运行期通过 proc table 加载 Dawn；把 Dawn 的头文件、
# CMake package 和平台库放进同一 VTK SDK，避免生成只在 CI 能 configure、
# 消费侧却缺 runtime 的半成品。
ExternalProject_Add_Step(
  vtk_sdk dawn_runtime
  COMMAND
    ${CMAKE_COMMAND} -DPANTA_DAWN_INSTALL_DIR=<SOURCE_DIR>/.gitlab/dawn
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
    "repository": "https://gitlab.kitware.com/vtk/vtk.git",
    "tag": "v9.7.0",
    "commit": "@PANTA_VTK_COMMIT@"
  },
  "build_type": "Release",
  "shared": true,
  "graphics_backend": "WebGPU",
  "window_system": "VTK hardware window",
  "macos_surface": "CocoaHardwareWindow",
  "dawn": {
    "version": "@PANTA_DAWN_VERSION@",
    "build_date": "@PANTA_DAWN_BUILD_DATE@",
    "platform": "@PANTA_DAWN_PLATFORM@",
    "sha256": "@PANTA_DAWN_SHA256@",
    "url": "@PANTA_DAWN_URL@"
  },
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
  DEPENDEES install dawn_runtime)
