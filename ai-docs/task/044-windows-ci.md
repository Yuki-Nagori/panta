# 044 — Windows CI 停滞诊断与修复

- 状态：done
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[042](042-unified-llvm-toolchain.md)
- 优先级：P1
- 负责人：Yuki
- 创建 / 更新：2026-09-19 / 2026-09-19

## 目标与背景

Windows CI 在 `cargo check --locked --workspace --all-targets` 中长时间没有输出，后续 build、工具链核验和 test 均未运行。最新运行 [35411655728](https://github.com/Yuki-Nagori/panta/actions/runs/35411655728/job/105812337396) 在约 73 分钟后取消；前一次 [35376430562](https://github.com/Yuki-Nagori/panta/actions/runs/35376430562/job/105702062809) 达到约 6 小时后取消。最新日志停于 panta-ffi build script 开始后的其他 Rust crate 编译，尚不能断定停在下载、解包或编译。

## 必读

- [验证与评审](../standards/validation-and-review.md)、[依赖获取](../standards/dependency-acquisition.md)
- [Rust](../standards/rust.md)、[测试](../standards/testing.md)、[注释](../standards/comments.md)
- [仓库文件](../standards/repository-hygiene.md)、[文档](../standards/documentation.md)、[生命周期](../standards/code-lifecycle.md)

## 范围与非目标

定位 Windows 托管工具链与 Cargo/native 构建停滞；补充实时阶段日志及有界失败，依据真实证据修复。保持固定 LLVM 22.1.7、MSVC ABI、Ninja 和 Cargo 统一入口，不更换为系统工具旁路，不扩大到 SDK 生产或业务功能。

## 实施步骤

1. 补齐供给阶段日志和 Windows Cargo build script 的实时输出，缩小停滞范围。
2. 修复已确认的供给/构建问题，运行相关回归测试与 workflow 静态检查。
3. 记录 Windows 实跑证据；远程未通过前保持任务未完成。

## 验收标准

- [x] Windows CI 能定位安装锁、下载、摘要校验、解包与编译阶段，失败有上下文。
- [x] 修复有相应的本地验证及 Windows check/build/toolchain/test 实跑证据。
- [x] 文档、任务及索引与真实验证状态一致。

## 清理与兼容例外

无兼容例外；替换的供给实现同步清理，不增加旧工具链回退。

## 验证计划与结果

- 2026-09-19：通过 `gh run view` 检查上述两次运行；最新提交 `8e98add` 的 macOS/Linux check/build/toolchain/test 成功，Windows 取消，没有可用的 native 失败诊断。
- 当前执行环境为 macOS arm64；本地验证不能代替 Windows CI。
- 2026-09-19（仓库根目录，Rust 1.98.1）：`cargo test --locked -p panta-build` 14/14 通过；新增真实子进程夹具覆盖成功、退出码 23、300 ms 超时终止/回收及可执行文件缺失。既有并发安装、失败保留旧版本、进程退出释放锁测试继续通过。
- `cargo clippy --locked -p panta-build --all-targets -- -D warnings`、`actionlint .github/workflows/ci.yml`、`git diff --check` 通过；使用 `cargo fmt --all` 格式化。
- macOS 托管模式下 `cargo check --locked --workspace --all-targets`、`cargo build --locked --workspace`、`cargo run --locked --package panta-tests -- toolchain` 全部通过，两条 C++ 链仍使用固定 LLVM 22.1.7，共享 CMake/Ninja/Qt/GoogleTest 缓存核验成功。未开启系统工具旁路；本轮为已有缓存的增量验证。

## 决策与工作记录

- 已确认原下载上限不包含所有重试：curl `--max-time 1800 --retry 3` 的单次计时会重置，可能累计约两小时；这与 Windows 停滞相关，但现有日志不足以证明它就是此次停滞根因。已增加下载总墙钟 30 分钟、解包 20 分钟的父进程限制及每 30 秒的阶段进度。
- 曾临时开启 Windows check/build 的 `-vv` 并上传失败诊断，用于定位工具链和 CTest 问题；根因确认后已清理这些排查选项与诊断步骤。没有替换工具链、跳过测试或更改 SHA256；删除旧无整体超时的直接 `status()` 调用，不引入兼容分支。
- 早期远程验证曾因自动审批拒绝一次推送而暂停；随后已有提交 `69d8c16` 进入远程运行，故本条阻塞已解除，后续以真实 CI 日志为准。
- 2026-09-19：远程运行 [35415943085](https://github.com/Yuki-Nagori/panta/actions/runs/35415943085) 验证了阶段日志；下载和 SHA256 校验成功，Windows `tar -xf` 解包 LLVM 在 20 分钟上限后退出码 1。Linux/macOS 同轮通过。确认故障点为 Windows 解包实现。
- 2026-09-19：改用 LLVM 官方 `LLVM-22.1.7-win64.exe`（SHA256 `e091fcf...9a0d1eb3`）并在 staging 目录执行 NSIS `/S /D=...` 静默安装；保留同版本、同来源、同目录的 clang-cl/lld/clang-format。待下一次 Windows CI 验证。
- 2026-09-19：新运行 [35418381255](https://github.com/Yuki-Nagori/panta/actions/runs/35418381255) 已证明安装包在 `target/panta-tools/llvm/...` 内约 64 秒完成；随后 clang-cl 因默认禁用异常而拒绝 CXX 生成代码中的 `throw`。已补 FFI/CMake 的 `/EHsc`，待下一次 Windows CI 验证。
- 2026-09-19：运行 [35418599708](https://github.com/Yuki-Nagori/panta/actions/runs/35418599708) 的 Windows job 已完成 112/112 个 native 编译步骤，clang-cl 与链接均成功；launcher 因只检查 `target/native/debug/app/panta-native.exe` 而未覆盖生成器的配置目录/构建树根目录，误报产物缺失。恢复受限于 `target/native/<profile>` 的多布局探测。
- 2026-09-19：运行 [35418983684](https://github.com/Yuki-Nagori/panta/actions/runs/35418983684) 已验证 check/build 与 launcher 产物定位通过；Test 的 CTest discovery 因 Windows 子进程缺少托管 Qt DLL，返回 `0xc0000135`。新增 native test 环境 PATH，将 `target/panta-deps/qt/staging/bin` 放在 MSVC SDK PATH 前，去除临时 CI 详细输出与失败诊断上传，待下一次 CI 验证。
- 2026-09-19：提交 `922139c` 的运行 [35419894410](https://github.com/Yuki-Nagori/panta/actions/runs/35419894410) 已验证 Qt DLL 环境修复、`PathHostTest` 及 `Qml.FormatCheck` 通过；Windows 仍有 `Qml.ThemeComponentParameters` 与 `Qml.ShellModuleLoads` 失败。当时怀疑 QML 测试运行时环境不完整（尚未证实），`d52137b` 已加入托管 Qt 的 QML 导入路径，`9aaf2c4` 又加入插件目录及 Basic Controls 样式，但运行仍复现，当前继续补齐平台插件路径。
- 2026-09-19：运行 [35421298483](https://github.com/Yuki-Nagori/panta/actions/runs/35421298483) 仍复现上述两个 QML 测试失败；`e97517e` 已将平台插件路径同时写入 native 测试进程环境，当前把同一 Qt 运行时设置下沉到 CTest 测试属性，避免 CTest 子进程环境差异。
- 2026-09-19：运行 [35421659951](https://github.com/Yuki-Nagori/panta/actions/runs/35421659951) 验证构建、QML 格式和其余 24 项通过，但两个启动 `QGuiApplication` 的 QML 测试仍失败；当时怀疑 Windows 的 `offscreen` QPA 启动问题（尚未证实），改为仅 Windows 使用 Qt `minimal` 平台，Linux/macOS 继续使用 `offscreen`。
- 2026-09-19：运行 [35423204587](https://github.com/Yuki-Nagori/panta/actions/runs/35423204587/job/105844516615) 仍只有两个 QML 启动测试失败，格式门禁已通过；额外的 CTest 重跑写在 CLI 入口，实际 CI 调用 integration/native.rs，未执行该诊断；此前据此推断启动原因不成立。当前修复将 `panta_shell` DLL 复制到 Windows QML 测试可执行文件目录，并补齐生成模块目录的运行时搜索路径，待下一次实跑验证。

- 2026-09-19：运行 [35423565513](https://github.com/Yuki-Nagori/panta/actions/runs/35423565513) 中 qmllint、QML 格式及其余 24 项测试通过，两个 QML 运行测试仍失败；DLL 复制未解决问题且缺少 COMMENT 导致 CMake lint 失败。本轮删除复制命令，恢复统一 offscreen，并让 QML 的 CTest 入口报告实际子进程退出状态、强制 Qt 日志到 stderr；根据真实错误再简化运行时设置。

- 本轮本地验证：native 重新配置与构建成功，CTest `^Qml\.` 三项全部通过，`cargo lint cmake`、`git diff --check` 通过；Windows 退出状态待推送后验证。

- 2026-09-19：运行 [35424136090](https://github.com/Yuki-Nagori/panta/actions/runs/35424136090/job/105847042839) 首次取得 QtTest 日志：两个测试均已正常启动，失败为 Shell 的 qrc 资源不存在，非 qmllint 或平台插件失败。Shell 原为无消费者符号引用的共享库；改用 STATIC NO_PLUGIN，让 Qt 将资源对象链接进消费者。清理 native 构建路径、QML2_IMPORT_PATH、重复平台插件路径及临时执行包装；保留 Qt 日志到 stderr 并统一 QML 测试环境。

- 静态资源修复本地验证：workspace build、`cargo lint qmllint`、`cargo lint cmake`、panta-build 14 项测试通过；默认 Bridge 与 `--no-default-features` 构建下三个 QML CTest 均通过；`otool -L` 确认测试不再依赖 Shell 动态库。最终 Windows CI 待验证。

- 最终代码本地完整回归：`cargo test --locked` 通过（全部 Rust 测试及 28/28 CTest），提交钩子的聚合 `cargo format` 与 `cargo lint clippy` 通过。完整测试在允许系统测试目录访问的环境运行，避免将沙箱路径限制误判为 PathHost 功能失败。

- Windows 最终验证：[35424505359 / Windows](https://github.com/Yuki-Nagori/panta/actions/runs/35424505359/job/105848017067)（代码提交 `d20ee8f`）check、build、toolchain、test 全部通过，26/26 CTest 成功，包含两个 QML 运行测试与格式检查；托管 LLVM/Qt 目录的 Actions cache 保存成功。

- 代码提交 `d20ee8f` 的整轮 [CI 35424505359](https://github.com/Yuki-Nagori/panta/actions/runs/35424505359) 全部通过：Windows/macOS/Linux check/build/toolchain/test、七项 lint、聚合格式、依赖审计及 Rust/native 覆盖率检查。后续提交仅同步本任务与索引的完成记录。

## 完成摘要

Windows LLVM 改用受限于 target 的官方安装包，统一异常与运行库设置并修复 launcher 产物定位。QML 格式检查忽略平台换行差异；Shell 改为静态资源模块，消除 Windows 不加载无符号引用 DLL 导致的 qrc 缺失。统一 QML 测试环境并让 QtTest 输出到 stderr，清理重复路径、DLL 复制和临时诊断包装。Windows check/build/toolchain/test 已通过，LLVM/Qt 缓存已保存；本地完整测试及 Bridge 关闭入口亦通过。
