/// VTK 私有场景构建：窗口视口与硬件离屏基准共用相同几何、材质和相机配置。
#pragma once

#include <memory>

class vtkActor;
class vtkRenderWindow;
class vtkWebGPURenderer;

namespace panta::visualization {
struct SurfaceMeshSnapshot;

/// 播放时间参与充填裁剪。网格指针或时间变化都要重建表面；只有网格换掉才重配相机。
struct PrimitiveGeometryUpdate {
    bool rebuild = false;
    bool refit_camera = false;
};

inline PrimitiveGeometryUpdate primitive_geometry_update(
    const std::shared_ptr<const SurfaceMeshSnapshot>& applied_mesh, double applied_time,
    const std::shared_ptr<const SurfaceMeshSnapshot>& pending_mesh, double pending_time) {
    const bool mesh_changed = applied_mesh != pending_mesh;
    const bool time_changed = applied_time != pending_time;
    return {mesh_changed || time_changed, mesh_changed};
}

/// GUI 线程调用；actor 非空；window 可空（仅 CPU 场景）。有窗口时先释放旧 mapper 的图形资源；空
/// mesh 使用欢迎字样。
void replace_primitive_geometry(vtkActor* actor, vtkRenderWindow* window,
                                const std::shared_ptr<const SurfaceMeshSnapshot>& mesh,
                                double playback_time);
/// 按 mesh 的 show_gates 与 gate_points 更新浇口标记：有推荐位置时显示红色球形，
/// 其余情况隐藏并释放旧几何。GUI 线程调用；actor 非空；window 可空（仅 CPU 场景）。
void replace_gate_geometry(vtkActor* actor, vtkRenderWindow* window,
                           const std::shared_ptr<const SurfaceMeshSnapshot>& mesh);
/// 使用 actor 的有效包围盒配置透视相机；aspect 和 fit_margin 必须为正。
void configure_default_camera(vtkWebGPURenderer* renderer, vtkActor* actor, double aspect,
                              double fit_margin);
} // namespace panta::visualization
