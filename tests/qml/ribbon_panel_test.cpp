// Ribbon 与侧栏组件的布局、路由和状态呈现回归。
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

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)
#ifdef PANTA_TEST_WITH_BRIDGE
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

#include "qml_component_test_helpers.hpp"
using namespace panta::test::qml;

class RibbonPanelTest final : public QObject {
    Q_OBJECT

  private slots:
    void ribbon_content_expands_without_losing_theme_defaults() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        QObject owner;
        auto* tile = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Composites/RibbonTile.qml"),
            owner);
        QVERIFY(tile != nullptr);
        qobject_cast<QQuickItem*>(tile)->setParentItem(window.contentItem());
        window.show();
        tile->setProperty("text", QStringLiteral("New\nProject"));
        QCOMPARE(tile->property("implicitWidth").toDouble(), 56.0);
        QCOMPARE(tile->property("implicitHeight").toDouble(), 68.0);
        QCOMPARE(tile->property("iconSize").toInt(), 26);
        tile->setProperty("text", QStringLiteral("Thermoplastics\nInjection Molding"));
        QTRY_VERIFY(tile->property("implicitWidth").toDouble() > 56.0);
        QCOMPARE(tile->property("implicitHeight").toDouble(), 68.0);
        tile->setProperty("showCaret", true);
        QCOMPARE(tile->property("implicitHeight").toDouble(), 68.0);
        tile->setProperty("iconSize", 32);
        QTRY_VERIFY(tile->property("implicitHeight").toDouble() > 68.0);
    }
    void ribbon_tabs_load_independently_data() {
        QTest::addColumn<QString>("componentName");
        QTest::addColumn<int>("groupCount");
        QTest::addColumn<int>("toolCount");
        QTest::addColumn<QString>("firstKey");
        QTest::newRow("home") << QStringLiteral("HomeRibbon") << 6 << 18
                              << QStringLiteral("import");
        QTest::newRow("start-learn")
            << QStringLiteral("StartLearnRibbon") << 3 << 7 << QStringLiteral("new-project");
    }
    void ribbon_tabs_load_independently() {
        QFETCH(QString, componentName);
        QFETCH(int, groupCount);
        QFETCH(int, toolCount);
        QFETCH(QString, firstKey);
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        QObject owner;
        auto* tab = qobject_cast<QQuickItem*>(
            create_component(engine,
                             QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/Ribbon/") +
                                 componentName + QStringLiteral(".qml"),
                             owner));
        QVERIFY(tab != nullptr);
        tab->setParentItem(window.contentItem());
        window.show();
        QTRY_COMPARE(visual_items(tab, QStringLiteral("ribbonTools")).size(), groupCount);
        QTRY_COMPARE(visual_items(tab, QStringLiteral("ribbonTileContent")).size(), toolCount);
        QTRY_VERIFY(tab->implicitWidth() > 0);
        QTRY_COMPARE(tab->implicitHeight(), 95.0);
        QCOMPARE(tab->width(), tab->implicitWidth());

        // 页签单独加载也能通过公共渲染报告 key，不依赖 App 或外壳 id。
        QSignalSpy actionRequested(tab, SIGNAL(actionRequested(QString)));
        QVERIFY(actionRequested.isValid());
        auto* button = visual_item(tab, QStringLiteral("ribbon-") + firstKey);
        QVERIFY(button != nullptr);
        QVERIFY(QMetaObject::invokeMethod(button, "clicked"));
        QCOMPARE(actionRequested.count(), 1);
        QCOMPARE(actionRequested.constFirst().constFirst().toString(), firstKey);
    }
    void ribbon_routes_only_existing_project_commands() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        QObject owner;
        auto* ribbon = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/RibbonPanel.qml"), owner);
        QVERIFY(ribbon != nullptr);
        qobject_cast<QQuickItem*>(ribbon)->setParentItem(window.contentItem());
        window.show();
        QSignalSpy newRequested(ribbon, SIGNAL(newProjectRequested()));
        QSignalSpy openRequested(ribbon, SIGNAL(openProjectRequested()));
        QVERIFY(newRequested.isValid() && openRequested.isValid());
        auto* ribbonItem = qobject_cast<QQuickItem*>(ribbon);
        auto* loader = visual_item(ribbonItem, QStringLiteral("ribbonLoader"));
        QVERIFY(loader != nullptr);
        QTRY_VERIFY(loader->width() > 0);
        QPointer<QQuickItem> previousTab = loader->property("item").value<QQuickItem*>();
        QVERIFY(previousTab);
        QVERIFY(loader->width() > 0);
        QCOMPARE(loader->width(), previousTab->width());
        ribbon->setProperty("activeRibbonTab", QStringLiteral("start-learn"));
        QCOMPARE(loader->property("item").value<QQuickItem*>(), previousTab.data());
        auto* newButton = visual_item(ribbonItem, QStringLiteral("ribbon-new-project"));
        auto* openButton = visual_item(ribbonItem, QStringLiteral("ribbon-open-project"));
        QVERIFY(newButton && openButton);
        QVERIFY(QMetaObject::invokeMethod(newButton, "clicked"));
        QVERIFY(QMetaObject::invokeMethod(openButton, "clicked"));
        QCOMPARE(newRequested.count(), 1);
        QCOMPARE(openRequested.count(), 1);
        newButton->forceActiveFocus(Qt::TabFocusReason);
        QTRY_COMPARE(window.activeFocusItem(), newButton);
        ribbon->setProperty("activeRibbonTab", QStringLiteral("home"));
        QTRY_VERIFY(previousTab.isNull());
        QVERIFY(visual_item(ribbonItem, QStringLiteral("ribbon-new-project")) == nullptr);
        previousTab = loader->property("item").value<QQuickItem*>();
        QVERIFY(previousTab);
        QTRY_COMPARE(loader->width(), previousTab->width());
        auto* importButton = visual_item(ribbonItem, QStringLiteral("ribbon-import"));
        QVERIFY(importButton != nullptr);
        QVERIFY(QMetaObject::invokeMethod(importButton, "clicked"));
        QCOMPARE(newRequested.count(), 1);
        QCOMPARE(openRequested.count(), 1);
        ribbon->setProperty("activeRibbonTab", QStringLiteral("start-learn"));
        QTRY_VERIFY(previousTab.isNull());
        QVERIFY(visual_item(ribbonItem, QStringLiteral("ribbon-import")) == nullptr);
        newButton = visual_item(ribbonItem, QStringLiteral("ribbon-new-project"));
        openButton = visual_item(ribbonItem, QStringLiteral("ribbon-open-project"));
        QVERIFY(newButton && openButton);
        QVERIFY(QMetaObject::invokeMethod(newButton, "clicked"));
        QVERIFY(QMetaObject::invokeMethod(openButton, "clicked"));
        QCOMPARE(newRequested.count(), 2);
        QCOMPARE(openRequested.count(), 2);
    }
    void layers_tab_row_tracks_imported_parts() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQuickWindow window;
        QObject owner;
        auto* panel = qobject_cast<QQuickItem*>(create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/LayersPanel.qml"), owner));
        QVERIFY(panel != nullptr);
        panel->setWidth(420);
        panel->setHeight(260);
        panel->setParentItem(window.contentItem());
        window.resize(640, 480);
        window.show();

        auto* toolbar = visual_item(panel, QStringLiteral("layersToolbar"));
        auto* tabRow = visual_item(panel, QStringLiteral("layersTabRow"));
        auto* content = visual_item(panel, QStringLiteral("layersContent"));
        QVERIFY(toolbar && tabRow && content);
        QTRY_VERIFY(panel->isVisible());
        QTRY_VERIFY(toolbar->property("visible").toBool());
        QVERIFY(!tabRow->property("visible").toBool());
        QVERIFY(content->property("visible").toBool());

        panel->setProperty("importedPartNames", QStringList{QStringLiteral("sample.stl")});
        QTRY_VERIFY(tabRow->property("visible").toBool());
        QVERIFY(panel->isVisible());

        panel->setProperty("importedPartNames", QStringList{});
        QTRY_VERIFY(!tabRow->property("visible").toBool());
        QVERIFY(toolbar->property("visible").toBool());
        QVERIFY(content->property("visible").toBool());
    }
    void tasks_inspector_title_uses_active_import_only() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* panel = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/SidebarPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        panel->setProperty("importedPartIds", QStringList{QStringLiteral("import-1")});
        panel->setProperty("importedPartNames", QStringList{QStringLiteral("latest")});
        panel->setProperty("activeDocumentId", QStringLiteral("welcome"));
        panel->setProperty("activeDocumentTitle", QStringLiteral("Welcome"));
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("latest"));

        panel->setProperty("activeDocumentId", QStringLiteral("import-1"));
        panel->setProperty("activeDocumentTitle", QStringLiteral("active"));
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("latest"));

        panel->setProperty("importedPartIds",
                           QStringList{QStringLiteral("import-1"), QStringLiteral("import-2")});
        panel->setProperty("importedPartNames",
                           QStringList{QStringLiteral("part"), QStringLiteral("part")});
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("1 · part"));
        panel->setProperty("activeDocumentId", QStringLiteral("import-2"));
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("2 · part"));
    }
};

QTEST_MAIN(RibbonPanelTest)
#include "ribbon_panel_test.moc"
