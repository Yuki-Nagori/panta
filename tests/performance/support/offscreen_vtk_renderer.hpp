#pragma once

#include "vtk_error_capture.hpp"
#include <QElapsedTimer>
#include <QQuickWindow>
#include <QSize>
#include <memory>
#include <navigation/viewport_orientation.hpp>
#include <panta/visualization/render_scene.hpp>
#include <primitive_scene.hpp>
#include <stdexcept>
#include <string>
#include <vtkActor.h>
#include <vtkSmartPointer.h>
#include <vtkWebGPUConfiguration.h>
#include <vtkWebGPURenderWindow.h>
#include <vtkWebGPURenderer.h>
#include <vtk_native_surface.hpp>
#include <webgpu/webgpu_cpp.h>
#include <welcome/welcome_scene.hpp>

namespace panta::test {

struct VtkRenderTiming {
    double submitMs;
    double completionMs;
};

/// 测试侧 VTK 硬件渲染目标：隐藏原生 surface，GUI 线程使用，无 Qt 事件呈现等待。
/// 完成耗时包含场景更新、CPU 提交和队列同步，不能当作 GPU 内核时间。
class OffscreenVtkRenderer final {
  public:
    explicit OffscreenVtkRenderer(QSize pixels) {
        if (pixels.isEmpty()) {
            throw std::invalid_argument("VTK render target must have positive dimensions");
        }
        m_hardware = panta::visualization::create_native_hardware_window(&m_host);
        if (!m_hardware) {
            throw std::runtime_error("VTK native hardware surface unavailable");
        }
        m_hardware->SetShowWindow(false);
        m_hardware->SetSize(pixels.width(), pixels.height());
        m_hardware->Create();
        m_window->SetHardwareWindow(m_hardware.get());
        m_window->SetOffScreenRendering(true);
        m_window->SetSize(pixels.width(), pixels.height());
        m_renderer->SetBackground2(0.98, 0.98, 0.98);
        m_renderer->GradientBackgroundOn();
        m_renderer->AddActor(m_actor);
        m_window->AddRenderer(m_renderer);
        m_orientation.attach(m_window);
        m_welcome.attach(m_window);
        m_window->Initialize();
        m_errors.check();
        auto* configuration = m_window->GetWGPUConfiguration();
        if (configuration == nullptr || !configuration->GetDevice()) {
            throw std::runtime_error("VTK WebGPU offscreen initialization failed");
        }
        wgpu::AdapterInfo info;
        if (!configuration->GetAdapter().GetInfo(&info) ||
            (info.adapterType != wgpu::AdapterType::DiscreteGPU &&
             info.adapterType != wgpu::AdapterType::IntegratedGPU)) {
            throw std::runtime_error("VTK benchmark requires an identifiable hardware GPU");
        }
        m_backend = configuration->GetBackendInUseAsString();
        m_pixels = pixels;
    }
    OffscreenVtkRenderer(const OffscreenVtkRenderer&) = delete;
    auto operator=(const OffscreenVtkRenderer&) -> OffscreenVtkRenderer& = delete;
    ~OffscreenVtkRenderer() {
        m_orientation.detach();
        m_welcome.detach();
        m_window->Finalize();
        m_window->SetHardwareWindow(nullptr);
        m_hardware->Destroy();
    }
    [[nodiscard]] auto backend() const -> const std::string& { return m_backend; }
    [[nodiscard]] auto render(const panta::visualization::RenderScene& scene) -> VtkRenderTiming {
        QElapsedTimer clock;
        clock.start();
        if (!m_hasGeometry || m_mesh != scene.mesh) {
            update_geometry(scene);
        }
        m_actor->SetVisibility(scene.primitive_visible);
        m_welcome.set_visible(scene.primitive_visible && scene.mesh == nullptr);
        m_renderer->SetBackground(scene.background.redF(), scene.background.greenF(),
                                  scene.background.blueF());
        m_orientation.update(m_renderer->GetActiveCamera());
        m_window->Render();
        const double submit = static_cast<double>(clock.nsecsElapsed()) / 1.0e6;
        m_window->WaitForCompletion();
        m_errors.check();
        return {submit, static_cast<double>(clock.nsecsElapsed()) / 1.0e6};
    }

  private:
    void update_geometry(const panta::visualization::RenderScene& scene) {
        panta::visualization::replace_primitive_geometry(m_actor, m_window, scene.mesh);
        const double aspect = static_cast<double>(m_pixels.width()) / m_pixels.height();
        panta::visualization::configure_default_camera(m_renderer, m_actor, aspect,
                                                       scene.mesh ? 1.25 : 2.25);
        m_mesh = scene.mesh;
        m_hasGeometry = true;
    }
    VtkErrorCapture m_errors;
    QQuickWindow m_host;
    panta::visualization::NativeHardwareWindow m_hardware;
    vtkSmartPointer<vtkWebGPURenderWindow> m_window = vtkSmartPointer<vtkWebGPURenderWindow>::New();
    vtkSmartPointer<vtkWebGPURenderer> m_renderer = vtkSmartPointer<vtkWebGPURenderer>::New();
    vtkSmartPointer<vtkActor> m_actor = vtkSmartPointer<vtkActor>::New();
    panta::visualization::ViewportOrientation m_orientation;
    panta::visualization::WelcomeScene m_welcome;
    std::shared_ptr<const panta::visualization::SurfaceMeshSnapshot> m_mesh;
    QSize m_pixels;
    std::string m_backend;
    bool m_hasGeometry = false;
};

} // namespace panta::test
