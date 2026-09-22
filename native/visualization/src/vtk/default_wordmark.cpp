#include "default_wordmark.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <map>
#include <utility>
#include <vector>
#include <vtkCellArray.h>
#include <vtkNew.h>
#include <vtkPointData.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkType.h>
#include <vtkUnsignedCharArray.h>
#include <vtkVectorText.h>

namespace panta::visualization {
namespace {

using Triangle = std::array<vtkIdType, 3>;
using Edge = std::pair<vtkIdType, vtkIdType>;
using Point = std::array<double, 3>;

// 欢迎图形的装饰色带，借用注塑云图的冷暖顺序；不映射物理量。
std::array<unsigned char, 3> wordmark_color(double x, double y) {
    constexpr std::array<Point, 6> palette{{{24, 42, 146},
                                            {16, 132, 207},
                                            {22, 174, 135},
                                            {91, 193, 57},
                                            {241, 207, 45},
                                            {220, 53, 36}}};
    const double value =
        std::clamp(0.12 + 0.74 * x + 0.13 * std::sin(8.0 * x + 3.0 * y) + 0.12 * y, 0.0, 1.0);
    const double index = value * static_cast<double>(palette.size() - 1);
    const auto low = static_cast<std::size_t>(index);
    const auto high = std::min(low + 1, palette.size() - 1);
    const double blend = index - static_cast<double>(low);
    std::array<unsigned char, 3> color{};
    for (std::size_t channel = 0; channel < color.size(); ++channel) {
        color[channel] = static_cast<unsigned char>(
            std::lround(palette[low][channel] * (1 - blend) + palette[high][channel] * blend));
    }
    return color;
}

} // namespace

vtkSmartPointer<vtkPolyData> create_default_wordmark() {
    vtkNew<vtkVectorText> source;
    source->SetText("panta");
    source->Update();
    auto* flat = source->GetOutput();
    double bounds[6];
    flat->GetBounds(bounds);
    const double center_x = (bounds[0] + bounds[1]) / 2;
    const double center_y = (bounds[2] + bounds[3]) / 2;
    const double width = bounds[1] - bounds[0];
    const double height = bounds[3] - bounds[2];
    std::vector<Point> vertices;
    for (vtkIdType i = 0; i < flat->GetNumberOfPoints(); ++i) {
        Point point{};
        flat->GetPoint(i, point.data());
        point[0] -= center_x;
        point[1] -= center_y;
        vertices.push_back(point);
    }
    std::vector<Triangle> triangles;
    vtkIdType count = 0;
    const vtkIdType* ids = nullptr;
    flat->GetPolys()->InitTraversal();
    while (flat->GetPolys()->GetNextCell(count, ids)) {
        // vtkVectorText 的输出固定为三角形，统一正面朝 +Z。
        Triangle triangle{ids[0], ids[1], ids[2]};
        const auto& a = vertices[triangle[0]];
        const auto& b = vertices[triangle[1]];
        const auto& c = vertices[triangle[2]];
        if ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) < 0) {
            std::swap(triangle[1], triangle[2]);
        }
        triangles.push_back(triangle);
    }
    // 共享边中点细分，保持字孔拓扑并给色带足够采样点；无需额外建模 SDK。
    for (int pass = 0; pass < 2; ++pass) {
        std::map<Edge, vtkIdType> midpoints;
        const auto midpoint = [&](vtkIdType a, vtkIdType b) {
            const Edge edge = std::minmax(a, b);
            const auto found = midpoints.find(edge);
            if (found != midpoints.end()) {
                return found->second;
            }
            const auto id = static_cast<vtkIdType>(vertices.size());
            const Point point{(vertices[a][0] + vertices[b][0]) / 2,
                              (vertices[a][1] + vertices[b][1]) / 2, 0};
            vertices.push_back(point);
            midpoints.emplace(edge, id);
            return id;
        };
        std::vector<Triangle> refined;
        for (const auto& triangle : triangles) {
            const auto a = triangle[0], b = triangle[1], c = triangle[2];
            const auto ab = midpoint(a, b), bc = midpoint(b, c), ca = midpoint(c, a);
            refined.insert(refined.end(), {{a, ab, ca}, {ab, b, bc}, {ca, bc, c}, {ab, bc, ca}});
        }
        triangles = std::move(refined);
    }

    vtkNew<vtkPoints> points;
    vtkNew<vtkUnsignedCharArray> colors;
    colors->SetName("WelcomeColor");
    colors->SetNumberOfComponents(3);
    constexpr double half_depth = 0.16;
    for (const double z : {-half_depth, half_depth}) {
        for (const auto& vertex : vertices) {
            points->InsertNextPoint(vertex[0], vertex[1], z);
            const auto color = wordmark_color(vertex[0] / width + 0.5, vertex[1] / height + 0.5);
            colors->InsertNextTypedTuple(color.data());
        }
    }
    const auto offset = static_cast<vtkIdType>(vertices.size());
    vtkNew<vtkCellArray> faces;
    std::map<Edge, std::vector<Edge>> edges;
    for (const auto& triangle : triangles) {
        const auto a = triangle[0], b = triangle[1], c = triangle[2];
        const Triangle back{c, b, a};
        const Triangle front{a + offset, b + offset, c + offset};
        faces->InsertNextCell(3, back.data());
        faces->InsertNextCell(3, front.data());
        for (const Edge edge : {Edge{a, b}, Edge{b, c}, Edge{c, a}}) {
            edges[std::minmax(edge.first, edge.second)].push_back(edge);
        }
    }
    // 只封闭轮廓边，字孔的内壁自然保留；内部三角边不能挤出成墙。
    for (const auto& [key, uses] : edges) {
        if (uses.size() != 1) {
            continue;
        }
        const auto [a, b] = uses.front();
        const Triangle side_a{a, b, b + offset};
        const Triangle side_b{a, b + offset, a + offset};
        faces->InsertNextCell(3, side_a.data());
        faces->InsertNextCell(3, side_b.data());
    }
    auto mesh = vtkSmartPointer<vtkPolyData>::New();
    mesh->SetPoints(points);
    mesh->SetPolys(faces);
    mesh->GetPointData()->SetScalars(colors);
    return mesh;
}

} // namespace panta::visualization
