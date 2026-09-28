/// 验证 Settings 跨 Rust locale、Qt translator 与 QML 翻译绑定的行为。
#include "settings.hpp"
#include <QChar>
#include <QCoreApplication>
#include <QJSEngine>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariant>
#include <QtQml/qqml.h>
#include <gtest/gtest.h>
#include <memory>
#include <qobject.h>

namespace {

using panta::bridge::Settings;

TEST(SettingsTest, LanguageSwitchRetranslatesQmlAndInvalidInputPreservesState) {
    const int settingsTypeId = qmlRegisterSingletonType<Settings>(
        "Panta.Test", 1, 0, "Settings",
        [](QQmlEngine*, QJSEngine*) -> QObject* { return new Settings; });
    QQmlEngine engine;
    auto* settings = engine.singletonInstance<Settings*>(settingsTypeId);
    ASSERT_NE(settings, nullptr);

    QQmlComponent textComponent(&engine);
    textComponent.setData(
        "import QtQml\nQtObject { property string label: qsTranslate(\"App\", \"Ready\") }",
        QUrl());
    std::unique_ptr<QObject> textObject(textComponent.create());
    ASSERT_NE(textObject, nullptr) << textComponent.errorString().toStdString();

    EXPECT_EQ(settings->language(), QStringLiteral("en"));
    EXPECT_TRUE(settings->availableLanguages().contains(QStringLiteral("zh-CN")));
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("Ready"));
    QSignalSpy changed(settings, &Settings::languageChanged);
    QSignalSpy errors(settings, &Settings::languageErrorChanged);

    EXPECT_FALSE(settings->setLanguage(QStringLiteral("fr")));
    EXPECT_TRUE(
        settings->languageError().startsWith(QStringLiteral("language.unsupported_locale")));
    EXPECT_EQ(settings->language(), QStringLiteral("en"));
    EXPECT_EQ(changed.count(), 0);
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("Ready"));

    ASSERT_TRUE(settings->setLanguage(QStringLiteral("zh-CN")))
        << settings->languageError().toStdString();
    EXPECT_EQ(settings->language(), QStringLiteral("zh-CN"));
    EXPECT_EQ(changed.count(), 1);
    EXPECT_EQ(errors.count(), 2);
    EXPECT_TRUE(settings->languageError().isEmpty());
    EXPECT_EQ(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("就绪"));

    EXPECT_FALSE(settings->setLanguage(QStringLiteral("zh_CN")));
    EXPECT_EQ(settings->language(), QStringLiteral("zh-CN"));
    EXPECT_EQ(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("就绪"));

    const QString invalidEncoding(QChar(0xD800));
    EXPECT_FALSE(settings->setLanguage(invalidEncoding));
    EXPECT_EQ(settings->languageError(), QStringLiteral("language.invalid_locale_encoding"));
    EXPECT_EQ(settings->language(), QStringLiteral("zh-CN"));
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("就绪"));

    ASSERT_TRUE(settings->setLanguage(QStringLiteral("en")));
    EXPECT_EQ(settings->language(), QStringLiteral("en"));
    EXPECT_EQ(changed.count(), 2);
    EXPECT_EQ(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
    EXPECT_EQ(textObject->property("label").toString(), QStringLiteral("Ready"));
}

} // namespace

int main(int argc, char** argv) {
    QCoreApplication app(argc, argv);
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
