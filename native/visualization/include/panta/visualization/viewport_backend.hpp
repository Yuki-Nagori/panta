/// 视口后端适配层（任务 007，standards/vtk.md 与 architecture/
/// visualization.md 的后端可替换边界）：应用与 QML 只消费本接口，
/// 第三方渲染集成（V1 = VTK）收敛在 src/vtk/ 适配器内。
///
/// 实现契约：
/// - item() 返回承载渲染的场景图条目，由 CaeViewport 作为子条目布局；
/// - apply_state() 在 GUI 线程调用，实现必须以自身渲染线程安全的方式
///   应用，并按 RenderScene::revision 拒绝迟到更新；
/// - 渲染错误经 qWarning 上报，不得静默表现为“已成功”。
#pragma once

#include <memory>

class QQuickItem;

namespace panta::visualization {

struct RenderScene;

class ViewportBackend {
  public:
    virtual ~ViewportBackend() = default;

    /// 承载渲染的场景图条目（后端自有 QQuickItem 实现）。
    virtual QQuickItem* item() = 0;

    /// GUI 线程：提交最新场景状态（含修订号）。
    virtual void apply_state(const RenderScene& state) = 0;
};

/// 应用对象构造前的图形环境准备（图形 API 与表面格式）。中性入口，
/// 当前由 VTK 适配器实现 QQuickVTKItem::setGraphicsApi 的语义。
void prepare_graphics_environment();

/// 后端工厂：更换或新增渲染后端只改此处与对应适配器实现。
std::unique_ptr<ViewportBackend> create_vtk_viewport_backend();

} // namespace panta::visualization
