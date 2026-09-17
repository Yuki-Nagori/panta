# 036 — 三平台 CI native 构建修复

- 状态：in-progress
- 阶段：验证基础
- 依赖：[004](004-cargo-native-orchestration.md)、[005](005-qt-qml-shell.md)、[018](018-cross-platform-ci.md)
- 优先级：P0
- 负责人：Yuki
- 创建 / 更新：2026-09-17 / 2026-09-17

## 目标与背景

GitHub Actions 的首次三平台运行暴露了 native 构建的环境差异：Ubuntu runner 没有 Qt Gui 所需的 OpenGL 开发包；Windows runner 的默认 Ninja 选择了 MinGW，而项目下载的是 MSVC Qt 预编译包，且 Rust `canonicalize()` 产生的 `//?/D:` 路径被 MinGW 误解析。修复后，Cargo 调度的 CMake 构建应在三平台使用匹配的编译器/Qt ABI，并正确定位单配置与多配置生成器的可执行产物。

## 必读

- [Cargo 与 build script 规范](../standards/cargo.md)
- [CMake 规范](../standards/cmake.md)
- [依赖获取与预编译规则](../standards/dependency-acquisition.md)
- [三平台 CI 基础](018-cross-platform-ci.md)
- [验证与评审](../standards/validation-and-review.md)

## 范围与非目标

范围：修正 launcher 的 native 源路径和多配置产物定位；为 Windows 选择与 Qt 预编译包匹配的 MSVC 生成器；为 Ubuntu 安装 Qt 配置所需的 OpenGL 开发包，并将 Qt 官方提供的 ICU 73 预编译运行库纳入 Linux Qt 供给；通过 GitHub Actions 复跑 build、test、fmt 与 clippy。

非目标：不引入依赖缓存、统一质量门禁、VTK/OCCT/Netgen SDK 或新的业务代码；这些范围分别由 012、032 和 031 管理。

## 前置条件与待决策

GitHub Actions 可访问 `Yuki-Nagori/panta`，并已确认 Qt 6.11.2 的 Linux 与 Windows 预编译归档可下载。Windows runner 固定为 `windows-2022`，以匹配 Qt 的 MSVC2022 ABI；若 runner 镜像或 Qt ABI 变更，需更新依赖清单与本任务验证证据。

## 实施步骤

1. 从 `gh run view --log-failed` 固定记录 Ubuntu 与 Windows 的失败上下文。
2. 去除 Windows 不兼容的长路径前缀转换，扩展 build script 对多配置 CMake 产物的处理，并将生成器平台变化加入重建追踪。
3. 更新 CI 矩阵：Linux 安装 OpenGL 开发包，Windows 使用 Visual Studio 17 2022 x64，其他平台继续使用 Ninja。
4. 将 Qt 官方仓库中与 6.11.2 工具匹配的 ICU 73 预编译归档按 SHA256 纳入 Linux staging/lib，确保 rcc、qtpaths、qmlimportscanner 等工具使用匹配 ABI。
5. 先运行本地 Rust/CMake 等价检查，再推送并用 `gh` 观察新的三平台 run；将真实结果回填本文与索引。

## 预计改动

`.github/workflows/ci.yml`、`crates/launcher/build.rs`、`native/cmake/qt-provision.cmake`、`native/app/CMakeLists.txt`、`native/foundation/CMakeLists.txt`、`native/bridge/CMakeLists.txt`、`qml/CMakeLists.txt`、`ai-docs/task-index.md`、`ai-docs/task/018-cross-platform-ci.md`、`ai-docs/standards/dependency-acquisition.md` 与本文件。

## 清理与兼容例外

删除 build script 中会生成 Windows `//?/` 路径的 `canonicalize()` 调用；无兼容例外，不保留 MinGW/未匹配 Qt ABI 的回退分支。

## 验收标准

- [ ] Ubuntu、macOS、Windows CI 的 Build、Test、Format、Clippy 全部成功。
- [ ] Windows 构建日志显示 Visual Studio 生成器与 MSVC 编译器，且 launcher 能定位 `panta-native.exe`。
- [ ] Ubuntu configure 能找到 `WrapOpenGL`/Qt6Gui，不依赖个人机器全局包。
- [ ] 单配置 Ninja 与多配置 Visual Studio 都能按 Cargo profile 选择 Debug/Release；失败时保留 CMake 原始诊断。
- [ ] task、workflow、依赖规范和索引描述一致，无失效路径或未登记兼容代码。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-17 | `gh run view 35123228708 --log-failed`、`gh run view 35105702071 --log-failed` | 固定失败根因 | Ubuntu 缺少 OpenGL；Windows 为 MinGW/`//?/D:` 路径错误 |
| 2026-09-17 | macOS 26.3.1 arm64；`cargo build --locked`、`cargo test --locked`、`cargo fmt --all -- --check`、`cargo clippy --locked --all-targets` | Rust/Cargo 调度 native 检查通过 | 全部通过；Rust 测试 12/12，使用 Qt 6.11.2 预编译归档 |
| 2026-09-17 | 同一构建树 `ctest --test-dir <OUT_DIR>/native-build --output-on-failure` | native 测试通过 | 6/6 通过 |
| 2026-09-17 | `actionlint .github/workflows/ci.yml`、Python YAML/矩阵断言、`python3 /tmp/panta-check-docs.py` | workflow 与文档结构有效 | 全部通过；检查 87 个 Markdown、37 个任务 |
| 2026-09-17 | GitHub Actions run `35176149286`（`994b730`） | 三平台四项检查全绿 | macOS、Windows 的 Build/Test/Format/Clippy 全部通过；Ubuntu 在 `rcc` 资源生成阶段因 Qt 工具缺少 `libicui18n.so.73` 失败 |
| — | GitHub Actions 新 run（`gh run watch`/`gh run view`） | 三平台四项检查全绿 | 待执行 |

## 风险与回退

Visual Studio 生成器可能随 runner 镜像升级而变化；若 `windows-2022` 下缺少所需实例，应同步调整固定 runner 与 Qt ABI，而不是退回 MinGW。回退仅撤销本任务变更，保留此前已验证的 macOS/Linux Ninja 路径。

## 决策与工作记录

- 2026-09-17：通过 `gh` 复核两个失败 run，确定 Ubuntu OpenGL 依赖和 Windows 路径/ABI 两类根因，创建本任务。
- 2026-09-17（实施）：build script 改为保留普通绝对路径，构建阶段显式传递 profile 配置并探测单/多配置产物；CI 使用 Ubuntu `libgl1-mesa-dev` 与 Windows 2022/MSVC 生成器。
- 2026-09-17（第二轮 run）：OpenGL 与 compiler/path 根因已分别解除；Windows 进一步暴露多配置产物位于构建树根部，以及 GTest discovery 在构建阶段缺少 Qt DLL；Ubuntu 暴露 Qt `qmlimportscanner` 依赖 ICU 73。对应修复为补充根部产物候选、将 GTest discovery 延后到 `ctest`，并移除当前无源可扫描且结果为空的 app 级 import scan。
- 2026-09-17（第三轮准备）：Ubuntu 构建继续暴露 `qtpaths` 仅用于生成 `.qmlls.build.ini` 的 ICU 73 依赖；Linux 跳过该 IDE 辅助文件生成，保留 QML typeinfo、cachegen、资源和运行时路径。
- 2026-09-17（第三轮 run）：Ubuntu 在 `rcc` 资源生成阶段仍因 Qt 官方 RHEL9 工具缺少 `libicui18n.so.73` 失败；macOS、Windows 四项检查均已通过。确认应消费 Qt 官方仓库同版本的 `icu-linux-Rhel8.6-x86_64.7z` 预编译归档，而不是使用系统 ICU 或本地编译。

## 完成摘要

未完成。待三平台新 run 全绿并回填验证证据后，将本任务与索引标记为 done。
