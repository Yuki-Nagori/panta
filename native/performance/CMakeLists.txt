# 手动性能工具集中登记；不注册到 CTest 或 CI 时间门禁。

add_executable(panta_bridge_project_stl_activation_cpu_benchmark EXCLUDE_FROM_ALL
               ../../tests/cpp/performance/project_stl_activation_cpu_benchmark.cpp)
panta_native_defaults(panta_bridge_project_stl_activation_cpu_benchmark)
target_link_libraries(panta_bridge_project_stl_activation_cpu_benchmark PRIVATE panta::bridge)

add_executable(panta_project_stl_viewport_gpu_benchmark EXCLUDE_FROM_ALL
               ../../tests/cpp/performance/project_stl_viewport_gpu_benchmark.cpp)
panta_native_defaults(panta_project_stl_viewport_gpu_benchmark)
target_include_directories(panta_project_stl_viewport_gpu_benchmark
                           PRIVATE ../visualization/src/vtk)
target_link_libraries(panta_project_stl_viewport_gpu_benchmark
                      PRIVATE panta::bridge panta::visualization Qt6::Gui Qt6::Quick)

if(WIN32)
  target_link_libraries(panta_bridge_project_stl_activation_cpu_benchmark PRIVATE Psapi)
  target_link_libraries(panta_project_stl_viewport_gpu_benchmark PRIVATE Psapi)
endif()

# lint 编译数据库需要与其他手动基准一致地生成 AUTOGEN 目录。
add_dependencies(panta_benchmark_moc panta_bridge_project_stl_activation_cpu_benchmark_autogen
                 panta_project_stl_viewport_gpu_benchmark_autogen)
