// 私有场景构建的 CPU 回归；不创建图形 surface 或提交 GPU 帧。
#include "primitive_scene.hpp"
#include "surface_mesh.hpp"
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
#include <vtkPointData.h>
#include <vtkPolyData.h>
#include <vtkPolyDataMapper.h>
#include <vtkProperty.h>
#include <vtkWeakPointer.h>
#include <vtkWebGPURenderer.h>

class PrimitiveSceneTest final : public QObject {
    Q_OBJECT
  private slots:
    void replaces_geometry_and_preserves_styles() {
        vtkNew<vtkActor> actor;
        panta::visualization::replace_primitive_geometry(actor, nullptr, {}, 0.0);
        vtkWeakPointer<vtkMapper> previous = actor->GetMapper();
        QVERIFY(previous != nullptr);
        QCOMPARE(actor->GetOrientation()[0], 16.0);
        QCOMPARE(actor->GetProperty()->GetAmbient(), 0.22);
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{10, 0, 0}}, {{0, 20, 0}}};
        mesh->show_edges = true;
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.0);
        QVERIFY(previous == nullptr);
        for (int axis = 0; axis < 3; ++axis) {
            QCOMPARE(actor->GetOrientation()[axis], 0.0);
        }
        QCOMPARE(actor->GetProperty()->GetColor()[0], 0.72);
        QCOMPARE(actor->GetProperty()->GetEdgeVisibility(), 1);
        QCOMPARE(actor->GetProperty()->GetSpecularPower(), 28.0);
        double bounds[6];
        actor->GetBounds(bounds);
        QCOMPARE(bounds[1], 10.0);
        QCOMPARE(bounds[3], 20.0);
        panta::visualization::replace_primitive_geometry(actor, nullptr, {}, 0.0);
        QCOMPARE(actor->GetOrientation()[1], -18.0);
    }
    void fits_camera_to_aspect_and_margin() {
        vtkNew<vtkActor> actor;
        vtkNew<vtkWebGPURenderer> renderer;
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{10, 0, 0}}, {{0, 20, 0}}};
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.0);
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
    void maps_fill_time_and_pressure_fields() {
        vtkNew<vtkActor> actor;
        auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{1, 0, 0}}, {{0, 1, 0}}};
        mesh->fill_times = {0.0, 1.0, 1.0};
        mesh->fill_duration = 1.0;
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.5);
        auto* mapper = vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
        QVERIFY(mapper != nullptr);
        auto* fill_data = mapper->GetInput();
        QVERIFY(fill_data != nullptr);
        QVERIFY(fill_data->GetPointData()->GetScalars() != nullptr);
        QCOMPARE(fill_data->GetNumberOfPolys(), vtkIdType{3});
        double first_bounds[6];
        fill_data->GetBounds(first_bounds);
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.9);
        mapper = vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
        QVERIFY(mapper != nullptr);
        double later_bounds[6];
        mapper->GetInput()->GetBounds(later_bounds);
        for (int i = 0; i < 6; ++i) {
            QCOMPARE(later_bounds[i], first_bounds[i]);
        }

        mesh->pressure_visible = true;
        mesh->pressures = {0.0, 0.5, 1.0};
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.8);
        mapper = vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
        QVERIFY(mapper != nullptr);
        auto* pressure_data = mapper->GetInput();
        QVERIFY(pressure_data != nullptr);
        QVERIFY(pressure_data->GetPointData()->GetScalars() != nullptr);
        QCOMPARE(pressure_data->GetNumberOfPolys(), vtkIdType{1});
    }
};
QTEST_GUILESS_MAIN(PrimitiveSceneTest)
#include "primitive_scene_test.moc"
