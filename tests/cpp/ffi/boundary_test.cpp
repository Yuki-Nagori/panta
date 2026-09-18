#include "panta_ffi.h"
#include "rust/cxx.h"
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
