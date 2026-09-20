/// Netgen OCC 路径的体网格适配器（任务 010）。src/netgen/ 是仓库中唯一
/// 允许包含 Netgen 头的位置；nglib v1 的查询接口不暴露面/区域索引，边界
/// 与区域映射经 libnglib 同源发布的 netgen C++ 头读取（Ng_Mesh* 即
/// netgen::Mesh*）。netgen 内部索引 1 起，转换为 IR 时统一压成 0 起。
///
/// mystdlib.h 提供的 `using namespace std` 是 netgen 遗留头
/// （ngarray/table/hashtabl/optmem）编译的硬前提（上游契约，clang-format
/// 的包含分类与 include-cleaner 豁免规则已按此配置）。该污染只存在于
/// 本适配器 TU；自有代码保持限定名。
#include <mystdlib.h>

#include <algorithm>
#include <array>
#include <cstddef>
#include <exception>
#include <filesystem>
#include <meshing/meshclass.hpp>
#include <meshing/meshtype.hpp>
#include <mutex>
#include <panta/mesh/mesh_ir.hpp>
#include <panta/mesh/netgen_mesher.hpp>
#include <string>
#include <system_error>
#include <utility>
#include <vector>

// 6.2.2604 制品的实现符号位于 namespace nglib（libnglib 只导出
// nglib::Ng_*，无全局符号），而上游 nglib.h/nglib_occ.h 声明在全局
// 命名空间；按制品 ABI 把两个头包含进 namespace nglib，使声明与符号
// 一致。包装只作用于本适配器 TU，调用处全部以 nglib:: 限定。
namespace nglib {
#include <nglib.h>
#include <nglib_occ.h>
} // namespace nglib

namespace panta::mesh {
namespace {

/// Netgen 与 OCCT 的网格生成路径使用进程级全局状态（standards/netgen.md），
/// 首期保守串行：全部生成共用一把进程级锁。
std::mutex& netgen_mutex() {
    static std::mutex mutex;
    return mutex;
}

/// nglib::Ng_Init 每进程一次；不调用 Ng_Exit——重复 Init/Exit 会清空全局
/// 状态，进程退出时由运行时回收。
void init_netgen_once() {
    static std::once_flag flag;
    std::call_once(flag, [] { nglib::Ng_Init(); });
}

/// 净化函数表：生成阶段失败统一转 kMeshingFailed 并携带阶段与返回码。
std::string meshing_failure_detail(const char* stage, nglib::Ng_Result result) {
    return std::string(stage) + " failed with Ng_Result " +
           std::to_string(static_cast<int>(result));
}

/// 排序去重：Netgen 的 1 起编号压缩为 IR 的 0 起连续 ID；返回压缩映射。
std::vector<MeshIndex> compact_ids(std::vector<int> values) {
    std::sort(values.begin(), values.end());
    values.erase(std::unique(values.begin(), values.end()), values.end());
    std::vector<MeshIndex> mapping;
    mapping.reserve(values.size());
    for (const int value : values)
        mapping.push_back(static_cast<MeshIndex>(value) - 1U);
    return mapping;
}

MeshIndex compressed_index(const std::vector<MeshIndex>& mapping, int one_based) {
    const auto found =
        std::lower_bound(mapping.begin(), mapping.end(), static_cast<MeshIndex>(one_based) - 1U);
    return static_cast<MeshIndex>(found - mapping.begin());
}

/// 把 netgen::Mesh 的节点、四面体与边界面元转换为 IR；区域取自体单元
/// 的域号，边界分组取自面描述子的 OCC 面序号（SurfNr）。
bool convert_to_ir(const netgen::Mesh& source, TetMeshGenerationResult& result) {
    result.mesh.nodes.reserve(static_cast<std::size_t>(source.GetNP()));
    for (int i = 1; i <= source.GetNP(); ++i) {
        const auto& point = source.Point(i);
        result.mesh.nodes.push_back({point(0), point(1), point(2)});
    }

    std::vector<int> domain_ids;
    std::vector<int> face_ids;
    domain_ids.reserve(static_cast<std::size_t>(source.GetNE()));
    face_ids.reserve(static_cast<std::size_t>(source.GetNSE()));
    for (const auto& element : source.VolumeElements()) {
        if (element.GetNP() != 4) {
            result.status = TetMeshGenerationStatus::kConversionFailed;
            result.detail = "volume element with " + std::to_string(element.GetNP()) +
                            " nodes is outside the smoke contract";
            return false;
        }
        domain_ids.push_back(element.GetIndex());
    }
    for (const auto& element : source.SurfaceElements()) {
        if (element.GetNP() != 3) {
            result.status = TetMeshGenerationStatus::kConversionFailed;
            result.detail = "surface element with " + std::to_string(element.GetNP()) +
                            " nodes is outside the smoke contract";
            return false;
        }
        const netgen::FaceDescriptor& descriptor = source.GetFaceDescriptor(element);
        face_ids.push_back(descriptor.SurfNr());
    }

    const std::vector<MeshIndex> domain_mapping = compact_ids(std::move(domain_ids));
    const std::vector<MeshIndex> face_mapping = compact_ids(std::move(face_ids));
    result.mesh.region_count = static_cast<MeshIndex>(domain_mapping.size());
    result.mesh.boundary_group_count = static_cast<MeshIndex>(face_mapping.size());

    result.mesh.tets.reserve(static_cast<std::size_t>(source.GetNE()));
    for (const auto& element : source.VolumeElements()) {
        const MeshIndex region = compressed_index(domain_mapping, element.GetIndex());
        std::array<MeshIndex, 4> nodes = {
            static_cast<MeshIndex>(element[0]) - 1U, static_cast<MeshIndex>(element[1]) - 1U,
            static_cast<MeshIndex>(element[2]) - 1U, static_cast<MeshIndex>(element[3]) - 1U};
        // 方向归一化：Netgen 的局部顺序与 IR 的正体积约定相反（见
        // generate_tet_mesh_from_step 内说明），有向体积为负时交换末两个
        // 节点，使 IR 四面体统一为标准正向。
        if (tet_six_times_volume(result.mesh.nodes[nodes[0]], result.mesh.nodes[nodes[1]],
                                 result.mesh.nodes[nodes[2]], result.mesh.nodes[nodes[3]]) < 0.0) {
            std::swap(nodes[2], nodes[3]);
        }
        result.mesh.tets.push_back({nodes, region});
    }
    result.mesh.boundary.reserve(static_cast<std::size_t>(source.GetNSE()));
    for (const auto& element : source.SurfaceElements()) {
        const netgen::FaceDescriptor& descriptor = source.GetFaceDescriptor(element);
        const MeshIndex group = compressed_index(face_mapping, descriptor.SurfNr());
        result.mesh.boundary.push_back(
            {{static_cast<MeshIndex>(element[0]) - 1U, static_cast<MeshIndex>(element[1]) - 1U,
              static_cast<MeshIndex>(element[2]) - 1U},
             group});
    }
    return true;
}

} // namespace

/// 生成收尾：释放几何对象。网格对象按官方示例（ng_occ.cpp 不调用
/// Ng_DeleteMesh）保留至进程结束——制品 v6.2.2604 的 Mesh 析构会释放
/// 悬空的边界名指针（上游在 tag 之后重构了名字/描述符所有权，master
/// 的 ~Mesh 已不再手工 delete），038 重新固定制品后恢复网格释放。
/// TODO(task 010): 制品升级后恢复 Ng_DeleteMesh 并回归全部网格用例。
void finish_generation(nglib::Ng_OCC_Geometry* geometry) { nglib::Ng_OCC_DeleteGeometry(geometry); }

TetMeshGenerationResult generate_tet_mesh_from_step(const std::filesystem::path& file,
                                                    const TetMeshGenerationParameters& parameters) {
    TetMeshGenerationResult result;
    const std::lock_guard<std::mutex> lock(netgen_mutex());
    init_netgen_once();

    std::error_code existence;
    if (!std::filesystem::is_regular_file(file, existence)) {
        result.status = TetMeshGenerationStatus::kFileNotFound;
        result.detail = "input is not a readable regular file";
        return result;
    }

    // 加载：Netgen 对不可解析输入不返回空指针而是抛异常（实测文本垃圾
    // 抛 "Couldn't load OCC geometry"），在此边界捕获转结构化状态。
    nglib::Ng_OCC_Geometry* geometry = nullptr;
    try {
        geometry = nglib::Ng_OCC_Load_STEP(file.string().c_str());
    } catch (const std::exception& error) {
        result.status = TetMeshGenerationStatus::kGeometryLoadFailed;
        result.detail = error.what();
        return result;
    }
    if (geometry == nullptr) {
        result.status = TetMeshGenerationStatus::kGeometryLoadFailed;
        result.detail = "Ng_OCC_Load_STEP returned no geometry";
        return result;
    }

    nglib::Ng_Mesh* native_mesh = nglib::Ng_NewMesh();
    try {
        nglib::Ng_Meshing_Parameters native_parameters;
        if (parameters.max_length_mm > 0.0)
            native_parameters.maxh = parameters.max_length_mm;
        // Netgen 内部节点顺序与标准 FEM 正体积约定相反（box 实测 236/236
        // 为负，且 nglib 的 invert_tets 在该制品 OCC 路径未生效）；方向
        // 归一化在 convert_to_ir 内完成，满足 Mesh IR 的方向契约。

        const nglib::Ng_Result set_size =
            nglib::Ng_OCC_SetLocalMeshSize(geometry, native_mesh, &native_parameters);
        if (set_size != nglib::NG_OK) {
            result.status = TetMeshGenerationStatus::kMeshingFailed;
            result.detail = meshing_failure_detail("Ng_OCC_SetLocalMeshSize", set_size);
        } else if (const nglib::Ng_Result edges =
                       nglib::Ng_OCC_GenerateEdgeMesh(geometry, native_mesh, &native_parameters);
                   edges != nglib::NG_OK) {
            result.status = TetMeshGenerationStatus::kMeshingFailed;
            result.detail = meshing_failure_detail("Ng_OCC_GenerateEdgeMesh", edges);
        } else if (const nglib::Ng_Result surfaces =
                       nglib::Ng_OCC_GenerateSurfaceMesh(geometry, native_mesh, &native_parameters);
                   surfaces != nglib::NG_OK) {
            result.status = TetMeshGenerationStatus::kMeshingFailed;
            result.detail = meshing_failure_detail("Ng_OCC_GenerateSurfaceMesh", surfaces);
        } else if (const nglib::Ng_Result volume =
                       nglib::Ng_GenerateVolumeMesh(native_mesh, &native_parameters);
                   volume != nglib::NG_OK) {
            result.status = TetMeshGenerationStatus::kMeshingFailed;
            result.detail = meshing_failure_detail("Ng_GenerateVolumeMesh", volume);
        }

        if (result.status == TetMeshGenerationStatus::kNone) {
            const auto* mesh = reinterpret_cast<const netgen::Mesh*>(native_mesh);
            if (mesh->GetNP() == 0 || mesh->GetNE() == 0) {
                result.status = TetMeshGenerationStatus::kEmptyMesh;
                result.detail = "generation finished without a non-empty mesh";
            } else if (!convert_to_ir(*mesh, result)) {
                result.mesh = TetMesh{};
            }
        }

        if (result.status == TetMeshGenerationStatus::kNone) {
            for (const Tetrahedron& tet : result.mesh.tets) {
                result.volume_mm3 += tet_six_times_volume(result.mesh.nodes[tet.nodes[0]],
                                                          result.mesh.nodes[tet.nodes[1]],
                                                          result.mesh.nodes[tet.nodes[2]],
                                                          result.mesh.nodes[tet.nodes[3]]) /
                                     6.0;
            }

            const MeshValidationReport report = validate_tet_mesh(result.mesh);
            if (!report.ok) {
                // 候选网格不发布：校验失败即丢弃并携带首个问题。
                result.status = TetMeshGenerationStatus::kInvalidMesh;
                result.detail = report.issues.front();
                result.mesh = TetMesh{};
                result.volume_mm3 = 0.0;
            }
        }
    } catch (const std::exception& error) {
        // 网格库异常的统一兜底：丢弃候选输出，保留异常原文作为诊断。
        result.status = TetMeshGenerationStatus::kInternalFailure;
        result.detail = error.what();
        result.mesh = TetMesh{};
        result.volume_mm3 = 0.0;
    }

    finish_generation(geometry);
    return result;
}

} // namespace panta::mesh
