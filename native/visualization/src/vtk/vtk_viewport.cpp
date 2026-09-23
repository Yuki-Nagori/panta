/// VtkViewport 实现：在 GUI 线程管理 VTK WebGPU 与原生 surface。
///
/// Qt Quick 只负责 QQuickItem 宿主的几何和可见性；VTK 的 render window
/// 直接绘制到平台 surface，避免把 OpenGL/WebGPU 资源混入 scenegraph。
#include "vtk_viewport.hpp"

#include "default_wordmark.hpp"
#include "mesh_source.hpp"
#include "navigation/viewport_camera.hpp"
#include "navigation/viewport_input.hpp"
#include "navigation/viewport_orientation.hpp"
#include "surface_mesh.hpp"
#include "vtk_native_surface.hpp"
#include <QElapsedTimer>
#include <QGuiApplication>
#include <QList>
#include <QLoggingCategory>
#include <QMetaObject>
#include <QQuickItem>
#include <QQuickWindow>
#include <QRectF>
#include <QSize>
#include <QString>
#include <QTimer>
#include <QWindow>
#include <QtCore/qnamespace.h>
#include <QtCore/qtmetamacros.h>
#include <QtGlobal>
#include <QtLogging>
#include <algorithm>
#include <array>
#include <cmath>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>
#include <vtkActor.h>
#include <vtkCamera.h>
#include <vtkCommand.h>
#include <vtkHardwareWindow.h>
#include <vtkNew.h>
#include <vtkObject.h>
#include <vtkPolyDataMapper.h>
#include <vtkPolyDataNormals.h>
#include <vtkProperty.h>
#if defined(Q_OS_MACOS)
#include <vtkCocoaRenderWindowInteractor.h>
#else
#include <vtkGenericRenderWindowInteractor.h>
#endif
#include <vtkRenderWindowInteractor.h>
#include <vtkSmartPointer.h>
#include <vtkWebGPURenderWindow.h>
#include <vtkWebGPURenderer.h>

namespace panta::visualization {

namespace {

Q_LOGGING_CATEGORY(viewport_log, "panta.viewport", QtWarningMsg)

constexpr int kCameraTransitionDurationMs = 260;

using Vector3 = std::array<double, 3>;

// 字样保持透视深度；按实际宽高比留出 25% 边距，窄窗口也不裁字。
void configure_default_camera(vtkWebGPURenderer* renderer, vtkActor* actor, double aspect) {
    double bounds[6];
    actor->GetBounds(bounds);
    constexpr double view_angle = 30.0;
    constexpr double half_angle_radians = 0.2617993877991494;
    const double half_width = (bounds[1] - bounds[0]) / 2;
    const double half_height = (bounds[3] - bounds[2]) / 2;
    const double half_depth = (bounds[5] - bounds[4]) / 2;
    const double distance =
        1.25 * std::max(half_height, half_width / aspect) / std::tan(half_angle_radians) +
        half_depth;
    vtkCamera* camera = renderer->GetActiveCamera();
    const double* center = actor->GetCenter();
    camera->SetPosition(center[0], center[1], center[2] + distance);
    camera->SetFocalPoint(center);
    camera->SetViewUp(0, 1, 0);
    camera->SetViewAngle(view_angle);
    renderer->ResetCameraClippingRange();
}

} // namespace

class ViewportInteractionCommand final : public vtkCommand {
  public:
    static ViewportInteractionCommand* New() { return new ViewportInteractionCommand; }

    void set_viewport(VtkViewport* viewport) { viewport_ = viewport; }

    void Execute(vtkObject* caller, unsigned long event_id, void*) override {
        auto* interactor = vtkRenderWindowInteractor::SafeDownCast(caller);
        if (viewport_ != nullptr && interactor != nullptr) {
            viewport_->handle_interaction_event(event_id, interactor);
        }
    }

  private:
    VtkViewport* viewport_ = nullptr;
};

struct VtkViewport::Impl {
    RenderScene pending;
    NativeSurface native_surface;
    NativeHardwareWindow hardware_window;
    vtkSmartPointer<vtkWebGPURenderWindow> render_window;
    vtkSmartPointer<vtkWebGPURenderer> renderer;
    vtkSmartPointer<vtkRenderWindowInteractor> interactor;
    vtkSmartPointer<vtkActor> primitive_actor;
    ViewportOrientation orientation;
    QMetaObject::Connection window_visibility_connection;
    QMetaObject::Connection scene_graph_initialized_connection;
    QList<QMetaObject::Connection> ancestor_connections;
    QTimer camera_transition_timer;
    QElapsedTimer camera_transition_clock;
    ViewportCameraTransition camera_transition;
    /// 最近一次提交给 render window 的像素尺寸；空值表示尚无有效提交。
    QSize applied_pixel_size;
    bool refresh_scheduled = false;
    bool scene_dirty = true;
    bool creation_warning_emitted = false;
    std::shared_ptr<const SurfaceMeshSnapshot> applied_mesh;
    bool pointer_dragging = false;
    bool cube_pressed = false;
    int last_pointer_x = 0;
    int last_pointer_y = 0;
};

VtkViewport::VtkViewport(QQuickItem* parent) : QQuickItem(parent), impl_(std::make_unique<Impl>()) {
    setFlag(ItemHasContents, false);
    impl_->camera_transition_timer.setInterval(16);
    impl_->camera_transition_timer.setTimerType(Qt::PreciseTimer);
    connect(&impl_->camera_transition_timer, &QTimer::timeout, this,
            &VtkViewport::advance_camera_transition);
    connect(this, &QQuickItem::windowChanged, this, &VtkViewport::bind_window);
    bind_window(window());
    watch_ancestors();
}

void VtkViewport::bind_window(QQuickWindow* new_window) {
    QObject::disconnect(impl_->window_visibility_connection);
    QObject::disconnect(impl_->scene_graph_initialized_connection);
    // 原生 surface 绑定具体窗口，换宿主或脱离窗口时必须释放旧资源。
    destroy_render_window();
    impl_->creation_warning_emitted = false;
    if (new_window == nullptr) {
        return;
    }
    impl_->window_visibility_connection = QObject::connect(
        new_window, &QWindow::visibilityChanged, this, [this](QWindow::Visibility visibility) {
            if (visibility == QWindow::Hidden || visibility == QWindow::Minimized) {
                stop_camera_transition();
            }
            impl_->scene_dirty |= visibility != QWindow::Hidden && visibility != QWindow::Minimized;
            schedule_refresh();
        });
    impl_->scene_graph_initialized_connection = QObject::connect(
        new_window, &QQuickWindow::sceneGraphInitialized, this, [this]() { schedule_refresh(); });
    schedule_refresh();
}

VtkViewport::~VtkViewport() {
    // QQuickItem 基类析构会发 windowChanged，此时 impl_ 已被销毁；先断开自回调。
    QObject::disconnect(this, nullptr, this, nullptr);
    destroy_render_window();
}

QQuickItem* VtkViewport::item() { return this; }

void VtkViewport::apply_state(const RenderScene& state) {
    if (state.revision < impl_->pending.revision) {
        return;
    }
    impl_->scene_dirty |= state.background != impl_->pending.background ||
                          state.primitive_visible != impl_->pending.primitive_visible ||
                          state.mesh != impl_->pending.mesh;
    impl_->pending = state;
    schedule_refresh();
}

void VtkViewport::geometryChange(const QRectF& new_geometry, const QRectF& old_geometry) {
    QQuickItem::geometryChange(new_geometry, old_geometry);
    // 零尺寸时 surface 被隐藏；恢复到相同像素尺寸也需要重新提交 buffer。
    impl_->scene_dirty |= old_geometry.isEmpty();
    schedule_refresh();
}

void VtkViewport::itemChange(ItemChange change, const ItemChangeData& value) {
    QQuickItem::itemChange(change, value);
    if (change == ItemParentHasChanged) {
        watch_ancestors();
    } else if (change == ItemVisibleHasChanged) {
        // 隐藏立即撤下 surface；恢复显示时补帧，Wayland 据此重新映射 buffer。
        if (!isVisible()) {
            stop_camera_transition();
            set_native_surface_visible(impl_->native_surface, false);
        }
        impl_->scene_dirty = true;
        schedule_refresh();
    } else if (change == ItemDevicePixelRatioHasChanged) {
        schedule_refresh();
    }
}

void VtkViewport::watch_ancestors() {
    for (const auto& connection : impl_->ancestor_connections) {
        QObject::disconnect(connection);
    }
    impl_->ancestor_connections.clear();
    // 原生 surface 使用 scene 坐标；祖先平移不会触发本条目的 geometryChange。
    for (auto* ancestor = parentItem(); ancestor != nullptr; ancestor = ancestor->parentItem()) {
        impl_->ancestor_connections.append(
            connect(ancestor, &QQuickItem::xChanged, this, &VtkViewport::schedule_refresh));
        impl_->ancestor_connections.append(
            connect(ancestor, &QQuickItem::yChanged, this, &VtkViewport::schedule_refresh));
        impl_->ancestor_connections.append(
            connect(ancestor, &QQuickItem::parentChanged, this, &VtkViewport::watch_ancestors));
    }
    schedule_refresh();
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
    if (!window()->isVisible() || !isVisible()) {
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

    update_mesh_actor();
    impl_->applied_mesh = impl_->pending.mesh;
    impl_->renderer->AddActor(impl_->primitive_actor);
    impl_->render_window->AddRenderer(impl_->renderer);
    impl_->orientation.attach(impl_->render_window);

#if defined(Q_OS_MACOS)
    impl_->interactor = vtkSmartPointer<vtkCocoaRenderWindowInteractor>::New();
#else
    impl_->interactor = vtkSmartPointer<vtkGenericRenderWindowInteractor>::New();
#endif
    impl_->interactor->SetRenderWindow(impl_->render_window);
    impl_->interactor->SetHardwareWindow(impl_->hardware_window.get());
    impl_->interactor->SetEnableRender(false);
    impl_->hardware_window->SetInteractor(impl_->interactor);
    auto interaction_command = vtkSmartPointer<ViewportInteractionCommand>::New();
    interaction_command->set_viewport(this);
    impl_->interactor->AddObserver(vtkCommand::LeftButtonPressEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::RightButtonPressEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::MouseMoveEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::LeftButtonReleaseEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::RightButtonReleaseEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::MouseWheelForwardEvent, interaction_command);
    impl_->interactor->AddObserver(vtkCommand::MouseWheelBackwardEvent, interaction_command);
    impl_->render_window->Initialize();
    impl_->interactor->Initialize();

    if (impl_->render_window->GetGenericContext() == nullptr) {
        warn_once("VtkViewport: VTK WebGPU device 初始化失败");
        destroy_render_window();
        return;
    }

    Q_EMIT sceneInitialized();
}

void VtkViewport::schedule_refresh() {
    // QWindow 状态信号在 Qt 尚未完成原生窗口切换时发出，窗口级
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

void VtkViewport::update_mesh_actor() {
    if (impl_->primitive_actor == nullptr) {
        return;
    }

    vtkSmartPointer<vtkPolyData> geometry;
    const bool imported_mesh = impl_->pending.mesh != nullptr;
    geometry =
        imported_mesh ? make_surface_poly_data(*impl_->pending.mesh) : create_default_wordmark();

    vtkNew<vtkPolyDataNormals> normals;
    normals->SetInputData(geometry);
    normals->SetFeatureAngle(45.0);
    normals->ConsistencyOn();
    normals->SplittingOn();
    vtkNew<vtkPolyDataMapper> mapper;
    mapper->SetInputConnection(normals->GetOutputPort());
    if (imported_mesh) {
        mapper->SetColorModeToDefault();
        mapper->SetScalarModeToDefault();
    } else {
        mapper->SetColorModeToDirectScalars();
        mapper->SetScalarModeToUsePointData();
    }
    impl_->primitive_actor->SetMapper(mapper);
    if (imported_mesh) {
        impl_->primitive_actor->SetOrientation(0.0, 0.0, 0.0);
    } else {
        impl_->primitive_actor->SetOrientation(16.0, -18.0, -4.0);
    }
    auto* material = impl_->primitive_actor->GetProperty();
    material->SetInterpolationToPhong();
    material->SetAmbient(0.25);
    material->SetDiffuse(0.75);
    material->SetSpecular(0.32);
    material->SetSpecularPower(36.0);
    if (imported_mesh) {
        material->SetColor(0.72, 0.82, 0.94);
    }
}

void VtkViewport::handle_interaction_event(unsigned long event_id,
                                           vtkRenderWindowInteractor* interactor) {
    if (impl_->renderer == nullptr || impl_->primitive_actor == nullptr ||
        impl_->render_window == nullptr || interactor == nullptr) {
        return;
    }

    const int* event_position = interactor->GetEventPosition();
    const int* render_size = impl_->render_window->GetSize();
    if (event_position == nullptr || render_size == nullptr || render_size[0] <= 0 ||
        render_size[1] <= 0) {
        return;
    }

    const int x = event_position[0];
    const int y = event_position[1];
    const int width = render_size[0];
    const int height = render_size[1];
    const auto action = classify_viewport_input(event_id, impl_->pointer_dragging);
    if (action == ViewportInputAction::ZoomIn || action == ViewportInputAction::ZoomOut) {
        stop_camera_transition();
        auto* camera = impl_->renderer->GetActiveCamera();
        ViewportCamera::zoom(camera, impl_->primitive_actor, action == ViewportInputAction::ZoomIn);
        impl_->renderer->ResetCameraClippingRange();
        impl_->orientation.update(camera);
        impl_->scene_dirty = true;
        schedule_refresh();
        return;
    }

    if (action == ViewportInputAction::PressCube) {
        impl_->cube_pressed = impl_->orientation.contains_cube(x, y, width, height);
        impl_->pointer_dragging = false;
        return;
    }

    if (action == ViewportInputAction::BeginRotation) {
        impl_->last_pointer_x = x;
        impl_->last_pointer_y = y;
        impl_->cube_pressed = false;
        impl_->pointer_dragging = true;
        stop_camera_transition();
        return;
    }

    if (action == ViewportInputAction::MoveRotation) {
        const int delta_x = x - impl_->last_pointer_x;
        const int delta_y = y - impl_->last_pointer_y;
        if (delta_x != 0 || delta_y != 0) {
            auto* camera = impl_->renderer->GetActiveCamera();
            camera->Azimuth(-static_cast<double>(delta_x) * 0.5);
            camera->Elevation(static_cast<double>(delta_y) * 0.5);
            camera->OrthogonalizeViewUp();
            impl_->renderer->ResetCameraClippingRange();
            impl_->orientation.update(camera);
            impl_->scene_dirty = true;
            impl_->last_pointer_x = x;
            impl_->last_pointer_y = y;
            schedule_refresh();
        }
        return;
    }

    if (action == ViewportInputAction::EndRotation) {
        impl_->pointer_dragging = false;
        impl_->cube_pressed = false;
        return;
    }
    if (action != ViewportInputAction::ReleaseCube) {
        return;
    }

    if (impl_->cube_pressed) {
        const auto direction = impl_->orientation.cube_direction(x, y, width, height);
        if (direction.has_value()) {
            const double* center = impl_->primitive_actor->GetCenter();
            const Vector3 focal_point = {center[0], center[1], center[2]};
            double direction_vector[3] = {0.0, 0.0, 1.0};
            double view_up[3] = {0.0, 0.0, 1.0};
            switch (*direction) {
            case CubeDirection::PositiveX:
                direction_vector[0] = 1.0;
                direction_vector[2] = 0.0;
                break;
            case CubeDirection::NegativeX:
                direction_vector[0] = -1.0;
                direction_vector[2] = 0.0;
                break;
            case CubeDirection::PositiveY:
                direction_vector[1] = 1.0;
                direction_vector[2] = 0.0;
                break;
            case CubeDirection::NegativeY:
                direction_vector[1] = -1.0;
                direction_vector[2] = 0.0;
                break;
            case CubeDirection::PositiveZ:
                view_up[1] = 1.0;
                view_up[2] = 0.0;
                break;
            case CubeDirection::NegativeZ:
                direction_vector[2] = -1.0;
                view_up[1] = 1.0;
                view_up[2] = 0.0;
                break;
            }
            auto* camera = impl_->renderer->GetActiveCamera();
            const CameraFrame target_frame{
                {direction_vector[0], direction_vector[1], direction_vector[2]},
                {view_up[0], view_up[1], view_up[2]}};
            if (impl_->camera_transition.start(camera, target_frame, focal_point,
                                               impl_->primitive_actor)) {
                // 从相机当前实姿态开始采样，连续点击会平滑重定向而不跳帧。
                impl_->camera_transition_clock.restart();
                impl_->camera_transition_timer.start();
            } else {
                stop_camera_transition();
            }
        }
    }
    impl_->pointer_dragging = false;
    impl_->cube_pressed = false;
}

void VtkViewport::advance_camera_transition() {
    if (impl_->renderer == nullptr || impl_->primitive_actor == nullptr ||
        impl_->render_window == nullptr || !impl_->camera_transition_clock.isValid() ||
        !impl_->camera_transition.active()) {
        stop_camera_transition();
        return;
    }

    const qint64 elapsed = impl_->camera_transition_clock.elapsed();
    auto* camera = impl_->renderer->GetActiveCamera();
    ViewportCamera::apply(camera, impl_->camera_transition.pose_at(static_cast<double>(elapsed),
                                                                   kCameraTransitionDurationMs));
    impl_->renderer->ResetCameraClippingRange();
    impl_->orientation.update(camera);
    impl_->scene_dirty = true;
    schedule_refresh();
    if (elapsed >= kCameraTransitionDurationMs) {
        stop_camera_transition();
    }
}

void VtkViewport::stop_camera_transition() {
    impl_->camera_transition_timer.stop();
    impl_->camera_transition_clock.invalidate();
    impl_->camera_transition.cancel();
}

void VtkViewport::sync_native_surface() {
    if (impl_->hardware_window == nullptr || window() == nullptr ||
        impl_->native_surface.view == nullptr) {
        return;
    }
    const bool visible = isVisible() && width() > 0 && height() > 0;
    set_native_surface_visible(impl_->native_surface, visible);
    if (!visible || impl_->render_window == nullptr) {
        stop_camera_transition();
        return;
    }
    ::panta::visualization::sync_native_surface(window(), this, impl_->hardware_window.get(),
                                                impl_->native_surface);
    qCDebug(viewport_log) << "surface synchronized";
    const qreal scale = window()->devicePixelRatio();
    const QSize pixel_size(qMax(1, qRound(width() * scale)), qMax(1, qRound(height() * scale)));
    bool resized = pixel_size != impl_->applied_pixel_size;
    if (impl_->primitive_actor != nullptr && impl_->pending.mesh != impl_->applied_mesh) {
        update_mesh_actor();
        impl_->applied_mesh = impl_->pending.mesh;
        resized = true;
    }
    if (!impl_->scene_dirty && !resized) {
        return;
    }
    if (resized) {
        stop_camera_transition();
        impl_->render_window->SetSize(pixel_size.width(), pixel_size.height());
        if (impl_->interactor != nullptr) {
            impl_->interactor->UpdateSize(pixel_size.width(), pixel_size.height());
        }
        configure_default_camera(impl_->renderer, impl_->primitive_actor,
                                 static_cast<double>(pixel_size.width()) / pixel_size.height());
    }
    impl_->orientation.update(impl_->renderer->GetActiveCamera());
    impl_->renderer->SetBackground(impl_->pending.background.redF(),
                                   impl_->pending.background.greenF(),
                                   impl_->pending.background.blueF());
    impl_->primitive_actor->SetVisibility(impl_->pending.primitive_visible);
    impl_->render_window->Render();
    qCDebug(viewport_log) << "frame submitted" << pixel_size;
    impl_->applied_pixel_size = pixel_size;
    impl_->scene_dirty = false;
}

void VtkViewport::destroy_render_window() {
    stop_camera_transition();
    impl_->applied_pixel_size = {};
    impl_->scene_dirty = true;
    impl_->pointer_dragging = false;
    impl_->cube_pressed = false;
    if (impl_->interactor != nullptr) {
        impl_->interactor->Disable();
        impl_->interactor->SetRenderWindow(nullptr);
    }
    if (impl_->hardware_window != nullptr) {
        impl_->hardware_window->SetInteractor(nullptr);
    }
    impl_->orientation.detach();
    if (impl_->render_window != nullptr) {
        impl_->render_window->Finalize();
        impl_->render_window = nullptr;
    }
    if (impl_->native_surface.view != nullptr) {
        detach_native_surface(impl_->native_surface);
    }
    impl_->renderer = nullptr;
    impl_->interactor = nullptr;
    impl_->primitive_actor = nullptr;
    impl_->applied_mesh.reset();
    if (impl_->hardware_window != nullptr) {
        impl_->hardware_window->Destroy();
        impl_->hardware_window.reset();
    }
}

std::unique_ptr<ViewportBackend> create_vtk_viewport_backend() {
    return std::make_unique<VtkViewport>();
}

} // namespace panta::visualization
