/// VtkViewport：VTK WebGPU 后端适配器。
///
/// 该类型只作为 QQuickItem 宿主参与布局；VTK 不进入 Qt Quick scenegraph。
/// 原生 view/layer/surface 的创建和尺寸同步由 vtk_native_surface 平台桥接完成。
#pragma once

#include <QQuickItem>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>

class vtkRenderWindowInteractor;

namespace panta::visualization {

class ViewportInteractionCommand;

class VtkViewport final : public QQuickItem, public ViewportBackend {
    Q_OBJECT

  public:
    explicit VtkViewport(QQuickItem* parent = nullptr);
    ~VtkViewport() override;

    QQuickItem* item() override;
    void apply_state(const RenderScene& state) override;

  signals:
    /// 原生 surface 与最小 VTK 场景已建立。
    void sceneInitialized();

  protected:
    void geometryChange(const QRectF& new_geometry, const QRectF& old_geometry) override;
    void itemChange(ItemChange change, const ItemChangeData& value) override;

  private:
    friend class ViewportInteractionCommand;

    void bind_window(QQuickWindow* window);
    void ensure_render_window();
    void schedule_refresh();
    void watch_ancestors();
    void update_mesh_actor();
    void handle_interaction_event(unsigned long event_id, vtkRenderWindowInteractor* interactor);
    void advance_camera_transition();
    void stop_camera_transition();
    /// 排队刷新时同步原生区域，仅状态/像素尺寸改变或恢复显示时提交帧。
    void sync_native_surface();
    void destroy_render_window();

    struct Impl;
    std::unique_ptr<Impl> impl_;
};

} // namespace panta::visualization
