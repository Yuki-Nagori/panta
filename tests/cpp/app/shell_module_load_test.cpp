// Panta.Shell 运行时加载测试：验证资源模块及其可选 Bridge 依赖的链接契约。

#include <QByteArray>
#include <QClipboard>
#include <QGuiApplication>
#include <QMimeData>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQuickWindow>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtQuickControls2/qquickstyle.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <QtTest/qtestmouse.h>
#include <edit_command_router.hpp>
#include <icon_provider.hpp>
#include <memory>
#include <qcontainerfwd.h>
#include <qlist.h>
#include <qlogging.h>
#include <qtenvironmentvariables.h>
#include <qtestsupport_core.h>
#ifdef PANTA_ENABLE_BRIDGE_MODULE
#include "../../support/qml/quick_item_helpers.hpp"
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QIODevice>
#include <QImage>
#include <QMetaEnum>
#include <QMetaProperty>
#include <QPoint>
#include <QPointF>
#include <QPointer>
#include <QQuickItem>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QUrl>
#include <QVariant>
#include <QtCore/qnamespace.h>
#include <QtTest/qtestkeyboard.h>
#include <project_view_model.hpp>
// App.qml 引入 Panta.Visualization（CaeViewport）：静态模块的消费方二进制
// 必须同时导入并链接其 plugin，否则运行时报 "module not installed"。
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)

namespace {

// 按需打印可视树几何，辅助定位嵌套布局与内容裁剪问题。
void dump_item_tree(const QQuickItem* item, int depth) {
    qWarning().nospace() << QString(static_cast<qsizetype>(depth) * 2, ' ')
                         << item->metaObject()->className() << " objectName=" << item->objectName()
                         << " geom=" << item->x() << "," << item->y() << " " << item->width() << "x"
                         << item->height();
    if (depth < 12) {
        const auto children = item->childItems();
        for (const QQuickItem* child : children) {
            dump_item_tree(child, depth + 1);
        }
    }
}

} // namespace
#endif

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)

class ShellModuleLoadTest final : public QObject {
    Q_OBJECT

#ifdef PANTA_ENABLE_BRIDGE_MODULE
    void verify_ribbon_alignment(QQuickItem* ribbon) {
        QVERIFY(ribbon != nullptr);
        const auto rows = visual_items(ribbon, QStringLiteral("ribbonTools"));
        QVERIFY(!rows.isEmpty());
        for (auto* tools : rows) {
            const auto* group = tools->parentItem();
            QVERIFY(group != nullptr);
            QCOMPARE(tools->x() + tools->width() / 2, (group->width() - 1) / 2);
        }
        for (const char* name : {"ribbonTileIcon", "ribbonTileLabel"}) {
            const auto items = visual_items(ribbon, QString::fromLatin1(name));
            QVERIFY(!items.isEmpty());
            const qreal top = items.constFirst()->mapToItem(ribbon, QPointF()).y();
            for (auto* item : items)
                QCOMPARE(item->mapToItem(ribbon, QPointF()).y(), top);
        }
    }

    void activate_menu(QQuickWindow* window, const char* key, bool keyboard) {
        auto* menu = visual_item(window->contentItem(), QStringLiteral("menu-") + key);
        auto* strip = window->findChild<QQuickItem*>(QStringLiteral("menuStrip"));
        QVERIFY(menu && strip);
        // 新 delegate 的 Row 布局延迟到 polish；先完成定位再计算鼠标命中点。
        QVERIFY(menu->parentItem() != nullptr);
        menu->parentItem()->ensurePolished();
        // 模态新建窗口关闭后，先恢复宿主激活，再发送键盘事件。
        window->requestActivate();
        QTRY_COMPARE(QGuiApplication::focusWindow(), window);
        menu->forceActiveFocus(Qt::TabFocusReason);
        QTRY_COMPARE(window->activeFocusItem(), menu);
        QTRY_VERIFY(menu->mapToItem(strip, QPointF()).x() >= -1);
        QTRY_VERIFY(menu->mapToItem(strip, QPointF()).x() + menu->width() <= strip->width() + 1);
        QSignalSpy clicked(menu, SIGNAL(clicked()));
        QVERIFY(clicked.isValid());
        if (keyboard) {
            QTest::keyClick(window, Qt::Key_Space);
        } else {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                menu->mapToScene(QPointF(menu->width() / 2, menu->height() / 2)).toPoint());
        }
        QVERIFY2(clicked.count() == 1, qPrintable(menu->objectName()));
        QTRY_VERIFY(menu->mapToItem(strip, QPointF()).x() >= -1);
        QTRY_VERIFY(menu->mapToItem(strip, QPointF()).x() + menu->width() <= strip->width() + 1);
    }

    void verify_ribbon_tab(QQuickWindow* window, const char* key, int toolCount) {
        auto* ribbon = window->findChild<QQuickItem*>(QStringLiteral("ribbonPanel"));
        QVERIFY(ribbon != nullptr);
        QTRY_COMPARE(ribbon->property("activeRibbonTab").toString(), QString::fromLatin1(key));
        QTRY_COMPARE(visual_items(ribbon, QStringLiteral("ribbonTileContent")).size(), toolCount);
        auto* strip = window->findChild<QQuickItem*>(QStringLiteral("menuStrip"));
        QVERIFY(strip != nullptr);
        auto* menuContent = strip->property("contentRoot").value<QQuickItem*>();
        QVERIFY(menuContent != nullptr);
        int highlighted = 0;
        for (auto* item : menuContent->childItems()) {
            if (item->objectName().startsWith(QStringLiteral("menu-")) &&
                item->property("highlighted").toBool()) {
                QCOMPARE(item->objectName(), QStringLiteral("menu-") + key);
                ++highlighted;
            }
        }
        QCOMPARE(highlighted, 1);
    }
#endif

  private slots:
    void initTestCase() {
        // 单独运行任一测试用例时，也必须与主入口使用相同的非原生样式。
        QQuickStyle::setStyle(QStringLiteral("Basic"));
    }

    void loads_shell_module() {
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
#ifdef PANTA_ENABLE_BRIDGE_MODULE
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
#else
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("AppNoBridge"));
#endif

        QVERIFY2(!engine.rootObjects().isEmpty(), "Panta.Shell entry failed to load");

        auto* root = engine.rootObjects().constFirst();
        QCOMPARE(root->property("minimumWidth").toInt(), 1280);
        QCOMPARE(root->property("minimumHeight").toInt(), 720);
        const auto assert_visible = [root](const char* object_name) {
            auto* object = root->findChild<QObject*>(QString::fromLatin1(object_name));
            if (object == nullptr) {
                return false;
            }
            return object->property("visible").toBool();
        };
        QVERIFY2(assert_visible("shellCaption"), "Shell caption is missing or hidden");
        auto* settings_router = root->findChild<QObject*>(QStringLiteral("settingsPageRouter"));
        auto* settings_dialog = root->findChild<QObject*>(QStringLiteral("settingsPageDialog"));
        QVERIFY2(settings_router != nullptr, "Settings page router is missing from Panta.Shell");
        QVERIFY2(settings_dialog != nullptr, "Settings page dialog is missing from Panta.Shell");
        QVariant settings_route_available;
        QVERIFY(QMetaObject::invokeMethod(settings_router, "openPage",
                                          Q_RETURN_ARG(QVariant, settings_route_available),
                                          Q_ARG(QVariant, QVariant(QStringLiteral("general")))));
        QVERIFY(settings_route_available.toBool());
        QTRY_VERIFY(settings_dialog->property("visible").toBool());
        QMetaObject::invokeMethod(settings_dialog, "close");
        auto* brand_tile = root->findChild<QObject*>(QStringLiteral("titleBrandTile"));
        auto* application_button =
            root->findChild<QObject*>(QStringLiteral("applicationMenuButton"));
        auto* backstage = root->findChild<QObject*>(QStringLiteral("applicationBackstage"));
        auto* platform_menu_bar =
            root->findChild<QObject*>(QStringLiteral("platformApplicationMenuBar"));
        QVERIFY(brand_tile != nullptr);
        QVERIFY(application_button != nullptr);
        QVERIFY(backstage != nullptr);
        QVERIFY(platform_menu_bar != nullptr);
        QCOMPARE(platform_menu_bar->property("visible").toBool(),
                 !brand_tile->property("visible").toBool());
        QVERIFY(root->findChild<QObject*>(QStringLiteral("applicationSettingsAction")) != nullptr);
        if (brand_tile->property("visible").toBool()) {
            QVERIFY(QMetaObject::invokeMethod(application_button, "click"));
            QTRY_VERIFY(backstage->property("visible").toBool());
            QCOMPARE(backstage->property("width").toInt(), 440);
            QCOMPARE(backstage->property("height").toInt(), 560);
            QVERIFY(root->findChild<QObject*>(QStringLiteral("backstageCloseCommand")) != nullptr);
            auto* backstage_settings =
                root->findChild<QObject*>(QStringLiteral("backstageSettingsButton"));
            QVERIFY(backstage_settings != nullptr);
            QVERIFY(QMetaObject::invokeMethod(backstage_settings, "click"));
            QTRY_VERIFY(!backstage->property("visible").toBool());
            QTRY_VERIFY(settings_dialog->property("visible").toBool());
            QMetaObject::invokeMethod(settings_dialog, "close");
        }
#ifdef PANTA_ENABLE_BRIDGE_MODULE
        QVERIFY2(root->findChild<QObject*>(QStringLiteral("caeViewport")) != nullptr,
                 "CaeViewport is missing from Panta.Shell");
        auto* document_bar = root->findChild<QQuickItem*>(QStringLiteral("documentTabBar"));
        QVERIFY2(document_bar != nullptr, "DocumentTabBar is missing from Panta.Shell");
        const auto documents = document_bar->property("documents").toList();
        QVERIFY2(documents.size() == 1, "startup shell must show exactly the Welcome tab");
        QVERIFY2(document_bar->property("activeDocumentId").toString() == QStringLiteral("welcome"),
                 "Welcome tab must be active at startup");
#endif

#ifdef PANTA_ENABLE_BRIDGE_MODULE
        const auto flags = static_cast<Qt::WindowFlags>(root->property("flags").toUInt());
        auto* chrome = root->findChild<QObject*>(QStringLiteral("titleChromePanel"));
        QVERIFY(chrome != nullptr);
        QVERIFY(flags.testFlag(Qt::ExpandedClientAreaHint));
        QVERIFY(flags.testFlag(Qt::NoTitleBarBackgroundHint));
#if defined(Q_OS_WIN)
        const bool macOS = false;
        QVERIFY(flags.testFlag(Qt::CustomizeWindowHint));
        QVERIFY(flags.testFlag(Qt::WindowMinimizeButtonHint));
        QVERIFY(flags.testFlag(Qt::WindowMaximizeButtonHint));
        QVERIFY(flags.testFlag(Qt::WindowCloseButtonHint));
        QVERIFY(!flags.testFlag(Qt::WindowTitleHint));
#elif defined(Q_OS_LINUX)
        const bool macOS = false;
        QVERIFY(flags.testFlag(Qt::CustomizeWindowHint));
        QVERIFY(flags.testFlag(Qt::WindowTitleHint));
        QVERIFY(flags.testFlag(Qt::WindowMinimizeButtonHint));
        QVERIFY(flags.testFlag(Qt::WindowMaximizeButtonHint));
        QVERIFY(flags.testFlag(Qt::WindowCloseButtonHint));
        QVERIFY(!flags.testFlag(Qt::WindowSystemMenuHint));
#else
        const bool macOS = true;
#endif
        QCOMPARE(root->property("topPadding").toDouble(), 0.0);
        QCOMPARE(root->property("leftPadding").toDouble(), 0.0);
        QCOMPARE(root->property("rightPadding").toDouble(), 0.0);

#if defined(Q_OS_LINUX)
        QCOMPARE(root->property("title").toString(), QString(QChar(0x200B)));
#else
        QVERIFY(root->property("title").toString().isEmpty());
#endif
        auto* brandTile = root->findChild<QObject*>(QStringLiteral("titleBrandTile"));
        auto* macBrandMark = root->findChild<QObject*>(QStringLiteral("titleBrandMarkMac"));
        QVERIFY(brandTile != nullptr);
        QVERIFY(macBrandMark == nullptr);
        QCOMPARE(brandTile->property("visible").toBool(), !macOS);

        // 标题组不能被 Layout 压到内容宽度以下；窄窗口 Tab 到搜索时应自动滚入视口。
        auto* shellWindow = qobject_cast<QQuickWindow*>(root);
        QVERIFY(shellWindow != nullptr);
        auto* strip = root->findChild<QQuickItem*>(QStringLiteral("titleStrip"));
        auto* search = root->findChild<QQuickItem*>(QStringLiteral("globalSearch"));
        QVERIFY(strip != nullptr);
        QVERIFY(search != nullptr);
        chrome->setProperty("nativeControlsLeftInset", 48.0);
        const qreal minimumInset = 48.0;
        QTRY_VERIFY(strip->x() >= minimumInset);
        QTRY_VERIFY(strip->x() + strip->width() <=
                    chrome->property("width").toReal() -
                        chrome->property("nativeControlsRightInset").toReal());
        strip->setProperty("contentX", strip->property("contentWidth").toReal());
        QTRY_VERIFY(strip->x() >= minimumInset);
        if (macOS) {
            chrome->setProperty("nativeControlsLeftInset", 120.0);
            QTRY_VERIFY(strip->x() >= 120.0);
        }
        shellWindow->showNormal();
        for (const int width : {1440, 1280}) {
            shellWindow->resize(width, 900);
            QTest::qWait(50);
            for (const char* name : {"titleQuickActions", "titleSearchGroup"}) {
                auto* group = root->findChild<QQuickItem*>(QString::fromLatin1(name));
                QVERIFY(group != nullptr);
                auto* content = group->findChild<QQuickItem*>(QStringLiteral("toolGroupContent"));
                QVERIFY(content != nullptr);
                QVERIFY2(content->x() >= 0, name);
                QVERIFY2(content->x() + content->width() <= group->width(), name);
            }
            if (width == 1440) {
                const auto bars = root->findChildren<QQuickItem*>(QStringLiteral("panelTabs"));
                QVERIFY(!bars.isEmpty());
                for (auto* bar : bars) {
                    const int count = bar->property("count").toInt();
                    QVERIFY(count > 0);
                    qreal segmentWidth = 0;
                    for (int index = 0; index < count; ++index) {
                        QQuickItem* tab = nullptr;
                        QVERIFY(QMetaObject::invokeMethod(
                            bar, "itemAt", Q_RETURN_ARG(QQuickItem*, tab), Q_ARG(int, index)));
                        QVERIFY(tab != nullptr);
                        if (index == 0)
                            segmentWidth = tab->width();
                        QCOMPARE(tab->width(), segmentWidth);
                        bar->setProperty("currentIndex", index);
                        QTest::qWait(10);
                        QCOMPARE(tab->width(), segmentWidth);
                        auto* background = tab->property("background").value<QQuickItem*>();
                        QVERIFY(background != nullptr);
                        QVERIFY(!background->childItems().isEmpty());
                        const auto* hover = background->childItems().constFirst();
                        QCOMPARE(hover->width(), segmentWidth - 4);
                        QCOMPARE(hover->height(), tab->height() - 4);
                        QCOMPARE(tab->height(), bar->property("availableHeight").toDouble());
                        const QPointF origin = tab->mapToItem(bar, QPointF());
                        QVERIFY(origin.y() >= 0);
                        QVERIFY(origin.y() + tab->height() <= bar->height());
                    }
                    bar->setProperty("currentIndex", 0);
                }
                const auto buttonContents =
                    visual_items(shellWindow->contentItem(), QStringLiteral("toolButtonContent"));
                QVERIFY(!buttonContents.isEmpty());
                for (const auto* content : buttonContents) {
                    const auto* button = content->parentItem()->parentItem();
                    const QPointF center = content->mapToItem(
                        button, QPointF(content->width() / 2, content->height() / 2));
                    QVERIFY(qAbs(center.y() - button->height() / 2) <= 0.5);
                    if (!button->property("contentAlignLeft").toBool()) {
                        QVERIFY2(qAbs(center.x() - button->width() / 2) <= 0.5,
                                 qPrintable(QStringLiteral("Button %1 (%2), width %3, center %4")
                                                .arg(button->objectName(),
                                                     button->property("text").toString())
                                                .arg(button->width())
                                                .arg(center.x())));
                    }
                }
                verify_ribbon_alignment(
                    root->findChild<QQuickItem*>(QStringLiteral("ribbonPanel")));
                for (const char* name :
                     {"ribbon-new-project", "ribbon-open-project", "ribbon-new-features",
                      "ribbon-start-here", "ribbon-tutorials", "ribbon-videos", "ribbon-help"}) {
                    auto* tile = visual_item(shellWindow->contentItem(), QString::fromLatin1(name));
                    QVERIFY(tile != nullptr);
                    QVERIFY(tile->width() >= 56.0);
                    QCOMPARE(tile->height(), 68.0);
                    const auto* group = tile->parentItem()->parentItem();
                    const qreal topGap = tile->mapToItem(group, QPointF()).y();
                    const qreal bottomGap = group->height() - 22 - topGap - tile->height();
                    QCOMPARE(topGap, bottomGap);
                    QVERIFY(topGap >= 0);
                    auto* content =
                        tile->findChild<QQuickItem*>(QStringLiteral("ribbonTileContent"));
                    QVERIFY(content != nullptr);
                    const QPointF center = content->mapToItem(
                        tile, QPointF(content->width() / 2, content->height() / 2));
                    QVERIFY(qAbs(center.x() - tile->width() / 2) <= 0.5);
                    QVERIFY(qAbs(center.y() - tile->height() / 2) <= 0.5);
                }
            }
            shellWindow->contentItem()->forceActiveFocus();
            search->forceActiveFocus();
            QTest::qWait(20);
            const QPointF position = search->mapToItem(strip, QPointF());
            QVERIFY(position.x() >= -1);
            QVERIFY2(
                position.x() + search->width() <= strip->width() + 1,
                qPrintable(QStringLiteral("window=%1 search x=%2 w=%3 strip=%4 scroll=%5 focus=%6")
                               .arg(width)
                               .arg(position.x())
                               .arg(search->width())
                               .arg(strip->width())
                               .arg(strip->property("contentX").toDouble())
                               .arg(shellWindow->activeFocusItem() == search)));
        }
        // 文案增长不应依赖窗口 resize 才扩大滚动范围或重新显露搜索焦点。
        auto* quickActions = root->findChild<QQuickItem*>(QStringLiteral("titleQuickActions"));
        auto* searchGroup = root->findChild<QQuickItem*>(QStringLiteral("titleSearchGroup"));
        QVERIFY(quickActions != nullptr);
        QVERIFY(searchGroup != nullptr);
        QQuickItem* animationButton = nullptr;
        for (auto* item : quickActions->findChildren<QQuickItem*>()) {
            if (item->property("text").toString() == QStringLiteral("Activate Animation (A)") &&
                item->property("contentPadding").isValid()) {
                animationButton = item;
                break;
            }
        }
        QVERIFY(animationButton != nullptr);
        const QString originalText = animationButton->property("text").toString();
        animationButton->setProperty("text", QString(160, QChar('W')));
        QTest::qWait(50);
        QVERIFY(quickActions->width() >= quickActions->implicitWidth());
        QVERIFY(searchGroup->mapToItem(quickActions->parentItem(), QPointF()).x() >=
                quickActions->x() + quickActions->width());
        const auto searchVisible = [&] {
            const QPointF position = search->mapToItem(strip, QPointF());
            return position.x() >= -1 && position.x() + search->width() <= strip->width() + 1;
        };
        // 焦点回滚要等 polish 与两轮 callLater 收敛；按最终几何断言，字段
        // 与上方窗口循环的诊断一致，避免 CI 失败时无数值可查。
        QTRY_VERIFY2(searchVisible(),
                     qPrintable(QStringLiteral("search x=%1 w=%2 strip=%3 scroll=%4")
                                    .arg(search->mapToItem(strip, QPointF()).x())
                                    .arg(search->width())
                                    .arg(strip->width())
                                    .arg(strip->property("contentX").toDouble()) +
                                QStringLiteral(" active=%1 focus=%2 contentWidth=%3")
                                    .arg(shellWindow->activeFocusItem()
                                             ? shellWindow->activeFocusItem()->objectName()
                                             : QStringLiteral("<null>"))
                                    .arg(shellWindow->activeFocusItem() == search)
                                    .arg(strip->property("contentWidth").toDouble())));
        animationButton->setProperty("text", originalText);
        QTest::qWait(50);

        // 从搜索字段沿真实 Shell Tab 顺序遍历，标签页必须能被键盘抵达。
        auto* documentBar = root->findChild<QQuickItem*>(QStringLiteral("documentTabBar"));
        QQuickItem* welcomeTab = nullptr;
        QVERIFY(documentBar != nullptr);
        documentBar->ensurePolished();
        QTRY_VERIFY(visual_item(documentBar, QStringLiteral("documentTab-welcome")) != nullptr);
        welcomeTab = visual_item(documentBar, QStringLiteral("documentTab-welcome"));
        shellWindow->resize(1440, 900);
        QTest::qWait(50);
        search->forceActiveFocus(Qt::TabFocusReason);
        QTRY_COMPARE(shellWindow->activeFocusItem(), search);
        QStringList focusTrail;
        QList<QQuickItem*> visited;
        bool reachedDocumentTab = false;
        for (int step = 0; step < 128; ++step) {
            auto* focused = shellWindow->activeFocusItem();
            if (focused == welcomeTab) {
                reachedDocumentTab = true;
                break;
            }
            if (focused == nullptr || visited.contains(focused)) {
                break;
            }
            visited.append(focused);
            focusTrail.append(focused->objectName().isEmpty()
                                  ? QString::fromLatin1(focused->metaObject()->className())
                                  : focused->objectName());
            QTest::keyClick(shellWindow, Qt::Key_Tab);
            QCoreApplication::processEvents();
        }
        QVERIFY2(reachedDocumentTab,
                 qPrintable(QStringLiteral("Tab focus path did not reach the document tab: %1")
                                .arg(focusTrail.join(QStringLiteral(" -> ")))));

        auto* searchButton = root->findChild<QQuickItem*>(QStringLiteral("searchButton"));
        QVERIFY(searchButton != nullptr);
        searchButton->forceActiveFocus();
        QTest::qWait(20);
        const QPoint searchCenter =
            searchButton->mapToScene(QPointF(searchButton->width() / 2, searchButton->height() / 2))
                .toPoint();
        QTest::mouseMove(shellWindow, searchCenter);
        QTRY_VERIFY2_WITH_TIMEOUT(
            searchButton->property("hovered").toBool(),
            qPrintable(QStringLiteral("search center=%1,%2 hoverEnabled=%3")
                           .arg(searchCenter.x())
                           .arg(searchCenter.y())
                           .arg(searchButton->property("hoverEnabled").toBool())),
            1000);
        QTest::mouseMove(shellWindow, QPoint(400, 200));
        shellWindow->contentItem()->forceActiveFocus();
        strip->setProperty("contentX", 0);

        // 布局取证与几何诊断（029，常驻）：设置环境变量时保存首帧 PNG 或
        // 打印几何树，供真实布局对照与后续优化；ctest 默认不设置，无副作用。
        const QString capturePath = qEnvironmentVariable("PANTA_SHELL_CAPTURE_PATH");
        const bool dumpGeometry = qEnvironmentVariableIsSet("PANTA_SHELL_DUMP_GEOMETRY");
        if (!capturePath.isEmpty() || dumpGeometry) {
            if (!capturePath.isEmpty()) {
                const int captureWidth = qEnvironmentVariableIntValue("PANTA_SHELL_CAPTURE_WIDTH");
                shellWindow->showNormal();
                shellWindow->resize(captureWidth > 0 ? captureWidth : 1440, 900);
                QTest::qWait(100);
            }
            if (dumpGeometry) {
                qWarning() << "shell window" << shellWindow->width() << shellWindow->height();
                dump_item_tree(shellWindow->contentItem(), 0);
            }
            if (!capturePath.isEmpty()) {
                const QImage frame = shellWindow->grabWindow();
                QVERIFY2(!frame.isNull(), "grabWindow returned an empty frame");
                QDir().mkpath(QFileInfo(capturePath).absolutePath());
                QVERIFY2(frame.save(capturePath), "Failed to save captured frame");
            }
        }
#endif
    }

    void edit_commands_follow_focus_in_separate_window() {
        QQmlApplicationEngine engine;
        QQmlComponent component(&engine);
        component.setData(QByteArrayLiteral("import QtQuick\n"
                                            "import QtQuick.Controls\n"
                                            "import Panta.Shell\n"
                                            "ApplicationWindow {\n"
                                            "  id: mainWindow\n"
                                            "  width: 320; height: 200; visible: true\n"
                                            "  TextField { objectName: 'editField'; text: 'Panta'; "
                                            "anchors.top: parent.top; focus: true }\n"
                                            "  Window {\n"
                                            "    id: dialogWindow\n"
                                            "    objectName: 'editDialogWindow'\n"
                                            "    transientParent: mainWindow\n"
                                            "    width: 240; height: 160; visible: false\n"
                                            "    TextArea { objectName: 'richEdit'; textFormat: "
                                            "TextEdit.RichText; anchors.fill: parent }\n"
                                            "  }\n"
                                            "}\n"),
                          QUrl(QStringLiteral("qrc:/EditCommandRouterTest.qml")));
        std::unique_ptr<QObject> root(component.create());
        QVERIFY2(root != nullptr, qPrintable(component.errorString()));
        auto* window = qobject_cast<QQuickWindow*>(root.get());
        QVERIFY(window != nullptr);
        auto* field = root->findChild<QObject*>(QStringLiteral("editField"));
        QVERIFY(field != nullptr);
        auto* dialog_window = root->findChild<QQuickWindow*>(QStringLiteral("editDialogWindow"));
        QVERIFY(dialog_window != nullptr);
        auto* router = engine.singletonInstance<panta::app::EditCommandRouter*>(
            QStringLiteral("Panta.Shell"), QStringLiteral("EditCommandRouter"));
        QVERIFY(router != nullptr);

        window->requestActivate();
        QVERIFY(QMetaObject::invokeMethod(field, "forceActiveFocus"));
        QTRY_COMPARE(QGuiApplication::focusWindow(), window);
        QTRY_VERIFY(router->canSelectAll());
        QVERIFY(QMetaObject::invokeMethod(router, "selectAll"));
        QTRY_VERIFY(router->canCopy());
        QVERIFY(QMetaObject::invokeMethod(router, "copy"));
        QCOMPARE(QGuiApplication::clipboard()->text(), QStringLiteral("Panta"));
        QTRY_VERIFY(router->canCut());
        QVERIFY(QMetaObject::invokeMethod(router, "cut"));
        QCOMPARE(field->property("text").toString(), QString{});

        QGuiApplication::clipboard()->setText(QStringLiteral("Paste from system clipboard"));
        QTRY_VERIFY(router->canPaste());
        QVERIFY(QMetaObject::invokeMethod(router, "paste"));
        QCOMPARE(field->property("text").toString(), QStringLiteral("Paste from system clipboard"));

        auto* rich_edit = root->findChild<QObject*>(QStringLiteral("richEdit"));
        QVERIFY(rich_edit != nullptr);
        dialog_window->show();
        dialog_window->requestActivate();
        QVERIFY(QMetaObject::invokeMethod(rich_edit, "forceActiveFocus"));
        QTRY_COMPARE(QGuiApplication::focusWindow(), dialog_window);
        auto* rich_text = new QMimeData;
        rich_text->setHtml(QStringLiteral("<b>Rich text</b>"));
        QGuiApplication::clipboard()->setMimeData(rich_text);
        QTRY_VERIFY(router->canPaste());
        QVERIFY(QMetaObject::invokeMethod(router, "paste"));
        QVERIFY(rich_edit->property("text").toString().contains(QStringLiteral("Rich text")));
        QVERIFY(router->canUndo());
        rich_edit->setProperty("readOnly", true);
        QTRY_VERIFY(!router->canUndo());
        QVERIFY(QMetaObject::invokeMethod(router, "undo"));
        QVERIFY(rich_edit->property("text").toString().contains(QStringLiteral("Rich text")));

        rich_edit->setProperty("readOnly", false);
        QVERIFY(QMetaObject::invokeMethod(router, "undo"));
        QVERIFY(router->canRedo());
        rich_edit->setProperty("readOnly", true);
        QTRY_VERIFY(!router->canRedo());
        const QString text_before_read_only_redo = rich_edit->property("text").toString();
        QVERIFY(QMetaObject::invokeMethod(router, "redo"));
        QCOMPARE(rich_edit->property("text").toString(), text_before_read_only_redo);
        window->close();
    }

    void settings_pages_register_and_dialog_actions_work() {
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        QQmlComponent component(&engine);
        component.setData(
            QByteArrayLiteral(
                "import QtQuick\n"
                "import QtQuick.Controls\n"
                "import Panta.Shell\n"
                "ApplicationWindow {\n"
                "  id: hostWindow\n"
                "  width: 640; height: 480; visible: true\n"
                "  SettingsPageRouter { id: router; objectName: 'settingsPageRouter'; ownerWindow: "
                "hostWindow }\n"
                "  Component { id: customPage; Item { objectName: 'customSettingsPage' } }\n"
                "  function registerCustomPage() { return router.registerPage('custom', 'Custom', "
                "'history', customPage) }\n"
                "  function replaceGeneralPage() { return router.registerPage('general', 'General "
                "custom', "
                "'', customPage) }\n"
                "  function registerPageWithoutIcon() { return router.registerPage('missing-icon', "
                "'Missing icon', '', customPage) }\n"
                "  function openCustomPage() { return router.openPage('custom') }\n"
                "}\n"),
            QUrl(QStringLiteral("qrc:/SettingsPageRouterTest.qml")));
        std::unique_ptr<QObject> root(component.create());
        QVERIFY2(root != nullptr, qPrintable(component.errorString()));

        QVariant result;
        QVERIFY(QMetaObject::invokeMethod(root.get(), "registerCustomPage",
                                          Q_RETURN_ARG(QVariant, result)));
        QVERIFY(result.toBool());
        auto* router = root->findChild<QObject*>(QStringLiteral("settingsPageRouter"));
        auto* dialog = root->findChild<QQuickWindow*>(QStringLiteral("settingsPageDialog"));
        QVERIFY(router != nullptr);
        QVERIFY(dialog != nullptr);

        QVERIFY(QMetaObject::invokeMethod(root.get(), "replaceGeneralPage",
                                          Q_RETURN_ARG(QVariant, result)));
        QVERIFY(result.toBool());
        QVERIFY(QMetaObject::invokeMethod(router, "pageForRoute", Q_RETURN_ARG(QVariant, result),
                                          Q_ARG(QVariant, QVariant(QStringLiteral("general")))));
        const QVariantMap replaced_general = result.toMap();
        QCOMPARE(replaced_general.value(QStringLiteral("iconName")).toString(),
                 QStringLiteral("tools-options"));
        QVERIFY(replaced_general.value(QStringLiteral("preserveSourceColors")).toBool());

        QVERIFY(QMetaObject::invokeMethod(root.get(), "registerPageWithoutIcon",
                                          Q_RETURN_ARG(QVariant, result)));
        QVERIFY(result.toBool());
        QVERIFY(
            QMetaObject::invokeMethod(router, "pageForRoute", Q_RETURN_ARG(QVariant, result),
                                      Q_ARG(QVariant, QVariant(QStringLiteral("missing-icon")))));
        const QVariantMap fallback_page = result.toMap();
        QCOMPARE(fallback_page.value(QStringLiteral("iconName")).toString(),
                 QStringLiteral("history"));
        QVERIFY(!fallback_page.value(QStringLiteral("preserveSourceColors")).toBool());

        QSignalSpy accepted_spy(router, SIGNAL(accepted()));
        QSignalSpy cancelled_spy(router, SIGNAL(cancelled()));

        QVERIFY(QMetaObject::invokeMethod(root.get(), "openCustomPage",
                                          Q_RETURN_ARG(QVariant, result)));
        QVERIFY(result.toBool());
        QTRY_VERIFY(dialog->isVisible());
        QCOMPARE(router->property("activeRouteId").toString(), QStringLiteral("custom"));
        QTRY_VERIFY(dialog->activeFocusItem() != nullptr);
        auto* loader = dialog->findChild<QObject*>(QStringLiteral("settingsPageLoader"));
        QVERIFY(loader != nullptr);
        QTRY_VERIFY(loader->property("item").value<QObject*>() != nullptr);
        QCOMPARE(loader->property("item").value<QObject*>()->objectName(),
                 QStringLiteral("customSettingsPage"));

        QVERIFY(QMetaObject::invokeMethod(dialog, "routeSelected",
                                          Q_ARG(QString, QStringLiteral("general"))));
        QTRY_COMPARE(router->property("activeRouteId").toString(), QStringLiteral("general"));
        QVERIFY(QMetaObject::invokeMethod(dialog, "routeSelected",
                                          Q_ARG(QString, QStringLiteral("custom"))));
        QTRY_COMPARE(router->property("activeRouteId").toString(), QStringLiteral("custom"));

        auto* ok_button = dialog->findChild<QObject*>(QStringLiteral("settingsOkButton"));
        QVERIFY(ok_button != nullptr);
        QVERIFY(QMetaObject::invokeMethod(ok_button, "click"));
        QCOMPARE(accepted_spy.count(), 1);
        QTRY_VERIFY(!dialog->isVisible());

        QVERIFY(QMetaObject::invokeMethod(root.get(), "openCustomPage",
                                          Q_RETURN_ARG(QVariant, result)));
        QVERIFY(result.toBool());
        QTRY_VERIFY(dialog->isVisible());
        QTRY_COMPARE(QGuiApplication::focusWindow(), dialog);
        QTest::keyClick(dialog, Qt::Key_Escape);
        QTRY_COMPARE(cancelled_spy.count(), 1);
        QTRY_VERIFY(!dialog->isVisible());
    }

#ifdef PANTA_ENABLE_BRIDGE_MODULE
    void ribbon_navigation_preserves_workspace_data() {
        QTest::addColumn<int>("windowWidth");
        QTest::addColumn<bool>("keyboard");
        QTest::newRow("wide-mouse") << 1440 << false;
        QTest::newRow("minimum-mouse") << 1280 << false;
        QTest::newRow("wide-keyboard") << 1440 << true;
        QTest::newRow("minimum-keyboard") << 1280 << true;
    }

    void ribbon_navigation_preserves_workspace() {
        QFETCH(int, windowWidth);
        QFETCH(bool, keyboard);
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
        QVERIFY(window != nullptr);
        auto* project =
            window->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* ribbon = window->findChild<QQuickItem*>(QStringLiteral("ribbonPanel"));
        QPointer<QQuickItem> projectItem =
            window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem"));
        QPointer<QQuickItem> viewport =
            window->findChild<QQuickItem*>(QStringLiteral("caeViewport"));
        QVERIFY(project && ribbon && projectItem && viewport);
        window->showNormal();
        window->resize(windowWidth, 900);
        QTest::qWait(50);
        verify_ribbon_tab(window, "start-learn", 7);
        QVERIFY(visual_item(window->contentItem(), QStringLiteral("menu-home")) == nullptr);
        QVERIFY(QMetaObject::invokeMethod(window, "selectRibbonTab",
                                          Q_ARG(QVariant, QStringLiteral("home"))));
        verify_ribbon_tab(window, "start-learn", 7);
        activate_menu(window, "start-learn", keyboard);
        activate_menu(window, "tools", keyboard);
        verify_ribbon_tab(window, "start-learn", 7);
        QVERIFY(project->currentPath().isEmpty());

        QVERIFY(project->createProject(QStringLiteral("Navigation"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        verify_ribbon_tab(window, "home", 18);
        activate_menu(window, "tools", keyboard);
        verify_ribbon_tab(window, "tools", 11);
        auto* databaseSearch = visual_item(ribbon, QStringLiteral("ribbon-database-search"));
        auto* macroSlot = visual_item(ribbon, QStringLiteral("ribbon-macro-10"));
        QVERIFY(databaseSearch && macroSlot);
        QVERIFY(!databaseSearch->isEnabled());
        QVERIFY(!macroSlot->isEnabled());
        QVERIFY(projectItem && viewport);
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem")),
                 projectItem.data());
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("caeViewport")), viewport.data());
        activate_menu(window, "results", keyboard);
        verify_ribbon_tab(window, "results", 8);
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem")),
                 projectItem.data());
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("caeViewport")), viewport.data());
        activate_menu(window, "view", keyboard);
        verify_ribbon_tab(window, "view", 7);
        auto* solidView = visual_item(ribbon, QStringLiteral("ribbon-view-solid"));
        auto* newViewWindow = visual_item(ribbon, QStringLiteral("ribbon-view-window-new"));
        auto* unitsCombo = visual_item(ribbon, QStringLiteral("ribbon-units-combo"));
        auto* viewNameCombo = visual_item(ribbon, QStringLiteral("ribbon-view-name-combo"));
        QVERIFY(solidView && newViewWindow && unitsCombo && viewNameCombo);
        QVERIFY(!solidView->isEnabled());
        QVERIFY(!newViewWindow->isEnabled());
        QVERIFY(!unitsCombo->isEnabled());
        QVERIFY(!viewNameCombo->isEnabled());
        QCOMPARE(unitsCombo->height(), viewNameCombo->height());
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem")),
                 projectItem.data());
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("caeViewport")), viewport.data());
        activate_menu(window, "tools", keyboard);
        verify_ribbon_tab(window, "tools", 11);
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem")),
                 projectItem.data());
        QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("caeViewport")), viewport.data());
        activate_menu(window, "home", keyboard);
        verify_ribbon_tab(window, "home", 18);
        QVERIFY(project->renameProject(QStringLiteral("Unsaved navigation project")));
        const QString path = project->currentPath();
        const QString name = project->currentName();
        QVERIFY(project->dirty());
        QSignalSpy projectChanged(project, &panta::bridge::ProjectViewModel::projectChanged);
        QVERIFY(projectChanged.isValid());

        // 导航只替换 Ribbon 页签，不修改快照或重建工作区，即使工程有未保存变更。
        for (int round = 0; round < 2; ++round) {
            for (const char* tab : {"start-learn", "home"}) {
                activate_menu(window, tab, keyboard);
                verify_ribbon_tab(window, tab,
                                  QString::fromLatin1(tab) == QStringLiteral("home") ? 18 : 7);
                activate_menu(window, tab, keyboard);
                QCOMPARE(project->currentPath(), path);
                QCOMPARE(project->currentName(), name);
                QVERIFY(project->dirty());
                QVERIFY(projectItem && viewport);
                QVERIFY(projectItem->isVisible());
                QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("projectTaskItem")),
                         projectItem.data());
                QCOMPARE(window->findChild<QQuickItem*>(QStringLiteral("caeViewport")),
                         viewport.data());
            }
        }
        QCOMPARE(projectChanged.count(), 0);
        activate_menu(window, "start-learn", keyboard);
        activate_menu(window, "community", keyboard);
        verify_ribbon_tab(window, "start-learn", 7);
        QVERIFY(project->openProject(fixture.filePath(QStringLiteral("missing.panta"))));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(!project->errorCode().isEmpty());
        QVERIFY(project->createProject(QStringLiteral("Navigation"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QCOMPARE(project->errorCode(), QStringLiteral("project.already_exists"));
        verify_ribbon_tab(window, "start-learn", 7);
        QCOMPARE(project->currentPath(), path);
        QCOMPARE(project->currentName(), name);
        QVERIFY(project->dirty());
        QVERIFY(project->renameProject(QStringLiteral("Renamed on start page")));
        verify_ribbon_tab(window, "start-learn", 7);
        QVERIFY(project->saveProject());
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        verify_ribbon_tab(window, "start-learn", 7);

        const QString captureDir = qEnvironmentVariable("PANTA_PROJECT_CAPTURE_DIR");
        if (!captureDir.isEmpty() && windowWidth == 1440 && !keyboard) {
            window->contentItem()->forceActiveFocus();
            auto* menuStrip = window->findChild<QQuickItem*>(QStringLiteral("menuStrip"));
            QVERIFY(menuStrip != nullptr);
            menuStrip->setProperty("contentX", 0);
            QTest::qWait(100);
            verify_ribbon_alignment(ribbon);
            const QImage frame = window->grabWindow();
            QVERIFY(!frame.isNull());
            QDir().mkpath(captureDir);
            QVERIFY(frame.save(QDir(captureDir).filePath(QStringLiteral("start-learn.png"))));
        }

        // 切回开始页的新建按钮仍连接对话框；取消不改变工程或页签。
        auto* newButton = visual_item(ribbon, QStringLiteral("ribbon-new-project"));
        auto* dialog = window->findChild<QQuickWindow*>(QStringLiteral("newProjectDialog"));
        QVERIFY(newButton && dialog);
        QVERIFY(QMetaObject::invokeMethod(newButton, "clicked"));
        QTRY_VERIFY(dialog->isVisible());
        QTRY_COMPARE(QGuiApplication::focusWindow(), dialog);
        dialog->close();
        window->requestActivate();
        QTRY_COMPARE(QGuiApplication::focusWindow(), window);
        verify_ribbon_tab(window, "start-learn", 7);
        QCOMPARE(project->currentPath(), path);

        // 同一路径重开也应选中 Home，不能只依赖 currentPath 的值变化。
        QVERIFY(project->openProject(path));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        verify_ribbon_tab(window, "home", 18);
        activate_menu(window, "start-learn", keyboard);
        QVERIFY(project->createProject(QStringLiteral("Second"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        verify_ribbon_tab(window, "home", 18);
    }

    void analysis_sequence_submission_keeps_window_until_completion() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
        QVERIFY(window);
        auto* project =
            window->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* dialog = window->findChild<QQuickWindow*>(QStringLiteral("analysisSequenceDialog"));
        QVERIFY(project && dialog);
        QVERIFY(project->createProject(QStringLiteral("Sequence"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        const QString sourcePath = fixture.filePath(QStringLiteral("part.stl"));
        QFile source(sourcePath);
        QVERIFY(source.open(QIODevice::WriteOnly));
        const QByteArray stl("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n");
        QCOMPARE(source.write(stl), stl.size());
        source.close();
        QVERIFY(project->importStl(sourcePath, QStringLiteral("dual-domain"),
                                   QStringLiteral("millimeters"), false));
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QVERIFY(dialog->isVisible());
        auto* panel = dialog->findChild<QObject*>(QStringLiteral("analysisSequencePanel"));
        auto* accept = dialog->findChild<QObject*>(QStringLiteral("analysisSequenceAccept"));
        auto* cancel = dialog->findChild<QObject*>(QStringLiteral("analysisSequenceCancel"));
        QVERIFY(panel && accept && cancel);
        panel->setProperty("selectedIndex", 1);
        const auto initial = project->planSettings();
        QSignalSpy requested(dialog, SIGNAL(selectionRequested(QString, double, QString, QString)));
        QSignalSpy finished(project, &panta::bridge::ProjectViewModel::projectOperationFinished);
        QVERIFY(requested.isValid() && finished.isValid());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QVERIFY(project->projectOperationPending());
        QVERIFY(dialog->property("saving").toBool());
        QVERIFY(dialog->isVisible());
        QVERIFY(!accept->property("enabled").toBool());
        QVERIFY(!cancel->property("enabled").toBool());
        QVERIFY(!dialog->close());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QCOMPARE(requested.count(), 1);
        QCOMPARE(project->planSettings(), initial);
        QTRY_COMPARE(finished.count(), 1);
        QCOMPARE(finished.constFirst().at(0).toString(), QStringLiteral("analysis-sequence"));
        QVERIFY(finished.constFirst().at(1).toBool());
        QTRY_VERIFY(!dialog->isVisible());
        QCOMPARE(project->planSettings().value(QStringLiteral("sequenceId")).toString(),
                 QStringLiteral("fill-pack"));

        // 弹窗捕获的修订过期后保留候选，不能把旧目标提交为当前方案。
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        panel->setProperty("selectedIndex", 0);
        QVERIFY(project->renameProject(QStringLiteral("New revision")));
        const auto revised = project->planSettings();
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(!project->errorCode().isEmpty());
        QVERIFY(dialog->isVisible());
        QVERIFY(!dialog->property("saving").toBool());
        QCOMPARE(panel->property("selectedIndex").toInt(), 0);
        QCOMPARE(project->planSettings(), revised);
        QVERIFY(dialog->close());
        QVERIFY(window->close());
    }

    void material_submission_keeps_windows_until_completion() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
        QVERIFY(window);
        auto* project =
            window->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* dialog = window->findChild<QQuickWindow*>(QStringLiteral("materialSelectionDialog"));
        QVERIFY(project && dialog);
        QVERIFY(project->createProject(QStringLiteral("Material"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        const QString sourcePath = fixture.filePath(QStringLiteral("part.stl"));
        QFile source(sourcePath);
        QVERIFY(source.open(QIODevice::WriteOnly));
        const QByteArray stl("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n");
        QCOMPARE(source.write(stl), stl.size());
        source.close();
        QVERIFY(project->importStl(sourcePath, QStringLiteral("dual-domain"),
                                   QStringLiteral("millimeters"), false));
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QVERIFY(dialog->property("visible").toBool());
        const auto initial = project->planSettings();
        QSignalSpy finished(project,
                            &panta::bridge::ProjectViewModel::materialConfirmationFinished);
        QVERIFY(finished.isValid());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        // 不处理事件循环，确保尚未被 Qt 定时器消费真实后台结果。
        QVERIFY(project->materialConfirmationPending());
        QVERIFY(dialog->property("saving").toBool());
        QVERIFY(!dialog->property("canConfirm").toBool());
        auto* cancel = dialog->findChild<QObject*>(QStringLiteral("materialCancel"));
        auto* close = dialog->findChild<QObject*>(QStringLiteral("dialogCloseButton"));
        QVERIFY(cancel && close);
        QVERIFY(!cancel->property("enabled").toBool());
        QVERIFY(!close->property("enabled").toBool());
        QVERIFY(!dialog->close());
        QVERIFY(!window->close());
        QVERIFY(dialog->isVisible() && window->isVisible());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QCOMPARE(project->planSettings(), initial);
        QTRY_COMPARE(finished.count(), 1);
        QVERIFY(finished.constFirst().constFirst().toBool());
        QVERIFY(!project->materialConfirmationPending());
        QTRY_VERIFY(!dialog->isVisible());
        QCOMPARE(project->planSettings().value(QStringLiteral("materialId")).toString(),
                 project->defaultMaterial().value(QStringLiteral("id")).toString());
        QVERIFY(window->close());
    }

    void project_file_dialog_routes_final_selection_data() {
        QTest::addColumn<QString>("storedName");
        QTest::newRow("plain") << QStringLiteral("Stored");
        QTest::newRow("encoded") << QStringLiteral("已保存 项目 #%");
    }

    void project_file_dialog_routes_final_selection() {
        QFETCH(QString, storedName);
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        panta::bridge::ProjectViewModel stored;
        QVERIFY(stored.createProject(storedName, fixture.path()));
        QTRY_VERIFY(!stored.projectOperationPending());
        QVERIFY(stored.errorCode().isEmpty());
        const QUrl selectedUrl = QUrl::fromLocalFile(stored.currentPath());
        // Qt 解码本地 URL 后使用 /；Windows 新建工程快照可含原生反斜杠。
        const QString selectedPath = selectedUrl.toLocalFile();

        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* root = engine.rootObjects().constFirst();
        auto* project =
            root->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* dialog = root->findChild<QObject*>(QStringLiteral("openProjectFileDialog"));
        auto* ribbon = root->findChild<QQuickItem*>(QStringLiteral("ribbonPanel"));
        QVERIFY(project && dialog && ribbon);

        const auto* meta = dialog->metaObject();
        const int modeIndex = meta->indexOfProperty("fileMode");
        QVERIFY(modeIndex >= 0);
        const int openFile = meta->property(modeIndex).enumerator().keyToValue("OpenFile");
        QVERIFY(openFile >= 0);
        QCOMPARE(dialog->property("fileMode").toInt(), openFile);
        QCOMPARE(dialog->property("options").toInt(), 0);
        QCOMPARE(dialog->property("currentFolder").toUrl(), project->defaultLocationUrl());
        const QStringList filters = dialog->property("nameFilters").toStringList();
        QCOMPARE(filters.size(), 1);
        QVERIFY(filters.constFirst().endsWith(QStringLiteral("(*.panta)")));

        QVERIFY(project->createProject(QStringLiteral("Current"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        QVERIFY(project->renameProject(QStringLiteral("Unsaved current")));
        const QString previousPath = project->currentPath();
        const QVariantMap previousSettings = project->planSettings();
        const QVariantList previousDocuments = project->openDocuments();
        QVERIFY(root->setProperty("activeRibbonTab", QStringLiteral("start-learn")));
        QTRY_COMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("start-learn"));
        QSignalSpy opened(project, &panta::bridge::ProjectViewModel::projectOpened);
        QVERIFY(opened.isValid());

        // 只驱动公开 QML 属性及信号，验证接线；不冒充 Cocoa 原生选择器验收。
        QVERIFY(dialog->setProperty("selectedFile", selectedUrl));
        QVERIFY(QMetaObject::invokeMethod(dialog, "rejected"));
        QCOMPARE(opened.count(), 0);
        QCOMPARE(project->currentPath(), previousPath);
        QCOMPARE(project->currentName(), QStringLiteral("Unsaved current"));
        QVERIFY(project->dirty());
        QCOMPARE(project->planSettings(), previousSettings);
        QCOMPARE(project->openDocuments(), previousDocuments);
        QCOMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("start-learn"));

        for (const bool disappears : {true, false}) {
            const QString path = fixture.filePath(disappears ? QStringLiteral("disappearing.panta")
                                                             : QStringLiteral("malformed.panta"));
            QFile file(path);
            QVERIFY(file.open(QIODevice::WriteOnly));
            QCOMPARE(file.write("{}"), 2);
            file.close();
            const QUrl invalid = QUrl::fromLocalFile(path);
            // FileDialog 拒绝预选不存在的文件；先选真实文件，再模拟选择后的删除。
            QVERIFY(dialog->setProperty("selectedFile", invalid));
            QCOMPARE(dialog->property("selectedFile").toUrl(), invalid);
            if (disappears)
                QVERIFY(QFile::remove(path));
            QVERIFY(QMetaObject::invokeMethod(dialog, "accepted"));
            QTRY_VERIFY(!project->projectOperationPending());
            QCOMPARE(opened.count(), 0);
            QVERIFY(!project->errorCode().isEmpty());
            QCOMPARE(project->currentPath(), previousPath);
            QCOMPARE(project->currentName(), QStringLiteral("Unsaved current"));
            QVERIFY(project->dirty());
            QCOMPARE(project->planSettings(), previousSettings);
            QCOMPARE(project->openDocuments(), previousDocuments);
            QCOMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("start-learn"));
        }

        QVERIFY(dialog->setProperty("selectedFile", selectedUrl));
        QVERIFY(QMetaObject::invokeMethod(dialog, "accepted"));
        QTRY_VERIFY(!project->projectOperationPending());
        QTRY_COMPARE(opened.count(), 1);
        QCOMPARE(opened.constFirst().constFirst().toString(), selectedPath);
        QCOMPARE(project->currentPath(), selectedPath);
        QCOMPARE(project->currentName(), storedName);
        QVERIFY(!project->dirty());
        QVERIFY(project->errorCode().isEmpty());
        QCOMPARE(project->activeDocumentId(), QStringLiteral("welcome"));
        QCOMPARE(project->openDocuments().size(), 1);
        QTRY_COMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("home"));
    }

    void project_workspace_tracks_service_data() {
        QTest::addColumn<bool>("openExisting");
        QTest::newRow("created") << false;
        QTest::newRow("opened") << true;
    }

    void project_workspace_tracks_service() {
        QFETCH(bool, openExisting);
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());

        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* root = engine.rootObjects().constFirst();
        auto* window = qobject_cast<QQuickWindow*>(root);
        auto* project =
            root->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* ribbon = root->findChild<QQuickItem*>(QStringLiteral("ribbonPanel"));
        auto* projectItem = root->findChild<QQuickItem*>(QStringLiteral("projectTaskItem"));
        QVERIFY(window && project && ribbon && projectItem);
        QCOMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("start-learn"));
        QVERIFY(!projectItem->isVisible());
        QCOMPARE(visual_items(ribbon, QStringLiteral("ribbonTileContent")).size(), 7);

        if (openExisting) {
            panta::bridge::ProjectViewModel savedProject;
            QVERIFY(savedProject.createProject(QStringLiteral("Stored"), fixture.path()));
            QTRY_VERIFY(!savedProject.projectOperationPending());
            QVERIFY(savedProject.errorCode().isEmpty());
            QVERIFY(project->openProjectUrl(QUrl::fromLocalFile(savedProject.currentPath())));
            QTRY_VERIFY(!project->projectOperationPending());
            QVERIFY(project->errorCode().isEmpty());
        } else {
            QVERIFY(project->createProject(QStringLiteral("Created"), fixture.path()));
            QTRY_VERIFY(!project->projectOperationPending());
            QVERIFY(project->errorCode().isEmpty());
        }
        QTRY_COMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("home"));
        QTRY_VERIFY(projectItem->isVisible());
        QTRY_COMPARE(visual_items(ribbon, QStringLiteral("ribbonTileContent")).size(), 18);
        QCOMPARE(projectItem->property("iconName").toString(), QStringLiteral("project-file"));
        QCOMPARE(projectItem->property("text").toString(),
                 QStringLiteral("Project '%1'").arg(project->currentName()));
        auto* home = visual_item(window->contentItem(), QStringLiteral("menu-home"));
        QVERIFY(home && home->property("highlighted").toBool());

        // 失败不能把当前工程视图退回首页，名称变更则必须更新可见任务项和标题。
        QVERIFY(project->openProject(fixture.filePath(QStringLiteral("missing.panta"))));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(!project->errorCode().isEmpty());
        QCOMPARE(ribbon->property("activeRibbonTab").toString(), QStringLiteral("home"));
        const QString renamed = QStringLiteral("Renamed project with a longer name");
        QVERIFY(project->renameProject(renamed));
        QCOMPARE(projectItem->property("text").toString(),
                 QStringLiteral("Project '%1'").arg(renamed));
        auto* caption = root->findChild<QObject*>(QStringLiteral("shellCaption"));
        QVERIFY(caption != nullptr);
        QCOMPARE(caption->property("text").toString(), QStringLiteral("panta 2027 · ") + renamed);

        auto* strip = root->findChild<QQuickItem*>(QStringLiteral("ribbonStrip"));
        auto* shared = visual_item(ribbon, QStringLiteral("ribbon-shared-views"));
        auto* menuStrip = root->findChild<QQuickItem*>(QStringLiteral("menuStrip"));
        auto* language = root->findChild<QQuickItem*>(QStringLiteral("languageButton"));
        QVERIFY(strip && shared && menuStrip && language);
        window->showNormal();
        for (const int width : {1440, 1280}) {
            window->resize(width, 900);
            QTest::qWait(50);
            QCOMPARE(ribbon->height(), 96.0);
            verify_ribbon_alignment(ribbon);
            for (auto* content : visual_items(menuStrip, QStringLiteral("toolButtonContent"))) {
                const auto* button = content->parentItem()->parentItem();
                QVERIFY(content->width() <= button->property("availableWidth").toDouble());
            }
            for (auto* content : visual_items(ribbon, QStringLiteral("ribbonTileContent"))) {
                const auto* tile = content->parentItem()->parentItem();
                const auto* group = tile->parentItem()->parentItem();
                QVERIFY(tile->width() >= 56.0);
                const QPointF top = tile->mapToItem(group, QPointF());
                QVERIFY(top.y() >= 0);
                QVERIFY(top.y() + tile->height() <= group->height() - 22);
                QVERIFY(content->width() <= tile->width());
                QVERIFY(content->height() <= tile->height());
            }
            for (auto* item : {shared, language}) {
                auto* container = item == shared ? strip : menuStrip;
                item->forceActiveFocus(Qt::TabFocusReason);
                QTest::qWait(50);
                const QPointF point = item->mapToItem(container, QPointF());
                QVERIFY(point.x() >= -1);
                QVERIFY(point.x() + item->width() <= container->width() + 1);
            }
        }
        auto* analyze = visual_item(ribbon, QStringLiteral("ribbon-analyze"));
        QVERIFY(analyze && !analyze->isEnabled());

        const QString captureDir = qEnvironmentVariable("PANTA_PROJECT_CAPTURE_DIR");
        if (!captureDir.isEmpty()) {
            window->resize(1440, 900);
            window->contentItem()->forceActiveFocus();
            strip->setProperty("contentX", 0);
            menuStrip->setProperty("contentX", 0);
            QTest::qWait(100);
            const QImage frame = window->grabWindow();
            QVERIFY(!frame.isNull());
            QDir().mkpath(captureDir);
            QVERIFY(frame.save(QDir(captureDir)
                                   .filePath(openExisting ? QStringLiteral("opened.png")
                                                          : QStringLiteral("created.png"))));
        }
    }

    void import_preview_waits_for_background_result_and_close_cancels() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        const auto path = fixture.filePath(QStringLiteral("preview.stl"));
        QFile source(path);
        QVERIFY(source.open(QIODevice::WriteOnly));
        QVERIFY(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n") > 0);
        source.close();
        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* root = engine.rootObjects().constFirst();
        auto* dialog = root->findChild<QObject*>(QStringLiteral("importDialog"));
        auto* model =
            root->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        QVERIFY(dialog && model);
        auto* interaction = model->importInteraction();
        auto* accept = dialog->findChild<QObject*>(QStringLiteral("importAccept"));
        QVERIFY(accept);
        QSignalSpy finished(model, &panta::bridge::ProjectViewModel::importPreviewFinished);
        QVERIFY(finished.isValid());
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Editing);
        QVERIFY(interaction->previewFile(path));
        QVERIFY(model->importPreviewPending());
        QVERIFY(!accept->property("enabled").toBool());
        QVERIFY(QMetaObject::invokeMethod(dialog, "close"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Closed);
        QVERIFY(!model->importPreviewPending());
        QVERIFY(!model->importPreviewReady());
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Editing);
        QVERIFY(interaction->previewFile(path));
        QTRY_VERIFY(model->importPreviewReady());
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Ready);
        QVERIFY(accept->property("enabled").toBool());
        QCOMPARE(finished.count(), 1);
        QVERIFY(QMetaObject::invokeMethod(dialog, "close"));
        QVERIFY(!model->importPreviewReady());
        QVERIFY(!accept->property("enabled").toBool());
    }

    void import_completion_does_not_close_a_reopened_dialog() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        const auto path = fixture.filePath(QStringLiteral("reopen.stl"));
        QFile source(path);
        QVERIFY(source.open(QIODevice::WriteOnly));
        QVERIFY(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n") > 0);
        source.close();

        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* root = engine.rootObjects().constFirst();
        auto* dialog = root->findChild<QObject*>(QStringLiteral("importDialog"));
        auto* model =
            root->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        QVERIFY(dialog && model);
        auto* interaction = model->importInteraction();
        QVERIFY(model->createProject(QStringLiteral("Reopen"), fixture.path()));
        QTRY_VERIFY(!model->projectOperationPending());
        QVERIFY(model->errorCode().isEmpty());
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Editing);
        QVERIFY(interaction->previewFile(path));
        QTRY_VERIFY(model->importPreviewReady());
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Ready);
        QVERIFY(QMetaObject::invokeMethod(dialog, "submit"));
        QVERIFY(model->importPending());
        const auto request = model->importRequestId();
        QVERIFY(request > 0);

        QVERIFY(QMetaObject::invokeMethod(dialog, "close"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Closed);
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QTRY_COMPARE(interaction->state(),
                     panta::bridge::ImportInteractionController::State::Editing);
        QVERIFY(dialog->property("visible").toBool());
        QVERIFY(QMetaObject::invokeMethod(model, "projectImportFinished", Qt::DirectConnection,
                                          Q_ARG(quint64, request), Q_ARG(bool, true),
                                          Q_ARG(bool, true)));
        QVERIFY(dialog->property("visible").toBool());
        QCOMPARE(interaction->state(), panta::bridge::ImportInteractionController::State::Editing);
        QVERIFY(dialog->property("visible").toBool());
    }

    void layers_tab_row_tracks_imports_and_reopened_projects() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());

        QQmlApplicationEngine engine;
        panta::install_icon_provider(engine);
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto* root = engine.rootObjects().constFirst();
        auto* window = qobject_cast<QQuickWindow*>(root);
        auto* project =
            root->findChild<panta::bridge::ProjectViewModel*>(QStringLiteral("projectModel"));
        auto* sidebarPanel = root->findChild<QQuickItem*>(QStringLiteral("sidebarPanel"));
        auto* layersPanel = root->findChild<QQuickItem*>(QStringLiteral("layersPanel"));
        auto* tabRow = root->findChild<QQuickItem*>(QStringLiteral("layersTabRow"));
        auto* toolbar = root->findChild<QQuickItem*>(QStringLiteral("layersToolbar"));
        auto* content = root->findChild<QQuickItem*>(QStringLiteral("layersContent"));
        QVERIFY(window && project && sidebarPanel && layersPanel && tabRow && toolbar && content);

        window->showNormal();
        window->resize(1440, 900);
        QTRY_VERIFY(sidebarPanel->isVisible());
        QTRY_VERIFY(layersPanel->isVisible());
        QTRY_VERIFY(toolbar->isVisible());
        QTRY_VERIFY(content->isVisible());
        QVERIFY(!tabRow->isVisible());

        QVERIFY(project->createProject(QStringLiteral("Imported"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        const QString importedProjectPath = project->currentPath();
        const QString firstStlPath = fixture.filePath(QStringLiteral("first.stl"));
        QFile firstStl(firstStlPath);
        QVERIFY(firstStl.open(QIODevice::WriteOnly | QIODevice::Text));
        QVERIFY(firstStl.write("solid sample\n"
                               "facet normal 0 0 1\n"
                               " outer loop\n"
                               "  vertex 0 0 0\n"
                               "  vertex 1 0 0\n"
                               "  vertex 0 1 0\n"
                               " endloop\n"
                               "endfacet\n"
                               "endsolid sample\n") > 0);
        firstStl.close();
        QVERIFY(project->inspectStl(firstStlPath));
        QTRY_VERIFY(project->importPreviewReady());
        QVERIFY(project->importStl(firstStlPath, QStringLiteral("dual-domain"),
                                   QStringLiteral("millimeters"), false));
        QTRY_VERIFY(tabRow->isVisible());
        QTRY_VERIFY(sidebarPanel->isVisible());
        QTRY_COMPARE(visual_items(sidebarPanel, QStringLiteral("importedPartEntry")).size(), 1);

        const QString secondStlPath = fixture.filePath(QStringLiteral("second.stl"));
        QVERIFY(QFile::copy(firstStlPath, secondStlPath));
        QVERIFY(project->inspectStl(secondStlPath));
        QTRY_VERIFY(project->importPreviewReady());
        QVERIFY(project->importStl(secondStlPath, QStringLiteral("dual-domain"),
                                   QStringLiteral("millimeters"), false));
        QTRY_COMPARE(visual_items(sidebarPanel, QStringLiteral("importedPartEntry")).size(), 2);
        QVERIFY(tabRow->isVisible());
        QVERIFY(sidebarPanel->isVisible());
        QVERIFY(project->saveProject());
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());

        QVERIFY(project->createProject(QStringLiteral("Empty"), fixture.path()));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        QTRY_VERIFY(!tabRow->isVisible());
        QTRY_VERIFY(layersPanel->isVisible());
        QTRY_VERIFY(toolbar->isVisible());
        QTRY_VERIFY(content->isVisible());
        QTRY_VERIFY(sidebarPanel->isVisible());

        QVERIFY(project->openProject(importedProjectPath));
        QTRY_VERIFY(!project->projectOperationPending());
        QVERIFY(project->errorCode().isEmpty());
        QTRY_VERIFY(tabRow->isVisible());
        QTRY_COMPARE(visual_items(sidebarPanel, QStringLiteral("importedPartEntry")).size(), 2);
        QVERIFY(sidebarPanel->isVisible());
    }
#endif
};

QTEST_MAIN(ShellModuleLoadTest)
#include "shell_module_load_test.moc"
