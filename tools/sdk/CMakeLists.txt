# Native SDK 制品生产 superbuild（任务 038）。
#
# 这是可审计构建描述的载体：以 ExternalProject 固定源码 commit、冻结配置
# 开关与依赖版本，产出统一布局的预编译 SDK（modules/native-dependency-supply.md）。
# 只在受信 CI/维护者环境中运行；不进入普通开发构建图（cargo build 不触达
# 本目录），开发者经 031 的 sdk-provision 消费已发布的归档。
#
# 入口约定：
#   cmake -S tools/sdk -B <build> -G Ninja
#   cmake --build <build> [--target <name>_sdk]
# VTK 描述会按需供给锁定预编译 Qt（-DQT_PROVISION_DIR 可指向共享缓存，
# 见 sdk-vtk.yml）；OCCT/Netgen 描述不消费 Qt（GUI/Visualization 全关），
# 用 -DPANTA_SDK_ENABLE_VTK=OFF 跳过 VTK 描述即不触发任何 Qt 下载
# （sdk-occt-netgen.yml 传此开关）。
# 产物：<build>/out/<name>/<version>/<triple>/（安装树）；
# 自检见 selfcheck/（真实 configure 消费工程），打包见 package.cmake。

cmake_minimum_required(VERSION 3.22)
project(panta_sdk_production NONE)

# triple 映射与 031 消费侧同源，避免生产/供给两侧命名漂移。
include(${CMAKE_CURRENT_SOURCE_DIR}/../../native/cmake/sdk-provision.cmake)
panta_sdk_triple(PANTA_SDK_TRIPLE)

set(PANTA_SDK_OUT_ROOT
    "${CMAKE_BINARY_DIR}/out"
    CACHE PATH "SDK 安装树根")
message(STATUS "panta SDK 生产：triple=${PANTA_SDK_TRIPLE} out=${PANTA_SDK_OUT_ROOT}")

# 生产顺序：occt → netgen（依赖自产 OCCT，同 triple 同 ABI）；vtk 独立。
include(${CMAKE_CURRENT_SOURCE_DIR}/occt.cmake)
include(${CMAKE_CURRENT_SOURCE_DIR}/netgen.cmake)
# VTK 消费 Qt（GUISupportQtQuick 在 C++ 层使用 Qt 类型，须对着锁定
# 6.11.2 编译，见 vtk.cmake）；qt-provision 在 configure 期执行下载/解包，
# 因此必须与 VTK 描述一起条件化——只生产 OCCT/Netgen 时零 Qt 触达。
option(PANTA_SDK_ENABLE_VTK "Provision Qt and configure the VTK SDK description" ON)
if(PANTA_SDK_ENABLE_VTK)
  include(${CMAKE_CURRENT_SOURCE_DIR}/../../native/cmake/qt-provision.cmake)
  include(${CMAKE_CURRENT_SOURCE_DIR}/vtk.cmake)
endif()
