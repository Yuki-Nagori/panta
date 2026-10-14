# 013 — 桌面安装布局与部署冒烟

- 状态：in-progress
- 阶段：交付基础
- 依赖：[011](011-test-quality-entrypoints.md)（已完成）
- 优先级：P1
- 负责人：待分配
- 创建 / 更新：2026-09-16 / 2026-09-19

## 目标与背景

验证安装产物可独立启动，发现并修复 Qt/QML 和 native 运行时依赖缺口。

本任务先完成 macOS arm64 主平台安装部署基础设施，并将 app 与全部 macOS 原生 SDK 的最低部署版本统一为 macOS 13.0。VTK/Dawn、OCCT/Netgen 和 GoogleTest SDK 已按该基线重产并发布，manifest 摘要已更新；消费端 CI 完成后才能确认完整依赖链支持该版本。当前 app 只链接 VTK；OCCT/Netgen 由几何、网格测试目标消费，因此桌面 app 的 OCCT/Netgen 加载验收仍未满足，不能靠无用地复制动态库伪装通过。

## 必读

- [规范：qt](../standards/qt.md)
- [规范：qml](../standards/qml.md)
- [规范：vtk](../standards/vtk.md)
- [规范：occt](../standards/occt.md)
- [规范：netgen](../standards/netgen.md)
- [规范：cmake](../standards/cmake.md)
- [架构：build-and-development](../architecture/build-and-development.md)

## 范围与非目标

范围：完成下列步骤与验收所需的最小基础设施。

非目标：不上传发行版、不签名公证、不宣称完整产品可发布。

## 前置条件与待决策

开始条件：所列依赖任务完成且有验证记录；动手前核实所需工具和主平台。步骤中尚未确定的版本、接口、目录或工具须先写入下方决策记录，并同步受影响规范。依赖未完成时保持 planned；外部条件无法满足时改 blocked 并写具体原因。

## 实施步骤

1. 确定主平台安装布局，将 Qt 部署机制接入 install，并补齐非 Qt 依赖与资源。
2. 记录运行时动态库定位、插件和 QML 模块来源，移除开发机绝对路径依赖。
3. 在不暴露开发工具路径的测试环境启动安装产物，检查窗口/视口和适配器冒烟。
4. 整理第三方许可文件、版本清单、已知限制；增加 Cargo 桌面安装入口并更新 README 和构建文档。
5. 添加手动 GitHub Actions workflow，为三平台生成短期保留的可搬迁测试包。

## 预计改动

native install/deploy 规则、包资源、依赖清单与部署说明。执行前根据真实结构修订；不得顺手实现非目标功能。

## 清理与兼容例外

当前计划不引入兼容层。实施时记录实际删除的旧实现/配置/依赖与失效引用；无替换则注明无废弃项。必要例外先按 [代码生命周期规范](../standards/code-lifecycle.md) 登记 COMPAT 标记、验证与清理任务，不以旧实现充当默认回退。

## 验收标准

- [ ] 安装产物在独立位置启动，基础窗口/VTK 可用；app 实际链接的 native 动态依赖可加载。OCCT/Netgen 接入桌面 app 后补加载验收。
- [x] macOS arm64 清理开发环境变量后仍可定位资源与依赖；记录平台和验证范围。
- [x] 安装树含 SDK 许可、版本 provenance 与 Qt SBOM 清单；未验证的平台没有标记为支持。
- [x] 已同步 Cargo 安装命令、README、构建说明和当前架构事实，task-index 保持 in-progress。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-10-14 | macOS arm64；`cargo build --locked` | 通过；生成 `target/native/debug/app/panta.app`，bundle 内 executable 为 `panta`。本机构建信息：Apple clang 17、Qt 6.11.2、VTK 9.7.0。 |
| 2026-10-14 | macOS arm64；`cmake --install target/native/debug --prefix /tmp/panta-013-install` | 通过；生成独立 `.app`，部署 Qt/QML 插件、VTK 动态库、SDK 许可和 provenance、Qt SBOM。 |
| 2026-10-14 | macOS arm64；`cargo panta-package /tmp/panta-package-final` | 通过；Cargo 桌面安装入口生成可搬迁目录，Qt 部署日志未报告 unresolved dependency。 |
| 2026-10-14 | 清除开发环境变量后运行安装 app 内的 executable；CUA 绑定安装目录 `.app` 检查真实窗口 | 通过；可见应用主窗口及 viewport。AX 检查确认安装版运行。 |
| 2026-10-14 | `otool -L` 与 `otool -l` 检查安装 app executable 和打包动态库（166 个） | 通过；无仓库、用户目录或 `/tmp` 绝对依赖路径，也无非系统绝对 RPATH。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38503461890） | macOS arm64 与 Linux x86_64 安装树和归档通过；Windows x86_64 在运行库收集阶段失败，CMake 将 Windows API Set 虚拟 DLL 报为未解析依赖。修复已加入预排除规则，等待 CI 复验。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38505719386） | macOS arm64 与 Linux x86_64 安装树和归档通过；Windows x86_64 越过 API Set 扫描后，在安装 `d3dcompiler_47.dll` 时失败：`C:\Windows` 被写入 `cmake_install.cmake` 后触发非法 `\W` 转义。安装与开发部署路径均改为共享的正斜杠路径，等待 Windows CI 复验。 |
| 待平台验收 | Windows x64、Linux x86_64 | 尚未在各平台安装树和真实窗口验收，不宣称受支持。 |

## 风险与回退

开发机环境变量可能掩盖漏打包的 QML 模块与动态库；从独立安装位置、隔离开发路径后验证运行。回退仅撤销本任务自身变更，保留已有工作与此前有效产物；范围扩大时先拆分任务。

## 决策与工作记录

- 2026-09-16：仅完成任务编排，未实施。
- 2026-10-14：开始实施前盘点确认：当前 app install 规则只安装 `panta-native`；基础库及 CMake export 单独安装。Windows 的 `windeployqt` 和 SDK DLL 复制仅挂在 app 的 `POST_BUILD`，用于 `target/native/<profile>/app` 裸启，不构成安装部署。任务继续覆盖安装树独立启动、Qt/QML 与运行库定位、许可清单及平台证据；先按 macOS arm64 主平台实现并验收，其他平台只有实际完成同等验证后才记录支持。VTK 窗口的 macOS 限制按 007 的实测现状记录，不以构建成功代替窗口验收。
- 2026-10-14：所有平台的应用 executable 统一命名为 `panta`（Windows 为 `panta.exe`）；macOS bundle 为 `panta.app`。新增 `Panta.App` 入口包装 Shell 页，Qt 部署脚本据此扫描传递的 QML imports。CMake 安装阶段部署 Qt/QML、应用链接闭包内的 native 动态库，并随包保存 SDK 许可/provenance 与 Qt SPDX/CycloneDX SBOM。launcher 按平台定位 executable。独立安装位置、清洁环境启动和真实窗口验收通过；166 个 app/打包动态库的 Mach-O 依赖/RPATH 检查没有发现开发机绝对路径。统一命名变更后的重建与安装验证待复验。
- 2026-10-14：独立评审确认 Windows Dawn 通过 `LoadLibrary` 动态加载 `d3dcompiler_47.dll`，CMake 依赖扫描无法发现；新增 Windows install 规则，从 `%WINDIR%/System32` 将 DLL 放到安装 `bin`。同时将 Qt SBOM 安装移出 Bridge 条件，确保 `--no-default-features` 的 Qt Shell 变体也有清单。Qt 部署脚本延后到 native 运行库复制之后，让 macdeployqt 能修正完整 app 链接闭包的 `@rpath`。以上 Windows 行为未在本机实测，等待 Windows CI/实机验收。
- 2026-10-14：新增手动 `Package desktop builds` workflow，三平台分别构建 Release、通过 `cargo run --locked --release --package panta-launcher -- package "$PACKAGE_DIR"` 执行安装并上传 1 天有效的测试归档。workflow 配置已本地检查，三平台结果须在 GitHub Actions 手动运行后记录；Linux 测试机需满足 README 所列系统运行时要求。
- 2026-10-14：首次手动打包中 macOS arm64 与 Linux x86_64 成功；Windows x86_64 的 `file(GET_RUNTIME_DEPENDENCIES)` 将 `api-ms-win-*` API Set 虚拟 DLL 报为未解析依赖。Windows 安装规则在扫描前排除 `api-ms-*` 和 `ext-ms-*` 系统名称。独立评审确认正则只匹配这两类 API Set；修复效果仍须由 Windows Actions 复验。
- 2026-10-14：Windows package run 38505719386 确认 API Set 预排除已生效；新失败来自 `install(FILES "$ENV{windir}/...")` 将 `C:\Windows` 写入生成的 CMake 安装脚本，`\W` 触发解析错误。安装规则与开发期拷贝共用 `PANTA_D3DCOMPILER_47_DLL`，配置时先把 Windows 系统根目录转换为 CMake 正斜杠格式；等待 Windows Actions 复验。
- 2026-10-14：将 app、CXX bridge、VTK/Dawn、OCCT、Netgen、GoogleTest 的 macOS 构建最低版本统一为 13.0；Qt 6.11.2 已支持 13.0。更新三个 SDK Release 说明为中文并移除重复一级标题。VTK/Dawn、OCCT/Netgen、GoogleTest 三个平台归档已按新基线重产，Release 与 manifest SHA 已更新；消费端 CI 和 app 安装验收仍待复验。
- 2026-10-14：将维护者提供的 Welcome tab CSS 视觉临时实现为 SVG，加入 app 窗口图标并替换 Welcome 页签上的文字 P。它作为临时品牌资源单独打包，不加入操作图标 symbol 清单。
- 当前范围限制：桌面 app 实际只链接 VTK。OCCT 和 Netgen 仍由独立 smoke test 消费，未进入 app 的链接闭包；第一条验收保持未完成，需在桌面功能正式接入后补库加载验收。Windows/Linux 安装与实机验收也未完成。

## 完成摘要

部分完成：macOS arm64 安装布局、Qt/QML 与 app 当前链接依赖部署已实现并验收。OCCT/Netgen 尚无桌面 app consumer，Windows/Linux 尚未实机验收；任务保持 in-progress，后续接入这些库或补齐平台证据后再评估完成。
