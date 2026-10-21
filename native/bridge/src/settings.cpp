#include "settings.hpp"

#include "panta_ffi.h"
#include <QByteArray>
#include <QDebug>
#include <QLatin1Char>
#include <QObject>
#include <QQmlEngine>
#include <QSettings>
#include <QSize>
#include <QString>
#include <QtCore/qlogging.h>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <panta/qt_adapter/system_motion_preference.hpp>
#include <panta/qt_adapter/translation_catalog.hpp>
#include <panta/qt_adapter/user_settings_store.hpp>
#include <qcontainerfwd.h>
#include <qqml.h>
#include <qtmetamacros.h>
#include <rust/cxx.h>
#include <string>
#include <utility>

namespace panta::bridge {

namespace {

QString catalogResourcePath(const QString& locale) {
    QString resourceLocale = locale;
    resourceLocale.replace(QLatin1Char('-'), QLatin1Char('_'));
    return QStringLiteral(":/i18n/panta_%1.qm").arg(resourceLocale);
}

bool activateTranslation(panta::qt_adapter::TranslationCatalog& translation, const QString& locale,
                         QString* error) {
    if (locale == QStringLiteral("en")) {
        translation.clear();
        return true;
    }
    return translation.install(catalogResourcePath(locale), error);
}

QString fromRustString(const rust::String& value) {
    return QString::fromUtf8(value.data(), static_cast<qsizetype>(value.size()));
}

bool toRustString(const QString& value, std::string* result, QString* error) {
    const QByteArray encoded = value.toUtf8();
    if (QString::fromUtf8(encoded) != value) {
        *error = QStringLiteral("settings.invalid_utf8");
        return false;
    }
    *result = std::string(encoded.constData(), static_cast<std::size_t>(encoded.size()));
    return true;
}

} // namespace

Settings::Settings(QObject* parent) : Settings(QString{}, parent) {}

Settings::Settings(QString settingsFileName, QObject* parent)
    : QObject(parent), m_preference(std::make_unique<panta::qt_adapter::SystemMotionPreference>()),
      m_translation(std::make_unique<panta::qt_adapter::TranslationCatalog>()),
      m_store(settingsFileName.isEmpty() ? std::make_unique<panta::qt_adapter::UserSettingsStore>()
                                         : std::make_unique<panta::qt_adapter::UserSettingsStore>(
                                               std::move(settingsFileName), QSettings::IniFormat)),
      m_service(panta::ffi::settings_service_new()) {
    connect(m_preference.get(), &panta::qt_adapter::SystemMotionPreference::reducedMotionChanged,
            this, &Settings::reducedMotionChanged);
    static_cast<void>(loadStoredSettings());
}

Settings::~Settings() = default;

bool Settings::reducedMotion() const { return m_preference->reducedMotion(); }

QString Settings::language() const {
    return fromRustString(panta::ffi::settings_service_current(*m_service).language);
}

QStringList Settings::availableLanguages() const {
    QStringList locales;
    const auto supported = panta::ffi::settings_service_supported_languages();
    locales.reserve(static_cast<qsizetype>(supported.size()));
    for (const auto& locale : supported) {
        locales.push_back(fromRustString(locale));
    }
    return locales;
}

QString Settings::settingsError() const { return m_settingsError; }

QString Settings::themeId() const {
    return fromRustString(panta::ffi::settings_service_current(*m_service).theme_id);
}

int Settings::windowWidth() const {
    return static_cast<int>(panta::ffi::settings_service_current(*m_service).window_width);
}

int Settings::windowHeight() const {
    return static_cast<int>(panta::ffi::settings_service_current(*m_service).window_height);
}

bool Settings::windowMaximized() const {
    return panta::ffi::settings_service_current(*m_service).window_maximized;
}

QStringList Settings::recentProjectPaths() const {
    QStringList paths;
    const auto snapshot = panta::ffi::settings_service_current(*m_service);
    paths.reserve(static_cast<qsizetype>(snapshot.recent_project_paths.size()));
    for (const auto& path : snapshot.recent_project_paths) {
        paths.push_back(fromRustString(path));
    }
    return paths;
}

bool Settings::setLanguage(const QString& locale) {
    auto candidate = currentSnapshot();
    std::string encoded;
    QString error;
    if (!toRustString(locale, &encoded, &error)) {
        setSettingsError(error);
        return false;
    }
    candidate.language = rust::String(encoded);

    panta::ffi::UserSettingsSnapshot validated;
    try {
        validated = panta::ffi::settings_service_validate(*m_service, candidate);
    } catch (const rust::Error& failure) {
        setSettingsError(QString::fromUtf8(failure.what()));
        return false;
    }
    const QString nextLocale = fromRustString(validated.language);
    const QString previousLocale = language();
    if (nextLocale == previousLocale) {
        return m_settingsError.isEmpty();
    }
    if (!activateTranslation(*m_translation, nextLocale, &error)) {
        setSettingsError(error);
        return false;
    }
    if (!commitCandidate(validated)) {
        QString rollbackError;
        if (!activateTranslation(*m_translation, previousLocale, &rollbackError)) {
            setSettingsError(
                QStringLiteral("settings.translation_rollback_failed: %1").arg(rollbackError));
        }
        retranslate();
        return false;
    }
    emit languageChanged();
    retranslate();
    return true;
}

bool Settings::setThemeId(const QString& id) {
    auto candidate = currentSnapshot();
    std::string encoded;
    QString error;
    if (!toRustString(id, &encoded, &error)) {
        setSettingsError(error);
        return false;
    }
    candidate.theme_id = rust::String(encoded);
    return commitCandidate(candidate);
}

bool Settings::saveWindowState(const QSize& size, bool maximized) {
    auto candidate = currentSnapshot();
    candidate.window_width = size.width() < 0 ? 0U : static_cast<std::uint32_t>(size.width());
    candidate.window_height = size.height() < 0 ? 0U : static_cast<std::uint32_t>(size.height());
    candidate.window_maximized = maximized;
    return commitCandidate(candidate);
}

bool Settings::recordRecentProject(const QString& path) {
    std::string encoded;
    QString error;
    if (!toRustString(path, &encoded, &error)) {
        setSettingsError(error);
        return false;
    }
    try {
        const auto candidate = panta::ffi::settings_service_candidate_recent_project(
            *m_service, rust::String(encoded));
        return commitCandidate(candidate);
    } catch (const rust::Error& failure) {
        setSettingsError(QString::fromUtf8(failure.what()));
        return false;
    }
}

void Settings::setSettingsError(const QString& error) {
    if (m_settingsError == error) {
        return;
    }
    m_settingsError = error;
    if (!error.isEmpty()) {
        qWarning().noquote() << "Application settings:" << error;
    }
    emit settingsErrorChanged();
}

void Settings::retranslate() {
    if (auto* engine = qmlEngine(this); engine != nullptr) {
        engine->retranslate();
    }
}

panta::ffi::UserSettingsSnapshot Settings::currentSnapshot() const {
    return panta::ffi::settings_service_current(*m_service);
}

bool Settings::commitCandidate(const panta::ffi::UserSettingsSnapshot& candidate) {
    panta::ffi::UserSettingsSnapshot validated;
    try {
        validated = panta::ffi::settings_service_validate(*m_service, candidate);
    } catch (const rust::Error& failure) {
        setSettingsError(QString::fromUtf8(failure.what()));
        return false;
    }
    if (!m_persistenceAvailable) {
        if (m_settingsError.isEmpty()) {
            setSettingsError(QStringLiteral("settings.persistence_disabled"));
        }
        return false;
    }
    if (validated == currentSnapshot()) {
        return m_settingsError.isEmpty();
    }

    QString error;
    if (!m_store->save(toRecord(validated), &error)) {
        setSettingsError(error);
        return false;
    }
    try {
        panta::ffi::settings_service_commit(*m_service, validated);
    } catch (const rust::Error& failure) {
        setSettingsError(QString::fromUtf8(failure.what()));
        return false;
    }

    setSettingsError({});
    emit preferencesChanged();
    return true;
}

bool Settings::loadStoredSettings() {
    panta::qt_adapter::UserSettingsRecord record;
    bool found = false;
    QString error;
    if (!m_store->load(&record, &found, &error)) {
        m_persistenceAvailable = false;
        setSettingsError(error);
        return false;
    }
    if (!found) {
        return true;
    }

    try {
        panta::ffi::UserSettingsSnapshot stored;
        if (!fromRecord(record, &stored, &error)) {
            m_persistenceAvailable = false;
            setSettingsError(error);
            return false;
        }
        const auto candidate = panta::ffi::settings_service_validate(*m_service, stored);
        const QString storedLocale = fromRustString(candidate.language);
        if (!activateTranslation(*m_translation, storedLocale, &error)) {
            m_persistenceAvailable = false;
            setSettingsError(error);
            return false;
        }
        panta::ffi::settings_service_commit(*m_service, candidate);
    } catch (const rust::Error& failure) {
        m_persistenceAvailable = false;
        setSettingsError(QString::fromUtf8(failure.what()));
        return false;
    }
    return true;
}

panta::qt_adapter::UserSettingsRecord
Settings::toRecord(const panta::ffi::UserSettingsSnapshot& snapshot) {
    panta::qt_adapter::UserSettingsRecord record;
    record.schemaVersion = snapshot.schema_version;
    record.themeId = fromRustString(snapshot.theme_id);
    record.language = fromRustString(snapshot.language);
    record.windowWidth = snapshot.window_width;
    record.windowHeight = snapshot.window_height;
    record.windowMaximized = snapshot.window_maximized;
    for (const auto& path : snapshot.recent_project_paths) {
        record.recentProjectPaths.push_back(fromRustString(path));
    }
    return record;
}

bool Settings::fromRecord(const panta::qt_adapter::UserSettingsRecord& record,
                          panta::ffi::UserSettingsSnapshot* snapshot, QString* error) {
    if (snapshot == nullptr || error == nullptr) {
        return false;
    }
    std::string themeId;
    std::string language;
    if (!toRustString(record.themeId, &themeId, error) ||
        !toRustString(record.language, &language, error)) {
        return false;
    }
    snapshot->schema_version = record.schemaVersion;
    snapshot->theme_id = rust::String(themeId);
    snapshot->language = rust::String(language);
    snapshot->window_width = record.windowWidth;
    snapshot->window_height = record.windowHeight;
    snapshot->window_maximized = record.windowMaximized;
    for (const auto& path : record.recentProjectPaths) {
        std::string encoded;
        if (!toRustString(path, &encoded, error)) {
            return false;
        }
        snapshot->recent_project_paths.push_back(rust::String(encoded));
    }
    return true;
}

} // namespace panta::bridge
