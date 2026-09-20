/// Panta.Visualization 运行时加载测试（任务 007）：验证静态模块注册与
/// CaeViewport 类型可创建。
///
/// 边界（standards/vtk.md）：无头环境（offscreen/软件 scenegraph）不受
/// VTK 支持会显式报错甚至中止，因此本测试只创建 QML 对象、不触发渲染
/// 线程管线；真实视口冒烟由窗口化 cargo run 人工记录。

#include <QGuiApplication>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QString>
#include <QUrl>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <cstdio>
#include <memory>

Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)

class ViewportModuleLoadTest final : public QObject {
    Q_OBJECT

  private slots:
    void creates_cae_viewport() {
        QQmlEngine engine;
        QQmlComponent component(&engine);
        component.setData("import Panta.Visualization\nCaeViewport {}\n",
                          QUrl(QStringLiteral("qrc:///qt/qml/panta-tests/viewport-load/main.qml")));
        QVERIFY2(component.isReady(), component.errorString().toUtf8().constData());
        std::unique_ptr<QObject> object(component.create());
        QVERIFY2(object != nullptr, "CaeViewport 创建失败");
    }
};

// Windows CI 曾在链接 VTK 静态库的本测试出现无输出挂起（任务 007 验证表
// 2026-09-20）：显式 main 逐阶段冲刷 stderr 定位挂点，根因解决后移除。
// TODO(task 007): Windows 挂起根因确认后删除阶段标记。
int main(int argc, char** argv) {
    std::fputs("viewport-test: reached main\n", stderr);
    std::fflush(stderr);
    QGuiApplication app(argc, argv);
    std::fputs("viewport-test: QGuiApplication ready\n", stderr);
    std::fflush(stderr);
    ViewportModuleLoadTest test;
    const int status = QTest::qExec(&test, argc, argv);
    std::fprintf(stderr, "viewport-test: finished status=%d\n", status);
    std::fflush(stderr);
    return status;
}
#include "viewport_module_load_test.moc"
