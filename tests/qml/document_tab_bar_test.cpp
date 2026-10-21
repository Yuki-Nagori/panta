// 文档页签的可访问性、选择、拖动和滚动回归。
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
#include <qaccessible_base.h>
#include <qpoint.h>
#include <qtestkeyboard.h>
#include <qtestmouse.h>
#include <qtestsupport_gui.h>

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)
#ifdef PANTA_TEST_WITH_BRIDGE
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

#include "qml_component_test_helpers.hpp"
using namespace panta::test::qml;

class DocumentTabBarTest final : public QObject {
    Q_OBJECT

  private slots:
    void document_tab_connectors_follow_html_reference() {
        // 采样 HTML 参考的凹弧和白线，防止连接件缺失或白色外溢。
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QQmlComponent component(
            &engine, QUrl(QStringLiteral(
                         "qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));

        QVariantList documents;
        documents.append(QVariantMap{{QStringLiteral("id"), QStringLiteral("welcome")},
                                     {QStringLiteral("kind"), QStringLiteral("welcome")},
                                     {QStringLiteral("state"), QStringLiteral("ready")},
                                     {QStringLiteral("title"), QStringLiteral("Welcome")},
                                     {QStringLiteral("message"), QString{}}});
        documents.append(QVariantMap{{QStringLiteral("id"), QStringLiteral("import-1")},
                                     {QStringLiteral("kind"), QStringLiteral("import")},
                                     {QStringLiteral("state"), QStringLiteral("ready")},
                                     {QStringLiteral("title"), QStringLiteral("part.stl")},
                                     {QStringLiteral("message"), QString{}}});

        auto* bar = qobject_cast<QQuickItem*>(component.createWithInitialProperties(
            QVariantMap{{QStringLiteral("objectName"), QStringLiteral("documentTabBar")},
                        {QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), QStringLiteral("welcome")}}));
        QVERIFY(bar != nullptr);
        bar->setParent(&owner);
        bar->setWidth(400);
        bar->setHeight(39);

        QQuickWindow window;
        window.resize(400, 39);
        bar->setParentItem(window.contentItem());
        window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&window), "window was not exposed");

        QAccessibleInterface* accessibleList = QAccessible::queryAccessibleInterface(bar);
        QVERIFY(accessibleList != nullptr);
        QCOMPARE(accessibleList->role(), QAccessible::PageTabList);
        QCOMPARE(accessibleList->text(QAccessible::Name), QStringLiteral("Open documents"));
        QTest::mouseMove(&window, QPoint(350, 20));

        const auto result = bar->grabToImage();
        QVERIFY(result != nullptr);
        QSignalSpy ready(result.data(), &QQuickItemGrabResult::ready);
        QVERIFY(ready.wait(5000));
        const QImage image = result->image();
        QCOMPARE(image.size(), QSize(400, 39));

        // 取样避开文字和抗锯齿边界。
        const QColor band_gray(QStringLiteral("#e7e4e1"));
        const QColor panel_white(QStringLiteral("#ffffff"));
        const auto near_white = [](const QColor& color) {
            return color.alpha() == 255 && color.red() >= 254 && color.green() >= 254 &&
                   color.blue() >= 254;
        };
        QCOMPARE(image.pixelColor(130, 37), QColor(QStringLiteral("#ffffff")));
        QCOMPARE(image.pixelColor(270, 37), QColor(QStringLiteral("#ffffff")));
        QCOMPARE(image.pixelColor(60, 10), panel_white);
        QCOMPARE(image.pixelColor(350, 20), band_gray);
        QVERIFY(near_white(image.pixelColor(133, 33)));
        QCOMPARE(image.pixelColor(136, 30), band_gray);
        QCOMPARE(image.pixelColor(139, 29), band_gray);

        bar->setProperty("activeDocumentId", QStringLiteral("import-1"));
        QTest::mouseMove(&window, QPoint(350, 20));
        const auto secondResult = bar->grabToImage();
        QVERIFY(secondResult != nullptr);
        QSignalSpy secondReady(secondResult.data(), &QQuickItemGrabResult::ready);
        QVERIFY(secondReady.wait(5000));
        const QImage secondImage = secondResult->image();
        QCOMPARE(secondImage.pixelColor(127, 29), band_gray);
        QVERIFY(near_white(secondImage.pixelColor(133, 33)));
        QVERIFY(near_white(secondImage.pixelColor(265, 33)));
        QCOMPARE(secondImage.pixelColor(271, 29), band_gray);

        QQuickItem* importTab = visual_item(bar, QStringLiteral("documentTab-import-1"));
        QVERIFY(importTab != nullptr);
        QQuickItem* importClose = visual_item(importTab, QStringLiteral("documentTabClose"));
        QVERIFY(importClose != nullptr);
        QTest::mouseMove(&window, QPoint(251, 20));
        QTRY_COMPARE(importClose->property("color").value<QColor>(),
                     QColor(QStringLiteral("#e5f1fb")));
        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, QPoint(251, 20));
        QTRY_COMPARE(importClose->property("color").value<QColor>(),
                     QColor(QStringLiteral("#d5e3ef")));
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, QPoint(251, 20));
        QTest::mouseMove(&window, QPoint(350, 20));
        QTRY_COMPARE(importClose->property("color").value<QColor>(), Qt::transparent);

        QQuickItem* welcomeTab = visual_item(bar, QStringLiteral("documentTab-welcome"));
        QVERIFY(welcomeTab != nullptr);
        QQuickItem* welcomeClose = visual_item(welcomeTab, QStringLiteral("documentTabClose"));
        QVERIFY(welcomeClose != nullptr);
        QTest::mouseMove(&window, QPoint(119, 20));
        QTRY_COMPARE(welcomeClose->property("color").value<QColor>(),
                     QColor(QStringLiteral("#e5f1fb")));
        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, QPoint(119, 20));
        QTRY_COMPARE(welcomeClose->property("color").value<QColor>(),
                     QColor(QStringLiteral("#d5e3ef")));
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, QPoint(119, 20));
        QTest::mouseMove(&window, QPoint(350, 20));
        QTRY_COMPARE(welcomeClose->property("color").value<QColor>(), Qt::transparent);

        QSignalSpy activated(bar, SIGNAL(activateDocument(QString)));
        QTest::mouseClick(&window, Qt::LeftButton, Qt::NoModifier, QPoint(200, 20));
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-1"));
        QQuickItem* dragArea = visual_item(welcomeTab, QStringLiteral("documentTabDragArea"));
        QVERIFY(dragArea != nullptr);
        bar->setProperty("order",
                         QStringList{QStringLiteral("import-1"), QStringLiteral("welcome")});
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(welcomeTab->x() - 134.0) < 0.1, 10000);

        bar->setProperty("order",
                         QStringList{QStringLiteral("welcome"), QStringLiteral("import-1")});
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(welcomeTab->x() - 2.0) < 0.1, 10000);

        // 垂直手势不开始拖动，也不改变显示顺序。
        const QStringList initialOrder = bar->property("order").toStringList();
        QVERIFY(QMetaObject::invokeMethod(bar, "begin_press", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 20.0), Q_ARG(QVariant, 20.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "detect_drag", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 21.0), Q_ARG(QVariant, 50.0)));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QCOMPARE(bar->property("order").toStringList(), initialOrder);

        // 关闭按钮独占按压；拖出按钮区域不能启动页签拖拽或关闭标签。
        QSignalSpy closed(bar, SIGNAL(closeDocument(QString)));
        const QPoint closeCenter =
            importClose->mapToScene(QPointF(importClose->width() / 2, importClose->height() / 2))
                .toPoint();
        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, closeCenter);
        QTest::mouseMove(&window, closeCenter + QPoint(-40, 0));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, closeCenter + QPoint(-40, 0));
        QCOMPARE(closed.count(), 0);

        QVERIFY(QMetaObject::invokeMethod(bar, "begin_press", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 20.0), Q_ARG(QVariant, 20.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "detect_drag", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 30.0), Q_ARG(QVariant, 20.0)));
        QCOMPARE(bar->property("draggedId").toString(), QStringLiteral("welcome"));
        QVERIFY(welcomeTab->x() > 2.0);
        QVERIFY(QMetaObject::invokeMethod(bar, "drag_move", Q_ARG(QVariant, 60.0)));
        QVERIFY(welcomeTab->x() > 2.0);
        QVERIFY(QMetaObject::invokeMethod(bar, "finish_drag", Q_ARG(QVariant, true)));
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(welcomeTab->x() - 2.0) < 0.1, 10000);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("welcome"));

        // 悬停移动不能复用上一次按下坐标；真实左键拖动才进入拖拽态。
        QTest::mouseMove(&window, QPoint(20, 20));
        QTest::mouseMove(&window, QPoint(80, 20));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QCOMPARE(dragArea->property("cursorShape").toInt(), int(Qt::OpenHandCursor));
        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, QPoint(20, 20));
        QTest::mouseMove(&window, QPoint(40, 20));
        QTRY_COMPARE(bar->property("draggedId").toString(), QStringLiteral("welcome"));
        QCOMPARE(dragArea->property("cursorShape").toInt(), int(Qt::ClosedHandCursor));
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, QPoint(40, 20));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QCOMPARE(dragArea->property("cursorShape").toInt(), int(Qt::OpenHandCursor));
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("welcome"));
        QTest::mouseMove(&window, QPoint(80, 20));
        QCOMPARE(bar->property("draggedId").toString(), QString{});

        QSignalSpy moved(bar, SIGNAL(moveDocument(int, int)));
        QVERIFY(QMetaObject::invokeMethod(bar, "begin_press", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 20.0), Q_ARG(QVariant, 20.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "detect_drag", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 40.0), Q_ARG(QVariant, 20.0)));
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("welcome"));
        QVERIFY(QMetaObject::invokeMethod(bar, "drag_move", Q_ARG(QVariant, 220.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "finish_drag", Q_ARG(QVariant, true)));
        QCOMPARE(moved.count(), 1);
        QCOMPARE(moved.takeFirst(), (QList<QVariant>{0, 1}));
        QVariantList reordered{documents[1], documents[0]};
        bar->setProperty("documents", reordered);
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(welcomeTab->x() - 134.0) < 0.1, 10000);

        bar->setProperty("reducedMotion", true);
        bar->setProperty("order",
                         QStringList{QStringLiteral("welcome"), QStringLiteral("import-1")});
        QCOMPARE(welcomeTab->x(), 2.0);
        bar->setProperty("order",
                         QStringList{QStringLiteral("import-1"), QStringLiteral("welcome")});
        QCOMPARE(welcomeTab->x(), 134.0);
    }
    void welcome_document_tab_retranslates_visible_and_accessible_titles() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QQmlComponent component(
            &engine, QUrl(QStringLiteral(
                         "qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));

        const QVariantList documents{
            QVariantMap{{QStringLiteral("id"), QStringLiteral("welcome")},
                        {QStringLiteral("kind"), QStringLiteral("welcome")},
                        {QStringLiteral("state"), QStringLiteral("ready")},
                        {QStringLiteral("title"), QStringLiteral("Welcome")},
                        {QStringLiteral("message"), QString{}}}};
        auto* bar = qobject_cast<QQuickItem*>(component.createWithInitialProperties(
            QVariantMap{{QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), QStringLiteral("welcome")}}));
        QVERIFY(bar != nullptr);
        bar->setParent(&owner);
        bar->setWidth(240);
        bar->setHeight(39);

        QQuickWindow window;
        window.resize(240, 39);
        bar->setParentItem(window.contentItem());
        window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&window), "window was not exposed");

        auto* tab = visual_item(bar, QStringLiteral("documentTab-welcome"));
        QVERIFY(tab != nullptr);
        auto* title = visual_item(tab, QStringLiteral("documentTabTitle"));
        auto* close = visual_item(tab, QStringLiteral("documentTabClose"));
        QVERIFY(title != nullptr);
        QVERIFY(close != nullptr);
        auto* tab_accessible = QAccessible::queryAccessibleInterface(tab);
        auto* close_accessible = QAccessible::queryAccessibleInterface(close);
        QVERIFY(tab_accessible != nullptr);
        QVERIFY(close_accessible != nullptr);
        QCOMPARE(title->property("text").toString(), QStringLiteral("Welcome"));
        QCOMPARE(tab_accessible->text(QAccessible::Name), QStringLiteral("Welcome"));
        QCOMPARE(close_accessible->text(QAccessible::Name), QStringLiteral("Close Welcome"));

        WelcomeTranslator translator;
        QVERIFY(QCoreApplication::installTranslator(&translator));
        engine.retranslate();
        QTRY_COMPARE(title->property("text").toString(), QStringLiteral("欢迎"));
        QTRY_COMPARE(tab_accessible->text(QAccessible::Name), QStringLiteral("欢迎"));
        QTRY_COMPARE(close_accessible->text(QAccessible::Name), QStringLiteral("Close 欢迎"));

        QVERIFY(QCoreApplication::removeTranslator(&translator));
        engine.retranslate();
        QTRY_COMPARE(title->property("text").toString(), QStringLiteral("Welcome"));
        QTRY_COMPARE(tab_accessible->text(QAccessible::Name), QStringLiteral("Welcome"));
        QTRY_COMPARE(close_accessible->text(QAccessible::Name), QStringLiteral("Close Welcome"));
    }
    void duplicate_document_tabs_are_visibly_and_accessibly_distinguished() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QQmlComponent component(
            &engine, QUrl(QStringLiteral(
                         "qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));

        const auto document = [](const QString& id, const QString& kind, const QString& title,
                                 int ordinal = 1, int count = 1) {
            return QVariantMap{{QStringLiteral("id"), id},
                               {QStringLiteral("kind"), kind},
                               {QStringLiteral("state"), QStringLiteral("ready")},
                               {QStringLiteral("title"), title},
                               {QStringLiteral("displayOrdinal"), ordinal},
                               {QStringLiteral("displayCount"), count},
                               {QStringLiteral("message"), QString{}}};
        };
        const QString longTitle = QStringLiteral("exceptionally-long-source-name");
        const QVariantList documents{
            document(QStringLiteral("welcome"), QStringLiteral("welcome"),
                     QStringLiteral("Welcome")),
            document(QStringLiteral("import-1"), QStringLiteral("import"), longTitle, 1, 2),
            document(QStringLiteral("import-2"), QStringLiteral("import"), longTitle, 2, 2)};
        auto* bar = qobject_cast<QQuickItem*>(component.createWithInitialProperties(
            QVariantMap{{QStringLiteral("objectName"), QStringLiteral("documentTabBar")},
                        {QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), QStringLiteral("import-1")}}));
        QVERIFY(bar != nullptr);
        bar->setParent(&owner);
        bar->setWidth(240);
        bar->setHeight(39);

        QQuickWindow window;
        window.resize(240, 39);
        bar->setParentItem(window.contentItem());
        window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&window), "window was not exposed");

        QQuickItem* firstTab = visual_item(bar, QStringLiteral("documentTab-import-1"));
        QQuickItem* secondTab = visual_item(bar, QStringLiteral("documentTab-import-2"));
        QVERIFY(firstTab != nullptr);
        QVERIFY(secondTab != nullptr);
        QQuickItem* firstTitle = visual_item(firstTab, QStringLiteral("documentTabTitle"));
        QQuickItem* secondTitle = visual_item(secondTab, QStringLiteral("documentTabTitle"));
        QVERIFY(firstTitle != nullptr);
        QVERIFY(secondTitle != nullptr);
        QCOMPARE(firstTab->width(), 130.0);
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(firstTab->x() - 134.0) < 0.1, 10000);
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(secondTab->x() - 266.0) < 0.1, 10000);
        QCOMPARE(firstTitle->property("text").toString(), QStringLiteral("1 · ") + longTitle);
        QCOMPARE(secondTitle->property("text").toString(), QStringLiteral("2 · ") + longTitle);
        QTRY_VERIFY(firstTitle->property("truncated").toBool());
        QTRY_VERIFY(secondTitle->property("truncated").toBool());

        QAccessibleInterface* accessibleTab = QAccessible::queryAccessibleInterface(firstTab);
        QVERIFY(accessibleTab != nullptr);
        QCOMPARE(accessibleTab->role(), QAccessible::PageTab);
        QCOMPARE(accessibleTab->text(QAccessible::Name), longTitle + QStringLiteral(", import 1"));
        QAccessibleInterface* accessibleSecondTab =
            QAccessible::queryAccessibleInterface(secondTab);
        QVERIFY(accessibleSecondTab != nullptr);
        QCOMPARE(accessibleTab->state().selected, true);
        QCOMPARE(accessibleSecondTab->state().selected, false);
        bar->setProperty("activeDocumentId", QStringLiteral("import-2"));
        QTRY_VERIFY(!accessibleTab->state().selected);
        QTRY_VERIFY(accessibleSecondTab->state().selected);
        bar->setProperty("activeDocumentId", QStringLiteral("import-1"));
        QTRY_VERIFY(accessibleTab->state().selected);
        QTRY_VERIFY(!accessibleSecondTab->state().selected);

        QSignalSpy activated(bar, SIGNAL(activateDocument(QString)));
        QCOMPARE(firstTab->property("activeFocusOnTab").toBool(), true);
        firstTab->forceActiveFocus();
        QTRY_VERIFY(firstTab->hasActiveFocus());
        const auto focusedImageResult = bar->grabToImage();
        QVERIFY(focusedImageResult);
        QSignalSpy focusedImageReady(focusedImageResult.data(), &QQuickItemGrabResult::ready);
        QVERIFY(focusedImageReady.wait(5000));
        QCOMPARE(focusedImageResult->image().pixelColor(150, 3), QColor(Qt::white));
        QTest::keyClick(&window, Qt::Key_Return);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-1"));

        QQuickItem* closeButton = visual_item(secondTab, QStringLiteral("documentTabClose"));
        QVERIFY(closeButton != nullptr);
        closeButton->forceActiveFocus();
        QTRY_VERIFY(closeButton->hasActiveFocus());
        QTest::mouseClick(
            &window, Qt::LeftButton, Qt::NoModifier,
            firstTab->mapToScene(QPointF(firstTab->width() / 2, firstTab->height() / 2)).toPoint());
        QTRY_VERIFY(firstTab->hasActiveFocus());
        QTRY_VERIFY(accessibleTab->state().focused);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-1"));

        QQuickItem* scroller = visual_item(bar, QStringLiteral("documentTabScroller"));
        QVERIFY(scroller != nullptr);
        secondTab->forceActiveFocus();
        QSignalSpy closed(bar, SIGNAL(closeDocument(QString)));
        QAccessibleInterface* accessibleClose = QAccessible::queryAccessibleInterface(
            visual_item(secondTab, QStringLiteral("documentTabClose")));
        QVERIFY(accessibleClose != nullptr);
        QCOMPARE(accessibleClose->role(), QAccessible::Button);
        QCOMPARE(accessibleClose->text(QAccessible::Name),
                 QStringLiteral("Close ") + longTitle + QStringLiteral(", import 2"));
        QTest::keyClick(&window, Qt::Key_Left);
        QTRY_VERIFY(firstTab->hasActiveFocus());
        QTRY_VERIFY(scroller->property("contentX").toReal() > 0.0);
        const qreal contentAfterLeft = scroller->property("contentX").toReal();
        QTest::keyClick(&window, Qt::Key_Right);
        QTRY_VERIFY(secondTab->hasActiveFocus());
        QTRY_VERIFY(scroller->property("contentX").toReal() > contentAfterLeft);
        QTRY_VERIFY(secondTab->x() + secondTab->width() <=
                    scroller->property("contentX").toReal() + scroller->width() + 0.1);
        QCOMPARE(activated.count(), 2);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-1"));
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-2"));
        QTest::keyClick(&window, Qt::Key_Space);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-2"));

        QCOMPARE(closeButton->property("activeFocusOnTab").toBool(), true);
        closeButton->forceActiveFocus();
        QTRY_VERIFY(closeButton->hasActiveFocus());
        QTest::keyClick(&window, Qt::Key_Space);
        QCOMPARE(closed.count(), 1);
        QCOMPARE(closed.takeFirst().at(0).toString(), QStringLiteral("import-2"));
    }
    void document_tab_drag_crosses_multiple_neighbors_without_changing_identity() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QQmlComponent component(
            &engine, QUrl(QStringLiteral(
                         "qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));

        const auto document = [](const QString& id) {
            return QVariantMap{{QStringLiteral("id"), id},
                               {QStringLiteral("kind"), QStringLiteral("import")},
                               {QStringLiteral("state"), QStringLiteral("ready")},
                               {QStringLiteral("title"), id + QStringLiteral(".stl")},
                               {QStringLiteral("message"), QString{}}};
        };
        const QVariantList documents{
            document(QStringLiteral("doc-1")), document(QStringLiteral("doc-2")),
            document(QStringLiteral("doc-3")), document(QStringLiteral("doc-4"))};
        auto* bar = qobject_cast<QQuickItem*>(component.createWithInitialProperties(
            QVariantMap{{QStringLiteral("objectName"), QStringLiteral("documentTabBar")},
                        {QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), QStringLiteral("doc-4")}}));
        QVERIFY(bar != nullptr);
        bar->setParent(&owner);
        bar->setWidth(600);
        bar->setHeight(39);

        QQuickWindow window;
        window.resize(600, 80);
        bar->setParentItem(window.contentItem());
        window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&window), "window was not exposed");

        QQuickItem* draggedTab = visual_item(bar, QStringLiteral("documentTab-doc-1"));
        QVERIFY(draggedTab != nullptr);
        QCOMPARE(draggedTab->y(), 3.0);
        QSignalSpy activated(bar, SIGNAL(activateDocument(QString)));
        QSignalSpy moved(bar, SIGNAL(moveDocument(int, int)));
        const QStringList initialOrder = bar->property("order").toStringList();
        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, QPoint(20, 20));
        QTest::mouseMove(&window, QPoint(21, 55));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QCOMPARE(bar->property("order").toStringList(), initialOrder);
        QCOMPARE(activated.count(), 0);
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, QPoint(21, 55));

        QTest::mousePress(&window, Qt::LeftButton, Qt::NoModifier, QPoint(20, 20));
        QTest::mouseMove(&window, QPoint(200, 20));
        QTRY_COMPARE(bar->property("draggedId").toString(), QStringLiteral("doc-1"));
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(draggedTab->x() - 182.0) < 0.1, 10000);
        QCOMPARE(draggedTab->opacity(), 1.0);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("doc-1"));
        bar->setProperty("activeDocumentId", QStringLiteral("doc-1"));

        QTest::mouseMove(&window, QPoint(520, 20));
        QTRY_COMPARE(bar->property("order").toStringList(),
                     (QStringList{QStringLiteral("doc-2"), QStringLiteral("doc-3"),
                                  QStringLiteral("doc-4"), QStringLiteral("doc-1")}));
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(draggedTab->x() - 470.0) < 0.1, 10000);
        QCOMPARE(draggedTab->y(), 3.0);
        QTest::mouseRelease(&window, Qt::LeftButton, Qt::NoModifier, QPoint(520, 20));
        QCOMPARE(bar->property("draggedId").toString(), QString{});
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(draggedTab->x() - 398.0) < 0.1, 10000);
        QCOMPARE(moved.count(), 1);
        QCOMPARE(moved.takeFirst(), (QList<QVariant>{0, 3}));

        QVariantList reordered{documents[1], documents[2], documents[3], documents[0]};
        bar->setProperty("documents", reordered);
        QQuickItem* movedTab = visual_item(bar, QStringLiteral("documentTab-doc-1"));
        QVERIFY(movedTab != nullptr);
        QTRY_VERIFY_WITH_TIMEOUT(qAbs(movedTab->x() - 398.0) < 0.1, 10000);
        QVERIFY(movedTab->property("isActive").toBool());
        QQuickItem* previousActiveTab = visual_item(bar, QStringLiteral("documentTab-doc-4"));
        QVERIFY(previousActiveTab != nullptr);
        QCOMPARE(previousActiveTab->property("isActive").toBool(), false);
    }
    void document_tab_scroll_preserves_active_edge_connectors() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QQmlComponent component(
            &engine, QUrl(QStringLiteral(
                         "qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));

        const auto document = [](const QString& id, const QString& kind, const QString& title) {
            return QVariantMap{{QStringLiteral("id"), id},
                               {QStringLiteral("kind"), kind},
                               {QStringLiteral("state"), QStringLiteral("ready")},
                               {QStringLiteral("title"), title},
                               {QStringLiteral("message"), QString{}}};
        };
        const QVariantList documents{document(QStringLiteral("welcome"), QStringLiteral("welcome"),
                                              QStringLiteral("Welcome")),
                                     document(QStringLiteral("import-1"), QStringLiteral("import"),
                                              QStringLiteral("one.stl")),
                                     document(QStringLiteral("import-2"), QStringLiteral("import"),
                                              QStringLiteral("two.stl"))};
        auto* bar = qobject_cast<QQuickItem*>(component.createWithInitialProperties(
            QVariantMap{{QStringLiteral("objectName"), QStringLiteral("documentTabBar")},
                        {QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), QStringLiteral("welcome")}}));
        QVERIFY(bar != nullptr);
        bar->setParent(&owner);
        bar->setWidth(240);
        bar->setHeight(39);

        QQuickWindow window;
        window.resize(240, 39);
        bar->setParentItem(window.contentItem());
        window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&window), "window was not exposed");

        const QColor bandGray(QStringLiteral("#e7e4e1"));
        const QColor panelWhite(QStringLiteral("#ffffff"));
        const auto grab = [bar]() {
            const auto result = bar->grabToImage();
            if (!result) {
                return QImage{};
            }
            QSignalSpy ready(result.data(), &QQuickItemGrabResult::ready);
            if (!ready.wait(5000)) {
                return QImage{};
            }
            return result->image();
        };

        const QImage firstImage = grab();
        QCOMPARE(firstImage.size(), QSize(240, 39));
        QVERIFY(firstImage.pixelColor(0, 34).lightness() > bandGray.lightness());

        bar->setProperty("activeDocumentId", QStringLiteral("import-2"));
        QQuickItem* scroller = visual_item(bar, QStringLiteral("documentTabScroller"));
        QQuickItem* lastTab = visual_item(bar, QStringLiteral("documentTab-import-2"));
        QVERIFY(scroller != nullptr);
        QVERIFY(lastTab != nullptr);
        const qreal endContentX = scroller->property("contentWidth").toReal() - scroller->width();
        scroller->setProperty("contentX", endContentX);
        QTRY_VERIFY(qAbs(scroller->property("contentX").toReal() - endContentX) < 0.1);
        const QPointF lastTabPosition = lastTab->mapToItem(bar, 0, 0);
        QVERIFY(qAbs(lastTabPosition.x() + lastTab->width() - 232.0) < 0.1);

        const QImage lastImage = grab();
        QCOMPARE(lastImage.size(), QSize(240, 39));
        QVERIFY(lastImage.pixelColor(234, 34).lightness() > bandGray.lightness());
        QCOMPARE(lastImage.pixelColor(236, 34), bandGray);
        QCOMPARE(lastImage.pixelColor(130, 37), panelWhite);
    }
};

QTEST_MAIN(DocumentTabBarTest)
#include "document_tab_bar_test.moc"
