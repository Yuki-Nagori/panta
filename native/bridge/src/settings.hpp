/// QML 设置入口：把 Rust 拥有的 locale 状态与 Qt 平台偏好适配为稳定属性。
#pragma once

#include <QObject>
#include <QStringList>
#include <QtQml/qqmlregistration.h>
#include <memory>

namespace panta::qt_adapter {
class SystemMotionPreference;
class TranslationCatalog;
} // namespace panta::qt_adapter

namespace panta::bridge {

class LanguageServiceClient;

/// 应用 QML 引擎内唯一的设置入口；避免多个实例持有互相冲突的语言状态。
class Settings : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool reducedMotion READ reducedMotion NOTIFY reducedMotionChanged)
    Q_PROPERTY(QString language READ language NOTIFY languageChanged)
    Q_PROPERTY(QStringList availableLanguages READ availableLanguages CONSTANT)
    Q_PROPERTY(QString languageError READ languageError NOTIFY languageErrorChanged)

  public:
    explicit Settings(QObject* parent = nullptr);
    ~Settings() override;

    [[nodiscard]] bool reducedMotion() const;
    [[nodiscard]] QString language() const;
    [[nodiscard]] QStringList availableLanguages() const;
    [[nodiscard]] QString languageError() const;

    /// 加载失败时保留上一份 translator 与 Rust locale。
    Q_INVOKABLE bool setLanguage(const QString& locale);

  signals:
    void reducedMotionChanged();
    void languageChanged();
    void languageErrorChanged();

  private:
    void setLanguageError(const QString& error);
    void retranslate();

    std::unique_ptr<panta::qt_adapter::SystemMotionPreference> m_preference;
    std::unique_ptr<panta::qt_adapter::TranslationCatalog> m_translation;
    std::unique_ptr<LanguageServiceClient> m_languageService;
    QString m_languageError;
};

} // namespace panta::bridge
