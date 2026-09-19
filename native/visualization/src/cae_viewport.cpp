/// CaeViewport 实现：持有后端中立的 RenderScene，经 ViewportBackend
/// 适配层驱动渲染；第三方实现细节全部在 src/vtk/。
#include "vtk/vtk_viewport.hpp"
#include <QObject>
#include <QRectF>
#include <memory>
#include <panta/visualization/cae_viewport.hpp>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>

namespace panta::visualization {

struct CaeViewport::Impl {
    RenderScene scene;
    std::unique_ptr<ViewportBackend> backend;
};

CaeViewport::CaeViewport(QQuickItem* parent) : QQuickItem(parent), impl_(std::make_unique<Impl>()) {
    impl_->backend = create_vtk_viewport_backend();
    if (auto* item = impl_->backend->item()) {
        item->setParentItem(this);
        // 后端信号与 QML 面信号对接：sceneInitialized 在渲染线程发出，跨
        // 线程时 Qt 自动走队列投递。具体适配器类型仅在本 cpp（模块内部）
        // 可见，公共头与 QML 面保持后端中立。
        if (auto* viewport = qobject_cast<VtkViewport*>(item)) {
            QObject::connect(viewport, &VtkViewport::sceneInitialized, this,
                             &CaeViewport::sceneReady);
        }
    }
}

CaeViewport::~CaeViewport() = default;

void CaeViewport::componentComplete() {
    QQuickItem::componentComplete();
    // 初次状态提交后，后端在自身渲染线程约束下构建场景；构建完成经
    // sceneReady 通知 GUI（渲染错误走 qWarning，不静默）。
    impl_->scene.revision = 1;
    impl_->backend->apply_state(impl_->scene);
}

void CaeViewport::geometryChange(const QRectF& new_geometry, const QRectF& old_geometry) {
    QQuickItem::geometryChange(new_geometry, old_geometry);
    // 后端条目铺满本条目：resize/高 DPI 由 Qt 场景图按设备像素比缩放。
    if (impl_->backend != nullptr) {
        if (auto* item = impl_->backend->item()) {
            item->setPosition(QPointF(0, 0));
            item->setSize(new_geometry.size());
        }
    }
}

} // namespace panta::visualization
