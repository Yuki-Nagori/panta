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
        panta::visualization::WelcomeScene scene;
        scene.set_visible(true);
        scene.attach(render_window);
        QCOMPARE(render_window->GetNumberOfLayers(), 2);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 1);

        auto* renderer = render_window->GetRenderers()->GetFirstRenderer();
        if (renderer == nullptr) {
            QFAIL("welcome overlay renderer was not attached");
            return;
        }
        QVERIFY(renderer->GetActiveCamera()->GetParallelProjection());
        vtkProp* prop = renderer->GetViewProps()->GetNextProp();
        auto* text = vtkBillboardTextActor3D::SafeDownCast(prop);
        QVERIFY(text != nullptr);
        QCOMPARE(QString::fromUtf8(text->GetInput()), QStringLiteral("Welcome!"));
        QVERIFY(text->GetVisibility());
        scene.set_visible(false);
        QVERIFY(!text->GetVisibility());
        scene.set_visible(true);
        QVERIFY(text->GetVisibility());

        scene.detach();
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 0);
        scene.detach();
    }
};

QTEST_APPLESS_MAIN(WelcomeSceneTest)
#include "welcome_scene_test.moc"
