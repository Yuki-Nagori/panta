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
    QCOMPARE(
        translator.translate("ProjectTaskItem", "Project '%1'").arg(QStringLiteral("mold-a 2027")),
        QStringLiteral("工程“mold-a 2027”"));
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
    QCOMPARE(translator.translate("MeshType", "Dual Domain"), QStringLiteral("双层面"));
    QCOMPARE(translator.translate("UiCommonUnitMillimeters", "Millimeters"),
             QStringLiteral("毫米"));
    QCOMPARE(translator.translate("ImportTask", "Create Mesh..."), QStringLiteral("创建网格..."));
    QCOMPARE(translator.translate("ProcessTask", "Process Settings (Default)"),
             QStringLiteral("工艺设置（默认）"));
    QCOMPARE(translator.translate("DialogAction", "Cancel"), QStringLiteral("取消"));
    QCOMPARE(translator.translate("DocumentTabBar", "Open documents"),
             QStringLiteral("打开的文档"));
    QCOMPARE(translator.translate("DocumentTabBar", "Close %1").arg(QStringLiteral("模型")),
             QStringLiteral("关闭模型"));
    QCOMPARE(translator.translate("DocumentTabBar", "%1, import %2")
                 .arg(QStringLiteral("模型"), QStringLiteral("2")),
             QStringLiteral("模型（第 2 个导入）"));
    QCOMPARE(translator.translate("DocumentTabBar", "%1, import %2")
                 .arg(QStringLiteral("part-a.stl"), QStringLiteral("3")),
             QStringLiteral("part-a.stl（第 3 个导入）"));
    QCOMPARE(translator.translate("FillSettings", "%1 steps · %2 s")
                 .arg(QStringLiteral("2"), QStringLiteral("4.5")),
             QStringLiteral("2 个步骤 · 4.5 秒"));
    QCOMPARE(translator.translate("DocumentTabBar", "Loading"), QStringLiteral("正在加载"));
    QCOMPARE(translator.translate("DocumentTabBar", "Failed to load"), QStringLiteral("加载失败"));
    QCOMPARE(translator.translate("DocumentTabBar",
                                  "The saved mesh asset is missing from the project package."),
             QStringLiteral("工程包中缺少已保存的网格资产。"));
    QCOMPARE(translator.translate("ProjectViewModel", "Enter a valid project name."),
             QStringLiteral("请输入有效的工程名称。"));
    QCOMPARE(translator.translate("ViewRibbonAccessibility", "Model Display"),
             QStringLiteral("模型显示"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "About"), QStringLiteral("关于"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Setting"), QStringLiteral("设置"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Quit"), QStringLiteral("退出"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Cut"), QStringLiteral("剪切"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Copy"), QStringLiteral("复制"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Paste"), QStringLiteral("粘贴"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Select All"), QStringLiteral("全选"));
    QCOMPARE(translator.translate("ApplicationMenu", "About panta"), QStringLiteral("关于 panta"));
    QCOMPARE(translator.translate("ApplicationMenu", "Settings…"), QStringLiteral("设置…"));
    QCOMPARE(translator.translate("ApplicationMenu", "Quit panta"), QStringLiteral("退出 panta"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Cut"),
             translator.translate("ApplicationMenu", "Cut"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Copy"),
             translator.translate("ApplicationMenu", "Copy"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Paste"),
             translator.translate("ApplicationMenu", "Paste"));
    QCOMPARE(translator.translate("QCocoaMenuItem", "Select All"),
             translator.translate("ApplicationMenu", "Select All"));
    QCOMPARE(translator.translate("MAC_APPLICATION_MENU", "About %1").arg(QStringLiteral("Panta")),
             QStringLiteral("关于 Panta"));
    QCOMPARE(translator.translate("MAC_APPLICATION_MENU", "Preferences..."),
             QStringLiteral("设置…"));
    QCOMPARE(translator.translate("MAC_APPLICATION_MENU", "Quit %1").arg(QStringLiteral("Panta")),
             QStringLiteral("退出 Panta"));
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
    QCOMPARE(translator.translate("DocumentTabBar", "Open documents"),
             QStringLiteral("Open documents"));
    QCOMPARE(translator.translate("DocumentTabBar", "Close %1"), QStringLiteral("Close %1"));
    QCOMPARE(translator.translate("DocumentTabBar", "%1, import %2"),
             QStringLiteral("%1, import %2"));
    QCOMPARE(
        translator.translate("ProjectTaskItem", "Project '%1'").arg(QStringLiteral("mold-a 2027")),
        QStringLiteral("Project 'mold-a 2027'"));
    QCOMPARE(translator.translate("DocumentTabBar", "%1, import %2")
                 .arg(QStringLiteral("part-a.stl"), QStringLiteral("3")),
             QStringLiteral("part-a.stl, import 3"));
    QCOMPARE(translator.translate("FillSettings", "%1 steps · %2 s")
                 .arg(QStringLiteral("2"), QStringLiteral("4.5")),
             QStringLiteral("2 steps · 4.5 s"));
    QCOMPARE(translator.translate("DocumentTabBar", "Loading"), QStringLiteral("Loading"));
    QCOMPARE(translator.translate("DocumentTabBar", "Failed to load"),
             QStringLiteral("Failed to load"));
    QCOMPARE(translator.translate("DocumentTabBar",
                                  "The saved mesh asset is missing from the project package."),
             QStringLiteral("The saved mesh asset is missing from the project package."));
    QCOMPARE(translator.translate("ProjectViewModel", "Enter a valid project name."),
             QStringLiteral("Enter a valid project name."));
    QCOMPARE(translator.translate("ViewRibbonAccessibility", "Model Display"),
             QStringLiteral("Model Display"));
}

QTEST_GUILESS_MAIN(QmLoadTest)
#include "qm_load_test.moc"
