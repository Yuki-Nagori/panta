# 083 — Windows 开发裸启应用的 Qt 运行库部署

- 状态：done
- 阶段：应用平台扩展
- 依赖：[005 Qt/QML 桌面壳](005-qt-qml-shell.md)、[042 统一 LLVM 工具链](042-unified-llvm-toolchain.md)
- 优先级：P2
- 负责人：Yuki
- 创建 / 更新：2026-09-26 / 2026-09-26

## 目标与背景

AGENTS 真实窗口验收要求直接启动 `target/native/debug/app/panta-native`；Windows 上 exe 链接 staging 的 Qt 导入库，但 DLL 搜索路径不含 staging/bin，裸启报"找不到 Qt6Qml.dll"（Qt6Core/Gui/QuickControls2 同理）。测试进程由 runner 注入 PATH 不受影响；手工拷 DLL 或永久改系统 PATH 都不是可维护方案。

完成后的可观察行为：Windows 上 `cargo build --locked` 后直接双击/命令行启动 panta-native.exe 即进入应用窗口，无需设置任何环境变量。

## 必读

- [验证与评审](../standards/validation-and-review.md)
- [注释规范](../standards/comments.md)
- [提交规范](../standards/commits.md)

## 范围与非目标

- `native/app/CMakeLists.txt`：WIN32 下为 `panta_native_app` 添加 POST_BUILD `windeployqt`（托管 Qt staging 自带工具），部署 DLL、平台/QML 插件与 Qt QML 模块到 exe 目录；`--qmldir` 指向仓库 `qml/` 供推导 Qt 模块集。
- 仅覆盖开发验收场景；不处理安装包/发布分发（后续任务）、不部署 MSVC CRT（开发机具备 Build Tools）、不改 macOS/Linux（分别有 .app 布局与 rpath，未出现此问题）。

## 前置条件与待决策

- 托管 Qt staging 已就绪（任务 041/005 供给路径）。
- windeployqt 每次重链接都会执行（秒级）；Qt staging 升级后依赖 CMake 重新 find_program 指向新路径。

## 实施步骤

1. 登记 task 并同步索引。
2. app CMakeLists 增加 WIN32 POST_BUILD 部署命令（find_program 约束在 staging/bin，与 qmlformat 同型）。
3. 重新构建触发重链接，核对 exe 目录产物（Qt6Core/Qml/Gui/Quick/QuickControls2/QuickDialogs2 DLL、platforms/qwindows.dll、qml/ 模块树）。
4. 不设任何环境变量裸启 exe，确认主窗口创建后关闭进程。

## 预计改动

- `native/app/CMakeLists.txt`（现存）
- `native/cmake/windows-app-runtime-deployment.cmake`（新建，部署逻辑模块）
- `ai-docs/task-index.md` 与本任务

## 清理与兼容例外

无废弃项；无兼容例外。

## 验收标准

- [x] 裸启（无 PATH/qt.conf 等任何环境设置）panta-native.exe 能创建应用主窗口。
- [x] `cargo test --locked --workspace` 不受部署步骤影响。
- [x] 任务与索引状态一致，验证证据已记录。

## 验证计划与结果

| 日期 | 环境 / 命令或场景 | 预期 | 实际结果 / 证据 |
|---|---|---|---|
| 2026-09-26 | Windows：`cargo build --locked --workspace` | 链接后自动执行三段部署（windeployqt / VTK bin 拷贝 / d3dcompiler 拷贝） | 通过；exe 旁出现 34 个 Qt6*.dll、46 个 VTK DLL、platforms/、qml/ 树、d3dcompiler_47.dll |
| 2026-09-26 | Windows：不设任何环境变量直接启动 `target/native/debug/panta-native.exe` | 进程存活并创建应用主窗口 | 通过；PowerShell Get-Process 报告 MainWindowTitle=`panta`、窗口句柄非零，随后正常终止进程。修复前同场景报"找不到 Qt6Qml.dll"系统对话框，部署 Qt 后再报 0xC0000135（缺 VTK DLL），补 d3dcompiler_47.dll 前还暴露 Dawn EnsureFXC Error 87 崩溃 |
| 2026-09-26 | GitHub Actions：HEAD `eb6cb88` [CI run 36177262410](https://github.com/Yuki-Nagori/panta/actions/runs/36177262410) | `cargo test --locked --workspace` 聚合门禁不受部署步骤影响 | 通过；三平台全绿（7m29s），含部署逻辑提交 |
| 2026-09-26 | Windows：验收复验——核对 exe 目录产物后不设任何环境变量裸启 | 产物齐备，主窗口创建后正常终止 | 通过；Qt6 DLL×33（含 Qt6Test.dll）、VTK DLL×46、platforms/qwindows.dll、qml/ 树、d3dcompiler_47.dll 在位；MainWindowTitle=`panta`、句柄 67846 |

## 风险与回退

windeployqt 部署不全导致运行期 QML 模块缺失时，回退为文档化 PATH 启动方式并在本任务记录缺口；POST_BUILD 失败会阻断构建，可临时注释该命令恢复。

## 决策与工作记录

- 2026-09-26：创建任务。维护者在 Windows 验收裸启时触发系统"找不到 Qt6Qml.dll"对话框；确认仓库无任何运行库部署逻辑，选择 windeployqt POST_BUILD 方案（对比：手工枚举 DLL/插件脆弱，全局 PATH 指向 staging 脆弱）。
- 2026-09-26：首轮部署后仍 0xC0000135——dumpbin 显示 exe 还依赖 14 个 VTK 共享库，增加 VTK staging bin 整目录拷贝；再裸启暴露 Dawn `EnsureFXC` 加载 d3dcompiler_47.dll 报 Error 87 后 SEH 崩溃（crash handler 落盘），补拷 System32 副本后主窗口创建成功。AGENTS 的 exe 路径以实际产出 `target/native/debug/panta-native.exe` 为准（文档写的 app/ 子目录是 .lib 位置）。
- 2026-09-26：部署逻辑上移为 `native/cmake/windows-app-runtime-deployment.cmake` 的 `panta_deploy_windows_app_runtime`，DLL 目录按已供给 SDK 的 staging 自动枚举（递归 *.dll 去重），覆盖 occt `win64/vc14/bin`（48 个）、vtk `bin`（46 个）、netgen `bin`（3 个）；当前 exe 导入表尚无 OCCT/Netgen，但 063 STL 导入链路随时可能拉入，预覆盖避免裸启再次断链。维护者要求文件命名携带 windows 语义。
- 2026-09-26：补拷 `Qt6Test.dll`——手动基准/测试 exe 与 app 同目录裸启时依赖它，windeployqt 按 app 依赖推导不会包含（四个基准 exe 裸启 0xC0000135 的根因）；带 staging 存在性守卫，非测试供给树跳过。
- 2026-09-26：验收收口。HEAD CI run 36177262410 三平台全绿覆盖测试门禁，复验裸启创建主窗口后关闭；AGENTS 真实窗口验收启动路径按实际产出修正为 `target/native/debug/panta-native(.exe)`（`app/` 子目录实为 .lib 位置）。

## 完成摘要

已完成（2026-09-26 验收收口）。裸启复验通过（MainWindowTitle=`panta`、句柄 67846），HEAD `eb6cb88` CI run 36177262410 三平台全绿覆盖 `cargo test --locked --workspace` 门禁；AGENTS 启动路径已同步实际产出。
