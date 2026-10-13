#include "navigation/viewport_orientation.hpp"
#include "welcome/welcome_scene.hpp"
#include <QString>
#include <QtCore/qobject.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <qtmetamacros.h>
#include <vtkBillboardTextActor3D.h>
#include <vtkCamera.h>
#include <vtkNew.h>
#include <vtkProp.h>
#include <vtkPropCollection.h>
#include <vtkRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkRendererCollection.h>

class WelcomeSceneTest final : public QObject {
    Q_OBJECT

  private slots:
    void attaches_static_label_and_releases_renderer() {
        vtkNew<vtkRenderWindow> render_window;
        panta::visualization::ViewportOrientation orientation;
        orientation.attach(render_window);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 2);

        panta::visualization::WelcomeScene scene;
        scene.sync(render_window, true, false);
        QCOMPARE(render_window->GetNumberOfLayers(), 2);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 3);

        vtkBillboardTextActor3D* text = nullptr;
        vtkRenderer* welcome_renderer = nullptr;
        auto* renderers = render_window->GetRenderers();
        renderers->InitTraversal();
        while (auto* renderer = renderers->GetNextItem()) {
            vtkProp* prop = renderer->GetViewProps()->GetNextProp();
            text = vtkBillboardTextActor3D::SafeDownCast(prop);
            if (text != nullptr) {
                welcome_renderer = renderer;
                break;
            }
        }
        if (welcome_renderer == nullptr) {
            QFAIL("welcome overlay renderer was not attached");
            return;
        }
        QVERIFY(welcome_renderer->GetActiveCamera()->GetParallelProjection());
        QVERIFY(text != nullptr);
        QCOMPARE(QString::fromUtf8(text->GetInput()), QStringLiteral("Welcome!"));
        QVERIFY(text->GetVisibility());
        scene.sync(render_window, true, true);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 2);
        scene.sync(render_window, true, false);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 3);
        scene.sync(render_window, false, false);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 2);
        scene.sync(render_window, true, false);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 3);
        scene.detach();
        orientation.detach();
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 0);
    }
};

QTEST_APPLESS_MAIN(WelcomeSceneTest)
#include "welcome_scene_test.moc"
