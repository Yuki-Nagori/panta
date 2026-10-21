/// 覆盖 QSettings 的单值快照读写、字段类型和未知键拒绝。
#include <QCoreApplication>
#include <QList>
#include <QObject>
#include <QProcess>
#include <QSettings>
#include <QTemporaryDir>
#include <QUuid>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <panta/qt_adapter/user_settings_store.hpp>
#include <qcontainerfwd.h>
#include <qtmetamacros.h>
#include <utility>

using panta::qt_adapter::UserSettingsRecord;
using panta::qt_adapter::UserSettingsStore;

namespace {

QList<QStringList> recentProjectPathVariants() {
    return {{},
            {QStringLiteral("/tmp/one,with-comma.panta")},
            {QStringLiteral("/tmp/one.panta"), QStringLiteral("/tmp/two.panta")}};
}

UserSettingsRecord expectedRecord(qsizetype index) {
    return {1,
            QStringLiteral("dark"),
            QStringLiteral("zh-CN"),
            1440,
            900,
            false,
            recentProjectPathVariants().at(index)};
}

class NativeSettingsCleanup final {
  public:
    NativeSettingsCleanup(QString organization, QString application)
        : m_organization(std::move(organization)), m_application(std::move(application)) {}

    ~NativeSettingsCleanup() {
        QSettings settings(QSettings::NativeFormat, QSettings::UserScope, m_organization,
                           m_application);
        settings.clear();
        settings.sync();
    }

  private:
    QString m_organization;
    QString m_application;
};

} // namespace

class UserSettingsStoreTest final : public QObject {
    Q_OBJECT

  private slots:
    void missing_settings_return_defaults_without_error() {
        QTemporaryDir temporaryDirectory;
        QVERIFY(temporaryDirectory.isValid());
        UserSettingsStore store(temporaryDirectory.filePath(QStringLiteral("settings.ini")),
                                QSettings::IniFormat);
        UserSettingsRecord record;
        bool found = true;
        QString error;

        QVERIFY(store.load(&record, &found, &error));
        QVERIFY(!found);
        QVERIFY(error.isEmpty());
    }

    void full_snapshot_round_trips_across_processes() {
        QTemporaryDir temporaryDirectory;
        QVERIFY(temporaryDirectory.isValid());
        const QString fileName = temporaryDirectory.filePath(QStringLiteral("settings.ini"));
        const auto recentLists = recentProjectPathVariants();
        for (qsizetype index = 0; index < recentLists.size(); ++index) {
            const UserSettingsRecord expected = expectedRecord(index);
            UserSettingsStore store(fileName, QSettings::IniFormat);
            QString error;
            QVERIFY2(store.save(expected, &error), qPrintable(error));

            QProcess reader;
            reader.start(
                QCoreApplication::applicationFilePath(),
                {QStringLiteral("--verify-stored-settings"), fileName, QString::number(index)});
            QVERIFY2(reader.waitForStarted(), qPrintable(reader.errorString()));
            QVERIFY2(reader.waitForFinished(), qPrintable(reader.errorString()));
            QCOMPARE(reader.exitStatus(), QProcess::NormalExit);
            QCOMPARE(reader.exitCode(), 0);
        }
    }

    void native_format_snapshot_round_trips_across_processes_in_isolated_settings() {
        const QString organization = QStringLiteral("PantaTests");
        const QString application =
            QStringLiteral("Settings-") + QUuid::createUuid().toString(QUuid::WithoutBraces);
        const NativeSettingsCleanup cleanup(organization, application);
        const auto recentLists = recentProjectPathVariants();
        for (qsizetype index = 0; index < recentLists.size(); ++index) {
            const UserSettingsRecord expected = expectedRecord(index);
            UserSettingsStore store(organization, application);
            QString error;
            QVERIFY2(store.save(expected, &error), qPrintable(error));

            QProcess reader;
            reader.start(QCoreApplication::applicationFilePath(),
                         {QStringLiteral("--verify-native-settings"), organization, application,
                          QString::number(index)});
            QVERIFY2(reader.waitForStarted(), qPrintable(reader.errorString()));
            QVERIFY2(reader.waitForFinished(), qPrintable(reader.errorString()));
            QCOMPARE(reader.exitStatus(), QProcess::NormalExit);
            QCOMPARE(reader.exitCode(), 0);
        }
    }

    void unknown_keys_and_wrong_types_are_rejected_without_partial_results() {
        QTemporaryDir temporaryDirectory;
        QVERIFY(temporaryDirectory.isValid());
        const QString fileName = temporaryDirectory.filePath(QStringLiteral("settings.ini"));
        UserSettingsStore store(fileName, QSettings::IniFormat);
        QString error;
        QVERIFY(store.save(
            UserSettingsRecord{
                1, QStringLiteral("light"), QStringLiteral("en"), 1280, 720, true, {}},
            &error));

        {
            QSettings settings(fileName, QSettings::IniFormat);
            settings.beginGroup(QStringLiteral("preferences"));
            settings.setValue(QStringLiteral("futureKey"), true);
            settings.endGroup();
            settings.sync();
        }
        UserSettingsRecord record;
        bool found = false;
        QVERIFY(!store.load(&record, &found, &error));
        QVERIFY(found);
        QCOMPARE(error, QStringLiteral("settings.unknown_key: futureKey"));

        {
            QSettings settings(fileName, QSettings::IniFormat);
            settings.beginGroup(QStringLiteral("preferences"));
            settings.remove(QStringLiteral("futureKey"));
            settings.setValue(QStringLiteral("snapshot"),
                              QStringLiteral("{\"schemaVersion\":4294967297,\"themeId\":\"light\","
                                             "\"language\":\"en\",\"windowWidth\":1280,"
                                             "\"windowHeight\":720,\"windowMaximized\":true,"
                                             "\"recentProjectPaths\":[]}"));
            settings.endGroup();
            settings.sync();
        }
        QVERIFY(!store.load(&record, &found, &error));
        QCOMPARE(error, QStringLiteral("settings.invalid_stored_type"));

        {
            QSettings settings(fileName, QSettings::IniFormat);
            settings.beginGroup(QStringLiteral("preferences"));
            settings.setValue(QStringLiteral("snapshot"),
                              QStringLiteral("{\"schemaVersion\":1,\"themeId\":\"light\","
                                             "\"language\":\"en\",\"windowWidth\":1280,"
                                             "\"windowHeight\":720,\"windowMaximized\":\"yes\","
                                             "\"recentProjectPaths\":[]}"));
            settings.endGroup();
            settings.sync();
        }
        QVERIFY(!store.load(&record, &found, &error));
        QCOMPARE(error, QStringLiteral("settings.invalid_stored_type"));
    }
};

int main(int argc, char* argv[]) {
    QCoreApplication application(argc, argv);
    const QStringList arguments = application.arguments();
    if (arguments.size() == 5 && arguments[1] == QStringLiteral("--verify-native-settings")) {
        const int index = arguments[4].toInt();
        const auto recentLists = recentProjectPathVariants();
        if (index < 0 || index >= recentLists.size()) {
            return 2;
        }
        UserSettingsStore store(arguments[2], arguments[3]);
        UserSettingsRecord actual;
        bool found = false;
        QString error;
        return store.load(&actual, &found, &error) && found && actual == expectedRecord(index) ? 0
                                                                                               : 1;
    }
    if (arguments.size() == 4 && arguments[1] == QStringLiteral("--verify-stored-settings")) {
        const int index = arguments[3].toInt();
        const auto recentLists = recentProjectPathVariants();
        if (index < 0 || index >= recentLists.size()) {
            return 2;
        }
        UserSettingsStore store(arguments[2], QSettings::IniFormat);
        UserSettingsRecord actual;
        bool found = false;
        QString error;
        return store.load(&actual, &found, &error) && found && actual == expectedRecord(index) ? 0
                                                                                               : 1;
    }

    UserSettingsStoreTest test;
    return QTest::qExec(&test, argc, argv);
}

#include "user_settings_store_test.moc"
