// 欢迎字样的 CPU 几何验证：封闭实体、字孔拓扑和有效顶点色不依赖 GPU。
#include "navigation/viewport_orientation.hpp"
#include "panta/visualization/mesh_source.hpp"
#include "surface_mesh.hpp"
#include "welcome/welcome_scene.hpp"
#include <QFile>
#include <QIODevice>
#include <QTemporaryDir>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <map>
#include <qobject.h>
#include <set>
#include <string>
#include <utility>
#include <vtkActor.h>
#include <vtkAlgorithmOutput.h>
#include <vtkCamera.h>
#include <vtkCellArray.h>
#include <vtkDataArray.h>
#include <vtkMatrix4x4.h>
#include <vtkNew.h>
#include <vtkPointData.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkPolyDataMapper.h>
#include <vtkPropCollection.h>
#include <vtkRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkRendererCollection.h>
#include <vtkSmartPointer.h>
#include <vtkTransformFilter.h>
#include <vtkType.h>
#include <vtkVectorText.h>

class WelcomeWordmarkTest final : public QObject {
    Q_OBJECT

  private slots:
    void creates_closed_colored_letters() {
        const auto mesh = panta::visualization::create_welcome_wordmark();
        QVERIFY(mesh->GetNumberOfPoints() > 0);
        double bounds[6];
        mesh->GetBounds(bounds);
        QVERIFY(bounds[1] - bounds[0] > 2 * (bounds[3] - bounds[2]));
        QVERIFY(bounds[5] - bounds[4] > 0.2);
        QVERIFY(std::abs((bounds[5] - bounds[4]) - 0.22) < 1e-6);
        for (std::size_t axis = 0; axis < 3; ++axis) {
            QVERIFY(std::abs(bounds[2 * axis] + bounds[2 * axis + 1]) < 1e-5);
        }
        auto* colors = mesh->GetPointData()->GetScalars();
        QVERIFY(colors != nullptr);
        QCOMPARE(colors->GetNumberOfComponents(), 3);
        QCOMPARE(colors->GetNumberOfTuples(), mesh->GetNumberOfPoints());
        int blue = 0, green = 0, warm = 0;
        for (vtkIdType i = 0; i < colors->GetNumberOfTuples(); ++i) {
            double rgb[3];
            colors->GetTuple(i, rgb);
            for (const double channel : rgb) {
                QVERIFY(std::isfinite(channel) && channel >= 0 && channel <= 255);
            }
            blue += rgb[2] > rgb[0] * 1.5 && rgb[2] > rgb[1];
            green += rgb[1] > rgb[0] && rgb[1] > rgb[2];
            warm += rgb[0] > rgb[1] && rgb[0] > rgb[2] * 1.5;
        }
        QVERIFY(blue > 0 && green > 0 && warm > 0);

        using Edge = std::pair<vtkIdType, vtkIdType>;
        std::map<Edge, std::pair<int, int>> edges;
        vtkIdType count = 0;
        const vtkIdType* ids = nullptr;
        double volume = 0;
        mesh->GetPolys()->InitTraversal();
        while (mesh->GetPolys()->GetNextCell(count, ids)) {
            QCOMPARE(count, 3);
            double a[3], b[3], c[3];
            mesh->GetPoint(ids[0], a);
            mesh->GetPoint(ids[1], b);
            mesh->GetPoint(ids[2], c);
            volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) + a[1] * (b[2] * c[0] - b[0] * c[2]) +
                       a[2] * (b[0] * c[1] - b[1] * c[0])) /
                      6;
            for (int i = 0; i < 3; ++i) {
                const auto from = ids[i], to = ids[(i + 1) % 3];
                QVERIFY(from != to);
                auto& [uses, direction] = edges[std::minmax(from, to)];
                ++uses;
                direction += from < to ? 1 : -1;
            }
        }
        for (const auto& [edge, usage] : edges) {
            QCOMPARE(usage.first, 2);
            QCOMPARE(usage.second, 0);
        }
        QVERIFY(volume > 0);
        // 五个独立字母，其中 p/a/a 各一个贯通孔，Euler 特征为 2*5 - 2*3。
        QCOMPARE(mesh->GetNumberOfPoints() - static_cast<vtkIdType>(edges.size()) +
                     mesh->GetNumberOfPolys(),
                 4);
    }

    void converts_surface_snapshot_to_poly_data() {
        panta::visualization::SurfaceMeshSnapshot snapshot;
        snapshot.vertices = {{{0.0, 0.0, 0.0}}, {{1.0, 0.0, 0.0}}, {{0.0, 1.0, 0.0}}};
        const auto mesh = panta::visualization::make_surface_poly_data(snapshot);
        QCOMPARE(mesh->GetNumberOfPoints(), vtkIdType(3));
        QCOMPARE(mesh->GetNumberOfPolys(), vtkIdType(1));
        QCOMPARE(mesh->GetPoints()->GetDataType(), VTK_DOUBLE);
    }

    void ignores_detached_and_invalid_cube_picks() {
        panta::visualization::ViewportOrientation orientation;
        QVERIFY(!orientation.cube_direction(940, 704, 1000, 800).has_value());
        QVERIFY(!orientation.contains_cube(600, 600, 1000, 800));
        QVERIFY(!orientation.contains_cube(0, 0, 0, 0));
    }

    void picks_projected_cube_faces() {
        panta::visualization::ViewportOrientation orientation;
        vtkNew<vtkRenderWindow> render_window;
        constexpr int width = 1000;
        constexpr int height = 800;
        render_window->SetSize(width, height);
        orientation.attach(render_window);

        vtkRenderer* cube_renderer = nullptr;
        auto* renderers = render_window->GetRenderers();
        renderers->InitTraversal();
        while (auto* renderer = vtkRenderer::SafeDownCast(renderers->GetNextItem())) {
            const double* viewport = renderer->GetViewport();
            if (viewport[0] > 0.89 && viewport[1] > 0.75) {
                cube_renderer = renderer;
                break;
            }
        }
        if (cube_renderer == nullptr) {
            QFAIL("orientation cube renderer was not created");
            return;
        }

        struct FaceView {
            panta::visualization::CubeDirection direction;
            std::array<double, 3> camera_position;
            std::array<double, 3> view_up;
            std::array<double, 3> label_position;
        };
        const std::array views = {
            FaceView{panta::visualization::CubeDirection::PositiveX,
                     {5.0, 0.0, 0.0},
                     {0.0, 0.0, 1.0},
                     {0.83, 0.0, 0.0}},
            FaceView{panta::visualization::CubeDirection::NegativeX,
                     {-5.0, 0.0, 0.0},
                     {0.0, 0.0, 1.0},
                     {-0.83, 0.0, 0.0}},
            FaceView{panta::visualization::CubeDirection::PositiveY,
                     {0.0, 5.0, 0.0},
                     {0.0, 0.0, 1.0},
                     {0.0, 0.83, 0.0}},
            FaceView{panta::visualization::CubeDirection::NegativeY,
                     {0.0, -5.0, 0.0},
                     {0.0, 0.0, 1.0},
                     {0.0, -0.83, 0.0}},
            FaceView{panta::visualization::CubeDirection::PositiveZ,
                     {0.0, 0.0, 5.0},
                     {0.0, 1.0, 0.0},
                     {0.0, 0.0, 0.83}},
            FaceView{panta::visualization::CubeDirection::NegativeZ,
                     {0.0, 0.0, -5.0},
                     {0.0, 1.0, 0.0},
                     {0.0, 0.0, -0.83}},
        };
        for (const auto& view : views) {
            vtkNew<vtkCamera> camera;
            camera->SetPosition(view.camera_position.data());
            camera->SetFocalPoint(0.0, 0.0, 0.0);
            camera->SetViewUp(view.view_up.data());
            orientation.update(camera);

            cube_renderer->SetWorldPoint(view.label_position[0], view.label_position[1],
                                         view.label_position[2], 1.0);
            cube_renderer->WorldToDisplay();
            const double* display = cube_renderer->GetDisplayPoint();
            const auto picked = orientation.cube_direction(
                static_cast<int>(std::lround(display[0])),
                static_cast<int>(std::lround(display[1])), width, height);
            QVERIFY(picked.has_value());
            QCOMPARE(static_cast<int>(picked.value_or(view.direction)),
                     static_cast<int>(view.direction));
        }
    }

    void tolerates_orientation_ablation_path() {
        panta::visualization::ViewportOrientation orientation;
        vtkNew<vtkCamera> camera;
        orientation.update(nullptr);
        orientation.update(camera);
        vtkNew<vtkRenderWindow> render_window;
        orientation.attach(render_window);
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 2);
        orientation.detach();
        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 0);
        orientation.detach();
        QVERIFY(!orientation.cube_direction(940, 704, 1000, 800).has_value());
    }

    void creates_axis_and_direction_labels() {
        panta::visualization::ViewportOrientation orientation;
        vtkNew<vtkRenderWindow> render_window;
        orientation.attach(render_window);

        QCOMPARE(render_window->GetRenderers()->GetNumberOfItems(), 2);
        int text_label_count = 0;
        std::set<std::string> labels;
        std::map<std::string, std::array<double, 3>> centers;
        std::map<std::string, std::array<double, 9>> bases;
        auto* renderers = render_window->GetRenderers();
        renderers->InitTraversal();
        while (auto* renderer = vtkRenderer::SafeDownCast(renderers->GetNextItem())) {
            auto* props = renderer->GetViewProps();
            props->InitTraversal();
            while (auto* prop = props->GetNextProp()) {
                auto* actor = vtkActor::SafeDownCast(prop);
                auto* mapper = actor == nullptr
                                   ? nullptr
                                   : vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
                auto* connection = mapper == nullptr ? nullptr : mapper->GetInputConnection(0, 0);
                auto* geometry = connection == nullptr
                                     ? nullptr
                                     : vtkTransformFilter::SafeDownCast(connection->GetProducer());
                auto* source_connection =
                    geometry == nullptr ? connection : geometry->GetInputConnection(0, 0);
                auto* label = source_connection == nullptr
                                  ? nullptr
                                  : vtkVectorText::SafeDownCast(source_connection->GetProducer());
                if (label == nullptr) {
                    continue;
                }
                ++text_label_count;
                labels.emplace(label->GetText());
                const double* center = actor->GetCenter();
                centers.emplace(label->GetText(), std::array{center[0], center[1], center[2]});
                const auto* matrix = actor->GetUserMatrix();
                QVERIFY(matrix != nullptr);
                std::array<double, 9> basis{};
                for (int row = 0; row < 3; ++row) {
                    for (int column = 0; column < 3; ++column) {
                        basis[static_cast<std::size_t>(row) * 3U +
                              static_cast<std::size_t>(column)] = matrix->GetElement(row, column);
                    }
                }
                bases.emplace(label->GetText(), basis);
            }
        }
        QCOMPARE(text_label_count, 6);
        const std::set<std::string> expected = {"BACK", "BOTTOM", "FRONT", "LEFT", "RIGHT", "TOP"};
        QVERIFY(labels == expected);
        const std::map<std::string, std::array<double, 3>> expected_centers = {
            {"RIGHT", {0.83, 0.0, 0.0}},   {"LEFT", {-0.83, 0.0, 0.0}}, {"TOP", {0.0, 0.83, 0.0}},
            {"BOTTOM", {0.0, -0.83, 0.0}}, {"FRONT", {0.0, 0.0, 0.83}}, {"BACK", {0.0, 0.0, -0.83}},
        };
        for (const auto& [label, expected_center] : expected_centers) {
            QVERIFY(centers.contains(label));
            for (std::size_t axis = 0; axis < expected_center.size(); ++axis) {
                const double actual = centers.at(label)[axis];
                const QString detail = QStringLiteral("%1 axis %2: actual %3, expected %4")
                                           .arg(QString::fromStdString(label))
                                           .arg(static_cast<int>(axis))
                                           .arg(actual)
                                           .arg(expected_center[axis]);
                QVERIFY2(std::abs(actual - expected_center[axis]) < 1e-5, qPrintable(detail));
            }
        }
        const std::map<std::string, std::array<double, 9>> expected_bases = {
            {"RIGHT", {0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0}},
            {"LEFT", {0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0}},
            {"TOP", {-1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0}},
            {"BOTTOM", {1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0}},
            {"FRONT", {1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0}},
            {"BACK", {-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0}},
        };
        for (const auto& [label, expected_basis] : expected_bases) {
            QVERIFY(bases.contains(label));
            for (std::size_t element = 0; element < expected_basis.size(); ++element) {
                QVERIFY(std::abs(bases.at(label)[element] - expected_basis[element]) < 1e-5);
            }
        }

        vtkNew<vtkCamera> camera;
        camera->SetPosition(0.0, 0.0, 5.0);
        camera->SetFocalPoint(0.0, 0.0, 0.0);
        camera->SetViewUp(0.0, 1.0, 0.0);
        orientation.update(camera);

        int visible_label_count = 0;
        std::string visible_label;
        renderers->InitTraversal();
        while (auto* renderer = vtkRenderer::SafeDownCast(renderers->GetNextItem())) {
            auto* props = renderer->GetViewProps();
            props->InitTraversal();
            while (auto* prop = props->GetNextProp()) {
                auto* actor = vtkActor::SafeDownCast(prop);
                auto* mapper = actor == nullptr
                                   ? nullptr
                                   : vtkPolyDataMapper::SafeDownCast(actor->GetMapper());
                auto* connection = mapper == nullptr ? nullptr : mapper->GetInputConnection(0, 0);
                auto* geometry = connection == nullptr
                                     ? nullptr
                                     : vtkTransformFilter::SafeDownCast(connection->GetProducer());
                auto* source_connection =
                    geometry == nullptr ? connection : geometry->GetInputConnection(0, 0);
                auto* label = source_connection == nullptr
                                  ? nullptr
                                  : vtkVectorText::SafeDownCast(source_connection->GetProducer());
                if (label != nullptr && actor->GetVisibility()) {
                    ++visible_label_count;
                    visible_label = label->GetText();
                }
            }
        }
        QCOMPARE(visible_label_count, 1);
        QCOMPARE(visible_label, std::string("FRONT"));
    }
};

QTEST_APPLESS_MAIN(WelcomeWordmarkTest)
#include "welcome_wordmark_test.moc"
