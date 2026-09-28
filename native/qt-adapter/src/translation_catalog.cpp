#include "panta/qt_adapter/translation_catalog.hpp"

#include <QCoreApplication>
#include <QTranslator>
#include <memory>
#include <utility>

namespace panta::qt_adapter {

TranslationCatalog::~TranslationCatalog() { clear(); }

bool TranslationCatalog::install(const QString& resourcePath, QString* error) {
    auto candidate = std::make_unique<QTranslator>();
    if (!candidate->load(resourcePath)) {
        if (error != nullptr) {
            *error = QStringLiteral("i18n.catalog_load_failed");
        }
        return false;
    }
    if (!QCoreApplication::installTranslator(candidate.get())) {
        if (error != nullptr) {
            *error = QStringLiteral("i18n.catalog_install_failed");
        }
        return false;
    }

    auto previous = std::move(m_translator);
    m_translator = std::move(candidate);
    if (previous != nullptr) {
        QCoreApplication::removeTranslator(previous.get());
    }
    if (error != nullptr) {
        error->clear();
    }
    return true;
}

void TranslationCatalog::clear() {
    if (m_translator == nullptr) {
        return;
    }
    QCoreApplication::removeTranslator(m_translator.get());
    m_translator.reset();
}

} // namespace panta::qt_adapter
