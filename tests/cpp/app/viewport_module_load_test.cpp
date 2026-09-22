/// Panta.Visualization 运行时加载测试（任务 007）：验证静态模块注册与
/// CaeViewport 类型可创建。
///
/// 默认只验证无头模块加载；PANTA_TEST_NATIVE_VIEWPORT=1 时在真实桌面
/// 验证帧合并、隐藏恢复、跨窗口重建与析构，不能使用 offscreen 平台。

#include <QGuiApplication>
#include <QLoggingCategory>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QString>
#include <QUrl>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <cstdio>
#include <cstring>
#include <memory>
#include <panta/visualization/render_scene.hpp>
#include <qlogging.h>
#include <qtenvironmentvariables.h>
#include <qtestsupport_core.h>
#include <vtk_viewport.hpp>

Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)

namespace {
int rendered_frames = 0;
int surface_syncs = 0;
QtMessageHandler previous_handler = nullptr;

void count_frames(QtMsgType type, const QMessageLogContext& context, const QString& message) {
    if (std::strcmp(context.category, "panta.viewport") == 0 &&
        message.startsWith(QStringLiteral("frame submitted"))) {
        ++rendered_frames;
    }
    if (std::strcmp(context.category, "panta.viewport") == 0 &&
        message == QStringLiteral("surface synchronized")) {
        ++surface_syncs;
    }
    if (previous_handler != nullptr) {
        previous_handler(type, context, message);
    } else {
        std::fprintf(stderr, "%s\n", qPrintable(message));
    }
}

struct FrameCapture {
    FrameCapture() {
        rendered_frames = 0;
        QLoggingCategory::setFilterRules(QStringLiteral("panta.viewport.debug=true"));
        previous_handler = qInstallMessageHandler(count_frames);
    }
    ~FrameCapture() {
        qInstallMessageHandler(previous_handler);
        QLoggingCategory::setFilterRules(QString());
    }
};
} // namespace

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

    void native_refresh_and_window_lifecycle() {
        if (!qEnvironmentVariableIsSet("PANTA_TEST_NATIVE_VIEWPORT")) {
            QSKIP("Native GPU lifecycle requires PANTA_TEST_NATIVE_VIEWPORT=1 and a real desktop");
        }
        const FrameCapture capture;
        QQuickWindow first;
        QQuickWindow second;
        first.resize(480, 360);
        second.resize(480, 360);
        QQuickItem parent(first.contentItem());
        panta::visualization::VtkViewport viewport(&parent);
        viewport.setSize(QSizeF(400, 300));
        QSignalSpy initialized(&viewport, &panta::visualization::VtkViewport::sceneInitialized);
        // 先消费尚未显示时的刷新，验证构造时已有窗口归属仍能监听后续 show。
        QCoreApplication::processEvents();
        first.show();
        QTRY_COMPARE_WITH_TIMEOUT(initialized.count(), 1, 10000);
        QTest::qWait(100);
        rendered_frames = 0;
        panta::visualization::RenderScene scene;
        for (int i = 0; i < 100; ++i) {
            scene.revision = i + 1;
            scene.background = QColor(i, 80, 160);
            viewport.apply_state(scene);
            viewport.setWidth(400 + i);
        }
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 1);

        rendered_frames = 0;
        surface_syncs = 0;
        viewport.apply_state(scene);
        ++scene.revision;
        viewport.apply_state(scene);
        parent.setPosition(QPointF(20, 10));
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 0);
        QCOMPARE(surface_syncs, 1);

        viewport.setVisible(false);
        scene.primitive_visible = false;
        ++scene.revision;
        viewport.apply_state(scene);
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 0);
        viewport.setVisible(true);
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 1);

        rendered_frames = 0;
        const qreal width = viewport.width();
        viewport.setWidth(0);
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 0);
        viewport.setWidth(width);
        QTest::qWait(100);
        QCOMPARE(rendered_frames, 1);

        second.show();
        parent.setParentItem(second.contentItem());
        QTRY_COMPARE_WITH_TIMEOUT(initialized.count(), 2, 10000);
        // 原窗口关闭后新宿主仍可更新；排队刷新在条目析构后不会再执行。
        first.close();
        viewport.setWidth(420);
    }

    void destroys_attached_viewport_with_pending_refresh() {
        QQuickWindow window;
        {
            panta::visualization::VtkViewport viewport(window.contentItem());
            viewport.setSize(QSizeF(200, 100));
            viewport.apply_state({});
        }
        // 基类析构的 windowChanged 与待执行的刷新都不得访问已释放的 Impl。
        QCoreApplication::processEvents();
    }
};

QTEST_MAIN(ViewportModuleLoadTest)
#include "viewport_module_load_test.moc"
