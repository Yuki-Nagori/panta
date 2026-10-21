#include "theme_view_model.hpp"

#include "panta_ffi.h"
#include <QtCore/qbytearray.h>
#include <QtCore/qfile.h>
#include <QtCore/qglobal.h>
#include <QtCore/qlogging.h>
#include <QtCore/qobject.h>
#include <QtCore/qstring.h>
#include <QtCore/qtmetamacros.h>
#include <QtCore/qvariant.h>
#include <QtGui/qcolor.h>
#include <cstddef>
#include <rust/cxx.h>
#include <string>
#include <utility>

namespace panta::app {

namespace {

QString readThemeResource(const QString& path) {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly)) {
        qFatal("Unable to read built-in theme resource %s: %s", qPrintable(path),
               qPrintable(file.errorString()));
    }
    const QByteArray bytes = file.readAll();
    const QString source = QString::fromUtf8(bytes);
    if (source.toUtf8() != bytes) {
        qFatal("Built-in theme resource is not valid UTF-8: %s", qPrintable(path));
    }
    return source;
}

QString fromRustString(const rust::String& value) {
    return QString::fromUtf8(value.data(), static_cast<qsizetype>(value.size()));
}

} // namespace

ThemeViewModel::ThemeViewModel(QObject* parent)
    : QObject(parent),
      m_service(panta::ffi::theme_service_new(
          toRustString(
              readThemeResource(QStringLiteral(":/qt/qml/Panta/Shell/Themes/gui-default.pa"))),
          {rust::String("light"), rust::String("dark")},
          {toRustString(readThemeResource(QStringLiteral(":/qt/qml/Panta/Shell/Themes/light.pa"))),
           toRustString(readThemeResource(QStringLiteral(":/qt/qml/Panta/Shell/Themes/dark.pa")))},
          rust::String("light"))) {
    const auto ids = panta::ffi::theme_service_theme_ids(*m_service);
    m_availableThemes.reserve(static_cast<qsizetype>(ids.size()));
    for (const auto& id : ids) {
        m_availableThemes.push_back(fromRustString(id));
    }
    const auto snapshot = panta::ffi::theme_service_current(*m_service);
    m_tokens = toVariantMap(snapshot);
    m_activeTheme = fromRustString(snapshot.id);
}

bool ThemeViewModel::selectTheme(const QString& id) {
    if (id == m_activeTheme) {
        setError({});
        return true;
    }
    try {
        const auto snapshot = panta::ffi::theme_service_select(*m_service, toRustString(id));
        const QVariantMap candidateTokens = toVariantMap(snapshot);
        const QString candidateId = fromRustString(snapshot.id);
        const bool tokensChangedValue = candidateTokens != m_tokens;
        const bool activeThemeChangedValue = candidateId != m_activeTheme;
        m_tokens = candidateTokens;
        m_activeTheme = candidateId;
        setError({});
        if (tokensChangedValue) {
            emit tokensChanged();
        }
        if (activeThemeChangedValue) {
            emit activeThemeChanged();
        }
        return true;
    } catch (const rust::Error& failure) {
        setError(QString::fromUtf8(failure.what()));
        return false;
    }
}

rust::String ThemeViewModel::toRustString(const QString& value) {
    const QByteArray bytes = value.toUtf8();
    // QString::toUtf8 会把不配对的 UTF-16 代理项替换为 U+FFFD；Rust 会将非 ASCII 主题 ID
    // 作为普通选择错误拒绝。
    return rust::String(std::string(bytes.constData(), static_cast<std::size_t>(bytes.size())));
}

QVariantMap ThemeViewModel::toVariantMap(const panta::ffi::ThemeSnapshot& snapshot) {
    QVariantMap values;
    for (const auto& token : snapshot.tokens) {
        const QString name = fromRustString(token.qml_name);
        const QString type = fromRustString(token.value_type);
        const QString value = fromRustString(token.value);
        QVariant qtValue;
        if (type == QStringLiteral("color")) {
            const QColor color(value);
            if (!color.isValid()) {
                qFatal("Rust theme schema returned an invalid QColor for %s", qPrintable(name));
            }
            qtValue = color;
        } else if (type == QStringLiteral("integer")) {
            bool valid = false;
            const qlonglong number = value.toLongLong(&valid);
            if (!valid) {
                qFatal("Rust theme schema returned an invalid integer for %s", qPrintable(name));
            }
            qtValue = QVariant::fromValue(number);
        } else if (type == QStringLiteral("real")) {
            bool valid = false;
            const double number = value.toDouble(&valid);
            if (!valid) {
                qFatal("Rust theme schema returned an invalid real for %s", qPrintable(name));
            }
            qtValue = number;
        } else if (type == QStringLiteral("boolean")) {
            if (value != QStringLiteral("true") && value != QStringLiteral("false")) {
                qFatal("Rust theme schema returned an invalid boolean for %s", qPrintable(name));
            }
            qtValue = value == QStringLiteral("true");
        } else {
            qFatal("Rust theme schema returned an unknown token type for %s", qPrintable(name));
        }
        values.insert(name, qtValue);
    }
    return values;
}

void ThemeViewModel::setError(QString error) {
    if (m_error == error) {
        return;
    }
    m_error = std::move(error);
    emit errorChanged();
}

} // namespace panta::app
