/// 验证 `.pa` → TS → lrelease QM → qrc 链路产出可加载的翻译数据。
/// 只断言已知字典条目（context + source 精确查找），不做语言切换行为测试（022）。
#include <QFile>
#include <QObject>
#include <QString>
#include <QTranslator>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>

class QmLoadTest : public QObject {
    Q_OBJECT
  private slots:
    /// zh-CN 字典经资源路径加载后按 context+source 返回译文，覆盖多个上下文。
    void zhCnDictionaryTranslates();
    /// en 字典同样可加载；译文与源文一致（基线语言）。
    void enBaselineDictionaryLoads();
};

void QmLoadTest::zhCnDictionaryTranslates() {
    QVERIFY2(QFile::exists(QStringLiteral(":/i18n/panta_zh_CN.qm")),
             "qrc 资源未注册：检查 panta_i18n 对象库是否被链接");
    QTranslator translator;
    QVERIFY(translator.load(QStringLiteral(":/i18n/panta_zh_CN.qm")));
    QCOMPARE(translator.translate("App", "Ready"), QStringLiteral("就绪"));
    QCOMPARE(translator.translate("TopChromePanel", "Sign in"), QStringLiteral("登录"));
    QCOMPARE(translator.translate("RibbonPanel", "Start"), QStringLiteral("启动"));
    QCOMPARE(translator.translate("ViewportPane", "Mesh"), QStringLiteral("网格"));
}

void QmLoadTest::enBaselineDictionaryLoads() {
    QVERIFY2(QFile::exists(QStringLiteral(":/i18n/panta_en.qm")),
             "qrc 资源未注册：检查 panta_i18n 对象库是否被链接");
    QTranslator translator;
    QVERIFY(translator.load(QStringLiteral(":/i18n/panta_en.qm")));
    QCOMPARE(translator.translate("App", "Ready"), QStringLiteral("Ready"));
    QCOMPARE(translator.translate("TasksPanel", "New Project"), QStringLiteral("New Project"));
}

QTEST_GUILESS_MAIN(QmLoadTest)
#include "qm_load_test.moc"
