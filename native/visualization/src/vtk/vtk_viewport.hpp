/// VtkViewport：VTK 后端适配器——本仓库 VTK 专属代码的唯一收敛点。
/// 基于 GUISupportQtQuick 的 QQuickVTKItem：VTK 状态只在渲染线程的
/// initializeVTK / destroyingVTK / dispatch_async 回调内触碰；GUI 经
/// apply_state 提交 RenderScene，回调按 revision 拒绝迟到更新。
/// 应用/QML 不感知本类型（standards/vtk.md：VTK 不是公共 API）。
#pragma once

#include <QQuickVTKItem.h>
#include <memory>
#include <mutex>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>

class vtkRenderWindow;

namespace panta::visualization {

class VtkViewport final : public QQuickVTKItem, public ViewportBackend {
    Q_OBJECT

  public:
    explicit VtkViewport(QQuickItem* parent = nullptr);
    ~VtkViewport() override;

    // ViewportBackend
    QQuickItem* item() override;
    void apply_state(const RenderScene& state) override;

  protected:
    // QQuickVTKItem 渲染线程回调（vtkUserData = vtkSmartPointer<vtkObject>）
    vtkUserData initializeVTK(vtkRenderWindow* render_window) override;
    void destroyingVTK(vtkRenderWindow* render_window, vtkUserData user_data) override;

  signals:
    /// 渲染线程管线搭建完成后发出（跨线程队列投递到 GUI）。
    void sceneInitialized();

  private:
    /// GUI 提交、渲染线程消费的待应用状态；revision 高者为新。
    std::mutex mutex_;
    RenderScene pending_{};
};

} // namespace panta::visualization
