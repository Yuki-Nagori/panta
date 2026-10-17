/// 翻译目录缺项或加载失败时，已安装的 Qt translator 必须继续生效。
#include <QCoreApplication>
#include <QFile>
#include <QObject>
#include <QString>
#include <QTemporaryDir>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <panta/qt_adapter/translation_catalog.hpp>

class TranslationCatalogTest : public QObject {
    Q_OBJECT

  private slots:
    void failedReplacementPreservesInstalledCatalog();
};

void TranslationCatalogTest::failedReplacementPreservesInstalledCatalog() {
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
    QString error = QStringLiteral("stale error");
    panta::qt_adapter::TranslationCatalog catalog;
    QVERIFY(catalog.install(QStringLiteral(":/i18n/panta_en.qm"), &error));
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
    QVERIFY(catalog.install(QStringLiteral(":/i18n/panta_zh_CN.qm"), &error));
    QVERIFY(error.isEmpty());
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));
    QCOMPARE(QCoreApplication::translate("App", "Uncatalogued source"),
             QStringLiteral("Uncatalogued source"));

    QVERIFY(!catalog.install(QStringLiteral(":/i18n/missing.qm"), &error));
    QCOMPARE(error, QStringLiteral("i18n.catalog_load_failed"));
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));

    QTemporaryDir temporaryDirectory;
    QVERIFY(temporaryDirectory.isValid());
    QFile damagedCatalog(temporaryDirectory.filePath(QStringLiteral("damaged.qm")));
    QVERIFY(damagedCatalog.open(QIODevice::WriteOnly));
    QCOMPARE(damagedCatalog.write("not a Qt translation catalog"),
             qint64(sizeof("not a Qt translation catalog") - 1));
    damagedCatalog.close();
    QVERIFY(!catalog.install(damagedCatalog.fileName(), &error));
    QCOMPARE(error, QStringLiteral("i18n.catalog_load_failed"));
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));

    catalog.clear();
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
}

QTEST_GUILESS_MAIN(TranslationCatalogTest)
#include "translation_catalog_test.moc"
