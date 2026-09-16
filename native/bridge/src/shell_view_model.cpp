#include "shell_view_model.hpp"

namespace panta::bridge {

ShellViewModel::ShellViewModel(QObject* parent) : QObject(parent) {}

auto ShellViewModel::caption() const -> const QString& {
    return m_caption;
}

void ShellViewModel::setCaption(const QString& value) {
    if (m_caption == value) {
        // 值未变化不重复通知（qt.md）；QML 重复绑定同一值时不得抖动。
        return;
    }
    m_caption = value;
    emit captionChanged();
}

auto ShellViewModel::count() const -> int {
    return m_count;
}

auto ShellViewModel::error() const -> const QString& {
    return m_error;
}

void ShellViewModel::setError(const QString& message) {
    if (m_error == message) {
        return;
    }
    m_error = message;
    emit errorChanged();
}

void ShellViewModel::tick() {
    ++m_count;
    setCaption(QStringLiteral("修订 %1").arg(m_count));
    emit countChanged();
}

}  // namespace panta::bridge
