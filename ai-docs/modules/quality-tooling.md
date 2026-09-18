# 跨语言质量工具链

[模块导航](README.md) · [实施任务 032](../task/032-cross-language-quality-gates.md) · [统一测试入口 011](../task/011-test-quality-entrypoints.md)

## 目标与范围

质量工具链覆盖仓库中已经存在的 Rust、C++20、QML/Qt 和 CMake；Python 与 Node 只有在对应源码真正进入仓库后才加入。工具版本、下载来源和报告格式由任务 032 固定，优先使用预编译工具，不把工具源码加入 native 构建图。

## 固定工具版本（任务 032）

| 工具 | 固定版本 | 安装/来源 | 用途 |
|---|---|---|---|
| cargo-deny | 0.20.2 | `cargo install --locked --version 0.20.2` | 依赖审计（RustSec advisory）、许可证、重复/通配依赖；配置 `deny.toml` |
| cargo-machete | 0.9.2 | `cargo install --locked --version 0.9.2` | 未使用依赖；build.rs 的 `DEP_*` 环境变量用法识别不到，经 `[package.metadata.cargo-machete]` 登记豁免（launcher/panta-ffi） |
| cargo-llvm-cov | 0.9.1 | `cargo install --locked --version 0.9.1` + rustup 组件 `llvm-tools-preview` | Rust line 覆盖率；Homebrew rust（无 rustup）机器需设 `LLVM_PROFDATA`/`LLVM_COV` 指向 CLT/Xcode 的 llvm 工具 |
| qmlformat / qmllint | 6.11.2 | Qt 预编译供给（`target/panta-deps/qt/staging/bin/`） | QML 格式门禁（CTest `Qml.FormatCheck`，qmlformat 无 --check，以 stdout diff 等价实现）与 lint |
| clang-format | 待固定 | 待供给（020 式预编译下载或 CI runner 组件） | C++ 格式；供给方式固定前不进门禁（032 增量二） |
| cmake-format | 未引入 | —— | 稳定可离线供给的版本确认前不引入（避免经 pip 引入 Python 运行时） |

已知工具限制（2026-09-18）：stable rustc 的 `-C instrument-coverage` 不产出分支覆盖数据，Rust 门禁先以 line 覆盖率执行，branch 待工具链支持后加入；launcher（启动胶水）按登记理由排除在 Rust 覆盖率统计外（进程编排 + build.rs 调度，C++ 侧覆盖率另行测量）。

质量门禁和真实图形冒烟分开。门禁必须可在无显示环境运行并能阻断错误；窗口、DPR、多显示屏和 OpenGL/Metal/Vulkan 检查作为平台场景单独记录，不能用图形冒烟代替单元覆盖率。

## 工具矩阵

| 范围 | 格式化 | 静态检查 | 测试与覆盖率 | 依赖/死代码 |
|---|---|---|---|---|
| Rust workspace | cargo fmt | cargo clippy；编译器告警 | cargo test；llvm-cov 或等价固定工具，line + branch 100% | cargo tree；cargo audit；未使用依赖工具在真实可用后接入 |
| C++20/native | clang-format | clang-tidy、编译器 W4/Wextra 基线 | CTest/GTest；llvm-cov 或平台等价物，line + branch 100% | CMake target/链接审计；未引用源文件和库由脚本核对 |
| QML/Qt | qmlformat | qmllint；模块 typeinfo 必须生成 | QtTest 加载组件和 ViewModel 行为，绑定/JavaScript 分支由可观察行为覆盖 | QML import、qmldir、资源引用审计 |
| CMake/工作流 | cmake-format（若固定版本可供给） | cmake configure、actionlint | 构建 preset 与 CTest 作为集成检查 | 目标图无环、依赖和生成目录边界检查 |

Rust/C++/QML 的测试命令必须复用生产构建图。覆盖率排除只允许生成的 moc、rcc、qmlcache、第三方源码和纯声明性布局；每一项排除写入配置和报告，不能通过大目录排除规避业务分支。

## 统一执行边界

Cargo 仍是开发者的一键入口，CMake 拥有 native 构建图。任务 011 提供统一入口后，入口按顺序执行格式、静态检查、构建、测试、覆盖率和审计；任一工具缺失、测试发现为空、报告生成失败或覆盖率低于 100% 都返回非零。入口不能通过自身再次调用自身，也不在没有 Node/Python 工程时创建 package.json 或 pyproject.toml。

CI 的三平台构建保留 macOS、Linux、Windows 差异，只上传带 commit、工具版本、平台和排除规则的报告。跨平台工具不能依赖单一开发机的系统安装；Qt/QML 工具和 native SDK 按预编译依赖规范缓存与校验。

## 覆盖率策略

100% 是纳入统计范围的自有可执行代码的 line + branch 目标。新增代码先写错误、状态、所有权和生命周期测试，再进入门禁；历史代码接入时按模块逐个补齐。QML 的声明性布局不强行映射为语句覆盖，但组件参数、信号、绑定重算、禁用/焦点和错误路径必须有行为断言。

工具无法可靠报告某类 QML 或跨平台 native 分支时，使用可观察的 C++ ViewModel、QtTest 或组件测试替代，并在报告中标注工具限制。真实窗口测试仍用于渲染和输入证据，不计入 100% 覆盖率数字。

## 分阶段落地

先固定现有 Rust/C++/QML/CMake 工具矩阵和报告目录，再接入 011 的统一命令与 CI 上传；随后增加受控失败测试，验证格式错误、lint 错误、空测试、覆盖率不足、依赖审计和未使用文件都会阻断。Python 任务 014 维持 deferred，Node 工具只有实际目录出现后单独登记。
