#include "panta_ffi.h"
#include "rust/cxx.h"
#include <cmath>
#include <gtest/gtest.h>
#include <utility>

TEST(FfiBoundary, CppCallsRustAndRustCallsCpp) {
    panta::ffi::FfiRequest request;
    request.text = "界";
    request.repeat = 2;

    const auto response = panta::ffi::process(request);

    EXPECT_EQ(response.value, "ffi:界界");
    EXPECT_EQ(response.repeat, 2U);
}

TEST(FfiBoundary, RustErrorBecomesCppException) {
    panta::ffi::FfiRequest request;
    request.text = "x";
    request.repeat = 0;

    EXPECT_THROW(static_cast<void>(panta::ffi::process(request)), rust::Error);
}

TEST(FfiBoundary, RustPanicAbortsInsteadOfThrowing) { EXPECT_DEATH(panta::ffi::panic_probe(), ""); }

TEST(FfiBoundary, OpaqueSessionCreateUseAndRelease) {
    // 前置为 0：若此前用例泄漏句柄，在此显式失败而非掩盖。
    ASSERT_EQ(panta::ffi::session_live_count(), 0U);

    auto alpha = panta::ffi::session_create("会话-α");
    auto beta = panta::ffi::session_create("会话-β");
    ASSERT_EQ(panta::ffi::session_live_count(), 2U);
    EXPECT_EQ(panta::ffi::session_label(*alpha), "会话-α");
    EXPECT_EQ(panta::ffi::session_label(*beta), "会话-β");

    // rust::Box 所有权 move 回 Rust；逆序释放并核对剩余存活数。
    EXPECT_EQ(panta::ffi::session_close(std::move(beta)), 1U);
    EXPECT_EQ(panta::ffi::session_close(std::move(alpha)), 0U);
    EXPECT_EQ(panta::ffi::session_live_count(), 0U);
}

TEST(FfiBoundary, OpaqueSessionRejectsInvalidLabel) {
    EXPECT_THROW(static_cast<void>(panta::ffi::session_create("")), rust::Error);

    // 拒绝路径不残留句柄。
    EXPECT_EQ(panta::ffi::session_live_count(), 0U);
}

TEST(FfiBoundary, MeshDtoRejectsIncompleteCoordinateTriples) {
    panta::ffi::TetMeshData data;
    data.nodes.push_back(0.0);
    data.nodes.push_back(1.0);
    const auto report = panta::ffi::mesh_validate_tet(std::move(data));
    ASSERT_EQ(report.issues.size(), 1U);
    EXPECT_EQ(report.issues[0], "invalid mesh DTO layout");
    EXPECT_DOUBLE_EQ(report.volume_mm3, 0.0);
}

TEST(FfiBoundary, MeshValidationReturnsRustComputedVolume) {
    panta::ffi::TetMeshData data;
    data.nodes = {0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0};
    data.tets = {0, 1, 2, 3};
    data.tet_regions = {0};
    data.boundary = {0, 1, 2};
    data.boundary_groups = {0};
    data.region_count = 1;
    data.boundary_group_count = 1;

    const auto report = panta::ffi::mesh_validate_tet(std::move(data));
    EXPECT_TRUE(report.issues.empty());
    EXPECT_TRUE(std::abs(report.volume_mm3 - 1.0 / 6.0) < 1e-15);
}
