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
#include <QSize>
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
    NativeHardwareWindow hardware_window;
    vtkSmartPointer<vtkWebGPURenderWindow> render_window;
    vtkSmartPointer<vtkWebGPURenderer> renderer;
    vtkSmartPointer<vtkActor> primitive_actor;
    QMetaObject::Connection window_visibility_connection;
    QMetaObject::Connection window_visible_connection;
    QMetaObject::Connection scene_graph_initialized_connection;
    /// 最近一次提交给 render window 的像素尺寸；空值表示尚无有效提交。
    QSize applied_pixel_size;
    bool refresh_scheduled = false;
    bool creation_warning_emitted = false;
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
        impl_->window_visibility_connection =
            QObject::connect(new_window, &QWindow::visibilityChanged, this,
                             [this](QWindow::Visibility) { schedule_refresh(); });
        impl_->window_visible_connection = QObject::connect(
            new_window, &QWindow::visibleChanged, this, [this](bool) { schedule_refresh(); });
        impl_->scene_graph_initialized_connection =
            QObject::connect(new_window, &QQuickWindow::sceneGraphInitialized, this,
                             [this]() { schedule_refresh(); });
        schedule_refresh();
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
    if (impl_->renderer == nullptr || impl_->render_window == nullptr ||
        impl_->primitive_actor == nullptr) {
        return;
    }

    impl_->renderer->SetBackground(impl_->pending.background.redF(),
                                   impl_->pending.background.greenF(),
                                   impl_->pending.background.blueF());
    impl_->primitive_actor->SetVisibility(impl_->pending.primitive_visible);
    // 隐藏期间跳过 Render，帧由重新显示时的强制提交补上。
    if (isVisible()) {
        impl_->render_window->Render();
    }
}

void VtkViewport::geometryChange(const QRectF& new_geometry, const QRectF& old_geometry) {
    QQuickItem::geometryChange(new_geometry, old_geometry);
    ensure_render_window();
    sync_native_surface();
}

void VtkViewport::itemChange(ItemChange change, const ItemChangeData& value) {
    QQuickItem::itemChange(change, value);
    if (change == ItemSceneChange) {
        // ItemSceneChange 的文档化载荷是 value.window；window() 何时切换为
        // 新窗口不属于契约，延后到事件循环读取已就绪的 window()。
        schedule_refresh();
    } else if (change == ItemVisibleHasChanged) {
        ensure_render_window();
        // 显示时强制补一帧（隐藏期间的状态变更没有渲染）；隐藏时下面的
        // 提交策略会跳过 Render。
        sync_native_surface(isVisible());
        if (impl_->native_surface.view != nullptr) {
            set_native_surface_visible(impl_->native_surface, isVisible());
        }
    } else if (change == ItemDevicePixelRatioHasChanged) {
        // 窗口跨显示器或系统缩放变化后，按旧 DPR 推导的像素尺寸失效。
        ensure_render_window();
        sync_native_surface();
    }
}

void VtkViewport::ensure_render_window() {
    if (impl_->render_window != nullptr) {
        return;
    }
    if (window() == nullptr) {
        return;
    }
    // QML 几何在原生窗口显示前就可用；在该阶段创建平台 surface 会让它
    // 铺满宿主视图，或对接到不完整的 Cocoa/Wayland 窗口层级。
    if (!window()->isVisible()) {
        return;
    }
    if (width() <= 0 || height() <= 0) {
        return;
    }
    // 创建会在后续几何/可见性事件里重试；告警每实例一次，避免不支持平台
    // 在每次重试时重复输出同一诊断。
    const auto warn_once = [this](const char* message) {
        if (!impl_->creation_warning_emitted) {
            impl_->creation_warning_emitted = true;
            qWarning("%s", message);
        }
    };
    if (QGuiApplication::platformName() == QStringLiteral("offscreen")) {
        warn_once("VtkViewport: offscreen 平台没有原生 surface，跳过 WebGPU 视口初始化");
        return;
    }

    impl_->hardware_window = create_native_hardware_window(window());
    if (impl_->hardware_window == nullptr) {
        warn_once("VtkViewport: 当前 Qt 平台没有可用的 VTK hardware window");
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
        warn_once("VtkViewport: 无法把 VTK 原生 surface 接入 Qt Quick 窗口");
        impl_->native_surface = {};
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
        warn_once("VtkViewport: VTK WebGPU device 初始化失败");
        destroy_render_window();
        return;
    }

    sync_native_surface();
    Q_EMIT sceneInitialized();
}

void VtkViewport::schedule_refresh() {
    // QWindow::visibleChanged 在 Qt 尚未完成原生窗口切换时发出，窗口级
    // 信号在同一次切换里也可能多次触发；排队一次延迟尝试，既等切换回到
    // 事件循环，又把重复信号折叠成一次。
    if (impl_->refresh_scheduled) {
        return;
    }
    impl_->refresh_scheduled = true;
    QTimer::singleShot(0, this, [this]() {
        impl_->refresh_scheduled = false;
        if (window() != nullptr && window()->isVisible()) {
            ensure_render_window();
            sync_native_surface();
        }
    });
}

void VtkViewport::sync_native_surface(bool force_render) {
    if (impl_->hardware_window == nullptr || window() == nullptr ||
        impl_->native_surface.view == nullptr) {
        return;
    }
    ::panta::visualization::sync_native_surface(window(), this, impl_->hardware_window.get(),
                                                impl_->native_surface);
    if (impl_->render_window == nullptr) {
        return;
    }
    const qreal scale = window()->devicePixelRatio();
    const QSize pixel_size(qMax(1, qRound(width() * scale)), qMax(1, qRound(height() * scale)));
    if (!force_render && pixel_size == impl_->applied_pixel_size) {
        return;
    }
    // 隐藏的原生 surface 不做渲染提交；重新显示时由可见分支强制补帧。
    if (!isVisible()) {
        return;
    }
    impl_->render_window->SetSize(pixel_size.width(), pixel_size.height());
    impl_->render_window->Render();
    impl_->applied_pixel_size = pixel_size;
}

void VtkViewport::destroy_render_window() {
    impl_->applied_pixel_size = QSize();
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
