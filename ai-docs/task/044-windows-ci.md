# 044 — Windows CI 停滞诊断与修复

- 状态：in-progress
- 阶段：验证基础
- 依赖：[018](018-cross-platform-ci.md)、[042](042-unified-llvm-toolchain.md)
- 优先级：P1
- 负责人：Codex
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

- [ ] Windows CI 能定位安装锁、下载、摘要校验、解包与编译阶段，失败有上下文。
- [ ] 修复有相应的本地验证及 Windows check/build/toolchain/test 实跑证据。
- [ ] 文档、任务及索引与真实验证状态一致。

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
- Windows check/build 开启 `-vv`，失败/取消时尝试收集已有 build script、CMake configure 和 CTest 日志。没有替换工具链、跳过测试或更改 SHA256；删除旧无整体超时的直接 `status()` 调用，不引入兼容分支。
- 早期远程验证曾因自动审批拒绝一次推送而暂停；随后已有提交 `69d8c16` 进入远程运行，故本条阻塞已解除，后续以真实 CI 日志为准。
- 2026-09-19：远程运行 [35415943085](https://github.com/Yuki-Nagori/panta/actions/runs/35415943085) 验证了阶段日志；下载和 SHA256 校验成功，Windows `tar -xf` 解包 LLVM 在 20 分钟上限后退出码 1。Linux/macOS 同轮通过。确认故障点为 Windows 解包实现。
- 2026-09-19：改用 LLVM 官方 `LLVM-22.1.7-win64.exe`（SHA256 `e091fcf...9a0d1eb3`）并在 staging 目录执行 NSIS `/S /D=...` 静默安装；保留同版本、同来源、同目录的 clang-cl/lld/clang-format。待下一次 Windows CI 验证。
- 2026-09-19：新运行 [35418381255](https://github.com/Yuki-Nagori/panta/actions/runs/35418381255) 已证明安装包在 `target/panta-tools/llvm/...` 内约 64 秒完成；随后 clang-cl 因默认禁用异常而拒绝 CXX 生成代码中的 `throw`。已补 FFI/CMake 的 `/EHsc`，待下一次 Windows CI 验证。

## 完成摘要

本地超时保护、阶段输出与失败日志采集已实现并验证；已确认 Windows LLVM 解包根因并改用官方安装包，修复后的 check/build/toolchain/test 仍待远程实跑，不能宣称 Windows CI 已恢复。
