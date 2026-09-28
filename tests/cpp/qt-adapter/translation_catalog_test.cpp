/// 翻译目录安装失败时，已安装的 Qt translator 必须继续生效。
#include <QCoreApplication>
#include <QObject>
#include <QString>
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

    QVERIFY(!catalog.install(QStringLiteral(":/i18n/missing.qm"), &error));
    QCOMPARE(error, QStringLiteral("i18n.catalog_load_failed"));
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("就绪"));

    catalog.clear();
    QCOMPARE(QCoreApplication::translate("App", "Ready"), QStringLiteral("Ready"));
}

QTEST_GUILESS_MAIN(TranslationCatalogTest)
#include "translation_catalog_test.moc"
