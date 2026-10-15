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
#include <vtkType.h>
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
        QCOMPARE(actor->GetProperty()->GetEdgeVisibility(), 0);
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
        // 未充填区域保持整面，包围盒不变；前沿沿底边从 0.5 推到 0.9。
        auto front_x = [](vtkDataSet* data) {
            double found = -1.0;
            for (vtkIdType index = 0; index < data->GetNumberOfPoints(); ++index) {
                double point[3];
                data->GetPoint(index, point);
                if (std::abs(point[1]) < 1e-9 && std::abs(point[2]) < 1e-9 && point[0] > 1e-6 &&
                    point[0] < 1.0 - 1e-6) {
                    found = point[0];
                }
            }
            return found;
        };
        QCOMPARE(front_x(fill_data), 0.5);
        panta::visualization::replace_primitive_geometry(actor, nullptr, mesh, 0.9);
        mapper = vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
        QVERIFY(mapper != nullptr);
        double later_bounds[6];
        mapper->GetInput()->GetBounds(later_bounds);
        for (int i = 0; i < 6; ++i) {
            QCOMPARE(later_bounds[i], first_bounds[i]);
        }
        QCOMPARE(front_x(mapper->GetInput()), 0.9);

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
    void playback_time_rebuilds_geometry_without_refitting_camera() {
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        const auto other = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        const auto unchanged =
            panta::visualization::primitive_geometry_update(mesh, 0.5, mesh, 0.5, true);
        QVERIFY(!unchanged.rebuild);
        QVERIFY(!unchanged.refit_camera);
        const auto time_only =
            panta::visualization::primitive_geometry_update(mesh, 0.5, mesh, 0.9, true);
        QVERIFY(time_only.rebuild);
        QVERIFY(!time_only.refit_camera);
        const auto ignored =
            panta::visualization::primitive_geometry_update(mesh, 0.5, mesh, 0.9, false);
        QVERIFY(!ignored.rebuild);
        QVERIFY(!ignored.refit_camera);
        const auto mesh_only =
            panta::visualization::primitive_geometry_update(mesh, 0.5, other, 0.5, false);
        QVERIFY(mesh_only.rebuild);
        QVERIFY(mesh_only.refit_camera);
        const auto replaced =
            panta::visualization::primitive_geometry_update(nullptr, 0.0, mesh, 1.0, true);
        QVERIFY(replaced.rebuild);
        QVERIFY(replaced.refit_camera);
    }
    void playback_affects_only_the_fill_time_surface() {
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        QVERIFY(!panta::visualization::playback_affects_surface(mesh));
        mesh->vertices = {{{0, 0, 0}}, {{1, 0, 0}}, {{0, 1, 0}}};
        mesh->fill_times = {0.0, 0.5, 1.0};
        mesh->fill_duration = 1.0;
        mesh->fields_visible = true;
        mesh->pressure_visible = false;
        QVERIFY(panta::visualization::playback_affects_surface(mesh));
        mesh->pressure_visible = true;
        QVERIFY(!panta::visualization::playback_affects_surface(mesh));
        mesh->pressure_visible = false;
        mesh->fields_visible = false;
        QVERIFY(!panta::visualization::playback_affects_surface(mesh));
        mesh->fields_visible = true;
        mesh->fill_times.pop_back();
        QVERIFY(!panta::visualization::playback_affects_surface(mesh));
        QVERIFY(!panta::visualization::playback_affects_surface(nullptr));
    }
    void toggles_gate_markers_with_snapshot_flags() {
        vtkNew<vtkActor> actor;
        panta::visualization::replace_gate_geometry(actor, nullptr, {});
        QCOMPARE(actor->GetVisibility(), 0);
        const auto mesh = std::make_shared<panta::visualization::SurfaceMeshSnapshot>();
        mesh->vertices = {{{0, 0, 0}}, {{10, 0, 0}}, {{0, 20, 0}}};
        mesh->gate_points = {{{1.0, 2.0, 0.0}}};
        mesh->show_gates = true;
        panta::visualization::replace_gate_geometry(actor, nullptr, mesh);
        QCOMPARE(actor->GetVisibility(), 1);
        auto* mapper = vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
        QVERIFY(mapper != nullptr);
        auto* markers = mapper->GetInput();
        QVERIFY(markers != nullptr);
        QVERIFY(markers->GetNumberOfPolys() > 0);
        double bounds[6];
        markers->GetBounds(bounds);
        // 球形标记半径 2.5 mm，包围盒中心对准推荐浇口点。球源按 float 计算，且
        // phi 采样不含赤道，横向宽度略小于直径，只约束中心与量级。
        QVERIFY(std::abs((bounds[0] + bounds[1]) / 2 - 1.0) < 1e-6);
        QVERIFY(std::abs((bounds[2] + bounds[3]) / 2 - 2.0) < 1e-6);
        QVERIFY(bounds[1] - bounds[0] > 4.9 && bounds[1] - bounds[0] <= 5.0 + 1e-6);
        QCOMPARE(actor->GetProperty()->GetColor()[0], 215.0 / 255.0);
        mesh->show_gates = false;
        panta::visualization::replace_gate_geometry(actor, nullptr, mesh);
        QCOMPARE(actor->GetVisibility(), 0);
    }
};
QTEST_GUILESS_MAIN(PrimitiveSceneTest)
#include "primitive_scene_test.moc"
