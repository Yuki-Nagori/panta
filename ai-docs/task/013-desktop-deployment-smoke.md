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
- [ ] macOS 归档中的 app 在全部资源安装完成后签名完整，`codesign --verify --deep --strict` 通过。
- [x] 已同步 Cargo 安装命令、README、构建说明和当前架构事实，task-index 保持 in-progress。

- [ ] 旧实现及失效引用已清理，无未登记兼容代码；每次提交按 [提交规范](../standards/commits.md) 同步 task 与实际行为。

## 验证计划与结果

上方命令和场景均为待执行计划。只在对应入口存在后执行，记录 cwd、平台/版本、完整命令、结果和必要日志路径；手工图形操作记录步骤与观察。失败、跳过及未覆盖范围分别注明。

| 日期 | 环境 / 命令或场景 | 结果 / 证据 |
|---|---|---|
| 2026-10-14 | macOS arm64；`cargo build --locked` | 通过；生成 `target/native/debug/app/panta.app`，bundle 内 executable 为 `panta`。本机构建信息：Apple clang 17、Qt 6.11.2、VTK 9.7.0。 |
| 2026-10-14 | macOS arm64；`cmake --install target/native/debug --prefix /tmp/panta-013-install` | 通过；生成独立 `.app`，部署 Qt/QML 插件、VTK 动态库、SDK 许可和 provenance、Qt SBOM。 |
| 2026-10-14 | macOS arm64；`cargo panta-package /tmp/panta-package-final` | 通过；Cargo 桌面安装入口生成可搬迁目录，Qt 部署日志未报告 unresolved dependency。 |
| 2026-10-14 | macOS；检查用户下载的 `panta.app`，运行 `codesign --verify --deep --strict` 并从 executable 启动 | 失败；Qt 部署脚本先签名 app，随后安装的 SDK licenses、provenance 与 Qt SBOM 被报告为 sealed resource missing/invalid，直接启动退出码 137。已将 Qt 部署脚本移至所有资源安装之后，并在 package workflow 加签名完整性检查；等待 CI 复验。 |
| 2026-10-14 | 清除开发环境变量后运行安装 app 内的 executable；CUA 绑定安装目录 `.app` 检查真实窗口 | 通过；可见应用主窗口及 viewport。AX 检查确认安装版运行。 |
| 2026-10-14 | `otool -L` 与 `otool -l` 检查安装 app executable 和打包动态库（166 个） | 通过；无仓库、用户目录或 `/tmp` 绝对依赖路径，也无非系统绝对 RPATH。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38503461890） | macOS arm64 与 Linux x86_64 安装树和归档通过；Windows x86_64 在运行库收集阶段失败，CMake 将 Windows API Set 虚拟 DLL 报为未解析依赖。修复已加入预排除规则，等待 CI 复验。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38505719386） | macOS arm64 与 Linux x86_64 安装树和归档通过；Windows x86_64 越过 API Set 扫描后，在安装 `d3dcompiler_47.dll` 时失败：`C:\Windows` 被写入 `cmake_install.cmake` 后触发非法 `\W` 转义。安装与开发部署路径均改为共享的正斜杠路径，等待 Windows CI 复验。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38516920066，job 115697697724） | macOS arm64 与 Linux x86_64 安装树和归档通过；Windows x86_64 已完成 Qt 部署和 `d3dcompiler_47.dll` 安装，随后在安装 VTK 许可证目录时失败：SDK 源路径 `D:\a\...` 被写入 `cmake_install.cmake`，其中 `\a` 被当作非法转义。修复为生成 install 规则前规范化 SDK 与 Qt SBOM 源路径，待 Windows CI 复验。 |
| 2026-10-14 | macOS；`cmake -P /tmp/panta-install-path-check.cmake`，使用 Windows 风格 SDK/Qt 路径生成并解析 CMake 安装脚本 | 通过；规范化后生成脚本中的路径为正斜杠，CMake 可正常解析。 |
| 2026-10-14 | macOS；`cmake --install target/native/debug --prefix /tmp/panta-package-signature-check`，随后运行 `codesign --verify --deep --strict /tmp/panta-package-signature-check/panta.app` | 通过；重新生成的安装脚本先安装 licenses、provenance 与 SBOM，最后执行 Qt 部署；本地安装 bundle 的签名封印有效。 |
| 2026-10-14 | 仓库根目录；`cargo test --locked --workspace` | 通过；Rust workspace、native CTest（75/75）与 QML 聚合测试通过。首次尝试因本地 OCCT SDK 缓存摘要不符触发重下载，受 DNS 限制未完成；重试后验证通过。 |
| 2026-10-14 | GitHub Actions `Package desktop builds`（run 38518883590） | macOS arm64、Windows x86_64、Linux x86_64 均成功；下载 artifact 后发现平台归档又被 `upload-artifact` 包装，产生外层 ZIP。改为 `actions/upload-artifact@v7` 的单文件 `archive: false` 模式，让 artifact 直接提供平台归档，避免重复解压；待新 workflow run 验收。 |
| 2026-10-14 | Windows 11 x64（Windows 11 Pro 26300，x86_64；clang-cl/lld-link 22.1.7，Windows Kits mt.exe 10.0.26100）；本地复现 CI 归档的启动失败链 | 事件查看器 SideBySide 33：`requestedExecutionLevel` 缺 `level`（用户桌面包与本机复现一致）。逐层修复后，`panta-windows-x86_64.zip`（SHA256 `F8AF8CD0…9BD6`）解压一次后 `bin\panta.exe` 正常启动出现 "panta" 真实窗口。 |
| 2026-10-14 | Windows 11 x64；`cargo test --locked --workspace` | 部分通过：71/73 CTest 与全部 Rust 测试通过；`Build.SdkProvision`（子 CMake 找不到 CXX 编译器）与 `Qml.FormatCheck`（qmlformat 格式基线）失败。stash 本轮改动后的基线运行同样失败这两项，确认与本轮 diff 无关，属本机环境限制（CI 门禁有完整工具链环境）。 |
| 待平台验收 | Linux x86_64 | 尚未安装树与真实窗口验收，不宣称受支持。 |

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
- 2026-10-14：Windows package run 38516920066/job 115697697724 确认前次路径修复有效；新错误位于 SDK 许可证安装规则，`PANTA_SDK_PROVISION_DIR` 的 Windows 反斜杠使 `cmake_install.cmake` 把 `\a` 解析为非法转义。规范化 SDK install 源路径及 Qt SBOM 源路径，等待 Windows package 复验。
- 2026-10-14：检查用户下载的 macOS `.app`，发现 Qt 部署签名后才安装 SDK licenses、provenance 与 Qt SBOM，导致 sealed resources 失效并使启动被系统终止。将 Qt 部署脚本安排在全部 install 资源之后，并在 package workflow 中运行 `codesign --verify --deep --strict`，等待打包 CI 验证。
- 2026-10-14：独立只读评审检查完整 diff 及 native install 源路径，未发现遗漏的 Windows 反斜杠路径或文档验收问题；Windows package 复验仍待运行。
- 2026-10-14：本机 `cargo format --check`、`cargo build --locked`、`cargo test --locked --workspace` 和清单 XML 结构检查通过；聚合测试的 75 项 native/QML CTest 全部通过。仓库聚合 `cargo lint` 因沙箱不允许 Clippy 锁服务绑定 TCP 而未启动，直接运行等价的 workspace Clippy 命令通过。LLVM PR #165285 已修正 LLD 内部 UAC 清单命名空间，仓库固定 LLVM 22.1.7 包含此修复；app 改用 Qt 6.11.2 官方 `QT_WINDOWS_APP_PROJECT_IDENTIFIER` 与 `QT_WINDOWS_APP_PROJECT_EXECUTION_LEVEL` 属性生成清单。Qt 自动清单还设置 `/MANIFEST:NO`，避免链接器重复生成默认清单。最终 PE 资源及 Windows 启动仍需在 Windows 复验。
- 2026-10-14：将维护者提供的 Welcome tab CSS 视觉临时实现为 SVG，加入 app 窗口图标并替换 Welcome 页签上的文字 P。它作为临时品牌资源单独打包，不加入操作图标 symbol 清单。
- 2026-10-14（Windows 启动失败根因链，本机复现并逐层修复）：
  1. **SxS 清单**：Windows 事件与提取出的 exe 资源显示，最终嵌入清单把 `level`/`uiAccess` 变成带 `ms_asmv2:` 前缀的属性，SxS 因此报告 `level` 缺失。Qt 官方自动清单使用无前缀的 `requestedExecutionLevel` 属性，并允许通过 CMake 属性设置应用标识与执行级别；自动清单路径还禁用链接器默认清单生成。LLVM [issue #120394](https://github.com/llvm/llvm-project/issues/120394) 记录了 LLD 内部 UAC 清单与输入清单合并时的命名空间缺陷；[PR #165285](https://github.com/llvm/llvm-project/pull/165285) 将 LLD 内部 `trustInfo` 改为 asm.v3，issue 因此关闭。仓库固定的 LLVM 22.1.7 已包含此修复。当前通过 Qt 6.11.2 官方 `QT_WINDOWS_APP_PROJECT_IDENTIFIER` 与 `QT_WINDOWS_APP_PROJECT_EXECUTION_LEVEL` 属性生成清单；最终 exe 资源仍需在 Windows 验收中检查。
  2. **系统 DLL 阴影**：依赖解析路径未规范化（`C:\WINDOWS\system32/user32.dll`，CMP0207），而 CMake REGEX 区分大小写，写死的 `Windows` 在 `%windir%=C:\WINDOWS` 的机器上失配，239 个系统 DLL（含 d3d12/dxgi/urlmon）被复制进 bin 并在运行期阴影系统副本，导致 D3D12 适配器枚举失败后 SEH 崩溃。POST_EXCLUDE 改为由 `%windir%` 生成大小写精确、分隔符无关的前缀正则（单遍 `string(REGEX REPLACE)`）。
  3. **延迟导入**：CMake 4.x 原生 PE 解析器以**小写名**匹配 PRE_EXCLUDE（错误输出仍为原名）；Windows 11 的 shell32/urlmon/DMCmnUtils/Print.PrintSupport.Source 经延迟导入引用的 WDAC/证明可选组件（hvsifiletrust/wpaxholder/pdmutilities/azureattest*）在本机无文件，resolve 失败直接报错。按名称小写加入 PRE_EXCLUDE。
  4. **Dawn 动态加载缺口**：`dxcompiler.dll`/`dxil.dll` 由 Dawn 运行期加载，依赖扫描不可见；从 Windows SDK redistributable（`Windows Kits/10/Redist/D3D/x64`）显式安装。当前桌面 app 尚未使用 OCCT/Netgen，安装包不复制未使用的 SDK DLL；开发裸启部署仍按任务 083 覆盖未来 consumer。
  验证：修复后包内无系统 DLL 阴影（保留 redist/d3dcompiler/dxc），`bin` 134 文件；解压一次启动真实窗口。诊断工具为临时脚本，未入库。
- 2026-10-14：独立只读评审确认 DXC/DXIL 开发部署路径与 Windows 依赖排除逻辑，并指出 OCCT/Netgen 的运行期加载说明及 Windows 完成摘要不准确。随后将 DXC/DXIL 查找集中为安装与开发部署共用函数、安装包移除未使用的 OCCT/Netgen DLL，修正文档。整理后的 Windows 构建与包尚待复验。
- 当前范围限制：桌面 app 实际链接 VTK；OCCT/Netgen 仍由独立 smoke test 消费，未进入 app 的链接闭包。Windows 11 x64 单机安装包已启动并显示真实窗口；本次 manifest/部署整理后的 Windows 包、Linux 安装与实机验收仍待验证。桌面 app 的 OCCT/Netgen 加载验收需待对应 consumer 接入后完成。

## 完成摘要

部分完成：macOS arm64 安装布局、Qt/QML 与 app 当前链接依赖部署已实现并验收；Windows 11 x64 的单机包启动也已通过。OCCT/Netgen 尚无桌面 app consumer，Linux 尚未实机验收，本次 Windows 调整仍待新包复验；任务保持 in-progress。
