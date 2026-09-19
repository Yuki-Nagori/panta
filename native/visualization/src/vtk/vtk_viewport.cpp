/// VtkViewport 实现：渲染线程回调内创建/更新/释放 VTK 管线。
/// 线程与生命周期约束见 vtk_viewport.hpp 与 standards/vtk.md。
#include "vtk_viewport.hpp"
#include <QObject>
#include <QQuickItem>
#include <QQuickVTKItem.h>
#include <QtCore/qtmetamacros.h>
#include <QtGlobal>
#include <QtLogging>
#include <memory>
#include <mutex>
#include <panta/visualization/render_scene.hpp>
#include <panta/visualization/viewport_backend.hpp>
#include <vtkActor.h>
#include <vtkNew.h>
#include <vtkObject.h>
#include <vtkObjectFactory.h>
#include <vtkPolyDataMapper.h>
#include <vtkRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkSphereSource.h>

namespace panta::visualization {
namespace {

/// vtkUserData 载体：承载渲染线程的管线对象，经 SafeDownCast 恢复。
/// 生命周期由 QQuickVTKItem 的 vtkSmartPointer 持有，销毁在渲染线程。
class VtkSceneData final : public vtkObject {
  public:
    static VtkSceneData* New();
    // vtkTypeMacro 生成的 NewInstance 按宏契约遮蔽基类版本，属 VTK 惯例；
    // 宏本体无独立头可包含，include-cleaner 一并抑制。
    // clang-format off
    vtkTypeMacro(VtkSceneData, vtkObject); // NOLINT(bugprone-derived-method-shadowing-base-method,misc-include-cleaner)
    // clang-format on

    vtkNew<vtkRenderer> renderer;
    vtkNew<vtkActor> triangle_actor;
    SceneRevision applied_revision = 0;

  private:
    VtkSceneData() = default;
    ~VtkSceneData() override = default;
};

vtkStandardNewMacro(VtkSceneData);

} // namespace

VtkViewport::VtkViewport(QQuickItem* parent) : QQuickVTKItem(parent) {}

VtkViewport::~VtkViewport() = default;

QQuickItem* VtkViewport::item() { return this; }

void VtkViewport::apply_state(const RenderScene& state) {
    {
        std::scoped_lock lock(mutex_);
        pending_ = state;
    }
    // 应用统一发生在渲染线程回调；初始化完成前提交的命令由集成层排队。
    dispatch_async([this](vtkRenderWindow* render_window, const vtkUserData& user_data) {
        auto* data = VtkSceneData::SafeDownCast(user_data);
        if (data == nullptr) {
            qWarning("VtkViewport: dispatch_async 收到未知场景数据，忽略");
            return;
        }
        RenderScene pending;
        {
            std::scoped_lock lock(mutex_);
            pending = pending_;
        }
        // 迟到更新拒绝：渲染线程已应用的修订更高时丢弃旧提交。
        if (pending.revision < data->applied_revision) {
            return;
        }
        data->applied_revision = pending.revision;
        data->renderer->SetBackground(pending.background.redF(), pending.background.greenF(),
                                      pending.background.blueF());
        data->triangle_actor->SetVisibility(pending.primitive_visible);
        render_window->Render();
    });
}

QQuickVTKItem::vtkUserData VtkViewport::initializeVTK(vtkRenderWindow* render_window) {
    auto* data = VtkSceneData::New();

    // 测试图元（007 范围：小型测试数据，完整网格后处理为非目标）。
    // SDK 未登记 FiltersSources 全集，实际头文件核对后选用 vtkSphereSource。
    vtkNew<vtkSphereSource> primitive;
    primitive->SetThetaResolution(24);
    primitive->SetPhiResolution(16);
    vtkNew<vtkPolyDataMapper> mapper;
    mapper->SetInputConnection(primitive->GetOutputPort());
    data->triangle_actor->SetMapper(mapper);
    data->renderer->AddActor(data->triangle_actor);

    RenderScene initial;
    {
        std::scoped_lock lock(mutex_);
        initial = pending_;
    }
    data->renderer->SetBackground(initial.background.redF(), initial.background.greenF(),
                                  initial.background.blueF());
    data->applied_revision = initial.revision;
    render_window->AddRenderer(data->renderer);
    // 渲染线程内发出：QML 侧经队列连接收到“场景已构建”通知。
    Q_EMIT sceneInitialized();
    return data;
}

void VtkViewport::destroyingVTK(vtkRenderWindow* render_window, vtkUserData user_data) {
    auto* data = VtkSceneData::SafeDownCast(user_data);
    if (data == nullptr) {
        qWarning("VtkViewport: destroyingVTK 收到未知场景数据，忽略");
        return;
    }
    // 仅解除渲染窗口关联；VTK 对象生命周期由 data 的引用计数在渲染线程
    // 结束（QQuickVTKItem 契约），GUI/worker 不触碰。
    render_window->RemoveRenderer(data->renderer);
}

void prepare_graphics_environment() {
    // 必须在 QGuiApplication 构造前调用（native/app/main.cpp）。
    VtkViewport::setGraphicsApi();
}

std::unique_ptr<ViewportBackend> create_vtk_viewport_backend() {
    return std::make_unique<VtkViewport>();
}

} // namespace panta::visualization
