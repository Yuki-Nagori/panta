// Panta.Shell 运行时加载测试：验证资源模块及其可选 Bridge 依赖的链接契约。

#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <QtQuickControls2/qquickstyle.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
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

// 布局几何诊断（029 常驻）：打印内容树前三层几何，供布局对照与优化。
void dump_item_tree(const QQuickItem* item, int depth) {
    qWarning().nospace() << QString(depth * 2, ' ') << item->metaObject()->className()
                         << " objectName=" << item->objectName() << " geom=" << item->x() << ","
                         << item->y() << " " << item->width() << "x" << item->height();
    if (depth < 6) {
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
        QVERIFY2(assert_visible("advanceRevisionButton"), "Revision button is missing or hidden");
        QVERIFY2(root->findChild<QObject*>(QStringLiteral("caeViewport")) != nullptr,
                 "CaeViewport is missing from Panta.Shell");
#endif

#ifdef PANTA_ENABLE_BRIDGE_MODULE
        // 布局取证与几何诊断（029，常驻）：设置环境变量时保存首帧 PNG 或
        // 打印几何树，供真实布局对照与后续优化；ctest 默认不设置，无副作用。
        const QString capturePath = qEnvironmentVariable("PANTA_SHELL_CAPTURE_PATH");
        const bool dumpGeometry = qEnvironmentVariableIsSet("PANTA_SHELL_DUMP_GEOMETRY");
        if (!capturePath.isEmpty() || dumpGeometry) {
            auto* window = qobject_cast<QQuickWindow*>(root);
            if (window == nullptr) {
                QFAIL("Shell root is not a QQuickWindow");
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
