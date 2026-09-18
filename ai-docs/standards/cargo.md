# Cargo workspace 与原生调度

查阅日期：2026-09-16。状态：workspace 骨架已按本规范落地并验证（任务 001）；native 调度、build script 约定由任务 004 实施后再验证。

适用于 workspace、依赖锁、build scripts 和统一命令入口。Rust 语言规范另见 [Rust](rust.md)。

## 官方依据

虚拟 workspace 需要显式指定 resolver；成员可继承 workspace 包属性、依赖及 lints。[Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

build script 产物应放在 `OUT_DIR`；重建追踪通过 `rerun-if-changed` 与 `rerun-if-env-changed` 声明，脚本运行在 host 而非 target。[Cargo Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

## 项目规则

- workspace 采用 edition 2024 与显式 `resolver = "3"`，成员与公共依赖集中在根 `Cargo.toml`（任务 001 已落地）。默认 launcher 唯一：`crates/launcher`（bin `panta-launcher`），根目录 `cargo run` 无 target 歧义。
- 应用仓库提交 `Cargo.lock`；CI 使用锁定依赖。不能提交仅含不存在目录的 workspace members，也不要为了结构整齐创建没有用途的 crate。
- Cargo 是用户入口，CMake 拥有 native 构建图。调度方案已定（任务 004）：launcher 的手写 `build.rs`（仅标准库）按 profile 映射构建类型并调用 CMake；有效配置与 presets 同源（默认值集中在 native/CMakeLists.txt）。未来 C++ 消费 Rust 库的接入点在 CMake 侧导入 Rust 产物（任务 006 定），不得从 build.rs 再触发 Cargo。
- 使用 build script 时只向 `OUT_DIR` 写生成产物；native 源文件、QML、资源与影响配置的环境变量均进入重建追踪。不要把构建日志误输出为 Cargo 指令。
- 明确 Debug/Release、target triple、编译器和依赖前缀映射。交叉编译未验证时给出不支持诊断，不把 host 探测结果当成 target 配置。
- launcher 用结构化进程参数启动 CMake 产物，转发退出码并定义终止行为；不依赖当前工作目录或硬编码个人绝对路径。
- workspace 根 `cargo test` 通过 `tests/` package 的显式 `integration/native.rs` 调度 CTest/QML；聚合器使用 `--no-tests=error` 和失败码传播，防止零个 native 测试显示“全部通过”。crate 私有 Rust 单元/集成测试仍按 [测试规范](testing.md) 归属。

## 验证

任务 004 验证首次构建、无改动构建、修改 C++/QML/资源后的重建、失败码与配置切换。首次 Cargo 骨架完成只说明 Rust 基础可用，不宣称桌面已经构建成功。
