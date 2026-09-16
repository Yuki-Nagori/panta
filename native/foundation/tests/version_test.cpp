#include <panta/foundation/version.hpp>

#include <cstdlib>

namespace {

// 期望值由 CMake project(VERSION) 注入，保证断言与版本单一来源一致。
constexpr panta::foundation::Version kExpected{PANTA_TEST_EXPECT_MAJOR, PANTA_TEST_EXPECT_MINOR,
                                               PANTA_TEST_EXPECT_PATCH};

} // namespace

int main() {
    const panta::foundation::Version actual = panta::foundation::native_version();
    if (actual.major != kExpected.major || actual.minor != kExpected.minor ||
        actual.patch != kExpected.patch) {
        return 1;
    }
    return 0;
}
