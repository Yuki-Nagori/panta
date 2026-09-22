#include "stl_mesh.hpp"

#include <QByteArray>
#include <QFile>
#include <QIODevice>
#include <QList>
#include <QtCore/qnamespace.h>
#include <QtEndian>
#include <array>
#include <cmath>
#include <cstring>
#include <vtkCellArray.h>
#include <vtkNew.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkType.h>

namespace panta::visualization {

namespace {

struct Triangle {
    std::array<std::array<double, 3>, 3> vertices{};
};

void set_error(QString* error, const QString& message) {
    if (error != nullptr) {
        *error = message;
    }
}

vtkSmartPointer<vtkPolyData> make_poly_data(const QList<Triangle>& triangles) {
    auto points = vtkSmartPointer<vtkPoints>::New();
    auto cells = vtkSmartPointer<vtkCellArray>::New();
    for (const auto& triangle : triangles) {
        vtkIdType ids[3]{};
        for (int index = 0; index < 3; ++index) {
            ids[index] = points->InsertNextPoint(triangle.vertices[index].data());
        }
        cells->InsertNextCell(3, ids);
    }

    auto poly_data = vtkSmartPointer<vtkPolyData>::New();
    poly_data->SetPoints(points);
    poly_data->SetPolys(cells);
    return poly_data;
}

bool finite_triangle(const Triangle& triangle) {
    for (const auto& vertex : triangle.vertices) {
        for (const double value : vertex) {
            if (!std::isfinite(value)) {
                return false;
            }
        }
    }
    return true;
}

bool read_binary(const QByteArray& bytes, QList<Triangle>* triangles) {
    if (bytes.size() < 84) {
        return false;
    }
    const auto* raw = reinterpret_cast<const uchar*>(bytes.constData());
    const quint32 count = qFromLittleEndian<quint32>(raw + 80);
    const qsizetype expectedSize = 84 + static_cast<qsizetype>(count) * 50;
    if (count == 0 || expectedSize != bytes.size()) {
        return false;
    }

    triangles->reserve(static_cast<qsizetype>(count));
    for (quint32 triangleIndex = 0; triangleIndex < count; ++triangleIndex) {
        const qsizetype offset = 84 + static_cast<qsizetype>(triangleIndex) * 50 + 12;
        Triangle triangle;
        for (int vertexIndex = 0; vertexIndex < 3; ++vertexIndex) {
            const auto* vertex = raw + offset + static_cast<qsizetype>(vertexIndex) * 12;
            for (int axis = 0; axis < 3; ++axis) {
                const quint32 bits =
                    qFromLittleEndian<quint32>(vertex + static_cast<qsizetype>(axis) * 4);
                float value = 0.0F;
                std::memcpy(&value, &bits, sizeof(value));
                triangle.vertices[vertexIndex][axis] = static_cast<double>(value);
            }
        }
        if (!finite_triangle(triangle)) {
            return false;
        }
        triangles->append(triangle);
    }
    return true;
}

bool read_ascii(const QByteArray& bytes, QList<Triangle>* triangles) {
    const QString text = QString::fromUtf8(bytes);
    if (text.isEmpty()) {
        return false;
    }

    Triangle triangle;
    int vertexCount = 0;
    for (const QString& line : text.split(QLatin1Char('\n'))) {
        const auto fields = line.simplified().split(QLatin1Char(' '), Qt::SkipEmptyParts);
        if (fields.size() != 4 ||
            fields.at(0).compare(QStringLiteral("vertex"), Qt::CaseInsensitive) != 0) {
            continue;
        }
        bool ok = true;
        std::array<double, 3> vertex{};
        for (int axis = 0; axis < 3; ++axis) {
            vertex[axis] = fields.at(axis + 1).toDouble(&ok);
            if (!ok) {
                return false;
            }
        }
        triangle.vertices[vertexCount++] = vertex;
        if (vertexCount == 3) {
            if (!finite_triangle(triangle)) {
                return false;
            }
            triangles->append(triangle);
            triangle = {};
            vertexCount = 0;
        }
    }
    return !triangles->isEmpty() && vertexCount == 0;
}

} // namespace

vtkSmartPointer<vtkPolyData> load_stl_mesh(const QString& path, QString* error) {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly)) {
        set_error(error, QStringLiteral("could not open STL file"));
        return nullptr;
    }
    const QByteArray bytes = file.readAll();
    QList<Triangle> triangles;
    if (!read_binary(bytes, &triangles) && !read_ascii(bytes, &triangles)) {
        set_error(error, QStringLiteral("invalid STL data"));
        return nullptr;
    }
    return make_poly_data(triangles);
}

} // namespace panta::visualization
