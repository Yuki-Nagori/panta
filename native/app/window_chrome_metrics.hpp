#pragma once

#include <QEvent>
#include <QMetaObject>
#include <QObject>
#include <QPointer>
#include <QQuickWindow>
#include <QVector>
#include <QtQml/qqmlregistration.h>

namespace panta {

/// 查询原生窗口控制占用的客户区边距，供标题栏布局避让系统控件。
class WindowChromeMetrics final : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(qreal leadingSystemControlsInset READ leadingSystemControlsInset NOTIFY
                   leadingSystemControlsInsetChanged)

  public:
    explicit WindowChromeMetrics(QObject* parent = nullptr);

    /// 跟踪主窗口生命周期，使原生按钮几何变化时刷新 QML 布局。
    Q_INVOKABLE void observeWindow(QQuickWindow* window);

    /// 返回最左侧系统标题栏控件右边缘相对客户区左侧的位置。
    qreal leadingSystemControlsInset() const;

  signals:
    void leadingSystemControlsInsetChanged();

  private:
    bool eventFilter(QObject* watched, QEvent* event) override;

    QPointer<QQuickWindow> window_;
    QVector<QMetaObject::Connection> windowConnections_;
};

} // namespace panta
