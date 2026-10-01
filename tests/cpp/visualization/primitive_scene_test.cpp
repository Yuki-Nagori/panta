// 私有场景构建的 CPU 回归；不创建图形 surface 或提交 GPU 帧。
#include "primitive_scene.hpp"
#include <QObject>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <cmath>
#include <memory>
#include <panta/visualization/mesh_source.hpp>
#include <vtkActor.h>
#include <vtkCamera.h>
#include <vtkMapper.h>
#include <vtkNew.h>
#include <vtkProperty.h>
#include <vtkWeakPointer.h>
#include <vtkWebGPURenderer.h>

class PrimitiveSceneTest final : public QObject {
    Q_OBJECT
  private slots:
    void replaces_geometry_and_preserves_styles() {
        vtkNew<vtkActor> actor;
        panta::visualization::replace_primitive_geometry(actor, nullptr, {});
        vtkWeakPointer<vtkMapper> previous = actor->GetMapper();
        QVERIFY(previous != nullptr);
        QCOMPARE(actor->GetOrientation()[0], 16.0);
        QCOMPARE(actor->GetProperty()->GetAmbient(), 0.22);
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{10, 0, 0}}, {{0, 20, 0}}};
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh);
        QVERIFY(previous == nullptr);
        for (int axis = 0; axis < 3; ++axis) {
            QCOMPARE(actor->GetOrientation()[axis], 0.0);
        }
        QCOMPARE(actor->GetProperty()->GetColor()[0], 0.72);
        QCOMPARE(actor->GetProperty()->GetSpecularPower(), 28.0);
        double bounds[6];
        actor->GetBounds(bounds);
        QCOMPARE(bounds[1], 10.0);
        QCOMPARE(bounds[3], 20.0);
        panta::visualization::replace_primitive_geometry(actor, nullptr, {});
        QCOMPARE(actor->GetOrientation()[1], -18.0);
    }
    void fits_camera_to_aspect_and_margin() {
        vtkNew<vtkActor> actor;
        vtkNew<vtkWebGPURenderer> renderer;
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{10, 0, 0}}, {{0, 20, 0}}};
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh);
        renderer->AddActor(actor);
        for (double aspect : {1.0, 0.25}) {
            panta::visualization::configure_default_camera(renderer, actor, aspect, 1.25);
            auto* camera = renderer->GetActiveCamera();
            QCOMPARE(camera->GetFocalPoint()[0], 5.0);
            QCOMPARE(camera->GetFocalPoint()[1], 10.0);
            const double expected =
                1.25 * (aspect == 1.0 ? 10.0 : 20.0) / std::tan(0.2617993877991494);
            QVERIFY(std::abs(camera->GetPosition()[2] - expected) < 1e-9);
            QCOMPARE(camera->GetViewAngle(), 30.0);
        }
    }
};
QTEST_GUILESS_MAIN(PrimitiveSceneTest)
#include "primitive_scene_test.moc"
