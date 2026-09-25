// 固定规模的 native→CXX→Rust 校验耗时；手动运行，不作为 CI 时间门禁。
#include <algorithm>
#include <array>
#include <chrono>
#include <cstddef>
#include <exception>
#include <iostream>
#include <panta/mesh/mesh_ir.hpp>
#include <ratio>

// 函数级 try 已 catch (const std::exception&) 全量兜底；MSVC STL 的
// _Throw_bad_array_new_length 是间接抛出，bugprone-exception-escape 无法
// 建模其被捕获，换任何写法均误报，此处显式豁免。
int main() try {  // NOLINT(bugprone-exception-escape)
    constexpr std::size_t kCount = 10'000;
    constexpr std::size_t kSamples = 21;
    panta::mesh::NativeTetMeshDto mesh;
    mesh.nodes.reserve(kCount * 4);
    mesh.tets.reserve(kCount);
    mesh.boundary.reserve(kCount);
    mesh.region_count = 1;
    mesh.boundary_group_count = 1;
    for (std::size_t index = 0; index < kCount; ++index) {
        const double origin = static_cast<double>(index) * 2.0;
        const auto node = static_cast<panta::mesh::MeshIndex>(index * 4);
        mesh.nodes.push_back({origin, 0.0, 0.0});
        mesh.nodes.push_back({origin + 1.0, 0.0, 0.0});
        mesh.nodes.push_back({origin, 1.0, 0.0});
        mesh.nodes.push_back({origin, 0.0, 1.0});
        mesh.tets.push_back({{node, node + 1, node + 2, node + 3}, 0});
        mesh.boundary.push_back({{node, node + 1, node + 2}, 0});
    }
    std::array<double, kSamples> elapsed_ms{};
    for (double& sample : elapsed_ms) {
        const auto started = std::chrono::steady_clock::now();
        const auto report = panta::mesh::validate_native_mesh_with_rust(mesh);
        sample =
            std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - started)
                .count();
        if (!report.ok || report.volume_mm3 <= 0.0) {
            std::cerr << "generated mesh failed validation\n";
            return 1;
        }
    }
    std::sort(elapsed_ms.begin(), elapsed_ms.end());
    std::cout << "tetrahedra=" << kCount << " nodes=" << mesh.nodes.size()
              << " boundary=" << mesh.boundary.size()
              << " native_to_rust_median_ms=" << elapsed_ms[kSamples / 2] << '\n';
} catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
}
