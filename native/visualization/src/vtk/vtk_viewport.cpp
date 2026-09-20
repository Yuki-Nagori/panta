/// VtkViewport 实现：在 GUI 线程管理 VTK WebGPU 与原生 surface。
///
/// Qt Quick 只负责 QQuickItem 宿主的几何和可见性；VTK 的 render window
/// 直接绘制到平台 surface，避免把 OpenGL/WebGPU 资源混入 scenegraph。
#include "vtk_viewport.hpp"
#include "vtk_native_surface.hpp"
#include <QGuiApplication>
#include <QMetaObject>
#include <QQuickItem>
#include <QQuickWindow>
#include <QRectF>
#include <QString>
#include <QTimer>
#include <QWindow>
#include <QtCore/qtmetamacros.h>
#include <QtGlobal>
#include <QtLogging>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>
#include <vtkActor.h>
#include <vtkCamera.h>
#include <vtkHardwareWindow.h>
#include <vtkNew.h>
#include <vtkPolyDataMapper.h>
#include <vtkProperty.h>
#include <vtkSmartPointer.h>
#include <vtkSphereSource.h>
#include <vtkWebGPURenderWindow.h>
#include <vtkWebGPURenderer.h>

namespace panta::visualization {

namespace {

constexpr double kDefaultSphereRadius = 0.5;
constexpr double kDefaultSphereRed = 0.18;
constexpr double kDefaultSphereGreen = 0.58;
constexpr double kDefaultSphereBlue = 0.95;
constexpr double kDefaultCameraX = 0.0;
constexpr double kDefaultCameraY = 0.0;
constexpr double kDefaultCameraZ = 4.5;
constexpr double kDefaultCameraNear = 0.1;
constexpr double kDefaultCameraFar = 100.0;
constexpr double kDefaultCameraViewAngle = 30.0;

void configure_default_camera(vtkWebGPURenderer* renderer) {
    vtkCamera* camera = renderer->GetActiveCamera();
    camera->SetPosition(kDefaultCameraX, kDefaultCameraY, kDefaultCameraZ);
    camera->SetFocalPoint(0.0, 0.0, 0.0);
    camera->SetViewUp(0.0, 1.0, 0.0);
    camera->SetViewAngle(kDefaultCameraViewAngle);
    camera->SetClippingRange(kDefaultCameraNear, kDefaultCameraFar);
}

} // namespace

struct VtkViewport::Impl {
    RenderScene pending;
    NativeSurface native_surface;
    std::unique_ptr<vtkHardwareWindow, void (*)(vtkHardwareWindow*)> hardware_window{nullptr,
                                                                                     nullptr};
    vtkSmartPointer<vtkWebGPURenderWindow> render_window;
    vtkSmartPointer<vtkWebGPURenderer> renderer;
    vtkSmartPointer<vtkActor> primitive_actor;
    QMetaObject::Connection window_visibility_connection;
    QMetaObject::Connection window_visible_connection;
    QMetaObject::Connection scene_graph_initialized_connection;
};

VtkViewport::VtkViewport(QQuickItem* parent) : QQuickItem(parent), impl_(std::make_unique<Impl>()) {
    setFlag(ItemHasContents, false);
    QObject::connect(this, &QQuickItem::windowChanged, this, [this](QQuickWindow* new_window) {
        QObject::disconnect(impl_->window_visibility_connection);
        QObject::disconnect(impl_->window_visible_connection);
        QObject::disconnect(impl_->scene_graph_initialized_connection);
        if (new_window == nullptr) {
            return;
        }
        const auto refresh_frame = [this]() {
            if (window() == nullptr || !window()->isVisible()) {
                return;
            }
            // QWindow::visibleChanged is emitted while Qt is still completing
            // the native window transition. Defer the first VTK attachment
            // until that transition has returned to the event loop.
            QTimer::singleShot(0, this, [this]() {
                if (window() != nullptr && window()->isVisible()) {
                    ensure_render_window();
                    sync_native_surface();
                }
            });
        };
        impl_->window_visibility_connection =
            QObject::connect(new_window, &QWindow::visibilityChanged, this,
                             [refresh_frame](QWindow::Visibility) { refresh_frame(); });
        impl_->window_visible_connection = QObject::connect(
            new_window, &QWindow::visibleChanged, this, [refresh_frame](bool) { refresh_frame(); });
        impl_->scene_graph_initialized_connection =
            QObject::connect(new_window, &QQuickWindow::sceneGraphInitialized, this, refresh_frame);
        QTimer::singleShot(0, this, refresh_frame);
    });
}

VtkViewport::~VtkViewport() { destroy_render_window(); }

QQuickItem* VtkViewport::item() { return this; }

void VtkViewport::apply_state(const RenderScene& state) {
    if (state.revision < impl_->pending.revision) {
        return;
    }
    impl_->pending = state;
    ensure_render_window();
    if (impl_->renderer == nullptr || impl_->render_window == nullptr) {
        return;
    }

    impl_->renderer->SetBackground(impl_->pending.background.redF(),
                                   impl_->pending.background.greenF(),
                                   impl_->pending.background.blueF());
    impl_->primitive_actor->SetVisibility(impl_->pending.primitive_visible);
    impl_->render_window->Render();
}

void VtkViewport::geometryChange(const QRectF& new_geometry, const QRectF& old_geometry) {
    QQuickItem::geometryChange(new_geometry, old_geometry);
    ensure_render_window();
    sync_native_surface();
}

void VtkViewport::itemChange(ItemChange change, const ItemChangeData& value) {
    QQuickItem::itemChange(change, value);
    if (change == ItemSceneChange) {
        ensure_render_window();
        sync_native_surface();
    } else if (change == ItemVisibleHasChanged) {
        ensure_render_window();
        sync_native_surface();
        if (impl_->native_surface.view != nullptr) {
            set_native_surface_visible(impl_->native_surface, isVisible());
        }
    }
}

void VtkViewport::ensure_render_window() {
    if (impl_->render_window != nullptr) {
        return;
    }
    if (window() == nullptr) {
        return;
    }
    // QML geometry is available before the native window is shown. Creating a
    // platform surface in that phase can make it cover the whole host view or
    // submit against an incomplete Cocoa/Wayland window hierarchy.
    if (!window()->isVisible()) {
        return;
    }
    if (width() <= 0 || height() <= 0) {
        return;
    }
    if (QGuiApplication::platformName() == QStringLiteral("offscreen")) {
        qWarning("VtkViewport: offscreen 平台没有原生 surface，跳过 WebGPU 视口初始化");
        return;
    }

    impl_->hardware_window = create_native_hardware_window(window());
    if (impl_->hardware_window == nullptr || impl_->hardware_window.get() == nullptr) {
        qWarning("VtkViewport: 当前 Qt 平台没有可用的 VTK hardware window");
        return;
    }

    const qreal scale = window()->devicePixelRatio();
    const int pixel_width = qMax(1, qRound(width() * scale));
    const int pixel_height = qMax(1, qRound(height() * scale));
    impl_->hardware_window->SetShowWindow(false);
    impl_->hardware_window->SetSize(pixel_width, pixel_height);
    impl_->hardware_window->Create();

    if (!attach_native_surface(window(), this, impl_->hardware_window.get(),
                               impl_->native_surface)) {
        qWarning("VtkViewport: 无法把 VTK 原生 surface 接入 Qt Quick 窗口");
        impl_->hardware_window->Destroy();
        impl_->hardware_window.reset();
        return;
    }

    impl_->render_window = vtkSmartPointer<vtkWebGPURenderWindow>::New();
    impl_->render_window->SetShowWindow(false);
    impl_->render_window->SetHardwareWindow(impl_->hardware_window.get());
    impl_->render_window->SetSize(pixel_width, pixel_height);
    impl_->renderer = vtkSmartPointer<vtkWebGPURenderer>::New();
    impl_->primitive_actor = vtkSmartPointer<vtkActor>::New();

    vtkNew<vtkSphereSource> primitive;
    primitive->SetRadius(kDefaultSphereRadius);
    primitive->SetThetaResolution(24);
    primitive->SetPhiResolution(16);
    vtkNew<vtkPolyDataMapper> mapper;
    mapper->SetInputConnection(primitive->GetOutputPort());
    impl_->primitive_actor->SetMapper(mapper);
    impl_->primitive_actor->GetProperty()->SetColor(kDefaultSphereRed, kDefaultSphereGreen,
                                                    kDefaultSphereBlue);
    impl_->renderer->AddActor(impl_->primitive_actor);
    configure_default_camera(impl_->renderer);
    impl_->renderer->SetBackground(impl_->pending.background.redF(),
                                   impl_->pending.background.greenF(),
                                   impl_->pending.background.blueF());
    impl_->primitive_actor->SetVisibility(impl_->pending.primitive_visible);
    impl_->render_window->AddRenderer(impl_->renderer);
    impl_->render_window->Initialize();

    if (impl_->render_window->GetGenericContext() == nullptr) {
        qWarning("VtkViewport: VTK WebGPU device 初始化失败");
        destroy_render_window();
        return;
    }

    sync_native_surface();
    Q_EMIT sceneInitialized();
}

void VtkViewport::sync_native_surface() {
    if (impl_->hardware_window == nullptr || window() == nullptr ||
        impl_->native_surface.view == nullptr) {
        return;
    }
    ::panta::visualization::sync_native_surface(window(), this, impl_->hardware_window.get(),
                                                impl_->native_surface);
    if (impl_->render_window != nullptr) {
        const qreal scale = window()->devicePixelRatio();
        const int pixel_width = qMax(1, qRound(width() * scale));
        const int pixel_height = qMax(1, qRound(height() * scale));
        impl_->render_window->SetSize(pixel_width, pixel_height);
        impl_->render_window->Render();
    }
}

void VtkViewport::destroy_render_window() {
    if (impl_ == nullptr) {
        return;
    }
    if (impl_->render_window != nullptr) {
        impl_->render_window->Finalize();
        impl_->render_window = nullptr;
    }
    if (impl_->native_surface.view != nullptr) {
        detach_native_surface(impl_->native_surface);
    }
    impl_->renderer = nullptr;
    impl_->primitive_actor = nullptr;
    if (impl_->hardware_window != nullptr) {
        impl_->hardware_window->Destroy();
        impl_->hardware_window.reset();
    }
}

std::unique_ptr<ViewportBackend> create_vtk_viewport_backend() {
    return std::make_unique<VtkViewport>();
}

} // namespace panta::visualization
