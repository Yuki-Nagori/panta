// Panta.Shell 运行时加载测试：验证资源模块与静态 Bridge plugin 的链接契约。

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/QtTest>

Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)

class ShellModuleLoadTest final : public QObject {
    Q_OBJECT

private slots:
    void loads_shell_module_with_bridge_type() {
        QQmlApplicationEngine engine;
        engine.loadFromModule(QStringLiteral("Panta.Shell"), QStringLiteral("App"));

        QVERIFY2(!engine.rootObjects().isEmpty(), "Panta.Shell App failed to load");
    }
};

QTEST_MAIN(ShellModuleLoadTest)
#include "shell_module_load_test.moc"
