# Native SDK 制品生产 superbuild（任务 038）。
#
# 这是可审计构建描述的载体：以 ExternalProject 固定源码 commit、冻结配置
# 开关与依赖版本，产出统一布局的预编译 SDK（modules/native-dependency-supply.md）。
# 只在受信 CI/维护者环境中运行；不进入普通开发构建图（cargo build 不触达
# 本目录），开发者经 031 的 sdk-provision 消费已发布的归档。
#
# 入口约定：
#   cmake -S tools/sdk -B <build> -G Ninja \
#     -DQT_PROVISION_DIR=<已有 Qt 缓存，可选；缺省按固定清单供给>
#   cmake --build <build>
# 产物：<build>/out/<name>/<version>/<triple>/（安装树）；
# 自检见 selfcheck/（真实 configure 消费工程），打包见 package.cmake。

cmake_minimum_required(VERSION 3.22)
project(panta_sdk_production NONE)

# Qt dev 依赖：复用 005/041 的锁定预编译供给（SHA 强校验），保证制品与
# 开发者运行时同一 Qt ABI；PANTA_QT_PROVISION_DIR 可指向共享缓存。
include(${CMAKE_CURRENT_SOURCE_DIR}/../../native/cmake/qt-provision.cmake)

# triple 映射与 031 消费侧同源，避免生产/供给两侧命名漂移。
include(${CMAKE_CURRENT_SOURCE_DIR}/../../native/cmake/sdk-provision.cmake)
panta_sdk_triple(PANTA_SDK_TRIPLE)

set(PANTA_SDK_OUT_ROOT "${CMAKE_BINARY_DIR}/out" CACHE PATH "SDK 安装树根")
message(STATUS "panta SDK 生产：triple=${PANTA_SDK_TRIPLE} out=${PANTA_SDK_OUT_ROOT}")

include(${CMAKE_CURRENT_SOURCE_DIR}/vtk.cmake)
