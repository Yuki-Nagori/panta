// 欢迎字样的 CPU 几何验证：封闭实体、字孔拓扑和有效顶点色不依赖 GPU。
#include "default_wordmark.hpp"
#include "stl_mesh.hpp"
#include <QFile>
#include <QIODevice>
#include <QTemporaryDir>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <map>
#include <utility>
#include <vtkCellArray.h>
#include <vtkDataArray.h>
#include <vtkPointData.h>
#include <vtkPolyData.h>
#include <vtkSmartPointer.h>
#include <vtkType.h>

class DefaultWordmarkTest final : public QObject {
    Q_OBJECT

  private slots:
    void creates_closed_colored_letters() {
        const auto mesh = panta::visualization::create_default_wordmark();
        QVERIFY(mesh->GetNumberOfPoints() > 0);
        double bounds[6];
        mesh->GetBounds(bounds);
        QVERIFY(bounds[1] - bounds[0] > 2 * (bounds[3] - bounds[2]));
        QVERIFY(bounds[5] - bounds[4] > 0.3);
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

    void reads_ascii_stl_as_triangle_poly_data() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        const QString path = fixture.filePath(QStringLiteral("sample.stl"));
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly | QIODevice::Text));
        QVERIFY(file.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n") > 0);
        file.close();

        QString error;
        const auto mesh = panta::visualization::load_stl_mesh(path, &error);
        QVERIFY2(mesh != nullptr, qPrintable(error));
        QCOMPARE(mesh->GetNumberOfPoints(), vtkIdType(3));
        QCOMPARE(mesh->GetNumberOfPolys(), vtkIdType(1));
    }
};

QTEST_APPLESS_MAIN(DefaultWordmarkTest)
#include "default_wordmark_test.moc"
