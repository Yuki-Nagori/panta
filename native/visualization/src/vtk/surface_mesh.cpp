#include "surface_mesh.hpp"

#include "mesh_source.hpp"
#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <vector>
#include <vtkCellArray.h>
#include <vtkFloatArray.h>
#include <vtkMath.h>
#include <vtkPointData.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkSmartPointer.h>
#include <vtkType.h>
#include <vtkUnsignedCharArray.h>

namespace panta::visualization {

vtkSmartPointer<vtkPolyData> make_surface_poly_data(const SurfaceMeshSnapshot& mesh) {
    const auto triangle_count = static_cast<vtkIdType>(mesh.vertices.size() / 3);
    const auto point_count = triangle_count * 3;
    auto points = vtkSmartPointer<vtkPoints>::New();
    points->SetDataTypeToDouble();
    points->SetNumberOfPoints(point_count);
    auto cells = vtkSmartPointer<vtkCellArray>::New();
    cells->AllocateExact(triangle_count, point_count);
    auto normals = vtkSmartPointer<vtkFloatArray>::New();
    normals->SetNumberOfComponents(3);
    normals->SetNumberOfTuples(point_count);
    for (vtkIdType triangle = 0; triangle < triangle_count; ++triangle) {
        const auto vertex = static_cast<std::size_t>(triangle) * 3;
        const auto& a = mesh.vertices[vertex];
        const auto& b = mesh.vertices[vertex + 1];
        const auto& c = mesh.vertices[vertex + 2];
        double ab[3]{b[0] - a[0], b[1] - a[1], b[2] - a[2]};
        double ac[3]{c[0] - a[0], c[1] - a[1], c[2] - a[2]};
        double normal[3]{};
        vtkMath::Cross(ab, ac, normal);
        const double length = std::hypot(normal[0], normal[1], normal[2]);
        if (length > 0.0 && std::isfinite(length)) {
            for (double& component : normal) {
                component /= length;
            }
        } else {
            for (double& component : normal) {
                component = 0.0;
            }
        }
        vtkIdType ids[3]{triangle * 3, triangle * 3 + 1, triangle * 3 + 2};
        for (std::size_t corner = 0; corner < 3; ++corner) {
            points->SetPoint(ids[corner], mesh.vertices[vertex + corner].data());
            normals->SetTuple3(ids[corner], normal[0], normal[1], normal[2]);
        }
        cells->InsertNextCell(3, ids);
    }
    auto data = vtkSmartPointer<vtkPolyData>::New();
    data->SetPoints(points);
    data->SetPolys(cells);
    // 当前三角面快照每面独占三个顶点，直接写面法线以避免重复三角化和拆点。
    data->GetPointData()->SetNormals(normals);
    return data;
}

namespace {
struct TimedVertex {
    std::array<double, 3> point;
    double time;
};

std::vector<TimedVertex> clip_time(const std::array<TimedVertex, 3>& triangle, double time,
                                   bool filled) {
    std::vector<TimedVertex> output;
    auto inside = [time, filled](double value) { return filled ? value <= time : value > time; };
    auto previous = triangle.back();
    for (const auto& current : triangle) {
        if (inside(previous.time) != inside(current.time)) {
            const double fraction = (time - previous.time) / (current.time - previous.time);
            TimedVertex intersection{{}, time};
            for (std::size_t axis = 0; axis < 3; ++axis) {
                intersection.point[axis] =
                    previous.point[axis] + fraction * (current.point[axis] - previous.point[axis]);
            }
            output.push_back(intersection);
        }
        if (inside(current.time)) {
            output.push_back(current);
        }
        previous = current;
    }
    return output;
}

std::array<unsigned char, 3> arrival_color(double fraction) {
    // 固定蓝→青→黄→红图例；与 QML 图例共享这些显示色值。
    constexpr std::array<std::array<double, 3>, 4> colors{
        {{35, 83, 210}, {20, 183, 204}, {249, 214, 66}, {215, 49, 46}}};
    const double scaled = std::clamp(fraction, 0.0, 1.0) * 3.0;
    const auto index = std::min(static_cast<std::size_t>(scaled), std::size_t{2});
    const double t = scaled - static_cast<double>(index);
    std::array<unsigned char, 3> color{};
    for (std::size_t axis = 0; axis < 3; ++axis) {
        color[axis] =
            static_cast<unsigned char>(colors[index][axis] * (1 - t) + colors[index + 1][axis] * t);
    }
    return color;
}
} // namespace

vtkSmartPointer<vtkPolyData> make_filling_poly_data(const SurfaceMeshSnapshot& mesh, double time) {
    if (mesh.fill_times.size() != mesh.vertices.size() || mesh.fill_duration <= 0) {
        return make_surface_poly_data(mesh);
    }
    SurfaceMeshSnapshot display;
    auto colors = vtkSmartPointer<vtkUnsignedCharArray>::New();
    colors->SetNumberOfComponents(3);
    colors->SetName("arrival_time_colors");
    auto append = [&](const std::vector<TimedVertex>& polygon, bool filled) {
        for (std::size_t corner = 1; corner + 1 < polygon.size(); ++corner) {
            for (const auto index : {std::size_t{0}, corner, corner + 1}) {
                const auto& vertex = polygon[index];
                display.vertices.push_back(vertex.point);
                const auto color = filled ? arrival_color(vertex.time / mesh.fill_duration)
                                          : std::array<unsigned char, 3>{205, 212, 220};
                colors->InsertNextTypedTuple(color.data());
            }
        }
    };
    for (std::size_t vertex = 0; vertex + 2 < mesh.vertices.size(); vertex += 3) {
        std::array<TimedVertex, 3> triangle{};
        bool finite = true;
        for (std::size_t corner = 0; corner < 3; ++corner) {
            triangle[corner] = {mesh.vertices[vertex + corner], mesh.fill_times[vertex + corner]};
            finite &= std::isfinite(triangle[corner].time);
        }
        // NaN 邻接面没有完整插值数据，保持灰色，不能补造充填时间。
        if (!finite) {
            append({triangle.begin(), triangle.end()}, false);
            continue;
        }
        append(clip_time(triangle, time, true), true);
        append(clip_time(triangle, time, false), false);
    }
    auto data = make_surface_poly_data(display);
    data->GetPointData()->SetScalars(colors);
    return data;
}

} // namespace panta::visualization
