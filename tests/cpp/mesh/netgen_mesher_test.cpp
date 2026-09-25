/// 任务 010 的 Netgen 生成冒烟：简单闭合实体（复用 009 的 box_mm.step，
/// 10×20×30 mm）→ 体网格 → 自有 IR 的转换与失败边界。
#include <filesystem>
#include <gtest/gtest.h>
#include <panta/mesh/mesh_ir.hpp>
#include <panta/mesh/netgen_mesher.hpp>

#ifndef PANTA_STRINGIFY
#define PANTA_STRINGIFY_(value) (#value)
#define PANTA_STRINGIFY(value) (PANTA_STRINGIFY_(value))
#endif

namespace {

namespace fs = std::filesystem;

using panta::mesh::generate_tet_mesh_from_step;
using panta::mesh::TetMeshGenerationParameters;
using panta::mesh::TetMeshGenerationStatus;
using panta::mesh::validate_native_mesh_with_rust;

const fs::path& fixture_dir() {
    // 夹具根经编译定义注入裸路径（cppcheck 兼容，见 009 同型说明）。
    static const fs::path dir = PANTA_STRINGIFY(PANTA_MESH_FIXTURE_DIR);
    return dir;
}

TEST(NetgenMesher, BoxYieldsValidTetMeshWithExpectedVolumeAndGroups) {
    TetMeshGenerationParameters parameters;
    parameters.max_length_mm = 5.0;
    const auto result = generate_tet_mesh_from_step(fixture_dir() / "box_mm.step", parameters);
    ASSERT_EQ(result.status, TetMeshGenerationStatus::kNone) << result.detail;

    EXPECT_FALSE(result.mesh.nodes.empty());
    EXPECT_FALSE(result.mesh.tets.empty());
    EXPECT_FALSE(result.mesh.boundary.empty());
    // 单一实体 → 单一区域；长方体 6 个面 → 6 个边界分组，且无丢失。
    EXPECT_EQ(result.mesh.region_count, 1u);
    EXPECT_EQ(result.mesh.boundary_group_count, 6u);
    EXPECT_EQ(validate_native_mesh_with_rust(result.mesh).ok, true);

    // 名义体积 10×20×30 = 6000 mm³；面网格贴合平面，断言容差 1%。
    EXPECT_NEAR(result.volume_mm3, 6000.0, 60.0);
}

TEST(NetgenMesher, MaxLengthControlsElementCount) {
    TetMeshGenerationParameters coarse;
    coarse.max_length_mm = 10.0;
    TetMeshGenerationParameters fine;
    fine.max_length_mm = 3.0;
    const auto coarse_mesh = generate_tet_mesh_from_step(fixture_dir() / "box_mm.step", coarse);
    ASSERT_EQ(coarse_mesh.status, TetMeshGenerationStatus::kNone) << coarse_mesh.detail;
    const auto fine_mesh = generate_tet_mesh_from_step(fixture_dir() / "box_mm.step", fine);
    ASSERT_EQ(fine_mesh.status, TetMeshGenerationStatus::kNone) << fine_mesh.detail;
    EXPECT_LT(coarse_mesh.mesh.tets.size(), fine_mesh.mesh.tets.size());
}

TEST(NetgenMesher, MissingFileIsRejectedWithoutOutput) {
    const auto result = generate_tet_mesh_from_step(fixture_dir() / "does_not_exist.step", {});
    EXPECT_EQ(result.status, TetMeshGenerationStatus::kFileNotFound);
    EXPECT_TRUE(result.mesh.tets.empty());
    EXPECT_NEAR(result.volume_mm3, 0.0, 1e-12);
}

TEST(NetgenMesher, GarbageInputIsRejectedWithoutCrash) {
    const auto result = generate_tet_mesh_from_step(fixture_dir() / "invalid_text.step", {});
    EXPECT_EQ(result.status, TetMeshGenerationStatus::kGeometryLoadFailed);
    EXPECT_TRUE(result.mesh.tets.empty());
}

TEST(NetgenMesher, FailureDoesNotCorruptSubsequentGeneration) {
    const auto failed = generate_tet_mesh_from_step(fixture_dir() / "invalid_text.step", {});
    ASSERT_EQ(failed.status, TetMeshGenerationStatus::kGeometryLoadFailed);

    TetMeshGenerationParameters parameters;
    parameters.max_length_mm = 5.0;
    const auto recovered = generate_tet_mesh_from_step(fixture_dir() / "box_mm.step", parameters);
    ASSERT_EQ(recovered.status, TetMeshGenerationStatus::kNone) << recovered.detail;
    EXPECT_EQ(recovered.mesh.boundary_group_count, 6u);
    EXPECT_NEAR(recovered.volume_mm3, 6000.0, 60.0);
}

TEST(NetgenMesher, RepeatedGenerationsReleaseNativeMeshes) {
    TetMeshGenerationParameters parameters;
    parameters.max_length_mm = 5.0;

    for (int generation = 0; generation < 3; ++generation) {
        const auto result = generate_tet_mesh_from_step(fixture_dir() / "box_mm.step", parameters);
        ASSERT_EQ(result.status, TetMeshGenerationStatus::kNone) << result.detail;
        EXPECT_FALSE(result.mesh.tets.empty());
    }
}

} // namespace
