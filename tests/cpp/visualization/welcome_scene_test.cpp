#include "panta/visualization/mesh_source.hpp"
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
    void fill_legend_uses_real_range_and_hides_for_geometry() {
        vtkNew<vtkRenderWindow> render_window;
        panta::visualization::WelcomeScene scene;
        scene.attach(render_window);
        scene.set_visible(false);
        auto* props = render_window->GetRenderers()->GetFirstRenderer()->GetViewProps();
        props->InitTraversal();
        props->GetNextProp();
        panta::visualization::SurfaceMeshSnapshot mesh;
        mesh.fill_times = {0, 2.087};
        mesh.fill_duration = 2.087;
        scene.set_result_legend(&mesh, 1000, 700);
        auto* legend = props->GetNextProp();
        QVERIFY(legend->GetVisibility());
        auto* title = vtkBillboardTextActor3D::SafeDownCast(props->GetNextProp());
        QVERIFY(title != nullptr);
        QCOMPARE(QString::fromUtf8(title->GetInput()), QStringLiteral("Fill time [s]\n= 2.087"));
        mesh.pressures = {0.5, 1.234};
        mesh.pressure_visible = true;
        scene.set_result_legend(&mesh, 1000, 700);
        QCOMPARE(QString::fromUtf8(title->GetInput()), QStringLiteral("Pressure [MPa]\n= 1.234"));
        mesh.pressure_visible = false;
        scene.set_result_legend(&mesh, 1000, 700);
        QCOMPARE(QString::fromUtf8(title->GetInput()), QStringLiteral("Fill time [s]\n= 2.087"));
        mesh.fields_visible = false;
        scene.set_result_legend(&mesh, 1000, 700);
        QVERIFY(!legend->GetVisibility());
        QVERIFY(!title->GetVisibility());
    }
};

QTEST_APPLESS_MAIN(WelcomeSceneTest)
#include "welcome_scene_test.moc"
