# 测试目录与入口规范

更新日期：2026-09-18。本文定义测试代码的归属、发现方式和统一命令；具体领域断言仍由 [验证与评审](validation-and-review.md)、[GTest](gtest.md)、[Rust](rust.md) 和 [QML](qml.md) 约束。

## 目录职责

根 `tests/` 是跨语言测试域，按职责固定分层：

```text
tests/
├── Cargo.toml                 # panta-tests 聚合 package
├── build.rs                   # 解析当前 profile 的 CMake/工具路径
├── src/main.rs                # format/lint/quality 调度器
├── integration/native.rs      # cargo test 触发的 CTest + qmllint 聚合
├── cpp/<module>/*_test.cpp    # C++20、GTest/QtTest 源文件
└── qml/*_test.cpp             # 面向 QML 组件的 QtTest 源文件
```

- `tests/src/` 只放入口编排代码，不放领域测试断言。
- `tests/integration/` 只放 `panta-tests` package 的 Cargo 集成测试；manifest 用显式 `[[test]]` 注册，不能依赖 Cargo 对 `tests/tests` 的默认猜测。
- `tests/cpp/` 和 `tests/qml/` 是测试源代码唯一归档位置。CMake target 仍在被测模块的 `CMakeLists.txt` 注册，源文件使用明确相对路径；测试二进制不安装、不导出。
- `native/cmake/tests/` 只保留 CMake 脚本和小型 configure fixture，不放 C++/QML 行为测试。
- 生产 QML 仍在 `qml/`；`tests/qml/` 只存 QtTest 驱动的验证代码，不复制生产组件。
- 小型、可追溯的输入夹具放 `tests/fixtures/<module>/`；生成物、日志和覆盖率报告放构建树或 `artifacts/`，不混入源码目录。

## Rust 测试边界

- 单元测试使用同一 `.rs` 文件中的 `#[cfg(test)] mod tests`，因为它们需要访问模块私有项；不要为了目录整齐把私有实现测试搬到根 `tests/`。
- crate 级公共行为、跨模块和黑盒测试放对应 crate 的 `tests/`，按领域命名（如 `crates/panta-dsl-core/tests/catalog.rs`）。它们只能使用公开 API。
- 根 `tests/integration/native.rs` 是跨语言聚合测试，不承载 Rust 领域断言；它只验证构建树、qmllint、CTest 的退出码和失败传播。
- 测试应断言行为、不变量和错误上下文；不要复制实现、依赖测试顺序或通过重复调用提升覆盖率。

## C++、QML 与 CMake

- C++ 单元/行为测试遵循 [GTest](gtest.md)，Qt 对象和 QML 资源加载使用 QtTest；每个可执行测试 target 名为 `<被测目标>_test`。
- `BUILD_TESTING=ON` 时注册测试；GTest 使用 `gtest_discover_tests`，QtTest 使用 `add_test`，测试名按 `模块.行为` 命名。禁止只构建测试二进制而不注册 CTest。
- `cargo test` 的根集成测试先构建 `all_qmllint`，再运行 `ctest --output-on-failure --no-tests=error -C <profile>`；空套件、构建失败和任一测试失败都返回非零。
- QML lint/format 使用同一托管 Qt 工具版本；真实窗口、GPU、DPR 和平台生命周期验证另行记录，不能把无头 CTest 结果写成完整图形验收。
- CMake 测试源不得通过宽泛 `GLOB` 自动发现；新增测试必须在对应模块 `CMakeLists.txt` 显式注册并同步 task/验证记录。

## 命令与 CI

本地日常入口：

```sh
cargo test       # workspace Rust + 根 tests/integration/native.rs
cargo format     # Rust、C++/CXX、QML 格式
cargo lint       # Clippy、machete、cmake-lint、qmllint、Clang-Tidy、IWYU、Cppcheck
```

`cargo lint <tool>` 用于单项定位。CI 对构建、测试、格式、每个 lint 工具和依赖审计分别建检查；CI 命令追加 `--locked`，确保 `Cargo.lock` 与 manifests 漂移时立即失败。`cargo quality` 只作为本地全量聚合入口，不作为 CI 的唯一检查名。

测试迁移必须同时更新 CMake source path、Cargo manifest、根 aliases、README、task 和索引，并用 `rg` 清理旧目录引用；禁止保留同一测试的双份实现。
