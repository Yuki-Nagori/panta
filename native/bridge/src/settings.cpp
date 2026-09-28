#include "settings.hpp"

#include "panta_ffi.h"
#include "rust/cxx.h"
#include <QByteArray>
#include <QLatin1Char>
#include <QQmlEngine>
#include <QString>
#include <QStringList>
#include <cstddef>
#include <memory>
#include <optional>
#include <panta/qt_adapter/system_motion_preference.hpp>
#include <panta/qt_adapter/translation_catalog.hpp>
#include <qcontainerfwd.h>
#include <qobject.h>
#include <qqml.h>
#include <qtmetamacros.h>
#include <string>

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

} // namespace

/// 只转换 QString/CXX 字符串并调用 Rust locale 服务；不持有 Qt 翻译目录。
class LanguageServiceClient final {
  public:
    LanguageServiceClient() : m_service(panta::ffi::language_service_new()) {}

    [[nodiscard]] QString language() const {
        return QString::fromUtf8(panta::ffi::language_service_current(*m_service));
    }

    [[nodiscard]] QStringList availableLanguages() const {
        QStringList locales;
        const auto supported = panta::ffi::language_service_supported_locales();
        locales.reserve(static_cast<qsizetype>(supported.size()));
        for (const auto& locale : supported) {
            locales.push_back(QString::fromUtf8(locale));
        }
        return locales;
    }

    [[nodiscard]] std::optional<QString> validate(const QString& candidate, QString* error) const {
        std::string utf8;
        if (!toUtf8(candidate, &utf8, error)) {
            return std::nullopt;
        }
        try {
            return QString::fromUtf8(panta::ffi::language_service_validate(*m_service, utf8));
        } catch (const rust::Error& failure) {
            *error = QString::fromUtf8(failure.what());
            return std::nullopt;
        }
    }

    [[nodiscard]] bool commit(const QString& candidate, QString* error) {
        std::string utf8;
        if (!toUtf8(candidate, &utf8, error)) {
            return false;
        }
        try {
            const auto committed =
                QString::fromUtf8(panta::ffi::language_service_commit(*m_service, utf8));
            if (committed == candidate) {
                return true;
            }
            *error = QStringLiteral("language.commit_mismatch");
            return false;
        } catch (const rust::Error& failure) {
            *error = QString::fromUtf8(failure.what());
            return false;
        }
    }

  private:
    [[nodiscard]] static bool toUtf8(const QString& value, std::string* utf8, QString* error) {
        const QByteArray encoded = value.toUtf8();
        if (QString::fromUtf8(encoded) != value) {
            *error = QStringLiteral("language.invalid_locale_encoding");
            return false;
        }
        *utf8 = std::string(encoded.constData(), static_cast<std::size_t>(encoded.size()));
        return true;
    }

    rust::Box<panta::ffi::LanguageService> m_service;
};

Settings::Settings(QObject* parent)
    : QObject(parent), m_preference(std::make_unique<panta::qt_adapter::SystemMotionPreference>()),
      m_translation(std::make_unique<panta::qt_adapter::TranslationCatalog>()),
      m_languageService(std::make_unique<LanguageServiceClient>()) {
    connect(m_preference.get(), &panta::qt_adapter::SystemMotionPreference::reducedMotionChanged,
            this, &Settings::reducedMotionChanged);
}

Settings::~Settings() = default;

bool Settings::reducedMotion() const { return m_preference->reducedMotion(); }

QString Settings::language() const { return m_languageService->language(); }

QStringList Settings::availableLanguages() const { return m_languageService->availableLanguages(); }

QString Settings::languageError() const { return m_languageError; }

bool Settings::setLanguage(const QString& locale) {
    QString error;
    const auto candidate = m_languageService->validate(locale, &error);
    if (!candidate.has_value()) {
        setLanguageError(error);
        return false;
    }
    const QString previous = language();
    if (*candidate == previous) {
        setLanguageError({});
        return true;
    }

    if (!activateTranslation(*m_translation, *candidate, &error)) {
        setLanguageError(error);
        return false;
    }

    if (!m_languageService->commit(*candidate, &error)) {
        QString stateRollbackError;
        if (!m_languageService->commit(previous, &stateRollbackError)) {
            error = QStringLiteral("language.state_rollback_failed: %1").arg(stateRollbackError);
        }
        QString translationRollbackError;
        if (!activateTranslation(*m_translation, previous, &translationRollbackError)) {
            error = QStringLiteral("language.rollback_failed: %1").arg(translationRollbackError);
        }
        retranslate();
        setLanguageError(error);
        return false;
    }

    setLanguageError({});
    emit languageChanged();
    retranslate();
    return true;
}

void Settings::retranslate() {
    if (auto* engine = qmlEngine(this); engine != nullptr) {
        engine->retranslate();
    }
}

void Settings::setLanguageError(const QString& error) {
    if (m_languageError == error) {
        return;
    }
    m_languageError = error;
    emit languageErrorChanged();
}

} // namespace panta::bridge
