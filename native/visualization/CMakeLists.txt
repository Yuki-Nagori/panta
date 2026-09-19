# VTK 原生视口模块（任务 007）：Panta.Visualization 静态 QML 模块。
# CaeViewport（应用/QML 面，零第三方类型）在 include/panta/visualization/；
# VTK 专属代码收敛在 src/vtk/ 适配器（standards/vtk.md：VTK 不是公共 API）。
# VTK 消费走 031 固定供给（panta_require_sdk，三平台同源，禁止源码回退）；
# 模块链接显式最小集，其余依赖经 imported target 传递。
panta_require_sdk("vtk" _vtk_staging)

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
  SOURCES
  include/panta/visualization/cae_viewport.hpp
  include/panta/visualization/render_scene.hpp
  include/panta/visualization/viewport_backend.hpp
  src/cae_viewport.cpp
  src/vtk/vtk_viewport.hpp
  src/vtk/vtk_viewport.cpp)
add_library(panta::visualization ALIAS panta_visualization)
panta_native_defaults(panta_visualization)
# GUISupportQtQuick 传递 RenderingOpenGL2/CommonCore；测试图元生成所需
# 的模块按 vtk.md 显式声明，不整包拉取 VTK。
target_link_libraries(panta_visualization PUBLIC VTK::GUISupportQtQuick VTK::FiltersSources
                                                 Qt6::Quick)
