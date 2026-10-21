/// 验证 Settings 跨 Rust locale、Qt translator 与 QML 翻译绑定的行为。
#include "project_view_model.hpp"
#include "settings.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QJSEngine>
#include <QObject>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQmlEngine>
#include <QSettings>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QTemporaryDir>
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
        QTemporaryDir settingsDirectory;
        QVERIFY(settingsDirectory.isValid());
        const QString settingsPath = settingsDirectory.filePath(QStringLiteral("settings.ini"));
        const int settingsTypeId = qmlRegisterSingletonType<Settings>(
            "Panta.Test", 1, 0, "Settings", [settingsPath](QQmlEngine*, QJSEngine*) -> QObject* {
                return new Settings(settingsPath);
            });
        QQmlEngine engine;
        auto* settings = engine.singletonInstance<Settings*>(settingsTypeId);
        QVERIFY(settings != nullptr);

        panta::bridge::ProjectViewModel projectModel;
        engine.rootContext()->setContextProperty("projectModel", &projectModel);
        QQmlComponent errorComponent(&engine);
        errorComponent.setData("import QtQml\nimport Panta.Test 1.0\nQtObject {"
                               " property string errorText: projectModel.error;"
                               " property Connections languageConnection: Connections {"
                               " target: Settings; function onLanguageChanged() {"
                               " projectModel.retranslateError() } } }",
                               QUrl());
        std::unique_ptr<QObject> errorObject(errorComponent.create());
        QVERIFY2(errorObject != nullptr, qPrintable(errorComponent.errorString()));
        QVERIFY(projectModel.createProject(QStringLiteral("../invalid"), QDir::tempPath()));
        QTRY_VERIFY(!projectModel.projectOperationPending());
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
        QSignalSpy errors(settings, &Settings::settingsErrorChanged);
        QVERIFY(changed.isValid());
        QVERIFY(errors.isValid());

        QVERIFY(!settings->setLanguage(QStringLiteral("fr")));
        QVERIFY(settings->settingsError().startsWith(QStringLiteral("settings.invalid_language")));
        QCOMPARE(settings->language(), QStringLiteral("en"));
        QCOMPARE(changed.count(), 0);
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("Ready"));

        QVERIFY2(settings->setLanguage(QStringLiteral("zh-CN")),
                 qPrintable(settings->settingsError()));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(changed.count(), 1);
        QCOMPARE(errors.count(), 2);
        QVERIFY(settings->settingsError().isEmpty());
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));
        QCOMPARE(errorObject->property("errorText").toString(),
                 QStringLiteral("请输入有效的工程名称。"));

        QVERIFY(!settings->setLanguage(QStringLiteral("zh_CN")));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));

        const QString invalidEncoding(QChar(0xD800));
        QVERIFY(!settings->setLanguage(invalidEncoding));
        QCOMPARE(settings->settingsError(), QStringLiteral("settings.invalid_utf8"));
        QCOMPARE(settings->language(), QStringLiteral("zh-CN"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("就绪"));

        QVERIFY(settings->setLanguage(QStringLiteral("en")));
        QCOMPARE(settings->language(), QStringLiteral("en"));
        QCOMPARE(changed.count(), 2);
        QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
        QCOMPARE(textObject->property("label").toString(), QStringLiteral("Ready"));
        QCOMPARE(errorObject->property("errorText").toString(),
                 QStringLiteral("Enter a valid project name."));

        QVERIFY(settings->setThemeId(QStringLiteral("dark")));
        QVERIFY(settings->saveWindowState(QSize(1600, 900), false));
        QVERIFY(
            settings->recordRecentProject(settingsDirectory.filePath(QStringLiteral("one.panta"))));

        Settings restored(settingsPath);
        QCOMPARE(restored.language(), QStringLiteral("en"));
        QCOMPARE(restored.themeId(), QStringLiteral("dark"));
        QCOMPARE(restored.windowWidth(), 1600);
        QCOMPARE(restored.windowHeight(), 900);
        QVERIFY(!restored.windowMaximized());
        QCOMPARE(restored.recentProjectPaths(),
                 QStringList{settingsDirectory.filePath(QStringLiteral("one.panta"))});
        QVERIFY(restored.settingsError().isEmpty());
    }

    void invalid_stored_snapshot_is_preserved_and_disables_writes() {
        QTemporaryDir settingsDirectory;
        QVERIFY(settingsDirectory.isValid());
        const QString settingsPath = settingsDirectory.filePath(QStringLiteral("settings.ini"));
        {
            QSettings stored(settingsPath, QSettings::IniFormat);
            stored.beginGroup(QStringLiteral("preferences"));
            stored.setValue(QStringLiteral("snapshot"),
                            QStringLiteral("{\"schemaVersion\":99,\"themeId\":\"light\","
                                           "\"language\":\"en\",\"windowWidth\":1280,"
                                           "\"windowHeight\":720,\"windowMaximized\":true,"
                                           "\"recentProjectPaths\":[]}"));
            stored.endGroup();
            stored.sync();
            QCOMPARE(stored.status(), QSettings::NoError);
        }

        Settings settings(settingsPath);
        QCOMPARE(settings.themeId(), QStringLiteral("light"));
        QVERIFY(
            settings.settingsError().startsWith(QStringLiteral("settings.unsupported_version")));
        QVERIFY(!settings.setThemeId(QStringLiteral("dark")));

        QSettings stored(settingsPath, QSettings::IniFormat);
        QVERIFY(stored.value(QStringLiteral("preferences/snapshot"))
                    .toString()
                    .contains(QStringLiteral("\"schemaVersion\":99")));
        QVERIFY(
            settings.settingsError().startsWith(QStringLiteral("settings.unsupported_version")));
    }

    void failed_write_keeps_state_and_same_value_theme_rollback_keeps_diagnostic() {
        QTemporaryDir settingsDirectory;
        QVERIFY(settingsDirectory.isValid());
        const QString settingsPath = settingsDirectory.filePath(QStringLiteral("settings.ini"));
        Settings settings(settingsPath);
        QVERIFY(settings.settingsError().isEmpty());
        QVERIFY(QDir().mkdir(settingsPath));

        QVERIFY(!settings.setThemeId(QStringLiteral("dark")));
        QCOMPARE(settings.themeId(), QStringLiteral("light"));
        QVERIFY(
            settings.settingsError().startsWith(QStringLiteral("settings.storage_access_error")));
        QVERIFY(!settings.setThemeId(QStringLiteral("light")));
        QVERIFY(
            settings.settingsError().startsWith(QStringLiteral("settings.storage_access_error")));
    }
};

QTEST_GUILESS_MAIN(SettingsTest)
#include "settings_test.moc"
