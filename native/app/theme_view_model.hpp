/// 将 Rust 已验证的主题快照转换为 QML 唯一 Theme 门面的 Qt 属性。
#pragma once

#include "panta_ffi.h"
#include <QObject>
#include <QStringList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>
#include <memory>
#include <rust/cxx.h>

namespace panta::app {

class ThemeViewModel final : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(QVariantMap tokens READ tokens NOTIFY tokensChanged)
    Q_PROPERTY(QStringList availableThemes READ availableThemes CONSTANT)
    Q_PROPERTY(QString activeTheme READ activeTheme NOTIFY activeThemeChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

  public:
    explicit ThemeViewModel(QObject* parent = nullptr);

    [[nodiscard]] const QVariantMap& tokens() const { return m_tokens; }
    [[nodiscard]] const QStringList& availableThemes() const { return m_availableThemes; }
    [[nodiscard]] const QString& activeTheme() const { return m_activeTheme; }
    [[nodiscard]] const QString& error() const { return m_error; }

    /// 已内置的主题同步切换；失败时不替换当前 QML 快照。
    Q_INVOKABLE bool selectTheme(const QString& id);

  signals:
    void tokensChanged();
    void activeThemeChanged();
    void errorChanged();

  private:
    [[nodiscard]] static rust::String toRustString(const QString& value);
    [[nodiscard]] static QVariantMap toVariantMap(const panta::ffi::ThemeSnapshot& snapshot);
    void setError(QString error);

    rust::Box<panta::ffi::ThemeService> m_service;
    QVariantMap m_tokens;
    QStringList m_availableThemes;
    QString m_activeTheme;
    QString m_error;
};

} // namespace panta::app
