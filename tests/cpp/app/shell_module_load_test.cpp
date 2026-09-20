// Panta.Shell 运行时加载测试：验证资源模块及其可选 Bridge 依赖的链接契约。

#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <cstdio>
#ifdef PANTA_ENABLE_BRIDGE_MODULE
#include <QtQml/qqmlextensionplugin.h>
#endif
#ifdef PANTA_ENABLE_BRIDGE_MODULE

// App.qml 引入 Panta.Visualization（CaeViewport）：静态模块的消费方二进制
// 必须同时导入并链接其 plugin，否则运行时报 "module not installed"。
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)
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
        QVERIFY2(assert_visible("revisionCount"), "Revision count is missing or hidden");
        QVERIFY2(root->findChild<QObject*>(QStringLiteral("caeViewport")) != nullptr,
                 "CaeViewport is missing from Panta.Shell");
#endif
    }
};

// Windows CI 曾在链接 Panta.Visualization 静态 plugin 的本测试出现无输出
// 挂起（任务 007 验证表 2026-09-20）：显式 main 逐阶段冲刷 stderr 定位
// 挂点，根因解决后移除。
// TODO(task 007): Windows 挂起根因确认后删除阶段标记。
int main(int argc, char** argv) {
    std::fputs("shell-test: reached main\n", stderr);
    std::fflush(stderr);
    QGuiApplication app(argc, argv);
    std::fputs("shell-test: QGuiApplication ready\n", stderr);
    std::fflush(stderr);
    ShellModuleLoadTest test;
    const int status = QTest::qExec(&test, argc, argv);
    std::fprintf(stderr, "shell-test: finished status=%d\n", status);
    std::fflush(stderr);
    return status;
}
#include "shell_module_load_test.moc"
