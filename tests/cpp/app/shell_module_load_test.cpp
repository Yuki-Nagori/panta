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
#include <QDir>
#include <QFileInfo>
#include <QImage>
#include <QQuickItem>
#include <QQuickWindow>
#include <QtQml/qqmlextensionplugin.h>
#endif
#ifdef PANTA_ENABLE_BRIDGE_MODULE

// App.qml 引入 Panta.Visualization（CaeViewport）：静态模块的消费方二进制
// 必须同时导入并链接其 plugin，否则运行时报 "module not installed"。
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)
#endif

// 布局几何诊断（029 常驻）：打印组件内容树几何，供布局对照与优化。
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

class ShellModuleLoadTest final : public QObject {
    Q_OBJECT

  private slots:
    void loads_shell_module() {
        // 与 main.cpp 同源：Shell 自绘控件要求非原生样式（029）。
        QQuickStyle::setStyle(QStringLiteral("Basic"));
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
                    root->findChildren<QQuickItem*>(QStringLiteral("toolButtonContent"));
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
                for (const char* name : {"ribbonStart", "ribbonNew", "ribbonLearn"}) {
                    auto* tile = root->findChild<QQuickItem*>(QString::fromLatin1(name));
                    QVERIFY(tile != nullptr);
                    QCOMPARE(tile->width(), 64.0);
                    QCOMPARE(tile->height(), tile->width());
                    const auto* ribbon = tile->parentItem()->parentItem();
                    const qreal topGap = tile->mapToItem(ribbon, QPointF()).y();
                    const qreal bottomGap = ribbon->height() - 1 - topGap - tile->height();
                    QCOMPARE(topGap, bottomGap);
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
            auto* window = qobject_cast<QQuickWindow*>(root);
            if (window == nullptr) {
                QFAIL("Shell root is not a QQuickWindow");
            }
            if (!capturePath.isEmpty()) {
                const int captureWidth = qEnvironmentVariableIntValue("PANTA_SHELL_CAPTURE_WIDTH");
                window->showNormal();
                window->resize(captureWidth > 0 ? captureWidth : 1440, 900);
                QTest::qWait(100);
            }
            if (dumpGeometry) {
                qWarning() << "shell window" << window->width() << window->height();
                dump_item_tree(window->contentItem(), 0);
            }
            if (!capturePath.isEmpty()) {
                const QImage frame = window->grabWindow();
                QVERIFY2(!frame.isNull(), "grabWindow returned an empty frame");
                QDir().mkpath(QFileInfo(capturePath).absolutePath());
                QVERIFY2(frame.save(capturePath), "Failed to save captured frame");
            }
        }
#endif
    }
};

QTEST_MAIN(ShellModuleLoadTest)
#include "shell_module_load_test.moc"
