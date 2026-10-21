// 分析日志、结果树与侧栏面板的显示和交互回归。
#include "../support/qml/quick_item_helpers.hpp"
#if defined(PANTA_TEST_WITH_BRIDGE)
#include "project_view_model.hpp"
using panta::bridge::ProjectViewModel;
#endif
#include <QColor>
#include <QCoreApplication>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QFont>
#include <QGuiApplication>
#include <QImage>
#include <QImageReader>
#include <QJSValue>
#include <QMetaType>
#include <QObject>
#include <QPointer>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QQuickItem>
#include <QQuickItemGrabResult>
#include <QQuickWindow>
#include <QRect>
#include <QRegularExpression>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QTemporaryDir>
#include <QTranslator>
#include <QUrl>
#include <QVariantMap>
#include <QVector>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>
#include <qaccessible.h>
#include <qtestkeyboard.h>

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)
#ifdef PANTA_TEST_WITH_BRIDGE
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

#include "qml_component_test_helpers.hpp"
using namespace panta::test::qml;

class AnalysisPanelTest final : public QObject {
    Q_OBJECT

  private slots:
    void logs_preserve_history_and_isolate_contexts() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* panel = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/AnalysisLogPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        auto* root = qobject_cast<QQuickItem*>(panel);
        auto* history = visual_item(root, QStringLiteral("analysisLogRunChoice"));
        auto* close = visual_item(root, QStringLiteral("analysisLogClose"));
        QVERIFY(history && close);
        QCOMPARE(panel->property("selectedRunIndex").toInt(), -1);
        QCOMPARE(panel->property("currentText").toString(),
                 QStringLiteral("No analysis logs yet."));
        QVERIFY(!history->property("enabled").toBool());

        const QVariantList runs{
            output_run(QStringLiteral("part-a"), QStringLiteral("run-1"), QStringLiteral("A1")),
            output_run(QStringLiteral("part-a"), QStringLiteral("run-2"), QStringLiteral("A2")),
            output_run(QStringLiteral("part-b"), QStringLiteral("run-2"), QStringLiteral("B2"))};
        panel->setProperty("contextId", QStringLiteral("part-a"));
        panel->setProperty("runs", runs);
        QTRY_COMPARE(panel->property("currentText").toString(), QStringLiteral("A2"));
        QCOMPARE(history->property("count").toInt(), 2);
        QVERIFY(QMetaObject::invokeMethod(history, "activated", Q_ARG(int, 0)));
        QTRY_COMPARE(panel->property("currentText").toString(), QStringLiteral("A1"));

        QSignalSpy closed(panel, SIGNAL(closeRequested()));
        QVERIFY(closed.isValid());
        QVERIFY(QMetaObject::invokeMethod(close, "clicked"));
        QCOMPARE(closed.count(), 1);
        panel->setProperty("visible", false);
        panel->setProperty("visible", true);
        QCOMPARE(panel->property("currentText").toString(), QStringLiteral("A1"));
        panel->setProperty("runs", runs);
        QCOMPARE(panel->property("currentText").toString(), QStringLiteral("A1"));

        panel->setProperty("contextId", QStringLiteral("part-b"));
        QTRY_COMPARE(panel->property("currentText").toString(), QStringLiteral("B2"));
        QCOMPARE(history->property("count").toInt(), 1);
        panel->setProperty("contextId", QStringLiteral("missing"));
        QTRY_COMPARE(panel->property("currentText").toString(),
                     QStringLiteral("No analysis logs yet."));
        QCOMPARE(panel->property("selectedRunIndex").toInt(), -1);
        QVERIFY(!history->property("enabled").toBool());
    }
    void logs_accept_empty_categories_and_change_tabs() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* panel = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/AnalysisLogPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        auto run =
            output_run(QStringLiteral("part"), QStringLiteral("run"), QStringLiteral("analysis"));
        run[QStringLiteral("categories")] = QVariantList{};
        panel->setProperty("contextId", QStringLiteral("part"));
        panel->setProperty("runs", QVariantList{run});
        QTRY_COMPARE(panel->property("currentText").toString(),
                     QStringLiteral("No analysis logs yet."));
        run[QStringLiteral("categories")] =
            QVariantList{QVariantMap{{QStringLiteral("id"), QStringLiteral("mesh")},
                                     {QStringLiteral("sourceText"), QStringLiteral("Mesh Log")},
                                     {QStringLiteral("text"), QStringLiteral("mesh text")}},
                         QVariantMap{{QStringLiteral("id"), QStringLiteral("analysis")},
                                     {QStringLiteral("sourceText"), QStringLiteral("Analysis Log")},
                                     {QStringLiteral("text"), QStringLiteral("analysis text")}}};
        panel->setProperty("runs", QVariantList{run});
        auto* tabs =
            visual_item(qobject_cast<QQuickItem*>(panel), QStringLiteral("analysisLogTabs"));
        QVERIFY(tabs != nullptr);
        QVERIFY(QMetaObject::invokeMethod(tabs, "tabActivated", Q_ARG(int, 0)));
        QTRY_COMPARE(panel->property("currentText").toString(), QStringLiteral("mesh text"));
        QVERIFY(QMetaObject::invokeMethod(tabs, "tabActivated", Q_ARG(int, 1)));
        QTRY_COMPARE(panel->property("currentText").toString(), QStringLiteral("analysis text"));
    }
    void results_are_exclusive_across_groups_and_remember_runs() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        window.resize(400, 300);
        QObject owner;
        auto run =
            output_run(QStringLiteral("part"), QStringLiteral("run-1"), QStringLiteral("log"));
        const QVariantList groups{
            QVariantMap{
                {QStringLiteral("sourceText"), QStringLiteral("Flow")},
                {QStringLiteral("results"),
                 QVariantList{
                     QVariantMap{{QStringLiteral("id"), QStringLiteral("fill")},
                                 {QStringLiteral("sourceText"), QStringLiteral("Fill time")}},
                     QVariantMap{
                         {QStringLiteral("id"), QStringLiteral("temperature")},
                         {QStringLiteral("sourceText"), QStringLiteral("Bulk temperature")}}}}},
            QVariantMap{{QStringLiteral("sourceText"), QString{}},
                        {QStringLiteral("results"),
                         QVariantList{QVariantMap{{QStringLiteral("id"), QStringLiteral("gate")},
                                                  {QStringLiteral("sourceText"),
                                                   QStringLiteral("Gating suitability")}}}}}};
        run[QStringLiteral("resultGroups")] = groups;
        QQmlComponent component(
            &engine,
            QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/AnalysisResultsTree.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));
        auto* tree = component.createWithInitialProperties({{QStringLiteral("run"), run}});
        QVERIFY2(tree != nullptr, qPrintable(component.errorString()));
        tree->setParent(&owner);
        auto* root = qobject_cast<QQuickItem*>(tree);
        root->setParentItem(window.contentItem());
        root->setWidth(400);
        QTRY_COMPARE(tree->property("selectedResultId").toString(), QStringLiteral("fill"));
        auto* fill = visual_item(root, QStringLiteral("analysisResult_fill"));
        auto* temperature = visual_item(root, QStringLiteral("analysisResult_temperature"));
        auto* gate = visual_item(root, QStringLiteral("analysisResult_gate"));
        QVERIFY(fill && temperature && gate);
        QVERIFY(fill->property("checked").toBool());
        QSignalSpy selected(tree, SIGNAL(resultSelected(QString)));
        QVERIFY(selected.isValid());
        QVERIFY(QMetaObject::invokeMethod(gate, "clicked"));
        QTRY_VERIFY(gate->property("checked").toBool());
        QVERIFY(!fill->property("checked").toBool());
        QCOMPARE(selected.constLast().constFirst().toString(), QStringLiteral("gate"));

        window.show();
        temperature->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(&window, Qt::Key_Space);
        QTRY_COMPARE(tree->property("selectedResultId").toString(), QStringLiteral("temperature"));
        QVERIFY(!gate->property("checked").toBool());
        QVERIFY(QMetaObject::invokeMethod(temperature, "clicked"));
        QVERIFY(temperature->property("checked").toBool());

        auto other = run;
        other[QStringLiteral("id")] = QStringLiteral("run-2");
        tree->setProperty("run", other);
        QTRY_COMPARE(tree->property("selectedResultId").toString(), QStringLiteral("fill"));
        tree->setProperty("run", run);
        QTRY_COMPARE(tree->property("selectedResultId").toString(), QStringLiteral("temperature"));
        tree->setProperty("run", QVariant{});
        QTRY_VERIFY(tree->property("selectedResultId").toString().isEmpty());
    }
    void logs_task_visibility_is_separate_from_completion() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* panel = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/SidebarPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        auto* root = qobject_cast<QQuickItem*>(panel);
        panel->setProperty("projectOpen", true);
        panel->setProperty("importedPartNames", QStringList{QStringLiteral("part.stl")});
        auto* action = visual_item(root, QStringLiteral("planTaskAction_logs"));
        auto* mark = visual_item(root, QStringLiteral("logsVisibilityIndicator"));
        QVERIFY(action && mark);
        QSignalSpy requested(panel, SIGNAL(logsRequested()));
        QVERIFY(requested.isValid());
        QVERIFY(QMetaObject::invokeMethod(action, "clicked"));
        QCOMPARE(requested.count(), 1);
        panel->setProperty("logsOpen", true);
        QCOMPARE(mark->property("color").value<QColor>(), QColor(QStringLiteral("#2e7ce0")));
        QVERIFY(QMetaObject::invokeMethod(action, "clicked"));
        QCOMPARE(requested.count(), 2);
        panel->setProperty("logsOpen", false);
        QCOMPARE(mark->property("color").value<QColor>(), QColor(QStringLiteral("#ffffff")));
    }
    void mesh_tool_requires_create_mesh_action() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* panel = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/SidebarPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        auto* tabs = panel->findChild<QObject*>(QStringLiteral("panelTabs"));
        auto* information = panel->findChild<QObject*>(QStringLiteral("toolsInformation"));
        auto* mesh_tool = panel->findChild<QObject*>(QStringLiteral("meshToolPanel"));
        QVERIFY(tabs != nullptr);
        QVERIFY(information != nullptr);
        QVERIFY(mesh_tool != nullptr);

        tabs->setProperty("currentIndex", 1);
        QVERIFY(information->property("visible").toBool());
        QVERIFY(!mesh_tool->property("visible").toBool());

        tabs->setProperty("currentIndex", 0);
        QVERIFY(QMetaObject::invokeMethod(
            panel->findChild<QObject*>(QStringLiteral("projectTasksPage")), "openPlanTask",
            Q_ARG(QVariant, QVariant(QStringLiteral("create-mesh")))));
        QCOMPARE(tabs->property("currentIndex").toInt(), 1);
        QVERIFY(!information->property("visible").toBool());
        QVERIFY(mesh_tool->property("visible").toBool());

        QVERIFY(QMetaObject::invokeMethod(tabs, "tabActivated", Q_ARG(int, 1)));
        QVERIFY(information->property("visible").toBool());
        QVERIFY(!mesh_tool->property("visible").toBool());

        tabs->setProperty("currentIndex", 0);
        tabs->setProperty("currentIndex", 1);
        QVERIFY(information->property("visible").toBool());
        QVERIFY(!mesh_tool->property("visible").toBool());
    }
    void sidebar_tabs_match_empty_and_open_project_states() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        QObject owner;
        auto* panel = qobject_cast<QQuickItem*>(create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/SidebarPanel.qml"), owner));
        QVERIFY(panel != nullptr);
        panel->setWidth(420);
        panel->setHeight(520);
        panel->setParentItem(window.contentItem());
        window.resize(640, 600);
        window.show();
        QTRY_VERIFY(panel->isVisible());

        auto* tabs = panel->findChild<QObject*>(QStringLiteral("panelTabs"));
        auto* tasks = visual_item(panel, QStringLiteral("projectTasksPage"));
        auto* tools = panel->findChild<QObject*>(QStringLiteral("toolsInformation"));
        auto* shared = panel->findChild<QObject*>(QStringLiteral("sharedViewsInformation"));
        QVERIFY(tabs && tasks && tools && shared);
        QCOMPARE(tabs->property("currentIndex").toInt(), 0);
        QVERIFY(tasks->property("visible").toBool());
        QVERIFY(!tools->property("visible").toBool());
        QVERIFY(!shared->property("visible").toBool());

        auto* openProject = visual_item(panel, QStringLiteral("openProjectTaskAction"));
        auto* newProject = visual_item(panel, QStringLiteral("newProjectTaskAction"));
        auto* projectEntry = visual_item(panel, QStringLiteral("projectTaskItem"));
        QVERIFY(openProject && newProject && projectEntry);
        QVERIFY(openProject->property("visible").toBool());
        QVERIFY(newProject->property("visible").toBool());
        QVERIFY(!projectEntry->property("visible").toBool());

        tabs->setProperty("currentIndex", 1);
        QTRY_VERIFY(tools->property("visible").toBool());
        QVERIFY(!tasks->property("visible").toBool());
        QVERIFY(!shared->property("visible").toBool());
        QCOMPARE(tools->property("titleText").toString(), QStringLiteral("Information"));
        const QStringList toolMessages{
            QStringLiteral("Use the tools above to access each tool."),
            QStringLiteral("Open a tool's help to learn how to use it."),
            QStringLiteral("Hold Ctrl while clicking to select multiple entities.")};
        QCOMPARE(tools->property("messages").toStringList(), toolMessages);

        tabs->setProperty("currentIndex", 2);
        QTRY_VERIFY(shared->property("visible").toBool());
        QVERIFY(!tasks->property("visible").toBool());
        QVERIFY(!tools->property("visible").toBool());
        QCOMPARE(shared->property("titleText").toString(), QStringLiteral("Information"));
        QCOMPARE(shared->property("messages").toStringList(),
                 QStringList{
                     QStringLiteral("Access Shared Views from the Shared panel on the Home tab.")});

        tabs->setProperty("currentIndex", 0);
        panel->setProperty("projectOpen", true);
        panel->setProperty("projectName", QStringLiteral("sample"));
        panel->setProperty("importedPartNames", QStringList{QStringLiteral("part.stl")});
        panel->setProperty("importedPartIds", QStringList{QStringLiteral("import-1")});
        QTRY_VERIFY(projectEntry->property("visible").toBool());
        QVERIFY(!openProject->property("visible").toBool());
        QVERIFY(!newProject->property("visible").toBool());

        QVERIFY(tasks->property("visible").toBool());
        QVERIFY(!tools->property("visible").toBool());
        QVERIFY(!shared->property("visible").toBool());
        auto* planTask = visual_item(panel, QStringLiteral("planTaskItem_imported-part"));
        auto* logsTask = visual_item(panel, QStringLiteral("planTaskAction_logs"));
        QVERIFY(planTask && logsTask);
        QVERIFY(planTask->property("visible").toBool());
        QVERIFY(logsTask->property("visible").toBool());
    }
    void mesh_tool_scrolls_in_narrow_dock() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* mesh_tool = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/MeshToolPanel.qml"), owner);
        QVERIFY(mesh_tool != nullptr);
        mesh_tool->setProperty("width", 320);
        mesh_tool->setProperty("height", 240);
        QCoreApplication::processEvents();
        QVERIFY(mesh_tool->property("contentHeight").toReal() > 240);
    }
};

QTEST_MAIN(AnalysisPanelTest)
#include "analysis_panel_test.moc"
