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
#include <QQuickWindow>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>
#include <qtestsupport_core.h>

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
};

QTEST_MAIN(ThemeComponentTest)
#include "theme_component_test.moc"
