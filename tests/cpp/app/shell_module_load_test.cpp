// Panta.Shell 运行时加载测试：验证资源模块及其可选 Bridge 依赖的链接契约。

#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QString>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtQuickControls2/qquickstyle.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <QtTest/qtestmouse.h>
#include <icon_provider.hpp>
#include <qlogging.h>
#include <qtenvironmentvariables.h>
#include <qtestsupport_core.h>
#ifdef PANTA_ENABLE_BRIDGE_MODULE
#include "../../qml/quick_item_helpers.hpp"
#include <QDir>
#include <QFileInfo>
#include <QImage>
#include <QPoint>
#include <QPointF>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QUrl>
#include <QtCore/qnamespace.h>
#include <QtQml/qqmlextensionplugin.h>
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
        const auto assert_visible = [root](const char* object_name) {
            auto* object = root->findChild<QObject*>(QString::fromLatin1(object_name));
            if (object == nullptr) {
                return false;
            }
            return object->property("visible").toBool();
        };
        QVERIFY2(assert_visible("shellCaption"), "Shell caption is missing or hidden");
#ifdef PANTA_ENABLE_BRIDGE_MODULE
        QVERIFY2(root->findChild<QObject*>(QStringLiteral("caeViewport")) != nullptr,
                 "CaeViewport is missing from Panta.Shell");
#endif

#ifdef PANTA_ENABLE_BRIDGE_MODULE
        // 标题组不能被 Layout 压到内容宽度以下；窄窗口 Tab 到搜索时应自动滚入视口。
        auto* shellWindow = qobject_cast<QQuickWindow*>(root);
        QVERIFY(shellWindow != nullptr);
        auto* strip = root->findChild<QQuickItem*>(QStringLiteral("titleStrip"));
        auto* search = root->findChild<QQuickItem*>(QStringLiteral("globalSearch"));
        QVERIFY(strip != nullptr);
        QVERIFY(search != nullptr);
        shellWindow->showNormal();
        for (const int width : {1440, 640}) {
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
                        QVERIFY(qAbs(center.x() - button->width() / 2) <= 0.5);
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
        const QPointF focusedPosition = search->mapToItem(strip, QPointF());
        QVERIFY(focusedPosition.x() >= -1);
        QVERIFY(focusedPosition.x() + search->width() <= strip->width() + 1);
        animationButton->setProperty("text", originalText);
        QTest::qWait(50);

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

#ifdef PANTA_ENABLE_BRIDGE_MODULE
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
        QVERIFY(!ribbon->property("projectOpen").toBool());
        QVERIFY(!projectItem->isVisible());
        QCOMPARE(visual_items(ribbon, QStringLiteral("ribbonTileContent")).size(), 7);

        if (openExisting) {
            panta::bridge::ProjectViewModel savedProject;
            QVERIFY(savedProject.createProject(QStringLiteral("Stored"), fixture.path()));
            QVERIFY(project->openProjectUrl(QUrl::fromLocalFile(savedProject.currentPath())));
        } else {
            QVERIFY(project->createProject(QStringLiteral("Created"), fixture.path()));
        }
        QTRY_VERIFY(ribbon->property("projectOpen").toBool());
        QTRY_VERIFY(projectItem->isVisible());
        QTRY_COMPARE(visual_items(ribbon, QStringLiteral("ribbonTileContent")).size(), 18);
        QCOMPARE(projectItem->property("iconName").toString(), QStringLiteral("project-file"));
        QCOMPARE(projectItem->property("text").toString(),
                 QStringLiteral("Project '%1'").arg(project->currentName()));
        auto* home = visual_item(window->contentItem(), QStringLiteral("menu-home"));
        QVERIFY(home && home->property("highlighted").toBool());

        // 失败不能把当前工程视图退回首页，名称变更则必须更新可见任务项和标题。
        QVERIFY(!project->openProject(fixture.filePath(QStringLiteral("missing.panta"))));
        QVERIFY(ribbon->property("projectOpen").toBool());
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
        for (const int width : {1440, 640}) {
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
#endif
};

QTEST_MAIN(ShellModuleLoadTest)
#include "shell_module_load_test.moc"
