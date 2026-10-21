/// QML 设置入口：把 Rust 拥有的 locale 状态与 Qt 平台偏好适配为稳定属性。
#pragma once

#include "panta_ffi.h"
#include <QObject>
#include <QSize>
#include <QStringList>
#include <QtQml/qqmlregistration.h>
#include <memory>
#include <panta/qt_adapter/user_settings_store.hpp>
#include <rust/cxx.h>

namespace panta::qt_adapter {
class SystemMotionPreference;
class TranslationCatalog;
} // namespace panta::qt_adapter

namespace panta::bridge {

/// 应用 QML 引擎内唯一的设置入口；避免多个实例持有互相冲突的语言状态。
class Settings : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool reducedMotion READ reducedMotion NOTIFY reducedMotionChanged)
    Q_PROPERTY(QString language READ language NOTIFY languageChanged)
    Q_PROPERTY(QStringList availableLanguages READ availableLanguages CONSTANT)
    Q_PROPERTY(QString settingsError READ settingsError NOTIFY settingsErrorChanged)
    Q_PROPERTY(QString themeId READ themeId NOTIFY preferencesChanged)
    Q_PROPERTY(int windowWidth READ windowWidth NOTIFY preferencesChanged)
    Q_PROPERTY(int windowHeight READ windowHeight NOTIFY preferencesChanged)
    Q_PROPERTY(bool windowMaximized READ windowMaximized NOTIFY preferencesChanged)
    Q_PROPERTY(QStringList recentProjectPaths READ recentProjectPaths NOTIFY preferencesChanged)

  public:
    explicit Settings(QObject* parent = nullptr);
    explicit Settings(QString settingsFileName, QObject* parent = nullptr);
    ~Settings() override;

    [[nodiscard]] bool reducedMotion() const;
    [[nodiscard]] QString language() const;
    [[nodiscard]] QStringList availableLanguages() const;
    [[nodiscard]] QString settingsError() const;
    [[nodiscard]] QString themeId() const;
    [[nodiscard]] int windowWidth() const;
    [[nodiscard]] int windowHeight() const;
    [[nodiscard]] bool windowMaximized() const;
    [[nodiscard]] QStringList recentProjectPaths() const;

    /// 加载失败时保留上一份 translator 与 Rust locale。
    Q_INVOKABLE bool setLanguage(const QString& locale);
    Q_INVOKABLE bool setThemeId(const QString& id);
    Q_INVOKABLE bool saveWindowState(const QSize& size, bool maximized);
    Q_INVOKABLE bool recordRecentProject(const QString& path);

  signals:
    void reducedMotionChanged();
    void languageChanged();
    void settingsErrorChanged();
    void preferencesChanged();

  private:
    void setSettingsError(const QString& error);
    void retranslate();
    [[nodiscard]] panta::ffi::UserSettingsSnapshot currentSnapshot() const;
    [[nodiscard]] bool commitCandidate(const panta::ffi::UserSettingsSnapshot& candidate);
    [[nodiscard]] bool loadStoredSettings();
    [[nodiscard]] static panta::qt_adapter::UserSettingsRecord
    toRecord(const panta::ffi::UserSettingsSnapshot& snapshot);
    [[nodiscard]] static bool fromRecord(const panta::qt_adapter::UserSettingsRecord& record,
                                         panta::ffi::UserSettingsSnapshot* snapshot,
                                         QString* error);

    std::unique_ptr<panta::qt_adapter::SystemMotionPreference> m_preference;
    std::unique_ptr<panta::qt_adapter::TranslationCatalog> m_translation;
    std::unique_ptr<panta::qt_adapter::UserSettingsStore> m_store;
    rust::Box<panta::ffi::UserSettingsService> m_service;
    QString m_settingsError;
    bool m_persistenceAvailable = true;
};

} // namespace panta::bridge
