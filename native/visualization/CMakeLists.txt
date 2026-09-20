# VTK 原生视口模块（任务 007）：Panta.Visualization 静态 QML 模块。
# CaeViewport（应用/QML 面，零第三方类型）在 include/panta/visualization/；
# VTK 专属代码收敛在 src/vtk/ 适配器（standards/vtk.md：VTK 不是公共 API）。
# VTK 消费走 031 固定供给（panta_require_sdk，三平台同源，禁止源码回退）；
# 模块链接显式最小集，其余依赖经 imported target 传递。
panta_require_sdk("vtk" _vtk_staging)

if(APPLE)
  enable_language(OBJCXX)
endif()

# 先声明 STATIC 库，qt_add_qml_module 才会生成静态 plugin（与 bridge 同型）。
# qt_add_qml_module 按 URI 自动创建别名 panta::visualization，无需手动。
add_library(panta_visualization STATIC)

# 公共头按 panta/<module>/<header>.hpp 布局（与 panta/foundation 同型）；
# include 目录必须在 qt_add_qml_module 之前声明——QML 类型注册的 include
# 形式按当时已声明的目标 include 目录推断。
target_include_directories(panta_visualization PUBLIC include)
target_include_directories(panta_visualization PRIVATE src include/panta/visualization)

qt_add_qml_module(
  panta_visualization
  URI
  Panta.Visualization
  VERSION
  1.0
  # 与 URI target path 对齐（qmllint import 解析依赖该布局，与 bridge 同型）。
  OUTPUT_DIRECTORY
  "${CMAKE_BINARY_DIR}/Panta/Visualization"
  SOURCES
  include/panta/visualization/cae_viewport.hpp
  include/panta/visualization/render_scene.hpp
  include/panta/visualization/viewport_backend.hpp
  src/cae_viewport.cpp
  src/vtk/vtk_viewport.hpp
  src/vtk/vtk_viewport.cpp
  src/vtk/vtk_native_surface.hpp)
if(APPLE)
  target_sources(panta_visualization PRIVATE src/vtk/vtk_native_surface.mm)
else()
  target_sources(panta_visualization PRIVATE src/vtk/vtk_native_surface.cpp)
endif()
add_library(panta::visualization ALIAS panta_visualization)
panta_native_defaults(panta_visualization)
# Linux embeds VTK in a Wayland subsurface instead of rendering into the
# top-level Qt surface, so the bridge also links the Wayland client protocol.
if(UNIX AND NOT APPLE)
  find_package(PkgConfig REQUIRED)
  pkg_check_modules(WAYLAND_CLIENT REQUIRED IMPORTED_TARGET wayland-client)
endif()
# WebGPU/hardware-window 路径不依赖 Qt VTK GUI 或 OpenGL scenegraph；
# 测试图元生成所需的 FiltersSources 和 Dawn 运行时显式列出。
target_link_libraries(
  panta_visualization PUBLIC VTK::RenderingWebGPU VTK::RenderingUI VTK::RenderingCore
                             VTK::FiltersSources dawn::webgpu_dawn Qt6::Quick)
if(UNIX AND NOT APPLE)
  target_link_libraries(panta_visualization PRIVATE PkgConfig::WAYLAND_CLIENT)
endif()
# VTK::RenderingWebGPU 声明需要模块自动初始化；否则 render window 能创建并提交帧，
# 但 WebGPU renderer/mapper 的实现工厂不会注册，场景会保持空白。
vtk_module_autoinit(
  TARGETS
  panta_visualization
  MODULES
  VTK::RenderingWebGPU
  VTK::RenderingUI
  VTK::RenderingCore
  VTK::FiltersSources)
