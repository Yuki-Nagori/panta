/// Mesh IR 有效性检查（任务 010）。检查项与顺序即报告的确定性顺序；
/// 发现问题不抛异常、不截断，逐项记录后统一返回。
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <panta/mesh/mesh_ir.hpp>
#include <string>
#include <vector>

namespace panta::mesh {
namespace {

/// 有向体积 ×6：与公共头的节点顺序约定一致，正值即方向正确。
double tet_six_times_volume(const std::array<double, 3>& p0, const std::array<double, 3>& p1,
                            const std::array<double, 3>& p2, const std::array<double, 3>& p3) {
    const double u[3] = {p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]};
    const double v[3] = {p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]};
    const double w[3] = {p3[0] - p0[0], p3[1] - p0[1], p3[2] - p0[2]};
    const double cross[3] = {u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2],
                             u[0] * v[1] - u[1] * v[0]};
    return cross[0] * w[0] + cross[1] * w[1] + cross[2] * w[2];
}

} // namespace

MeshValidationReport validate_tet_mesh(const TetMesh& mesh) {
    MeshValidationReport report;

    if (mesh.nodes.empty())
        report.issues.push_back("mesh has no nodes");
    if (mesh.tets.empty())
        report.issues.push_back("mesh has no volume elements");

    for (std::size_t i = 0; i < mesh.nodes.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            if (!std::isfinite(mesh.nodes[i][axis])) {
                report.issues.push_back("node " + std::to_string(i) +
                                        " has a non-finite coordinate");
                break;
            }
        }
    }

    // 区域引用范围与完整性：引用必须落在声明数量内，且每个区域被引用。
    std::vector<bool> region_used(mesh.region_count, false);
    for (std::size_t i = 0; i < mesh.tets.size(); ++i) {
        const Tetrahedron& tet = mesh.tets[i];
        // 索引与重复检查先行：越界时不读取节点坐标，避免未定义行为。
        bool nodes_usable = true;
        for (const MeshIndex node : tet.nodes) {
            if (node >= mesh.nodes.size()) {
                report.issues.push_back("tetrahedron " + std::to_string(i) + " references node " +
                                        std::to_string(node) + " outside the node range");
                nodes_usable = false;
            }
        }
        if (tet.nodes[0] == tet.nodes[1] || tet.nodes[0] == tet.nodes[2] ||
            tet.nodes[0] == tet.nodes[3] || tet.nodes[1] == tet.nodes[2] ||
            tet.nodes[1] == tet.nodes[3] || tet.nodes[2] == tet.nodes[3]) {
            report.issues.push_back("tetrahedron " + std::to_string(i) + " repeats a node");
            nodes_usable = false;
        }
        if (nodes_usable) {
            const double six_volume =
                tet_six_times_volume(mesh.nodes[tet.nodes[0]], mesh.nodes[tet.nodes[1]],
                                     mesh.nodes[tet.nodes[2]], mesh.nodes[tet.nodes[3]]);
            if (six_volume <= 0.0) {
                report.issues.push_back("tetrahedron " + std::to_string(i) +
                                        " has non-positive signed volume");
            }
        }
        if (tet.region >= mesh.region_count) {
            report.issues.push_back("tetrahedron " + std::to_string(i) + " references region " +
                                    std::to_string(tet.region) + " outside the declared count");
        } else {
            region_used[tet.region] = true;
        }
    }
    for (std::uint32_t region = 0; region < mesh.region_count; ++region) {
        if (!region_used[region]) {
            report.issues.push_back("region " + std::to_string(region) + " is not referenced");
        }
    }

    std::vector<bool> group_used(mesh.boundary_group_count, false);
    for (std::size_t i = 0; i < mesh.boundary.size(); ++i) {
        const SurfaceTriangle& tri = mesh.boundary[i];
        for (const MeshIndex node : tri.nodes) {
            if (node >= mesh.nodes.size()) {
                report.issues.push_back("boundary triangle " + std::to_string(i) +
                                        " references node " + std::to_string(node) +
                                        " outside the node range");
                break;
            }
        }
        if (tri.nodes[0] == tri.nodes[1] || tri.nodes[0] == tri.nodes[2] ||
            tri.nodes[1] == tri.nodes[2]) {
            report.issues.push_back("boundary triangle " + std::to_string(i) + " repeats a node");
        }
        if (tri.group >= mesh.boundary_group_count) {
            report.issues.push_back("boundary triangle " + std::to_string(i) +
                                    " references group " + std::to_string(tri.group) +
                                    " outside the declared count");
        } else {
            group_used[tri.group] = true;
        }
    }
    for (std::uint32_t group = 0; group < mesh.boundary_group_count; ++group) {
        if (!group_used[group]) {
            report.issues.push_back("boundary group " + std::to_string(group) +
                                    " is not referenced");
        }
    }

    report.ok = report.issues.empty();
    return report;
}

} // namespace panta::mesh
