#include "shell_view_model.hpp"

#include <QObject>
#include <QString>
#include <QtCore/qtmetamacros.h>
#if defined(Q_OS_WIN)
// windows.h 为 SDK 伞头：SPI_GETCLIENTAREAANIMATION 无 granular 头可替代；
// 包含行 NOLINT 与例外理由见 standards/comments.md（登记例外）。
#include <windows.h> // NOLINT(misc-include-cleaner)
#endif

namespace panta::bridge {

ShellViewModel::ShellViewModel(QObject* parent) : QObject(parent) {}

bool ShellViewModel::reducedMotion() const {
#if defined(Q_OS_WIN)
    // 「在 Windows 中动画控件和窗口」即用户所指的系统动效开关；每次
    // 读取即时查询，运行期切换场景极少，不为它维护信号源。
    BOOL animation = TRUE;
    SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &animation, 0);
    return animation == FALSE;
#else
    // macOS / Linux 的系统偏好源待接入；默认不减少动画。
    return false;
#endif
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
