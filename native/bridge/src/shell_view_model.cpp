#include "shell_view_model.hpp"

#include <QObject>
#include <QString>
#include <QtCore/qtmetamacros.h>

namespace panta::bridge {

ShellViewModel::ShellViewModel(QObject* parent) : QObject(parent) {}

bool ShellViewModel::reducedMotion() const {
    // TODO(task 080): 系统动效偏好源待接入。Windows 需经
    // SPI_GETCLIENTAREAANIMATION（windows.h 伞头在本仓库 clang-cl TU 的
    // granular/伞头组合尚不稳定，见 standards/comments.md 例外条款），
    // 接入时另行为该 TU 登记包含策略；当前一律返回 false（动画照播）。
    return false;
}

auto ShellViewModel::caption() const -> const QString& { return m_caption; }

void ShellViewModel::setCaption(const QString& value) {
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
    setCaption(QStringLiteral("修订 %1").arg(m_count));
    emit countChanged();
}

} // namespace panta::bridge
