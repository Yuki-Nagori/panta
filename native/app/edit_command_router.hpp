/// 将 macOS 编辑菜单命令路由到 Qt Quick 当前焦点文本控件。
#pragma once

#include <QObject>
#include <QPointer>
#include <QString>
#include <QtQml/qqmlregistration.h>

namespace panta::app {

class EditCommandRouter final : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY stateChanged)
    Q_PROPERTY(bool canRedo READ canRedo NOTIFY stateChanged)
    Q_PROPERTY(bool canCut READ canCut NOTIFY stateChanged)
    Q_PROPERTY(bool canCopy READ canCopy NOTIFY stateChanged)
    Q_PROPERTY(bool canPaste READ canPaste NOTIFY stateChanged)
    Q_PROPERTY(bool canSelectAll READ canSelectAll NOTIFY stateChanged)

  public:
    explicit EditCommandRouter(QObject* parent = nullptr);

    [[nodiscard]] bool canUndo() const;
    [[nodiscard]] bool canRedo() const;
    [[nodiscard]] bool canCut() const;
    [[nodiscard]] bool canCopy() const;
    [[nodiscard]] bool canPaste() const;
    [[nodiscard]] bool canSelectAll() const;

    Q_INVOKABLE void undo();
    Q_INVOKABLE void redo();
    Q_INVOKABLE void cut();
    Q_INVOKABLE void copy();
    Q_INVOKABLE void paste();
    Q_INVOKABLE void selectAll();

  signals:
    void stateChanged();

  private slots:
    void refreshFocusObject();
    void refreshState();

  private:
    [[nodiscard]] bool hasMethod(const char* method) const;
    [[nodiscard]] bool boolProperty(const char* property) const;
    [[nodiscard]] QString stringProperty(const char* property) const;
    [[nodiscard]] int intProperty(const char* property) const;
    void invoke(const char* method);

    QPointer<QObject> m_focusObject;
};

} // namespace panta::app
