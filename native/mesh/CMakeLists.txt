# 网格模块（任务 010）：core 为自有 Mesh IR，netgen 为生成适配器。
# netgen 子目录消费 OCCT imported targets，必须在 geometry（含
# find_package(OpenCASCADE)）之后加入构建图。
add_subdirectory(core)
