#include "primitive_scene.hpp"

#include "surface_mesh.hpp"
#include "welcome/welcome_scene.hpp"
#include <algorithm>
#include <cmath>
#include <memory>
#include <vtkActor.h>
#include <vtkCamera.h>
#include <vtkNew.h>
#include <vtkPolyData.h>
#include <vtkPolyDataMapper.h>
#include <vtkPolyDataNormals.h>
#include <vtkProperty.h>
#include <vtkRenderWindow.h>
#include <vtkSmartPointer.h>
#include <vtkWebGPURenderer.h>

namespace panta::visualization {

// 按视口宽高比和调用者边距适配透视相机，窄窗口也能容纳完整几何。
void configure_default_camera(vtkWebGPURenderer* renderer, vtkActor* actor, double aspect,
                              double fit_margin) {
    double bounds[6];
    actor->GetBounds(bounds);
    constexpr double view_angle = 30.0;
    constexpr double half_angle_radians = 0.2617993877991494;
    const double half_width = (bounds[1] - bounds[0]) / 2;
    const double half_height = (bounds[3] - bounds[2]) / 2;
    const double half_depth = (bounds[5] - bounds[4]) / 2;
    const double distance =
        fit_margin * std::max(half_height, half_width / aspect) / std::tan(half_angle_radians) +
        half_depth;
    vtkCamera* camera = renderer->GetActiveCamera();
    const double* center = actor->GetCenter();
    camera->SetPosition(center[0], center[1], center[2] + distance);
    camera->SetFocalPoint(center);
    camera->SetViewUp(0, 1, 0);
    camera->SetViewAngle(view_angle);
    renderer->ResetCameraClippingRange();
}

void replace_primitive_geometry(vtkActor* actor, vtkRenderWindow* window,
                                const std::shared_ptr<const SurfaceMeshSnapshot>& mesh) {
    vtkSmartPointer<vtkPolyData> geometry;
    const bool imported_mesh = mesh != nullptr;
    geometry = imported_mesh ? make_surface_poly_data(*mesh) : create_welcome_wordmark();

    vtkNew<vtkPolyDataMapper> mapper;
    if (imported_mesh) {
        mapper->SetInputData(geometry);
        mapper->SetColorModeToDefault();
        mapper->SetScalarModeToDefault();
    } else {
        vtkNew<vtkPolyDataNormals> normals;
        normals->SetInputData(geometry);
        normals->SetFeatureAngle(45.0);
        normals->ConsistencyOn();
        normals->SplittingOn();
        mapper->SetInputConnection(normals->GetOutputPort());
        mapper->SetColorModeToDirectScalars();
        mapper->SetScalarModeToUsePointData();
    }
    auto* previous_mapper = actor->GetMapper();
    if (previous_mapper != nullptr && window != nullptr) {
        // 页签切换会替换整条 mapper 管线；先显式释放旧窗口的图形资源，
        // 不依赖 mapper 析构来回收 WebGPU buffers。
        previous_mapper->ReleaseGraphicsResources(window);
    }
    actor->SetMapper(mapper);
    if (imported_mesh) {
        actor->SetOrientation(0.0, 0.0, 0.0);
    } else {
        actor->SetOrientation(16.0, -18.0, -4.0);
    }
    auto* material = actor->GetProperty();
    material->SetInterpolationToPhong();
    material->SetAmbient(0.22);
    material->SetDiffuse(0.78);
    material->SetSpecular(0.28);
    material->SetSpecularPower(28.0);
    if (imported_mesh) {
        material->SetColor(0.72, 0.82, 0.94);
    }
}

} // namespace panta::visualization
