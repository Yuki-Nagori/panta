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
    QCOMPARE(translator.translate("RibbonGroupStart", "Start"), QStringLiteral("启动"));
    QCOMPARE(translator.translate("RibbonActionNewProject", "New\nProject"),
             QStringLiteral("新建\n工程"));
    QCOMPARE(translator.translate("UiCommonNavigation", "Home"), QStringLiteral("主页"));
    QCOMPARE(translator.translate("ProjectTaskItem", "Project '%1'").arg(QStringLiteral("01")),
             QStringLiteral("工程“01”"));
    QCOMPARE(translator.translate("UiCommonModeling", "Mesh"), QStringLiteral("网格"));
    QCOMPARE(translator.translate("NewProjectDialog", "Create New Project"),
             QStringLiteral("新建工程"));
    QCOMPARE(translator.translate("UiCommonNavigation", "Browse"), QStringLiteral("浏览"));
    QCOMPARE(translator.translate("IconActionOpenProject", "Open Project"),
             QStringLiteral("打开工程"));
    QCOMPARE(translator.translate("UiCommon", "Import"), QStringLiteral("导入"));
    QCOMPARE(translator.translate("UiCommon", "File"), QStringLiteral("文件"));
    QCOMPARE(translator.translate("UiCommonNavigation", "Tools"), QStringLiteral("工具"));
    QCOMPARE(translator.translate("UiCommonResults", "Results"), QStringLiteral("结果"));
    QCOMPARE(translator.translate("ImportDialogForm", "Mesh type"), QStringLiteral("网格类型"));
    QCOMPARE(translator.translate("ImportMeshDualDomain", "Dual Domain"), QStringLiteral("双层面"));
    QCOMPARE(translator.translate("UiCommonUnitMillimeters", "Millimeters"),
             QStringLiteral("毫米"));
    QCOMPARE(translator.translate("ImportTask", "Create Mesh..."), QStringLiteral("创建网格..."));
    QCOMPARE(translator.translate("ProcessTask", "Process Settings (Default)"),
             QStringLiteral("工艺设置（默认）"));
    QCOMPARE(translator.translate("DialogAction", "Cancel"), QStringLiteral("取消"));
}

void QmLoadTest::enBaselineDictionaryLoads() {
    QVERIFY2(QFile::exists(QStringLiteral(":/i18n/panta_en.qm")),
             "qrc 资源未注册：检查 panta_i18n 对象库是否被链接");
    QTranslator translator;
    QVERIFY(translator.load(QStringLiteral(":/i18n/panta_en.qm")));
    QCOMPARE(translator.translate("App", "Ready"), QStringLiteral("Ready"));
    QCOMPARE(translator.translate("UiCommonNavigation", "New Project"),
             QStringLiteral("New Project"));
    QCOMPARE(translator.translate("NewProjectDialog", "Create New Project"),
             QStringLiteral("Create New Project"));
    QCOMPARE(translator.translate("UiCommon", "Import"), QStringLiteral("Import"));
    QCOMPARE(translator.translate("UiCommon", "File"), QStringLiteral("File"));
    QCOMPARE(translator.translate("UiCommonModeling", "Mesh"), QStringLiteral("Mesh"));
    QCOMPARE(translator.translate("ImportTask", "Create Mesh..."),
             QStringLiteral("Create Mesh..."));
}

QTEST_GUILESS_MAIN(QmLoadTest)
#include "qm_load_test.moc"
