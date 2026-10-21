#include "panta/qt_adapter/user_settings_store.hpp"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>
#include <QMetaType>
#include <QSettings>
#include <QVariant>
#include <array>
#include <cmath>
#include <limits>
#include <qbytearrayview.h>
#include <qcontainerfwd.h>
#include <utility>

namespace panta::qt_adapter {

namespace {

constexpr auto kGroup = "preferences";
constexpr auto kSnapshot = "snapshot";
constexpr auto kSchemaVersion = "schemaVersion";
constexpr auto kThemeId = "themeId";
constexpr auto kLanguage = "language";
constexpr auto kWindowWidth = "windowWidth";
constexpr auto kWindowHeight = "windowHeight";
constexpr auto kWindowMaximized = "windowMaximized";
constexpr auto kRecentProjectPaths = "recentProjectPaths";
constexpr std::array<const char*, 7> kFields{kSchemaVersion,     kThemeId,      kLanguage,
                                             kWindowWidth,       kWindowHeight, kWindowMaximized,
                                             kRecentProjectPaths};

QString statusMessage(QSettings::Status status, const QString& detail) {
    QString key;
    switch (status) {
    case QSettings::NoError:
        return {};
    case QSettings::AccessError:
        key = QStringLiteral("settings.storage_access_error");
        break;
    case QSettings::FormatError:
        key = QStringLiteral("settings.storage_format_error");
        break;
    }
    return detail.isEmpty() ? key : QStringLiteral("%1: %2").arg(key, detail);
}

QJsonObject toJson(const UserSettingsRecord& record) {
    QJsonArray recentPaths;
    for (const QString& path : record.recentProjectPaths) {
        recentPaths.append(path);
    }
    return {{QLatin1StringView(kSchemaVersion), static_cast<double>(record.schemaVersion)},
            {QLatin1StringView(kThemeId), record.themeId},
            {QLatin1StringView(kLanguage), record.language},
            {QLatin1StringView(kWindowWidth), static_cast<double>(record.windowWidth)},
            {QLatin1StringView(kWindowHeight), static_cast<double>(record.windowHeight)},
            {QLatin1StringView(kWindowMaximized), record.windowMaximized},
            {QLatin1StringView(kRecentProjectPaths), recentPaths}};
}

bool hasExactFields(const QJsonObject& object) {
    if (object.size() != static_cast<qsizetype>(kFields.size())) {
        return false;
    }
    for (const char* field : kFields) {
        if (!object.contains(QLatin1StringView(field))) {
            return false;
        }
    }
    return true;
}

bool toUnsignedInteger(const QJsonValue& value, quint32* result) {
    if (result == nullptr || !value.isDouble()) {
        return false;
    }
    const double number = value.toDouble();
    if (!std::isfinite(number) || number < 0 || std::floor(number) != number ||
        number > std::numeric_limits<quint32>::max()) {
        return false;
    }
    *result = static_cast<quint32>(number);
    return true;
}

bool fromJson(const QJsonObject& object, UserSettingsRecord* record) {
    if (record == nullptr || !hasExactFields(object)) {
        return false;
    }
    const QJsonValue themeId = object.value(QLatin1StringView(kThemeId));
    const QJsonValue language = object.value(QLatin1StringView(kLanguage));
    const QJsonValue maximized = object.value(QLatin1StringView(kWindowMaximized));
    const QJsonValue recent = object.value(QLatin1StringView(kRecentProjectPaths));
    quint32 schemaVersion = 0;
    quint32 windowWidth = 0;
    quint32 windowHeight = 0;
    QStringList recentPaths;
    if (!toUnsignedInteger(object.value(QLatin1StringView(kSchemaVersion)), &schemaVersion) ||
        !themeId.isString() || !language.isString() ||
        !toUnsignedInteger(object.value(QLatin1StringView(kWindowWidth)), &windowWidth) ||
        !toUnsignedInteger(object.value(QLatin1StringView(kWindowHeight)), &windowHeight) ||
        !maximized.isBool() || !recent.isArray()) {
        return false;
    }
    for (const auto& value : recent.toArray()) {
        if (!value.isString()) {
            return false;
        }
        recentPaths.push_back(value.toString());
    }

    record->schemaVersion = schemaVersion;
    record->themeId = themeId.toString();
    record->language = language.toString();
    record->windowWidth = windowWidth;
    record->windowHeight = windowHeight;
    record->windowMaximized = maximized.toBool();
    record->recentProjectPaths = std::move(recentPaths);
    return true;
}

} // namespace

UserSettingsStore::UserSettingsStore() = default;

UserSettingsStore::UserSettingsStore(QString fileName, QSettings::Format format)
    : m_fileName(std::move(fileName)), m_format(format) {}

UserSettingsStore::UserSettingsStore(QString organizationName, QString applicationName)
    : m_organizationName(std::move(organizationName)),
      m_applicationName(std::move(applicationName)) {}

QSettings UserSettingsStore::createSettings() const {
    if (!m_fileName.isEmpty()) {
        return QSettings(m_fileName, m_format);
    }
    return QSettings(QSettings::NativeFormat, QSettings::UserScope,
                     m_organizationName.isEmpty() ? QStringLiteral("Yuki-Nagori")
                                                  : m_organizationName,
                     m_applicationName.isEmpty() ? QStringLiteral("Panta") : m_applicationName);
}

bool UserSettingsStore::load(UserSettingsRecord* record, bool* found, QString* error) const {
    if (record == nullptr || found == nullptr || error == nullptr) {
        return false;
    }
    auto settings = createSettings();
    settings.sync();
    if (settings.status() != QSettings::NoError) {
        *error = statusMessage(settings.status(), settings.fileName());
        return false;
    }

    settings.beginGroup(QLatin1StringView(kGroup));
    const QStringList keys = settings.allKeys();
    *found = !keys.isEmpty();
    if (!*found) {
        return true;
    }
    for (const QString& key : keys) {
        if (key != QLatin1StringView(kSnapshot)) {
            *error = QStringLiteral("settings.unknown_key: %1").arg(key);
            return false;
        }
    }
    if (keys.size() != 1) {
        *error = QStringLiteral("settings.missing_key: snapshot");
        return false;
    }

    const QVariant snapshot = settings.value(QLatin1StringView(kSnapshot));
    if (snapshot.metaType().id() != QMetaType::QString) {
        *error = QStringLiteral("settings.invalid_stored_type");
        return false;
    }
    QJsonParseError parseError;
    const QJsonDocument document =
        QJsonDocument::fromJson(snapshot.toString().toUtf8(), &parseError);
    UserSettingsRecord parsed;
    if (parseError.error != QJsonParseError::NoError || !document.isObject() ||
        !fromJson(document.object(), &parsed)) {
        *error = QStringLiteral("settings.invalid_stored_type");
        return false;
    }
    *record = std::move(parsed);
    return true;
}

bool UserSettingsStore::save(const UserSettingsRecord& record, QString* error) const {
    if (error == nullptr) {
        return false;
    }
    auto settings = createSettings();
    settings.setAtomicSyncRequired(true);
    const QString snapshot =
        QString::fromUtf8(QJsonDocument(toJson(record)).toJson(QJsonDocument::Compact));
    settings.beginGroup(QLatin1StringView(kGroup));
    settings.setValue(QLatin1StringView(kSnapshot), snapshot);
    settings.endGroup();
    settings.sync();
    if (settings.status() != QSettings::NoError) {
        *error = statusMessage(settings.status(), settings.fileName());
        return false;
    }
    return true;
}

} // namespace panta::qt_adapter
