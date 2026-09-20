/// Qt Quick 原生窗口与 VTK hardware window 的平台桥接。
///
/// 该头文件只暴露 VTK 基类和不透明 surface 状态；AppKit、Win32、Wayland
/// 类型及其生命周期全部留在平台实现中。
#pragma once

#include <memory>

class QQuickItem;
class QQuickWindow;
class vtkHardwareWindow;

namespace panta::visualization {

struct NativeSurface {
    void* host = nullptr;
    void* view = nullptr;
};

/// 为 Qt Quick 窗口创建使用其原生 surface 的 VTK hardware window。
std::unique_ptr<vtkHardwareWindow, void (*)(vtkHardwareWindow*)>
create_native_hardware_window(QQuickWindow* window);

/// 将 hardware window 的 view/layer 接入 Qt Quick 原生窗口。
bool attach_native_surface(QQuickWindow* window, QQuickItem* item, vtkHardwareWindow* hardware,
                           NativeSurface& surface);

void sync_native_surface(QQuickWindow* window, QQuickItem* item, vtkHardwareWindow* hardware,
                         NativeSurface& surface);
void set_native_surface_visible(NativeSurface& surface, bool visible);
void detach_native_surface(NativeSurface& surface);

} // namespace panta::visualization
