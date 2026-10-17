#include "shell_view_model.hpp"

#include <QCoreApplication>
#include <QObject>
#include <QString>
#include <QtCore/qtmetamacros.h>

namespace panta::bridge {

ShellViewModel::ShellViewModel(QObject* parent) : QObject(parent) {}

auto ShellViewModel::caption() const -> const QString& { return m_caption; }

void ShellViewModel::setCaption(const QString& value) {
    m_captionUsesRevision = false;
    if (m_caption == value) {
        // 值未变化不重复通知（qt.md）；QML 重复绑定同一值时不得抖动。
        return;
    }
    m_caption = value;
    emit captionChanged();
}

auto ShellViewModel::count() const -> int { return m_count; }

auto ShellViewModel::error() const -> const QString& { return m_error; }

void ShellViewModel::setError(const QString& message) {
    if (m_error == message) {
        return;
    }
    m_error = message;
    emit errorChanged();
}

void ShellViewModel::tick() {
    ++m_count;
    m_captionUsesRevision = true;
    retranslateCaption();
    emit countChanged();
}

void ShellViewModel::retranslateCaption() {
    if (!m_captionUsesRevision) {
        return;
    }
    const QString caption =
        QCoreApplication::translate("ShellViewModel", "Revision %1").arg(m_count);
    if (m_caption == caption) {
        return;
    }
    m_caption = caption;
    emit captionChanged();
}

} // namespace panta::bridge
