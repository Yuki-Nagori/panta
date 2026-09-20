/// 最小 Mesh IR（任务 010，architecture/mesh.md 的自有中间表示最小边界）。
///
/// 网格模块拥有独立于 Netgen/VTK 的数据边界：本头文件只含自有类型与
/// 标准库，任何网格库类型不得进入。约定：
/// - 节点与元素索引一律 0 起；宽度固定 std::uint32_t（冒烟规模，
///   索引宽度与内存布局的扩展性记录在任务 010）；
/// - 坐标与生成参数一律毫米（与 geometry 契约一致）；
/// - 四面体局部节点顺序按有向体积为正定义，由
///   tet_six_times_volume 操作化定义，校验与适配器共用同一实现。
#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <vector>

namespace panta::mesh {

/// 节点/单元/分组共用的索引类型。
using MeshIndex = std::uint32_t;

/// 体单元（四面体）：region 为 0 起的区域（材料域）ID。
struct Tetrahedron {
    std::array<MeshIndex, 4> nodes{};
    MeshIndex region = 0;
};

/// 边界面元（三角形）：group 为 0 起的边界分组 ID，对应来源几何的面分组。
struct SurfaceTriangle {
    std::array<MeshIndex, 3> nodes{};
    MeshIndex group = 0;
};

/// 体网格：区域与边界分组数量显式记录，供分组完整性检查。
struct TetMesh {
    std::vector<std::array<double, 3>> nodes;
    std::vector<Tetrahedron> tets;
    std::vector<SurfaceTriangle> boundary;
    MeshIndex region_count = 0;
    MeshIndex boundary_group_count = 0;
};

/// 四面体有向体积 ×6：正值即 IR 约定的标准正向
/// （det[(p1-p0)×(p2-p0), (p3-p0)] > 0）。
inline double tet_six_times_volume(const std::array<double, 3>& p0, const std::array<double, 3>& p1,
                                   const std::array<double, 3>& p2,
                                   const std::array<double, 3>& p3) {
    const double u[3] = {p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]};
    const double v[3] = {p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]};
    const double w[3] = {p3[0] - p0[0], p3[1] - p0[1], p3[2] - p0[2]};
    const double cross[3] = {u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2],
                             u[0] * v[1] - u[1] * v[0]};
    return cross[0] * w[0] + cross[1] * w[1] + cross[2] * w[2];
}

/// 有效性检查结果；issues 为确定性顺序的英文诊断（面向日志，不直接展示
/// 给最终用户）。
struct MeshValidationReport {
    bool ok = true;
    std::vector<std::string> issues;
};

/// 按 architecture/mesh.md 的冒烟最小集检查：索引范围、非有限坐标、
/// 元素内重复节点、四面体有向体积为正、区域/分组引用范围，以及分组
/// 完整性（每个区域与每个分组都至少被引用一次）。
MeshValidationReport validate_tet_mesh(const TetMesh& mesh);

} // namespace panta::mesh
