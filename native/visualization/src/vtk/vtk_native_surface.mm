/// macOS Cocoa/Metal surface bridge. VTK owns the Cocoa hardware view and its
/// CAMetalLayer; this file only reparents that view into Qt Quick's native view.
#include "vtk_native_surface.hpp"

#include <QQuickItem>
#include <QQuickWindow>
#include <QPointF>
#include <QtGlobal>
#include <vtkCocoaHardwareWindow.h>
#include <vtkHardwareWindow.h>

#import <Cocoa/Cocoa.h>

namespace panta::visualization {
namespace {

void release_hardware_window(vtkHardwareWindow* hardware) {
    if (hardware != nullptr) {
        hardware->Delete();
    }
}

NSView* qt_native_view(QQuickWindow* window) {
    if (window == nullptr) {
        return nullptr;
    }
    return reinterpret_cast<NSView*>(window->winId());
}

} // namespace

std::unique_ptr<vtkHardwareWindow, void (*)(vtkHardwareWindow*)>
create_native_hardware_window(QQuickWindow*) {
    return {vtkCocoaHardwareWindow::New(), &release_hardware_window};
}

bool attach_native_surface(QQuickWindow* window, QQuickItem*, vtkHardwareWindow* hardware,
                           NativeSurface& surface) {
    auto* cocoa = vtkCocoaHardwareWindow::SafeDownCast(hardware);
    NSView* host = qt_native_view(window);
    NSView* view = cocoa != nullptr ? cocoa->GetViewId() : nullptr;
    if (cocoa == nullptr || host == nullptr || view == nullptr || cocoa->GetMetalLayer() == nullptr) {
        return false;
    }

    [view removeFromSuperview];
    [view setHidden:NO];
    [view setWantsLayer:YES];
    [view setAutoresizingMask:NSViewNotSizable];
    [host addSubview:view positioned:NSWindowAbove relativeTo:nil];
    surface.host = host;
    surface.view = view;
    return true;
}

void sync_native_surface(QQuickWindow*, QQuickItem* item, vtkHardwareWindow* hardware,
                         NativeSurface& surface) {
    auto* host = static_cast<NSView*>(surface.host);
    auto* view = static_cast<NSView*>(surface.view);
    auto* cocoa = vtkCocoaHardwareWindow::SafeDownCast(hardware);
    if (host == nullptr || view == nullptr || cocoa == nullptr || item == nullptr) {
        return;
    }

    const QPointF scene_position = item->mapToScene(QPointF(0, 0));
    const CGFloat width = static_cast<CGFloat>(item->width());
    const CGFloat height = static_cast<CGFloat>(item->height());
    const int logical_width = qMax(1, qRound(item->width()));
    const int logical_height = qMax(1, qRound(item->height()));

    // vtkCocoaHardwareWindow::SetSize() uses Cocoa screen coordinates, not
    // backing pixels. It also updates the VTK-owned NSView frame, so apply it
    // before restoring the QQuickItem position in the host view.
    cocoa->SetSize(logical_width, logical_height);
    const NSRect host_bounds = [host bounds];
    const CGFloat y = host_bounds.size.height - static_cast<CGFloat>(scene_position.y()) - height;
    [view setFrame:NSMakeRect(static_cast<CGFloat>(scene_position.x()), y, width, height)];
}

void set_native_surface_visible(NativeSurface& surface, bool visible) {
    if (auto* view = static_cast<NSView*>(surface.view); view != nullptr) {
        [view setHidden:visible ? NO : YES];
    }
}

void detach_native_surface(NativeSurface& surface) {
    if (auto* view = static_cast<NSView*>(surface.view); view != nullptr) {
        [view removeFromSuperview];
    }
    surface = {};
}

} // namespace panta::visualization
