#include "window_chrome_metrics.hpp"

#include <QEvent>
#include <QObject>
#include <QQuickWindow>
#include <QWindow>
#include <QtCore/qtmetamacros.h>

namespace panta {

WindowChromeMetrics::WindowChromeMetrics(QObject* parent) : QObject(parent) {}

void WindowChromeMetrics::observeWindow(QQuickWindow* window) {
    if (window_ == window) {
        return;
    }
    if (window_ != nullptr) {
        window_->removeEventFilter(this);
    }
    for (const auto& connection : windowConnections_) {
        QObject::disconnect(connection);
    }
    windowConnections_.clear();
    window_ = window;
    if (window != nullptr) {
        window->installEventFilter(this);
        const auto notify = [this] { emit leadingSystemControlsInsetChanged(); };
        windowConnections_.append(connect(window, &QWindow::visibilityChanged, this, notify));
        windowConnections_.append(connect(window, &QWindow::windowStateChanged, this, notify));
        windowConnections_.append(connect(window, &QWindow::screenChanged, this, notify));
        windowConnections_.append(connect(window, &QObject::destroyed, this, [this] {
            window_ = nullptr;
            emit leadingSystemControlsInsetChanged();
        }));
    }
    emit leadingSystemControlsInsetChanged();
}

bool WindowChromeMetrics::eventFilter(QObject* watched, QEvent* event) {
    if (watched == window_ && event->type() == QEvent::Expose) {
        emit leadingSystemControlsInsetChanged();
    }
    return QObject::eventFilter(watched, event);
}

#ifndef Q_OS_MACOS
auto WindowChromeMetrics::leadingSystemControlsInset() const -> qreal { return 0; }
#endif

} // namespace panta
