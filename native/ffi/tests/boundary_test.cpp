#include "panta_ffi.h"

#include <gtest/gtest.h>

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

TEST(FfiBoundary, RustPanicAbortsInsteadOfThrowing) {
    EXPECT_DEATH(panta::ffi::panic_probe(), "");
}
