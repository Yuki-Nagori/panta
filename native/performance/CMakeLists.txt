# 手动性能套件：cargo performance 先执行辅助代码测试，再运行 CPU / GPU 基准。
# 所有目标从默认构建与 CTest 排除；质量工具仍检查源码和 AUTOMOC 产物。
set(_performance_sources "${CMAKE_CURRENT_SOURCE_DIR}/../../tests/performance")
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/performance")
set(_vtk_private "${CMAKE_CURRENT_SOURCE_DIR}/../visualization/src/vtk")

function(panta_performance_target name source)
  add_executable(${name} EXCLUDE_FROM_ALL "${_performance_sources}/${source}")
  panta_native_defaults(${name})
  add_dependencies(panta_benchmark_moc ${name}_autogen)
endfunction()

panta_performance_target(panta_benchmark_support_test support/benchmark_support_test.cpp)
target_link_libraries(panta_benchmark_support_test PRIVATE Qt6::Test)

panta_performance_target(panta_mesh_ir_benchmark cpu/mesh_ir_benchmark.cpp)
target_link_libraries(panta_mesh_ir_benchmark PRIVATE panta::mesh)

foreach(_kind cpu gpu)
  panta_performance_target(panta_qml_${_kind}_benchmark ${_kind}/project_docks_benchmark.cpp)
  target_link_libraries(panta_qml_${_kind}_benchmark PRIVATE panta_shell panta_shellplugin Qt6::Gui
                                                             Qt6::Qml Qt6::Quick Qt6::Test)
  qt_add_resources(
    panta_qml_${_kind}_benchmark
    panta_control_gallery
    PREFIX
    "/panta-benchmark"
    BASE
    "${CMAKE_CURRENT_SOURCE_DIR}/../../tests/fixtures/qml"
    FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/../../tests/fixtures/qml/ControlGallery.qml")
  if(PANTA_ENABLE_BRIDGE_MODULE)
    target_link_libraries(panta_qml_${_kind}_benchmark PRIVATE panta::bridge)
    target_compile_definitions(panta_qml_${_kind}_benchmark PRIVATE PANTA_TEST_WITH_BRIDGE=1)
  endif()
endforeach()
find_package(Qt6 6.11 REQUIRED COMPONENTS GuiPrivate)
target_link_libraries(panta_qml_gpu_benchmark PRIVATE Qt6::GuiPrivate)

if(PANTA_ENABLE_BRIDGE_MODULE)
  panta_require_sdk("vtk" _performance_vtk_staging)
  target_link_libraries(panta_benchmark_support_test PRIVATE VTK::CommonCore)
  target_compile_definitions(panta_benchmark_support_test PRIVATE PANTA_TEST_WITH_VTK=1)
  panta_performance_target(panta_viewport_navigation_cpu_benchmark
                           cpu/viewport_navigation_benchmark.cpp)
  target_include_directories(panta_viewport_navigation_cpu_benchmark PRIVATE "${_vtk_private}")
  target_link_libraries(
    panta_viewport_navigation_cpu_benchmark
    PRIVATE panta::visualization Qt6::Test VTK::CommonDataModel VTK::FiltersSources
            VTK::RenderingCore)

  panta_performance_target(panta_viewport_gpu_benchmark gpu/viewport_benchmark.cpp)
  target_include_directories(panta_viewport_gpu_benchmark PRIVATE "${_vtk_private}")
  target_link_libraries(panta_viewport_gpu_benchmark PRIVATE panta::visualization Qt6::Gui
                                                             Qt6::Quick Qt6::Test)

  panta_performance_target(panta_bridge_project_stl_activation_cpu_benchmark
                           cpu/project_stl_activation_benchmark.cpp)
  target_link_libraries(panta_bridge_project_stl_activation_cpu_benchmark PRIVATE panta::bridge)
  panta_performance_target(panta_project_stl_viewport_gpu_benchmark
                           gpu/project_stl_viewport_benchmark.cpp)
  target_include_directories(panta_project_stl_viewport_gpu_benchmark PRIVATE "${_vtk_private}")
  target_link_libraries(panta_project_stl_viewport_gpu_benchmark
                        PRIVATE panta::bridge panta::visualization Qt6::Gui Qt6::Quick)
  panta_performance_target(panta_performance_fixture support/project_fixture.cpp)
  target_link_libraries(panta_performance_fixture PRIVATE panta::bridge)
  if(WIN32)
    target_link_libraries(panta_bridge_project_stl_activation_cpu_benchmark PRIVATE Psapi)
    target_link_libraries(panta_project_stl_viewport_gpu_benchmark PRIVATE Psapi)
  endif()
  foreach(_vtk_gpu_target panta_viewport_gpu_benchmark panta_project_stl_viewport_gpu_benchmark)
    target_link_libraries(${_vtk_gpu_target} PRIVATE VTK::RenderingWebGPU VTK::RenderingCore
                                                     VTK::FiltersCore VTK::CommonDataModel)
  endforeach()
endif()
