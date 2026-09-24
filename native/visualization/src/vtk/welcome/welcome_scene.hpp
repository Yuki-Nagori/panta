#pragma once

#include <vtkSmartPointer.h>

class vtkBillboardTextActor3D;
class vtkPolyData;
class vtkRenderWindow;
class vtkRenderer;

namespace panta::visualization {

/// 负责 Welcome 字样几何、装饰渐变与固定屏幕文案的 VTK 场景部件。
class WelcomeScene final {
  public:
    ~WelcomeScene();

    void attach(vtkRenderWindow* render_window);
    void detach();
    void set_visible(bool visible);

  private:
    vtkSmartPointer<vtkRenderWindow> render_window_;
    vtkSmartPointer<vtkRenderer> renderer_;
    vtkSmartPointer<vtkBillboardTextActor3D> text_actor_;
};

/// 创建居中的封闭 panta 字形网格；颜色不映射求解结果，也不持有 GPU 对象。
vtkSmartPointer<vtkPolyData> create_welcome_wordmark();

} // namespace panta::visualization
