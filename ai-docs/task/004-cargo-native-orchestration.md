# 004 — Cargo 调度 CMake 与运行入口

- 状态：done
- 阶段：M0
- 依赖：[001](001-cargo-config.md)、[003](003-cmake-native-skeleton.md)（均已完成）
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-16 / 2026-09-16

## 目标与背景

让根目录 Cargo 命令构建并运行 CMake 产物，验证无环依赖与增量追踪。

## 必读

- [规范：cargo](../standards/cargo.md)
- [规范：cmake](../standards/cmake.md)
- [规范：ninja](../standards/ninja.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不将当前 native 骨架称为桌面；CTest 聚合由 011 完成。

## 前置条件与待决策

依赖 001/003 已完成。实施前待决策项（已定，见决策记录）：build.rs/cmake crate/辅助调度选型；native 侧可运行产物的形态与命名；launcher 定位机制（编译期注入 vs 运行时探测）；工具供给与调度的拆分（004 vs 020）。

## 实施步骤

1. 选定 build.rs/cmake crate/辅助调度方案，记录 Cargo→CMake 图和未来 Rust native 库接入位置。
2. 映射 profile、target、依赖前缀和产物目录；build script 生成物只写 OUT_DIR。
3. 实现 launcher 对 native 产物定位、参数和退出码转发；不依赖启动 cwd，不拼 shell 字符串。
4. 追踪 CMake、C++、QML/资源目录及环境参数；此时没有 QML 时先定义扩展点，005 补齐实际验证。

## 预计改动

launcher、build.rs/调度模块、Cargo manifest、native CMake 与构建说明。实际改动：新增 `crates/launcher/build.rs` 与 `native/app/{CMakeLists.txt,main.cpp}`；重写 `crates/launcher/src/main.rs`（转发语义）；`native/CMakeLists.txt` 增加默认值单一来源与 `panta_native_defaults()` 公共函数并接入 app 子目录；`native/foundation/CMakeLists.txt` 改用公共函数；presets 移除已收编的默认值。Cargo manifest 无需改动（build.rs 自动识别）。文档：README、build-and-development、cargo.md、dependency-acquisition、repository-layout、task-index。

## 清理与兼容例外

移除 launcher 过渡行为：原"暂不接受参数"诊断与 `EXIT_USAGE` 常量（参数处理转由 native 产物承担并原样转发）；原"桌面尚未接入"诊断改为"产物缺失/不可启动"语义（退出码 69 保留 EX_UNAVAILABLE 含义）。README/build-and-development 中相应过时表述同步。无兼容层。

## 验收标准

- [x] cargo build --locked 可构建 native 骨架；cargo run 启动 native 骨架并正确返回退出码。
- [x] 首次/无改动/修改 C++ 与配置后构建行为符合预期，失败构建不被 launcher 掩盖。
- [x] 含空格的工作路径及 Debug/Release 定位正确，没有 Cargo↔CMake 递归。
- [x] 已同步相关架构/规范、当前可用命令和 task-index 状态，未将规划能力写成已完成。

- [x] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

以下均在 cwd=仓库根、macOS 26.3.1 arm64、Apple clang 17.0.0、cmake 4.3.3 + Ninja 1.13.2、rustup 1.98.1 执行：

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-09-16 | `cargo build --locked`（首次） | build.rs 调度 CMake configure+build 成功；native 构建树位于 `target/debug/build/panta-launcher-*/out/native-build/`（只写 OUT_DIR）；launcher 链接成功 |
| 2026-09-16 | `cargo run --locked` | 输出 `panta-native 0.1.0`，launcher 退出码 0（可执行位置无关 cwd：以注入的绝对路径启动） |
| 2026-09-16 | `cargo run -- --version`；直接运行 `panta-launcher --bogus` | `--version` 转发并输出版本（0）；未知参数由 native 产物报用法错误，退出码 64 原样转发 |
| 2026-09-16 | `cargo test --locked` | 4 passed（产物缺失→69、注入缺失→配置异常、子进程退出码 1/0 转发） |
| 2026-09-16 | 连续两次 `cargo build --locked` | 第二次全部 Fresh（0.00s）：build.rs 未重跑、CMake/Ninja 未调用 |
| 2026-09-16 | `touch native/app/main.cpp` 后构建 | build.rs 重跑，Ninja 仅重编 main.cpp 并重链（0.9s）；随后恢复 Fresh |
| 2026-09-16 | 失败注入：main.cpp 临时追加 `#error injected_for_verification` 后构建 | `cargo build` 以构建脚本失败退出，编译器错误（文件/行号/信息）完整透出，无掩盖；还原后恢复绿 |
| 2026-09-16 | `CARGO_TARGET_DIR="/tmp/panta space dir/target"` 构建+运行 | 含空格路径全链成功（构建、注入、定位、运行均以参数数组传递，无 shell 拼接） |
| 2026-09-16 | `cargo build --release` | CMakeCache 实测 `CMAKE_BUILD_TYPE:STRING=Release`（profile→构建类型映射生效）；release 产物运行正常，与 debug 树互不影响 |
| 2026-09-16 | native 直接诊断路径回归：`cmake --preset debug` + build + ctest + install | presets 在默认值迁移（安装前缀/compile_commands 收编进 CMakeLists）后行为不变；安装树新增 `bin/panta-native`，ctest 1/1 通过 |
| 2026-09-16 | `cargo fmt --all -- --check`、`cargo clippy --locked --all-targets` | 均通过，无警告 |

未覆盖：Windows（.exe 后缀与 CRT 分支按约定实现，未实测，待 012 runner）；信号终止映射（128+signal）未实测（需向子进程发信号）；CMAKE 环境变量旁路未实测；工具二进制自动供给（见 task 020）。

## 风险与回退

Cargo/CMake 递归：图单向（Cargo→build.rs→CMake→native targets），CMake 侧无 Cargo 调用点，实测无递归。QML/环境参数变化：rerun 清单含 native 全部源/配置路径与 4 个环境变量，`qml/`、`resources/` 为 005 的显式扩展点。回退仅撤销本任务自身变更，恢复 001 的 launcher 诊断行为；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-09-16（决策）调度方案：launcher 的手写 `build.rs`（仅标准库）。否决 `cmake` crate：它只是 Command 薄封装，而本任务需要 profile→构建类型映射、与 presets 同源的有效配置、显式 rerun 清单与转发语义，手写更直接且不给锁文件加依赖。已同步 cargo.md。
- 2026-09-16（决策）native 可运行产物：新增 `native/app` 可执行骨架（CMake target `panta_native_app`，OUTPUT_NAME `panta-native`），打印 foundation 版本；005 在同目录以 Qt 替换实现、保留入口位置与可执行名（改名须同步 build.rs 定位约定）。这落实验收"cargo run 启动 native 骨架"，且不预建 Qt。
- 2026-09-16（决策）产物定位：编译期注入（`cargo:rustc-env=PANTA_NATIVE_BIN`，`option_env!` 读取）。理由：launcher 与 native 产物总由同一 `cargo build` 产出，编译期注入实现最简且与 cwd 无关；代价是二进制不可离开 target 树单独分发——桌面分发（013）由 CMake 产物承担，不受影响。退出码转发规则：低 8 位（POSIX）、信号 128+signal、无信号号回退 130；Windows 大码截断为已记录限制。
- 2026-09-16（决策）有效配置单一来源：构建类型默认 Debug、安装前缀 `${CMAKE_BINARY_DIR}/install`、`CMAKE_EXPORT_COMPILE_COMMANDS=ON` 收编进 native/CMakeLists.txt（`NOT DEFINED` 时设定）；presets 与 build.rs 只传构建类型，消除两套默认值（cargo.md）。公共 target 约束（C++20/无扩展/告警）收拢为 `panta_native_defaults()`。
- 2026-09-16（决策）范围拆分：工具二进制的下载供给从托管原则中拆出为新任务 [020](020-toolchain-provisioning.md)（004 交付调度与诊断，本机暂需预装 CMake/Ninja，`CMAKE` 环境变量可旁路）；dependency-acquisition 与 README 同步此边界。
- 2026-09-16（实施）实施中修正两处自身错误：run_step 误传 `&Command`（需 `&mut`）；run 泛型参数与 `&Vec` 借用不匹配（E0271），改为切片参数。另验证出本机无 `/bin/false|true`，转发测试改用 `/usr/bin`。
- 待记录：005 接入 QML/资源时的 rerun 清单追加；006 的 Rust 库→CMake 导入接入点。

## 完成摘要

已交付：`cargo build --locked` 一条命令构建 Rust workspace + native 骨架（构建树限于 OUT_DIR），`cargo run` 启动 `panta-native` 并转发参数与退出码（0/64/69/128+signal 语义成文）；有效配置与 native presets 单一来源同源；rerun 追踪覆盖 native 源/配置与环境变量，QML/资源扩展点已定义；图单向无递归。验证：首次/无改动/改源重建、失败不掩盖、含空格路径、Debug/Release 映射与隔离全部通过。剩余限制：CMake/Ninja 仍需本机预装（020 补自动供给）；Windows 分支与信号映射未实测（012 runner）；native 测试未入 cargo test 聚合（011）。后续：005（Qt 桌面）与 006（FFI 契约）已 ready，020（工具供给）planned。
