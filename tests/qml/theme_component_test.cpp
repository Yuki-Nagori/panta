// QML 组件的主题参数、资源解析及 Ribbon 页签组合边界回归。
#include "quick_item_helpers.hpp"
#include <QColor>
#include <QDir>
#include <QFont>
#include <QGuiApplication>
#include <QImage>
#include <QImageReader>
#include <QObject>
#include <QPointer>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QQuickItem>
#include <QQuickItemGrabResult>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>
#include <qaccessible.h>
#include <qaccessible_base.h>
#include <qtestkeyboard.h>
#include <qtestmouse.h>
#include <qtestsupport_core.h>
#include <qtestsupport_gui.h>

namespace {

QObject* create_component(QQmlEngine& engine, const QString& path, QObject& owner) {
    QQmlComponent component(&engine, QUrl(path));
    if (!component.isReady()) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    QObject* object = component.create();
    if (object == nullptr) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    object->setParent(&owner);
    return object;
}

} // namespace

class ThemeComponentTest final : public QObject {
    Q_OBJECT

  private slots:
    void defaults_use_theme() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* label = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedLabel.qml"),
            owner);
        QVERIFY(label != nullptr);
        QCOMPARE(label->property("textColor").value<QColor>(), QColor(QStringLiteral("#1c1c1c")));
        QCOMPARE(label->property("textSize").toInt(), 13);

        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        QCOMPARE(button->property("controlHeight").toInt(), 24);
        QCOMPARE(button->property("contentPadding").toInt(), 4);
        QCOMPARE(button->property("contentColor").value<QColor>(),
                 QColor(QStringLiteral("#4a4a4a")));
        QCOMPARE(button->property("implicitHeight").toInt(), 24);

        QObject* surface = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/PanelSurface.qml"),
            owner);
        QVERIFY(surface != nullptr);
        QCOMPARE(surface->property("surfaceColor").value<QColor>(),
                 QColor(QStringLiteral("#ffffff")));
    }

    void explicit_values_override_theme() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        button->setProperty("controlHeight", 48);
        button->setProperty("contentPadding", 20);
        QCOMPARE(button->property("controlHeight").toInt(), 48);
        QCOMPARE(button->property("contentPadding").toInt(), 20);
        QCOMPARE(button->property("implicitHeight").toInt(), 48);
        QCOMPARE(button->property("leftPadding").toInt(), 20);
        QCOMPARE(button->property("rightPadding").toInt(), 20);
        // 未覆盖属性仍取 Theme 默认：显式覆盖不得破坏其余绑定。
        QCOMPARE(button->property("contentColor").value<QColor>(),
                 QColor(QStringLiteral("#4a4a4a")));
    }

    void disabled_state_is_visibly_weakened() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        QCOMPARE(button->property("opacity").toDouble(), 1.0);
        button->setProperty("enabled", false);
        QCOMPARE(button->property("opacity").toDouble(), 0.4);
    }

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
        QTest::qWait(50);
        QSignalSpy newRequested(ribbon, SIGNAL(newProjectRequested()));
        QSignalSpy openRequested(ribbon, SIGNAL(openProjectRequested()));
        QVERIFY(newRequested.isValid() && openRequested.isValid());
        auto* ribbonItem = qobject_cast<QQuickItem*>(ribbon);
        auto* loader = visual_item(ribbonItem, QStringLiteral("ribbonLoader"));
        QVERIFY(loader != nullptr);
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
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/TasksPanel.qml"), owner);
        QVERIFY(panel != nullptr);
        panel->setProperty("importedPartIds", QStringList{QStringLiteral("import-1")});
        panel->setProperty("importedPartName", QStringLiteral("latest.stl"));
        panel->setProperty("activeDocumentId", QStringLiteral("welcome"));
        panel->setProperty("activeDocumentTitle", QStringLiteral("Welcome"));
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("latest.stl"));

        panel->setProperty("activeDocumentId", QStringLiteral("import-1"));
        panel->setProperty("activeDocumentTitle", QStringLiteral("active.stl"));
        QCOMPARE(panel->property("activePartTitle").toString(), QStringLiteral("active.stl"));
    }

    void button_font_size_reaches_its_label() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        button->setProperty("text", QStringLiteral("Font override"));
        QObject* label = nullptr;
        for (auto* child : button->findChildren<QObject*>()) {
            if (child->property("text").toString() == QStringLiteral("Font override")) {
                label = child;
                break;
            }
        }
        QVERIFY(label != nullptr);
        QFont font = button->property("font").value<QFont>();
        font.setPixelSize(19);
        button->setProperty("font", font);
        QCOMPARE(label->property("font").value<QFont>().pixelSize(), 19);
    }

    void icon_resolves_module_resource() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* icon = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedIcon.qml"),
            owner);
        QVERIFY(icon != nullptr);
        icon->setProperty("name", QStringLiteral("caret-down"));
        const QUrl source = icon->property("source").value<QUrl>();
        QCOMPARE(source, QUrl(QStringLiteral("image://panta-icons/caret-down/ff4a4a4a")));
        // qtsvg 供给的 qsvg 图像格式插件必须能解码模块内 SVG 资源。
        QImageReader reader(QStringLiteral(":/qt/qml/Panta/Shell/icons/caret-down.svg"));
        QCOMPARE(reader.canRead(), true);
        QCOMPARE(reader.size(), QSize(24, 24));
        icon->setProperty("color", QColor(QStringLiteral("#ffffff")));
        QCOMPARE(icon->property("source").value<QUrl>(),
                 QUrl(QStringLiteral("image://panta-icons/caret-down/ffffffff")));
        icon->setProperty("name", QString());
        QVERIFY(icon->property("source").value<QUrl>().isEmpty());
    }
    void svg_resources_render_with_caller_color() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        auto* provider = static_cast<QQuickImageProvider*>(engine.imageProvider("panta-icons"));
        QVERIFY(provider != nullptr);
        const QStringList icons = QDir(QStringLiteral(":/qt/qml/Panta/Shell/icons"))
                                      .entryList({QStringLiteral("*.svg")}, QDir::Files);
        QVERIFY(!icons.isEmpty());
        for (const QString& file : icons) {
            for (const int pixels : {16, 18, 24, 48}) {
                QSize original;
                const QString name = file.chopped(4);
                const QImage frame =
                    provider->requestImage(name + "/ff2878b8", &original, QSize(pixels, pixels));
                QVERIFY2(!frame.isNull(), qPrintable(file));
                QCOMPARE(original, QSize(24, 24));
                QCOMPARE(frame.size(), QSize(pixels, pixels));
                int colored = 0;
                for (int y = 0; y < pixels; ++y) {
                    for (int x = 0; x < pixels; ++x) {
                        const QColor pixel = frame.pixelColor(x, y);
                        if (pixel.alpha() > 240) {
                            // 允许预乘 alpha 的整数舍入；不允许硬编码黑色漏出。
                            QVERIFY(qAbs(pixel.red() - 40) <= 1);
                            QVERIFY(qAbs(pixel.green() - 120) <= 1);
                            QVERIFY(qAbs(pixel.blue() - 184) <= 1);
                            ++colored;
                        }
                        if (x == 0 || y == 0 || x == pixels - 1 || y == pixels - 1) {
                            QCOMPARE(pixel.alpha(), 0);
                        }
                    }
                }
                QVERIFY2(colored > 0, qPrintable(file));
            }
        }
        QSize original;
        const QImage white =
            provider->requestImage("pane-close/ffffffff", &original, QSize(24, 24));
        QVERIFY(white.pixelColor(6, 6).lightness() > 240);
        const QImage transparent =
            provider->requestImage("pane-close/00ffffff", nullptr, QSize(24, 24));
        QCOMPARE(transparent.pixelColor(6, 6).alpha(), 0);
        QVERIFY(provider->requestImage("../document-new/ffffffff", nullptr, {}).isNull());
        QVERIFY(provider->requestImage("pane-close/not-a-color", nullptr, {}).isNull());
    }

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
        QTest::mouseMove(&window, QPoint(80, 20));
        QCOMPARE(bar->property("draggedId").toString(), QString{});

        QSignalSpy moved(bar, SIGNAL(moveDocument(int, int)));
        QVERIFY(QMetaObject::invokeMethod(bar, "begin_press", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 20.0), Q_ARG(QVariant, 20.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "detect_drag", Q_ARG(QVariant, "welcome"),
                                          Q_ARG(QVariant, 40.0), Q_ARG(QVariant, 20.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "drag_move", Q_ARG(QVariant, 220.0)));
        QVERIFY(QMetaObject::invokeMethod(bar, "finish_drag", Q_ARG(QVariant, true)));
        QCOMPARE(moved.count(), 1);
        QCOMPARE(moved.takeFirst(), (QList<QVariant>{0, 1}));
        QVariantList reordered{documents[1], documents[0]};
        bar->setProperty("documents", reordered);
        QTRY_COMPARE(welcomeTab->x(), 134.0);
        QTest::qWait(200);
        QCOMPARE(welcomeTab->x(), 134.0);
    }

    void duplicate_document_tabs_are_visibly_and_accessibly_distinguished() {
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
                                              QStringLiteral("sample.stl")),
                                     document(QStringLiteral("import-2"), QStringLiteral("import"),
                                              QStringLiteral("sample.stl"))};
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
        QCOMPARE(firstTitle->property("text").toString(), QStringLiteral("1 · sample.stl"));
        QCOMPARE(secondTitle->property("text").toString(), QStringLiteral("2 · sample.stl"));

        QAccessibleInterface* accessibleTab = QAccessible::queryAccessibleInterface(firstTab);
        QVERIFY(accessibleTab != nullptr);
        QCOMPARE(accessibleTab->role(), QAccessible::PageTab);
        QCOMPARE(accessibleTab->text(QAccessible::Name), QStringLiteral("sample.stl, import 1"));

        QSignalSpy activated(bar, SIGNAL(activateDocument(QString)));
        QCOMPARE(firstTab->property("activeFocusOnTab").toBool(), true);
        firstTab->forceActiveFocus();
        QTRY_VERIFY(firstTab->hasActiveFocus());
        QTest::keyClick(&window, Qt::Key_Return);
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
                 QStringLiteral("Close sample.stl, import 2"));
        QTest::keyClick(&window, Qt::Key_Left);
        QTRY_VERIFY(firstTab->hasActiveFocus());
        QTRY_VERIFY(scroller->property("contentX").toReal() > 0.0);
        const qreal contentAfterLeft = scroller->property("contentX").toReal();
        QTest::keyClick(&window, Qt::Key_Right);
        QTRY_VERIFY(secondTab->hasActiveFocus());
        QTRY_VERIFY(scroller->property("contentX").toReal() > contentAfterLeft);
        QCOMPARE(activated.count(), 2);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-1"));
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-2"));
        QTest::keyClick(&window, Qt::Key_Space);
        QCOMPARE(activated.count(), 1);
        QCOMPARE(activated.takeFirst().at(0).toString(), QStringLiteral("import-2"));

        QQuickItem* closeButton = visual_item(secondTab, QStringLiteral("documentTabClose"));
        QVERIFY(closeButton != nullptr);
        QCOMPARE(closeButton->property("activeFocusOnTab").toBool(), true);
        closeButton->forceActiveFocus();
        QTRY_VERIFY(closeButton->hasActiveFocus());
        QTest::keyClick(&window, Qt::Key_Space);
        QCOMPARE(closed.count(), 1);
        QCOMPARE(closed.takeFirst().at(0).toString(), QStringLiteral("import-2"));
    }
};

QTEST_MAIN(ThemeComponentTest)
#include "theme_component_test.moc"
