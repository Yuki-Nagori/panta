#include "../../support/qt/frame_submission_capture.hpp"
/// Panta.Visualization 运行时加载测试（任务 007）：验证静态模块注册与
/// CaeViewport 类型可创建。
///
/// 默认只验证无头模块加载；PANTA_TEST_NATIVE_VIEWPORT=1 时在真实桌面
/// 验证帧合并、隐藏恢复、跨窗口重建与析构，不能使用 offscreen 平台。

#include "panta/visualization/mesh_source.hpp"
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
#include <qtenvironmentvariables.h>
#include <qtestsupport_core.h>
#include <utility>
#include <vector>
#include <vtk_viewport.hpp>

Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)

namespace panta::visualization {
class ViewportTestAccess final {
  public:
    static const void* actor_identity(const VtkViewport& viewport) {
        return viewport.test_actor_identity();
    }
    static bool has_mapper(const VtkViewport& viewport) { return viewport.test_has_mapper(); }
    static bool previous_mapper_released(const VtkViewport& viewport) {
        return viewport.test_previous_mapper_released();
    }
    static bool interaction_observers_registered(const VtkViewport& viewport) {
        return viewport.test_interaction_observers_registered();
    }
    static bool previous_window_resources_released(const VtkViewport& viewport) {
        return viewport.test_previous_window_resources_released();
    }
};
} // namespace panta::visualization

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
        panta::test::FrameSubmissionCapture capture;
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
        capture.reset();
        panta::visualization::RenderScene scene;
        for (int i = 0; i < 100; ++i) {
            scene.revision = i + 1;
            scene.background = QColor(i, 80, 160);
            viewport.apply_state(scene);
            viewport.setWidth(400 + i);
        }
        QTest::qWait(100);
        QCOMPARE(capture.count(), 1);

        auto snapshot = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        snapshot->vertices = {{{0.0, 0.0, 0.0}, {1.0, 0.0, 0.0}, {0.0, 1.0, 0.0}}};
        const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> snapshot_lifetime =
            snapshot;
        scene.mesh = snapshot;
        snapshot.reset();
        ++scene.revision;
        capture.reset();
        viewport.apply_state(scene);
        QTRY_VERIFY_WITH_TIMEOUT(capture.count() > 0, 10000);
        QVERIFY(!snapshot_lifetime.expired());
        const void* actor_identity =
            panta::visualization::ViewportTestAccess::actor_identity(viewport);
        QVERIFY(actor_identity != nullptr);
        QVERIFY(panta::visualization::ViewportTestAccess::has_mapper(viewport));
        QVERIFY(
            panta::visualization::ViewportTestAccess::interaction_observers_registered(viewport));

        std::vector<std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot>> old_snapshots;
        old_snapshots.push_back(snapshot_lifetime);
        for (int cycle = 0; cycle < 12; ++cycle) {
            auto next = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
            const double x = static_cast<double>(cycle + 1);
            next->vertices = {{{x, 0.0, 0.0}, {x + 1.0, 0.0, 0.0}, {x, 1.0, 0.0}}};
            old_snapshots.emplace_back(next);
            scene.mesh = std::move(next);
            ++scene.revision;
            capture.reset();
            viewport.apply_state(scene);
            QTRY_VERIFY_WITH_TIMEOUT(capture.count() > 0, 10000);
            QCOMPARE(panta::visualization::ViewportTestAccess::actor_identity(viewport),
                     actor_identity);
            QVERIFY(panta::visualization::ViewportTestAccess::previous_mapper_released(viewport));
            QVERIFY(panta::visualization::ViewportTestAccess::has_mapper(viewport));
            QVERIFY(panta::visualization::ViewportTestAccess::interaction_observers_registered(
                viewport));
            for (std::size_t index = 0; index + 1 < old_snapshots.size(); ++index) {
                QVERIFY(old_snapshots[index].expired());
            }
        }

        viewport.setVisible(false);
        scene.mesh.reset();
        ++scene.revision;
        viewport.apply_state(scene);
        QTRY_VERIFY_WITH_TIMEOUT(snapshot_lifetime.expired(), 5000);
        QVERIFY(panta::visualization::ViewportTestAccess::previous_mapper_released(viewport));
        capture.reset();
        viewport.setVisible(true);
        QTRY_VERIFY_WITH_TIMEOUT(capture.count() > 0, 10000);

        capture.reset();
        viewport.apply_state(scene);
        ++scene.revision;
        viewport.apply_state(scene);
        parent.setPosition(QPointF(20, 10));
        QTest::qWait(100);
        QCOMPARE(capture.count(), 0);
        QCOMPARE(capture.surface_sync_count(), 1);

        viewport.setVisible(false);
        scene.primitive_visible = false;
        ++scene.revision;
        viewport.apply_state(scene);
        QTest::qWait(100);
        QCOMPARE(capture.count(), 0);
        viewport.setVisible(true);
        QTest::qWait(100);
        QCOMPARE(capture.count(), 1);

        capture.reset();
        const qreal width = viewport.width();
        viewport.setWidth(0);
        QTest::qWait(100);
        QCOMPARE(capture.count(), 0);
        viewport.setWidth(width);
        QTest::qWait(100);
        QCOMPARE(capture.count(), 1);

        second.show();
        parent.setParentItem(second.contentItem());
        QTRY_COMPARE_WITH_TIMEOUT(initialized.count(), 2, 10000);
        QVERIFY(
            panta::visualization::ViewportTestAccess::previous_window_resources_released(viewport));
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
