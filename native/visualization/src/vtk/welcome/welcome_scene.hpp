#pragma once

#include <panta/visualization/mesh_source.hpp>
#include <vector>
#include <vtkSmartPointer.h>

class vtkBillboardTextActor3D;
class vtkActor;
class vtkPolyData;
class vtkRenderWindow;
class vtkRenderer;

namespace panta::visualization {

/// 负责 Welcome 字样几何、固定屏幕文案与结果色标的 VTK 场景部件。
class WelcomeScene final {
  public:
    ~WelcomeScene();

    void attach(vtkRenderWindow* render_window);
    void detach();
    void set_visible(bool visible);
    /// 色标使用屏幕相机，不随模型旋转。
    void set_result_legend(const SurfaceMeshSnapshot* mesh, int width, int height);

  private:
    vtkSmartPointer<vtkRenderWindow> render_window_;
    vtkSmartPointer<vtkRenderer> renderer_;
    vtkSmartPointer<vtkBillboardTextActor3D> text_actor_;
    vtkSmartPointer<vtkActor> legend_actor_;
    std::vector<vtkSmartPointer<vtkBillboardTextActor3D>> legend_labels_;
};

/// 创建居中的封闭 panta 字形网格；颜色不映射求解结果，也不持有 GPU 对象。
vtkSmartPointer<vtkPolyData> create_welcome_wordmark();

} // namespace panta::visualization
