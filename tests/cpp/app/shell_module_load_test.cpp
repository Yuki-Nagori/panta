// Panta.Shell 运行时加载测试：验证资源模块及其可选 Bridge 依赖的链接契约。

#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#ifdef PANTA_ENABLE_BRIDGE_MODULE
#include <QtQml/qqmlextensionplugin.h>
#endif
#ifdef PANTA_ENABLE_BRIDGE_MODULE

// App.qml 引入 Panta.Visualization（CaeViewport）：静态模块的消费方二进制
// 必须同时导入并链接其 plugin，否则运行时报 "module not installed"。
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

class ShellModuleLoadTest final : public QObject {
    Q_OBJECT

  private slots:
    void loads_shell_module() {
        QQmlApplicationEngine engine;
#ifdef PANTA_ENABLE_BRIDGE_MODULE
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));
#else
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("AppNoBridge"));
#endif

        QVERIFY2(!engine.rootObjects().isEmpty(), "Panta.Shell entry failed to load");
    }
};

QTEST_MAIN(ShellModuleLoadTest)
#include "shell_module_load_test.moc"
