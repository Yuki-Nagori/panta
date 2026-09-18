# OCCT 8.0.1 制品构建描述（任务 038；维护者决策 2026-09-18：三平台统一自
# 托管，不消费官方 Windows SDK——官方仅 Windows 有归档，跨平台工具链不一
# 致；009 集成如需再评估切换）。
#
# 源码固定：tag V8.0.1（lightweight）→ commit
# b8f597c677811d1f9f4d8a97f5ae2825c0353a42（2026-09-18 git ls-remote 复核）。
# 模块冻结：保留 Modeling/DataExchange/ApplicationFramework（STEP 读取链路
# 与其 CAF 依赖），关闭 Visualization（渲染归 VTK）、Draw、DETools——
# 同时免除 Tcl/freetype 等第三方依赖。许可证：LGPL-2.1 + OCCT 例外。

set(PANTA_OCCT_VERSION 8.0.1)
set(PANTA_OCCT_COMMIT b8f597c677811d1f9f4d8a97f5ae2825c0353a42)
set(PANTA_OCCT_INSTALL_DIR "${PANTA_SDK_OUT_ROOT}/occt/${PANTA_OCCT_VERSION}/${PANTA_SDK_TRIPLE}")

include(ExternalProject)
ExternalProject_Add(occt_sdk
  GIT_REPOSITORY https://github.com/Open-Cascade-SAS/OCCT.git
  GIT_TAG ${PANTA_OCCT_COMMIT}
  GIT_SHALLOW TRUE
  GIT_PROGRESS TRUE
  BUILD_COMMAND ${CMAKE_COMMAND} --build . --config Release
  CMAKE_ARGS
    -DCMAKE_BUILD_TYPE=Release
    -DBUILD_LIBRARY_TYPE=Shared
    -DBUILD_MODULE_Draw=OFF
    -DBUILD_MODULE_Visualization=OFF
    -DBUILD_MODULE_DETools=OFF
    -DCMAKE_INSTALL_PREFIX=${PANTA_OCCT_INSTALL_DIR}
  INSTALL_COMMAND ${CMAKE_COMMAND} --install . --config Release
  COMMAND ${CMAKE_COMMAND} -E make_directory
          ${PANTA_OCCT_INSTALL_DIR}/share/licenses/OCCT
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          <SOURCE_DIR>/LICENSE_LGPL_21.txt
          ${PANTA_OCCT_INSTALL_DIR}/share/licenses/OCCT/LICENSE_LGPL_21.txt
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          <SOURCE_DIR>/OCCT_LGPL_EXCEPTION.txt
          ${PANTA_OCCT_INSTALL_DIR}/share/licenses/OCCT/OCCT_LGPL_EXCEPTION.txt
  USES_TERMINAL_DOWNLOAD TRUE
  USES_TERMINAL_BUILD TRUE
)

file(WRITE "${CMAKE_BINARY_DIR}/occt-panta-sdk.json.in"
[=[
{
  "name": "occt",
  "version": "@PANTA_OCCT_VERSION@",
  "triple": "@PANTA_SDK_TRIPLE@",
  "source": {
    "repository": "https://github.com/Open-Cascade-SAS/OCCT.git",
    "tag": "V8.0.1",
    "commit": "@PANTA_OCCT_COMMIT@"
  },
  "build_type": "Release",
  "shared": true,
  "modules_off": ["Draw", "Visualization", "DETools"],
  "modules_highlights": ["ModelingData", "ModelingAlgorithms", "DataExchange", "ApplicationFramework"],
  "cmake_package": "OpenCASCADEConfig.cmake",
  "license": "share/licenses/OCCT/ (LGPL-2.1 + OCCT exception)"
}
]=])
configure_file("${CMAKE_BINARY_DIR}/occt-panta-sdk.json.in"
  "${CMAKE_BINARY_DIR}/occt-panta-sdk-configure.json" @ONLY)
ExternalProject_Add_Step(occt_sdk metadata
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          ${CMAKE_BINARY_DIR}/occt-panta-sdk-configure.json
          ${PANTA_OCCT_INSTALL_DIR}/panta-sdk.json
  DEPENDEES install
)
