#include "welcome_scene.hpp"

#include <QByteArray>
#include <QCoreApplication>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <map>
#include <utility>
#include <vector>
#include <vtkBillboardTextActor3D.h>
#include <vtkCamera.h>
#include <vtkCellArray.h>
#include <vtkNew.h>
#include <vtkPointData.h>
#include <vtkPoints.h>
#include <vtkPolyData.h>
#include <vtkRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkTextProperty.h>
#include <vtkType.h>
#include <vtkUnsignedCharArray.h>
#include <vtkVectorText.h>

namespace panta::visualization {
namespace {

constexpr int kWelcomeLabelFontSize = 50;
constexpr double kWelcomeLabelVerticalPosition = -0.5;

using Triangle = std::array<vtkIdType, 3>;
using Edge = std::pair<vtkIdType, vtkIdType>;
using Point = std::array<double, 3>;

struct EdgeUse {
    Edge direction;
    int count = 0;
};

// 品牌渐变沿字形从冷色过渡到暖色，不表示物理量或求解结果。
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

vtkSmartPointer<vtkPolyData> create_welcome_wordmark() {
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
    vertices.reserve(flat->GetNumberOfPoints());
    for (vtkIdType i = 0; i < flat->GetNumberOfPoints(); ++i) {
        Point point{};
        flat->GetPoint(i, point.data());
        point[0] -= center_x;
        point[1] -= center_y;
        vertices.push_back(point);
    }
    std::vector<Triangle> triangles;
    triangles.reserve(flat->GetNumberOfPolys());
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
        refined.reserve(4 * triangles.size());
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
    const auto offset = static_cast<vtkIdType>(vertices.size());
    points->SetNumberOfPoints(2 * offset);
    colors->SetNumberOfTuples(2 * offset);
    constexpr double half_depth = 0.11;
    for (vtkIdType i = 0; i < offset; ++i) {
        const auto& vertex = vertices[i];
        points->SetPoint(i, vertex[0], vertex[1], -half_depth);
        points->SetPoint(i + offset, vertex[0], vertex[1], half_depth);
        const auto color = wordmark_color(vertex[0] / width + 0.5, vertex[1] / height + 0.5);
        colors->SetTypedTuple(i, color.data());
        colors->SetTypedTuple(i + offset, color.data());
    }
    vtkNew<vtkCellArray> faces;
    // 每条边只需计数与方向，不为内部三角边分配动态列表。
    std::map<Edge, EdgeUse> edges;
    for (const auto& triangle : triangles) {
        const auto a = triangle[0], b = triangle[1], c = triangle[2];
        const Triangle back{c, b, a};
        const Triangle front{a + offset, b + offset, c + offset};
        faces->InsertNextCell(3, back.data());
        faces->InsertNextCell(3, front.data());
        for (const Edge edge : {Edge{a, b}, Edge{b, c}, Edge{c, a}}) {
            auto& use = edges[std::minmax(edge.first, edge.second)];
            use.direction = edge;
            ++use.count;
        }
    }
    // 只封闭轮廓边，字孔的内壁自然保留；内部三角边不能挤出成墙。
    for (const auto& [key, use] : edges) {
        if (use.count != 1) {
            continue;
        }
        const auto [a, b] = use.direction;
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

WelcomeScene::~WelcomeScene() { detach(); }

void WelcomeScene::attach(vtkRenderWindow* render_window) {
    if (render_window == nullptr || renderer_ != nullptr) {
        return;
    }
    render_window_ = render_window;
    if (render_window_->GetNumberOfLayers() < 2) {
        render_window_->SetNumberOfLayers(2);
    }

    renderer_ = vtkSmartPointer<vtkRenderer>::New();
    renderer_->SetLayer(1);
    renderer_->SetErase(false);
    renderer_->InteractiveOff();
    renderer_->SetViewport(0.0, 0.0, 1.0, 1.0);
    auto* camera = renderer_->GetActiveCamera();
    camera->SetPosition(0.0, 0.0, 10.0);
    camera->SetFocalPoint(0.0, 0.0, 0.0);
    camera->SetViewUp(0.0, 1.0, 0.0);
    camera->SetParallelProjection(true);
    camera->SetParallelScale(1.0);

    text_actor_ = vtkSmartPointer<vtkBillboardTextActor3D>::New();
    const QByteArray text = QCoreApplication::translate("ViewportWelcome", "Welcome!").toUtf8();
    text_actor_->SetInput(text.constData());
    // 第二层使用独立的平行相机，文字固定在视口下方且不参与场景拾取。
    text_actor_->SetPosition(0.0, kWelcomeLabelVerticalPosition, 0.0);
    text_actor_->PickableOff();
    vtkNew<vtkTextProperty> property;
    property->SetFontFamilyToArial();
    property->SetFontSize(kWelcomeLabelFontSize);
    property->SetColor(0.34, 0.34, 0.34);
    property->SetJustificationToCentered();
    property->SetVerticalJustificationToCentered();
    text_actor_->SetTextProperty(property);
    text_actor_->ForceOpaqueOn();
    renderer_->AddActor(text_actor_);
    render_window_->AddRenderer(renderer_);
}

void WelcomeScene::detach() {
    if (render_window_ != nullptr && renderer_ != nullptr) {
        render_window_->RemoveRenderer(renderer_);
    }
    text_actor_ = nullptr;
    renderer_ = nullptr;
    render_window_ = nullptr;
}

void WelcomeScene::set_visible(bool visible) {
    if (text_actor_ != nullptr) {
        text_actor_->SetVisibility(visible);
    }
}

} // namespace panta::visualization
