#include "edit_command_router.hpp"

#include <QGuiApplication>
#include <QMetaMethod>
#include <QMetaProperty>
#include <QObject>
#include <QVariant>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <qtmetamacros.h>

namespace panta::app {

EditCommandRouter::EditCommandRouter(QObject* parent) : QObject(parent) {
    if (auto* application = qobject_cast<QGuiApplication*>(QGuiApplication::instance())) {
        connect(application, &QGuiApplication::focusObjectChanged, this,
                &EditCommandRouter::refreshFocusObject);
    }
    refreshFocusObject();
}

bool EditCommandRouter::canUndo() const {
    return hasMethod("undo()") && !boolProperty("readOnly") && boolProperty("canUndo");
}

bool EditCommandRouter::canRedo() const {
    return hasMethod("redo()") && !boolProperty("readOnly") && boolProperty("canRedo");
}

bool EditCommandRouter::canCut() const {
    return hasMethod("cut()") && !boolProperty("readOnly") &&
           !stringProperty("selectedText").isEmpty();
}

bool EditCommandRouter::canCopy() const {
    return hasMethod("copy()") && !stringProperty("selectedText").isEmpty();
}

bool EditCommandRouter::canPaste() const {
    return hasMethod("paste()") && !boolProperty("readOnly") && boolProperty("canPaste");
}

bool EditCommandRouter::canSelectAll() const {
    return hasMethod("selectAll()") && intProperty("length") > 0;
}

void EditCommandRouter::undo() {
    if (canUndo()) {
        invoke("undo");
    }
}

void EditCommandRouter::redo() {
    if (canRedo()) {
        invoke("redo");
    }
}

void EditCommandRouter::cut() {
    if (canCut()) {
        invoke("cut");
    }
}

void EditCommandRouter::copy() {
    if (canCopy()) {
        invoke("copy");
    }
}

void EditCommandRouter::paste() {
    if (canPaste()) {
        invoke("paste");
    }
}

void EditCommandRouter::selectAll() {
    if (canSelectAll()) {
        invoke("selectAll");
    }
}

void EditCommandRouter::refreshFocusObject() {
    auto* focusObject = QGuiApplication::focusObject();
    if (m_focusObject == focusObject) {
        refreshState();
        return;
    }

    if (m_focusObject) {
        disconnect(m_focusObject, nullptr, this, nullptr);
    }
    m_focusObject = focusObject;
    if (focusObject) {
        const auto refreshMethod =
            metaObject()->method(metaObject()->indexOfSlot("refreshState()"));
        for (const char* name :
             {"canUndo", "canRedo", "canPaste", "readOnly", "selectedText", "length"}) {
            const int index = focusObject->metaObject()->indexOfProperty(name);
            if (index < 0) {
                continue;
            }
            const auto notifySignal = focusObject->metaObject()->property(index).notifySignal();
            if (notifySignal.isValid()) {
                QObject::connect(focusObject, notifySignal, this, refreshMethod,
                                 Qt::UniqueConnection);
            }
        }
        connect(focusObject, &QObject::destroyed, this, &EditCommandRouter::refreshFocusObject);
    }
    refreshState();
}

void EditCommandRouter::refreshState() { emit stateChanged(); }

bool EditCommandRouter::hasMethod(const char* method) const {
    return m_focusObject && m_focusObject->metaObject()->indexOfMethod(method) >= 0;
}

bool EditCommandRouter::boolProperty(const char* property) const {
    if (!m_focusObject) {
        return false;
    }
    const int index = m_focusObject->metaObject()->indexOfProperty(property);
    return index >= 0 && m_focusObject->metaObject()->property(index).read(m_focusObject).toBool();
}

QString EditCommandRouter::stringProperty(const char* property) const {
    if (!m_focusObject) {
        return {};
    }
    const int index = m_focusObject->metaObject()->indexOfProperty(property);
    return index >= 0 ? m_focusObject->metaObject()->property(index).read(m_focusObject).toString()
                      : QString{};
}

int EditCommandRouter::intProperty(const char* property) const {
    if (!m_focusObject) {
        return 0;
    }
    const int index = m_focusObject->metaObject()->indexOfProperty(property);
    return index >= 0 ? m_focusObject->metaObject()->property(index).read(m_focusObject).toInt()
                      : 0;
}

void EditCommandRouter::invoke(const char* method) {
    if (m_focusObject) {
        QMetaObject::invokeMethod(m_focusObject, method, Qt::DirectConnection);
    }
}

} // namespace panta::app
