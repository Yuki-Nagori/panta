# GoogleTest 1.18.0 static SDKs

Panta project CI builds and self-checks a static GoogleTest SDK for macOS arm64,
Linux x86_64, and Windows x86_64. Each archive contains the upstream CMake
package, headers, static libraries, source metadata, and BSD-3-Clause license.
The normal project build continues to use its current FetchContent path until a
separate task registers and validates these assets for consumption.

## Source and build

- Upstream: <https://github.com/google/googletest>
- Release: `v1.18.0`
- Commit: `063de7e9578f82b369302001269680b4b1553359`
- Build: Release, C++17, static libraries, GoogleMock disabled, shared CRT on
  Windows.
- Self-check: the archive-local `GTestConfig.cmake` must provide
  `GTest::gtest` and `GTest::gtest_main`; CI compiles, links, and runs a test
  executable before packaging.
- License: BSD-3-Clause, copied from the upstream source tree to
  `share/licenses/GoogleTest/LICENSE` in every archive.

The release uses the single tag `sdk-googletest-1.18.0`. Reproduction clears
all previous assets and uploads the complete three-platform set after all
matrix jobs pass. Every archive has an adjacent `.sha256` file; consumers must
pin the actual published checksum.
