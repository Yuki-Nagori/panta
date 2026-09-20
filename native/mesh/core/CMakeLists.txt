# Mesh IR（任务 010）：自有网格中间表示，独立于 Netgen/VTK 的数据边界
# （architecture/mesh.md）。纯自有类型 + 标准库，不依赖任何第三方与 Qt；
# 网格库适配见 ../netgen/。
add_library(panta_mesh_core STATIC)
add_library(panta::mesh_core ALIAS panta_mesh_core)

target_include_directories(panta_mesh_core PUBLIC include)

target_sources(panta_mesh_core PRIVATE include/panta/mesh/mesh_ir.hpp src/mesh_ir.cpp)

panta_native_defaults(panta_mesh_core)

# 冒烟阶段不进安装导出集：消费方（网格业务任务）接入时再按 foundation
# 模式补 install(TARGETS) 与 package 配置。
if(BUILD_TESTING)
  add_executable(panta_mesh_ir_test ../../../tests/cpp/mesh/mesh_ir_test.cpp)
  panta_native_defaults(panta_mesh_ir_test)
  target_link_libraries(panta_mesh_ir_test PRIVATE panta::mesh_core GTest::gtest_main)
  # 在构建后生成稳定的 CTest 用例清单（与 foundation 同型，见 native/CMakeLists.txt）。
  gtest_discover_tests(panta_mesh_ir_test DISCOVERY_MODE POST_BUILD)
endif()
