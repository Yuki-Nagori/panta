/// 验证 Settings 跨 Rust locale、Qt translator 与 QML 翻译绑定的行为。
#include "project_view_model.hpp"
#include "settings.hpp"
#include "shell_view_model.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QJSEngine>
#include <QObject>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQmlEngine>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariant>
#include <QtQml/qqml.h>
#include <QtTest/qtest.h>
#include <memory>
#include <qtestcase.h>
#include <qtmetamacros.h>

using panta::bridge::Settings;

class SettingsTest final : public QObject {
    Q_OBJECT

  private slots:
    void language_switch_retranslates_qml_and_invalid_input_preserves_state() {
        const int settingsTypeId = qmlRegisterSingletonType<Settings>(
            "Panta.Test", 1, 0, "Settings",
            [](QQmlEngine*, QJSEngine*) -> QObject* { return new Settings; });
        QQmlEngine engine;
        auto* settings = engine.singletonInstance<Settings*>(settingsTypeId);
        QVERIFY(settings != nullptr);

        panta::bridge::ProjectViewModel projectModel;
        panta::bridge::ShellViewModel shellModel;
        shellModel.tick();
        engine.rootContext()->setContextProperty("projectModel", &projectModel);
        engine.rootContext()->setContextProperty("shellModel", &shellModel);
        QQmlComponent errorComponent(&engine);
        errorComponent.setData("import QtQml\nimport Panta.Test 1.0\nQtObject {"
                               " property string errorText: projectModel.error;"
                               " property Connections languageConnection: Connections {"
                               " target: Settings; function onLanguageChanged() {"
                               " projectModel.retranslateError();"
                               " shellModel.retranslateCaption() } } }",
                               QUrl());
        std::unique_ptr<QObject> errorObject(errorComponent.create());
        QVERIFY2(errorObject != nullptr, qPrintable(errorComponent.errorString()));
        QVERIFY(!projectModel.createProject(QStringLiteral("../invalid"), QDir::tempPath()));
        QCOMPARE(errorObject->property("errorText").toString(),
                 QStringLiteral("Enter a valid project name."));

        QQmlComponent textComponent(&engine);
        textComponent.setData(
            "import QtQml\nQtObject { property string label: qsTranslate(\"App\", \"Ready\") }",
            QUrl());
        std::unique_ptr<QObject> textObject(textComponent.create());
        QVERIFY2(textObject != nullptr, qPrintable(textComponent.errorString()));

        QCOMPARE(settings->language(), QStringLiteral("en"));
        QVERIFY(settings->availableLanguages().contains(QStringLiteral("zh-CN")));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("Ready"));
        QSignalSpy changed(settings, &Settings::languageChanged);
        QSignalSpy errors(settings, &Settings::languageErrorChanged);
        QVERIFY(changed.isValid());
        QVERIFY(errors.isValid());

        QVERIFY(!settings->setLanguage(QStringLiteral("fr")));
        QVERIFY(
            settings->languageError().startsWith(QStringLiteral("language.unsupported_locale")));
        QCOMPARE(settings->language(), QStringLiteral("en"));
        QCOMPARE(changed.count(), 0);
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("Ready"));

        QVERIFY2(settings->setLanguage(QStringLiteral("zh-CN")),
                 qPrintable(settings->languageError()));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(changed.count(), 1);
        QCOMPARE(errors.count(), 2);
        QVERIFY(settings->languageError().isEmpty());
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));
        QCOMPARE(errorObject->property("errorText").toString(),
                 QStringLiteral("请输入有效的工程名称。"));
        QCOMPARE(shellModel.caption(), QStringLiteral("修订 1"));

        QVERIFY(!settings->setLanguage(QStringLiteral("zh_CN")));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));

        const QString invalidEncoding(QChar(0xD800));
        QVERIFY(!settings->setLanguage(invalidEncoding));
        QCOMPARE(settings->languageError(), QStringLiteral("language.invalid_locale_encoding"));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));

        QVERIFY(settings->setLanguage(QStringLiteral("en")));
        QCOMPARE(settings->language(), QStringLiteral("en"));
        QCOMPARE(changed.count(), 2);
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("Ready"));
        QCOMPARE(errorObject->property("errorText").toString(),
                 QStringLiteral("Enter a valid project name."));
        QCOMPARE(shellModel.caption(), QStringLiteral("Revision 1"));
    }
};

QTEST_GUILESS_MAIN(SettingsTest)
#include "settings_test.moc"
